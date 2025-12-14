import inspect
import json
import os
import re
import typing
import warnings
from typing import Any, Dict, List, Set, Type, TypeVar, Union

T = TypeVar("T")


def _merge_dicts(base: Dict[str, Any], override: Dict[str, Any]) -> Dict[str, Any]:
    """Recursively merge two dictionaries.

    Values in `override` overwrite values in `base`.
    If both values are dictionaries, they are merged recursively.
    """
    merged = base.copy()
    for key, value in override.items():
        if key in merged and isinstance(merged[key], dict) and isinstance(value, dict):
            merged[key] = _merge_dicts(merged[key], value)
        else:
            merged[key] = value
    return merged


def _flatten_dict(
    d: Dict[str, Any], parent_key: str = "", sep: str = "."
) -> Dict[str, Any]:
    """Flatten a nested dictionary."""
    items: List[Any] = []
    for k, v in d.items():
        new_key = f"{parent_key}{sep}{k}" if parent_key else k
        if isinstance(v, dict):
            items.extend(_flatten_dict(v, new_key, sep=sep).items())
        else:
            items.append((new_key, v))
    return dict(items)


def _get_value_by_path(d: Dict[str, Any], path: str) -> Any:
    """Get value from nested dictionary by dot-separated path."""
    keys = path.split(".")
    curr = d
    for key in keys:
        if not isinstance(curr, dict) or key not in curr:
            raise KeyError(path)
        curr = curr[key]
    return curr


def _resolve_interpolations(config: Dict[str, Any]) -> Dict[str, Any]:
    """Resolve variable interpolations in the format ${variable.name}."""

    # Regex to match ${...}
    pattern = re.compile(r"\$\{([^\}]+)\}")

    # We need to resolve values iteratively until no more changes occur
    # To prevent infinite loops (cycles), we'll limit iterations
    max_iterations = 100

    # Helper to check if a value contains interpolation
    def has_interpolation(val: Any) -> bool:
        return isinstance(val, str) and "${" in val

    # Helper to resolve a single string value
    def resolve_value(val: str, root_config: Dict[str, Any], history: Set[str]) -> Any:
        if not isinstance(val, str):
            return val

        matches = list(pattern.finditer(val))
        if not matches:
            return val

        # Case 1: The value is exactly "${key}" (preserve type)
        if len(matches) == 1 and matches[0].group(0) == val:
            key = matches[0].group(1)
            if key in history:
                raise ValueError(
                    f"Circular dependency detected: {' -> '.join(history)} -> {key}"
                )

            try:
                ref_val = _get_value_by_path(root_config, key)
            except KeyError:
                raise KeyError(f"Interpolation key '{key}' not found")

            # If the referenced value itself has interpolation, resolve it recursively
            if has_interpolation(ref_val):
                return resolve_value(ref_val, root_config, history | {key})
            return ref_val

        # Case 2: String interpolation "prefix ${key} suffix" (always string)
        result = val
        # Process matches in reverse order to keep indices valid?
        # Actually string replace is safer since we don't know the length of replacement

        # We need to find all keys first
        keys_to_replace = []
        for match in matches:
            keys_to_replace.append(match.group(1))

        for key in keys_to_replace:
            if key in history:
                raise ValueError(
                    f"Circular dependency detected: {' -> '.join(history)} -> {key}"
                )

            try:
                ref_val = _get_value_by_path(root_config, key)
            except KeyError:
                raise KeyError(f"Interpolation key '{key}' not found")

            # If referenced value needs resolution
            if has_interpolation(ref_val):
                ref_val = resolve_value(ref_val, root_config, history | {key})

            # Convert to string for concatenation
            result = result.replace(f"${{{key}}}", str(ref_val))

        return result

    # Traverse and replace
    def traverse_and_resolve(
        node: Any, root_config: Dict[str, Any], history: Set[str]
    ) -> Any:
        if isinstance(node, dict):
            return {
                k: traverse_and_resolve(v, root_config, history)
                for k, v in node.items()
            }
        elif isinstance(node, list):
            return [traverse_and_resolve(item, root_config, history) for item in node]
        elif isinstance(node, str):
            if "${" in node:
                return resolve_value(node, root_config, history)
            return node
        else:
            return node

    return traverse_and_resolve(config, config, set())


def _coerce_type(value: Any, target_type: Any) -> Any:
    """Coerce value to target_type."""
    if value is None:
        return None

    if target_type is Any:
        return value

    # Handle typing.Optional
    origin = getattr(target_type, "__origin__", None)
    if origin is Union:
        args = getattr(target_type, "__args__", [])
        if type(None) in args:
            # It's Optional[T]
            non_none_args = [arg for arg in args if arg is not type(None)]
            if len(non_none_args) == 1:
                return _coerce_type(value, non_none_args[0])
        # Other Unions are not supported yet, return as is
        return value

    # Handle typing.List
    if origin is list:
        args = getattr(target_type, "__args__", [])
        item_type = args[0] if args else Any
        if isinstance(value, list):
            return [_coerce_type(item, item_type) for item in value]
        # Try to parse string as list? "[1, 2]" -> [1, 2]
        # For now, strict list requirement or single item wrapping?
        # Let's assume input must be list if target is list
        if not isinstance(value, list):
            # Try simple comma separation for string input?
            if isinstance(value, str):
                # simplistic split, mainly for CLI args
                return [
                    _coerce_type(item.strip(), item_type) for item in value.split(",")
                ]
        return value

    # Handle typing.Dict
    if origin is dict:
        args = getattr(target_type, "__args__", [])
        key_type = args[0] if args else Any
        val_type = args[1] if len(args) > 1 else Any
        if isinstance(value, dict):
            return {
                _coerce_type(k, key_type): _coerce_type(v, val_type)
                for k, v in value.items()
            }
        return value

    # Handle basic types
    if target_type is int:
        return int(float(value)) if isinstance(value, (str, float)) else int(value)
    if target_type is float:
        return float(value)
    if target_type is bool:
        if isinstance(value, str):
            return value.lower() in ("true", "1", "yes", "on", "t")
        return bool(value)
    if target_type is str:
        return str(value)

    # Handle Nested Class (Dataclass-like or simple class with annotations)
    if isinstance(target_type, type) and hasattr(target_type, "__annotations__"):
        if isinstance(value, dict):
            return validate(value, target_type)

    return value


def validate(data: Dict[str, Any], schema_cls: Type[T]) -> T:
    """Validate and coerce configuration dictionary against a class schema.

    This function creates an instance of `schema_cls` populated with values from `data`.
    It performs type coercion based on type hints in `schema_cls`.

    Args:
        data: Configuration dictionary.
        schema_cls: Class with type annotations defining the schema.

    Returns:
        Instance of schema_cls.

    Raises:
        ValueError: If required fields are missing.
        TypeError: If type coercion fails.
    """
    if not isinstance(data, dict):
        raise TypeError(f"Config must be a dictionary, got {type(data)}")

    # Create instance
    # We don't call __init__ to avoid requiring specific signature
    # We construct the object and set attributes
    instance = object.__new__(schema_cls)

    annotations = typing.get_type_hints(schema_cls)

    for name, type_hint in annotations.items():
        # Check if field exists
        if name in data:
            raw_value = data[name]
            try:
                coerced_value = _coerce_type(raw_value, type_hint)
                setattr(instance, name, coerced_value)
            except (ValueError, TypeError) as e:
                raise TypeError(f"Failed to convert field '{name}' to {type_hint}: {e}")
        else:
            # Check for default value in class definition
            if hasattr(schema_cls, name):
                default = getattr(schema_cls, name)
                # handle dataclass field default?
                # For standard class variable defaults
                setattr(instance, name, default)
            else:
                # Handle Optional without default (should be None?)
                # If type is Optional and no default provided, we usually expect None or missing is error?
                # Python doesn't enforce Optional = None default automatically.
                # But typically Optional fields are nullable.
                origin = getattr(type_hint, "__origin__", None)
                is_optional = False
                if origin is Union:
                    args = getattr(type_hint, "__args__", [])
                    if type(None) in args:
                        is_optional = True

                if is_optional:
                    setattr(instance, name, None)
                else:
                    raise ValueError(f"Missing required field: '{name}'")

    return instance


def _load_single_file(path: str) -> Dict[str, Any]:
    """Load a single configuration file based on extension."""
    ext = os.path.splitext(path)[1].lower()

    if ext == ".json":
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)

    elif ext in (".yaml", ".yml"):
        try:
            import yaml
        except ImportError:
            raise ImportError(
                "PyYAML is required to load .yaml/.yml files. Please install it with `pip install PyYAML`."
            )
        with open(path, "r", encoding="utf-8") as f:
            return yaml.safe_load(f)

    # Default to TOML for .toml or unknown extensions (for backward compatibility)
    try:
        import toml
    except ImportError:
        warnings.warn(
            "package toml is required by hyperparameter, please install toml with `pip install toml`"
        )
        raise

    with open(path, "r", encoding="utf-8") as f:
        return toml.load(f)


class _ConfigLoader:
    """Config loader that can be used as context manager."""

    def __init__(
        self,
        path: Union[str, List[str], Dict[str, Any]],
        schema: typing.Optional[Type[T]] = None,
    ):
        self._path = path
        self._schema = schema
        self._config: Union[Dict[str, Any], T, None] = None
        self._scope = None

    def _load(self) -> Union[Dict[str, Any], T]:
        """Load and return the configuration."""
        cfg: Dict[str, Any] = {}

        if isinstance(self._path, dict):
            cfg = self._path
        elif isinstance(self._path, str):
            cfg = _load_single_file(self._path)
        elif isinstance(self._path, list):
            for p in self._path:
                new_config = _load_single_file(p)
                cfg = _merge_dicts(cfg, new_config)
        else:
            raise TypeError(
                f"path must be a string, a list of strings, or a dict, got {type(self._path)}"
            )

        # Apply interpolation
        cfg = _resolve_interpolations(cfg)

        # Apply validation if schema provided
        if self._schema is not None:
            return validate(cfg, self._schema)

        return cfg

    def __enter__(self) -> "scope":
        """Enter context and inject config into scope."""
        from .api import scope

        if self._config is None:
            self._config = self._load()

        # If schema was used, convert to dict for scope
        if isinstance(self._config, dict):
            config_dict = self._config
        else:
            # Convert validated object to dict
            config_dict = {
                k: getattr(self._config, k)
                for k in typing.get_type_hints(type(self._config)).keys()
            }

        self._scope = scope(**config_dict)
        return self._scope.__enter__()

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context."""
        if self._scope is not None:
            return self._scope.__exit__(exc_type, exc_val, exc_tb)


def config(
    path: Union[str, List[str], Dict[str, Any]],
    schema: typing.Optional[Type[T]] = None,
) -> Union[Dict[str, Any], T, _ConfigLoader]:
    """Load configuration from a file or a list of files.

    Can be used in two ways:

    1. As a function call returning config dict/object:
       >>> cfg = config("config.toml")
       >>> cfg = config(["base.toml", "override.toml"])
       >>> cfg = config("config.toml", schema=AppConfig)

    2. As a context manager (auto-inject into scope):
       >>> with config("config.toml"):
       ...     train()

    If a list of files is provided, they are loaded in order and merged.
    Later files override earlier ones. Nested dictionaries are merged recursively.

    Supports variable interpolation: ${key.subkey}

    Args:
        path: Single file path, list of file paths, or a dictionary config.
        schema: Optional class with type annotations for validation and coercion.

    Supported formats:
    - JSON (.json)
    - YAML (.yaml, .yml) - requires PyYAML
    - TOML (.toml) - requires toml (default)
    """
    loader = _ConfigLoader(path, schema)
    return loader._load()


# Keep load and loads for backward compatibility and direct usage
def load(
    path: Union[str, List[str], Dict[str, Any]], schema: typing.Optional[Type[T]] = None
) -> Union[Dict[str, Any], T]:
    """Load configuration from a file or a list of files.

    If a list of files is provided, they are loaded in order and merged.
    Later files override earlier ones. Nested dictionaries are merged recursively.

    Supports variable interpolation: ${key.subkey}

    Args:
        path: Single file path, list of file paths, or a dictionary config.
        schema: Optional class with type annotations for validation and coercion.

    Supported formats:
    - JSON (.json)
    - YAML (.yaml, .yml) - requires PyYAML
    - TOML (.toml) - requires toml (default)
    """
    return config(path, schema)


def loads(cfg: str):
    try:
        import toml
    except Exception as e:
        warnings.warn(
            "package toml is required by hyperparameter, please install toml with `pip install toml`"
        )
        raise e
    val = toml.loads(cfg)
    return _resolve_interpolations(val)


def dumps(cfg) -> str:
    try:
        import toml
    except Exception as e:
        warnings.warn(
            "package toml is required by hyperparameter, please install toml with `pip install toml`"
        )
        raise e
    return toml.dumps(cfg)
