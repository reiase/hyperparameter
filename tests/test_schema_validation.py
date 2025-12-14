import dataclasses
from typing import Any, Dict, Type, Union

import pytest
from hyperparameter import loader


@dataclasses.dataclass
class ServerConfig:
    host: str = "localhost"
    port: int = 8080


@dataclasses.dataclass
class AppConfig:
    name: str
    server: ServerConfig
    debug: bool = False


def test_schema_validation_basic():
    config = {
        "name": "my-app",
        "server": {"host": "127.0.0.1", "port": 9090},
        "debug": True,
    }

    # Load with schema
    loaded = loader.load(config, schema=AppConfig)

    assert isinstance(loaded, AppConfig)
    assert loaded.name == "my-app"
    assert isinstance(loaded.server, ServerConfig)
    assert loaded.server.port == 9090
    assert loaded.debug is True


def test_schema_validation_type_error():
    config = {"name": "my-app", "server": {"port": "invalid-port"}}  # Should be int

    # Depending on implementation (pydantic vs pure dataclass), this might raise varying errors
    # We'll assume strict typing or at least conversion failure raises error
    # dacite or similar library usually raises TypeError or custom error
    # For now, let's just assert it raises *some* exception
    with pytest.raises(Exception):
        loader.load(config, schema=AppConfig)


def test_schema_validation_missing_required():
    config = {
        "server": {}
        # missing 'name' which is required in AppConfig
    }

    with pytest.raises(Exception):
        loader.load(config, schema=AppConfig)


def test_load_without_schema():
    # Backward compatibility
    config = {"a": 1}
    loaded = loader.load(config)
    assert isinstance(loaded, dict)
    assert loaded["a"] == 1
