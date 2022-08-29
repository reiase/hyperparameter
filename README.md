**H**_yper_**P**_arameter_
===========================

A hyper-parameter library for researchers, data scientists and machine learning engineers.

Quick Start
-----------

### Object-Style API:

```python
>>> from hyperparameter import HyperParameter

>>> hp = HyperParameter(a=1, b={'c': 2})
>>> hp.a == 1
True
>>> hp.b.c == 2  # (nested parameter)
True

```

If we want safe access to undefined parameters with default values, we can use `hp()` instead of `hp`:

```python
>>> hp = HyperParameter()
>>> hp().a.b.c.get_or_else(3) # (default value for undefined parameter)
3
>>> hp().a.b.c(3)             # (shortcut for `get_or_else`)
3

>>> hp().a.b.c = 4          # set value to param `a.b.c`
>>> hp().a.b.c(3)           # (default value is ignored) 
4

```

### Scoped Parameter

```python
>>> from hyperparameter import param_scope

# scoped parameter
>>> with param_scope(a=1) as ps: 
...     ps.a == 1
True

```

When nested, the parameter modifications are limited to the inner scope:
``` python
>>> with param_scope(a=1) as ps:
...     with param_scope(a=2) as ps2:
...         ps2.a == 2 # True, a=2 for inner scope
...     ps.a == 1      # True, a=1 for outer scope
True
True

```

The nested scope feature can be used to config the default behavior when used in functions:

```python
#change function behavior with scoped parameter:
def dnn(input):
    # receive parameter using param_scope
    with param_scope() as ps:
        output = linear(inputs)
        return activation_fn(
                    output, 
                    activation=ps().activation("relu"))

# call function with default parameter
dnn()

# passing parameter using param_scope
with param_scope(activation="sigmoid"): 
    dnn()
```

### Predefined Parameter
```python
@auto_param #convert keyword arguments into hyper parameters
def model_train(X, y, learning_rate = 1.0, penalty = 'l1'):
    LR = LogisticRegression(C=1.0, 
        lr=local_param('learning_rate'), 
        penalty=local_param('penalty'))
    LR.fit(X, y)

# specify predefined parameter using `param_scope`
with param_scope('model_train.learning_rate=0.01'):
    model_train(X, y)
```

Examples
--------

### [parameter tunning for researchers](examples/sparse_lr/README.md)

This example shows how to use hyperparameter in your research projects, and make your experiments reproducible.

### [experiment tracing for data scientists](examples/mnist/README.md)

This example shows experiment management with hyperparameter, and tracing the results with mlflow.tracing.

Todo.

### design-pattern for system engineers

Todo.
