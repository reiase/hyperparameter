"""
@auto_param 装饰器测试

测试模块：
1. TestAutoParamBasic: 基础功能
2. TestAutoParamWithScope: 与 param_scope 配合使用
3. TestAutoParamPriority: 参数优先级
4. TestAutoParamClass: 类装饰器
5. TestAutoParamNamespace: 命名空间
"""

from unittest import TestCase

from hyperparameter import auto_param, param_scope


class TestAutoParamBasic(TestCase):
    """@auto_param 基础功能测试"""

    def test_basic_function(self):
        """基础函数装饰"""

        @auto_param("foo")
        def foo(a, b=1, c=2.0, d=False, e="str"):
            return a, b, c, d, e

        result = foo(0)
        self.assertEqual(result, (0, 1, 2.0, False, "str"))

    def test_all_default_args(self):
        """全默认参数"""

        @auto_param("func")
        def func(a=1, b=2, c=3):
            return a, b, c

        self.assertEqual(func(), (1, 2, 3))

    def test_no_default_args(self):
        """无默认参数"""

        @auto_param("func")
        def func(a, b, c):
            return a, b, c

        self.assertEqual(func(1, 2, 3), (1, 2, 3))

    def test_mixed_args(self):
        """混合参数"""

        @auto_param("func")
        def func(a, b=2):
            return a, b

        self.assertEqual(func(1), (1, 2))
        self.assertEqual(func(1, 3), (1, 3))


class TestAutoParamWithScope(TestCase):
    """@auto_param 与 param_scope 配合测试"""

    def test_scope_override_dict(self):
        """使用字典覆盖"""

        @auto_param("foo")
        def foo(a, b=1, c=2.0, d=False, e="str"):
            return a, b, c, d, e

        with param_scope(**{"foo.b": 2}):
            self.assertEqual(foo(1), (1, 2, 2.0, False, "str"))

        with param_scope(**{"foo.c": 3.0}):
            self.assertEqual(foo(1), (1, 1, 3.0, False, "str"))

    def test_scope_override_direct(self):
        """直接属性覆盖"""

        @auto_param("foo")
        def foo(a, b=1, c=2.0, d=False, e="str"):
            return a, b, c, d, e

        with param_scope():
            param_scope.foo.b = 2
            self.assertEqual(foo(1), (1, 2, 2.0, False, "str"))
            param_scope.foo.c = 3.0
            self.assertEqual(foo(1), (1, 2, 3.0, False, "str"))

        # 作用域外恢复默认
        self.assertEqual(foo(1), (1, 1, 2.0, False, "str"))

    def test_scope_override_all(self):
        """覆盖所有参数"""

        @auto_param("func")
        def func(a=1, b=2, c=3):
            return a, b, c

        with param_scope(**{"func.a": 10, "func.b": 20, "func.c": 30}):
            self.assertEqual(func(), (10, 20, 30))

    def test_nested_scope_override(self):
        """嵌套作用域覆盖"""

        @auto_param("func")
        def func(x=1):
            return x

        with param_scope(**{"func.x": 10}):
            self.assertEqual(func(), 10)
            with param_scope(**{"func.x": 20}):
                self.assertEqual(func(), 20)
            self.assertEqual(func(), 10)


class TestAutoParamPriority(TestCase):
    """参数优先级测试：直接传参 > scope 覆盖 > 默认值"""

    def test_direct_arg_highest_priority(self):
        """直接传参优先级最高"""

        @auto_param("func")
        def func(x=1):
            return x

        with param_scope(**{"func.x": 10}):
            # 直接传参覆盖 scope
            self.assertEqual(func(x=100), 100)

    def test_scope_over_default(self):
        """scope 覆盖默认值"""

        @auto_param("func")
        def func(x=1):
            return x

        with param_scope(**{"func.x": 10}):
            self.assertEqual(func(), 10)

    def test_default_when_no_override(self):
        """无覆盖时使用默认值"""

        @auto_param("func")
        def func(x=1):
            return x

        self.assertEqual(func(), 1)


class TestAutoParamClass(TestCase):
    """类装饰器测试"""

    def test_class_init(self):
        """类 __init__ 参数"""

        @auto_param("MyClass")
        class MyClass:
            def __init__(self, x=1, y=2):
                self.x = x
                self.y = y

        obj = MyClass()
        self.assertEqual(obj.x, 1)
        self.assertEqual(obj.y, 2)

    def test_class_with_scope(self):
        """类与 scope 配合"""

        @auto_param("MyClass")
        class MyClass:
            def __init__(self, x=1, y=2):
                self.x = x
                self.y = y

        with param_scope(**{"MyClass.x": 10}):
            obj = MyClass()
            self.assertEqual(obj.x, 10)
            self.assertEqual(obj.y, 2)

    def test_class_direct_arg(self):
        """类直接传参"""

        @auto_param("MyClass")
        class MyClass:
            def __init__(self, x=1, y=2):
                self.x = x
                self.y = y

        with param_scope(**{"MyClass.x": 10}):
            obj = MyClass(x=100)
            self.assertEqual(obj.x, 100)


class TestAutoParamNamespace(TestCase):
    """命名空间测试"""

    def test_custom_namespace(self):
        """自定义命名空间"""

        @auto_param("myns.func")
        def func(a=1):
            return a

        with param_scope(**{"myns.func.a": 42}):
            self.assertEqual(func(), 42)

    def test_deep_namespace(self):
        """深层命名空间"""

        @auto_param("a.b.c.d.func")
        def func(x=1):
            return x

        with param_scope(**{"a.b.c.d.func.x": 100}):
            self.assertEqual(func(), 100)

    def test_no_namespace(self):
        """无命名空间（使用函数名）"""

        @auto_param
        def myfunc(x=1):
            return x

        with param_scope(**{"myfunc.x": 50}):
            self.assertEqual(myfunc(), 50)

    def test_multiple_functions_same_namespace(self):
        """同一命名空间多个函数"""

        @auto_param("shared")
        def func1(a=1):
            return a

        @auto_param("shared")
        def func2(a=2):
            return a

        with param_scope(**{"shared.a": 100}):
            self.assertEqual(func1(), 100)
            self.assertEqual(func2(), 100)


class TestAutoParamTypeConversion(TestCase):
    """类型转换测试"""

    def test_string_to_int(self):
        """字符串转整数"""

        @auto_param("func")
        def func(x=1):
            return x

        with param_scope(**{"func.x": "42"}):
            result = func()
            self.assertEqual(result, 42)

    def test_string_to_float(self):
        """字符串转浮点数"""

        @auto_param("func")
        def func(x=1.0):
            return x

        with param_scope(**{"func.x": "3.14"}):
            result = func()
            self.assertAlmostEqual(result, 3.14)

    def test_string_to_bool(self):
        """字符串转布尔"""

        @auto_param("func")
        def func(flag=False):
            return flag

        with param_scope(**{"func.flag": "true"}):
            self.assertTrue(func())

        with param_scope(**{"func.flag": "false"}):
            self.assertFalse(func())


if __name__ == "__main__":
    import pytest

    pytest.main([__file__, "-v"])
