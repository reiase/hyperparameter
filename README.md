**H**_yper_**P**_arameter_
===========================

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

HyperParameter is a pythonic configuration framework designed to simplify the massive configuration in complex applications. The key feature is a dynamic hierarchical parameter tree composited with scopes. HyperParameter is particularly well suited for machine learning experiments and related systems, which have many parameters and nested codes.

Key Conceptions
---------------

1. `parameter tree`, a nested python dict with support for default values and object-style API;
1. `param_scope`, a context manager for compositing the ` parameter tree` with nested `param_scope`;
2. `auto_param`, a decorator allowing default parameters of a function or class to be supplied from `param_scope`;


Quick Start
-----------

A quick example for defining a model with HyperParameter:

```python
@auto_param
def dnn(input, layers=3, activation="relu"):
  	"""
  	build a DNN model with the following configurations:
  		- dnn.layers(default: 3)
  		- dnn.activation(default: "relu")
  	"""
    for i in range(layers):
        input = Linear(input)
        input = activation_fn(
            activation,
            input
        )
    return input

# call dnn with default configuration 
# and create a 3 layer dnn with relu activation
dnn(x)

# passing parameter using param_scope
with param_scope(**{
        "dnn.layers": 4, 
        "dnn.activation": "sigmoid"}):
    # create a 4 layer dnn with sigmoid activation
    dnn()
```

Another example for building ML system:

```python
@auto_param
def inference(x, backend="tvm"):
    ...

with param_scope(backend="onnx"):
    inference(x)
```

Advanced Usage
--------------
### Nested Scope and Configuration Composition

HyperParameter uses nested  `param_scope` for configuration composition :

``` python
from hyperparameter import param_scope
# on initialization, the parameter tree is empty: {}
with param_scope(a=1) as ps:
    # in the with context, the composited parameter tree is {"a": 1}
    ps == {"a": 1}
    with param_scope(a=2, b=3) as ps2:
        # in the nested scope, the composited parameter tree is {"a": 2, "b": 3}
        # param `b` is a new, and param `a` is overwrite by new value
        ps2 == {"a": 2, "b": 3}
    # when exit the inner scope, the modification of inner scope is cleaned up
    # the composited parameter tree is {"a": 1}
    ps == {"a": 1}
```

### Manage Parameters from CMD Line

It is recommended to use three-layer configuration for complex programmes:

1. `inline default values`;
2. `config file`, which will override `inline default values`;
3. `cmdline arguments` that override both `config file` and `inline default values`;

```python
from hyperparameter import param_scope, auto_param

@auto_param
def main(a=0, b=1): # `inline default values`
    print(a, b)

if __name__ == "__main__":
    import argparse
    import json
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=str, default=None)
    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with open(args.config) as f:
        cfg = json.load(f) # read config file
    with param_scope(**cfg): # scope for `config file`
        with param_scope(*args.define): # scope for `cmdline args`
            main()
```

Examples
--------

### [parameter tunning for researchers](examples/sparse_lr/README.md)

This example shows how to use hyperparameter in your research projects, and make your experiments reproducible.

### [experiment tracing for data scientists](examples/mnist/README.md)

This example shows experiment management with hyperparameter, and tracing the results with mlflow.tracing.
