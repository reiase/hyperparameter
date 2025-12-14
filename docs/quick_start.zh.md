# 快速开始

Hyperparameter 是一个配置参数管理库，提供线程安全的作用域控制、自动参数绑定和 CLI 集成。

```bash
pip install hyperparameter
```

---

## 1. 基础用法

### 1.1 参数读取与默认值

```python
import hyperparameter as hp

# 使用 | 运算符提供默认值
lr = hp.scope.train.lr | 0.001
batch_size = hp.scope.train.batch_size | 32

# 使用函数调用语法提供默认值（与 | 等价）
use_cache = hp.scope.model.cache(True)

# 不带参数调用：参数不存在时抛出 KeyError
required_value = hp.scope.model.required_key()  # KeyError if missing
```

`hp.scope.key(default)` 与 `hp.scope.key | default` 等价。不带参数调用 `hp.scope.key()` 表示该参数为必需项，缺失时抛出 `KeyError`。

### 1.2 作用域与自动回滚

```python
import hyperparameter as hp

print(hp.scope.model.dropout | 0.1)  # 0.1

with hp.scope(**{"model.dropout": 0.3}):
    print(hp.scope.model.dropout | 0.1)  # 0.3

print(hp.scope.model.dropout | 0.1)  # 0.1，作用域退出后自动回滚
```

`with` 语句退出时，该作用域内的所有参数修改自动撤销。

---

## 2. @hp.param 装饰器

### 2.1 函数参数自动绑定

```python
import hyperparameter as hp

@hp.param("train")
def train(lr=1e-3, batch_size=32, epochs=10):
    print(f"lr={lr}, batch_size={batch_size}, epochs={epochs}")

train()  # 使用函数签名中的默认值

with hp.scope(**{"train.lr": 5e-4, "train.batch_size": 64}):
    train()  # lr=0.0005, batch_size=64, epochs=10

train(lr=1e-2)  # 直接传参，优先级最高
```

参数解析优先级：直接传参 > scope 覆盖 > 函数签名默认值。

### 2.2 命令行覆盖

```python
# train.py
import hyperparameter as hp

@hp.param("train")
def train(lr=1e-3, batch_size=32, warmup_steps=500):
    print(f"lr={lr}, batch_size={batch_size}, warmup={warmup_steps}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with hp.scope(*args.define):
        train()
```

```bash
python train.py -D train.lr=5e-4 -D train.batch_size=64
```

通过 `-D key=value` 在运行时覆盖参数，无需修改代码。

---

## 3. 嵌套作用域

### 3.1 多模型对比实验

```python
import hyperparameter as hp

@hp.param("modelA")
def run_model_a(dropout=0.1, hidden=128):
    print(f"ModelA: dropout={dropout}, hidden={hidden}")

@hp.param("modelB")
def run_model_b(dropout=0.2, hidden=256):
    print(f"ModelB: dropout={dropout}, hidden={hidden}")

base = {"data.path": "/data/mnist"}
variants = [
    {"modelA.dropout": 0.3},
    {"modelB.hidden": 512, "modelB.dropout": 0.15},
]

with hp.scope(**base):
    for cfg in variants:
        with hp.scope(**cfg):
            run_model_a()
            run_model_b()
```

外层作用域设置公共配置，内层作用域覆盖特定参数，作用域之间相互隔离。

### 3.2 动态 key 访问

```python
import hyperparameter as hp

def train_task(task_name):
    lr = scope[f"task.{task_name}.lr"] | 1e-3
    wd = scope[f"task.{task_name}.weight_decay"] | 0.01
    print(f"{task_name}: lr={lr}, weight_decay={wd}")

with hp.scope(**{
    "task.cls.lr": 1e-3,
    "task.cls.weight_decay": 0.01,
    "task.seg.lr": 5e-4,
    "task.seg.weight_decay": 0.001,
}):
    train_task("cls")
    train_task("seg")
```

使用 `scope[key]` 语法支持动态构造的 key。

---

## 4. 线程安全

### 4.1 请求级隔离

```python
import hyperparameter as hp

def rerank(items):
    use_new = hp.scope.rerank.use_new(False)
    threshold = hp.scope.rerank.threshold | 0.8
    if use_new:
        return [x for x in items if x.score >= threshold]
    return items

def handle_request(request):
    with hp.scope(**request.overrides):
        return rerank(request.items)
```

每个请求在独立作用域中执行，配置修改不会影响其他并发请求。

### 4.2 多线程数据处理

```python
import concurrent.futures
import hyperparameter as hp

def preprocess(shard, cfg):
    with hp.scope(**cfg):
        clean = hp.scope.pre.clean_noise(False)
        norm = hp.scope.pre.norm | "zscore"
        # 处理逻辑
        return processed_shard

cfg = {"pre.clean_noise": True, "pre.norm": "minmax"}
shards = load_shards()

with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(lambda s: preprocess(s, cfg), shards))
```

线程安全保证：
- 配置字典可安全传递给多个线程
- 每个线程的 `scope` 修改相互隔离
- 作用域退出时自动清理

---

## 5. 典型应用场景

### 5.1 LLM 推理配置

```python
import hyperparameter as hp

def generate(prompt):
    max_tokens = hp.scope.llm.max_tokens | 256
    temperature = hp.scope.llm.temperature | 0.7
    return llm_call(prompt, max_tokens=max_tokens, temperature=temperature)

# 默认配置
generate("hello")

# 临时修改
with hp.scope(**{"llm.max_tokens": 64, "llm.temperature": 0.2}):
    generate("short answer")
```

### 5.2 A/B 测试

```python
import hyperparameter as hp

def get_experiment_config(user_id):
    if hash(user_id) % 100 < 10:  # 10% 流量
        return {"search.algo": "v2", "search.boost": 1.5}
    return {}

def search(query):
    algo = hp.scope.search.algo | "v1"
    boost = hp.scope.search.boost | 1.0
    # 搜索逻辑

def handle_request(user_id, query):
    with hp.scope(**get_experiment_config(user_id)):
        return search(query)
```

### 5.3 ETL 任务配置

```python
import hyperparameter as hp

def run_job(name, overrides=None):
    with hp.scope(**(overrides or {})):
        batch = hp.scope.etl.batch_size | 500
        retry = hp.scope.etl.retry | 3
        timeout = hp.scope.etl.timeout | 30
        # ETL 逻辑

run_job("daily")
run_job("full_rebuild", {"etl.batch_size": 5000, "etl.timeout": 300})
```

### 5.4 早停调度

```python
import hyperparameter as hp

def check_early_stop(metric, best, wait):
    patience = hp.scope.scheduler.patience | 5
    delta = hp.scope.scheduler.min_delta | 0.001
    
    if metric > best + delta:
        return False, metric, 0
    wait += 1
    return wait >= patience, best, wait
```

---

## 6. Rust 接口

### 6.1 基础用法

```rust
use hyperparameter::*;

fn train() {
    with_params! {
        get lr = train.lr or 0.001f64;
        get batch_size = train.batch_size or 32i64;
        println!("lr={}, batch_size={}", lr, batch_size);
    }
}

fn main() {
    train();  // 使用默认值

    with_params! {
        set train.lr = 0.0005f64;
        set train.batch_size = 64i64;
        train();
    };

    train();  // 回滚到默认值
}
```

### 6.2 线程隔离与 frozen()

```rust
use hyperparameter::*;
use std::thread;

fn worker(id: i64) {
    with_params! {
        set worker.id = id;
        for i in 0..5 {
            with_params! {
                set worker.iter = i;
                get wid = worker.id or -1i64;
                get witer = worker.iter or -1i64;
                println!("Worker {} iter {}", wid, witer);
            };
        }
    };
}

fn main() {
    with_params! {
        set global.seed = 42i64;
        frozen();  // 快照当前配置作为新线程的初始状态
    };

    let handles: Vec<_> = (0..4)
        .map(|id| thread::spawn(move || worker(id)))
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
```

`frozen()` 将当前配置快照为全局基线，新线程从该快照开始，后续修改线程间隔离。

---

## 7. API 速查

| 用法 | 说明 |
|------|------|
| `hp.scope.a.b \| default` | 读取参数，提供默认值 |
| `hp.scope.a.b(default)` | 同上，函数调用语法 |
| `hp.scope.a.b()` | 读取必需参数，缺失时抛出 KeyError |
| `scope["a.b"]` | 动态 key 访问 |
| `with hp.scope(**dict):` | 创建作用域，覆盖参数 |
| `with hp.scope(*list):` | 从字符串列表（如 CLI）创建作用域 |
| `@hp.param("ns")` | 装饰器，自动绑定函数参数到 `ns.*` |

---

## 8. 注意事项

1. **key 命名**：使用 `.` 分隔层级，如 `train.optimizer.lr`
2. **类型一致**：同一 key 的值应保持类型一致
3. **默认值**：始终提供默认值，避免 KeyError
4. **作用域范围**：尽量缩小作用域范围，避免不必要的参数暴露
5. **线程边界**：跨进程场景需使用 Rust 后端或自定义存储适配器
