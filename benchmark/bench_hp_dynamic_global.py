import time
from hyperparameter import auto_param, param_scope, run_cli

@auto_param
def main():
    start = time.time()
    acc = 0
    
    for _ in range(1_000_000):
        acc += param_scope.model.layers._0.size | 10

    duration = time.time() - start
    print(f"Hyperparameter Time: {duration:.4f} seconds (acc={acc})")
    return duration

if __name__ == "__main__":
    run_cli()
