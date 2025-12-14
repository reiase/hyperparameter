# API Reference

This document provides a complete reference for the Hyperparameter Python API.

---

## scope

`scope` is the core class for managing hyperparameters with thread-safe scoping.

### Import

```python
import hyperparameter as hp
```

### Creating scope

```python
# Empty scope
ps = hp.scope()

# From keyword arguments
ps = hp.scope(lr=0.001, batch_size=32)

# From string arguments (key=value format)
ps = hp.scope("lr=0.001", "batch_size=32")

# From dictionary
ps = hp.scope(**{"train.lr": 0.001, "train.batch_size": 32})

# Empty scope (clears inherited values)
ps = hp.scope.empty()
ps = hp.scope.empty(lr=0.001)
```

### Reading Parameters

```python
# Using | operator (returns default if missing)
lr = hp.scope.train.lr | 0.001

# Using function call (returns default if missing)
lr = hp.scope.train.lr(0.001)

# Without default (raises KeyError if missing)
lr = hp.scope.train.lr()

# Dynamic key access
key = "train.lr"
lr = scope[key] | 0.001
```

### Writing Parameters

```python
with hp.scope() as ps:
    # Attribute assignment
    hp.scope.train.lr = 0.001
    
    # Via instance
    ps.train.batch_size = 32
```

### Context Manager (with statement)

```python
# Basic usage
with hp.scope(**{"lr": 0.001}):
    print(hp.scope.lr())  # 0.001

# Nested scopes
with hp.scope(**{"a": 1}):
    print(hp.scope.a())  # 1
    with hp.scope(**{"a": 2}):
        print(hp.scope.a())  # 2
    print(hp.scope.a())  # 1 (auto-rollback)
```

### Static Methods

#### `hp.scope.empty(*args, **kwargs)`

Creates a new empty scope, clearing any inherited values.

```python
with hp.scope(**{"inherited": 1}):
    with hp.scope.empty(**{"fresh": 2}) as ps:
        print(ps.inherited("missing"))  # "missing"
        print(ps.fresh())  # 2
```

#### `hp.scope.current()`

Returns the current active hp.scope.

```python
with hp.scope(**{"key": "value"}):
    ps = hp.scope.current()
    print(ps.key())  # "value"
```

#### `hp.scope.frozen()`

Snapshots the current scope as the global baseline for new threads.

```python
with hp.scope(**{"global_config": 42}):
    hp.scope.frozen()
    # New threads will inherit global_config=42
```

#### `hp.scope.init(params=None)`

Initializes scope for a new thread.

```python
def thread_target():
    hp.scope.init({"thread_param": 1})
    # ...
```

### Instance Methods

#### `ps.keys()`

Returns an iterable of all parameter keys.

```python
with hp.scope(**{"a": 1, "b.c": 2}) as ps:
    print(list(ps.keys()))  # ['a', 'b.c']
```

#### `ps.storage()`

Returns the underlying storage object.

#### `ps.update(dict)`

Updates the scope with values from a dictionary.

#### `ps.clear()`

Clears all parameters in the current hp.scope.

---

## @hp.param

Decorator that automatically binds function parameters to hyperparameters.

### Import

```python
import hyperparameter as hp
```

### Basic Usage

```python
@hp.param("train")
def train(lr=0.001, batch_size=32, epochs=10):
    print(f"lr={lr}, batch_size={batch_size}")

# Uses function defaults
train()  # lr=0.001, batch_size=32

# Override via scope
with hp.scope(**{"train.lr": 0.01}):
    train()  # lr=0.01, batch_size=32

# Direct arguments have highest priority
train(lr=0.1)  # lr=0.1, batch_size=32
```

### With Custom Namespace

```python
@hp.param("myapp.config.train")
def train(lr=0.001):
    print(f"lr={lr}")

with hp.scope(**{"myapp.config.train.lr": 0.01}):
    train()  # lr=0.01
```

### Without Namespace (uses function name)

```python
@hp.param
def my_function(x=1):
    return x

with hp.scope(**{"my_function.x": 2}):
    my_function()  # returns 2
```

### Class Decorator

```python
@hp.param("Model")
class Model:
    def __init__(self, hidden_size=256, dropout=0.1):
        self.hidden_size = hidden_size
        self.dropout = dropout

with hp.scope(**{"Model.hidden_size": 512}):
    model = Model()  # hidden_size=512, dropout=0.1
```

### Parameter Resolution Priority

1. **Direct arguments** (highest priority)
2. **scope overrides**
3. **Function signature defaults** (lowest priority)

---

## launch

Entry point for CLI applications with automatic argument parsing.

### Import

```python
import hyperparameter as hp
```

### Single Function

```python
@hp.param("app")
def main(input_file, output_file="out.txt", verbose=False):
    """Process input file.
    
    Args:
        input_file: Path to input file
        output_file: Path to output file
        verbose: Enable verbose output
    """
    pass

if __name__ == "__main__":
    hp.launch(main)
```

Run:
```bash
python app.py input.txt --output_file result.txt --verbose
python app.py input.txt -D app.verbose=true
```

### Multiple Functions (Subcommands)

```python
@hp.param("train")
def train(epochs=10, lr=0.001):
    """Train the model."""
    pass

@hp.param("eval")
def evaluate(checkpoint="model.pt"):
    """Evaluate the model."""
    pass

if __name__ == "__main__":
    hp.launch()  # Discovers all @hp.param functions
```

Run:
```bash
python app.py train --epochs 20
python app.py eval --checkpoint best.pt
```

### CLI Options

| Option | Description |
|--------|-------------|
| `-D, --define KEY=VALUE` | Override hyperparameters |
| `-lps, --list-param-scope` | List all registered parameters |
| `-ep, --explain-param KEY` | Show details for a parameter |
| `-h, --help` | Show help message |

---

## launch

Alternative to `launch` with slightly different behavior.

```python
import hyperparameter as hp

if __name__ == "__main__":
    hp.launch()
```

---

## Type Conversion

When reading parameters with a default value, automatic type conversion is applied based on the default's type.

### Boolean Conversion

```python
with hp.scope(**{"flag": "true"}):
    hp.scope.flag(False)  # True

# Recognized true values: "true", "True", "TRUE", "t", "T", "yes", "YES", "y", "Y", "1", "on", "ON"
# Recognized false values: "false", "False", "FALSE", "f", "F", "no", "NO", "n", "N", "0", "off", "OFF"
```

### Integer Conversion

```python
with hp.scope(**{"count": "42"}):
    hp.scope.count(0)  # 42 (int)

with hp.scope(**{"value": "3.14"}):
    hp.scope.value(0)  # 3.14 (float, precision preserved)
```

### Float Conversion

```python
with hp.scope(**{"rate": "0.001"}):
    hp.scope.rate(0.0)  # 0.001
```

### String Conversion

```python
with hp.scope(**{"count": 42}):
    hp.scope.count("0")  # "42" (string)
```

---

## Thread Safety

### Thread Isolation

Each thread has its own parameter hp.scope. Changes in one thread do not affect others.

```python
import threading

def worker():
    with hp.scope(**{"worker_id": threading.current_thread().name}):
        print(hp.scope.worker_id())

threads = [threading.Thread(target=worker) for _ in range(3)]
for t in threads:
    t.start()
for t in threads:
    t.join()
```

### Propagating to New Threads

Use `frozen()` to propagate values to new threads:

```python
with hp.scope(**{"global_config": 42}):
    hp.scope.frozen()

def worker():
    print(hp.scope.global_config())  # 42

t = threading.Thread(target=worker)
t.start()
t.join()
```

---

## Error Handling

### KeyError

Raised when accessing a required parameter that is missing:

```python
with hp.scope():
    hp.scope.missing()  # Raises KeyError
```

### Safe Access

Always provide a default to avoid KeyError:

```python
with hp.scope():
    hp.scope.missing | "default"  # Returns "default"
    hp.scope.missing("default")   # Returns "default"
```

---

## Advanced Features

### Nested Dictionary Flattening

Nested dictionaries are automatically flattened:

```python
with hp.scope(**{"model": {"hidden": 256, "layers": 4}}):
    print(scope["model.hidden"]())  # 256
    print(hp.scope.model.layers())     # 4
```

### Dynamic Key Construction

```python
for task in ["train", "eval"]:
    key = f"config.{task}.batch_size"
    value = getattr(hp.scope.config, task).batch_size | 32
```

### Accessing Underlying Storage

```python
with hp.scope(**{"a": 1, "b": 2}) as ps:
    storage = ps.storage()
    print(storage.storage())  # {'a': 1, 'b': 2}
```

---

## Command Line Tool: hp

Hyperparameter provides a CLI tool `hp` for analyzing hyperparameters in Python packages.

### Installation

After installing hyperparameter, the `hp` command is available:

```bash
pip install hyperparameter
hp --help
```

### Commands

#### hp list / hp ls

List hyperparameters:

```bash
# List all packages using hyperparameter
hp ls
hp list

# List hyperparameters in a package
hp ls mypackage

# Tree view
hp ls mypackage --tree
hp ls mypackage -t

# Scope options
hp ls mypackage --self       # Only self (default)
hp ls mypackage --all        # Include dependencies
hp ls mypackage --deps       # Only dependencies

# Output formats
hp ls mypackage -f text      # Default text format
hp ls mypackage -f markdown  # Markdown format
hp ls mypackage -f json      # JSON format

# Save to file
hp ls mypackage -o report.md -f markdown
```

#### Package Discovery

When running `hp ls` without arguments, it scans all installed packages:

```
Packages using hyperparameter (3):
============================================================
Package                        Version      Params   Funcs
------------------------------------------------------------
myapp                          1.0.0        15       5
ml-toolkit                     0.2.1        8        3
config-manager                 2.1.0        4        2
------------------------------------------------------------

Use 'hp ls <package>' to see hyperparameters in a package.
```

#### hp describe / hp desc

View hyperparameter details:

```bash
# Exact match
hp desc train.lr mypackage

# Fuzzy search
hp desc lr mypackage

# Default to current directory
hp desc train.lr
```

### Example Output

#### List (Tree View)

```
Hyperparameters in myapp:
----------------------------------------
📁 train
  📄 lr = 0.001
  📄 batch_size = 32
  📄 epochs = 10
📁 model
  📄 hidden_size = 256
  📄 dropout = 0.1

Total: 5 hyperparameters
```

#### Describe

```
============================================================
Hyperparameter: train.lr
============================================================

  Default: 0.001
  Type: float
  Namespace: train
  Function: train

  Source: myapp
  Location: train.py:15

  Description: Training function with configurable learning rate.

  Usage:
    # Access via scope
    value = hp.scope.train.lr | <default>
    
    # Set via command line
    --train.lr=<value>
```
