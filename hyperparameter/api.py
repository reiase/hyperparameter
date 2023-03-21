import functools
import inspect
from typing import Any, Callable, Dict

from hyperparameter.storage import TLSKVStorage


class _DynamicDispatch:
    """Dynamic call dispatcher"""

    __slots__ = ("_func", "_name")

    def __get__(self):  # a trick that let doctest descover this class
        pass

    def __init__(self, func: Callable, name=None):
        self._func = func
        self._name = name

    def __call__(self, *args, **kws) -> Any:
        if self._name is None:
            return self._func(*args, **kws)
        return self._func().__getattr__(self._name)(*args, **kws)

    def __repr__(self) -> str:
        """
        >>> @_dynamic_dispatch
        ... def debug_print(*args, **kws):
        ...     return _HyperParameter(**kws)
        >>> debug_print.a.b.c # doctest: +ELLIPSIS
        <a.b.c in <...._HyperParameter object at ...>>
        """
        return repr(self._func().__getattr__(self._name))

    def __getattr__(self, name: str) -> Any:
        """
        >>> @_dynamic_dispatch
        ... def debug_print(*args, **kws):
        ...     return _HyperParameter(**kws)
        >>> debug_print.a.b = 1
        >>> debug_print().storage().storage()
        {}
        """
        if name.startswith("__") and name.endswith("__"):
            raise AttributeError
        if name == "pytest_mock_example_attribute_that_shouldnt_exist":
            raise AttributeError
        if name in self.__slots__:
            return self.__dict__[name]
        if hasattr(self._func, name):
            return getattr(self._func, name)
        if self._name is not None:
            self._name = f"{self._name}.{name}"
            return self
        return _dynamic_dispatch(self._func, name)

    def __setattr__(self, name: str, value: Any) -> None:
        """
        >>> hp = _HyperParameter()
        >>> @_dynamic_dispatch
        ... def debug_print():
        ...     return hp
        >>> debug_print.a.b.c = 1
        >>> hp.storage().storage()
        {'a.b.c': 1}
        """
        if name in self.__slots__:
            return self.__dict__.__setitem__(name, value)
        target = self._func().__getattr__(self._name)
        target.__setattr__(name, value)

    def __or__(self, default: Any) -> Any:
        return _ParamAccessor(self._func(), self._name) | default


def _dynamic_dispatch(func, name=None):
    """Wraps function with a class to allow __getattr__ on a function."""
    clz = type(func.__name__, (_DynamicDispatch, object), dict(__doc__=func.__doc__))
    return clz(func, name)


class _ParamAccessor:
    """Accessor that handles missing parameters and default values

    Examples
    ---------
    0. missing parameter
    >>> hp = _HyperParameter()
    >>> hp.param1 # doctest: +ELLIPSIS
    <param1 in <...._HyperParameter object at ...>

    1. default values for missing parameters
    >>> hp.param1(1)
    1

    2. missing parameter as boolen value
    >>> if not hp.param1:
    ...     print("hp.param1 is missing")
    hp.param1 is missing

    3. handle missing parameter with "|" operator
    >>> hp.param1 | 1
    1
    """

    def __init__(self, root, /, name=None):
        self._root = root
        self._name = name

    def get_or_else(self, default: Any = None):
        value = self._root.get(self._name)
        return default if isinstance(value, _ParamAccessor) else value

    def __getitem__(self, index: str) -> Any:
        return self.__getattr__(index)

    def __getattr__(self, name: str) -> Any:
        if name in ("_root", "_name"):
            return self.__dict__[name]
        name = f"{self._name}.{name}" if self._name else name
        return _ParamAccessor(self._root, name)

    def __setattr__(self, name: str, value: Any) -> None:
        if name in ("_root", "_name"):
            return self.__dict__.__setitem__(name, value)
        name = f"{self._name}.{name}" if self._name else name
        self._root.put(name, value)

    def __repr__(self) -> str:
        return f"<{self._name} in {repr(self._root)}>"

    def __bool__(self):
        return False

    def __call__(self, default: Any = None) -> Any:
        """shortcut for get_or_else"""
        return self.get_or_else(default)

    def __or__(self, default: Any) -> Any:
        return self.get_or_else(default)

    __nonzero__ = __bool__


class _HyperParameter:
    """Hyperparameter

    Examples
    --------
    >>> hp = _HyperParameter(param1=1, obj1={"prop1": "a"})
    >>> hp.param1()
    1
    >>> hp.obj1.prop1()
    'a'
    """

    def __init__(self, storage=None, /, **kws):
        if storage is None:
            storage = TLSKVStorage(None, _ParamAccessor)
        self.__dict__["_storage"] = storage
        self.update(kws)

    def update(self, kws: Dict[str, Any]) -> None:
        return self._storage.update(kws)

    def get(self, name: str) -> Any:
        return self._storage.get(name)

    def put(self, name: str, value: Any) -> None:
        return self._storage.put(name, value)

    def storage(self):
        return self._storage

    def keys(self):
        return self.storage().keys()

    def __getitem__(self, key: str) -> Any:
        """get parameter with dict-style api
        Examples
        ----------
        >>> hp = _HyperParameter(param1=1, obj1={"prop1": "a"})
        >>> hp["param1"]
        1
        >>> hp["obj1.prop1"]
        'a'
        """
        return self._storage.get(key)

    def __setitem__(self, key: str, value: Any) -> None:
        """set parameter with dict-style api
        Examples
        ----------
        >>> hp = _HyperParameter(param1=1, obj1={"prop1": "a"})
        >>> hp["param1"] = 2
        >>> hp["obj1.prop1"] = "b"
        >>> hp["param1"]
        2
        >>> hp["obj1.prop1"]
        'b'
        >>> hp["param2"] = 2
        >>> hp["obj2.prop2"] = "b"
        >>> hp["param2"]
        2
        >>> hp["obj2.prop2"]
        'b'
        """
        return self._storage.put(key, value)

    def __getattr__(self, name: str) -> Any:
        """get parameter with object-style api
        Examples
        --------
        >>> hp = _HyperParameter(param1=1, obj1={"prop1": "a"})
        >>> hp.param1()
        1
        >>> hp.obj1.prop1()
        'a'
        """
        if name in self.__dict__:
            return self.__dict__[name]
        return _ParamAccessor(self, name)

    def __setattr__(self, name: str, value: Any) -> None:
        """set parameter with object-style api
        Examples
        --------
        >>> hp = _HyperParameter()
        >>> hp.param1 = 1
        >>> hp.obj1.prop1 = 'a'
        >>> hp.storage().storage()
        {'param1': 1, 'obj1.prop1': 'a'}
        """
        if name in self.__dict__:
            return self.__dict__.__setitem__(name, value)
        self.put(name, value)

    def __iter__(self):
        return self._storage.__iter__()

    def __call__(self) -> Any:
        return _ParamAccessor(self)


@_dynamic_dispatch
class param_scope(_HyperParameter):
    """A thread-safe hyperparameter context scope

    Examples
    --------
    **create new `param_scope`**
    >>> ps = param_scope(a="a", b="b")           # create from call arguments
    >>> ps = param_scope(**{"a": "a", "b": "b"}) # create from a dict

    **read parameters from `param_scope`**
    >>> ps.a() # read parameter
    'a'
    >>> ps.c("c")  # read parameter with default value if missing
    'c'
    >>> ps.c | "c" # another way for reading missing parameters
    'c'

    **`param_scope` as a context scope**
    >>> with param_scope(**{"a": "a"}) as ps:
    ...     print(ps.a())
    a

    **read parameter from param_scope in a function**
    >>> def foo():
    ...    with param_scope() as ps:
    ...        return ps.a()
    >>> with param_scope(**{"a": "a", "b": "b"}) as ps:
    ...     foo() # foo should get param_scope using a with statement
    'a'

    **modify parameters in nested scopes**
    >>> with param_scope(**{'a': 1, 'b': 2}) as ps:
    ...     ps.storage().storage()
    ...     with param_scope(**{'b': 3}) as ps:
    ...         ps.storage().storage()
    ...     with param_scope() as ps:
    ...         ps.storage().storage()
    {'a': 1, 'b': 2}
    {'a': 1, 'b': 3}
    {'a': 1, 'b': 2}

    **use object-style parameter key in param_scope**
    >>> with param_scope(**{"a.b.c": [1,2]}) as ps:
    ...     ps.a.b.c()
    [1, 2]

    **access parameter with `param_scope`**
    >>> with param_scope(x=1):
    ...     param_scope.x(2) # read parameter
    ...     param_scope.y(2) # read a missing parameter with default value
    ...     param_scope.z | 3
    1
    2
    3

    **convert param_scope to dict**:
    >>> ps = param_scope(a=1, b=2)
    >>> dict(iter(ps))
    {'a': 1, 'b': 2}
    """

    def __init__(self, *args, **kwargs):
        super().__init__()
        self.update(kwargs)
        for line in args:
            if "=" in line:
                k, v = line.split("=", 1)
                self.put(k, v)

    def __enter__(self):
        """
        Examples
        --------
        >>> param_scope.p = "origin"
        >>> with param_scope(**{"p": "origin"}) as ps:
        ...     ps.storage().storage()      # outer scope
        ...     with param_scope() as ps:   # unmodified scope
        ...         ps.storage().storage()  # inner scope
        ...     with param_scope(**{"p": "modified"}) as ps: # modified scope
        ...         ps.storage().storage()  # inner scope with modified params
        ...     _ = param_scope(**{"p": "modified"}) # not used in with ctx
        ...     with param_scope() as ps:   # unmodified scope
        ...         ps.storage().storage()  # inner scope
        {'p': 'origin'}
        {'p': 'origin'}
        {'p': 'modified'}
        {'p': 'origin'}
        """

        self._storage.enter()
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self._storage.exit()

    def __call__(self) -> Any:
        return _ParamAccessor(self)

    @staticmethod
    def init(params=None):
        """init param_scope for a new thread."""
        param_scope(**params).__enter__()


_param_scope = param_scope._func


def auto_param(name_or_func):
    """Convert keyword arguments into hyperparameters

    Examples
    --------

    >>> @auto_param
    ... def foo(a, b=2, c='c', d=None):
    ...     print(a, b, c, d)

    >>> foo(1)
    1 2 c None

    >>> with param_scope('foo.b=3'):
    ...     foo(2)
    2 3 c None

    classes are also supported:
    >>> @auto_param
    ... class foo:
    ...     def __init__(self, a, b=2, c='c', d=None):
    ...         print(a, b, c, d)

    >>> obj = foo(1)
    1 2 c None

    >>> with param_scope('foo.b=3'):
    ...     obj = foo(2)
    2 3 c None

    >>> @auto_param('myns.foo.params')
    ... def foo(a, b=2, c='c', d=None):
    ...     print(a, b, c, d)
    >>> foo(1)
    1 2 c None

    >>> with param_scope('myns.foo.params.b=3'):
    ...     foo(2)
    2 3 c None
    """

    if callable(name_or_func):
        return auto_param(None)(name_or_func)

    def wrapper(func):
        predef_kws = {}
        predef_val = {}

        if name_or_func is None:
            namespace = func.__name__
        else:
            namespace = name_or_func

        signature = inspect.signature(func)
        for k, v in signature.parameters.items():
            if v.default != v.empty:
                name = "{}.{}".format(namespace, k)
                predef_kws[k] = name
                predef_val[name] = v.default

        @functools.wraps(func)
        def inner(*arg, **kws):
            with param_scope() as hp:
                local_params = {}
                for k, v in predef_kws.items():
                    if getattr(hp(), v).get_or_else(None) is not None and k not in kws:
                        kws[k] = getattr(hp(), v).get_or_else(None)
                        local_params[v] = hp.get(v)
                    else:
                        local_params[v] = predef_val[v]
                return func(*arg, **kws)

        return inner

    return wrapper
