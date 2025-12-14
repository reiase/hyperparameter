"""
param_scope 核心功能测试

测试模块：
1. TestParamScopeCreate: 创建 param_scope 的各种方式
2. TestParamScopeAccess: 参数访问（读/写）
3. TestParamScopeWith: with 语句和作用域
4. TestParamScopeTypeConversion: 类型转换
5. TestParamScopeBool: 布尔值处理
6. TestParamScopeMissingVsDefault: 缺失值与默认值
7. TestParamScopeClear: 清空操作
"""

from unittest import TestCase

from hyperparameter import param_scope


class TestParamScopeCreate(TestCase):
    """测试 param_scope 创建的各种方式"""

    def test_create_empty(self):
        """从空创建"""
        ps = param_scope()
        self.assertIsNotNone(ps)

    def test_create_from_kwargs(self):
        """从关键字参数创建"""
        ps = param_scope(a=1, b=2)
        self.assertEqual(ps.a | 0, 1)
        self.assertEqual(ps.b | 0, 2)

    def test_create_from_string_args(self):
        """从字符串参数创建（key=value 格式）"""
        ps = param_scope("a=1", "b=2")
        self.assertEqual(ps.a | 0, 1)
        self.assertEqual(ps.b | 0, 2)

    def test_create_with_dotted_name(self):
        """创建带点号分隔的 key"""
        ps = param_scope("a.b.c=1")
        self.assertEqual(ps.a.b.c | 0, 1)

    def test_create_from_dict(self):
        """从字典创建"""
        ps = param_scope(**{"a.b.c": 1, "A.B.C": 2})
        self.assertEqual(ps.a.b.c | 0, 1)
        self.assertEqual(ps.A.B.C | 0, 2)

    def test_create_with_nested_dict(self):
        """从嵌套字典创建"""
        ps = param_scope(**{"a": {"b": {"c": 1}}})
        self.assertEqual(ps.a.b.c | 0, 1)

    def test_create_empty_via_static_method(self):
        """使用 empty() 静态方法创建"""
        ps = param_scope.empty()
        self.assertEqual(ps.any_key | "default", "default")

    def test_create_empty_with_params(self):
        """empty() 带参数创建"""
        ps = param_scope.empty(a=1, b=2)
        self.assertEqual(ps.a | 0, 1)
        self.assertEqual(ps.b | 0, 2)


class TestParamScopeAccess(TestCase):
    """测试参数访问（读/写）"""

    def test_access_undefined_short_name(self):
        """访问未定义的短名称，使用默认值"""
        self.assertEqual(param_scope.a | 0, 0)
        self.assertEqual(param_scope.a(1), 1)
        self.assertEqual(param_scope().a(1), 1)

    def test_access_undefined_long_name(self):
        """访问未定义的长名称，使用默认值"""
        self.assertEqual(param_scope.a.b.c | 0, 0)
        self.assertEqual(param_scope.a.b.c(1), 1)
        self.assertEqual(param_scope().a.b.c(1), 1)

    def test_direct_write_static(self):
        """直接写入（静态方式）"""
        with param_scope():
            param_scope.a = 1
            self.assertEqual(param_scope.a(), 1)

        # 检查参数不泄漏
        with self.assertRaises(KeyError):
            param_scope.a()

    def test_direct_write_instance(self):
        """直接写入（实例方式）"""
        with param_scope():
            ps = param_scope()
            ps.b = 2
            self.assertEqual(ps.b(), 2)

        # 检查参数不泄漏
        with self.assertRaises(KeyError):
            param_scope.b()

    def test_bracket_access_read(self):
        """方括号读取"""
        with param_scope(**{"a.b.c": 42}) as ps:
            self.assertEqual(ps["a.b.c"] | 0, 42)

    def test_bracket_access_dynamic_key(self):
        """方括号动态 key"""
        with param_scope(**{"task_0_lr": 0.1, "task_1_lr": 0.2}) as ps:
            for i in range(2):
                # 使用下划线避免 . 的问题
                self.assertAlmostEqual(getattr(ps, f"task_{i}_lr") | 0.0, 0.1 * (i + 1))


class TestParamScopeWith(TestCase):
    """测试 with 语句和作用域"""

    def test_with_empty(self):
        """空 with 语句"""
        with param_scope() as ps:
            self.assertEqual(ps.a | 1, 1)

    def test_with_kwargs(self):
        """带关键字参数的 with"""
        with param_scope(a=1) as ps:
            self.assertEqual(ps.a | 0, 1)

    def test_with_string_args(self):
        """带字符串参数的 with"""
        with param_scope("a=1") as ps:
            self.assertEqual(ps.a | 0, 1)

    def test_with_dict(self):
        """带字典的 with"""
        with param_scope(**{"a": 1}) as ps:
            self.assertEqual(ps.a | 0, 1)

    def test_nested_scopes(self):
        """嵌套作用域"""
        with param_scope() as ps1:
            self.assertEqual(ps1.a | "empty", "empty")
            with param_scope(a="non-empty") as ps2:
                self.assertEqual(ps2.a | "empty", "non-empty")
            self.assertEqual(ps1.a | "empty", "empty")

    def test_deeply_nested_scopes(self):
        """深度嵌套作用域"""
        with param_scope(a=1) as ps1:
            with param_scope(a=2) as ps2:
                with param_scope(a=3) as ps3:
                    with param_scope(a=4) as ps4:
                        self.assertEqual(ps4.a | 0, 4)
                    self.assertEqual(ps3.a | 0, 3)
                self.assertEqual(ps2.a | 0, 2)
            self.assertEqual(ps1.a | 0, 1)

    def test_scope_isolation(self):
        """作用域隔离：内层修改不影响外层"""
        with param_scope() as ps1:
            with param_scope(a="value") as ps2:
                ps2.b = 42
            # b 不应该泄漏到外层
            with self.assertRaises(KeyError):
                ps1.b()

    def test_scope_override_and_restore(self):
        """作用域覆盖和恢复"""
        with param_scope(key=1):
            self.assertEqual(param_scope.key(), 1)
            with param_scope(key=2):
                self.assertEqual(param_scope.key(), 2)
            self.assertEqual(param_scope.key(), 1)


class TestParamScopeTypeConversion(TestCase):
    """测试类型转换"""

    def test_default_int(self):
        """整数类型转换"""
        with param_scope(a=1, b="1", c="1.12", d="not int", e=False) as ps:
            self.assertEqual(ps.a | 0, 1)
            self.assertEqual(ps.b | 1, 1)
            self.assertEqual(ps.c | 1, 1.12)  # 保留精度
            self.assertEqual(ps.d | 1, "not int")  # 无法转换
            self.assertEqual(ps.e | 1, 0)  # False -> 0

    def test_default_float(self):
        """浮点数类型转换"""
        with param_scope(a=1, b="1", c="1.12", d="not float", e=False) as ps:
            self.assertEqual(ps.a | 0.0, 1)
            self.assertEqual(ps.b | 1.0, 1)
            self.assertAlmostEqual(ps.c | 1.0, 1.12)
            self.assertEqual(ps.d | 1.0, "not float")
            self.assertEqual(ps.e | 1.0, 0)

    def test_default_str(self):
        """字符串类型转换"""
        with param_scope(a=1, b="1", c="1.12", d="text", e=False) as ps:
            self.assertEqual(ps.a | "0", "1")
            self.assertEqual(ps.b | "0", "1")
            self.assertEqual(ps.c | "0", "1.12")
            self.assertEqual(ps.d | "0", "text")
            self.assertEqual(ps.e | "0", "False")

    def test_default_bool(self):
        """布尔类型转换"""
        with param_scope(a=1, b="1", c="1.12", d="text", e=False) as ps:
            self.assertTrue(ps.a | False)
            self.assertTrue(ps.b | False)
            self.assertFalse(ps.c | False)  # "1.12" -> False
            self.assertFalse(ps.d | False)  # "text" -> False
            self.assertFalse(ps.e | False)

    def test_bool_string_conversion(self):
        """布尔字符串转换"""
        with param_scope(
            **{
                "t1": "true",
                "t2": "True",
                "t3": "yes",
                "t4": "1",
                "f1": "false",
                "f2": "False",
                "f3": "no",
                "f4": "0",
            }
        ) as ps:
            self.assertTrue(ps.t1(False))
            self.assertTrue(ps.t2(False))
            self.assertTrue(ps.t3(False))
            self.assertTrue(ps.t4(False))
            self.assertFalse(ps.f1(True))
            self.assertFalse(ps.f2(True))
            self.assertFalse(ps.f3(True))
            self.assertFalse(ps.f4(True))


class TestParamScopeBool(TestCase):
    """测试布尔值处理"""

    def test_bool_truthy(self):
        """真值判断"""
        with param_scope(a=True, b=0, c="false") as ps:
            self.assertTrue(bool(ps.a))
            self.assertFalse(bool(ps.b))
            self.assertTrue(bool(ps.c))  # 非空字符串为真

    def test_bool_missing(self):
        """缺失值的布尔判断"""
        ps = param_scope()
        self.assertFalse(bool(ps.missing))


class TestParamScopeMissingVsDefault(TestCase):
    """测试缺失值与默认值的区别"""

    def test_missing_uses_default(self):
        """缺失值使用默认值"""
        with param_scope() as ps:
            self.assertEqual(ps.missing | 123, 123)

    def test_explicit_false_not_missing(self):
        """显式 False 不是缺失值"""
        with param_scope(flag=False) as ps:
            self.assertFalse(ps.flag | True)

    def test_explicit_zero_not_missing(self):
        """显式 0 不是缺失值"""
        with param_scope(value=0) as ps:
            self.assertEqual(ps.value | 999, 0)

    def test_explicit_empty_string_not_missing(self):
        """显式空字符串不是缺失值"""
        with param_scope(text="") as ps:
            self.assertEqual(ps.text | "default", "")


class TestParamScopeClear(TestCase):
    """测试清空操作"""

    def test_clear_on_empty(self):
        """清空空存储"""
        ps = param_scope.empty()
        ps.clear()  # 不应抛出异常

    def test_clear_removes_all(self):
        """清空移除所有参数"""
        ps = param_scope(a=1, b=2, c=3)
        ps.clear()
        self.assertEqual(ps.a | "gone", "gone")
        self.assertEqual(ps.b | "gone", "gone")
        self.assertEqual(ps.c | "gone", "gone")


class TestParamScopeKeys(TestCase):
    """测试 keys 操作"""

    def test_keys_returns_all(self):
        """keys() 返回所有 key"""
        with param_scope(**{"a": 1, "b.c": 2, "d.e.f": 3}) as ps:
            keys = list(ps.keys())
            self.assertIn("a", keys)
            self.assertIn("b.c", keys)
            self.assertIn("d.e.f", keys)

    def test_keys_contains_set_keys(self):
        """keys() 包含已设置的 key"""
        with param_scope.empty(test_key=42) as ps:
            keys = list(ps.keys())
            self.assertIn("test_key", keys)


class TestParamScopeIteration(TestCase):
    """测试迭代操作"""

    def test_dict_conversion(self):
        """转换为字典"""
        with param_scope(**{"a": 1, "b": 2}) as ps:
            # 使用 storage() 获取底层存储
            storage = ps.storage()
            if hasattr(storage, "storage"):
                d = storage.storage()
            else:
                d = dict(storage) if hasattr(storage, "__iter__") else {}
            self.assertEqual(d.get("a"), 1)
            self.assertEqual(d.get("b"), 2)

    def test_keys_access(self):
        """通过 keys() 访问"""
        with param_scope(**{"x": 10, "y": 20}) as ps:
            keys = list(ps.keys())
            self.assertIn("x", keys)
            self.assertIn("y", keys)


if __name__ == "__main__":
    import pytest

    pytest.main([__file__, "-v"])
