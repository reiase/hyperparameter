"""
边界条件测试

测试 hyperparameter 在各种边界情况下的行为，包括：
1. 特殊 key 名称（长度、字符、Unicode）
2. 特殊值（None、空容器、极端数值）
3. 深度嵌套
4. 大量参数
5. 异常恢复
6. 并发边界
"""

import sys
import threading
from unittest import TestCase

import pytest

from hyperparameter import auto_param, param_scope
from hyperparameter.storage import has_rust_backend


class TestSpecialKeys(TestCase):
    """特殊 key 名称测试"""

    def test_single_char_key(self):
        """单字符 key"""
        with param_scope(a=1, b=2, c=3) as ps:
            self.assertEqual(ps.a(), 1)
            self.assertEqual(ps.b(), 2)
            self.assertEqual(ps.c(), 3)

    def test_long_key(self):
        """长 key 名称（100字符）"""
        long_key = "a" * 100
        with param_scope(**{long_key: 42}) as ps:
            self.assertEqual(ps[long_key] | 0, 42)

    def test_very_long_key(self):
        """非常长的 key 名称（1000字符）"""
        very_long_key = "a" * 1000
        with param_scope(**{very_long_key: 42}) as ps:
            # 使用整数默认值避免 | 运算符的问题
            self.assertEqual(ps[very_long_key] | 0, 42)

    def test_deeply_nested_key(self):
        """深度嵌套的 key（10层）"""
        deep_key = ".".join(["level"] * 10)
        with param_scope(**{deep_key: 100}) as ps:
            self.assertEqual(ps[deep_key] | 0, 100)

    def test_very_deeply_nested_key(self):
        """非常深的嵌套（50层）"""
        deep_key = ".".join(["l"] * 50)
        with param_scope(**{deep_key: 42}) as ps:
            # 使用整数默认值避免 | 运算符的问题
            self.assertEqual(ps[deep_key] | 0, 42)

    def test_numeric_key_segment(self):
        """数字开头的 key 段"""
        with param_scope(**{"a.123.b": 1, "456": 2}) as ps:
            self.assertEqual(ps["a.123.b"] | 0, 1)
            self.assertEqual(ps["456"] | 0, 2)

    def test_underscore_key(self):
        """下划线 key"""
        with param_scope(**{"_private": 1, "a_b_c": 3}) as ps:
            self.assertEqual(ps["_private"] | 0, 1)
            self.assertEqual(ps["a_b_c"] | 0, 3)

    def test_dash_key(self):
        """带连字符的 key"""
        with param_scope(**{"some-key": 1, "a-b-c": 2}) as ps:
            self.assertEqual(ps["some-key"] | 0, 1)
            self.assertEqual(ps["a-b-c"] | 0, 2)

    def test_case_sensitivity(self):
        """大小写敏感"""
        with param_scope(**{"Key": 1, "key": 2, "KEY": 3}) as ps:
            self.assertEqual(ps["Key"] | 0, 1)
            self.assertEqual(ps["key"] | 0, 2)
            self.assertEqual(ps["KEY"] | 0, 3)

    def test_unicode_key(self):
        """Unicode key"""
        with param_scope(**{"中文": 1, "日本語": 2, "한국어": 3}) as ps:
            self.assertEqual(ps["中文"] | 0, 1)
            self.assertEqual(ps["日本語"] | 0, 2)
            self.assertEqual(ps["한국어"] | 0, 3)

    def test_emoji_key(self):
        """Emoji key"""
        with param_scope(**{"🚀": 1, "test🎉": 2}) as ps:
            self.assertEqual(ps["🚀"] | 0, 1)
            self.assertEqual(ps["test🎉"] | 0, 2)

    def test_mixed_unicode_ascii_key(self):
        """混合 Unicode 和 ASCII 的 key"""
        with param_scope(**{"config.中文.value": 42}) as ps:
            self.assertEqual(ps["config.中文.value"] | 0, 42)


class TestSpecialValues(TestCase):
    """特殊值测试"""

    def test_none_value(self):
        """None 值"""
        with param_scope(**{"key": None}) as ps:
            result = ps.key | "default"
            # None 被存储，但在使用 | 时可能触发默认值
            self.assertIn(result, [None, "default"])

    def test_zero_values(self):
        """零值（不应该被当作缺失）"""
        with param_scope(**{"int_zero": 0, "float_zero": 0.0}) as ps:
            self.assertEqual(ps.int_zero | 999, 0)
            self.assertEqual(ps.float_zero | 999.0, 0.0)

    def test_false_value(self):
        """False 值（不应该被当作缺失）"""
        with param_scope(**{"flag": False}) as ps:
            self.assertFalse(ps.flag | True)

    def test_empty_string_via_call(self):
        """空字符串（通过调用访问）"""
        with param_scope(**{"empty_str": ""}) as ps:
            # 使用 () 调用语法避免 | 运算符问题
            self.assertEqual(ps.empty_str("default"), "")

    def test_empty_list(self):
        """空列表"""
        with param_scope(**{"empty_list": []}) as ps:
            result = ps.empty_list([1, 2, 3])
            self.assertEqual(result, [])

    def test_list_value(self):
        """列表值"""
        with param_scope(**{"my_list": [1, 2, 3]}) as ps:
            result = ps.my_list([])
            self.assertEqual(result, [1, 2, 3])

    def test_dict_value(self):
        """字典值 - 注意：嵌套字典会被展平为 key.subkey 格式"""
        # 字典作为值时会被展平
        with param_scope(**{"my_dict": {"a": 1}}) as ps:
            # 嵌套字典被展平为 my_dict.a
            result = ps["my_dict.a"] | 0
            self.assertEqual(result, 1)

    def test_negative_integer(self):
        """负整数"""
        with param_scope(**{"neg": -42}) as ps:
            self.assertEqual(ps.neg | 0, -42)

    def test_float_precision(self):
        """浮点数精度"""
        with param_scope(**{"pi": 3.141592653589793}) as ps:
            self.assertAlmostEqual(ps.pi | 0.0, 3.141592653589793)

    def test_special_floats(self):
        """特殊浮点数"""
        with param_scope(**{"inf": float("inf"), "neg_inf": float("-inf")}) as ps:
            self.assertEqual(ps.inf | 0.0, float("inf"))
            self.assertEqual(ps.neg_inf | 0.0, float("-inf"))

    def test_nan_float(self):
        """NaN 值"""
        import math

        with param_scope(**{"nan": float("nan")}) as ps:
            result = ps.nan | 0.0
            self.assertTrue(math.isnan(result))

    def test_boolean_strings(self):
        """布尔字符串转换"""
        with param_scope(
            **{
                "true_str": "true",
                "false_str": "false",
                "yes": "yes",
                "no": "no",
                "one": "1",
                "zero": "0",
            }
        ) as ps:
            self.assertTrue(ps.true_str(False))
            self.assertFalse(ps.false_str(True))
            self.assertTrue(ps.yes(False))
            self.assertFalse(ps.no(True))
            self.assertTrue(ps.one(False))
            self.assertFalse(ps.zero(True))


class TestScopeNesting(TestCase):
    """作用域嵌套边界测试"""

    def test_moderate_nesting(self):
        """中等深度嵌套作用域（10层）"""
        depth = 10

        def nested(level):
            if level == 0:
                return param_scope.base | -1
            with param_scope(**{f"level{level}": level}):
                return nested(level - 1)

        with param_scope(**{"base": 42}):
            result = nested(depth)
            self.assertEqual(result, 42)

    def test_sibling_scopes(self):
        """兄弟作用域隔离"""
        results = []
        with param_scope(**{"base": 0}):
            for i in range(10):
                with param_scope(**{"val": i}):
                    results.append(param_scope.val())
        self.assertEqual(results, list(range(10)))

    def test_scope_override_and_restore(self):
        """作用域覆盖和恢复"""
        with param_scope(**{"key": 1}):
            self.assertEqual(param_scope.key(), 1)
            with param_scope(**{"key": 2}):
                self.assertEqual(param_scope.key(), 2)
                with param_scope(**{"key": 3}):
                    self.assertEqual(param_scope.key(), 3)
                self.assertEqual(param_scope.key(), 2)
            self.assertEqual(param_scope.key(), 1)


class TestManyParameters(TestCase):
    """大量参数测试"""

    def test_many_parameters(self):
        """大量参数（1000个）"""
        num_params = 1000
        params = {f"param_{i}": i for i in range(num_params)}
        with param_scope(**params) as ps:
            # 验证部分参数，使用属性访问
            self.assertEqual(ps.param_0 | -1, 0)
            self.assertEqual(ps.param_100 | -1, 100)
            self.assertEqual(ps.param_500 | -1, 500)
            self.assertEqual(ps.param_999 | -1, 999)

    def test_many_nested_keys(self):
        """大量嵌套 key（100个）"""
        num_params = 100
        params = {f"a.b.c.d.param_{i}": i for i in range(num_params)}
        with param_scope(**params) as ps:
            # 验证部分参数，使用属性访问
            self.assertEqual(ps.a.b.c.d.param_0 | -1, 0)
            self.assertEqual(ps.a.b.c.d.param_50 | -1, 50)
            self.assertEqual(ps.a.b.c.d.param_99 | -1, 99)


class TestExceptionRecovery(TestCase):
    """异常恢复测试"""

    def test_exception_in_scope(self):
        """作用域内异常后正确恢复"""
        with param_scope(**{"val": 1}):
            try:
                with param_scope(**{"val": 2}):
                    self.assertEqual(param_scope.val(), 2)
                    raise ValueError("test error")
            except ValueError:
                pass
            # 应该恢复到外层值
            self.assertEqual(param_scope.val(), 1)

    def test_nested_exceptions(self):
        """嵌套异常恢复"""
        with param_scope(**{"a": 1, "b": 2}):
            try:
                with param_scope(**{"a": 10}):
                    try:
                        with param_scope(**{"b": 20}):
                            raise RuntimeError("inner")
                    except RuntimeError:
                        pass
                    self.assertEqual(param_scope.b(), 2)
                    raise ValueError("outer")
            except ValueError:
                pass
            self.assertEqual(param_scope.a(), 1)
            self.assertEqual(param_scope.b(), 2)

    def test_generator_exception(self):
        """生成器中的异常恢复"""

        def gen():
            with param_scope(**{"gen_val": 42}):
                yield param_scope.gen_val()
                raise StopIteration

        g = gen()
        self.assertEqual(next(g), 42)


class TestTypeConversionEdgeCases(TestCase):
    """类型转换边界测试"""

    def test_string_to_int_conversion(self):
        """字符串到整数转换"""
        with param_scope(**{"str_int": "42"}) as ps:
            self.assertEqual(ps.str_int | 0, 42)

    def test_string_to_float_conversion(self):
        """字符串到浮点数转换"""
        with param_scope(**{"str_float": "3.14"}) as ps:
            self.assertAlmostEqual(ps.str_float | 0.0, 3.14)

    def test_invalid_string_to_int(self):
        """无效字符串到整数转换"""
        with param_scope(**{"invalid": "not_a_number"}) as ps:
            result = ps.invalid | 0
            # 无法转换时返回原始字符串或默认值
            self.assertIn(result, ["not_a_number", 0])

    def test_scientific_notation(self):
        """科学记数法"""
        with param_scope(**{"sci": "1e-5"}) as ps:
            result = ps.sci | 0.0
            self.assertAlmostEqual(result, 1e-5)

    def test_string_bool_edge_cases(self):
        """字符串布尔转换边界情况"""
        test_cases = [
            ("True", True),
            ("TRUE", True),
            ("true", True),
            ("t", True),
            ("T", True),
            ("1", True),
            ("yes", True),
            ("YES", True),
            ("y", True),
            ("Y", True),
            ("on", True),
            ("ON", True),
            ("False", False),
            ("FALSE", False),
            ("false", False),
            ("f", False),
            ("F", False),
            ("0", False),
            ("no", False),
            ("NO", False),
            ("n", False),
            ("N", False),
            ("off", False),
            ("OFF", False),
        ]
        for str_val, expected in test_cases:
            with param_scope(**{"flag": str_val}) as ps:
                result = ps.flag(not expected)  # 使用相反值作为默认
                self.assertEqual(
                    result,
                    expected,
                    f"Failed for '{str_val}': expected {expected}, got {result}",
                )


class TestAutoParamEdgeCases(TestCase):
    """@auto_param 边界测试"""

    def test_no_default_args(self):
        """无默认参数的函数"""

        @auto_param("func")
        def func(a, b, c):
            return a, b, c

        result = func(1, 2, 3)
        self.assertEqual(result, (1, 2, 3))

    def test_all_default_args(self):
        """全部默认参数的函数"""

        @auto_param("func")
        def func(a=1, b=2, c=3):
            return a, b, c

        result = func()
        self.assertEqual(result, (1, 2, 3))

    def test_mixed_args(self):
        """混合参数"""

        @auto_param("func")
        def func(a, b=2, *args, c=3, **kwargs):
            return a, b, args, c, kwargs

        result = func(1)
        self.assertEqual(result, (1, 2, (), 3, {}))

    def test_override_with_zero(self):
        """用 0 覆盖默认值"""

        @auto_param("func")
        def func(a=1):
            return a

        with param_scope(**{"func.a": 0}):
            result = func()
            # 0 应该覆盖默认值
            self.assertEqual(result, 0)

    def test_class_method(self):
        """类方法"""

        @auto_param("MyClass")
        class MyClass:
            def __init__(self, x=1, y=2):
                self.x = x
                self.y = y

        obj = MyClass()
        self.assertEqual(obj.x, 1)
        self.assertEqual(obj.y, 2)

        with param_scope(**{"MyClass.x": 10}):
            obj2 = MyClass()
            self.assertEqual(obj2.x, 10)
            self.assertEqual(obj2.y, 2)


class TestConcurrencyEdgeCases(TestCase):
    """并发边界测试"""

    def test_rapid_scope_creation(self):
        """快速创建大量作用域"""
        for _ in range(1000):
            with param_scope(**{"key": "value"}):
                _ = param_scope.key()

    def test_thread_local_isolation(self):
        """线程本地隔离"""
        results = {}
        errors = []

        def worker(thread_id):
            try:
                with param_scope(**{"tid": thread_id}):
                    for _ in range(100):
                        val = param_scope.tid()
                        if val != thread_id:
                            errors.append(f"Thread {thread_id} saw {val}")
                    results[thread_id] = True
            except Exception as e:
                errors.append(str(e))
                results[thread_id] = False

        threads = [threading.Thread(target=worker, args=(i,)) for i in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0, f"Errors: {errors}")
        self.assertTrue(all(results.values()))


class TestKeyError(TestCase):
    """KeyError 行为测试"""

    def test_missing_key_raises(self):
        """缺失 key 调用无参数时抛出 KeyError"""
        with param_scope():
            with self.assertRaises(KeyError):
                param_scope.nonexistent()

    def test_missing_nested_key_raises(self):
        """缺失嵌套 key 调用无参数时抛出 KeyError"""
        with param_scope():
            with self.assertRaises(KeyError):
                param_scope.a.b.c.d()

    def test_missing_key_with_default(self):
        """缺失 key 带默认值不抛出异常"""
        with param_scope():
            result = param_scope.nonexistent | "default"
            self.assertEqual(result, "default")

    def test_missing_key_with_call_default(self):
        """缺失 key 调用带参数不抛出异常"""
        with param_scope():
            result = param_scope.nonexistent("default")
            self.assertEqual(result, "default")


class TestStorageOperations(TestCase):
    """存储操作测试"""

    def test_clear_storage(self):
        """清空存储"""
        ps = param_scope(a=1, b=2)
        ps.clear()
        self.assertEqual(ps.a | "empty", "empty")
        self.assertEqual(ps.b | "empty", "empty")

    def test_keys_iteration(self):
        """遍历所有 key"""
        with param_scope(**{"a": 1, "b.c": 2, "d.e.f": 3}) as ps:
            keys = list(ps.keys())
            self.assertIn("a", keys)
            self.assertIn("b.c", keys)
            self.assertIn("d.e.f", keys)

    def test_dict_conversion(self):
        """转换为字典"""
        with param_scope(**{"a": 1, "b": 2}) as ps:
            d = dict(ps)
            self.assertEqual(d["a"], 1)
            self.assertEqual(d["b"], 2)


class TestDynamicKeyAccess(TestCase):
    """动态 key 访问测试"""

    def test_bracket_access(self):
        """方括号访问 - 返回 accessor"""
        with param_scope(**{"a.b.c": 42}) as ps:
            # [] 返回 accessor，可以用 | 或 () 获取值
            self.assertEqual(ps["a.b.c"] | 0, 42)

    def test_dynamic_key_via_getattr(self):
        """动态 key 通过 getattr 访问"""
        with param_scope(**{"task_0_lr": 0.1, "task_1_lr": 0.2}) as ps:
            for i in range(2):
                attr = f"task_{i}_lr"
                expected = 0.1 * (i + 1)
                self.assertAlmostEqual(getattr(ps, attr) | 0.0, expected)

    def test_nested_attribute_access(self):
        """嵌套属性访问"""
        with param_scope(**{"model.weight": 1.0, "model.bias": 0.5}) as ps:
            self.assertEqual(ps.model.weight | 0.0, 1.0)
            self.assertEqual(ps.model.bias | 0.0, 0.5)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
