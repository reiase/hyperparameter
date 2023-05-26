Hyperparameter
===============

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

<p align="center">

**Hyperparameter, Make configurable AI applications.Build for Python hackers.**

</p>

快速开始
-------

`Hyperparameter` 使用装饰器 `auto _ param` 自动将keyword参数转化为超参：

```python
from hyperparameter import auto_param

@auto_param("foo")
def foo(x, y=1, z="a"):
    return f"x={x}, y={y}, z={z}"
```

超参数可以通过 `param_scope` 控制：

```python
from hyperparameter import param_scope

foo(1) # x=1, y=1, z='a'
with param_scope(**{"foo.y":2}):
    foo(1) # x=1, y=2, z='a'
```

高级用法
-------

### 读写超参

```python
from hyperparameter import param_scope

# 创建param_scope
with param_scope():
    pass

with param_scope("foo.y=1", "foo.z=b"):
    pass

with param_scope(**{"foo.y":1, "foo.z":2}):
    pass

# 读取超参（待默认值）
with param_scope(**{"foo.y":2}) as ps:
    y = ps.foo.y(1)  
    y = ps.foo.y | 1
    y = param_scope.foo.y(1)
    y = param_scope.foo.y | 1
    foo(1) # x=1, y=2, z='a'

# 写入超参
with param_scope(**{"foo.y":2}) as ps:
    ps.foo.y = 2
    param_scope.foo.y = 2
```

### 嵌套Scope

`Hyperparameter` support nested `param_scope`:

``` python
from hyperparameter import param_scope

# 当前没有param_scope，使用函数定义的默认值
foo(1) # x=1, y=1, z='a'

# 开启新的param_scope
# 并将`foo.y`默认值设为`2`
with param_scope(**{"foo.y":2}) as ps:
    # 发现一个param_scope `ps`, 
    # 并从`ps`中获取`foo.y`的默认值
    foo(1) # x=1, y=2, z='a'

    # 开启另一个新的param_scope
    # 并将`foo.y`默认值设为`3`
    with param_scope(**{"foo.z": "b"}) as ps2:
        # 发现一个嵌套param_scope `ps2`, 
        # 并从`ps2`中获取`foo.y`的默认值
        foo(1) # x=1, y=2, z='b'
    # `ps2` 结束，参数清理，`foo.y` 恢复为`2`
    foo(1) # x=1, y=2, z='a'
# `ps` 结束，参数清理，`foo.y` 恢复为`1`
foo(1) # x=1, y=1, z='a'
```

### CMD Line Arguments

CLI应用示例代码: 

```python
from hyperparameter import param_scope, auto_param

@auto_param
def main(a=0, b=1): # `inline default values`
    print(a, b)

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()

    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    with param_scope(*args.define):
        main()
```

更多示例
--------

### [超参搜索](examples/sparse_lr/README.md)

该示例展示如何在研究项目中使用hyperparameter，并让模型实验可以复现。
### [实验管理](examples/mnist/README.md)

该示例演示如何使用hyperparameter进行实验管理，并对接mlflow的tracing系统。
