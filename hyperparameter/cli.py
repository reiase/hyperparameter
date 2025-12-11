"""CLI support for hyperparameter auto_param functions."""

from __future__ import annotations

import argparse
import inspect
import sys
from typing import Any, Callable, Dict, List, Optional, Tuple

# Import param_scope locally to avoid circular import
# param_scope is defined in api.py, but we import it here to avoid circular dependency
def _get_param_scope():
    """Lazy import of param_scope to avoid circular imports."""
    from .api import param_scope
    return param_scope


def _parse_param_help(doc: Optional[str]) -> Dict[str, str]:
    """Parse param help from docstring (Google/NumPy/reST)."""
    if not doc:
        return {}
    lines = [line.rstrip() for line in doc.splitlines()]
    help_map: Dict[str, str] = {}

    # Google style: Args:/Arguments:
    def parse_google():
        in_args = False
        for line in lines:
            if not in_args:
                if line.strip().lower() in ("args:", "arguments:"):
                    in_args = True
                continue
            if line.strip() == "":
                if in_args:
                    break
                continue
            if not line.startswith(" "):
                break
            stripped = line.strip()
            if ":" in stripped:
                name_part, desc = stripped.split(":", 1)
                name_part = name_part.strip()
                if "(" in name_part and ")" in name_part:
                    name_part = name_part.split("(")[0].strip()
                if name_part:
                    help_map.setdefault(name_part, desc.strip())

    # NumPy style: Parameters
    def parse_numpy():
        in_params = False
        current_name = None
        for line in lines:
            if not in_params:
                if line.strip().lower() == "parameters":
                    in_params = True
                continue
            if line.strip() == "":
                if current_name is not None:
                    current_name = None
                continue
            if not line.startswith(" "):
                # section ended
                break
            # parameter line: name : type
            if ":" in line:
                name_part = line.split(":", 1)[0].strip()
                current_name = name_part
                # description may follow on same line after type, but we skip
                if current_name and current_name not in help_map:
                    # next indented lines are description
                    continue
            elif current_name:
                desc = line.strip()
                if desc:
                    help_map.setdefault(current_name, desc)

    # reST/Sphinx: :param name: desc
    def parse_rest():
        for line in lines:
            striped = line.strip()
            if striped.startswith(":param"):
                # forms: :param name: desc  or :param type name: desc
                parts = striped.split(":param", 1)[1].strip()
                if ":" in parts:
                    before, desc = parts.split(":", 1)
                    tokens = before.split()
                    name = tokens[-1] if tokens else ""
                    if name:
                        help_map.setdefault(name, desc.strip())

    parse_google()
    parse_numpy()
    parse_rest()
    return help_map


def _arg_type_from_default(default: Any) -> Optional[Callable[[str], Any]]:
    if isinstance(default, bool):
        def _to_bool(v: str) -> bool:
            return v.lower() in ("1", "true", "t", "yes", "y", "on")
        return _to_bool
    if default is None:
        return None
    return type(default)


def _extract_first_paragraph(docstring: Optional[str]) -> Optional[str]:
    """Extract the first paragraph from a docstring for cleaner help output.
    
    The first paragraph is defined as text up to the first blank line or
    the first line that starts with common docstring section markers like
    'Args:', 'Returns:', 'Examples:', etc.
    """
    if not docstring:
        return None
    
    lines = docstring.strip().split('\n')
    first_paragraph = []
    
    for line in lines:
        stripped = line.strip()
        # Stop at blank lines
        if not stripped:
            break
        # Stop at common docstring section markers
        if stripped.lower() in ('args:', 'arguments:', 'parameters:', 'returns:', 
                                'raises:', 'examples:', 'note:', 'warning:', 
                                'see also:', 'todo:'):
            break
        first_paragraph.append(stripped)
    
    result = ' '.join(first_paragraph).strip()
    return result if result else None


def _find_related_auto_param_functions(func: Callable, caller_globals: Optional[Dict] = None) -> List[Tuple[str, Callable]]:
    """Find all @auto_param functions related to the given function's namespace.
    
    Returns a list of (full_namespace, function) tuples.
    """
    namespace = getattr(func, "_auto_param_namespace", func.__name__)
    if not isinstance(namespace, str):
        return []
    
    # Extract base namespace (e.g., "transformers" from "transformers.runtime")
    base_ns = namespace.split(".")[0]
    
    related = []
    seen = set()
    
    # Check caller_globals (current module)
    if caller_globals:
        for obj in caller_globals.values():
            if not callable(obj) or id(obj) in seen:
                continue
            seen.add(id(obj))
            obj_ns = getattr(obj, "_auto_param_namespace", None)
            if isinstance(obj_ns, str) and obj_ns.startswith(base_ns + ".") and obj_ns != namespace:
                related.append((obj_ns, obj))
        
        # Check imported modules
        for name, obj in caller_globals.items():
            if inspect.ismodule(obj):
                try:
                    for attr_name in dir(obj):
                        if attr_name.startswith("_"):
                            continue
                        try:
                            attr = getattr(obj, attr_name, None)
                            if callable(attr) and id(attr) not in seen:
                                seen.add(id(attr))
                                obj_ns = getattr(attr, "_auto_param_namespace", None)
                                if isinstance(obj_ns, str) and obj_ns.startswith(base_ns + ".") and obj_ns != namespace:
                                    related.append((obj_ns, attr))
                        except (AttributeError, TypeError):
                            continue
                except Exception:
                    continue
    
    # Also check the function's own module and related modules in the same package
    func_module = getattr(func, "__module__", None)
    modules_to_check = []
    
    if func_module and func_module in sys.modules:
        modules_to_check.append(sys.modules[func_module])
    
    # Check for related modules in the same package
    # e.g., if func is in pulsing.cli.__main__, check pulsing.cli.transformers_backend
    if func_module:
        module_parts = func_module.split(".")
        if len(module_parts) > 1:
            package_name = ".".join(module_parts[:-1])
            
            # Try to find backend modules in the same package
            # Check all modules in sys.modules that are in the same package
            package_prefix = package_name + "."
            for mod_name, mod in sys.modules.items():
                if mod_name.startswith(package_prefix) and mod_name != func_module:
                    # Check if it's a backend module (contains _backend or backend in name)
                    if "_backend" in mod_name or mod_name.endswith("backend"):
                        if mod not in modules_to_check:
                            modules_to_check.append(mod)
            
            # Also try to import related backend modules if they exist but aren't loaded
            # This handles lazy imports. Try both absolute and relative import styles
            try:
                import importlib
                # Try common backend module names with different patterns
                backend_patterns = [
                    f"{package_name}.transformers_backend",
                    f"{package_name}.vllm_backend",
                ]
                # Add base_ns specific backend if base_ns is available
                if base_ns:
                    backend_patterns.append(f"{package_name}.{base_ns}_backend")
                
                for backend_name in backend_patterns:
                    if backend_name not in sys.modules:
                        try:
                            mod = importlib.import_module(backend_name)
                            if mod not in modules_to_check:
                                modules_to_check.append(mod)
                        except (ImportError, ModuleNotFoundError, ValueError):
                            pass
            except Exception:
                pass
    
    # Check all identified modules
    for mod in modules_to_check:
        try:
            for attr_name in dir(mod):
                if attr_name.startswith("_"):
                    continue
                try:
                    attr = getattr(mod, attr_name, None)
                    if callable(attr) and id(attr) not in seen:
                        seen.add(id(attr))
                        obj_ns = getattr(attr, "_auto_param_namespace", None)
                        if isinstance(obj_ns, str) and obj_ns.startswith(base_ns + ".") and obj_ns != namespace:
                            related.append((obj_ns, attr))
                except (AttributeError, TypeError):
                    continue
        except Exception:
            continue
    
    # Sort by namespace for consistent output
    related.sort(key=lambda x: x[0])
    return related


def _format_advanced_params_help(related_funcs: List[Tuple[str, Callable]]) -> str:
    """Format help text for advanced parameters available via -D."""
    if not related_funcs:
        return ""
    
    lines = ["\nAdvanced parameters (via -D flag):"]
    lines.append("  Use -D <namespace>.<param>=<value> to configure advanced options.")
    lines.append("")
    
    for full_ns, related_func in related_funcs:
        sig = inspect.signature(related_func)
        param_help = _parse_param_help(related_func.__doc__)
        
        # Get function description
        func_desc = _extract_first_paragraph(related_func.__doc__) or related_func.__name__
        lines.append(f"  {full_ns}:")
        lines.append(f"    {func_desc}")
        
        for name, param in sig.parameters.items():
            # Skip VAR_KEYWORD and VAR_POSITIONAL
            if param.kind == inspect.Parameter.VAR_KEYWORD or param.kind == inspect.Parameter.VAR_POSITIONAL:
                continue
            
            help_text = param_help.get(name, "")
            default = param.default if param.default is not inspect._empty else None
            
            param_key = f"{full_ns}.{name}"
            if help_text:
                help_text = help_text.split("\n")[0].strip()  # First line only
                if default is not None:
                    lines.append(f"    -D {param_key}=<value>  {help_text} (default: {default})")
                else:
                    lines.append(f"    -D {param_key}=<value>  {help_text}")
            else:
                if default is not None:
                    lines.append(f"    -D {param_key}=<value>  (default: {default})")
                else:
                    lines.append(f"    -D {param_key}=<value>")
        
        lines.append("")
    
    return "\n".join(lines)


def _build_parser_for_func(func: Callable, prog: Optional[str] = None, caller_globals: Optional[Dict] = None) -> argparse.ArgumentParser:
    sig = inspect.signature(func)
    # Use first paragraph of docstring for cleaner help output
    description = _extract_first_paragraph(func.__doc__) or func.__doc__
    
    # Find related @auto_param functions for advanced parameters help
    related_funcs = _find_related_auto_param_functions(func, caller_globals) if caller_globals else []
    epilog = _format_advanced_params_help(related_funcs) if related_funcs else None
    
    parser = argparse.ArgumentParser(
        prog=prog or func.__name__,
        description=description,
        epilog=epilog,
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend", help="Override params, e.g., a.b=1")
    parser.add_argument(
        "-lps",
        "--list-params",
        action="store_true",
        help="List parameter names, defaults, and current values (after --define overrides), then exit.",
    )
    parser.add_argument(
        "-ep",
        "--explain-param",
        nargs="*",
        metavar="NAME",
        help="Explain the source of specific parameters (default, CLI arg, or --define override), then exit. If omitted, prints all.",
    )
    param_help = _parse_param_help(func.__doc__)

    for name, param in sig.parameters.items():
        if param.default is inspect._empty:
            parser.add_argument(name, type=param.annotation if param.annotation is not inspect._empty else str, help=param_help.get(name))
        else:
            arg_type = _arg_type_from_default(param.default)
            help_text = param_help.get(name)
            if help_text:
                help_text = f"{help_text} (default: {param.default})"
            else:
                help_text = f"(default from auto_param: {param.default})"
            parser.add_argument(
                f"--{name}",
                dest=name,
                type=arg_type,
                default=argparse.SUPPRESS,
                help=help_text,
            )
    return parser


def _describe_parameters(func: Callable, defines: List[str], arg_overrides: Dict[str, Any]) -> List[Tuple[str, str, str, Any, str, Any]]:
    """Return [(func_name, param_name, full_key, value, source, default)] under current overrides."""
    namespace = getattr(func, "_auto_param_namespace", func.__name__)
    func_name = getattr(func, "__name__", namespace)
    sig = inspect.signature(func)
    results: List[Tuple[str, str, str, Any, str, Any]] = []
    _MISSING = object()
    ps = _get_param_scope()
    with ps(*defines) as hp:
        storage_snapshot = hp.storage().storage()
        for name, param in sig.parameters.items():
            default = param.default if param.default is not inspect._empty else _MISSING
            if name in arg_overrides:
                value = arg_overrides[name]
                source = "cli-arg"
            else:
                full_key = f"{namespace}.{name}"
                in_define = full_key in storage_snapshot
                if default is _MISSING:
                    value = "<required>"
                else:
                    value = getattr(hp(), full_key).get_or_else(default)
                source = "--define" if in_define else ("default" if default is not _MISSING else "required")
            printable_default = "<required>" if default is _MISSING else default
            results.append((func_name, name, full_key, value, source, printable_default))
    return results


def _maybe_explain_and_exit(func: Callable, args_dict: Dict[str, Any], defines: List[str]) -> bool:
    list_params = bool(args_dict.pop("list_params", False))
    explain_targets = args_dict.pop("explain_param", None)
    if explain_targets is not None and len(explain_targets) == 0:
        print("No parameter names provided to --explain-param. Please specify at least one.")
        sys.exit(1)
    if not list_params and not explain_targets:
        return False

    rows = _describe_parameters(func, defines, args_dict)
    target_set = set(explain_targets) if explain_targets is not None else None
    if explain_targets is not None and target_set is not None and all(full_key not in target_set for _, _, full_key, _, _, _ in rows):
        missing = ", ".join(explain_targets)
        print(f"No matching parameters for: {missing}")
        sys.exit(1)
    for func_name, name, full_key, value, source, default in rows:
        # Use fully qualified key for matching to avoid collisions.
        if target_set is not None and full_key not in target_set:
            continue
        default_repr = "<required>" if default == "<required>" else repr(default)
        func_module = getattr(func, "__module__", "unknown")
        location = f"{func_module}.{func_name}"
        print(f"{full_key}:")
        print(f"    function={func_name}, location={location}, default={default_repr}")
    return True


def launch(func: Optional[Callable] = None, *, _caller_globals=None, _caller_locals=None, _caller_module=None) -> None:
    """Launch CLI for @auto_param functions.

    - launch(f): expose a single @auto_param function f as CLI.
    - launch(): expose all @auto_param functions in the caller module as subcommands.

    Args:
        func: Optional function to launch. If None, discovers all @auto_param functions in caller module.
        _caller_globals: Explicitly pass caller's globals dict (for entry point support).
        _caller_locals: Explicitly pass caller's locals dict (for entry point support).
        _caller_module: Explicitly pass caller's module name or module object (for entry point support).
                       Can be a string (module name) or a module object.
    """
    if _caller_globals is None or _caller_locals is None:
        caller_frame = inspect.currentframe().f_back  # type: ignore
        if caller_frame is not None:
            caller_globals = caller_frame.f_globals
            caller_locals = caller_frame.f_locals
        else:
            # Fallback: try to find the caller module from sys.modules
            caller_globals = {}
            caller_locals = {}
            if _caller_module is not None:
                if isinstance(_caller_module, str):
                    if _caller_module in sys.modules:
                        mod = sys.modules[_caller_module]
                        caller_globals = mod.__dict__
                        caller_locals = mod.__dict__
                elif hasattr(_caller_module, '__dict__'):
                    caller_globals = _caller_module.__dict__
                    caller_locals = _caller_module.__dict__
            else:
                # Last resort: try to find the module that called us by walking the stack
                frame = inspect.currentframe()
                if frame is not None:
                    # Walk up the stack to find a module frame
                    current = frame.f_back
                    while current is not None:
                        globs = current.f_globals
                        # Check if this looks like a module (has __name__ and __file__)
                        if '__name__' in globs and '__file__' in globs:
                            caller_globals = globs
                            caller_locals = current.f_locals
                            break
                        current = current.f_back
    else:
        caller_globals = _caller_globals
        caller_locals = _caller_locals

    if func is None:
        seen_ids = set()
        candidates = []
        for obj in list(caller_locals.values()) + list(caller_globals.values()):
            if not callable(obj):
                continue
            ns = getattr(obj, "_auto_param_namespace", None)
            if not isinstance(ns, str):
                continue
            # Skip private helpers (e.g., _foo) when exposing subcommands.
            name = getattr(obj, "__name__", "")
            if isinstance(name, str) and name.startswith("_"):
                continue
            oid = id(obj)
            if oid in seen_ids:
                continue
            seen_ids.add(oid)
            candidates.append(obj)
        if not candidates:
            raise RuntimeError("No @auto_param functions found to launch.")

        if len(candidates) == 1:
            import sys
            
            func = candidates[0]
            parser = _build_parser_for_func(func, caller_globals=caller_globals)
            argv = sys.argv[1:]
            if argv and argv[0] == func.__name__:
                argv = argv[1:]
            args = parser.parse_args(argv)
            args_dict = vars(args)
            defines = args_dict.pop("define", [])
            if _maybe_explain_and_exit(func, args_dict, defines):
                return None
            param_scope = _get_param_scope()
            with param_scope(*defines):
                return func(**args_dict)

        parser = argparse.ArgumentParser(description="hyperparameter auto-param CLI")
        subparsers = parser.add_subparsers(dest="command", required=True)
        func_map: Dict[str, Callable] = {}
        for f in candidates:
            # Use first paragraph of docstring for cleaner help output
            help_text = _extract_first_paragraph(f.__doc__) or f.__doc__
            
            # Find related @auto_param functions for advanced parameters help
            related_funcs = _find_related_auto_param_functions(f, caller_globals)
            epilog = _format_advanced_params_help(related_funcs) if related_funcs else None
            
            sub = subparsers.add_parser(
                f.__name__,
                help=help_text,
                epilog=epilog,
                formatter_class=argparse.RawDescriptionHelpFormatter
            )
            func_map[f.__name__] = f
            sub.add_argument("-D", "--define", nargs="*", default=[], action="extend", help="Override params, e.g., a.b=1")
            sub.add_argument(
                "-lps",
                "--list-params",
                action="store_true",
                help="List parameter names, defaults, and current values (after --define overrides), then exit.",
            )
            sub.add_argument(
                "-ep",
                "--explain-param",
                nargs="*",
                metavar="NAME",
                help="Explain the source of specific parameters (default, CLI arg, or --define override), then exit. If omitted, prints all.",
            )
            sig = inspect.signature(f)
            param_help = _parse_param_help(f.__doc__)
            for name, param in sig.parameters.items():
                if param.default is inspect._empty:
                    sub.add_argument(name, type=param.annotation if param.annotation is not inspect._empty else str, help=param_help.get(name))
                else:
                    arg_type = _arg_type_from_default(param.default)
                    help_text = param_help.get(name)
                    if help_text:
                        help_text = f"{help_text} (default: {param.default})"
                    else:
                        help_text = f"(default from auto_param: {param.default})"
                    sub.add_argument(
                        f"--{name}",
                        dest=name,
                        type=arg_type,
                        default=argparse.SUPPRESS,
                        help=help_text,
                    )
        args = parser.parse_args()
        args_dict = vars(args)
        cmd = args_dict.pop("command")
        defines = args_dict.pop("define", [])
        target = func_map[cmd]
        if _maybe_explain_and_exit(target, args_dict, defines):
            return None
        param_scope = _get_param_scope()
        with param_scope(*defines):
            # Freeze first so new threads spawned inside target inherit these overrides.
            param_scope.frozen()
            return target(**args_dict)

    if not hasattr(func, "_auto_param_namespace"):
        raise ValueError("launch() expects a function decorated with @auto_param")
    parser = _build_parser_for_func(func, caller_globals=caller_globals)
    args = parser.parse_args()
    args_dict = vars(args)
    defines = args_dict.pop("define", [])
    if _maybe_explain_and_exit(func, args_dict, defines):
        return None
    param_scope = _get_param_scope()
    with param_scope(*defines):
        param_scope.frozen()
        return func(**args_dict)


def run_cli(func: Optional[Callable] = None, *, _caller_module=None) -> None:
    """Alias for launch() with a less collision-prone name.
    
    Args:
        func: Optional function to launch. If None, discovers all @auto_param functions in caller module.
        _caller_module: Explicitly pass caller's module name or module object (for entry point support).
                       This is useful when called via entry points where frame inspection may fail.
                       Can be a string (module name) or a module object.
    
    Examples:
        # In __main__.py or entry point script:
        if __name__ == "__main__":
            import sys
            run_cli(_caller_module=sys.modules[__name__])
        
        # Or simply:
        if __name__ == "__main__":
            run_cli(_caller_module=__name__)
    """
    caller_frame = inspect.currentframe().f_back  # type: ignore
    if caller_frame is not None:
        caller_globals = caller_frame.f_globals
        caller_locals = caller_frame.f_locals
    else:
        caller_globals = {}
        caller_locals = {}
        # Try to use _caller_module if provided
        if _caller_module is not None:
            if isinstance(_caller_module, str):
                if _caller_module in sys.modules:
                    mod = sys.modules[_caller_module]
                    caller_globals = mod.__dict__
                    caller_locals = mod.__dict__
            elif hasattr(_caller_module, '__dict__'):
                caller_globals = _caller_module.__dict__
                caller_locals = _caller_module.__dict__
    
    return launch(func, _caller_globals=caller_globals, _caller_locals=caller_locals, _caller_module=_caller_module)
