from unittest import TestCase

from hyperparameter import auto_param, param_scope


class TestAutoParam(TestCase):
    def test_auto_param_func(self):
        @auto_param("foo")
        def foo(a, b=1, c=2.0, d=False, e="str"):
            return a, b, c, d, e

        with param_scope(**{"foo.b": 2}):
            self.assertEqual(foo(1), (1, 2, 2.0, False, "str"))

        with param_scope(**{"foo.c": 3.0}):
            self.assertEqual(foo(1), (1, 1, 3.0, False, "str"))

    def test_auto_param_func2(self):
        @auto_param("foo")
        def foo(a, b=1, c=2.0, d=False, e="str"):
            return a, b, c, d, e

        with param_scope():
            param_scope.foo.b = 2
            self.assertEqual(foo(1), (1, 2, 2.0, False, "str"))
            param_scope.foo.c = 3.0
            self.assertEqual(foo(1), (1, 2, 3.0, False, "str"))
        self.assertEqual(foo(1), (1, 1, 2.0, False, "str"))
