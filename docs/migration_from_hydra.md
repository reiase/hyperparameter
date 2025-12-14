# Migrating from Hydra

This guide helps you migrate existing projects from Hydra to Hyperparameter. We'll cover the key differences and provide side-by-side comparisons.

## Why Migrate?

| Aspect | Hydra | Hyperparameter |
| :--- | :--- | :--- |
| **Performance** | Pure Python (slower in loops) | Rust backend (6-850x faster) |
| **Dependencies** | Heavy (antlr4, omegaconf, etc.) | Minimal (only `toml`) |
| **Config Style** | Top-down (pass `cfg` everywhere) | Bottom-up (inject into functions) |
| **Scoping** | Static (compose at startup) | Dynamic (change at runtime) |

## Quick Comparison

### Defining Parameters

**Hydra:**
```python
# config.yaml
model:
  hidden_size: 256
  dropout: 0.1

# main.py
import hydra
from omegaconf import DictConfig

@hydra.main(config_path=".", config_name="config")
def main(cfg: DictConfig):
    print(cfg.model.hidden_size)  # 256
```

**Hyperparameter:**
```python
# config.toml
[model]
hidden_size = 256
dropout = 0.1

# main.py
from hyperparameter import auto_param, param_scope, loader

@auto_param("model")
def build_model(hidden_size=128, dropout=0.0):
    print(hidden_size)  # 256 (from config)

if __name__ == "__main__":
    cfg = loader.load("config.toml")
    with param_scope(**cfg):
        build_model()
```

### Config Composition (Multiple Files)

**Hydra:**
```yaml
# config.yaml
defaults:
  - model: resnet
  - dataset: imagenet
  - _self_

# model/resnet.yaml
name: resnet50
layers: 50
```

**Hyperparameter:**
```python
from hyperparameter import loader, param_scope

# Load and merge multiple configs (later files override earlier)
cfg = loader.load(["base.toml", "model/resnet.toml", "dataset/imagenet.toml"])

with param_scope(**cfg):
    train()
```

### Variable Interpolation

**Hydra (OmegaConf):**
```yaml
paths:
  data_dir: /data
  output_dir: ${paths.data_dir}/outputs
```

**Hyperparameter:**
```toml
[paths]
data_dir = "/data"
output_dir = "${paths.data_dir}/outputs"
```

Both support the same `${key}` syntax!

### Schema Validation

**Hydra (with dataclass):**
```python
from dataclasses import dataclass
from hydra.core.config_store import ConfigStore

@dataclass
class ModelConfig:
    hidden_size: int = 256
    dropout: float = 0.1

cs = ConfigStore.instance()
cs.store(name="model_config", node=ModelConfig)
```

**Hyperparameter:**
```python
from dataclasses import dataclass
from hyperparameter import loader

@dataclass
class ModelConfig:
    hidden_size: int = 256
    dropout: float = 0.1

# Direct validation, no ConfigStore needed
cfg = loader.load("config.toml", schema=ModelConfig)
print(cfg.hidden_size)  # IDE autocomplete works!
```

### Command Line Overrides

**Hydra:**
```bash
python train.py model.hidden_size=512 model.dropout=0.2
```

**Hyperparameter:**
```bash
python train.py -D model.hidden_size=512 -D model.dropout=0.2
# Or with config file:
python train.py -C config.toml -D model.hidden_size=512
```

### Dynamic Scoping (Hyperparameter Exclusive)

This is something Hydra **cannot** do easily:

```python
from hyperparameter import auto_param, param_scope

@auto_param("layer")
def create_layer(dropout=0.1):
    return f"Layer with dropout={dropout}"

# Different dropout for different layers - no code change needed!
with param_scope(**{"layer.dropout": 0.1}):
    layer1 = create_layer()  # dropout=0.1

with param_scope(**{"layer.dropout": 0.5}):
    layer2 = create_layer()  # dropout=0.5
```

## Migration Checklist

- [ ] **Config Files**: Convert YAML to TOML/JSON (or keep YAML with PyYAML installed)
- [ ] **Decorators**: Replace `@hydra.main` with `@auto_param` + `launch()`
- [ ] **Config Access**: Replace `cfg.x.y` with `param_scope.x.y | default` or function injection
- [ ] **Composition**: Replace `defaults` list with `loader.load([file1, file2])`
- [ ] **Interpolation**: Same syntax `${key}` works
- [ ] **CLI**: Replace positional overrides with `-D key=value`

## What You'll Gain

1. **Performance**: 6x faster in dynamic access, 850x faster with injection
2. **Simplicity**: No ConfigStore, no `@hydra.main` boilerplate
3. **Flexibility**: Dynamic scoping for complex control flows
4. **Lightweight**: Fewer dependencies, faster startup

## What You'll Lose (For Now)

1. **Sweeper Plugins**: No built-in Optuna/Ax integration (but easy to implement manually)
2. **Launcher Plugins**: No SLURM/submitit integration
3. **Output Management**: No automatic `outputs/date/time` directories
4. **Tab Completion**: No shell autocomplete for config options

These features may be added in future versions based on community feedback.

## Need Help?

- [GitHub Issues](https://github.com/reiase/hyperparameter/issues)
- [Examples Directory](https://github.com/reiase/hyperparameter/tree/main/examples)

