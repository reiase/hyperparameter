# Hyperparameter

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

<p align="center">

**Hyperparameter, Make configurable AI applications. Build for Python/Rust hackers.**

</p>

超参数是一个多功能库，旨在简化机器学习算法和系统开发中超参数的管理和控制。专为AI研究人员与机器学习系统（MLSYS）开发者设计，超参数提供了一个统一的解决方案，侧重于在Python中易于使用、在Rust和C++中高性能访问，并提供了一组宏，以实现无缝超参数管理。

## 主要特性

### 针对Python用户

- **Pythonic语法：** 使用keyword参数语法定义超参数；
  
- **直观的作用域：** 通过`with`语句控制参数的作用域；
  
- **配置文件：** 从配置文件轻松加载参数；

### 针对Rust和C++用户

- **高性能后端：** 超参数在Rust中实现，提供了强大且高性能的超参数管理后端。在Rust和C++中以最小开销访问超参数，非常适合注重性能的ML和系统开发者。

- **参数管理宏：** 超参数为Rust和C++用户提供了一组宏。这些宏模仿了Python的`with`语句，并遵循特定于语言的作用域规则。

- **编译时哈希：** Rust和C++接口都利用了超参数名称的编译时哈希，降低了运行时哈希计算的开销。

## 快速开始

### 安装

```bash
pip install hyperparameter
```

### Python

```python
from hyperparameter import auto_param, param_scope

@auto_param("foo")
def foo(x=1, y="a"):
    return f"x={x}, y={y}"

foo()  # x=1, y='a'

with param_scope(**{"foo.x": 2}):
    foo()  # x=2, y='a'
```

### Rust

```rust
fn foo() -> i32 {
    with_params! {
        get x = foo.x or 1i32; // 读取带有默认值的超参数
        println!("x={}", x);
    }
}

fn main() {
    foo(); // x=1

    with_params! {
        set foo.x = 2i32; // 设置超参数
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
x = param_scope.foo.x | "default value"
```

#### Rust

```rust
get x = foo.x or "default value";
```

### 控制参数值的作用域

#### Python

```python
with param_scope() as ps: # 第1个作用域开始
    ps.foo.x=1
    with param_scope() as ps2: # 第2个作用域开始
        ps.foo.y=2
    # 第2个作用域结束
# 第1个作用域结束
```

#### Rust

```rust
with_params!{ // 第1个作用域开始
    set foo.x=1;

    with_params!{ //第2个作用域开始
        set foo.y=2

    } // 第2个作用域结束
} // 第1个作用域结束
```

### 线程隔离/线程安全

#### Python

```python
@auto_param("foo")
def foo(x=1): # 打印超参数 foo.x
    print(f"foo.x={x}")

with param_scope() as ps:
    ps.foo.x=2 # 在当前线程设置foo.x 

中修改 foo.x
    
    foo() # foo.x=2
    threading.Thread(target=foo).start() # foo.x=1，新线程的超参数值不受主线程的影响
```

#### Rust

```rust
fn foo() { // 打印超参数 foo.x
    with_params!{
        get x = foo.x or 1;

        println!("foo.x={}", x);
    }
}

fn main() {
    with_params!{
        set foo.x = 2; // 在当前线程中修改 foo.x
        
        foo(); // foo.x=2
        thread::spawn(foo); // foo.x=1，新线程的超参数值不受主线程的影响
    }
}
```

### 命令行应用

在命令行应用中，通常使用命令行参数（例如，`-D, --define`）定义超参数，并在命令行上控制超参数。以下是Python和Rust中的示例：

#### Python

```python
# example.py
from hyperparameter import param_scope, auto_param

@auto_param("example")
def main(a=0, b=1):
    print(f"example.a={a}, example.b={b}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()

    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with param_scope(*args.define):
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
        get a = example.a or 0;
        get b = example.b or 1;
        
        println!("example.a={}, example.b={}",a ,b);
    }
}
```

## 更多示例

### [parameter tunning for researchers](examples/sparse_lr/README.md)

该示例演示了如何在研究项目中使用超参数，并使实验可重现。

### [experiment tracing for data scientists](examples/mnist/README.md)

该示例展示了使用超参数进行实验管理，并通过mlflow.tracing进行结果追踪。