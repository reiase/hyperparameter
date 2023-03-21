import threading
from typing import Any, Dict, Iterable


class Storage:
    """Base class for all storage implementations"""

    def child(self) -> "Storage":
        return None

    def storage(self) -> Dict[str, Any]:
        pass

    def keys(self) -> Iterable:
        pass

    def update(self, kws: Dict[str, Any]) -> None:
        return None

    def get(self, name: str) -> Any:
        return None

    def put(self, name: str, value: Any) -> None:
        return None

    def __iter__(self):
        pass


class TLSKVStorage(Storage):
    """Pure Python implementation of a key-value storage"""

    __slots__ = ("_storage", "_parent", "_accessor")
    tls = threading.local()

    def __init__(self, parent=None, accessor=None) -> None:
        self._storage = None
        self._parent = parent
        self._accessor = accessor
        super().__init__()
        # super().__init__(parent, accessor)

        if hasattr(TLSKVStorage.tls, "his") and len(TLSKVStorage.tls.his) > 0:
            parent = TLSKVStorage.tls.his[-1]
            # super().__init__(parent, accessor)
            self.update(parent._storage)
        # else:
        #     super().__init__(None, accessor)

    def __iter__(self):
        return iter(self._storage.items())

    def child(self) -> "Storage":
        obj = KVStorage(self, self._accessor)
        obj.update(self._storage)
        return obj

    def storage(self) -> Dict[str, Any]:
        return self._storage

    def keys(self) -> Iterable:
        return self._storage.keys()

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

    def enter(self):
        if not hasattr(TLSKVStorage.tls, "his"):
            TLSKVStorage.tls.his = []
        TLSKVStorage.tls.his.append(self)
        return TLSKVStorage.tls.his[-1]

    def exit(self):
        TLSKVStorage.tls.his.pop()