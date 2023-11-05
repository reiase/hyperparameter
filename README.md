Hyperparameter
===========================

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

<p align="center">

**Hyperparameter, Make configurable AI applications. Build for Python/Rust hackers.**

</p>

Hyperparameter is a Python/Rust library for managing hyperparameters that control the learning process of an ML model or the behaviors of an underlying machine learning system.

Quick Start
-----------

### Using `Hyperparameter` in Python

First, with decorator `auto_param`, we can define hyperparameters by adding keyword arguments to a function:

```python
from hyperparameter import auto_param

@auto_param("foo")
def foo(
      x=1,  # define `foo.y`=1 
      y="a" # define`foo.z`="a"
    ): 
    return f"x={x}, y={y}, z={z}"
```

Then, we can control hyperparameters with `param_scope`:

```python
from hyperparameter import param_scope

foo() # x=1, y='a'
with param_scope(**{"foo.x":2}):
    foo() # x=2, y='a'
```

### Using `Hyperparameter` in Rust

```rust
fn foo() -> i32{
    with_params! { // create scope
        get x = foo.x or 1i32; // read foo.x with default value `1`

        println!("x={}", x);
    } // scope end
}

fn main() {
    foo(); // x=1，param foo.x=1
    with_params! {// create scope
        set foo.x = 2i32; // set param foo.x = 2

        foo(); // x = 2, param foo.x=2
    }// scope end
    foo(); // x=1，param foo.x=1
}
```

Features
--------

- Default value for all parameters:

    ```python
    # python
    x = param_scope.foo.x | "default value"
    ```
    ```rust
    // rust
    get x = foo.x or "default value";
    ```

- Scoped parameter values：

    ```python
    # python
    with param_scope() as ps: # 1st scope start
        ps.foo.x=1
        with param_scope() as ps2: # 2nd scope start
            ps.foo.y=2
        # 2nd scope end
    # 1st scope end
    ```
    ```rust
    // rust
    with_params!{ // 1st scope start
        set foo.x=1;

        with_params!{ //2nd scope start
            set foo.y=2

        } // 2nd scope end
    } // 1st scope end
    ```

- Thread Isolation and Thread Safety 

    ```python
    # python
    @auto_param("foo")
    def foo(x=1): # print foo.x
        print(f"foo.x={x}")
    
    with param_scope() as ps:
        ps.foo.x=2 # modify foo.x in current thread
        
        foo() # foo.x=2
        threading.Thread(target=foo).start() # foo.x=1, the above modification does not affect new thread 
    ```
    ```rust
    // rust
    fn foo() { // print foo.x
        with_params!{
            get x = foo.x or 1;

            println!("foo.x={}", x);
        }
    }

    fn main() {
        with_params!{
            set foo.x = 2; // modify foo.x in current thread
            
            foo(); // foo.x=2
            thread::spawn(foo); // foo.x=1, the above modification does not affect new thread 
        }
    }
    ```

Build CMD Line Application with `Hyperparameter`
------------------------------------------------

We can define parameters with a command line argument (for example, `-D, --define`), and call the application with following command:
```bash
./example \
    ... 
    -D example.a=1 \
    -D example.b=2 \
    ...
```

A quick implementation of `-D,--define`:

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

```rust
//rust
fn foo() {
    with_params! {
        get a = example.a or 0;
        get b = example.b or 1;
        
        println!("example.a={}, example.b={}",a ,b);
    }
}

#[derive(Parser, Debug)]
struct DeriveArgs {
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

fn main() {
    let args = DeriveArgs::parse();
    with_params! {
        params ParamScope::from(&args.define);

        foo()
    }
}

```


Examples
--------

### [parameter tunning for researchers](examples/sparse_lr/README.md)

This example demonstrates how to use hyperparameter in research projects, and make experiments reproducible.

### [experiment tracing for data scientists](examples/mnist/README.md)

This example showcases experiment management with hyperparameter and result tracing with mlflow.tracing.
