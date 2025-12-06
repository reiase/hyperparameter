import importlib.metadata
import os

from .api import auto_param, launch, param_scope, run_cli

__all__ = ["param_scope", "auto_param", "launch", "run_cli"]


def _load_version() -> str:
    try:
        return importlib.metadata.version("hyperparameter")
    except importlib.metadata.PackageNotFoundError:
        env_version = os.environ.get("HYPERPARAMETER_VERSION")
        if env_version:
            return env_version
        return "0.0.0"


VERSION = _load_version()

include = os.path.dirname(__file__)
try:
    import hyperparameter.librbackend

    lib = hyperparameter.librbackend.__file__
# trunk-ignore(flake8/E722)
except:
    lib = ""
