import importlib.metadata

from .api import auto_param, param_scope
from .debug import DebugConsole

__all__ = ["param_scope", "auto_param", "DebugConsole"]

VERSION = importlib.metadata.version("hyperparameter")

# trunk-ignore(flake8/E402)
import os

include = os.path.dirname(__file__)
try:
    import hyperparameter.librbackend

    lib = hyperparameter.librbackend.__file__
# trunk-ignore(flake8/E722)
except:
    lib = ""
