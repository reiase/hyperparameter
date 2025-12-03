from __future__ import annotations

import os
import threading
from typing import Any, Callable, Dict, Iterable, Optional, Iterator, Tuple

GLOBAL_STORAGE: Dict[str, Any] = {}


class Storage:
    """Base class for all storage implementations"""

    # storage operations
    def child(self) -> Optional["Storage"]:
        return None

    def storage(self) -> Dict[str, Any]:
        """Return the underlying storage dictionary."""
        pass

    # dict operations
    def keys(self) -> Iterable[str]:
        """Return an iterable of all keys in storage."""
        pass

    def update(self, kws: Dict[str, Any] = {}) -> None:
        """Update storage with key-value pairs from dictionary."""
        return None

    def clear(self) -> None:
        """Clear all entries from storage."""
        pass

    def __iter__(self) -> Iterator[Tuple[str, Any]]:
        """Return an iterator over (key, value) pairs."""
        pass

    # kv operations
    def get(self, name: str, accessor: Optional[Callable] = None) -> Any:
        """Get a value by name, optionally using an accessor function."""
        return None

    def put(self, name: str, value: Any) -> None:
        """Store a value with the given name."""
        return None

    # context operations
    def enter(self) -> "Storage":
        """Enter a new storage context scope."""
        pass

    def exit(self) -> None:
        """Exit the current storage context scope."""
        pass

    @staticmethod
    def current() -> Optional["Storage"]:
        """Get the current storage instance from thread-local storage."""
        pass


class TLSKVStorage(Storage):
    """Pure Python implementation of a key-value storage"""

    __slots__ = ("_storage", "_parent")
    tls = threading.local()

    def __init__(self, parent: Optional["TLSKVStorage"] = None) -> None:
        self._storage: Optional[Dict[str, Any]] = None
        self._parent: Optional["TLSKVStorage"] = parent
        super().__init__()

        if not hasattr(TLSKVStorage.tls, "his"):
            TLSKVStorage.tls.his = [TLSKVStorage.__new__(TLSKVStorage)]
            TLSKVStorage.tls.his[-1]._storage = GLOBAL_STORAGE
            TLSKVStorage.tls.his[-1]._parent = None
            self.update(GLOBAL_STORAGE)

        elif hasattr(TLSKVStorage.tls, "his") and len(TLSKVStorage.tls.his) > 0:
            parent = TLSKVStorage.tls.his[-1]
            if parent._storage is not None:
                self.update(parent._storage)

    def __iter__(self) -> Iterator[Tuple[str, Any]]:
        if self._storage is None:
            return iter([])
        return iter(self._storage.items())

    def child(self) -> "TLSKVStorage":
        obj = TLSKVStorage(self)
        if self._storage is not None:
            obj.update(self._storage)
        return obj

    def storage(self) -> Dict[str, Any]:
        if self._storage is None:
            self._storage = {}
        return self._storage

    def keys(self) -> Iterable[str]:
        if self._storage is None:
            return []
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

    def get_entry(self, *args: Any, **kwargs: Any) -> Any:
        """Get entry by hash (Rust backend only)."""
        raise RuntimeError("hyperparameter is not built with rust backend")

    def get(self, name: str, accessor: Optional[Callable] = None) -> Any:
        if name in self.__slots__:
            return self.__dict__[name]
        curr: Optional["TLSKVStorage"] = self
        while curr is not None and curr._storage is not None:
            if name in curr._storage:
                return curr._storage[name]
            curr = curr._parent
        raise KeyError(f"Parameter '{name}' not found in storage")
        # return accessor(self, name)

    def put(self, name: str, value: Any) -> None:
        if name in self.__slots__:
            return self.__dict__.__setitem__(name, value)
        return self.update({name: value})

    def enter(self) -> "TLSKVStorage":
        if not hasattr(TLSKVStorage.tls, "his"):
            TLSKVStorage.tls.his = []
        TLSKVStorage.tls.his.append(self)
        return TLSKVStorage.tls.his[-1]

    def exit(self) -> None:
        TLSKVStorage.tls.his.pop()

    @staticmethod
    def current() -> "TLSKVStorage":
        if not hasattr(TLSKVStorage.tls, "his") or len(TLSKVStorage.tls.his) == 0:
            TLSKVStorage.tls.his = [TLSKVStorage()]
        return TLSKVStorage.tls.his[-1]

    @staticmethod
    def frozen() -> None:
        """Freeze current thread-local storage to global storage."""
        if hasattr(TLSKVStorage.tls, "his") and len(TLSKVStorage.tls.his) > 0:
            GLOBAL_STORAGE.update(TLSKVStorage.tls.his[-1].storage())


has_rust_backend: bool = False


def xxh64(*args: Any, **kwargs: Any) -> int:
    """Compute xxhash64 hash (Rust backend only)."""
    raise RuntimeError("hyperparameter is not built with rust backend")


try:
    if os.environ.get("HYPERPARAMETER_BACKEND", "RUST") == "RUST":
        from hyperparameter.librbackend import KVStorage, xxh64

        TLSKVStorage = KVStorage
        has_rust_backend = True
except:
    import traceback

    traceback.print_exc()
