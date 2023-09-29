import os
import threading
from typing import Any, Callable, Dict, Iterable


GLOBAL_STORAGE = {}


class Storage:
    """Base class for all storage implementations"""

    # storage operations
    def child(self) -> "Storage":
        return None

    def storage(self) -> Dict[str, Any]:
        pass

    # dict operations
    def keys(self) -> Iterable:
        pass

    def update(self, kws: Dict[str, Any] = {}) -> None:
        return None

    def clear(self):
        pass

    def __iter__(self):
        pass

    # kv operations
    def get(self, name: str, accessor: Callable = None) -> Any:
        return None

    def put(self, name: str, value: Any) -> None:
        return None

    # context operations
    def enter(self):
        pass

    def exit(self):
        pass

    @staticmethod
    def current():
        pass


class TLSKVStorage(Storage):
    """Pure Python implementation of a key-value storage"""

    __slots__ = ("_storage", "_parent")
    tls = threading.local()

    def __init__(self, parent=None) -> None:
        self._storage = None
        self._parent = parent
        super().__init__()

        if not hasattr(TLSKVStorage.tls, "his"):
            TLSKVStorage.tls.his = [TLSKVStorage.__new__(TLSKVStorage)]
            TLSKVStorage.tls.his[-1]._storage = GLOBAL_STORAGE
            TLSKVStorage.tls.his[-1]._parent = None
            self.update(GLOBAL_STORAGE)

        elif hasattr(TLSKVStorage.tls, "his") and len(TLSKVStorage.tls.his) > 0:
            parent = TLSKVStorage.tls.his[-1]
            self.update(parent._storage)

    def __iter__(self):
        return iter(self._storage.items())

    def child(self) -> "Storage":
        obj = TLSKVStorage(self)
        obj.update(self._storage)
        return obj

    def storage(self) -> Dict[str, Any]:
        return self._storage

    def keys(self) -> Iterable:
        return self._storage.keys()

    def update(self, kws: Dict[str, Any] = {}) -> None:
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

        if kws is not None:
            return _update(kws, prefix=None)

    def clear(self):
        self._storage.clear()
        
    def get_entry(self, *args, **kwargs):
        raise RuntimeError("hyperparameter is not build with rust backend")

    def get(self, name: str, accessor: Callable = None) -> Any:
        if name in self.__slots__:
            return self.__dict__[name]
        curr = self
        while curr is not None and curr._storage is not None:
            if name in curr._storage:
                return curr._storage[name]
            curr = curr._parent
        raise ValueError()
        # return accessor(self, name)

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

    @staticmethod
    def current():
        if not hasattr(TLSKVStorage.tls, "his") or len(TLSKVStorage.tls.his) == 0:
            TLSKVStorage.tls.his = [TLSKVStorage()]
        return TLSKVStorage.tls.his[-1]

    @staticmethod
    def frozen():
        GLOBAL_STORAGE.update(TLSKVStorage.tls.his[-1].storage())


has_rust_backend = False


def xxh64(*args, **kwargs):
    raise RuntimeError("hyperparameter is not build with rust backend")


try:
    if os.environ.get("HYPERPARAMETER_BACKEND", "RUST") == "RUST":
        from hyperparameter.librbackend import KVStorage
        from hyperparameter.librbackend import xxh64

        TLSKVStorage = KVStorage
        has_rust_backend = True
except:
    import traceback

    traceback.print_exc()
