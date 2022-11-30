from typing import List, Optional, Set


def _sorted_set(s):
    retval = list(s)
    retval.sort()
    return retval


class Tracker:
    """parameter access tracker

    Examples
    --------
    >>> from hyperparameter import param_scope
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

    def read(self, key: Optional[str] = None) -> Optional[List[str]]:
        return self._get.add(key) if key else _sorted_set(self._get)

    def write(self, key: Optional[str] = None) -> Optional[List[str]]:
        return self._put.add(key) if key else _sorted_set(self._put)

    def all(self):
        return _sorted_set(self._get.union(self._put))


_tracker = Tracker()


def reads():
    return _tracker.read()


def writes():
    return _tracker.write()


def all_params():
    return _tracker.all()
