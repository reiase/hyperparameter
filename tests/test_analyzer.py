"""
Hyperparameter Analyzer 测试
"""

import os
import tempfile
from pathlib import Path
from unittest import TestCase

from hyperparameter.analyzer import (
    HyperparameterAnalyzer,
    ParamInfo,
    FunctionInfo,
    ScopeUsage,
    AnalysisResult,
)


class TestHyperparameterAnalyzer(TestCase):
    """测试 HyperparameterAnalyzer"""

    def setUp(self):
        self.analyzer = HyperparameterAnalyzer(verbose=False)
        self.temp_dir = tempfile.mkdtemp()

    def tearDown(self):
        import shutil

        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def _write_temp_file(self, filename: str, content: str) -> Path:
        """写入临时文件"""
        path = Path(self.temp_dir) / filename
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w") as f:
            f.write(content)
        return path

    def test_analyze_param_function(self):
        """测试分析 @hp.param 函数"""
        code = '''
import hyperparameter as hp

@hp.param("train")
def train(lr=0.001, batch_size=32, epochs=10):
    """Training function."""
    pass
'''
        self._write_temp_file("module.py", code)
        result = self.analyzer.analyze_package(self.temp_dir)

        self.assertEqual(len(result.functions), 1)
        func = result.functions[0]
        self.assertEqual(func.name, "train")
        self.assertEqual(func.namespace, "train")
        self.assertEqual(len(func.params), 3)

        param_names = [p.name for p in func.params]
        self.assertIn("lr", param_names)
        self.assertIn("batch_size", param_names)
        self.assertIn("epochs", param_names)

    def test_analyze_param_class(self):
        """测试分析 @hp.param 类"""
        code = """

@hp.param("Model")
class Model:
    def __init__(self, hidden_size=256, dropout=0.1):
        self.hidden_size = hidden_size
        self.dropout = dropout
"""
        self._write_temp_file("model.py", code)
        result = self.analyzer.analyze_package(self.temp_dir)

        self.assertEqual(len(result.functions), 1)
        func = result.functions[0]
        self.assertEqual(func.name, "Model")
        self.assertEqual(func.namespace, "Model")
        self.assertEqual(len(func.params), 2)

    def test_analyze_scope_usage(self):
        """测试分析 scope 使用"""
        code = """

def func():
    lr = hp.scope.train.lr | 0.001
    batch_size = hp.scope.train.batch_size | 32
"""
        self._write_temp_file("usage.py", code)
        result = self.analyzer.analyze_package(self.temp_dir)

        self.assertGreater(len(result.scope_usages), 0)
        keys = set(u.key for u in result.scope_usages)
        self.assertIn("train.lr", keys)
        self.assertIn("train.batch_size", keys)

    def test_analyze_nested_namespace(self):
        """测试嵌套命名空间"""
        code = """

@hp.param("app.config.train")
def train(lr=0.001):
    pass
"""
        self._write_temp_file("nested.py", code)
        result = self.analyzer.analyze_package(self.temp_dir)

        self.assertEqual(len(result.functions), 1)
        self.assertEqual(result.functions[0].namespace, "app.config.train")

    def test_format_text(self):
        """测试文本格式输出"""
        result = AnalysisResult(
            package="test",
            functions=[
                FunctionInfo(
                    name="train",
                    namespace="train",
                    module="module",
                    file="/path/to/module.py",
                    line=10,
                    params=[
                        ParamInfo(name="lr", default=0.001),
                        ParamInfo(name="epochs", default=10),
                    ],
                )
            ],
        )

        report = self.analyzer.format_report(result, format="text")

        self.assertIn("test", report)
        self.assertIn("train", report)
        self.assertIn("lr", report)

    def test_format_markdown(self):
        """测试 Markdown 格式输出"""
        result = AnalysisResult(
            package="test",
            functions=[
                FunctionInfo(
                    name="train",
                    namespace="train",
                    module="module",
                    file="/path/to/module.py",
                    line=10,
                    params=[ParamInfo(name="lr", default=0.001)],
                )
            ],
        )

        report = self.analyzer.format_report(result, format="markdown")

        self.assertIn("# Hyperparameter Analysis", report)
        self.assertIn("| Namespace |", report)
        self.assertIn("`train`", report)

    def test_format_json(self):
        """测试 JSON 格式输出"""
        import json

        result = AnalysisResult(
            package="test",
            functions=[
                FunctionInfo(
                    name="train",
                    namespace="train",
                    module="module",
                    file="/path/to/module.py",
                    line=10,
                    params=[ParamInfo(name="lr", default=0.001)],
                )
            ],
        )

        report = self.analyzer.format_report(result, format="json")
        data = json.loads(report)

        self.assertEqual(data["package"], "test")
        self.assertEqual(len(data["functions"]), 1)
        self.assertEqual(data["functions"][0]["name"], "train")

    def test_analyze_multiple_files(self):
        """测试分析多个文件"""
        code1 = """

@hp.param("module1")
def func1(x=1):
    pass
"""
        code2 = """

@hp.param("module2")
def func2(y=2):
    pass
"""
        self._write_temp_file("pkg/module1.py", code1)
        self._write_temp_file("pkg/module2.py", code2)
        self._write_temp_file("pkg/__init__.py", "")

        result = self.analyzer.analyze_package(os.path.join(self.temp_dir, "pkg"))

        self.assertEqual(len(result.functions), 2)
        namespaces = {f.namespace for f in result.functions}
        self.assertEqual(namespaces, {"module1", "module2"})

    def test_param_default_values(self):
        """测试提取默认值"""
        code = """

@hp.param("test")
def test_func(
    int_param=42,
    float_param=3.14,
    str_param="hello",
    bool_param=True,
    none_param=None,
    list_param=[1, 2, 3],
    neg_param=-1,
):
    pass
"""
        self._write_temp_file("defaults.py", code)
        result = self.analyzer.analyze_package(self.temp_dir)

        self.assertEqual(len(result.functions), 1)
        params = {p.name: p.default for p in result.functions[0].params}

        self.assertEqual(params["int_param"], 42)
        self.assertAlmostEqual(params["float_param"], 3.14)
        self.assertEqual(params["str_param"], "hello")
        self.assertEqual(params["bool_param"], True)
        self.assertIsNone(params["none_param"])
        self.assertEqual(params["list_param"], [1, 2, 3])
        self.assertEqual(params["neg_param"], -1)


class TestAnalysisResult(TestCase):
    """测试 AnalysisResult 数据类"""

    def test_empty_result(self):
        """测试空结果"""
        result = AnalysisResult(package="empty")

        self.assertEqual(result.package, "empty")
        self.assertEqual(len(result.functions), 0)
        self.assertEqual(len(result.scope_usages), 0)
        self.assertEqual(len(result.dependencies), 0)


if __name__ == "__main__":
    import pytest

    pytest.main([__file__, "-v"])
