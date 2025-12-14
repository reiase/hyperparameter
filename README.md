# Hyperparameter

<p align="center">
  <img src="hyperparameter.svg" alt="Hyperparameter logo" width="180" />
</p>

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

<p align="center">

**Hyperparameter, Make configurable AI applications. Build for Python/Rust hackers.**

</p>

Hyperparameter is a versatile library designed to streamline the management and control of hyperparameters in machine learning algorithms and system development. Tailored for AI researchers and Machine Learning Systems (MLSYS) developers, Hyperparameter offers a unified solution with a focus on ease of use in Python, high-performance access in Rust and C++, and a set of macros for seamless hyperparameter management.

## 5-Minute Try

```bash
pip install hyperparameter

# Run a ready-to-use demo
python -m hyperparameter.examples.quickstart

# Try the @auto_param CLI: override defaults from the command line
python -m hyperparameter.examples.quickstart --define greet.name=Alice --enthusiasm=3

# Inspect params and defaults
python -m hyperparameter.examples.quickstart -lps
    python -m hyperparameter.examples.quickstart -ep greet.name
    
    # Running from source? Use module mode or install editable
    # python -m hyperparameter.examples.quickstart
    # or: pip install -e .
    ```
    
    ## Why Hyperparameter?
    
    ### 🚀 Unmatched Performance (vs Hydra)
    
    Hyperparameter is built on a high-performance Rust backend, making it significantly faster than pure Python alternatives like Hydra, especially in inner-loop parameter access.
    
    | Method | Time (1M iters) | Speedup (vs Hydra) |
    | :--- | :--- | :--- |
    | **HP: Injected (Native Speed)** | **0.0184s** | **856.73x** 🚀 |
    | **HP: Dynamic (Optimized)** | **2.4255s** | **6.50x** ⚡️ |
    | **Hydra (Baseline)** | 15.7638s | 1.00x |
    
    > Benchmark scenario: Accessing a nested parameter `model.layers.0.size` 1,000,000 times in a loop.
    > See `benchmark/` folder for reproduction scripts.
    
    ### ✨ Zero-Dependency Schema Validation
    
    Hyperparameter supports structural validation using standard Python type hints without introducing heavy dependencies (like Pydantic or OmegaConf).
    
    ```python
    from dataclasses import dataclass
    from hyperparameter import loader
    
    @dataclass
    class AppConfig:
        host: str
        port: int
        debug: bool = False
    
    # Validates types and converts automatically: "8080" -> 8080 (int)
    cfg = loader.load("config.toml", schema=AppConfig)
    ```
    
    ## Key Features

### For Python Users

- **Pythonic Syntax:** Define hyperparameters using keyword argument syntax;

    - **Intuitive Scoping:** Control parameter scope through `with` statement;
    
    - **Configuration File:** Easy to load parameters from config files (JSON/TOML/YAML) with composition and interpolation support;
    
    - **Zero-Overhead Validation:** Optional schema validation using standard Python type hints;
    
    ### For Rust and C++ Users

- **High-Performance Backend:** Hyperparameter is implemented in Rust, providing a robust and high-performance backend for hyperparameter management. Access hyperparameters in Rust and C++ with minimal overhead, making it ideal for ML and system developers who prioritize performance.

- **Macro-Based Parameter Management:** Hyperparameter provides a set of macros for both Rust and C++ users. These macros mimic Python's `with` statements and adhere to language-specific scoping rules.

- **Compile-Time Hashing:** Both Rust and C++ interfaces utilize compile-time hashing of hyperparameter names, reducing runtime hash computation overhead.

## Quick Start

### Installation

```bash
pip install hyperparameter
```

### Python

```python
from hyperparameter import auto_param, param_scope

@auto_param("foo")
def foo(x=1, y="a"):
    return f"x={x}, y={y}"

foo()  # x=1, y='a'

with param_scope(**{"foo.x": 2}):
    foo()  # x=2, y='a'
```

### Rust

```rust
fn foo() -> i32 {
    with_params! {
        @get x = foo.x or 1i32; // Read hyperparameter with default value

        println!("x={}", x);
    }
}

fn main() {
    foo(); // x=1

    with_params! {
        @set foo.x = 2i32; // Set hyperparameter

        foo(); // x=2
    }

    foo(); // x=1
}
```

### C++

```cpp
ASSERT(1 == GET_PARAM(a.b, 1), "get undefined param");
{
  auto guard = WITH_PARAMS(a, 1,        //
                            a.b, 2.0,    //
                            a.b.c, true, //
                            a.b.c.d, "str");
  ASSERT(1 == GET_PARAM(a, 0), "get int value");
  ASSERT(1 == GET_PARAM(a, 0), "get int value");
}
```

## Detailed Usage Examples

### Support for Default Values

#### Python

```python
x = param_scope.foo.x | "default value"
```

#### Rust

```rust
@get x = foo.x or "default value";
```

### Scope Control of Parameter Values

#### Python

```python
with param_scope() as ps: # 1st scope start
    ps.foo.x=1
    with param_scope() as ps2: # 2nd scope start
        ps.foo.y=2
    # 2nd scope end
# 1st scope end
```

#### Rust

```rust
with_params!{ // 1st scope start
    @set foo.x=1;

    with_params!{ //2nd scope start
        @set foo.y=2

        ...
    } // 2nd scope end
} // 1st scope end
```

### Thread Isolation/Thread Safety

#### Python

```python
@auto_param("foo")
def foo(x=1): # Print hyperparameter foo.x
    print(f"foo.x={x}")

with param_scope() as ps:
    ps.foo.x=2 # Modify foo.x in the current thread
    
    foo() # foo.x=2
    threading.Thread(target=foo).start() # foo.x=1, new thread's hyperparameter value is not affected by the main thread
```

#### Rust

```rust
fn foo() { // Print hyperparameter foo.x
    with_params!{
        @get x = foo.x or 1;

        println!("foo.x={}", x);
    }
}

fn main() {
    with_params!{
        @set foo.x = 2; // Modify foo.x in the current thread
        
        foo(); // foo.x=2
        thread::spawn(foo); // foo.x=1, new thread's hyperparameter value is not affected by the main thread
    }
}
```

### Command Line Application

In command line applications, it's common to define hyperparameters using command line arguments (e.g., `-D, --define`) and control hyperparameters on the command line. Here's an example in Python and Rust:

#### Python

```python
# example.py
from hyperparameter import param_scope, auto_param

@auto_param("example")
def main(a=0, b=1):
    print(f"example.a={a}, example.b={b}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()

    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with param_scope(*args.define):
        main()
```

#### Rust

```rust
// example.rs
use hyperparameter::*;
use hyperparameter_derive::Parser;

fn main() {
    #[derive(Parser, Debug)]
    struct DeriveArgs {
        #[arg(short = 'D', long)]
        define: Vec<String>,
    }

    let args = DeriveArgs::parse();

    with_params! {
        params ParamScope::from(&args.define);

        foo()
    }
}

fn foo() {
    with_params! {
        @get a = example.a or 0;
        @get b = example.b or 1;
        
        println!("example.a={}, example.b={}",a ,b);
    }
}
```

## More Examples

### [parameter tunning for researchers](examples/sparse_lr/README.md)

This example demonstrates how to use hyperparameter in research projects, and make experiments reproducible.

### [experiment tracing for data scientists](examples/mnist/README.md)

This example showcases experiment management with hyperparameter and result tracing with mlflow.tracing.

## Behavior Guarantees (Semantic Contract)

- **Keys & hashing:** keys use `.` for nesting, case is preserved, and hashing uses the same UTF-8 input and seed across Python/Rust/C++; invalid characters are an error.
- **Read precedence:** current thread’s innermost scope > parent scopes outward > frozen global snapshot > user default. Writes only affect the current scope and rollback on exit.
- **Defaults vs. missing:** only missing keys fall back to defaults; explicit `None`/`False`/`0` are treated as existing values. Type conversion rules (bool/int/float/str) are consistent across languages; invalid values use a best-effort conversion and otherwise fall back to the provided default (no silent random values).
- **Threads & `frozen()`:** each thread starts from the frozen global snapshot; mutations stay in-thread unless `frozen()` is called, which atomically updates the global snapshot. Global mutations are lock-protected in the Python backend, matching Rust semantics.
- **Error model:** reading an undefined key without a default raises a key error; backend load failure falls back to the Python backend without noisy tracebacks; no silent failure on type errors.
- **Multiprocess notice:** cross-process consistency requires a shared backend (e.g., Rust backend or user-provided storage adapter); the built-in Python backend only guards threads, not processes.
