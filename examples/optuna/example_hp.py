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

with param_scope(**study.best_params):
    print(f"{study.best_params} => {objective()}")