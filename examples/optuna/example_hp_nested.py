import optuna
import hyperparameter as hp

@hp.param
def objective_x(x = 0.0):
    return (x - 2) ** 2

@hp.param
def objective_y(y = 0.0):
    return (y - 1) ** 4

def objective():
    return objective_x() + objective_y()

def wrapper(trial):
    trial = lazy_dispatch(trial)
    with hp.scope(**{
        "objective_x.x": trial.suggest_float('objective_x.x', -10, 10),
        "objective_y.y": trial.suggest_float('objective_y.y', -10, 10)
    }):
        return objective()

study = optuna.create_study()
study.optimize(wrapper, n_trials=100)

study.best_params  # E.g. {'x': 2.002108042}

with hp.scope(**study.best_params):
    print(f"{study.best_params} => {objective()}")