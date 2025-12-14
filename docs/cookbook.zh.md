# Cookbook: 常用配置方案

本 Cookbook 提供常见配置管理场景的实用解决方案。

## 目录

- [多环境配置 (dev/staging/prod)](#多环境配置)
- [配置继承](#配置继承)
- [密钥管理](#密钥管理)
- [特性开关](#特性开关)
- [A/B 测试配置](#ab-测试配置)
- [多阶段训练 (RL/ML)](#多阶段训练)

---

## 多环境配置

### 问题
你需要为开发、预发布和生产环境使用不同的配置。

### 解决方案

**目录结构:**
```
config/
├── base.toml          # 共享默认值
├── dev.toml           # 开发环境覆盖
├── staging.toml       # 预发布环境覆盖
└── prod.toml          # 生产环境覆盖
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
batch_size = 8  # 更小的 batch 加速迭代
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

**使用方式:**
```bash
ENV=dev python main.py      # batch_size=8
ENV=prod python main.py     # batch_size=256
```

---

## 配置继承

### 问题
你有多个模型变体，它们共享通用设置但在特定参数上有所不同。

### 解决方案

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
    """加载带继承的模型配置"""
    base_config = "config/models/base_transformer.toml"
    model_config = f"config/models/{model_name}.toml"
    return hp.config([base_config, model_config])

# 使用
cfg = load_model_config("bert_large")
# 结果: num_layers=24, hidden_size=1024, dropout=0.1 (继承自 base)
```

---

## 密钥管理

### 问题
你需要管理敏感值（API 密钥、密码），但不能提交到 git。

### 解决方案

**config/app.toml:**
```toml
[api]
base_url = "https://api.example.com"
timeout = 30

[database]
host = "${DATABASE_HOST}"      # 从环境变量读取
password = "${DATABASE_PASS}"
```

**config/secrets.local.toml** (已加入 gitignore):
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
    
    # 如果存在本地密钥文件则加载
    secrets_file = Path("config/secrets.local.toml")
    if secrets_file.exists():
        configs.append(str(secrets_file))
    
    cfg = hp.config(configs)
    return cfg

cfg = load_config_with_secrets()
```

---

## 特性开关

### 问题
你想在不修改代码的情况下启用/禁用功能。

### 解决方案

**config/features.toml:**
```toml
[features]
new_ui = false
experimental_model = false
debug_mode = true
rate_limiting = true

[features.ab_test]
enabled = true
variant = "control"  # "control" 或 "treatment"
```

**main.py:**
```python
import hyperparameter as hp

cfg = hp.config("config/features.toml")

with hp.scope(**cfg):
    # 在代码任何地方检查特性开关
    if hp.scope.features.new_ui | False:
        render_new_ui()
    else:
        render_old_ui()
    
    if hp.scope.features.experimental_model | False:
        model = ExperimentalModel()
    else:
        model = StableModel()
```

**通过命令行切换:**
```bash
python main.py -D features.new_ui=true
```

---

## A/B 测试配置

### 问题
你需要使用不同的参数配置运行实验。

### 解决方案

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
    # ... 训练代码 ...

def get_experiment_config(user_id: str):
    """基于 user_id 的确定性分配"""
    bucket = hash(user_id) % 100
    
    if bucket < 50:
        return {"experiment.model_type": "baseline"}
    else:
        return {"experiment.model_type": "new_model", "experiment.learning_rate": 0.0005}

# 使用
user_config = get_experiment_config("user_123")
with hp.scope(**user_config):
    run_experiment()
```

---

## 多阶段训练

### 问题
你的训练流水线有多个阶段，每个阶段需要不同的配置。

### 解决方案

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
    # 阶段 1: 高学习率预训练
    with hp.scope(**{"train.pretrain.lr": 0.001, "train.pretrain.epochs": 20}):
        pretrain()
    
    # 阶段 2: 低学习率微调
    with hp.scope(**{"train.finetune.lr": 0.00005}):
        finetune()
    
    # 阶段 3: RL 训练，探索率递减
    for stage, exploration in enumerate([0.5, 0.3, 0.1, 0.05]):
        with hp.scope(**{"train.rl.exploration": exploration}):
            print(f"--- RL Stage {stage + 1} ---")
            rl_train()

if __name__ == "__main__":
    run_pipeline()
```

这展示了 Hydra 难以实现的**动态作用域**特性。

---

## 最佳实践

### 1. 使用类型提示获得更好的 IDE 支持

```python
from dataclasses import dataclass
import hyperparameter as hp

@dataclass
class ModelConfig:
    hidden_size: int = 256
    dropout: float = 0.1
    activation: str = "relu"

cfg: ModelConfig = hp.config("config.toml", schema=ModelConfig)
# 现在 cfg.hidden_size 有自动补全了！
```

### 2. 按关注点组织配置

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

### 3. 使用插值避免重复 (DRY)

```toml
[paths]
root = "/project"
data = "${paths.root}/data"
models = "${paths.root}/models"
logs = "${paths.root}/logs"
```

### 4. 为配置文件添加注释

```toml
# BERT 变体的模型配置
# 参考: https://arxiv.org/abs/1810.04805

[model]
# Transformer 层数 (base 为 12，large 为 24)
num_layers = 12

# 隐藏维度 (base 为 768，large 为 1024)
hidden_size = 768
```

