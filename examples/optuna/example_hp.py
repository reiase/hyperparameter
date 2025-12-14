import optuna
import hyperparameter as hp

@hp.param
def objective(x = 0.0):
    return (x - 2) ** 2

def wrapper(trial):
    trial = lazy_dispatch(trial)
    with hp.scope(**{
        "objective.x": trial.suggest_float('objective.x', -10, 10)
    }):
        return objective()

study = optuna.create_study()
study.optimize(wrapper, n_trials=100)

study.best_params  # E.g. {'x': 2.002108042}

with hp.scope(**study.best_params):
    print(f"{study.best_params} => {objective()}")