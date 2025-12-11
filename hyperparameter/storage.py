from __future__ import annotations

import os
import threading
from contextvars import ContextVar
from typing import Any, Callable, Dict, Iterable, Optional, Iterator, Tuple

GLOBAL_STORAGE: Dict[str, Any] = {}
GLOBAL_STORAGE_LOCK = threading.RLock()
_CTX_STACK: ContextVar[Tuple["TLSKVStorage", ...]] = ContextVar("_HP_CTX_STACK", default=())


def _get_ctx_stack() -> Tuple["TLSKVStorage", ...]:
    return _CTX_STACK.get()


def _push_ctx_stack(item: "TLSKVStorage") -> Tuple["TLSKVStorage", ...]:
    stack = _CTX_STACK.get()
    new_stack = stack + (item,)
    _CTX_STACK.set(new_stack)
    return new_stack


def _pop_ctx_stack() -> Tuple["TLSKVStorage", ...]:
    stack = _CTX_STACK.get()
    if not stack:
        return stack
    new_stack = stack[:-1]
    _CTX_STACK.set(new_stack)
    return new_stack


def _copy_storage(src: Any, dst: Any) -> None:
    """Best-effort copy from src to dst."""
    try:
        data = src.storage() if hasattr(src, "storage") else src
        if isinstance(data, dict) and hasattr(dst, "update"):
            dst.update(data)
            return
    except Exception:
        pass
    try:
        keys = src.keys()
        for k in keys:
            try:
                v = src.get(k)
                if hasattr(dst, "put"):
                    dst.put(k, v)
            except Exception:
                continue
    except Exception:
        pass


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

    def update(self, kws: Optional[Dict[str, Any]] = None) -> None:
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


class _DictStorage(Storage):
    """Pure Python implementation of a key-value storage"""

    __slots__ = ("_storage", "_parent")
    tls = threading.local()

    def __init__(self, parent: Optional["_DictStorage"] = None) -> None:
        self._storage: Optional[Dict[str, Any]] = None
        self._parent: Optional["_DictStorage"] = parent
        super().__init__()

        stack = _get_ctx_stack()
        if stack:
            parent = stack[-1].storage()
            _copy_storage(parent, self)
        else:
            with GLOBAL_STORAGE_LOCK:
                global_snapshot = dict(GLOBAL_STORAGE)
            self.update(global_snapshot)

    def __iter__(self) -> Iterator[Tuple[str, Any]]:
        if self._storage is None:
            return iter([])
        return iter(self._storage.items())

    def child(self) -> "_DictStorage":
        obj = _DictStorage(self)
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

    def update(self, kws: Optional[Dict[str, Any]] = None) -> None:
        if kws is None:
            return

        if self._storage is None:
            self._storage = {}

        storage = self._storage

        def _update(values: Dict[str, Any], prefix: Optional[str] = None) -> None:
            for k, v in values.items():
                key = f"{prefix}.{k}" if prefix is not None else f"{k}"
                if isinstance(v, dict):
                    _update(v, prefix=key)
                else:
                    storage[key] = v

        _update(kws, prefix=None)

    def clear(self):
        if self._storage is None:
            self._storage = {}
            return
        self._storage.clear()

    def get_entry(self, *args: Any, **kwargs: Any) -> Any:
        """Get entry by hash (Rust backend only)."""
        raise RuntimeError("hyperparameter is not built with rust backend")

    def get(self, name: str, accessor: Optional[Callable] = None) -> Any:
        if name in self.__slots__:
            return self.__dict__[name]
        curr: Optional["_DictStorage"] = self
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

    def enter(self) -> "_DictStorage":
        return self

    def exit(self) -> None:
        return None

    @staticmethod
    def current() -> "_DictStorage":
        return _DictStorage()

    @staticmethod
    def frozen() -> None:
        """Freeze current thread-local storage to global storage."""
        stack = _get_ctx_stack()
        if stack:
            with GLOBAL_STORAGE_LOCK:
                GLOBAL_STORAGE.update(stack[-1].storage())


has_rust_backend: bool = False
_BackendStorage = _DictStorage


def xxh64(*args: Any, **kwargs: Any) -> int:
    """Compute xxhash64 hash (Rust backend only)."""
    raise RuntimeError("hyperparameter is not built with rust backend")


try:
    if os.environ.get("HYPERPARAMETER_BACKEND", "RUST") == "RUST":
        from hyperparameter.librbackend import KVStorage, xxh64  # type: ignore

        _BackendStorage = KVStorage
        has_rust_backend = True
except Exception:
    # Fallback to pure-Python backend; avoid noisy tracebacks at import time.
    has_rust_backend = False
    print("Warning: Falling back to pure-Python backend for hyperparameter storage.")


class TLSKVStorage(Storage):
    """ContextVar-backed storage wrapper for both Python and Rust backends."""

    __slots__ = ("_inner", "_handler")

    def __init__(self, inner: Optional[Any] = None) -> None:
        stack = _get_ctx_stack()
        
        if inner is not None:
            self._inner = inner
        elif stack:
            # inherit from current context
            parent = stack[-1].storage()
            if hasattr(parent, "clone"):
                self._inner = parent.clone()
            else:
                cloned = _BackendStorage()
                _copy_storage(parent, cloned)
                self._inner = cloned
        else:
            self._inner = _BackendStorage()
            # seed from global
            with GLOBAL_STORAGE_LOCK:
                snapshot = dict(GLOBAL_STORAGE)
            _copy_storage(snapshot, self._inner)
        
        # Handler 直接使用 storage 对象的地址（id）
        # 这样比较非常快（整数比较），且唯一标识 storage 对象
        # 在 64 位系统上，id() 返回的是 int64
        self._handler = id(self._inner)
        
        # 设置 Rust 侧的 thread-local handler（关键！）
        self._set_rust_handler(self._handler)
    
    def _set_rust_handler(self, handler: Optional[int]) -> None:
        """设置 Rust 侧的 thread-local handler
        handler 是 storage 对象的地址（id(storage)）
        """
        if has_rust_backend:
            try:
                from hyperparameter.librbackend import set_python_handler
                set_python_handler(handler)  # 直接写入 Rust thread-local
            except Exception:
                # 如果 Rust 后端不可用，忽略
                pass

    def __iter__(self) -> Iterator[Tuple[str, Any]]:
        return iter(self._inner)

    def child(self) -> "TLSKVStorage":
        if hasattr(self._inner, "child"):
            return TLSKVStorage(self._inner.child())
        # Best-effort new storage
        return TLSKVStorage(_BackendStorage())

    def storage(self) -> Any:
        if hasattr(self._inner, "storage"):
            return self._inner.storage()
        return self._inner

    def keys(self) -> Iterable[str]:
        return self._inner.keys()

    def update(self, kws: Optional[Dict[str, Any]] = None) -> None:
        # 确保 Rust 侧 handler 是最新的
        self._set_rust_handler(self._handler)
        return self._inner.update(kws)

    def clear(self) -> None:
        return self._inner.clear()

    def get_entry(self, *args: Any, **kwargs: Any) -> Any:
        if hasattr(self._inner, "get_entry"):
            return self._inner.get_entry(*args, **kwargs)
        raise RuntimeError("get_entry not supported without rust backend")

    def get(self, name: str, accessor: Optional[Callable] = None) -> Any:
        # 确保 Rust 侧 handler 是最新的
        self._set_rust_handler(self._handler)
        return self._inner.get(name, accessor) if accessor else self._inner.get(name)

    def put(self, name: str, value: Any) -> None:
        # 确保 Rust 侧 handler 是最新的
        self._set_rust_handler(self._handler)
        return self._inner.put(name, value)

    def enter(self) -> "TLSKVStorage":
        # 先设置 Rust 侧的 handler
        self._set_rust_handler(self._handler)
        
        if hasattr(self._inner, "enter"):
            self._inner.enter()
        _push_ctx_stack(self)
        return self

    def exit(self) -> None:
        if hasattr(self._inner, "exit"):
            self._inner.exit()
        stack = _get_ctx_stack()
        if stack and stack[-1] is self:
            _pop_ctx_stack()
            # 退出时，恢复父级 handler 或清空
            if stack:
                parent_handler = stack[-1]._handler if hasattr(stack[-1], '_handler') else None
                self._set_rust_handler(parent_handler)
            else:
                self._set_rust_handler(None)

    @staticmethod
    def current() -> "TLSKVStorage":
        stack = _get_ctx_stack()
        if not stack:
            ts = TLSKVStorage()
            _push_ctx_stack(ts)
            return ts
        return stack[-1]

    @staticmethod
    def frozen() -> None:
        stack = _get_ctx_stack()
        if stack:
            with GLOBAL_STORAGE_LOCK:
                GLOBAL_STORAGE.update(stack[-1].storage())
