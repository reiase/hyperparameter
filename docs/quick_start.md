# Quick Start

Hyperparameter is a configuration management library providing thread-safe scoping, automatic parameter binding, and CLI integration.

```bash
pip install hyperparameter
```

---

## 1. Basic Usage

### 1.1 Reading Parameters with Defaults

```python
from hyperparameter import param_scope

# Use | operator to provide default values
lr = param_scope.train.lr | 0.001
batch_size = param_scope.train.batch_size | 32

# Use function call syntax (equivalent to |)
use_cache = param_scope.model.cache(True)

# Call without arguments: raises KeyError if missing
required_value = param_scope.model.required_key()  # KeyError if missing
```

`param_scope.key(default)` is equivalent to `param_scope.key | default`. Calling `param_scope.key()` without arguments treats the parameter as required and raises `KeyError` if missing.

### 1.2 Scoping and Auto-Rollback

```python
from hyperparameter import param_scope

print(param_scope.model.dropout | 0.1)  # 0.1

with param_scope(**{"model.dropout": 0.3}):
    print(param_scope.model.dropout | 0.1)  # 0.3

print(param_scope.model.dropout | 0.1)  # 0.1, auto-rollback on scope exit
```

All parameter modifications within a `with` block are automatically reverted when the scope exits.

---

## 2. @auto_param Decorator

### 2.1 Automatic Parameter Binding

```python
from hyperparameter import auto_param, param_scope

@auto_param("train")
def train(lr=1e-3, batch_size=32, epochs=10):
    print(f"lr={lr}, batch_size={batch_size}, epochs={epochs}")

train()  # Uses function signature defaults

with param_scope(**{"train.lr": 5e-4, "train.batch_size": 64}):
    train()  # lr=0.0005, batch_size=64, epochs=10

train(lr=1e-2)  # Direct arguments take highest priority
```

Parameter resolution priority: direct arguments > param_scope overrides > function signature defaults.

### 2.2 CLI Override

```python
# train.py
from hyperparameter import auto_param, param_scope

@auto_param("train")
def train(lr=1e-3, batch_size=32, warmup_steps=500):
    print(f"lr={lr}, batch_size={batch_size}, warmup={warmup_steps}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with param_scope(*args.define):
        train()
```

```bash
python train.py -D train.lr=5e-4 -D train.batch_size=64
```

Override parameters at runtime with `-D key=value` without modifying code.

---

## 3. Nested Scopes

### 3.1 Multi-Model Comparison

```python
from hyperparameter import param_scope, auto_param

@auto_param("modelA")
def run_model_a(dropout=0.1, hidden=128):
    print(f"ModelA: dropout={dropout}, hidden={hidden}")

@auto_param("modelB")
def run_model_b(dropout=0.2, hidden=256):
    print(f"ModelB: dropout={dropout}, hidden={hidden}")

base = {"data.path": "/data/mnist"}
variants = [
    {"modelA.dropout": 0.3},
    {"modelB.hidden": 512, "modelB.dropout": 0.15},
]

with param_scope(**base):
    for cfg in variants:
        with param_scope(**cfg):
            run_model_a()
            run_model_b()
```

Outer scopes set shared configuration; inner scopes override specific parameters. Scopes are isolated from each other.

### 3.2 Dynamic Key Access

```python
from hyperparameter import param_scope

def train_task(task_name):
    lr = param_scope[f"task.{task_name}.lr"] | 1e-3
    wd = param_scope[f"task.{task_name}.weight_decay"] | 0.01
    print(f"{task_name}: lr={lr}, weight_decay={wd}")

with param_scope(**{
    "task.cls.lr": 1e-3,
    "task.cls.weight_decay": 0.01,
    "task.seg.lr": 5e-4,
    "task.seg.weight_decay": 0.001,
}):
    train_task("cls")
    train_task("seg")
```

Use `param_scope[key]` syntax for dynamically constructed keys.

---

## 4. Thread Safety

### 4.1 Request-Level Isolation

```python
from hyperparameter import param_scope

def rerank(items):
    use_new = param_scope.rerank.use_new(False)
    threshold = param_scope.rerank.threshold | 0.8
    if use_new:
        return [x for x in items if x.score >= threshold]
    return items

def handle_request(request):
    with param_scope(**request.overrides):
        return rerank(request.items)
```

Each request executes in an isolated scope. Configuration changes do not affect other concurrent requests.

### 4.2 Multi-threaded Data Processing

```python
import concurrent.futures
from hyperparameter import param_scope

def preprocess(shard, cfg):
    with param_scope(**cfg):
        clean = param_scope.pre.clean_noise(False)
        norm = param_scope.pre.norm | "zscore"
        # Processing logic
        return processed_shard

cfg = {"pre.clean_noise": True, "pre.norm": "minmax"}
shards = load_shards()

with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(lambda s: preprocess(s, cfg), shards))
```

Thread safety guarantees:
- Configuration dicts can be safely passed to multiple threads
- Each thread's `param_scope` modifications are isolated
- Automatic cleanup on scope exit

---

## 5. Common Use Cases

### 5.1 LLM Inference Configuration

```python
from hyperparameter import param_scope

def generate(prompt):
    max_tokens = param_scope.llm.max_tokens | 256
    temperature = param_scope.llm.temperature | 0.7
    return llm_call(prompt, max_tokens=max_tokens, temperature=temperature)

# Default configuration
generate("hello")

# Temporary override
with param_scope(**{"llm.max_tokens": 64, "llm.temperature": 0.2}):
    generate("short answer")
```

### 5.2 A/B Testing

```python
from hyperparameter import param_scope

def get_experiment_config(user_id):
    if hash(user_id) % 100 < 10:  # 10% traffic
        return {"search.algo": "v2", "search.boost": 1.5}
    return {}

def search(query):
    algo = param_scope.search.algo | "v1"
    boost = param_scope.search.boost | 1.0
    # Search logic

def handle_request(user_id, query):
    with param_scope(**get_experiment_config(user_id)):
        return search(query)
```

### 5.3 ETL Job Configuration

```python
from hyperparameter import param_scope

def run_job(name, overrides=None):
    with param_scope(**(overrides or {})):
        batch = param_scope.etl.batch_size | 500
        retry = param_scope.etl.retry | 3
        timeout = param_scope.etl.timeout | 30
        # ETL logic

run_job("daily")
run_job("full_rebuild", {"etl.batch_size": 5000, "etl.timeout": 300})
```

### 5.4 Early Stopping

```python
from hyperparameter import param_scope

def check_early_stop(metric, best, wait):
    patience = param_scope.scheduler.patience | 5
    delta = param_scope.scheduler.min_delta | 0.001
    
    if metric > best + delta:
        return False, metric, 0
    wait += 1
    return wait >= patience, best, wait
```

---

## 6. Rust Interface

### 6.1 Basic Usage

```rust
use hyperparameter::*;

fn train() {
    with_params! {
        get lr = train.lr or 0.001f64;
        get batch_size = train.batch_size or 32i64;
        println!("lr={}, batch_size={}", lr, batch_size);
    }
}

fn main() {
    train();  // Uses default values

    with_params! {
        set train.lr = 0.0005f64;
        set train.batch_size = 64i64;
        train();
    };

    train();  // Rollback to defaults
}
```

### 6.2 Thread Isolation with frozen()

```rust
use hyperparameter::*;
use std::thread;

fn worker(id: i64) {
    with_params! {
        set worker.id = id;
        for i in 0..5 {
            with_params! {
                set worker.iter = i;
                get wid = worker.id or -1i64;
                get witer = worker.iter or -1i64;
                println!("Worker {} iter {}", wid, witer);
            };
        }
    };
}

fn main() {
    with_params! {
        set global.seed = 42i64;
        frozen();  // Snapshot current config as initial state for new threads
    };

    let handles: Vec<_> = (0..4)
        .map(|id| thread::spawn(move || worker(id)))
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
```

`frozen()` snapshots the current configuration as the global baseline. New threads start from this snapshot, with subsequent modifications isolated between threads.

---

## 7. API Reference

| Usage | Description |
|-------|-------------|
| `param_scope.a.b \| default` | Read parameter with default value |
| `param_scope.a.b(default)` | Same as above, function call syntax |
| `param_scope.a.b()` | Read required parameter, raises KeyError if missing |
| `param_scope["a.b"]` | Dynamic key access |
| `with param_scope(**dict):` | Create scope with parameter overrides |
| `with param_scope(*list):` | Create scope from string list (e.g., CLI args) |
| `@auto_param("ns")` | Decorator to bind function parameters to `ns.*` |

---

## 8. Best Practices

1. **Key naming**: Use `.` to separate hierarchy levels, e.g., `train.optimizer.lr`
2. **Type consistency**: Keep the same type for a given key across usages
3. **Default values**: Always provide defaults to avoid KeyError
4. **Scope minimization**: Keep scopes as narrow as possible
5. **Process boundaries**: Cross-process scenarios require Rust backend or custom storage adapter
