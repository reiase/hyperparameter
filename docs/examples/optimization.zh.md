参数寻优
=======

原示例出自`optuna`项目的[quick start example](https://optuna.org/#code_quickstart). [Optuna](https://optuna.org/) 是一款易于使用的开源超参优化框架:

```python
import optuna

def objective(trial):
    x = trial.suggest_float('x', -10, 10)
    return (x - 2) ** 2

study = optuna.create_study()
study.optimize(objective, n_trials=100)

study.best_params  # E.g. {'x': 2.002108042}
```

以上代码创建了一个 `study` 对象，用来搜索最小化目标函数 `(x-2)^2` 的参数`x` 的最优取值，.

使用 [`HyperParameter`](https://github.com/reiase/hyperparameter)进行超参搜索
-----------------------------------------

我们可以借助 [`HyperParameter`](https://github.com/reiase/hyperparameter)将上述搜索过程大幅简化：

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

通过 [`auto_param`](https://reiase.github.io/hyperparameter/quick_start/#auto_param) 装饰器，我们对目标函数进行了`超参化`，使其能够从[`param_scope`](https://reiase.github.io/hyperparameter/quick_start/#param_scope)读取参数。之后我们定义了一个辅助函数来对接`param_scope`和 `optuna` 的`trial` 接口，并开始超参寻优。

使用 `auto_param` 与 `param_scope` 的好处是将代码不再耦合`optuna`，可以在生产代码中复用代码。

生产化部署
---------

可以通过直接将 `study` 搜索到的最优参数传递给 `param_scope` 来是实现实验结果的复现以及生产化部署。

```python
with param_scope(**study.best_params):
    print(f"{study.best_params} => {objective()}")
```

多层嵌套函数的参数优化
-------------------

`param_scope` 和 `auto_param` 可以用于优化复杂问题中的嵌套函数的参数优化，比如：

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

使用 `auto_param` 可以避免在嵌套函数之间传递 `trial` 对象，让代码看起来更为自然直接。
