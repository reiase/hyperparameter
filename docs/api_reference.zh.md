# API 参考文档

本文档提供 Hyperparameter Python API 的完整参考。

---

## scope

`scope` 是管理超参数的核心类，提供线程安全的作用域控制。

### 导入

```python
import hyperparameter as hp
```

### 创建 scope

```python
# 空作用域
ps = hp.scope()

# 从关键字参数创建
ps = hp.scope(lr=0.001, batch_size=32)

# 从字符串参数创建（key=value 格式）
ps = hp.scope("lr=0.001", "batch_size=32")

# 从字典创建
ps = hp.scope(**{"train.lr": 0.001, "train.batch_size": 32})

# 空作用域（清除继承的值）
ps = hp.scope.empty()
ps = hp.scope.empty(lr=0.001)
```

### 读取参数

```python
# 使用 | 运算符（缺失时返回默认值）
lr = hp.scope.train.lr | 0.001

# 使用函数调用（缺失时返回默认值）
lr = hp.scope.train.lr(0.001)

# 无默认值（缺失时抛出 KeyError）
lr = hp.scope.train.lr()

# 动态 key 访问
key = "train.lr"
lr = scope[key] | 0.001
```

### 写入参数

```python
with hp.scope() as ps:
    # 属性赋值
    hp.scope.train.lr = 0.001
    
    # 通过实例
    ps.train.batch_size = 32
```

### 上下文管理器（with 语句）

```python
# 基本用法
with hp.scope(**{"lr": 0.001}):
    print(hp.scope.lr())  # 0.001

# 嵌套作用域
with hp.scope(**{"a": 1}):
    print(hp.scope.a())  # 1
    with hp.scope(**{"a": 2}):
        print(hp.scope.a())  # 2
    print(hp.scope.a())  # 1（自动回滚）
```

### 静态方法

#### `hp.scope.empty(*args, **kwargs)`

创建一个新的空作用域，清除所有继承的值。

```python
with hp.scope(**{"inherited": 1}):
    with hp.scope.empty(**{"fresh": 2}) as ps:
        print(ps.inherited("missing"))  # "missing"
        print(ps.fresh())  # 2
```

#### `hp.scope.current()`

返回当前活动的作用域。

```python
with hp.scope(**{"key": "value"}):
    ps = hp.scope.current()
    print(ps.key())  # "value"
```

#### `hp.scope.frozen()`

将当前作用域快照为新线程的全局基线。

```python
with hp.scope(**{"global_config": 42}):
    hp.scope.frozen()
    # 新线程将继承 global_config=42
```

#### `hp.scope.init(params=None)`

为新线程初始化 scope。

```python
def thread_target():
    hp.scope.init({"thread_param": 1})
    # ...
```

### 实例方法

#### `ps.keys()`

返回所有参数 key 的可迭代对象。

```python
with hp.scope(**{"a": 1, "b.c": 2}) as ps:
    print(list(ps.keys()))  # ['a', 'b.c']
```

#### `ps.storage()`

返回底层存储对象。

#### `ps.update(dict)`

使用字典更新作用域。

#### `ps.clear()`

清除当前作用域中的所有参数。

---

## @hp.param

装饰器，自动将函数参数绑定到超参数。

### 导入

```python
import hyperparameter as hp
```

### 基本用法

```python
@hp.param("train")
def train(lr=0.001, batch_size=32, epochs=10):
    print(f"lr={lr}, batch_size={batch_size}")

# 使用函数默认值
train()  # lr=0.001, batch_size=32

# 通过 scope 覆盖
with hp.scope(**{"train.lr": 0.01}):
    train()  # lr=0.01, batch_size=32

# 直接传参优先级最高
train(lr=0.1)  # lr=0.1, batch_size=32
```

### 自定义命名空间

```python
@hp.param("myapp.config.train")
def train(lr=0.001):
    print(f"lr={lr}")

with hp.scope(**{"myapp.config.train.lr": 0.01}):
    train()  # lr=0.01
```

### 无命名空间（使用函数名）

```python
@hp.param
def my_function(x=1):
    return x

with hp.scope(**{"my_function.x": 2}):
    my_function()  # 返回 2
```

### 类装饰器

```python
@hp.param("Model")
class Model:
    def __init__(self, hidden_size=256, dropout=0.1):
        self.hidden_size = hidden_size
        self.dropout = dropout

with hp.scope(**{"Model.hidden_size": 512}):
    model = Model()  # hidden_size=512, dropout=0.1
```

### 参数解析优先级

1. **直接传参**（最高优先级）
2. **scope 覆盖**
3. **函数签名默认值**（最低优先级）

---

## launch

CLI 应用程序入口，支持自动参数解析。

### 导入

```python
import hyperparameter as hp
```

### 单函数模式

```python
@hp.param("app")
def main(input_file, output_file="out.txt", verbose=False):
    """处理输入文件。
    
    Args:
        input_file: 输入文件路径
        output_file: 输出文件路径
        verbose: 启用详细输出
    """
    pass

if __name__ == "__main__":
    hp.launch(main)
```

运行：
```bash
python app.py input.txt --output_file result.txt --verbose
python app.py input.txt -D app.verbose=true
```

### 多函数模式（子命令）

```python
@hp.param("train")
def train(epochs=10, lr=0.001):
    """训练模型。"""
    pass

@hp.param("eval")
def evaluate(checkpoint="model.pt"):
    """评估模型。"""
    pass

if __name__ == "__main__":
    hp.launch()  # 自动发现所有 @hp.param 函数
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

## launch

`launch` 的替代方案，行为略有不同。

```python
import hyperparameter as hp

if __name__ == "__main__":
    hp.launch()
```

---

## 类型转换

读取参数时，会根据默认值的类型自动进行类型转换。

### 布尔值转换

```python
with hp.scope(**{"flag": "true"}):
    hp.scope.flag(False)  # True

# 识别的真值: "true", "True", "TRUE", "t", "T", "yes", "YES", "y", "Y", "1", "on", "ON"
# 识别的假值: "false", "False", "FALSE", "f", "F", "no", "NO", "n", "N", "0", "off", "OFF"
```

### 整数转换

```python
with hp.scope(**{"count": "42"}):
    hp.scope.count(0)  # 42 (int)

with hp.scope(**{"value": "3.14"}):
    hp.scope.value(0)  # 3.14 (float，保留精度)
```

### 浮点数转换

```python
with hp.scope(**{"rate": "0.001"}):
    hp.scope.rate(0.0)  # 0.001
```

### 字符串转换

```python
with hp.scope(**{"count": 42}):
    hp.scope.count("0")  # "42" (string)
```

---

## 线程安全

### 线程隔离

每个线程有自己的参数作用域，一个线程的修改不会影响其他线程。

```python
import threading

def worker():
    with hp.scope(**{"worker_id": threading.current_thread().name}):
        print(hp.scope.worker_id())

threads = [threading.Thread(target=worker) for _ in range(3)]
for t in threads:
    t.start()
for t in threads:
    t.join()
```

### 传播到新线程

使用 `frozen()` 将值传播到新线程：

```python
with hp.scope(**{"global_config": 42}):
    hp.scope.frozen()

def worker():
    print(hp.scope.global_config())  # 42

t = threading.Thread(target=worker)
t.start()
t.join()
```

---

## 错误处理

### KeyError

访问缺失的必需参数时抛出：

```python
with hp.scope():
    hp.scope.missing()  # 抛出 KeyError
```

### 安全访问

始终提供默认值以避免 KeyError：

```python
with hp.scope():
    hp.scope.missing | "default"  # 返回 "default"
    hp.scope.missing("default")   # 返回 "default"
```

---

## 高级特性

### 嵌套字典展平

嵌套字典会自动展平：

```python
with hp.scope(**{"model": {"hidden": 256, "layers": 4}}):
    print(scope["model.hidden"]())  # 256
    print(hp.scope.model.layers())     # 4
```

### 动态 key 构造

```python
for task in ["train", "eval"]:
    key = f"config.{task}.batch_size"
    value = getattr(hp.scope.config, task).batch_size | 32
```

### 访问底层存储

```python
with hp.scope(**{"a": 1, "b": 2}) as ps:
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

---

## 命令行工具：hp

Hyperparameter 提供 `hp` 命令行工具，用于分析 Python 包中的超参数使用情况。

### 安装

安装 hyperparameter 后，`hp` 命令即可使用：

```bash
pip install hyperparameter
hp --help
```

### 命令

#### hp list / hp ls

列出超参数：

```bash
# 列出所有使用 hyperparameter 的包
hp ls
hp list

# 列出包中的超参数
hp ls mypackage

# 树状显示
hp ls mypackage --tree
hp ls mypackage -t

# 范围选项
hp ls mypackage --self       # 仅自身（默认）
hp ls mypackage --all        # 包含依赖
hp ls mypackage --deps       # 仅依赖

# 输出格式
hp ls mypackage -f text      # 默认文本格式
hp ls mypackage -f markdown  # Markdown 格式
hp ls mypackage -f json      # JSON 格式

# 保存到文件
hp ls mypackage -o report.md -f markdown
```

#### 包发现

不带参数运行 `hp ls` 时，会扫描所有已安装的包：

```
Packages using hyperparameter (3):
============================================================
Package                        Version      Params   Funcs
------------------------------------------------------------
myapp                          1.0.0        15       5
ml-toolkit                     0.2.1        8        3
config-manager                 2.1.0        4        2
------------------------------------------------------------

Use 'hp ls <package>' to see hyperparameters in a package.
```

#### hp describe / hp desc

查看超参数详情：

```bash
# 精确匹配
hp desc train.lr mypackage

# 模糊搜索
hp desc lr mypackage

# 默认当前目录
hp desc train.lr
```

### 示例输出

#### 列表（树状视图）

```
Hyperparameters in myapp:
----------------------------------------
📁 train
  📄 lr = 0.001
  📄 batch_size = 32
  📄 epochs = 10
📁 model
  📄 hidden_size = 256
  📄 dropout = 0.1

Total: 5 hyperparameters
```

#### 描述

```
============================================================
Hyperparameter: train.lr
============================================================

  Default: 0.001
  Type: float
  Namespace: train
  Function: train

  Source: myapp
  Location: train.py:15

  Description: Training function with configurable learning rate.

  Usage:
    # 通过 scope 访问
    value = hp.scope.train.lr | <default>
    
    # 通过命令行设置
    --train.lr=<value>
```

### 使用场景

1. **项目审计**：快速了解项目中所有可配置的超参数
2. **文档生成**：自动生成超参数文档
3. **依赖分析**：发现依赖库中的超参数，统一管理
4. **代码审查**：检查超参数使用是否规范
