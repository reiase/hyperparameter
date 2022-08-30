Quick Start
===========

HyperParameter is a configuration and parameter management library for Python. HyperParameter provides the following features:

1. `HyperParameter` class, providing object-style api for accessing dict data and handling undefine parameters with default values;
2. `param_scope` context manager, maintains a thread-safe `HyperParameter` and manage parameter modifications with nested `param_scope`;
3. `auto_param` decorator, allowing default parameters of a function or class to be supplied from `param_scope`;

HyperParameter is particularly well suited for machine learning experiments and related systems, which have many parameters and nested codes.

`HyperParameter`
----------------

### create hyperparameter

```python
>>> from hyperparameter import HyperParameter
>>> hp = HyperParameter(param1=1, param2="A")
>>> hp
{'param1': 1, 'param2': 'A'}

```

### access parameters in hyperparameter

we can access the parameter stored in HyperParameter with object-style API:

```python
>>> hp.param1
1

>>> hp.param2
'A'

```

nested parameters are also supported:

```python
>>> hp = HyperParameter(param1=1, obj2={"prop3": 3})
>>> hp.obj2.prop3
3

```

### access undefined parameter

Undefined parameter is treated as false in `if` statement: 

```python
>>> if not hp.undefined_param: print("param not defined")
param not defined

```

The hyperparameter object is a `Callable`, and will be turned into `safe-mode` when called. In safe-mode the users must provide a default value when accessing parameters.

```python
>>> hp().undefined_param("default_value")
'default_value'

```

`param_scope`
-------------

### manage hyper-parameters with `param_scope`

`param_scope` create a `HyperParameter` object for the context it manages:

```python
>>> from hyperparameter import param_scope
>>> with param_scope(param1=1, param2="A") as ps:
...     ps.param1
1

```

### access hyper-parameter in functions

`param_scope` is often used to configure the behavior of a function.

```python
>>> def foo(): # access parameter
...     ps = param_scope.current()
...     return ps.param1

>>> def foo_with_default(): # access parameter with default value
...     ps = param_scope.current()
...     return ps().param1(2) # default value is `2`

>>> with param_scope(param1=1, param2="A") as ps:
...     foo()
1

>>> with param_scope(param1=1, param2="A") as ps:
...     foo_with_default()
1

>>> with param_scope(param2="A") as ps:
...     foo_with_default()
2

```

`auto_param`
------------

### Define a function with `auto_param` decorator

`auto_param` provides an easier way to define a configurable function. The configurable parameters and their default values are defined with kwargs if `auto_param` is used.

```python
>>> from hyperparameter import auto_param
>>> @auto_param
... def foo(a, b=1, c=2):
...     return (a, b, c)

>>> foo(0)
(0, 1, 2)

```

### config function default parameter with `param_scope`

We can config the default behavior of a function with `param_scope`. Since the `param_scope` API is scoped and thread-safe, we can use it without too much warry.

```python
>>> with param_scope("foo.b=0"):
...     foo(0)
(0, 0, 2)

```