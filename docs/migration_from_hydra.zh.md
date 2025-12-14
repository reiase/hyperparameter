# 从 Hydra 迁移指南

本指南帮助你将现有项目从 Hydra 迁移到 Hyperparameter。我们将介绍两者的关键差异，并提供对照示例。

## 为什么要迁移？

| 方面 | Hydra | Hyperparameter |
| :--- | :--- | :--- |
| **性能** | 纯 Python（循环中较慢） | Rust 后端（快 6-850 倍） |
| **依赖** | 重型（antlr4, omegaconf 等） | 极简（仅需 `toml`） |
| **配置风格** | 自上而下（到处传递 `cfg`） | 自下而上（注入到函数中） |
| **作用域** | 静态（启动时组装） | 动态（运行时可变） |

## 快速对比

### 定义参数

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
    print(hidden_size)  # 256 (来自配置文件)

if __name__ == "__main__":
    cfg = loader.load("config.toml")
    with param_scope(**cfg):
        build_model()
```

### 配置组合（多文件）

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

# 加载并合并多个配置（后面的文件覆盖前面的）
cfg = loader.load(["base.toml", "model/resnet.toml", "dataset/imagenet.toml"])

with param_scope(**cfg):
    train()
```

### 变量插值

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

两者都支持相同的 `${key}` 语法！

### Schema 校验

**Hydra (使用 dataclass):**
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

# 直接校验，无需 ConfigStore
cfg = loader.load("config.toml", schema=ModelConfig)
print(cfg.hidden_size)  # IDE 自动补全可用！
```

### 命令行覆盖

**Hydra:**
```bash
python train.py model.hidden_size=512 model.dropout=0.2
```

**Hyperparameter:**
```bash
python train.py -D model.hidden_size=512 -D model.dropout=0.2
# 或配合配置文件：
python train.py -C config.toml -D model.hidden_size=512
```

### 动态作用域（Hyperparameter 独有）

这是 Hydra **很难做到**的：

```python
from hyperparameter import auto_param, param_scope

@auto_param("layer")
def create_layer(dropout=0.1):
    return f"Layer with dropout={dropout}"

# 不同层使用不同的 dropout —— 无需修改代码！
with param_scope(**{"layer.dropout": 0.1}):
    layer1 = create_layer()  # dropout=0.1

with param_scope(**{"layer.dropout": 0.5}):
    layer2 = create_layer()  # dropout=0.5
```

## 迁移清单

- [ ] **配置文件**: 将 YAML 转换为 TOML/JSON（或安装 PyYAML 后继续使用 YAML）
- [ ] **装饰器**: 将 `@hydra.main` 替换为 `@auto_param` + `launch()`
- [ ] **配置访问**: 将 `cfg.x.y` 替换为 `param_scope.x.y | default` 或函数注入
- [ ] **配置组合**: 将 `defaults` 列表替换为 `loader.load([file1, file2])`
- [ ] **变量插值**: 相同的 `${key}` 语法可直接使用
- [ ] **命令行**: 将位置参数覆盖替换为 `-D key=value`

## 你将获得

1. **性能提升**: 动态访问快 6 倍，注入模式快 850 倍
2. **简洁性**: 无需 ConfigStore，无需 `@hydra.main` 样板代码
3. **灵活性**: 动态作用域，适应复杂控制流
4. **轻量级**: 更少依赖，更快启动

## 你将暂时失去

1. **Sweeper 插件**: 无内置 Optuna/Ax 集成（但可轻松手动实现）
2. **Launcher 插件**: 无 SLURM/submitit 集成
3. **输出管理**: 无自动 `outputs/date/time` 目录
4. **Tab 补全**: 无配置选项的 Shell 自动补全

这些功能可能会根据社区反馈在未来版本中添加。

## 需要帮助？

- [GitHub Issues](https://github.com/reiase/hyperparameter/issues)
- [示例目录](https://github.com/reiase/hyperparameter/tree/main/examples)

