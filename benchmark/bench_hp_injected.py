import time
import hyperparameter as hp

@hp.param
def main(layer_size: int = 10):
    start = time.time()
    acc = 0
    
    for _ in range(1_000_000):
        acc += layer_size

    duration = time.time() - start
    print(f"Hyperparameter Time: {duration:.4f} seconds (acc={acc})")
    return duration

if __name__ == "__main__":
    hp.launch()
