import typing
from hyperparameter import loader
import pytest


def test_validate_simple_types():
    config = {
        "lr": "0.01",  # String, should be converted to float
        "batch_size": "32",  # String, should be converted to int
        "enable_logging": "true",  # String, should be converted to bool
    }

    class TrainConfig:
        lr: float
        batch_size: int
        enable_logging: bool

    validated = loader.validate(config, TrainConfig)

    assert validated.lr == 0.01
    assert isinstance(validated.lr, float)
    assert validated.batch_size == 32
    assert isinstance(validated.batch_size, int)
    assert validated.enable_logging is True
    assert isinstance(validated.enable_logging, bool)


def test_validate_nested_class():
    config = {"server": {"port": "8080"}}

    class ServerConfig:
        port: int

    class AppConfig:
        server: ServerConfig

    validated = loader.validate(config, AppConfig)

    assert validated.server.port == 8080
    assert isinstance(validated.server, ServerConfig)


def test_validate_nested_dict_annotation():
    config = {"params": {"a": "1", "b": "2"}}

    class ModelConfig:
        params: typing.Dict[str, int]

    validated = loader.validate(config, ModelConfig)

    assert validated.params["a"] == 1
    assert validated.params["b"] == 2


def test_validate_list_annotation():
    config = {"layers": ["128", "256"]}

    class NetConfig:
        layers: typing.List[int]

    validated = loader.validate(config, NetConfig)

    assert validated.layers == [128, 256]
    assert isinstance(validated.layers[0], int)


def test_validate_missing_field():
    config = {"a": 1}

    class Config:
        a: int
        b: int

    with pytest.raises(ValueError, match="Missing required field"):
        loader.validate(config, Config)


def test_validate_optional_field():
    config = {"a": 1}

    class Config:
        a: int
        b: typing.Optional[int] = None

    validated = loader.validate(config, Config)
    assert validated.a == 1
    assert validated.b is None


def test_validate_extra_fields_ignored():
    config = {"a": 1, "unknown": 2}

    class Config:
        a: int

    validated = loader.validate(config, Config)
    assert validated.a == 1
    assert not hasattr(validated, "unknown")
