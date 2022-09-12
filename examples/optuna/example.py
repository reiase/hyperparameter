import optuna

def objective(trial):
    x = trial.suggest_float('x.x.y', -10, 10)
    return (x - 2) ** 2

study = optuna.create_study()
study.optimize(objective, n_trials=100)

print(study.best_params)  # E.g. {'x': 2.002108042}