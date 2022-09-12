Hyper-Parameter Optimization
============================

This example is based on `optuna` [quick start example](https://optuna.org/#code_quickstart). [Optuna](https://optuna.org/) is an open-source [hyperparameter](https://github.com/reiase/hyperparameter) optimization framework which is easy to use:

```python
import optuna

def objective(trial):
    x = trial.suggest_float('x', -10, 10)
    return (x - 2) ** 2

study = optuna.create_study()
study.optimize(objective, n_trials=100)

study.best_params  # E.g. {'x': 2.002108042}
```

The above example creates a `study` object to search for the best parameter `x` that minimizes the objective function `(x-2)^2`.

Parameter Searching with [`HyperParameter`](https://github.com/reiase/hyperparameter)
-----------------------------------------

Parameter searching can be much easier with [`HyperParameter`](https://github.com/reiase/hyperparameter):

```python
import optuna
from hyperparameter import param_scope, auto_param, lazy_dispatch

@auto_param
def objective(x = 0.0):
    return (x - 2) ** 2

def wrapper(trial):
    trial = lazy_dispatch(trial)
    with param_scope(**{
        "objective.x": trial.suggest_float('objective.x', -10, 10)
    }):
        return objective()

study = optuna.create_study()
study.optimize(wrapper, n_trials=100)

study.best_params  # E.g. {'x': 2.002108042}
```

We directly apply [the `auto_param` decorator](https://reiase.github.io/hyperparameter/quick_start/#auto_param) to the objective function so that it accepts parameters from [`param_scope`](https://reiase.github.io/hyperparameter/quick_start/#param_scope). Then we define a wrapper function that adapts `param_scope` API to `optuna`'s `trial` API and starts the parameter experiment as suggested in `optuna`'s example.

Put the Best Parameters into Production
---------------------------------------

To put the best parameters into production, we can directly pass them to `param_scope`. This is very convenient if you want to put a ML model into production.

```python
with param_scope(**study.best_params):
    print(f"{study.best_params} => {objective()}")
```

Optimization on Nested Functions
--------------------------------

`param_scope` and `auto_param` also support complex problems with nested functions:

```python
@auto_param
def objective_x(x = 0.0):
    return (x - 2) ** 2

@auto_param
def objective_y(y = 0.0):
    return (y - 1) ** 3

def objective():
    return objective_x() * objective_y()

def wrapper(trial):
    trial = lazy_dispatch(trial)
    with param_scope(**{
        "objective_x.x": trial.suggest_float('objective_x.x', -10, 10),
        "objective_y.y": trial.suggest_float('objective_y.y', -10, 10)
    }):
        return objective()

study = optuna.create_study()
study.optimize(wrapper, n_trials=100)

study.best_params  # E.g. {'x': 2.002108042}
```
