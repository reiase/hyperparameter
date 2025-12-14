import importlib.metadata
import os
import warnings

from .api import scope, param, launch
from .loader import config

__all__ = ["scope", "param", "launch", "config"]


def _load_version() -> str:
    try:
        return importlib.metadata.version("hyperparameter")
    except importlib.metadata.PackageNotFoundError:
        env_version = os.environ.get("HYPERPARAMETER_VERSION")
        if env_version:
            return env_version
        warnings.warn(
            "hyperparameter package metadata not found; falling back to 0.0.0+local. "
            "Install the package or use `pip install -e .` for an accurate version.",
        )
        return "0.0.0+local"


VERSION = _load_version()

include = os.path.dirname(__file__)
try:
    import hyperparameter.librbackend

    lib = hyperparameter.librbackend.__file__
# trunk-ignore(flake8/E722)
except:
    lib = ""
