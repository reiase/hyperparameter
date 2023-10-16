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

`Hyperparameter` uses the `auto_param` decorator to convert keywords arguments into configurable parameters:

```python
from hyperparameter import auto_param, param_scope

# Define the function 'foo' with configurable parameters
@auto_param("foo")
def foo(x=1):
    return f"x={x}"

# Control the parameters using 'param_scope'
foo()  # x=1, if no params is defined
with param_scope(**{"foo.x": 2}):
    foo()  # x=2', when `foo.x` is set to `2`
```

Rust version of the above example:
```rust
fn foo() -> i32{
    with_params! {
        get x = foo.x or 1i32;

        x
    }
}

fn main() {
    println!("foo() = {}", foo()); // foo() = 1
    with_params! {
        set foo.x = 2i32;

        println!("foo() = {}", foo()); // foo() = 2
    }
}
```

Advanced Usage
--------------

### Read/Write Parameters

```python
from hyperparameter import param_scope

# create param_scope
with param_scope():
    pass

with param_scope("foo.y=1", "foo.z=b"):
    pass

with param_scope(**{"foo.y":1, "foo.z":2}):
    pass

# read param with default value
with param_scope(**{"foo.y":2}) as ps:
    y = ps.foo.y(1)  
    y = ps.foo.y | 1
    y = param_scope.foo.y(1)
    y = param_scope.foo.y | 1
    foo(1) # x=1, y=2, z='a'

# wite values to param_scope
with param_scope(**{"foo.y":2}) as ps:
    ps.foo.y = 2
    param_scope.foo.y = 2
```

### Nested Scope

`Hyperparameter` support nested `param_scope`:

``` python
from hyperparameter import param_scope

# no param_scope, use the default value defined in foo
foo(1) # x=1, y=1, z='a'

# start a new param_scope
# and set the default value of `foo.y` to `2`
with param_scope(**{"foo.y":2}) as ps:
    # found one param_scope `ps`, 
    # and receive default value of `foo.y` from `ps`
    foo(1) # x=1, y=2, z='a'

    # start another param_scope
    # and set the default value of `foo.y` to `3`
    with param_scope(**{"foo.z": "b"}) as ps2:
        # found nested param_scope `ps2`, 
        # and receive default values of `foo.z` from `ps2`
        foo(1) # x=1, y=2, z='b'
    # `ps2` ends here, and `foo.y` is restored to `2`
    foo(1) # x=1, y=2, z='a'
# `ps` ends here, and `foo.y` is restored to `1`
foo(1) # x=1, y=1, z='a'
```

### CMD Line Arguments

An example CLI app: 

```python
from hyperparameter import param_scope, auto_param

@auto_param
def main(a=0, b=1): # `inline default values`
    print(a, b)

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()

    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with param_scope(*args.define):
        main()
```

Examples
--------

### [parameter tunning for researchers](examples/sparse_lr/README.md)

This example demonstrates how to use hyperparameter in research projects, and make experiments reproducible.

### [experiment tracing for data scientists](examples/mnist/README.md)

This example showcases experiment management with hyperparameter and result tracing with mlflow.tracing.
