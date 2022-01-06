**H**_yper_**P**_arameter_
===========================
A hyper-parameter library for researchers, data scientists and machine learning engineers.

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
params().a.b.c(3) # 3 (shortcut for default value)

params().a.b.c = 4 # set value to param `a.b.c`
params().a.b.c(3) # 4 (default value is ignored) 
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
        hp.a == 2 # True
    hp.a == 1 # True
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
