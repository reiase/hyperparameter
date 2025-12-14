import pytest
import hyperparameter as hp
from hyperparameter.loader import _resolve_interpolations


def test_interpolation_basic():
    config = {
        "server": {"host": "localhost", "port": 8080},
        "database": {"url": "http://${server.host}:${server.port}/db"},
        "service": {"name": "my-service", "full_name": "${service.name}-v1"},
    }

    resolved = _resolve_interpolations(config)

    assert resolved["database"]["url"] == "http://localhost:8080/db"
    assert resolved["service"]["full_name"] == "my-service-v1"


def test_interpolation_type_preservation():
    config = {
        "a": 100,
        "b": "${a}",  # Should preserve int type
        "c": "value is ${a}",  # Should become string
    }

    resolved = _resolve_interpolations(config)

    assert resolved["b"] == 100
    assert isinstance(resolved["b"], int)
    assert resolved["c"] == "value is 100"


def test_interpolation_nested():
    config = {"a": "A", "b": {"c": "${a}", "d": {"e": "${b.c}"}}}

    resolved = _resolve_interpolations(config)
    assert resolved["b"]["d"]["e"] == "A"


def test_interpolation_missing_key():
    config = {"a": "${missing_key}"}
    with pytest.raises(KeyError):
        _resolve_interpolations(config)


def test_interpolation_circular():
    config = {"a": "${b}", "b": "${a}"}
    with pytest.raises(ValueError, match="Circular dependency"):
        _resolve_interpolations(config)
