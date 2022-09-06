**H**_yper_**P**_arameter_
===========================

<h3 align="center">
  <p style="text-align: center;">
  <a href="README.md" target="_blank">ENGLISH</a> | <a href="README.zh.md">中文文档</a>
  </p>
</h3>

HyperParameter 是轻量级python代码配置框架，用户仅需对代码添加标记即可实现配置化。特别适用于机器学习模型的参数管理，帮助开发者对接MLOps工具；也适用于大型Python应用的配置管理。

### 一个示例

假设我们开发了一个MLP结构，并使用在某个模型中：

```python
class MLP(nn.Module):
  def __init__(self, units=[64, 32, 16], activation="ReLU", use_bn=False):
    ...

class MyAwesomeModel(nn.Module):
    def __init__(self):
        self.mlp = MLP()
        ...

model = MyAwesomeModel()
```

如果我们想尝试修改MLP结构的参数来提升模型效果，通常要修改其代码，或者在MyAwesomeModel中添加额外参数。前者代码难以维护，而后者需要大量冗余代码。

HyperParameter 通过 `auto_param`装饰器自动抽取配置参数：

```python
from hyperparameter import auto_param

@auto_param
class MLP(nn.Module):
  def __init__(self, units=[64, 32, 16], activation="ReLU", use_bn=False):
    ...
```

上述代码将自动生成三个参数，分别对应`MLP.__init__`方法的keyword参数：

1. `MLP.units`，默认值`[64, 32, 16]`;
2. `MLP.activation`，默认值`ReLU`;
3. `MLP.use_bn`，默认值`False`;

三个参数与keyword参数同名，并被放在`MLP`这个namespace下。参数值通过`param_scope`来控制：

```python
from hyperparameter import param_scope

with param_scope(**{"MLP.activation": "sigmoid"}):
    model = MyAwesomeModel()
```

高级用法
-------
### 嵌套scope

我们可以通过嵌套`param_scope`来精细控制参数，当退出一个`param_scope`，该scope对参数的修改自动失效:

``` python
from hyperparameter import auto_param, param_scope

@auto_param
def foo(a=1, b=2):
    print(f"a={a}, b={b}")

with param_scope(a=3):
    foo() # a=3, b=2!
    with param_scope(a=2, b=3):
        foo() # a=2, b=3!
    foo() # a=3, b=2!
```

### 从命令行管理参数

我们通常推荐使用如下三层结构管理参数:

1. 代码的inline默认值，即写在函数或者类定义中的默认值;
2. 配置文件，会覆盖inline默认值;
3. 命令行参数，会覆盖配置文件和inline默认值;

我们推荐的命令脚本示例如下：

```python
from hyperparameter import param_scope, auto_param

@auto_param
def main(a=0, b=1): # inline默认值
    print(a, b)

if __name__ == "__main__":
    import argparse
    import json
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=str, default=None)
    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    if args.config is not None: # 加载配置文件
        with open(args.config) as f:
            cfg = json.load(f)
    else:
        cfg = {}

    with param_scope(**cfg): # 配置文件的scope
        with param_scope(*args.define): # 命令行参数的scope
            main()
```

更多示例
-------

### [超参调节](examples/sparse_lr/README.md)

该示例展示了如何使用`hyperparameter`来搭建机器学习项目，并保证复现性。

### [试验跟踪](examples/mnist/README.md)

该例子说明了如何通过`hyperparameter`进行试验管理，并跟踪试验结果。
