import time
import hyperparameter as hp

@hp.param
def main():
    start = time.time()
    acc = 0
    
    for _ in range(1_000_000):
        acc += hp.scope.model.layers._0.size | 10

    duration = time.time() - start
    print(f"Hyperparameter Time: {duration:.4f} seconds (acc={acc})")
    return duration

if __name__ == "__main__":
    hp.launch()
