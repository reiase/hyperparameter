import time
import hyperparameter as hp

@hp.param
def main():
    start = time.time()
    acc = 0
    
    # We use hp.scope directly, which is the idiomatic way 
    # to access parameters anywhere in the code.
    with hp.scope() as ps:
        for _ in range(1_000_000):
            acc += ps.model.layers._0.size | 10

    duration = time.time() - start
    print(f"Hyperparameter Time: {duration:.4f} seconds (acc={acc})")
    return duration

if __name__ == "__main__":
    # Pre-populate scope to simulate loaded config
    with hp.scope(**{"model.layers._0.size": 10}):
        main()
