import time
import hydra
from omegaconf import DictConfig

@hydra.main(version_base=None, config_name="config", config_path=".")
def main(cfg: DictConfig):
    start = time.time()
    acc = 0
    
    # Corrected to 1 million iterations
    for _ in range(1_000_000):
        # Access: model.layers.0.size
        acc += cfg.model.layers[0].size
        
    duration = time.time() - start
    print(f"Hydra Time: {duration:.4f} seconds (acc={acc})")
    return duration

if __name__ == "__main__":
    main()
