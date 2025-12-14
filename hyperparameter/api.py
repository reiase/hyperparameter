from __future__ import annotations

import functools
import inspect
from typing import Any, Callable, Dict, Optional, Union, TypeVar, overload

from hyperparameter.storage import TLSKVStorage, has_rust_backend, xxh64

from .tune import Suggester

T = TypeVar("T")
_MISSING = object()


def _repr_dict(d: Dict[str, Any]) -> list:
    """Helper function to represent dictionary as sorted list of tuples."""
    d = [(k, v) for k, v in d.items()]
    d.sort()
    return d


class _DynamicDispatch:
    """Dynamic call dispatcher"""

    __slots__ = ("_func", "_name")

    def __get__(self) -> None:  # a trick that let doctest discover this class
        pass

    def __init__(self, func: Callable, name: Optional[str] = None) -> None:
        self._func = func
        self._name = name

    def __current__(self) -> Any:
        if hasattr(self._func, "current"):
            return self._func.current()
        return self._func()

    def __call__(self, *args: Any, **kws: Any) -> Any:
        if self._name is None:
            return self._func(*args, **kws)
        return self.__current__().__getattr__(self._name)(*args, **kws)

    def __repr__(self) -> str:
        """
        >>> @_dynamic_dispatch
        ... def debug_print(*args, **kws):
        ...     return _HyperParameter(**kws)
        >>> debug_print.a.b.c # doctest: +ELLIPSIS
        <a.b.c in <...._HyperParameter object at ...>>
        """
        return repr(self.__current__().__getattr__(self._name))

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
        target = self.__current__().__getattr__(self._name)
        target.__setattr__(name, value)

    def __or__(self, default: Any) -> Any:
        return _ParamAccessor(self.__current__(), self._name) | default


def _dynamic_dispatch(func: Callable, name: Optional[str] = None) -> _DynamicDispatch:
    """Wraps function with a class to allow __getattr__ on a function."""
    clz = type(func.__name__, (_DynamicDispatch, object), dict(__doc__=func.__doc__))
    return clz(func, name)


class _ParamAccessor:
    """Missing parameter and default value handler for hyperparameters.

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

    def __init__(self, root: Any, name: Optional[str] = None) -> None:
        self._root = root
        self._name = name

    def get_or_else(self, default: Optional[T] = None) -> Union[T, Any]:
        """Get parameter value or return default with type conversion.

        Args:
            default: Default value to return if parameter is missing. The type of
                    default determines the conversion behavior:
                    - bool: Converts string/int to bool
                    - int: Converts value to int
                    - float: Converts value to float
                    - str: Converts value to string
                    - None: Returns value as-is

        Returns:
            The parameter value (converted to match default type) or the default value.
        """
        value = self._root.get(self._name)

        # Handle missing parameter
        if isinstance(value, _ParamAccessor):
            return default

        # Handle suggester (lazy evaluation)
        if isinstance(value, Suggester):
            return value()

        # Handle None default - return value as-is
        if default is None:
            return value

        # Type-specific conversion based on default type
        default_type = type(default)

        # Boolean conversion
        if default_type is bool:
            return self._convert_to_bool(value, default)

        # Integer conversion
        if default_type is int:
            # If value is a string that can be converted to float, return float to preserve precision
            if isinstance(value, str):
                try:
                    float_val = float(value)
                    # If it's a whole number, return int; otherwise return float
                    if float_val.is_integer():
                        return int(float_val)
                    else:
                        return float_val
                except (ValueError, TypeError):
                    pass
            converted_int = self._convert_to_int(value, default)
            return value if converted_int is None else converted_int

        # Float conversion
        if default_type is float:
            converted_float = self._convert_to_float(value, default)
            return value if converted_float is None else converted_float

        # String conversion
        if default_type is str:
            return str(value)

        # For other types, return value as-is
        return value

    def _convert_to_bool(self, value: Any, default: bool) -> bool:
        """Convert value to boolean with support for string representations."""
        if value is None:
            return default
        if isinstance(value, bool):
            return value
        if isinstance(value, int):
            return value != 0
        if isinstance(value, str):
            lower_val = value.lower()
            if lower_val in ("y", "yes", "t", "true", "on", "1"):
                return True
            elif lower_val in ("n", "no", "f", "false", "off", "0"):
                return False
        return default

    def _convert_to_int(self, value: Any, default: int) -> Optional[Union[int, float]]:
        """Convert value to integer with fallback handling."""
        if value is None:
            return default
        try:
            return int(value)
        except (ValueError, TypeError):
            try:
                # Try converting float to int
                return int(float(value))
            except (ValueError, TypeError):
                return None

    def _convert_to_float(self, value: Any, default: float) -> Optional[float]:
        """Convert value to float with error handling."""
        try:
            return float(value)
        except (ValueError, TypeError):
            return None

    def __getitem__(self, index: str) -> "_ParamAccessor":
        return self.__getattr__(index)

    def __getattr__(self, name: str) -> "_ParamAccessor":
        if name in ("_root", "_name"):
            return self.__dict__[name]
        name = f"{self._name}.{name}" if self._name else name
        return _ParamAccessor(self._root, name)

    def __setattr__(self, name: str, value: Any) -> None:
        if name in ("_root", "_name"):
            return self.__dict__.__setitem__(name, value)
        name = f"{self._name}.{name}" if self._name else name
        if isinstance(value, dict):
            self._root.update({name: value})
        else:
            self._root.put(name, value)

    def __repr__(self) -> str:
        return f"<{self._name} in {repr(self._root)}>"

    def __bool__(self) -> bool:
        try:
            value = self._root.get(self._name)
        except Exception:
            return False

        if isinstance(value, _ParamAccessor):
            return False
        if isinstance(value, Suggester):
            return bool(value())
        return bool(value)

    def __call__(self, default: Union[T, object] = _MISSING) -> Union[T, Any]:
        """Get parameter value.

        If default is not provided, the parameter is considered required and will raise KeyError if missing.
        If default is provided, it acts as get_or_else(default).
        """
        if default is _MISSING:
            value = self._root.get(self._name)
            if isinstance(value, _ParamAccessor):
                raise KeyError(
                    f"Hyperparameter '{self._name}' is required but not defined."
                )
            if isinstance(value, Suggester):
                return value()
            return value
        return self.get_or_else(default)

    def __or__(self, default: T) -> Union[T, Any]:
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

    def __init__(self, storage: Optional[Any] = None, **kws: Any) -> None:
        if storage is None:
            storage = TLSKVStorage()
        self.__dict__["_storage"] = storage
        self.update(kws)

    def keys(self) -> Any:
        return self.storage().keys()

    def update(self, kws: Dict[str, Any]) -> "_HyperParameter":
        self._storage.update(kws)
        return self

    def clear(self) -> "_HyperParameter":
        self._storage.clear()
        return self

    def get(self, name: str) -> Union[Any, "_ParamAccessor"]:
        try:
            return self._storage.get(name)
        except (KeyError, ValueError):
            return _ParamAccessor(self, name)

    def put(self, name: str, value: Any) -> None:
        return self._storage.put(name, value)

    def storage(self) -> Any:
        return self._storage

    def __getitem__(self, key: str) -> Any:
        """dict-style api for parameter reading.
        Examples
        ----------
        >>> hp = _HyperParameter(param1=1, obj1={"prop1": "a"})
        >>> hp["param1"]
        1
        >>> hp["obj1.prop1"]
        'a'
        """
        try:
            return self._storage.get(key)
        except (KeyError, ValueError):
            return _ParamAccessor(self._storage, key)

    def __setitem__(self, key: str, value: Any) -> None:
        """dict-style api for parameter writing.
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
        if isinstance(value, dict):
            return self.update({key: value})
        return self._storage.put(key, value)

    def __getattr__(self, name: str) -> Any:
        """object-style api for parameter reading.
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
        """object-style api for parameter writing.
        Examples
        --------
        >>> hp = _HyperParameter()
        >>> hp.param1 = 1
        >>> hp.obj1.prop1 = 'a'
        >>> _repr_dict(hp.storage().storage())
        [('obj1.prop1', 'a'), ('param1', 1)]
        """
        if name in self.__dict__:
            return self.__dict__.__setitem__(name, value)
        self.put(name, value)

    def __iter__(self) -> Any:
        return self._storage.__iter__()

    def __call__(self) -> _ParamAccessor:
        return _ParamAccessor(self)


def _coerce_with_default(value: Any, default: Any) -> Any:
    """Best-effort conversion of value toward the type of default."""
    if default is None:
        return value
    default_type = type(default)
    if default_type is bool:
        if value is None:
            return default
        if isinstance(value, bool):
            return value
        if isinstance(value, int):
            return value != 0
        if isinstance(value, str):
            lower_val = value.lower()
            if lower_val in ("y", "yes", "t", "true", "on", "1"):
                return True
            if lower_val in ("n", "no", "f", "false", "off", "0"):
                return False
        return default
    if default_type is int:
        if isinstance(value, str):
            try:
                float_val = float(value)
                if float_val.is_integer():
                    return int(float_val)
                return float_val
            except (ValueError, TypeError):
                return value
        try:
            return int(value)
        except (ValueError, TypeError):
            try:
                return int(float(value))
            except (ValueError, TypeError):
                return value
    if default_type is float:
        try:
            return float(value)
        except (ValueError, TypeError):
            return value
    if default_type is str:
        return str(value)
    return value


@_dynamic_dispatch
class scope(_HyperParameter):
    """A thread-safe hyperparameter context scope

    Examples
    --------
    **create new `scope`**
    >>> ps = scope(a="a", b="b")           # create from call arguments
    >>> ps = scope(**{"a": "a", "b": "b"}) # create from a dict

    **read parameters from `scope`**
    >>> ps.a() # read parameter
    'a'
    >>> ps.c("c")  # read parameter with default value if missing
    'c'
    >>> ps.c | "c" # another way for reading missing parameters
    'c'

    **`scope` as a context scope**
    >>> with scope(**{"a": "a"}) as ps:
    ...     print(ps.a())
    a

    **read parameter from scope in a function**
    >>> def foo():
    ...    with scope() as ps:
    ...        return ps.a()
    >>> with scope(**{"a": "a", "b": "b"}) as ps:
    ...     foo() # foo should get scope using a with statement
    'a'

    **modify parameters in nested scopes**
    >>> with scope.empty(**{'a': 1, 'b': 2}) as ps:
    ...     _repr_dict(ps.storage().storage())
    ...     with scope(**{'b': 3}) as ps:
    ...         _repr_dict(ps.storage().storage())
    ...     with scope() as ps:
    ...         _repr_dict(ps.storage().storage())
    [('a', 1), ('b', 2)]
    [('a', 1), ('b', 3)]
    [('a', 1), ('b', 2)]

    **use object-style parameter key in scope**
    >>> with scope(**{"a.b.c": [1,2]}) as ps:
    ...     ps.a.b.c()
    [1, 2]

    **access parameter with `scope`**
    >>> with scope(x=1):
    ...     scope.x(2) # read parameter
    ...     scope.y(2) # read a missing parameter with default value
    ...     scope.y | 2
    ...     scope.z = 3
    ...     scope.z | 0
    1
    2
    2
    3

    **convert scope to dict**:
    >>> ps = scope.empty(a=1, b=2)
    >>> _repr_dict(dict(ps))
    [('a', 1), ('b', 2)]
    """

    def __init__(self, *args: str, **kwargs: Any) -> None:
        super().__init__()
        self.update(kwargs)
        for line in args:
            if "=" in line:
                k, v = line.split("=", 1)
                self.put(k, v)

    def __enter__(self) -> "scope":
        """enter a `scope` context

        Examples
        --------
        >>> with scope():
        ...     scope.p = "origin"
        ...     with scope(**{"p": "origin"}) as ps:
        ...         ps.storage().storage()      # outer scope
        ...         with scope() as ps:   # unmodified scope
        ...             ps.storage().storage()  # inner scope
        ...         with scope(**{"p": "modified"}) as ps: # modified scope
        ...             ps.storage().storage()  # inner scope with modified params
        ...         _ = scope(**{"p": "modified"}) # not used in with ctx
        ...         with scope() as ps:   # unmodified scope
        ...             ps.storage().storage()  # inner scope
        {'p': 'origin'}
        {'p': 'origin'}
        {'p': 'modified'}
        {'p': 'origin'}
        """

        self._storage.enter()
        return self

    def __exit__(
        self,
        exc_type: Optional[Any],
        exc_value: Optional[Any],
        traceback: Optional[Any],
    ) -> None:
        self._storage.exit()

    def __call__(self) -> _ParamAccessor:
        return _ParamAccessor(self)

    @staticmethod
    def empty(*args: str, **kwargs: Any) -> "scope":
        """create an empty `scope`.

        Examples
        --------
        >>> with scope(a="not empty") as ps: # start a new scope `a` = 'not empty'
        ...     scope.a("empty")             # read parameter `a`
        ...     with scope.empty() as ps2:   # parameter `a` is cleared in ps2
        ...         scope.a("empty")         # read parameter `a` = 'empty'
        'not empty'
        'empty'
        """
        retval = scope().clear().update(kwargs)
        for line in args:
            if "=" in line:
                k, v = line.split("=", 1)
                retval.put(k, v)
        return retval

    @staticmethod
    def current() -> "scope":
        """get current `scope`

        Examples
        --------
        >>> with scope(a=1) as ps:
        ...     scope.current().a("empty") # read `a` from current `scope`
        '1'

        >>> with scope() as ps1:
        ...     with scope(a=1) as ps2:
        ...         scope.current().a = 2  # set parameter `a` = 2
        ...         scope.a("empty")       # read `a` in `ps2`
        ...     scope.a("empty")           # read `a` in `ps1`, where `a` is not set
        '2'
        'empty'
        """
        retval = scope()
        retval._storage = TLSKVStorage.current()
        return retval

    @staticmethod
    def init(params: Optional[Dict[str, Any]] = None) -> None:
        """init scope for a new thread."""
        if params is None:
            params = {}
        scope(**params).__enter__()

    @staticmethod
    def frozen() -> None:
        with scope():
            TLSKVStorage.frozen()


_scope = scope._func


@overload
def param(func: Callable) -> Callable: ...


@overload
def param(name: str) -> Callable[[Callable], Callable]: ...


def param(
    name_or_func: Union[str, Callable, None],
) -> Union[Callable, Callable[[Callable], Callable]]:
    """Convert keyword arguments into hyperparameters

    Examples
    --------

    >>> @param
    ... def foo(a, b=2, c='c', d=None):
    ...     print(a, b, c, d)

    >>> foo(1)
    1 2 c None

    >>> with scope('foo.b=3'):
    ...     foo(2)
    2 3 c None

    classes are also supported:
    >>> @param
    ... class foo:
    ...     def __init__(self, a, b=2, c='c', d=None):
    ...         print(a, b, c, d)

    >>> obj = foo(1)
    1 2 c None

    >>> with scope('foo.b=3'):
    ...     obj = foo(2)
    2 3 c None

    >>> @param('myns.foo.params')
    ... def foo(a, b=2, c='c', d=None):
    ...     print(a, b, c, d)
    >>> foo(1)
    1 2 c None

    >>> with scope('myns.foo.params.b=3'):
    ...     foo(2)
    2 3 c None

    >>> with scope('myns.foo.params.b=3'):
    ...     scope.myns.foo.params.b = 4
    ...     foo(2)
    2 4 c None
    """

    if callable(name_or_func):
        return param(None)(name_or_func)

    if has_rust_backend:

        def hashed_wrapper(func: Callable) -> Callable:
            predef_kws: Dict[str, int] = {}
            predef_defaults: Dict[str, Any] = {}

            if name_or_func is None:
                namespace = func.__name__
            else:
                namespace = name_or_func

            signature = inspect.signature(func)
            for k, v in signature.parameters.items():
                if v.default != v.empty:
                    name = "{}.{}".format(namespace, k)
                    predef_kws[k] = xxh64(name)
                    predef_defaults[k] = v.default

            @functools.wraps(func)
            def inner(*arg: Any, **kws: Any) -> Any:
                with scope() as hp:
                    for k, v in predef_kws.items():
                        if k not in kws:
                            try:
                                val = hp._storage.get_entry(v)
                                kws[k] = _coerce_with_default(val, predef_defaults[k])
                            except ValueError:
                                pass
                    return func(*arg, **kws)

            inner._auto_param_namespace = namespace  # type: ignore[attr-defined]
            inner._auto_param_wrapped = func  # type: ignore[attr-defined]
            return inner

        return hashed_wrapper

    def wrapper(func: Callable) -> Callable:
        predef_kws: Dict[str, str] = {}
        predef_val: Dict[str, Any] = {}

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
        def inner(*arg: Any, **kws: Any) -> Any:
            with scope() as hp:
                local_params: Dict[str, Any] = {}
                for k, v in predef_kws.items():
                    if k not in kws:
                        val = getattr(hp(), v).get_or_else(predef_val[v])
                        kws[k] = val
                        local_params[v] = hp.get(v)
                    else:
                        local_params[v] = predef_val[v]
                return func(*arg, **kws)

        inner._auto_param_namespace = namespace  # type: ignore[attr-defined]
        inner._auto_param_wrapped = func  # type: ignore[attr-defined]
        return inner

    return wrapper


# Import CLI functions from cli.py to maintain backward compatibility
from .cli import launch
