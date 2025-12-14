# Cookbook: Common Recipes

This cookbook provides practical solutions for common configuration management scenarios.

## Table of Contents

- [Multi-Environment Configuration (dev/staging/prod)](#multi-environment-configuration)
- [Configuration Inheritance](#configuration-inheritance)
- [Secrets Management](#secrets-management)
- [Feature Flags](#feature-flags)
- [A/B Testing Configuration](#ab-testing-configuration)
- [Multi-Stage Training (RL/ML)](#multi-stage-training)

---

## Multi-Environment Configuration

### Problem
You need different configurations for development, staging, and production environments.

### Solution

**Directory Structure:**
```
config/
├── base.toml          # Shared defaults
├── dev.toml           # Development overrides
├── staging.toml       # Staging overrides
└── prod.toml          # Production overrides
```

**base.toml:**
```toml
[database]
host = "localhost"
port = 5432
pool_size = 5

[logging]
level = "INFO"
format = "%(asctime)s - %(message)s"

[model]
batch_size = 32
learning_rate = 0.001
```

**dev.toml:**
```toml
[database]
host = "localhost"

[logging]
level = "DEBUG"

[model]
batch_size = 8  # Smaller for faster iteration
```

**prod.toml:**
```toml
[database]
host = "prod-db.example.com"
pool_size = 20

[logging]
level = "WARNING"

[model]
batch_size = 256
```

**main.py:**
```python
import os
import hyperparameter as hp

def load_config():
    env = os.environ.get("ENV", "dev")
    return hp.config([
        "config/base.toml",
        f"config/{env}.toml"
    ])

@hp.param("model")
def train(batch_size=32, learning_rate=0.001):
    print(f"Training with batch_size={batch_size}, lr={learning_rate}")

if __name__ == "__main__":
    cfg = load_config()
    with hp.scope(**cfg):
        train()
```

**Usage:**
```bash
ENV=dev python main.py      # batch_size=8
ENV=prod python main.py     # batch_size=256
```

---

## Configuration Inheritance

### Problem
You have multiple model variants that share common settings but differ in specific parameters.

### Solution

**config/models/base_transformer.toml:**
```toml
[model]
type = "transformer"
num_layers = 6
hidden_size = 512
num_heads = 8
dropout = 0.1
activation = "gelu"
```

**config/models/bert_base.toml:**
```toml
[model]
num_layers = 12
hidden_size = 768
num_heads = 12
vocab_size = 30522
```

**config/models/bert_large.toml:**
```toml
[model]
num_layers = 24
hidden_size = 1024
num_heads = 16
vocab_size = 30522
```

**main.py:**
```python
import hyperparameter as hp

def load_model_config(model_name: str):
    """Load model config with inheritance."""
    base_config = "config/models/base_transformer.toml"
    model_config = f"config/models/{model_name}.toml"
    return hp.config([base_config, model_config])

# Usage
cfg = load_model_config("bert_large")
# Result: num_layers=24, hidden_size=1024, dropout=0.1 (inherited)
```

---

## Secrets Management

### Problem
You need to manage sensitive values (API keys, passwords) without committing them to git.

### Solution

**config/app.toml:**
```toml
[api]
base_url = "https://api.example.com"
timeout = 30

[database]
host = "${DATABASE_HOST}"      # From environment variable
password = "${DATABASE_PASS}"
```

**config/secrets.local.toml** (gitignored):
```toml
[api]
key = "sk-your-actual-api-key"

[database]
password = "actual-password"
```

**.gitignore:**
```
config/secrets.local.toml
```

**main.py:**
```python
import os
from pathlib import Path
import hyperparameter as hp

def load_config_with_secrets():
    configs = ["config/app.toml"]
    
    # Load local secrets if exists
    secrets_file = Path("config/secrets.local.toml")
    if secrets_file.exists():
        configs.append(str(secrets_file))
    
    cfg = hp.config(configs)
    return cfg

cfg = load_config_with_secrets()
```

---

## Feature Flags

### Problem
You want to enable/disable features without code changes.

### Solution

**config/features.toml:**
```toml
[features]
new_ui = false
experimental_model = false
debug_mode = true
rate_limiting = true

[features.ab_test]
enabled = true
variant = "control"  # "control" or "treatment"
```

**main.py:**
```python
import hyperparameter as hp

cfg = hp.config("config/features.toml")

with hp.scope(**cfg):
    # Check feature flags anywhere in code
    if hp.scope.features.new_ui | False:
        render_new_ui()
    else:
        render_old_ui()
    
    if hp.scope.features.experimental_model | False:
        model = ExperimentalModel()
    else:
        model = StableModel()
```

**Toggle via CLI:**
```bash
python main.py -D features.new_ui=true
```

---

## A/B Testing Configuration

### Problem
You need to run experiments with different parameter configurations.

### Solution

```python
import hyperparameter as hp
import random

@hp.param("experiment")
def run_experiment(
    model_type="baseline",
    learning_rate=0.001,
    batch_size=32
):
    print(f"Running: {model_type}, lr={learning_rate}, bs={batch_size}")
    # ... training code ...

def get_experiment_config(user_id: str):
    """Deterministic assignment based on user_id."""
    bucket = hash(user_id) % 100
    
    if bucket < 50:
        return {"experiment.model_type": "baseline"}
    else:
        return {"experiment.model_type": "new_model", "experiment.learning_rate": 0.0005}

# Usage
user_config = get_experiment_config("user_123")
with hp.scope(**user_config):
    run_experiment()
```

---

## Multi-Stage Training (RL/ML)

### Problem
You have a training pipeline with multiple stages, each needing different configurations.

### Solution

```python
import hyperparameter as hp

@hp.param("train.pretrain")
def pretrain(lr=0.001, epochs=10, warmup=True):
    print(f"Pretraining: lr={lr}, epochs={epochs}, warmup={warmup}")

@hp.param("train.finetune")
def finetune(lr=0.0001, epochs=5, freeze_backbone=True):
    print(f"Finetuning: lr={lr}, epochs={epochs}, freeze={freeze_backbone}")

@hp.param("train.rl")
def rl_train(lr=0.00001, episodes=1000, exploration=0.1):
    print(f"RL Training: lr={lr}, episodes={episodes}, exploration={exploration}")

def run_pipeline():
    # Stage 1: Pretrain with high LR
    with hp.scope(**{"train.pretrain.lr": 0.001, "train.pretrain.epochs": 20}):
        pretrain()
    
    # Stage 2: Finetune with low LR
    with hp.scope(**{"train.finetune.lr": 0.00005}):
        finetune()
    
    # Stage 3: RL with decaying exploration
    for stage, exploration in enumerate([0.5, 0.3, 0.1, 0.05]):
        with hp.scope(**{"train.rl.exploration": exploration}):
            print(f"--- RL Stage {stage + 1} ---")
            rl_train()

if __name__ == "__main__":
    run_pipeline()
```

This showcases the **dynamic scoping** feature that Hydra cannot easily replicate.

---

## Tips and Best Practices

### 1. Use Type Hints for Better IDE Support

```python
from dataclasses import dataclass
import hyperparameter as hp

@dataclass
class ModelConfig:
    hidden_size: int = 256
    dropout: float = 0.1
    activation: str = "relu"

cfg: ModelConfig = hp.config("config.toml", schema=ModelConfig)
# Now cfg.hidden_size has autocomplete!
```

### 2. Organize Configs by Concern

```
config/
├── model/
│   ├── bert.toml
│   └── gpt.toml
├── training/
│   ├── default.toml
│   └── distributed.toml
├── data/
│   └── preprocessing.toml
└── base.toml
```

### 3. Use Interpolation for DRY Configs

```toml
[paths]
root = "/project"
data = "${paths.root}/data"
models = "${paths.root}/models"
logs = "${paths.root}/logs"
```

### 4. Document Your Config Files

```toml
# Model configuration for BERT variants
# See: https://arxiv.org/abs/1810.04805

[model]
# Number of transformer layers (12 for base, 24 for large)
num_layers = 12

# Hidden dimension (768 for base, 1024 for large)
hidden_size = 768
```

