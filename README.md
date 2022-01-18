**H**_yper_**P**_arameter_
===========================

A hyper-parameter library for researchers, data scientists and machine learning engineers.

- [**H**_yper_**P**_arameter_](#hyperparameter)
- [Quick Start](#quick-start)
  - [Object-Style API:](#object-style-api)
  - [Scoped Parameter](#scoped-parameter)
  - [Predefined Parameter](#predefined-parameter)
- [Examples](#examples)
  - [parameter tunning for researchers](#parameter-tunning-for-researchers)
  - [experiment tracing for data scientists](#experiment-tracing-for-data-scientists)
  - [design-pattern for system engineers](#design-pattern-for-system-engineers)

Quick Start
============

## Object-Style API:

```python
from hyperparameter import HyperParameter

params = HyperParameter(a=1, b={'c': 2})
params.a == 1   # True
params.b.c == 2 # True (nested parameter)
```

or becomes powerful with `params()`:

```python
params().a.b.c.getOrElse(3) # 3 (default value)
params().a.b.c(3)           # 3 (shortcut for default value)

params().a.b.c = 4          # set value to param `a.b.c`
params().a.b.c(3)           # 4 (default value is ignored) 
```

## Scoped Parameter

```python
from hyperparameter import param_scope

# scoped parameter
with param_scope(a=1) as hp: 
    hp.a == 1 # True
```
or becomes powerful with `nested scope`:
``` python
with param_scope(a=1) as hp: 
    with param_scope(a=2) as hp: 
        hp.a == 2 # True, a=2 for inner scope
    hp.a == 1     # True, a=1 for outer scope
```

even more powerful when using `param_scope` in function:

```python
#change function behavior with scoped parameter:
def foo(arg):
    # receive parameter using param_scope
    with param_scope() as hp: 
        if (hp().param1.getOrElse(1) == 1):
            return 1
        else:
            return 2
        ...

# call function with default parameter
foo() # 1

# passing parameter using param_scope
with param_scope(param1=2): 
    foo() # 2
```

## Predefined Parameter
```python
@let( # predefine two parameter for `model_train`
    learning_rate = 1.0,
    penalty = 'l1'
)
def model_train(X, y):
    LR = LogisticRegression(C=1.0, 
        lr=local_param('learning_rate'), 
        penalty=local_param('penalty'))
    LR.fit(X, y)

# specify predefined parameter using `param_scope`
with param_scope('model_train.learning_rate=0.01'):
    model_train(X, y)
```

Examples
========

## [parameter tunning for researchers](examples/sparse_lr/README.md)

This example shows how to use hyperparameter in your research projects, and make your experiments reproducible.

## experiment tracing for data scientists

Todo.

## design-pattern for system engineers

Todo.