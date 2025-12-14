import time
from hyperparameter import auto_param, param_scope

@auto_param
def main():
    start = time.time()
    acc = 0
    
    for _ in range(1_000_000):
        with param_scope() as ps:
            acc += ps.model.layers._0.size | 10

    duration = time.time() - start
    print(f"Hyperparameter Time: {duration:.4f} seconds (acc={acc})")
    return duration

if __name__ == "__main__":
    # Pre-populate scope to simulate loaded config
    with param_scope(**{"model.layers._0.size": 10}):
        main()
