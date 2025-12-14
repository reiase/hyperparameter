# API 参考文档

本文档提供 Hyperparameter Python API 的完整参考。

---

## param_scope

`param_scope` 是管理超参数的核心类，提供线程安全的作用域控制。

### 导入

```python
from hyperparameter import param_scope
```

### 创建 param_scope

```python
# 空作用域
ps = param_scope()

# 从关键字参数创建
ps = param_scope(lr=0.001, batch_size=32)

# 从字符串参数创建（key=value 格式）
ps = param_scope("lr=0.001", "batch_size=32")

# 从字典创建
ps = param_scope(**{"train.lr": 0.001, "train.batch_size": 32})

# 空作用域（清除继承的值）
ps = param_scope.empty()
ps = param_scope.empty(lr=0.001)
```

### 读取参数

```python
# 使用 | 运算符（缺失时返回默认值）
lr = param_scope.train.lr | 0.001

# 使用函数调用（缺失时返回默认值）
lr = param_scope.train.lr(0.001)

# 无默认值（缺失时抛出 KeyError）
lr = param_scope.train.lr()

# 动态 key 访问
key = "train.lr"
lr = param_scope[key] | 0.001
```

### 写入参数

```python
with param_scope() as ps:
    # 属性赋值
    param_scope.train.lr = 0.001
    
    # 通过实例
    ps.train.batch_size = 32
```

### 上下文管理器（with 语句）

```python
# 基本用法
with param_scope(**{"lr": 0.001}):
    print(param_scope.lr())  # 0.001

# 嵌套作用域
with param_scope(**{"a": 1}):
    print(param_scope.a())  # 1
    with param_scope(**{"a": 2}):
        print(param_scope.a())  # 2
    print(param_scope.a())  # 1（自动回滚）
```

### 静态方法

#### `param_scope.empty(*args, **kwargs)`

创建一个新的空作用域，清除所有继承的值。

```python
with param_scope(**{"inherited": 1}):
    with param_scope.empty(**{"fresh": 2}) as ps:
        print(ps.inherited("missing"))  # "missing"
        print(ps.fresh())  # 2
```

#### `param_scope.current()`

返回当前活动的作用域。

```python
with param_scope(**{"key": "value"}):
    ps = param_scope.current()
    print(ps.key())  # "value"
```

#### `param_scope.frozen()`

将当前作用域快照为新线程的全局基线。

```python
with param_scope(**{"global_config": 42}):
    param_scope.frozen()
    # 新线程将继承 global_config=42
```

#### `param_scope.init(params=None)`

为新线程初始化 param_scope。

```python
def thread_target():
    param_scope.init({"thread_param": 1})
    # ...
```

### 实例方法

#### `ps.keys()`

返回所有参数 key 的可迭代对象。

```python
with param_scope(**{"a": 1, "b.c": 2}) as ps:
    print(list(ps.keys()))  # ['a', 'b.c']
```

#### `ps.storage()`

返回底层存储对象。

#### `ps.update(dict)`

使用字典更新作用域。

#### `ps.clear()`

清除当前作用域中的所有参数。

---

## @auto_param

装饰器，自动将函数参数绑定到超参数。

### 导入

```python
from hyperparameter import auto_param
```

### 基本用法

```python
@auto_param("train")
def train(lr=0.001, batch_size=32, epochs=10):
    print(f"lr={lr}, batch_size={batch_size}")

# 使用函数默认值
train()  # lr=0.001, batch_size=32

# 通过 param_scope 覆盖
with param_scope(**{"train.lr": 0.01}):
    train()  # lr=0.01, batch_size=32

# 直接传参优先级最高
train(lr=0.1)  # lr=0.1, batch_size=32
```

### 自定义命名空间

```python
@auto_param("myapp.config.train")
def train(lr=0.001):
    print(f"lr={lr}")

with param_scope(**{"myapp.config.train.lr": 0.01}):
    train()  # lr=0.01
```

### 无命名空间（使用函数名）

```python
@auto_param
def my_function(x=1):
    return x

with param_scope(**{"my_function.x": 2}):
    my_function()  # 返回 2
```

### 类装饰器

```python
@auto_param("Model")
class Model:
    def __init__(self, hidden_size=256, dropout=0.1):
        self.hidden_size = hidden_size
        self.dropout = dropout

with param_scope(**{"Model.hidden_size": 512}):
    model = Model()  # hidden_size=512, dropout=0.1
```

### 参数解析优先级

1. **直接传参**（最高优先级）
2. **param_scope 覆盖**
3. **函数签名默认值**（最低优先级）

---

## launch

CLI 应用程序入口，支持自动参数解析。

### 导入

```python
from hyperparameter import launch
```

### 单函数模式

```python
@auto_param("app")
def main(input_file, output_file="out.txt", verbose=False):
    """处理输入文件。
    
    Args:
        input_file: 输入文件路径
        output_file: 输出文件路径
        verbose: 启用详细输出
    """
    pass

if __name__ == "__main__":
    launch(main)
```

运行：
```bash
python app.py input.txt --output_file result.txt --verbose
python app.py input.txt -D app.verbose=true
```

### 多函数模式（子命令）

```python
@auto_param("train")
def train(epochs=10, lr=0.001):
    """训练模型。"""
    pass

@auto_param("eval")
def evaluate(checkpoint="model.pt"):
    """评估模型。"""
    pass

if __name__ == "__main__":
    launch()  # 自动发现所有 @auto_param 函数
```

运行：
```bash
python app.py train --epochs 20
python app.py eval --checkpoint best.pt
```

### CLI 选项

| 选项 | 说明 |
|------|------|
| `-D, --define KEY=VALUE` | 覆盖超参数 |
| `-lps, --list-param-scope` | 列出所有注册的参数 |
| `-ep, --explain-param KEY` | 显示参数详情 |
| `-h, --help` | 显示帮助信息 |

---

## run_cli

`launch` 的替代方案，行为略有不同。

```python
from hyperparameter import run_cli

if __name__ == "__main__":
    run_cli()
```

---

## 类型转换

读取参数时，会根据默认值的类型自动进行类型转换。

### 布尔值转换

```python
with param_scope(**{"flag": "true"}):
    param_scope.flag(False)  # True

# 识别的真值: "true", "True", "TRUE", "t", "T", "yes", "YES", "y", "Y", "1", "on", "ON"
# 识别的假值: "false", "False", "FALSE", "f", "F", "no", "NO", "n", "N", "0", "off", "OFF"
```

### 整数转换

```python
with param_scope(**{"count": "42"}):
    param_scope.count(0)  # 42 (int)

with param_scope(**{"value": "3.14"}):
    param_scope.value(0)  # 3.14 (float，保留精度)
```

### 浮点数转换

```python
with param_scope(**{"rate": "0.001"}):
    param_scope.rate(0.0)  # 0.001
```

### 字符串转换

```python
with param_scope(**{"count": 42}):
    param_scope.count("0")  # "42" (string)
```

---

## 线程安全

### 线程隔离

每个线程有自己的参数作用域，一个线程的修改不会影响其他线程。

```python
import threading

def worker():
    with param_scope(**{"worker_id": threading.current_thread().name}):
        print(param_scope.worker_id())

threads = [threading.Thread(target=worker) for _ in range(3)]
for t in threads:
    t.start()
for t in threads:
    t.join()
```

### 传播到新线程

使用 `frozen()` 将值传播到新线程：

```python
with param_scope(**{"global_config": 42}):
    param_scope.frozen()

def worker():
    print(param_scope.global_config())  # 42

t = threading.Thread(target=worker)
t.start()
t.join()
```

---

## 错误处理

### KeyError

访问缺失的必需参数时抛出：

```python
with param_scope():
    param_scope.missing()  # 抛出 KeyError
```

### 安全访问

始终提供默认值以避免 KeyError：

```python
with param_scope():
    param_scope.missing | "default"  # 返回 "default"
    param_scope.missing("default")   # 返回 "default"
```

---

## 高级特性

### 嵌套字典展平

嵌套字典会自动展平：

```python
with param_scope(**{"model": {"hidden": 256, "layers": 4}}):
    print(param_scope["model.hidden"]())  # 256
    print(param_scope.model.layers())     # 4
```

### 动态 key 构造

```python
for task in ["train", "eval"]:
    key = f"config.{task}.batch_size"
    value = getattr(param_scope.config, task).batch_size | 32
```

### 访问底层存储

```python
with param_scope(**{"a": 1, "b": 2}) as ps:
    storage = ps.storage()
    print(storage.storage())  # {'a': 1, 'b': 2}
```

---

## Rust 接口

### with_params! 宏

```rust
use hyperparameter::*;

fn main() {
    with_params! {
        // 设置参数
        set train.lr = 0.001f64;
        set train.batch_size = 32i64;
        
        // 读取参数
        get lr = train.lr or 0.001f64;
        get batch_size = train.batch_size or 32i64;
        
        println!("lr={}, batch_size={}", lr, batch_size);
    };
}
```

### 参数设置

```rust
with_params! {
    set key = value;  // 设置参数
}
```

### 参数读取

```rust
with_params! {
    get var = key or default;  // 读取参数，提供默认值
}
```

### frozen()

```rust
with_params! {
    set global.config = 42i64;
    frozen();  // 快照为全局基线
};
```

### ParamScope

```rust
use hyperparameter::ParamScope;

let ps = ParamScope::from(&["key=value".to_string()]);
with_params! {
    params ps;
    // ...
};
```

---

## 存储后端

### Python 后端

纯 Python 实现，使用 `ContextVar` 实现线程安全。

### Rust 后端

高性能 Rust 实现，提供：
- 编译时 key 哈希
- 更快的参数访问
- 跨语言一致性

检查后端：

```python
from hyperparameter.storage import has_rust_backend
print(has_rust_backend)  # True/False
```

强制使用 Python 后端：

```bash
export HYPERPARAMETER_BACKEND=PYTHON
```
