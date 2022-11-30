from operator import getitem
from typing import Any, Callable


class Suggester:
    def __init__(self, callback: Callable) -> None:
        self._callback = callback

    def __call__(self):
        return self._callback()


def suggest_from(callback: Callable) -> Suggester:
    """Suggest parameter from a callback function

    Examples
    --------
    >>> class ValueWrapper:
    ...     def __init__(self, lst):
    ...         self._lst = lst
    ...         self._offset = 0
    ...     def __call__(self):
    ...         index, self._offset = self._offset % len(self._lst), self._offset + 1
    ...         return self._lst[index]

    >>> from hyperparameter import param_scope, suggest_from
    >>> with param_scope(suggested = suggest_from(ValueWrapper([1,2,3]))) as ps:
    ...     ps().suggested()
    ...     ps().suggested()
    ...     ps().suggested()
    1
    2
    3
    """
    return Suggester(callback)


def unwrap_suggester(func):
    def wrapper(*args, **kwargs):
        retval = func(*args, **kwargs)
        if isinstance(retval, Suggester):
            return retval()
        return retval

    return wrapper


class LazyDispatch:
    """Dynamic call dispatcher

    Examples
    --------

    >>> class ExampleObj:
    ...     def get_42(self, offset):
    ...         return 42+offset

    >>> lazy_obj = lazy_dispatch(ExampleObj())
    >>> lazy_obj.get_42(0)()
    42
    """

    def __init__(self, obj: Any, name=None, index=None):
        self._obj = obj
        self._name = name
        self._index = index

    def __call__(self, *args, **kws) -> Any:
        obj = self._obj
        for n in self._name.split("."):
            obj = getattr(obj, n)
        if self._index is not None:
            obj = getitem(obj, self._index)
        return Suggester(lambda: obj(*args, **kws))

    def __getattr__(self, name: str) -> Any:
        if self._name is not None:
            name = f"{self._name}.{name}"
        return lazy_dispatch(self._obj, name, self._index)

    def __getitem__(self, index):
        return lazy_dispatch(self._obj, self._name, index)


def lazy_dispatch(obj, name=None, index=None):
    """Wraps an object for lazy dispatch"""
    return LazyDispatch(obj, name, index)
