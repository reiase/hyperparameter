from .api import auto_param, param_scope
import importlib.metadata
from .debug import DebugConsole

__all__ = ["param_scope", "auto_param", "DebugConsole"]

VERSION = importlib.metadata.version("hyperparameter")

import os
include = os.path.dirname(__file__)
try:
    import hyperparameter.librbackend
    lib = hyperparameter.librbackend.__file__
except:
    lib = ""