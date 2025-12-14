"""
Hyperparameter Analyzer - 分析 Python 包中的超参数使用情况

功能：
1. 扫描包中所有 @param 装饰的函数/类
2. 扫描 scope 的使用
3. 分析依赖包中的超参数
4. 生成超参数报告
"""

from __future__ import annotations

import ast
import importlib
import importlib.util
import inspect
import os
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple


@dataclass
class ParamInfo:
    """超参数信息"""

    name: str  # 参数名（如 train.lr）
    default: Any = None  # 默认值
    type_hint: Optional[str] = None  # 类型提示
    source_file: Optional[str] = None  # 来源文件
    source_line: Optional[int] = None  # 来源行号
    docstring: Optional[str] = None  # 参数说明
    namespace: Optional[str] = None  # 命名空间


@dataclass
class FunctionInfo:
    """@param 函数信息"""

    name: str  # 函数名
    namespace: str  # 命名空间
    module: str  # 模块名
    file: str  # 文件路径
    line: int  # 行号
    docstring: Optional[str] = None  # 文档字符串
    params: List[ParamInfo] = field(default_factory=list)  # 参数列表


@dataclass
class ScopeUsage:
    """scope 使用信息"""

    key: str  # 参数键
    file: str  # 文件路径
    line: int  # 行号
    context: str  # 上下文代码


@dataclass
class AnalysisResult:
    """分析结果"""

    package: str  # 包名
    functions: List[FunctionInfo] = field(default_factory=list)
    scope_usages: List[ScopeUsage] = field(default_factory=list)
    dependencies: Dict[str, "AnalysisResult"] = field(default_factory=dict)


class HyperparameterAnalyzer:
    """超参数分析器"""

    def __init__(self, verbose: bool = False):
        self.verbose = verbose
        self._visited_modules: Set[str] = set()
        self._visited_files: Set[str] = set()

    def analyze_package(
        self, package_name: str, include_deps: bool = False
    ) -> AnalysisResult:
        """分析一个 Python 包

        Args:
            package_name: 包名或模块路径
            include_deps: 是否包含依赖分析

        Returns:
            AnalysisResult: 分析结果
        """
        result = AnalysisResult(package=package_name)

        # 尝试导入包
        try:
            if os.path.exists(package_name):
                # 是文件路径
                self._analyze_path(Path(package_name), result)
            else:
                # 是包名
                spec = importlib.util.find_spec(package_name)
                if spec:
                    # 处理命名空间包（spec.origin 可能为 None）
                    if spec.submodule_search_locations:
                        # 命名空间包或普通包，扫描所有搜索路径
                        for loc in spec.submodule_search_locations:
                            self._analyze_path(Path(loc), result)
                    elif spec.origin:
                        # 单文件模块
                        package_path = Path(spec.origin).parent
                        self._analyze_path(package_path, result)

                    # 分析依赖
                    if include_deps:
                        self._analyze_dependencies(package_name, result)
        except Exception as e:
            if self.verbose:
                print(f"Warning: Failed to analyze {package_name}: {e}")

        return result

    def _analyze_path(self, path: Path, result: AnalysisResult) -> None:
        """分析目录或文件"""
        if path.is_file() and path.suffix == ".py":
            self._analyze_file(path, result)
        elif path.is_dir():
            for py_file in path.rglob("*.py"):
                if "__pycache__" not in str(py_file):
                    self._analyze_file(py_file, result)

    def _analyze_file(self, file_path: Path, result: AnalysisResult) -> None:
        """分析单个 Python 文件"""
        file_str = str(file_path.absolute())
        if file_str in self._visited_files:
            return
        self._visited_files.add(file_str)

        try:
            with open(file_path, "r", encoding="utf-8") as f:
                source = f.read()
            tree = ast.parse(source, filename=str(file_path))
        except (SyntaxError, UnicodeDecodeError) as e:
            if self.verbose:
                print(f"Warning: Failed to parse {file_path}: {e}")
            return

        # 分析 AST
        analyzer = _ASTAnalyzer(str(file_path), source)
        analyzer.visit(tree)

        result.functions.extend(analyzer.functions)
        result.scope_usages.extend(analyzer.scope_usages)

    def _analyze_dependencies(self, package_name: str, result: AnalysisResult) -> None:
        """分析包的依赖"""
        try:
            # 尝试获取包的依赖
            import importlib.metadata as metadata

            try:
                requires = metadata.requires(package_name)
                if requires:
                    for req in requires:
                        # 解析依赖名（去掉版本等）
                        dep_name = req.split()[0].split(";")[0].split("[")[0]
                        dep_name = dep_name.replace("-", "_")

                        # 检查是否使用了 hyperparameter
                        if self._uses_hyperparameter(dep_name):
                            dep_result = self.analyze_package(
                                dep_name, include_deps=False
                            )
                            if dep_result.functions or dep_result.scope_usages:
                                result.dependencies[dep_name] = dep_result
            except metadata.PackageNotFoundError:
                pass
        except Exception as e:
            if self.verbose:
                print(f"Warning: Failed to analyze dependencies: {e}")

    def _uses_hyperparameter(self, package_name: str) -> bool:
        """检查包是否使用了 hyperparameter"""
        try:
            spec = importlib.util.find_spec(package_name)
            if spec:
                # 检查命名空间包
                if spec.submodule_search_locations:
                    for loc in spec.submodule_search_locations:
                        loc_path = Path(loc)
                        if loc_path.exists():
                            # 检查目录中前几个 py 文件
                            for py_file in list(loc_path.rglob("*.py"))[:10]:
                                try:
                                    content = py_file.read_text(encoding="utf-8")
                                    if (
                                        "hyperparameter" in content
                                        or "scope" in content
                                    ):
                                        return True
                                except Exception:
                                    pass
                # 检查单文件模块
                elif spec.origin:
                    with open(spec.origin, "r", encoding="utf-8") as f:
                        content = f.read()
                    return "hyperparameter" in content or "scope" in content
        except Exception:
            pass
        return False

    def find_hp_packages(self) -> List[Dict[str, Any]]:
        """查找所有使用了 hyperparameter 的已安装包

        Returns:
            List of dicts with package info: {name, version, location, param_count}
        """
        import importlib.metadata as metadata

        hp_packages = []

        for dist in metadata.distributions():
            name = dist.metadata.get("Name", "")
            if not name or name == "hyperparameter":
                continue

            # 检查依赖
            requires = dist.requires or []
            uses_hp = any("hyperparameter" in (r or "").lower() for r in requires)

            if not uses_hp:
                # 快速检查包内容
                try:
                    pkg_name = name.replace("-", "_")
                    if self._uses_hyperparameter(pkg_name):
                        uses_hp = True
                except Exception:
                    pass

            if uses_hp:
                # 分析这个包
                try:
                    pkg_name = name.replace("-", "_")
                    result = self.analyze_package(pkg_name, include_deps=False)
                    param_count = sum(len(f.params) for f in result.functions)
                    param_count += len(set(u.key for u in result.scope_usages))

                    if param_count > 0 or result.functions:
                        hp_packages.append(
                            {
                                "name": name,
                                "version": dist.metadata.get("Version", "?"),
                                "location": (
                                    str(dist._path) if hasattr(dist, "_path") else "?"
                                ),
                                "param_count": param_count,
                                "function_count": len(result.functions),
                            }
                        )
                except Exception:
                    # 无法分析，但确实依赖 hyperparameter
                    hp_packages.append(
                        {
                            "name": name,
                            "version": dist.metadata.get("Version", "?"),
                            "location": "?",
                            "param_count": 0,
                            "function_count": 0,
                        }
                    )

        return sorted(hp_packages, key=lambda x: x["name"].lower())

    def format_report(self, result: AnalysisResult, format: str = "text") -> str:
        """格式化报告

        Args:
            result: 分析结果
            format: 输出格式 (text, json, markdown)

        Returns:
            str: 格式化后的报告
        """
        if format == "json":
            return self._format_json(result)
        elif format == "markdown":
            return self._format_markdown(result)
        else:
            return self._format_text(result)

    def _format_text(self, result: AnalysisResult, indent: int = 0) -> str:
        """文本格式报告"""
        lines = []
        prefix = "  " * indent

        lines.append(f"{prefix}{'=' * 60}")
        lines.append(f"{prefix}Package: {result.package}")
        lines.append(f"{prefix}{'=' * 60}")

        if result.functions:
            lines.append(f"\n{prefix}@param Functions ({len(result.functions)}):")
            lines.append(f"{prefix}{'-' * 40}")

            # 按命名空间分组
            by_namespace: Dict[str, List[FunctionInfo]] = {}
            for func in result.functions:
                by_namespace.setdefault(func.namespace, []).append(func)

            for ns in sorted(by_namespace.keys()):
                funcs = by_namespace[ns]
                lines.append(f"\n{prefix}  [{ns}]")
                for func in funcs:
                    rel_file = os.path.basename(func.file)
                    lines.append(f"{prefix}    {func.name} ({rel_file}:{func.line})")
                    for param in func.params:
                        default_str = (
                            f" = {param.default!r}" if param.default is not None else ""
                        )
                        lines.append(f"{prefix}      - {ns}.{param.name}{default_str}")

        if result.scope_usages:
            lines.append(f"\n{prefix}scope Usages ({len(result.scope_usages)}):")
            lines.append(f"{prefix}{'-' * 40}")

            # 按 key 分组
            by_key: Dict[str, List[ScopeUsage]] = {}
            for usage in result.scope_usages:
                by_key.setdefault(usage.key, []).append(usage)

            for key in sorted(by_key.keys()):
                usages = by_key[key]
                lines.append(f"\n{prefix}  {key}")
                for usage in usages[:3]:  # 只显示前3个
                    rel_file = os.path.basename(usage.file)
                    lines.append(f"{prefix}    {rel_file}:{usage.line}")
                if len(usages) > 3:
                    lines.append(f"{prefix}    ... and {len(usages) - 3} more")

        if result.dependencies:
            lines.append(f"\n{prefix}Dependencies with Hyperparameters:")
            lines.append(f"{prefix}{'-' * 40}")
            for dep_name, dep_result in result.dependencies.items():
                lines.append(f"\n{prefix}  {dep_name}:")
                dep_lines = self._format_text(dep_result, indent + 2)
                lines.append(dep_lines)

        # 汇总
        total_params = sum(len(f.params) for f in result.functions)
        unique_keys = set(u.key for u in result.scope_usages)

        lines.append(f"\n{prefix}Summary:")
        lines.append(f"{prefix}  - {len(result.functions)} @param functions")
        lines.append(f"{prefix}  - {total_params} hyperparameters")
        lines.append(f"{prefix}  - {len(unique_keys)} unique scope keys")

        return "\n".join(lines)

    def _format_markdown(self, result: AnalysisResult) -> str:
        """Markdown 格式报告"""
        lines = []

        lines.append(f"# Hyperparameter Analysis: {result.package}")
        lines.append("")

        if result.functions:
            lines.append("## @param Functions")
            lines.append("")
            lines.append("| Namespace | Function | File | Parameters |")
            lines.append("|-----------|----------|------|------------|")

            for func in result.functions:
                rel_file = os.path.basename(func.file)
                params = ", ".join(p.name for p in func.params)
                lines.append(
                    f"| `{func.namespace}` | `{func.name}` | {rel_file}:{func.line} | {params} |"
                )
            lines.append("")

        if result.scope_usages:
            lines.append("## scope Usage")
            lines.append("")

            by_key: Dict[str, List[ScopeUsage]] = {}
            for usage in result.scope_usages:
                by_key.setdefault(usage.key, []).append(usage)

            for key in sorted(by_key.keys()):
                usages = by_key[key]
                lines.append(f"### `{key}`")
                lines.append("")
                for usage in usages[:5]:
                    rel_file = os.path.basename(usage.file)
                    lines.append(f"- {rel_file}:{usage.line}")
                if len(usages) > 5:
                    lines.append(f"- ... and {len(usages) - 5} more")
                lines.append("")

        if result.dependencies:
            lines.append("## Dependencies")
            lines.append("")
            for dep_name in result.dependencies:
                lines.append(f"- `{dep_name}`")
            lines.append("")

        # Summary
        total_params = sum(len(f.params) for f in result.functions)
        unique_keys = set(u.key for u in result.scope_usages)

        lines.append("## Summary")
        lines.append("")
        lines.append(f"- **@param functions**: {len(result.functions)}")
        lines.append(f"- **Hyperparameters**: {total_params}")
        lines.append(f"- **Unique scope keys**: {len(unique_keys)}")

        return "\n".join(lines)

    def _format_json(self, result: AnalysisResult) -> str:
        """JSON 格式报告"""
        import json

        def to_dict(obj):
            if isinstance(obj, AnalysisResult):
                return {
                    "package": obj.package,
                    "functions": [to_dict(f) for f in obj.functions],
                    "scope_usages": [to_dict(u) for u in obj.scope_usages],
                    "dependencies": {
                        k: to_dict(v) for k, v in obj.dependencies.items()
                    },
                }
            elif isinstance(obj, FunctionInfo):
                return {
                    "name": obj.name,
                    "namespace": obj.namespace,
                    "module": obj.module,
                    "file": obj.file,
                    "line": obj.line,
                    "docstring": obj.docstring,
                    "params": [to_dict(p) for p in obj.params],
                }
            elif isinstance(obj, ParamInfo):
                return {
                    "name": obj.name,
                    "default": repr(obj.default) if obj.default is not None else None,
                    "type_hint": obj.type_hint,
                }
            elif isinstance(obj, ScopeUsage):
                return {
                    "key": obj.key,
                    "file": obj.file,
                    "line": obj.line,
                }
            return obj

        return json.dumps(to_dict(result), indent=2, ensure_ascii=False)


class _ASTAnalyzer(ast.NodeVisitor):
    """AST 分析器"""

    def __init__(self, file_path: str, source: str):
        self.file_path = file_path
        self.source = source
        self.source_lines = source.splitlines()
        self.functions: List[FunctionInfo] = []
        self.scope_usages: List[ScopeUsage] = []
        self._current_class: Optional[str] = None

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        """访问类定义"""
        # 检查是否有 @param 装饰器
        namespace = self._get_param_namespace(node.decorator_list)
        if namespace:
            self._add_function_info(node, namespace, is_class=True)

        old_class = self._current_class
        self._current_class = node.name
        self.generic_visit(node)
        self._current_class = old_class

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        """访问函数定义"""
        self._visit_function(node)

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        """访问异步函数定义"""
        self._visit_function(node)

    def _visit_function(self, node) -> None:
        """分析函数定义"""
        # 检查是否有 @param 装饰器
        namespace = self._get_param_namespace(node.decorator_list)
        if namespace:
            self._add_function_info(node, namespace)

        # 分析函数体中的 scope 使用
        self._analyze_scope_usages(node)

        self.generic_visit(node)

    def _get_param_namespace(self, decorators: List[ast.expr]) -> Optional[str]:
        """获取 @param 或 @auto_param 的命名空间（兼容新旧用法）
        
        支持：
        - @param 或 @param("ns")
        - @auto_param 或 @auto_param("ns")
        - @hp.param 或 @hp.param("ns")
        """
        param_names = ("param", "auto_param")  # 支持新旧两种名称
        for dec in decorators:
            # @param (无括号)
            if isinstance(dec, ast.Name) and dec.id in param_names:
                return None  # 无参数，使用函数名
            # @hp.param (无括号，属性访问形式)
            elif isinstance(dec, ast.Attribute) and dec.attr in param_names:
                return None  # 无参数，使用函数名
            elif isinstance(dec, ast.Call):
                func = dec.func
                # @param("ns")
                if isinstance(func, ast.Name) and func.id in param_names:
                    if dec.args and isinstance(dec.args[0], ast.Constant):
                        return dec.args[0].value
                    return None  # 无参数
                # @hp.param("ns")
                elif isinstance(func, ast.Attribute) and func.attr in param_names:
                    if dec.args and isinstance(dec.args[0], ast.Constant):
                        return dec.args[0].value
                    return None
        return None  # 没有 @param

    def _add_function_info(
        self, node, namespace: Optional[str], is_class: bool = False
    ) -> None:
        """添加函数/类信息"""
        name = node.name
        if namespace is None:
            namespace = name

        # 获取参数信息
        params = []
        if is_class:
            # 类：从 __init__ 获取参数
            for item in node.body:
                if isinstance(item, ast.FunctionDef) and item.name == "__init__":
                    params = self._extract_params(item.args, namespace)
                    break
        else:
            params = self._extract_params(node.args, namespace)

        # 获取文档字符串
        docstring = ast.get_docstring(node)

        # 确定模块名
        module = os.path.splitext(os.path.basename(self.file_path))[0]

        func_info = FunctionInfo(
            name=name,
            namespace=namespace,
            module=module,
            file=self.file_path,
            line=node.lineno,
            docstring=docstring,
            params=params,
        )
        self.functions.append(func_info)

    def _extract_params(self, args: ast.arguments, namespace: str) -> List[ParamInfo]:
        """提取函数参数"""
        params = []

        # 处理默认值
        defaults = args.defaults
        num_defaults = len(defaults)
        num_args = len(args.args)

        for i, arg in enumerate(args.args):
            if arg.arg in ("self", "cls"):
                continue

            # 检查是否有默认值
            default_idx = i - (num_args - num_defaults)
            default = None
            if default_idx >= 0 and default_idx < len(defaults):
                default = self._get_constant_value(defaults[default_idx])

            # 类型提示
            type_hint = None
            if arg.annotation:
                type_hint = (
                    ast.unparse(arg.annotation) if hasattr(ast, "unparse") else None
                )

            param = ParamInfo(
                name=arg.arg,
                default=default,
                type_hint=type_hint,
                source_file=self.file_path,
                source_line=arg.lineno if hasattr(arg, "lineno") else None,
                namespace=namespace,
            )
            params.append(param)

        # 处理 kwonly 参数
        for i, arg in enumerate(args.kwonlyargs):
            default = None
            if i < len(args.kw_defaults) and args.kw_defaults[i]:
                default = self._get_constant_value(args.kw_defaults[i])

            type_hint = None
            if arg.annotation:
                type_hint = (
                    ast.unparse(arg.annotation) if hasattr(ast, "unparse") else None
                )

            param = ParamInfo(
                name=arg.arg,
                default=default,
                type_hint=type_hint,
                source_file=self.file_path,
                source_line=arg.lineno if hasattr(arg, "lineno") else None,
                namespace=namespace,
            )
            params.append(param)

        return params

    def _get_constant_value(self, node: ast.expr) -> Any:
        """获取常量值"""
        if isinstance(node, ast.Constant):
            return node.value
        elif isinstance(node, ast.Num):  # Python 3.7 兼容
            return node.n
        elif isinstance(node, ast.Str):  # Python 3.7 兼容
            return node.s
        elif isinstance(node, ast.NameConstant):  # Python 3.7 兼容
            return node.value
        elif isinstance(node, ast.List):
            return [self._get_constant_value(e) for e in node.elts]
        elif isinstance(node, ast.Dict):
            return {
                self._get_constant_value(k): self._get_constant_value(v)
                for k, v in zip(node.keys, node.values)
                if k is not None
            }
        elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
            val = self._get_constant_value(node.operand)
            return -val if val is not None else None
        return None

    def _analyze_scope_usages(self, node) -> None:
        """分析 scope 使用"""
        for child in ast.walk(node):
            # 查找 scope.xxx 或 scope.xxx.yyy
            if isinstance(child, ast.Attribute):
                key = self._extract_scope_key(child)
                if key:
                    context = self._get_source_line(child.lineno)
                    usage = ScopeUsage(
                        key=key,
                        file=self.file_path,
                        line=child.lineno,
                        context=context,
                    )
                    self.scope_usages.append(usage)

    def _extract_scope_key(self, node: ast.Attribute) -> Optional[str]:
        """提取 scope 或 param_scope 的键（兼容新旧两种用法）
        
        支持：
        - scope.train.lr (旧用法)
        - param_scope.train.lr (旧用法)
        - hp.scope.train.lr (新用法，hp 是任意别名)
        """
        scope_names = ("scope", "param_scope")  # 支持新旧两种名称
        parts = []
        current = node

        while isinstance(current, ast.Attribute):
            parts.append(current.attr)
            current = current.value

        # 方式 1: scope.xxx 或 param_scope.xxx
        if isinstance(current, ast.Name) and current.id in scope_names:
            parts.reverse()
            return ".".join(parts)
        
        # 方式 2: hp.scope.xxx (hp 可以是任意名称)
        if isinstance(current, ast.Name) and parts and parts[-1] in scope_names:
            parts.pop()  # 移除 "scope"
            parts.reverse()
            if parts:  # 确保还有内容
                return ".".join(parts)

        return None

    def _get_source_line(self, lineno: int) -> str:
        """获取源代码行"""
        if 0 < lineno <= len(self.source_lines):
            return self.source_lines[lineno - 1].strip()
        return ""


def _collect_params(
    result: AnalysisResult, include_deps: bool = False
) -> Dict[str, Dict[str, Any]]:
    """收集所有参数信息

    Returns:
        Dict[key, {default, type_hint, file, line, docstring, source}]
    """
    all_params: Dict[str, Dict[str, Any]] = {}

    def add_from_result(res: AnalysisResult, source: str):
        for func in res.functions:
            for param in func.params:
                full_key = f"{func.namespace}.{param.name}"
                if full_key not in all_params:
                    all_params[full_key] = {
                        "default": param.default,
                        "type_hint": param.type_hint,
                        "file": func.file,
                        "line": func.line,
                        "docstring": func.docstring,
                        "source": source,
                        "function": func.name,
                        "namespace": func.namespace,
                    }

        for usage in res.scope_usages:
            if usage.key not in all_params:
                all_params[usage.key] = {
                    "default": None,
                    "type_hint": None,
                    "file": usage.file,
                    "line": usage.line,
                    "docstring": None,
                    "source": source,
                    "context": usage.context,
                }

    add_from_result(result, result.package)

    if include_deps:
        for dep_name, dep_result in result.dependencies.items():
            add_from_result(dep_result, dep_name)

    return all_params


def _print_params_list(params: Dict[str, Dict[str, Any]], tree: bool = False):
    """打印参数列表"""
    if not params:
        print("  (no hyperparameters found)")
        return

    if tree:
        # 树状显示
        tree_dict: Dict[str, Any] = {}
        for key in sorted(params.keys()):
            parts = key.split(".")
            current = tree_dict
            for i, part in enumerate(parts):
                if i == len(parts) - 1:
                    current[part] = {"_info": params[key]}
                else:
                    if part not in current or not isinstance(current.get(part), dict):
                        current[part] = {}
                    current = current[part]

        def print_tree(node: Dict, indent: int = 0):
            for key, value in sorted(node.items()):
                if key == "_info":
                    continue
                if isinstance(value, dict) and "_info" not in value:
                    print("  " * indent + f"📁 {key}")
                    print_tree(value, indent + 1)
                else:
                    info = value.get("_info", {}) if isinstance(value, dict) else {}
                    default = info.get("default")
                    default_str = f" = {default!r}" if default is not None else ""
                    print("  " * indent + f"📄 {key}{default_str}")

        print_tree(tree_dict)
    else:
        # 列表显示
        for key in sorted(params.keys()):
            info = params[key]
            default = info.get("default")
            default_str = f" = {default!r}" if default is not None else ""
            print(f"  {key}{default_str}")


def _describe_param(params: Dict[str, Dict[str, Any]], name: str):
    """描述单个参数"""
    # 精确匹配
    if name in params:
        info = params[name]
        _print_param_detail(name, info)
        return

    # 模糊匹配
    matches = [k for k in params.keys() if name in k]

    if not matches:
        print(f"Hyperparameter '{name}' not found.")
        print("\nAvailable hyperparameters:")
        for key in sorted(params.keys())[:10]:
            print(f"  {key}")
        if len(params) > 10:
            print(f"  ... and {len(params) - 10} more")
        return

    if len(matches) == 1:
        key = matches[0]
        _print_param_detail(key, params[key])
    else:
        print(f"Multiple matches for '{name}':")
        for key in sorted(matches):
            info = params[key]
            default = info.get("default")
            default_str = f" = {default!r}" if default is not None else ""
            print(f"  {key}{default_str}")


def _print_param_detail(name: str, info: Dict[str, Any]):
    """打印参数详情"""
    print(f"\n{'=' * 60}")
    print(f"Hyperparameter: {name}")
    print(f"{'=' * 60}")

    if info.get("default") is not None:
        print(f"\n  Default: {info['default']!r}")

    if info.get("type_hint"):
        print(f"  Type: {info['type_hint']}")

    if info.get("namespace"):
        print(f"  Namespace: {info['namespace']}")

    if info.get("function"):
        print(f"  Function: {info['function']}")

    print(f"\n  Source: {info.get('source', 'unknown')}")

    if info.get("file"):
        rel_file = os.path.basename(info["file"])
        print(f"  Location: {rel_file}:{info.get('line', '?')}")

    if info.get("context"):
        print(f"\n  Context: {info['context']}")

    if info.get("docstring"):
        doc = info["docstring"]
        # 只显示第一段
        first_para = doc.split("\n\n")[0].replace("\n", " ").strip()
        if len(first_para) > 100:
            first_para = first_para[:100] + "..."
        print(f"\n  Description: {first_para}")

    # 使用示例
    print(f"\n  Usage:")
    print(f"    # 通过 scope 访问")
    print(f"    value = scope.{name} | <default>")
    print(f"    ")
    print(f"    # 通过命令行设置")
    parts = name.split(".")
    if len(parts) >= 2:
        print(f"    --{parts[0]}.{'.'.join(parts[1:])}=<value>")
    else:
        print(f"    --{name}=<value>")


def _list_hp_packages(analyzer: HyperparameterAnalyzer, format: str = "text"):
    """列出所有使用 hyperparameter 的包"""
    print("\nScanning installed packages...")
    packages = analyzer.find_hp_packages()

    if not packages:
        print("\nNo packages using hyperparameter found.")
        print("Try: hp ls <package_name> to analyze a specific package.")
        return

    if format == "json":
        import json

        print(json.dumps(packages, indent=2, ensure_ascii=False))
        return

    print(f"\nPackages using hyperparameter ({len(packages)}):")
    print("=" * 60)
    print(f"{'Package':<30} {'Version':<12} {'Params':<8} {'Funcs':<8}")
    print("-" * 60)

    for pkg in packages:
        name = pkg["name"][:29]
        version = pkg["version"][:11]
        params = pkg["param_count"]
        funcs = pkg["function_count"]
        print(f"{name:<30} {version:<12} {params:<8} {funcs:<8}")

    print("-" * 60)
    print(f"\nUse 'hp ls <package>' to see hyperparameters in a package.")


def main():
    """命令行入口"""
    import argparse

    parser = argparse.ArgumentParser(
        prog="hp",
        description="Hyperparameter Analyzer - 分析 Python 包中的超参数使用",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  hp ls                          列出使用 hyperparameter 的包
  hp ls mypackage                列出包中的超参数
  hp ls mypackage --tree         树状显示
  hp ls mypackage --all          包含依赖包的超参数
  hp desc train.lr               查看 train.lr 的详细信息
  hp desc lr                     模糊搜索包含 'lr' 的超参数
""",
    )

    subparsers = parser.add_subparsers(dest="command", help="可用命令")

    # list/ls 命令
    list_parser = subparsers.add_parser("list", aliases=["ls"], help="列出超参数")
    list_parser.add_argument(
        "package", nargs="?", default=None, help="包名或路径（不指定则列出所有 hp 包）"
    )

    scope_group = list_parser.add_mutually_exclusive_group()
    scope_group.add_argument(
        "--all", "-a", action="store_true", help="包含依赖包的超参数"
    )
    scope_group.add_argument(
        "--deps", "-d", action="store_true", help="只显示依赖包的超参数"
    )
    scope_group.add_argument(
        "--self",
        "-s",
        action="store_true",
        default=True,
        help="只显示自身的超参数（默认）",
    )

    list_parser.add_argument("--tree", "-t", action="store_true", help="树状显示")
    list_parser.add_argument(
        "--format",
        "-f",
        choices=["text", "json", "markdown"],
        default="text",
        help="输出格式",
    )
    list_parser.add_argument("--output", "-o", help="输出文件")
    list_parser.add_argument("--verbose", "-v", action="store_true", help="详细输出")

    # describe/desc 命令
    desc_parser = subparsers.add_parser(
        "describe", aliases=["desc"], help="查看超参数详情"
    )
    desc_parser.add_argument("name", help="超参数名称（支持模糊匹配）")
    desc_parser.add_argument(
        "package", nargs="?", default=".", help="包名或路径（默认当前目录）"
    )
    desc_parser.add_argument("--all", "-a", action="store_true", help="包含依赖包")

    args = parser.parse_args()

    if args.command in ("list", "ls"):
        analyzer = HyperparameterAnalyzer(verbose=getattr(args, "verbose", False))

        # 如果没有指定包，列出所有使用 hp 的包
        if args.package is None:
            _list_hp_packages(analyzer, format=args.format)
            return

        # 分析指定包
        include_deps = args.all or args.deps
        result = analyzer.analyze_package(args.package, include_deps=include_deps)

        # 收集参数
        all_params = _collect_params(result, include_deps=args.all)

        # 如果只要依赖，过滤掉自身的
        if args.deps:
            all_params = {
                k: v for k, v in all_params.items() if v.get("source") != result.package
            }

        # 输出
        if args.format == "json":
            import json

            print(json.dumps(all_params, indent=2, ensure_ascii=False, default=repr))
        elif args.format == "markdown":
            report = analyzer.format_report(result, format="markdown")
            if args.output:
                with open(args.output, "w", encoding="utf-8") as f:
                    f.write(report)
                print(f"Report saved to {args.output}")
            else:
                print(report)
        else:
            print(f"\nHyperparameters in {args.package}:")
            print("-" * 40)
            _print_params_list(all_params, tree=args.tree)
            print(f"\nTotal: {len(all_params)} hyperparameters")

    elif args.command in ("describe", "desc"):
        analyzer = HyperparameterAnalyzer()
        result = analyzer.analyze_package(args.package, include_deps=args.all)
        all_params = _collect_params(result, include_deps=args.all)

        _describe_param(all_params, args.name)

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
