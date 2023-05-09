from .api import auto_param, param_scope
import importlib.metadata

__all__ = ["param_scope", "auto_param"]

VERSION = importlib.metadata.version("hyperparameter")

import os
include = os.path.dirname(__file__)
try:
    import hyperparameter.rbackend
    lib = hyperparameter.rbackend.__file__
except:
    lib = ""