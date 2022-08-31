**H**_yper_**P**_arameter_
===========================

HyperParameter is a configuration framework designed for data scientists and machine learning enginners. HyperParameter provides the following features:

1. `param_scope`, a context manager maintains a thread-safe global parameter configuration and manage parameter modifications with nested `param_scope`;
2. `auto_param`, a decorator allowing default parameters of a function or class to be supplied from `param_scope`;

HyperParameter is particularly well suited for machine learning experiments and related systems, which have many parameters and nested codes.

Quick Start
-----------

A quick example for defining model with HyperParameter:

```python
@auto_param()
def dnn(input, layers=3, activation="relu"):
    for i in range(layers):
        input = Linear(input)
        input = activation_fn(
            activation,
            input
        )
    return input

# create a 3 layer dnn with relu activation
dnn(x)

# passing parameter using param_scope
with param_scope(
        "dnn.layers=4", 
        "dnn.activation=sigmoid"):
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

We can also load parameters from config file (e.g., YAML config):

```YAML
dnn:
    layers: 4
    activation: sigmoid
```

```python
cfg = yaml.load(f)

with param_scope(**cfg):
    dnn(x)
```

Advanced Usage
--------------
### Nested Scope

When nested, the `param_scope` will manage parameter modifications :
``` python
>>> from hyperparameter import param_scope
>>> with param_scope(a=1) as ps:
...     with param_scope(a=2) as ps2:
...         ps2.a == 2 # True, a=2 for inner scope
...     ps.a == 1      # True, a=1 for outer scope
True
True

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
