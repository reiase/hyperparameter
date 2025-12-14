"""
pytest 配置和公共 fixtures

测试模块组织：
- test_scope.py: scope 基础功能（创建、访问、作用域、类型转换）
- test_param.py: @hp.param 装饰器
- test_scope_thread.py: 线程隔离
- test_scope_async_thread.py: 异步+线程混合
- test_stress_async_threads.py: 压力测试
- test_edge_cases.py: 边界条件测试
- test_launch.py: CLI launch 功能
- test_rust_backend.py: Rust 后端
- test_hash_consistency.py: hash 一致性
"""

import pytest
import hyperparameter as hp
from hyperparameter.storage import has_rust_backend


@pytest.fixture
def clean_scope():
    """提供一个干净的 scope 环境"""
    with hp.scope.empty() as ps:
        yield ps


@pytest.fixture
def nested_scope():
    """提供一个嵌套的 scope 环境"""
    with hp.scope(**{"level1.a": 1, "level1.b": 2}) as outer:
        with hp.scope(**{"level2.c": 3}) as inner:
            yield outer, inner


@pytest.fixture
def rust_backend_only():
    """跳过非 Rust 后端的测试"""
    if not has_rust_backend:
        pytest.skip("Rust backend required")


# 常用测试数据
SPECIAL_KEYS = [
    "a",
    "a.b",
    "a.b.c.d.e.f.g.h.i.j",  # 深度嵌套
    "CamelCase",
    "snake_case",
    "with-dash",
    "with123numbers",
    "UPPERCASE",
    "MixedCase123",
]

SPECIAL_VALUES = [
    0,
    -1,
    1,
    0.0,
    -0.0,
    1.0,
    -1.0,
    float("inf"),
    float("-inf"),
    "",
    "a",
    "hello world",
    True,
    False,
    None,
    [],
    {},
    [1, 2, 3],
    {"a": 1},
]

UNICODE_KEYS = [
    "中文key",
    "日本語",
    "한국어",
    "emoji🚀",
    "Ελληνικά",
    "العربية",
]

LONG_KEYS = [
    "a" * 100,
    "a" * 1000,
    ".".join(["level"] * 50),  # 50 层嵌套
]
