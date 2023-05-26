Hyperparameter
===========================

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

<p align="center">

**Hyperparameter, Make configurable AI applications.Build for Python hackers.**

</p>

Quick Start
-----------

`Hyperparameter` uses `auto _ param` decorator to convert keywords arguments into configurable parameters:

```python
from hyperparameter import auto_param

@auto_param("foo")
def foo(x, y=1, z="a"):
    return f"x={x}, y={y}, z={z}"
```

The parameters can be controlled with `param_scope`

```python
from hyperparameter import param_scope

foo(1) # x=1, y=1, z='a'
with param_scope(**{"foo.y":2}):
    foo(1) # x=1, y=2, z='a'
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

This example shows how to use hyperparameter in your research projects, and make your experiments reproducible.

### [experiment tracing for data scientists](examples/mnist/README.md)

This example shows experiment management with hyperparameter, and tracing the results with mlflow.tracing.
