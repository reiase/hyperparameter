Structured Parameter
====================

The main idea of `HyperParameter` is to organize the parameters and parameter accesses into a tree structure, by which we can refer to each parameter with a unique name and identify each access to the parameters. Then we can modify each parameter and control the access to the parameters.

Nested Parameters and Parameter Path
------------------------------------

The parameters are stored in a nested dict, a very common solution for python programs, and can be easily serialized into JSON or YAML format. For example:

```python
cfg = {
    "param1": 1, # unique name: param1
    "obj2": {
        "param3": "value4", # unique name: obj2.param3
        "param5": 6,        # unique name: obj2.param5
    },
}
```

We can directly refer to the value `value4` by `cfg["obj2"]["param3"]`. But we also need to check whether the parameter is missing from the cfg, and handle the default value.

`HyperParameter` offers a tiny DSL to access the parameters. The DSL syntax is very similar to `jsonpath`, but compatible with python syntax so that the interpreter and code editor can check for syntax errors. (I found this feature saves me a lot of time.). The first thing to use the DSL is converting the cfg into `HyperParameter`, and then we can use the DSL:

```python
# convert a nested dict into HyperParameter
hp = HyperParameter(**cfg) 

# or create the HyperParameter object from scratch
hp = HyperParameter(param1=1, obj2={"param3": "value4"})

# the DSL for access the parameter
param = hp().obj2.param3(default="undefined")
```

`hp().obj2.param3(default="undefined")` is the inline DSL for reading parameter from `HyperParameter` object. It looks like a `jsonpath` expression but has support for default values.

Best Practice for Structure Parameters with Parameter Path
-----------------------------------------------

### A Quick Example of Recommendation Model

Suppose we are building a wide&deep model with `keras`.

```python
class WideAndDeepModel(keras.Model):
    def __init__(self, units=30, activation="relu", **kwargs):
        super().__init__(**kwargs)
        self.hidden1 = keras.layers.Dense(units, activation=activation)
        self.hidden2 = keras.layers.Dense(units, activation=activation)
        self.main_output = keras.layers.Dense(1)
        self.aux_output = keras.layers.Dense(1)
        
    def call(self, inputs):
        input_A , input_B = inputs
        hidden1 = self.hidden1(input_B)
        hidden2 = self.hidden2(hidden1)
        concat = keras.layers.concatenate([input_A, hidden2])
        main_output = self.main_output()(concat)
        aux_output = self.aux_output()(hidden2)
        return main_outputi, aux_output
```

The model is straightforward and does not support many parameters. If we want to add batch normalization, dropout, and leaky-relu tricks to the model, we have to modify the code as follows:

```python
class WideAndDeepModel(keras.Model):
    def __init__(self, 
        units=[30, 30, 30],
        activation="relu",
        use_bn=False,
        bn_momentum=0.99,
        bn_epsilon=0.001,
        bn_center=True,
        bn_scale=True,
        bn_beta_initializer="zeros",
        bn_gamma_initializer="ones",
        bn_moving_mean_initializer="zeros",
        bn_moving_variance_initializer="ones",
        use_dropout=False,
        ...):

        ...
        self.bn1 = keras.layers.BatchNormalization(
            momentum=bn_momentum,
            epsilon=bn_epsilon,
            center=bn_center,
            scale=bn_scale,
            beta_initializer=bn_beta_initializer,
            gamma_initializer=bn_gamma_initializer,
            ...,
        )
```

The code becomes too complicated, having dozens of parameters to handle, most of which are not used.

### A Fast Trial of Structured Parameter

We can simplify the code with `auto_param`, which automatically converts the parameters into a parameter tree. And then, we can specify the parameters by name:

```python
# add parameter support for custom functions with a decorator
@auto_param("myns.rec.rank.dropout")
class dropout:
    def __init__(self, ratio=0.5):
        ...

# add parameter support for library functions 
wrapped_bn = auto_param("myns.rec.rank.bn")(keras.layers.BatchNormalization)
```

`myns.rec.rank` is the namespace for my project, and `myns.rec.rank.dropout` refers to the function defined in our code. We can refer to the keyword arguments (e.g. `ratio=0.5`) with the path `hp().myns.rec.rank.dropout`. 

After making the building block configurable, we can simplify the model:
```python
class WideAndDeepModel(keras.Model):
    def __init__(self, 
        units=[30, 30, 30],
        activation="relu",
        ...):

        ...
        self.bn1 = wrapped_bn()
        self.dropout1 = dropout()
```
And we can change the parameters of the `BN` layers with `param_scope`:

```python
with param_scope(**{
    "myns.rec.rank.dropout.ratio": 0.6,
    "myns.rec.rank.bn.center": False,
    ...
}):
    model = WideAndDeepModel()
```

Or read the parameters from a JSON file:

```python
with open("model.cfg.json") as f:
    cfg = json.load(f)
with param_scope(**cfg):
    model = WideAndDeepModel() 
```

### Fine-grained Control of Structured Parameters

In the last section, we have introduced how to structure the parameters with `auto_param` and modify them with `param_scope` by their path.
However, we may also need to access the same parameter in different places in our code, e.g., different layers in a DNN model.

In such situation, we can break our code into named scopes. And then, we can identify each access to the parameters and set a value for each access.

To add named scopes to our code, we can use `param_scope`:

```python
class WideAndDeepModel(keras.Model):
    def __init__(self, 
        units=[30, 30, 30],
        activation="relu",
        ...):

        ...
        with param_scope["layer1"]():
            self.bn1 = wrapped_bn()
            self.dropout1 = dropout()
        with param_scope["layer2"]():
            self.bn2 = wrapped_bn()
            self.dropout2 = dropout()
        ...

with param_scope["wdmodel"]():
    model = WideAndDeepModel()
```

`param_scope["layer1"]` creates a named scope called `layer1`. Since the scope is created inside another named scope `param_scope["wdmodel"]`, its full path should be `wdmodel.layer1`. We can specify different values of a parameter according to its path. For example:

```python
with param_scope["wdmodel"](**{
    "myns.rec.rank.dropout.ratio@wdmodel#layer1": 0.6,
    "myns.rec.rank.dropout.ratio@wdmodel#layer2": 0.7,
}):
    model = WideAndDeepModel()
```

With the above code, we get a drop ratio of 0.6 for `layer1` and 0.7 for `layer2`.