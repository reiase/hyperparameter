Hyperparameter
===============

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

<p align="center">

**构建易于配置的机器学习应用与系统**

</p>

`Hyperparameter` 是一个Python与Rust的超参数管理库，
典型应用场景是：
- 控制模型训练过程的超参数；
- 控制机器学习系统的底层行为；

快速开始
-------

### 在Python中使用`Hyperparameter`

首先，借助`auto _ param`装饰器，使用keyword参数语法定义超参数：

```python
from hyperparameter import auto_param

@auto_param("foo")
def foo(
      x=1,  # 定义超参`foo.y`=1 
      y="a" # 定义超参`foo.z`="a"
    ): 
    return f"x={x}, y={y}, z={z}"
```

之后，可以通过`param_scope`上下文控制超参数的取值：

```python
from hyperparameter import param_scope

foo() # x=1, y='a'
with param_scope(**{"foo.x":2}):
    foo() # x=2, y='a'
```

### 在Rust中使用`Hyperparameter`

通过`with_params!`宏创建参数scope，并读写超参数

```rust
fn foo() -> i32{
    with_params! { // 创建scope
        get x = foo.x or 1i32; // 读取超参 foo.x，默认值为1

        println!("x={}", x);
    } // scope结束
}

fn main() {
    foo(); // x=1，超参foo.x=1
    with_params! {// 创建scope
        set foo.x = 2i32; // 设置超参 foo.x = 2

        foo(); // x = 2，超参foo.x=2
    }// scope结束, scope内超参foo.x=2失效
    foo(); // x=1，超参foo.x=1
}
```

主要特性
-------

- 支持超参默认值；

    ```python
    # python
    x = param_scope.foo.x | "default value"
    ```
    ```rust
    // rust
    get x = foo.x or "default value";
    ```

- scope控制参数取值：
    ```python
    # python
    with param_scope() as ps: # 1st scope start
        ps.foo.x=1
        with param_scope() as ps2: # 2nd scope start
            ps.foo.y=2
        # 2nd scope end
    # 1st scope end
    ```
    ```rust
    // rust
    with_params!{ // 1st scope start
        set foo.x=1;

        with_params!{ //2nd scope start
            set foo.y=2

        } // 2nd scope end
    } // 1st scope end
    ```

- 线程隔离/线程安全

    ```python
    # python
    @auto_param("foo")
    def foo(x=1): # 打印超参foo.x
        print(f"foo.x={x}")
    
    with param_scope() as ps:
        ps.foo.x=2 # 修改当前线程的foo.x
        
        foo() # foo.x=2
        threading.Thread(target=foo).start() # foo.x=1，新线程中超参取值不受主线程影响
    ```
    ```rust
    // rust
    fn foo() { // 打印超参foo.x
        with_params!{
            get x = foo.x or 1;

            println!("foo.x={}", x);
        }
    }

    fn main() {
        with_params!{
            set foo.x = 2; // 修改当前线程的foo.x
            
            foo(); // foo.x=2
            thread::spawn(foo); // foo.x=1，新线程中超参取值不受主线程影响
        }
    }
    ```

构建命令行应用
------------

在命令行应用中的常见用法是添加一个用来定义超参的命令行参数（比如：
`-D, --define`，用来定义命名超参的取值，可重复多次），并通过如下方式在命令行控制超参数：
```bash
./example \
    ... 
    -D example.a=1 \
    -D example.b=2 \
    ...
```

以下为`-D,--define`参数在Python与Rust中的实现方法：

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

```rust
//rust
fn foo() {
    with_params! {
        get a = example.a or 0;
        get b = example.b or 1;
        
        println!("example.a={}, example.b={}",a ,b);
    }
}

#[derive(Parser, Debug)]
struct DeriveArgs {
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

fn main() {
    let args = DeriveArgs::parse();
    with_params! {
        params ParamScope::from(&args.define);

        foo()
    }
}

```

更多示例
--------

### [超参搜索](examples/sparse_lr/README.md)

该示例展示如何在研究项目中使用hyperparameter，并让模型实验可以复现。
### [实验管理](examples/mnist/README.md)

该示例演示如何使用hyperparameter进行实验管理，并对接mlflow的tracing系统。
