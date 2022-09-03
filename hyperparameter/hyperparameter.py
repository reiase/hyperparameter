import inspect
import threading
from typing import Any, Callable, Dict, Set


def _sorted_set(s):
    retval = list(s)
    retval.sort()
    return retval


class _Tracker:
    """parameter access tracker

    Examples
    --------
    >>> _tracker.clear()
    >>> ps = param_scope(a=1, b={'c': 2})
    >>> reads(), writes() # empty read/write trackers
    ([], [])

    Only access through accessor is tracked, raw access
    to the parameter is not tracked.
    >>> ps.a,  ps().b.c(3)
    (1, 2)
    >>> reads()
    ['b.c']

    >>> ps.a = 1
    >>> ps().a.b.c = 1
    >>> writes()
    ['a.b.c']
    """

    def __init__(self) -> None:
        self._get: Set[str] = set()
        self._put: Set[str] = set()

    def clear(self):
        self._get.clear()
        self._put.clear()

    def read(self, key: str = None) -> Set[str]:
        return self._get.add(key) if key else _sorted_set(self._get)

    def write(self, key: str = None) -> Set[str]:
        return self._put.add(key) if key else _sorted_set(self._put)

    def all(self):
        return _sorted_set(self._get.union(self._put))


_tracker = _Tracker()


def reads():
    return _tracker.read()


def writes():
    return _tracker.write()


def all_params():
    return _tracker.all()


class _Accessor(dict):
    """Helper for accessing hyper-parameters."""

    def __init__(self, root, path=None, suffix=None):
        super().__init__()
        self._root = root
        self._path = path
        self._suffix = suffix

    def get_or_else(self, default: Any = None):
        """Get value for the parameter, or get default value if the parameter is not defined."""
        if self._suffix is not None:
            suffixes = self._suffix.replace(".", "#").split("#")
            while suffixes:
                suffix = "#".join(suffixes)
                full_name = f"{self._path}@{suffix}"
                value = self._root.get(full_name)
                if not isinstance(value, _Accessor):
                    _tracker.read(full_name)
                    return value
                suffixes.pop()
        _tracker.read(self._path)
        value = self._root.get(self._path)
        return default if isinstance(value, _Accessor) else value

    def __getattr__(self, name: str) -> Any:
        # _path and _root are not allowed as keys for user.
        if name in ["_path", "_root", "_suffix"]:
            return self[name]
        return _Accessor(
            self._root,
            f"{self._path}.{name}" if self._path else name,
            suffix=self._suffix,
        )

    def __setattr__(self, name: str, value: Any):
        # _path and _root are not allowed as keys for user.
        if name in ["_path", "_root", "_suffix"]:
            return self.__setitem__(name, value)
        full_name = f"{self._path}.{name}" if self._path is not None else name
        full_name = (
            f"{full_name}@{self._suffix.replace('.', '#')}"
            if self._suffix is not None
            else full_name
        )
        _tracker.write(full_name)
        self._root.put(full_name, value)

    def __bool__(self):
        return False

    def __call__(self, default: Any = None) -> Any:
        """shortcut for get_or_else"""
        return self.get_or_else(default)

    __nonzero__ = __bool__


class DynamicDispatch:
    """Dynamic call dispatcher

    Examples
    --------

    >>> @dynamic_dispatch
    ... def debug_print(*args, **kws):
    ...     hp = param_scope()
    ...     name = hp._name
    ...     index = hp._index
    ...     return (name, index, args, kws)

    >>> debug_print()
    (None, None, (), {})

    >>> debug_print.a.b.c[1,2](3, 4, a=5, b=6)
    ('a.b.c', (1, 2), (3, 4), {'a': 5, 'b': 6})
    """

    def __init__(self, func: Callable, name=None, index=None):
        self._func = func
        self._name = name
        self._index = index

    def __call__(self, *args, **kws) -> Any:
        with _param_scope(_index=self._index, _name=self._name):
            return self._func(*args, **kws)

    def __getattr__(self, name: str) -> Any:
        if self._name is not None:
            name = f"{self._name}.{name}"
        return dynamic_dispatch(self._func, name, self._index)

    def __getitem__(self, index):
        return dynamic_dispatch(self._func, self._name, index)


def dynamic_dispatch(func, name=None, index=None):
    """Wraps function with a class to allow __getitem__ and __getattr__ on a function."""
    new_class = type(
        func.__name__,
        (
            DynamicDispatch,
            object,
        ),
        dict(__doc__=func.__doc__),
    )
    return new_class(func, name, index)


class HyperParameter(dict):
    """HyperParameter is an extended dict with features for better parameter management.

    **create and access hyper-parameters**
    ======================================

    Examples
    --------
    >>> hp = HyperParameter(param1=1, param2=2, obj1={'propA': 'A'})

    Once the HyperParameter object is created, you can access the values using the object-style api:
    >>> hp.param1
    1
    >>> hp.obj1.propA
    'A'

    or using the dict-style api (for legacy codes):
    >>> hp['param1']
    1
    >>> hp['obj1']['propA']
    'A'

    The object-style api also support creating or updating the parameters:
    >>> hp.a.b.c = 1

    **undefined parameters and default values**
    ===========================================

    Examples
    --------
    1. undefined parameter in `if` statement:
    >>> params = HyperParameter()
    >>> if not params.undefined_int: print("parameter undefined")
    parameter undefined

    2. default value for undefined parameter
    >>> params = HyperParameter()
    >>> params.undefined_int(10)
    10
    """

    def __init__(self, **kws):
        super().__init__()
        self.update(kws)

    def update(self, kws):
        for k, v in kws.items():
            if isinstance(v, dict):
                if k in self and isinstance(self[k], dict):
                    vv = HyperParameter(**self[k])
                    vv.update(v)
                    v = vv
                else:
                    v = HyperParameter(**v)
            self.put(k, v)

    def put(self, name: str, value: Any) -> None:
        """create or update the parameter with the given `name`

        Parameters
        ----------
        name : str
            parameter name, 'obj.prop' is supported
        value : Any
            parameter value

        Examples
        --------
        >>> cfg = HyperParameter()
        >>> cfg.put('param1', 1)
        >>> cfg.put('obj1.propA', 'A')

        >>> cfg.param1
        1

        >>> cfg.obj1.propA
        'A'
        """

        path = name.split(".")
        obj = self
        for p in path[:-1]:
            if p not in obj or (not isinstance(obj[p], dict)):
                obj[p] = HyperParameter()
            obj = obj[p]
        obj[path[-1]] = value

    def get(self, name: str) -> Any:
        """get the parameter with the given name

        Parameters
        ----------
        name : str
            parameter name

        Returns
        -------
        Any
            parameter value

        Examples
        --------
        >>> cfg = HyperParameter(a=1, b = {'c':2, 'd': 3})
        >>> cfg.get('a')
        1
        >>> cfg.get('b.c')
        2
        """

        path = name.split(".")
        obj = self
        for p in path[:-1]:
            if p not in obj:
                return _Accessor(obj, p)
            obj = obj[p]
        return obj[path[-1]] if path[-1] in obj else _Accessor(self, name)

    def __setitem__(self, key: str, value: Any) -> None:
        """set value and convert the value into `HyperParameter` if necessary

        Parameters
        ----------
        key : str
            parameter name
        value : Any
            parameter value
        """

        if isinstance(value, dict):
            return dict.__setitem__(self, key, HyperParameter(**value))
        return dict.__setitem__(self, key, value)

    def __getattr__(self, name: str) -> Any:
        """read parameter with object-style api

        Parameters
        ----------
        name : str
            parameter name

        Returns
        -------
        Any
            parameter value

        Examples
        --------
        for simple parameters:
        >>> hp = HyperParameter(a=1, b = {'c':2, 'd': 3})
        >>> hp.a
        1

        for nested parameters:
        >>> hp.b.c
        2

        >>> getattr(hp, 'b.c')
        2
        """
        return self.get(name)

    def __setattr__(self, name: str, value: Any) -> None:
        """create/update parameter with object-style api

        Parameters
        ----------
        name : str
            parameter name
        value : Any
            parameter value

        Examples
        --------
        >>> hp = HyperParameter(a=1, b = {'c':2, 'd': 3})
        >>> hp.e = 4

        >>> hp['e']
        4

        >>> setattr(hp, 'A.B.C', 1)
        >>> hp.A.B.C
        1
        """
        self.put(name, value)

    def __call__(self, suffix=None) -> Any:
        """Return a parameter accessor.

        Returns
        -------
        Any
            accessor of the current parameter

        Examples
        --------
        1. default parameter for undefined parameter
        >>> cfg = HyperParameter(a=1, b = {'c':2, 'd': 3})
        >>> cfg().a('default')   # default value for simple parameter
        1

        >>> cfg().b.c('default') # default value for nested parameter
        2

        >>> cfg().b.undefined('default')
        'default'

        2. hyper-parameter with suffix
        >>> cfg(suffix="ns").a("undefined")
        1

        >>> cfg(suffix="ns").a = 4
        >>> cfg
        {'a': 1, 'b': {'c': 2, 'd': 3}, 'a@ns': 4}

        >>> cfg("ns.ns").a()
        4
        """

        suffix = dict.get(self, "#suffix#", None) if suffix is None else suffix
        return _Accessor(self, suffix=suffix)


class _param_scope(HyperParameter):
    """A thread-safe context scope that manages hyperparameters

    Examples
    --------
    create a scoped HyperParameter
    >>> with param_scope(**{'a': 1, 'b': 2}) as ps:
    ...     print(ps.a)
    1

    read parameter in a function
    >>> def foo():
    ...    with param_scope() as ps:
    ...        return ps.a
    >>> with param_scope(**{'a': 1, 'b': 2}) as ps:
    ...     foo() # foo should get param_scope using a with statement
    1

    update some config only in new scope
    >>> with param_scope(**{'a': 1, 'b': 2}) as ps:
    ...     ps
    ...     with param_scope(**{'b': 3}) as ps:
    ...         ps
    ...     with param_scope() as ps:
    ...         ps
    {'a': 1, 'b': 2}
    {'a': 1, 'b': 3}
    {'a': 1, 'b': 2}

    set config with object-style key:
    >>> with param_scope(**{"a.b.c": [1,2]}) as ps:
    ...     ps.a.b.c
    [1, 2]
    """

    tls = threading.local()

    def __init__(self, *args, **kws):
        # Check if nested param_scope, if so, update current scope to include previous.
        if hasattr(_param_scope.tls, "history") and len(_param_scope.tls.history) > 0:
            self.update(_param_scope.tls.history[-1])
        self.update(kws)
        for line in args:
            if "=" in line:
                k, v = line.split("=", 1)
                self.put(k, v)

    def __enter__(self):
        if not hasattr(_param_scope.tls, "history"):
            _param_scope.tls.history = []
        _param_scope.tls.history.append(self)
        return _param_scope.tls.history[-1]

    def __exit__(self, exc_type, exc_value, traceback):
        _param_scope.tls.history.pop()

    @staticmethod
    def current():
        if not hasattr(_param_scope.tls, "history"):
            _param_scope.init()
        return _param_scope.tls.history[-1]

    @staticmethod
    def init(params=None):
        """init param_scope for a new thread."""
        _param_scope.tls.history = []
        _param_scope.tls.history.append(
            params if params is not None else HyperParameter()
        )


class _ParamScopeWrapper:
    def __init__(self, index=None):
        self._index = index

    def __call__(self, *args, **kwargs):
        retval = _param_scope(*args, **kwargs)
        if self._index is not None:
            retval._suffix = self._index
        return retval

    def __getitem__(self, index):
        return _ParamScopeWrapper(index)

    def current(self):
        return _param_scope.current()


param_scope = _ParamScopeWrapper()


"""
Tracker callback for auto_param
"""
_callback: Callable = None


def set_auto_param_callback(func: Callable[[Dict[str, Any]], None]):
    """report hyperparameter value to a tracker, for example, `mlflow.tracking`"""
    global _callback
    _callback = func


_auto_param_tracker: Set[str] = set()


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

    >>> defined = list(auto_param.defined)
    >>> defined.sort()
    >>> defined
    ['foo.b', 'foo.c', 'foo.d', 'myns.foo.params.b', 'myns.foo.params.c', 'myns.foo.params.d']
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
                _tracker.read(name)
                _auto_param_tracker.add(name)
                predef_val[name] = v.default

        def inner(*arg, **kws):
            with param_scope() as hp:
                local_params = {}
                for k, v in predef_kws.items():
                    if getattr(hp(), v).get_or_else(None) is not None and k not in kws:
                        kws[k] = hp.get(v)
                        local_params[v] = hp.get(v)
                    else:
                        local_params[v] = predef_val[v]
                if _callback is not None:
                    _callback(local_params)
                return func(*arg, **kws)

        return inner

    return wrapper


auto_param.defined = _auto_param_tracker
