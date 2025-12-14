# Architecture Overview

This document explains the internal architecture of Hyperparameter, including how the Rust backend and Python frontend work together.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Python User Code                         │
│  @auto_param, param_scope, loader.load(), etc.              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Python API Layer                           │
│  hyperparameter/api.py, hyperparameter/cli.py               │
│  - Decorators (@auto_param)                                 │
│  - Context managers (param_scope)                           │
│  - CLI argument parsing                                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Storage Abstraction                        │
│  hyperparameter/storage.py                                  │
│  - TLSKVStorage (Thread-Local Storage)                      │
│  - Automatic backend selection                              │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐    ┌─────────────────────────────┐
│   Rust Backend          │    │   Python Fallback Backend    │
│   (librbackend.so)      │    │   (Pure Python dict)         │
│   - xxhash for keys     │    │   - Used when Rust unavailable│
│   - Thread-local storage│    │   - Same API contract         │
│   - Lock-free reads     │    │                               │
└─────────────────────────┘    └─────────────────────────────┘
```

## Component Details

### 1. Python API Layer (`hyperparameter/api.py`)

This is what users interact with directly.

**Key Classes:**

- **`param_scope`**: A context manager that creates a new parameter scope.
  ```python
  with param_scope(foo=1, bar=2) as ps:
      # ps.foo() returns 1
      # Nested scopes inherit from parent
  ```

- **`_ParamAccessor`**: Handles the `param_scope.x.y.z | default` syntax.
  ```python
  # This chain: param_scope.model.layers.size | 10
  # Creates: _ParamAccessor(root, "model.layers.size")
  # The `|` operator calls get_or_else(10)
  ```

- **`auto_param` decorator**: Inspects function signature and injects values.
  ```python
  @auto_param("model")
  def foo(hidden_size=256):  # Looks up "model.hidden_size"
      pass
  ```

### 2. Storage Layer (`hyperparameter/storage.py`)

The storage layer abstracts the underlying key-value store.

**Key Features:**

- **Thread-Local Storage (TLS)**: Each thread has its own parameter stack.
- **Scoped Updates**: Changes are local to the current scope and roll back on exit.
- **Backend Selection**: Automatically uses Rust backend if available.

```python
class TLSKVStorage:
    """Thread-local key-value storage with scope stack."""
    
    def enter(self):
        """Push a new scope onto the stack."""
        
    def exit(self):
        """Pop the current scope, rolling back changes."""
        
    def get(self, key: str) -> Any:
        """Look up key in current scope, then parent scopes."""
        
    def put(self, key: str, value: Any):
        """Set key in current scope only."""
```

### 3. Rust Backend (`src/core/`, `src/py/`)

The Rust backend provides high-performance parameter access.

**Why Rust?**

1. **Compile-time key hashing**: Keys like `"model.layers.size"` are hashed at compile time using `xxhash`, eliminating runtime string hashing overhead.

2. **Lock-free reads**: Thread-local storage means no mutex contention on reads.

3. **Zero-copy string handling**: Rust's string handling avoids Python's string interning overhead.

**Key Rust Components:**

```rust
// src/core/src/storage.rs
pub struct ThreadLocalStorage {
    stack: Vec<HashMap<u64, Value>>,  // Scope stack
}

// src/core/src/xxh.rs
pub const fn xxhash(s: &str) -> u64 {
    // Compile-time xxhash64
}

// src/core/src/api.rs
pub fn get_param<T>(key_hash: u64, default: T) -> T {
    // Fast lookup by pre-computed hash
}
```

**Python Binding (`src/py/`):**

Uses PyO3 to expose Rust functions to Python:

```rust
#[pyfunction]
fn get_entry(key_hash: u64) -> PyResult<PyObject> {
    // Called from Python with pre-computed hash
}
```

### 4. Config Loader (`hyperparameter/loader.py`)

The loader handles configuration file parsing and processing.

**Pipeline:**

```
File(s) → Parse → Merge → Interpolate → Validate → Dict/Object
```

1. **Parse**: Support for TOML, JSON, YAML
2. **Merge**: Deep merge multiple configs (later overrides earlier)
3. **Interpolate**: Resolve `${variable}` references
4. **Validate**: Optional schema validation against class type hints

```python
def load(path, schema=None):
    config = _load_and_merge(path)
    config = _resolve_interpolations(config)
    if schema:
        return validate(config, schema)
    return config
```

## Data Flow Example

Let's trace what happens when you run:

```python
from hyperparameter import auto_param, param_scope

@auto_param("model")
def train(lr=0.001):
    print(lr)

with param_scope(**{"model.lr": 0.01}):
    train()
```

**Step-by-step:**

1. **`param_scope(**{"model.lr": 0.01})`**:
   - Creates new `TLSKVStorage` scope
   - Computes hash: `xxhash("model.lr")` → `0x1234...`
   - Stores: `{0x1234...: 0.01}` in current thread's scope stack

2. **`train()` called**:
   - `@auto_param` wrapper runs
   - For each kwarg with default (`lr=0.001`):
     - Computes hash: `xxhash("model.lr")`
     - Calls `storage.get_entry(0x1234...)`
     - Rust backend returns `0.01`
   - Calls `train(lr=0.01)`

3. **Scope exit**:
   - `param_scope.__exit__()` called
   - Pops scope from stack
   - `model.lr` no longer accessible

## Performance Characteristics

### Why Hyperparameter is Fast

| Operation | Hydra/OmegaConf | Hyperparameter |
| :--- | :--- | :--- |
| Key lookup | String hash at runtime | Pre-computed xxhash |
| Type checking | On every access | Optional, at load time |
| Thread safety | Global lock | Thread-local (no lock) |
| Memory | Python dicts + wrappers | Rust HashMap |

### When to Use Which Access Pattern

| Pattern | Speed | Use Case |
| :--- | :--- | :--- |
| `@auto_param` injection | 🚀🚀🚀 Fastest | Hot loops, performance-critical |
| `with param_scope() as ps: ps.x` | 🚀🚀 Fast | Most code |
| `param_scope.x` (global) | 🚀 Moderate | Convenience, one-off access |

## Thread Safety Model

```
Thread 1                    Thread 2
────────                    ────────
param_scope(a=1)           
│                          param_scope(a=2)
│   a = 1                  │   a = 2
│                          │
└── exit                   └── exit
    a = undefined              a = undefined
```

Each thread has **independent scope stacks**. Changes in one thread never affect another.

**`frozen()` for cross-thread defaults:**

```python
with param_scope(a=1):
    param_scope.frozen()  # Snapshot current scope as global default
    
# New threads will see a=1 as their initial state
```

## Extending Hyperparameter

### Custom Storage Backend

```python
from hyperparameter.storage import TLSKVStorage

class RedisBackedStorage(TLSKVStorage):
    """Example: Redis-backed storage for distributed systems."""
    
    def get(self, key):
        # Try local first
        value = super().get(key)
        if value is None:
            # Fall back to Redis
            value = self.redis.get(key)
        return value
```

### Custom Type Coercion

```python
from hyperparameter.loader import _coerce_type

def _coerce_type(value, target_type):
    # Add custom type handling
    if target_type is MyCustomType:
        return MyCustomType.from_string(value)
    # ... existing logic
```

## File Structure

```
hyperparameter/
├── __init__.py          # Public API exports
├── api.py               # Core Python API (param_scope, auto_param)
├── cli.py               # CLI support (launch, run_cli)
├── loader.py            # Config loading, interpolation, validation
├── storage.py           # Storage abstraction, TLS
└── tune.py              # Hyperparameter tuning utilities

src/
├── core/                # Rust core library
│   └── src/
│       ├── api.rs       # Public Rust API
│       ├── storage.rs   # Thread-local storage
│       ├── value.rs     # Value type handling
│       └── xxh.rs       # Compile-time xxhash
├── macros/              # Rust procedural macros
└── py/                  # PyO3 Python bindings
```

