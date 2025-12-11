from __future__ import annotations

import argparse
import functools
import inspect
import sys
from typing import Any, Callable, Dict, Optional, Union, TypeVar, overload, List, Tuple

from hyperparameter.storage import TLSKVStorage, has_rust_backend, xxh64

from .tune import Suggester

T = TypeVar("T")


def _repr_dict(d: Dict[str, Any]) -> List[Tuple[str, Any]]:
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

    def __call__(self, default: Optional[T] = None) -> Union[T, Any]:
        """shortcut for get_or_else"""
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


def _parse_param_help(doc: Optional[str]) -> Dict[str, str]:
    """Parse param help from docstring (Google/NumPy/reST)."""
    if not doc:
        return {}
    lines = [line.rstrip() for line in doc.splitlines()]
    help_map: Dict[str, str] = {}

    # Google style: Args:/Arguments:
    def parse_google():
        in_args = False
        for line in lines:
            if not in_args:
                if line.strip().lower() in ("args:", "arguments:"):
                    in_args = True
                continue
            if line.strip() == "":
                if in_args:
                    break
                continue
            if not line.startswith(" "):
                break
            stripped = line.strip()
            if ":" in stripped:
                name_part, desc = stripped.split(":", 1)
                name_part = name_part.strip()
                if "(" in name_part and ")" in name_part:
                    name_part = name_part.split("(")[0].strip()
                if name_part:
                    help_map.setdefault(name_part, desc.strip())

    # NumPy style: Parameters
    def parse_numpy():
        in_params = False
        current_name = None
        for line in lines:
            if not in_params:
                if line.strip().lower() == "parameters":
                    in_params = True
                continue
            if line.strip() == "":
                if current_name is not None:
                    current_name = None
                continue
            if not line.startswith(" "):
                # section ended
                break
            # parameter line: name : type
            if ":" in line:
                name_part = line.split(":", 1)[0].strip()
                current_name = name_part
                # description may follow on same line after type, but we skip
                if current_name and current_name not in help_map:
                    # next indented lines are description
                    continue
            elif current_name:
                desc = line.strip()
                if desc:
                    help_map.setdefault(current_name, desc)

    # reST/Sphinx: :param name: desc
    def parse_rest():
        for line in lines:
            striped = line.strip()
            if striped.startswith(":param"):
                # forms: :param name: desc  or :param type name: desc
                parts = striped.split(":param", 1)[1].strip()
                if ":" in parts:
                    before, desc = parts.split(":", 1)
                    tokens = before.split()
                    name = tokens[-1] if tokens else ""
                    if name:
                        help_map.setdefault(name, desc.strip())

    parse_google()
    parse_numpy()
    parse_rest()
    return help_map


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
    >>> with param_scope.empty(**{'a': 1, 'b': 2}) as ps:
    ...     _repr_dict(ps.storage().storage())
    ...     with param_scope(**{'b': 3}) as ps:
    ...         _repr_dict(ps.storage().storage())
    ...     with param_scope() as ps:
    ...         _repr_dict(ps.storage().storage())
    [('a', 1), ('b', 2)]
    [('a', 1), ('b', 3)]
    [('a', 1), ('b', 2)]

    **use object-style parameter key in param_scope**
    >>> with param_scope(**{"a.b.c": [1,2]}) as ps:
    ...     ps.a.b.c()
    [1, 2]

    **access parameter with `param_scope`**
    >>> with param_scope(x=1):
    ...     param_scope.x(2) # read parameter
    ...     param_scope.y(2) # read a missing parameter with default value
    ...     param_scope.y | 2
    ...     param_scope.z = 3
    ...     param_scope.z | 0
    1
    2
    2
    3

    **convert param_scope to dict**:
    >>> ps = param_scope.empty(a=1, b=2)
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

    def __enter__(self) -> "param_scope":
        """enter a `param_scope` context

        Examples
        --------
        >>> with param_scope():
        ...     param_scope.p = "origin"
        ...     with param_scope(**{"p": "origin"}) as ps:
        ...         ps.storage().storage()      # outer scope
        ...         with param_scope() as ps:   # unmodified scope
        ...             ps.storage().storage()  # inner scope
        ...         with param_scope(**{"p": "modified"}) as ps: # modified scope
        ...             ps.storage().storage()  # inner scope with modified params
        ...         _ = param_scope(**{"p": "modified"}) # not used in with ctx
        ...         with param_scope() as ps:   # unmodified scope
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
    def empty(*args: str, **kwargs: Any) -> "param_scope":
        """create an empty `param_scope`.

        Examples
        --------
        >>> with param_scope(a="not empty") as ps: # start a new param_scope `a` = 'not empty'
        ...     param_scope.a("empty")             # read parameter `a`
        ...     with param_scope.empty() as ps2:   # parameter `a` is cleared in ps2
        ...         param_scope.a("empty")         # read parameter `a` = 'empty'
        'not empty'
        'empty'
        """
        retval = param_scope().clear().update(kwargs)
        for line in args:
            if "=" in line:
                k, v = line.split("=", 1)
                retval.put(k, v)
        return retval

    @staticmethod
    def current() -> "param_scope":
        """get current `param_scope`

        Examples
        --------
        >>> with param_scope(a=1) as ps:
        ...     param_scope.current().a("empty") # read `a` from current `param_scope`
        '1'

        >>> with param_scope() as ps1:
        ...     with param_scope(a=1) as ps2:
        ...         param_scope.current().a = 2  # set parameter `a` = 2
        ...         param_scope.a("empty")       # read `a` in `ps2`
        ...     param_scope.a("empty")           # read `a` in `ps1`, where `a` is not set
        '2'
        'empty'
        """
        retval = param_scope()
        retval._storage = TLSKVStorage.current()
        return retval

    @staticmethod
    def init(params: Optional[Dict[str, Any]] = None) -> None:
        """init param_scope for a new thread."""
        if params is None:
            params = {}
        param_scope(**params).__enter__()

    @staticmethod
    def frozen() -> None:
        with param_scope():
            TLSKVStorage.frozen()


_param_scope = param_scope._func


@overload
def auto_param(func: Callable) -> Callable: ...


@overload
def auto_param(name: str) -> Callable[[Callable], Callable]: ...


def auto_param(
    name_or_func: Union[str, Callable, None],
) -> Union[Callable, Callable[[Callable], Callable]]:
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

    >>> with param_scope('myns.foo.params.b=3'):
    ...     param_scope.myns.foo.params.b = 4
    ...     foo(2)
    2 4 c None
    """

    if callable(name_or_func):
        return auto_param(None)(name_or_func)

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
                with param_scope() as hp:
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
            with param_scope() as hp:
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


def _arg_type_from_default(default: Any) -> Optional[Callable[[str], Any]]:
    if isinstance(default, bool):
        def _to_bool(v: str) -> bool:
            return v.lower() in ("1", "true", "t", "yes", "y", "on")
        return _to_bool
    if default is None:
        return None
    return type(default)


def _extract_first_paragraph(docstring: Optional[str]) -> Optional[str]:
    """Extract the first paragraph from a docstring for cleaner help output.
    
    The first paragraph is defined as text up to the first blank line or
    the first line that starts with common docstring section markers like
    'Args:', 'Returns:', 'Examples:', etc.
    """
    if not docstring:
        return None
    
    lines = docstring.strip().split('\n')
    first_paragraph = []
    
    for line in lines:
        stripped = line.strip()
        # Stop at blank lines
        if not stripped:
            break
        # Stop at common docstring section markers
        if stripped.lower() in ('args:', 'arguments:', 'parameters:', 'returns:', 
                                'raises:', 'examples:', 'note:', 'warning:', 
                                'see also:', 'todo:'):
            break
        first_paragraph.append(stripped)
    
    result = ' '.join(first_paragraph).strip()
    return result if result else None


def _build_parser_for_func(func: Callable, prog: Optional[str] = None) -> argparse.ArgumentParser:
    sig = inspect.signature(func)
    # Use first paragraph of docstring for cleaner help output
    description = _extract_first_paragraph(func.__doc__) or func.__doc__
    parser = argparse.ArgumentParser(prog=prog or func.__name__, description=description)
    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend", help="Override params, e.g., a.b=1")
    parser.add_argument(
        "-lps",
        "--list-params",
        action="store_true",
        help="List parameter names, defaults, and current values (after --define overrides), then exit.",
    )
    parser.add_argument(
        "-ep",
        "--explain-param",
        nargs="*",
        metavar="NAME",
        help="Explain the source of specific parameters (default, CLI arg, or --define override), then exit. If omitted, prints all.",
    )
    param_help = _parse_param_help(func.__doc__)

    for name, param in sig.parameters.items():
        if param.default is inspect._empty:
            parser.add_argument(name, type=param.annotation if param.annotation is not inspect._empty else str, help=param_help.get(name))
        else:
            arg_type = _arg_type_from_default(param.default)
            help_text = param_help.get(name)
            if help_text:
                help_text = f"{help_text} (default: {param.default})"
            else:
                help_text = f"(default from auto_param: {param.default})"
            parser.add_argument(
                f"--{name}",
                dest=name,
                type=arg_type,
                default=argparse.SUPPRESS,
                help=help_text,
            )
    return parser


def _describe_parameters(func: Callable, defines: List[str], arg_overrides: Dict[str, Any]) -> List[Tuple[str, str, str, Any, str, Any]]:
    """Return [(func_name, param_name, full_key, value, source, default)] under current overrides."""
    namespace = getattr(func, "_auto_param_namespace", func.__name__)
    func_name = getattr(func, "__name__", namespace)
    sig = inspect.signature(func)
    results: List[Tuple[str, str, str, Any, str, Any]] = []
    _MISSING = object()
    with param_scope(*defines) as hp:
        storage_snapshot = hp.storage().storage()
        for name, param in sig.parameters.items():
            default = param.default if param.default is not inspect._empty else _MISSING
            if name in arg_overrides:
                value = arg_overrides[name]
                source = "cli-arg"
            else:
                full_key = f"{namespace}.{name}"
                in_define = full_key in storage_snapshot
                if default is _MISSING:
                    value = "<required>"
                else:
                    value = getattr(hp(), full_key).get_or_else(default)
                source = "--define" if in_define else ("default" if default is not _MISSING else "required")
            printable_default = "<required>" if default is _MISSING else default
            results.append((func_name, name, full_key, value, source, printable_default))
    return results


def _maybe_explain_and_exit(func: Callable, args_dict: Dict[str, Any], defines: List[str]) -> bool:
    list_params = bool(args_dict.pop("list_params", False))
    explain_targets = args_dict.pop("explain_param", None)
    if explain_targets is not None and len(explain_targets) == 0:
        print("No parameter names provided to --explain-param. Please specify at least one.")
        sys.exit(1)
    if not list_params and not explain_targets:
        return False

    rows = _describe_parameters(func, defines, args_dict)
    target_set = set(explain_targets) if explain_targets is not None else None
    if explain_targets is not None and target_set is not None and all(full_key not in target_set for _, _, full_key, _, _, _ in rows):
        missing = ", ".join(explain_targets)
        print(f"No matching parameters for: {missing}")
        sys.exit(1)
    for func_name, name, full_key, value, source, default in rows:
        # Use fully qualified key for matching to avoid collisions.
        if target_set is not None and full_key not in target_set:
            continue
        default_repr = "<required>" if default == "<required>" else repr(default)
        func_module = getattr(func, "__module__", "unknown")
        location = f"{func_module}.{func_name}"
        print(f"{full_key}:")
        print(f"    function={func_name}, location={location}, default={default_repr}")
    return True


def launch(func: Optional[Callable] = None, *, _caller_globals=None, _caller_locals=None, _caller_module=None) -> None:
    """Launch CLI for @auto_param functions.

    - launch(f): expose a single @auto_param function f as CLI.
    - launch(): expose all @auto_param functions in the caller module as subcommands.

    Args:
        func: Optional function to launch. If None, discovers all @auto_param functions in caller module.
        _caller_globals: Explicitly pass caller's globals dict (for entry point support).
        _caller_locals: Explicitly pass caller's locals dict (for entry point support).
        _caller_module: Explicitly pass caller's module name or module object (for entry point support).
                       Can be a string (module name) or a module object.
    """
    if _caller_globals is None or _caller_locals is None:
        caller_frame = inspect.currentframe().f_back  # type: ignore
        if caller_frame is not None:
            caller_globals = caller_frame.f_globals
            caller_locals = caller_frame.f_locals
        else:
            # Fallback: try to find the caller module from sys.modules
            caller_globals = {}
            caller_locals = {}
            if _caller_module is not None:
                if isinstance(_caller_module, str):
                    if _caller_module in sys.modules:
                        mod = sys.modules[_caller_module]
                        caller_globals = mod.__dict__
                        caller_locals = mod.__dict__
                elif hasattr(_caller_module, '__dict__'):
                    caller_globals = _caller_module.__dict__
                    caller_locals = _caller_module.__dict__
            else:
                # Last resort: try to find the module that called us by walking the stack
                frame = inspect.currentframe()
                if frame is not None:
                    # Walk up the stack to find a module frame
                    current = frame.f_back
                    while current is not None:
                        globs = current.f_globals
                        # Check if this looks like a module (has __name__ and __file__)
                        if '__name__' in globs and '__file__' in globs:
                            caller_globals = globs
                            caller_locals = current.f_locals
                            break
                        current = current.f_back
    else:
        caller_globals = _caller_globals
        caller_locals = _caller_locals

    if func is None:
        seen_ids = set()
        candidates = []
        for obj in list(caller_locals.values()) + list(caller_globals.values()):
            if not callable(obj):
                continue
            ns = getattr(obj, "_auto_param_namespace", None)
            if not isinstance(ns, str):
                continue
            # Skip private helpers (e.g., _foo) when exposing subcommands.
            name = getattr(obj, "__name__", "")
            if isinstance(name, str) and name.startswith("_"):
                continue
            oid = id(obj)
            if oid in seen_ids:
                continue
            seen_ids.add(oid)
            candidates.append(obj)
        if not candidates:
            raise RuntimeError("No @auto_param functions found to launch.")

        if len(candidates) == 1:
            import sys
            
            func = candidates[0]
            parser = _build_parser_for_func(func)
            argv = sys.argv[1:]
            if argv and argv[0] == func.__name__:
                argv = argv[1:]
            args = parser.parse_args(argv)
            args_dict = vars(args)
            defines = args_dict.pop("define", [])
            if _maybe_explain_and_exit(func, args_dict, defines):
                return None
            with param_scope(*defines):
                return func(**args_dict)

        parser = argparse.ArgumentParser(description="hyperparameter auto-param CLI")
        subparsers = parser.add_subparsers(dest="command", required=True)
        func_map: Dict[str, Callable] = {}
        for f in candidates:
            # Use first paragraph of docstring for cleaner help output
            help_text = _extract_first_paragraph(f.__doc__) or f.__doc__
            sub = subparsers.add_parser(f.__name__, help=help_text)
            func_map[f.__name__] = f
            sub.add_argument("-D", "--define", nargs="*", default=[], action="extend", help="Override params, e.g., a.b=1")
            sub.add_argument(
                "-lps",
                "--list-params",
                action="store_true",
                help="List parameter names, defaults, and current values (after --define overrides), then exit.",
            )
            sub.add_argument(
                "-ep",
                "--explain-param",
                nargs="*",
                metavar="NAME",
                help="Explain the source of specific parameters (default, CLI arg, or --define override), then exit. If omitted, prints all.",
            )
            sig = inspect.signature(f)
            param_help = _parse_param_help(f.__doc__)
            for name, param in sig.parameters.items():
                if param.default is inspect._empty:
                    sub.add_argument(name, type=param.annotation if param.annotation is not inspect._empty else str, help=param_help.get(name))
                else:
                    arg_type = _arg_type_from_default(param.default)
                    help_text = param_help.get(name)
                    if help_text:
                        help_text = f"{help_text} (default: {param.default})"
                    else:
                        help_text = f"(default from auto_param: {param.default})"
                    sub.add_argument(
                        f"--{name}",
                        dest=name,
                        type=arg_type,
                        default=argparse.SUPPRESS,
                        help=help_text,
                    )
        args = parser.parse_args()
        args_dict = vars(args)
        cmd = args_dict.pop("command")
        defines = args_dict.pop("define", [])
        target = func_map[cmd]
        if _maybe_explain_and_exit(target, args_dict, defines):
            return None
        with param_scope(*defines):
            # Freeze first so new threads spawned inside target inherit these overrides.
            param_scope.frozen()
            return target(**args_dict)

    if not hasattr(func, "_auto_param_namespace"):
        raise ValueError("launch() expects a function decorated with @auto_param")
    parser = _build_parser_for_func(func)
    args = parser.parse_args()
    args_dict = vars(args)
    defines = args_dict.pop("define", [])
    if _maybe_explain_and_exit(func, args_dict, defines):
        return None
    with param_scope(*defines):
        param_scope.frozen()
        return func(**args_dict)


def run_cli(func: Optional[Callable] = None, *, _caller_module=None) -> None:
    """Alias for launch() with a less collision-prone name.
    
    Args:
        func: Optional function to launch. If None, discovers all @auto_param functions in caller module.
        _caller_module: Explicitly pass caller's module name or module object (for entry point support).
                       This is useful when called via entry points where frame inspection may fail.
                       Can be a string (module name) or a module object.
    
    Examples:
        # In __main__.py or entry point script:
        if __name__ == "__main__":
            import sys
            run_cli(_caller_module=sys.modules[__name__])
        
        # Or simply:
        if __name__ == "__main__":
            run_cli(_caller_module=__name__)
    """
    caller_frame = inspect.currentframe().f_back  # type: ignore
    if caller_frame is not None:
        caller_globals = caller_frame.f_globals
        caller_locals = caller_frame.f_locals
    else:
        caller_globals = {}
        caller_locals = {}
        # Try to use _caller_module if provided
        if _caller_module is not None:
            if isinstance(_caller_module, str):
                if _caller_module in sys.modules:
                    mod = sys.modules[_caller_module]
                    caller_globals = mod.__dict__
                    caller_locals = mod.__dict__
            elif hasattr(_caller_module, '__dict__'):
                caller_globals = _caller_module.__dict__
                caller_locals = _caller_module.__dict__
    
    return launch(func, _caller_globals=caller_globals, _caller_locals=caller_locals, _caller_module=_caller_module)
