from typing import Any, Dict


class Storage:
    """Base class for all storage implementations"""

    def child(self) -> "Storage":
        return None

    def storage(self) -> Dict[str, Any]:
        pass

    def update(self, kws: Dict[str, Any]) -> None:
        return None

    def get(self, name: str) -> Any:
        return None

    def put(self, name: str, value: Any) -> None:
        return None


class KVStorage(Storage):
    """Pure Python implementation of a key-value storage"""

    __slots__ = ("_storage", "_parent", "_accessor")

    def __init__(self, parent=None, accessor=None) -> None:
        self._storage = None
        self._parent = parent
        self._accessor = accessor
        super().__init__()

    def child(self) -> "Storage":
        obj = KVStorage(self, self._accessor)
        obj.update(self._storage)
        return obj

    def storage(self) -> Dict[str, Any]:
        return self._storage

    def update(self, kws: Dict[str, Any]) -> None:
        if self._storage is None:
            self._storage = {}

        storage = self._storage

        def _update(values={}, prefix=None):
            for k, v in values.items():
                key = f"{prefix}.{k}" if prefix is not None else f"{k}"
                if isinstance(v, dict):
                    _update(v, prefix=key)
                else:
                    storage[key] = v

        return _update(kws, prefix=None)

    def get(self, name: str) -> Any:
        if name in self.__slots__:
            return self.__dict__[name]
        curr = self
        while curr is not None:
            if name in curr._storage:
                return curr._storage[name]
            curr = curr._parent
        return self._accessor(self, name)

    def put(self, name: str, value: Any) -> None:
        if name in self.__slots__:
            return self.__dict__.__setitem__(name, value)
        return self.update({name: value})
