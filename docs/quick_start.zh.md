快速开始
=======

`HyperParameter` 是一个配置参数管理框架，为Python应用提供超参配置与参数调优等功能。可通过如下命令快速安装：

```shell
pip install hyperparameter
```

主要特性：

1. `param_scope` 上下文，向Python应用提供线程安全的、可嵌套的参数管理上下文；提供对象化的树状参数管理，并支持默认值；

```python
>>> from hyperparameter import param_scope
>>> with param_scope(param1=1) as ps:
...     print(f"param1={ps.param1}, param2={ps.param2('undefined')}")
param1=1, param2=undefined

```

2. `auto_param` 装饰器，自动将函数（或者class）的默认参数转化为超参，并接受`param_scope`的参数控制；

```python
>>> from hyperparameter import auto_param, param_scope
>>> @auto_param
... def foo(a, b="default"):
...     print(f"a={a}, b={b}")

>>> foo(0)
a=0, b=default

>>> with param_scope(**{"foo.b": "modified"}):
...     foo(0)
a=0, b=modified

```

超参配置
-------

1. 通过`param_scope`可以直接读取超参配置，而无需任何配置：

```python
>>> from hyperparameter import param_scope
>>> def foo():
...     # read parameter from param_scope
...     p = param_scope.param("default")
...     p2 = param_scope.namespace.param2("default2")
...     print(f"p={p}, p2={p2}")

```

在上述函数`foo`中，尝试访问名为`param`的超参，超参默认值为`default`。`param_scope`首先尝试从上下文中读取同名参数并返回给调用者，若超参未定义则返回默认值。为了更好的组织参数，也可以给参数名添加命名空间`namespace.param`。命名空间也支持嵌套多层，比如`namespace.subspace.param`。


2. 通过`param_scope`传递超参

```python
# call `foo` with default parameter
>>> foo()
p=default, p2=default2

# call `foo` with modified parameter
>>> with param_scope("namespace.param2=modified"):
...     foo()
p=default, p2=modified

```

通过`with param_scope(...)`传递参数的时候支持两种语法，字符串语法与字典语法。字典语法如下所示：

```python
# call `foo` with modified parameter
>>> with param_scope(**{
...         "param": "modified",
...         "namespace": {"param2": "modified2"}
...     }):
...     foo()
p=modified, p2=modified2

```
字典语法适合配合配置文件使用。


3. `param_scope`可以穿透多层函数调用传递参数：

```python
>>> def bar():
...     foo()

# call `foo` within nested function call
>>> with param_scope("namespace.param2=modified"):
...     bar()
p=default, p2=modified

```

### manage hyper-parameters with `param_scope`

`param_scope` create a `HyperParameter` object for the context it manages:

```python
>>> from hyperparameter import param_scope
>>> with param_scope(param1=1, param2="A") as ps:
...     ps.param1
1

```

自动超参
-------

1. `auto_param` 可以自动为函数（或者class）添加超参配置功能

```python
>>> from hyperparameter import auto_param
>>> @auto_param
... def foo(param, param1=1):
...     print(f"param={param}, param1={param1}")

>>> foo(0)
param=0, param1=1

```

2. 通过`param_scope`向`auto_param`传递参数：

```python
>>> with param_scope(**{"foo.param1": 0}):
...     foo(0)
param=0, param1=0

```