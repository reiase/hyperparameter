<p align="center">
  <img src="hyperparameter.svg" alt="Hyperparameter Logo" width="180" height="180">
</p>

<h1 align="center">Hyperparameter</h1>

<p align="center">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
</p>

<p align="center">
  <strong>Make configurable AI applications. Build for Python/Rust hackers.</strong>
</p>

`Hyperparameter` 是一个多功能超参数管理库，旨在简化机器学习算法和系统开发中超参数的管理和控制。专为 AI 研究人员和机器学习系统（MLSYS）开发者设计，Hyperparameter 提供了一个统一的解决方案，侧重于在 Python 中易于使用、在 Rust 和 C++ 中高性能访问，并提供了一组宏以实现无缝超参数管理。

## 5分钟尝鲜

```bash
pip install hyperparameter

# 运行现成的演示
python -m hyperparameter.examples.quickstart

# 尝试 @hp.param CLI：从命令行覆盖默认值
python -m hyperparameter.examples.quickstart --define greet.name=Alice --enthusiasm=3

# 检查参数和默认值
python -m hyperparameter.examples.quickstart -lps
python -m hyperparameter.examples.quickstart -ep greet.name

# 从源码运行？使用模块模式或安装为可编辑模式
# python -m hyperparameter.examples.quickstart
# 或者: pip install -e .
```

## 为什么选择 Hyperparameter?

### 🚀 无与伦比的性能 (vs Hydra)

Hyperparameter 基于高性能 Rust 后端构建，使其比 Hydra 等纯 Python 替代方案快得多，特别是在内循环参数访问中。

| Method | Time (1M iters) | Speedup (vs Hydra) |
| :--- | :--- | :--- |
| **HP: Injected (Native Speed)** | **0.0184s** | **856.73x** 🚀 |
| **HP: Dynamic (Optimized)** | **2.4255s** | **6.50x** ⚡️ |
| **Hydra (Baseline)** | 15.7638s | 1.00x |

> 基准测试场景：在循环中访问嵌套参数 `model.layers.0.size` 1,000,000 次。
> 参见 `benchmark/` 文件夹获取复现脚本。

### ✨ 零依赖 Schema 校验

Hyperparameter 支持使用标准 Python 类型提示进行结构校验，无需引入重型依赖（如 Pydantic 或 OmegaConf）。

```python
from dataclasses import dataclass
import hyperparameter as hp

@dataclass
class AppConfig:
    host: str
    port: int
    debug: bool = False

# 自动校验类型并转换："8080" -> 8080 (int)
cfg = hp.config("config.toml", schema=AppConfig)
```

## 主要特性

### 针对 Python 用户

- **Pythonic 语法：** 使用关键字参数语法定义超参数；
  - **直观的作用域：** 通过 `with` 语句控制参数的作用域；
  - **强大的配置加载：** 支持 JSON/TOML/YAML 多文件组合加载 (Composition) 与变量插值 (Interpolation)；
  - **零开销校验：** 支持可选的基于 Python Type Hints 的 Schema 校验；
    
### 针对 Rust 和 C++ 用户

- **高性能后端：** Hyperparameter 在 Rust 中实现，提供了强大且高性能的超参数管理后端。在 Rust 和 C++ 中以最小开销访问超参数，非常适合注重性能的 ML 和系统开发者。

- **参数管理宏：** Hyperparameter 为 Rust 和 C++ 用户提供了一组宏。这些宏模仿了 Python 的 `with` 语句，并遵循特定于语言的作用域规则。

- **编译时哈希：** Rust 和 C++ 接口都利用了超参数名称的编译时哈希，降低了运行时哈希计算的开销。

## 快速开始

### 安装

```bash
pip install hyperparameter
```

### Python

```python
import hyperparameter as hp

@hp.param("foo")
def foo(x=1, y="a"):
    return f"x={x}, y={y}"

foo()  # x=1, y='a'

with hp.scope(**{"foo.x": 2}):
    foo()  # x=2, y='a'
```

### Rust

```rust
fn foo() -> i32 {
    with_params! {
        @get x = foo.x or 1i32; // 读取带有默认值的超参数

        println!("x={}", x);
    }
}

fn main() {
    foo(); // x=1

    with_params! {
        @set foo.x = 2i32; // 设置超参数

        foo(); // x=2
    }

    foo(); // x=1
}
```

### C++

```cpp
ASSERT(1 == GET_PARAM(a.b, 1), "get undefined param");
{
  auto guard = WITH_PARAMS(a, 1,        //
                            a.b, 2.0,    //
                            a.b.c, true, //
                            a.b.c.d, "str");
  ASSERT(1 == GET_PARAM(a, 0), "get int value");
  ASSERT(1 == GET_PARAM(a, 0), "get int value");
}
```

## 详细使用示例

### 参数默认值

#### Python

```python
x = hp.scope.foo.x | "default value"
```

#### Rust

```rust
@get x = foo.x or "default value";
```

### 控制参数值的作用域

#### Python

```python
with hp.scope() as ps: # 第1个作用域开始
    ps.foo.x=1
    with hp.scope() as ps2: # 第2个作用域开始
        ps.foo.y=2
    # 第2个作用域结束
# 第1个作用域结束
```

#### Rust

```rust
with_params!{ // 第1个作用域开始
    @set foo.x=1;

    with_params!{ //第2个作用域开始
        @set foo.y=2

        ...
    } // 第2个作用域结束
} // 第1个作用域结束
```

### 线程隔离/线程安全

#### Python

```python
@hp.param("foo")
def foo(x=1): # 打印超参数 foo.x
    print(f"foo.x={x}")

with hp.scope() as ps:
    ps.foo.x=2 # 在当前线程中修改 foo.x
    
    foo() # foo.x=2
    threading.Thread(target=foo).start() # foo.x=1，新线程的超参数值不受主线程的影响
```

#### Rust

```rust
fn foo() { // 打印超参数 foo.x
    with_params!{
        @get x = foo.x or 1;

        println!("foo.x={}", x);
    }
}

fn main() {
    with_params!{
        @set foo.x = 2; // 在当前线程中修改 foo.x
        
        foo(); // foo.x=2
        thread::spawn(foo); // foo.x=1，新线程的超参数值不受主线程的影响
    }
}
```

### 命令行应用

在命令行应用中，通常使用命令行参数（例如，`-D, --define`）定义超参数，并在命令行上控制超参数。以下是 Python 和 Rust 中的示例：

#### Python

```python
# example.py
import hyperparameter as hp

@hp.param("example")
def main(a=0, b=1):
    print(f"example.a={a}, example.b={b}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()

    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with hp.scope(*args.define):
        main()
```

#### Rust

```rust
// example.rs
use hyperparameter::*;
use hyperparameter_derive::Parser;

fn main() {
    #[derive(Parser, Debug)]
    struct DeriveArgs {
        #[arg(short = 'D', long)]
        define: Vec<String>,
    }

    let args = DeriveArgs::parse();

    with_params! {
        params ParamScope::from(&args.define);

        foo()
    }
}

fn foo() {
    with_params! {
        @get a = example.a or 0;
        @get b = example.b or 1;
        
        println!("example.a={}, example.b={}",a ,b);
    }
}
```

## 更多示例

### [parameter tuning for researchers](examples/sparse_lr/README.md)

该示例演示了如何在研究项目中使用超参数，并使实验可重现。

### [experiment tracing for data scientists](examples/mnist/README.md)

该示例展示了使用超参数进行实验管理，并通过 mlflow.tracing 进行结果追踪。

## 行为保证 (语义契约)

- **键与哈希 (Keys & hashing):** 键使用 `.` 进行嵌套，保留大小写，并且在 Python/Rust/C++ 中使用相同的 UTF-8 输入和种子进行哈希；非法字符将报错。
- **读取优先级 (Read precedence):** 当前线程的最内层作用域 > 父作用域向外 > 冻结的全局快照 > 用户默认值。写入仅影响当前作用域，并在退出时回滚。
- **默认值与缺失 (Defaults vs. missing):** 只有缺失的键才会回退到默认值；显式的 `None`/`False`/`0` 被视为存在的值。类型转换规则 (bool/int/float/str) 在各语言间保持一致；无效值使用尽力而为的转换，否则回退到提供的默认值（无静默随机值）。
- **线程与 `frozen()` (Threads & `frozen()`):** 每个线程从冻结的全局快照开始；变异保持在线程内，直到调用 `frozen()`，它原子地更新全局快照。全局变异在 Python 后端受到锁保护，与 Rust 语义匹配。
- **错误模型 (Error model):** 读取未定义的键且无默认值会引发键错误；后端加载失败会回退到 Python 后端，无嘈杂的回溯信息；类型错误无静默失败。
- **多进程注意 (Multiprocess notice):** 跨进程一致性需要共享后端（例如，Rust 后端或用户提供的存储适配器）；内置的 Python 后端仅保护线程，不保护进程。
