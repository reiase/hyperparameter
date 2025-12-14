import sys
from unittest import TestCase

import hyperparameter as hp
import hyperparameter.cli as hp_cli


# Module-level param to test global discovery
@hp.param("global_func")
def global_func(x=1):
    return ("global", x)


class TestLaunch(TestCase):
    def test_launch_single_function(self):
        calls = []

        @hp.param("foo")
        def foo(a=1, b=2):
            calls.append((a, b))
            return a, b

        argv_backup = sys.argv
        sys.argv = ["prog", "--b", "5"]
        try:
            result = hp.launch(foo)
        finally:
            sys.argv = argv_backup

        self.assertEqual(result, (1, 5))
        self.assertEqual(calls[-1], (1, 5))

    def test_launch_subcommands_and_define(self):
        calls = {"foo": [], "bar": []}

        @hp.param("foo")
        def foo(a=1, b=2):
            calls["foo"].append((a, b))
            return a, b

        @hp.param("bar")
        def bar(x=0):
            calls["bar"].append(x)
            return x

        argv_backup = sys.argv
        sys.argv = ["prog", "foo", "-D", "foo.b=7"]
        try:
            result = hp.launch()
        finally:
            sys.argv = argv_backup

        self.assertEqual(result, (1, 7))
        self.assertEqual(calls["foo"][-1], (1, 7))
        self.assertFalse(calls["bar"])  # bar not executed

    def test_launch_subcommands_positional_and_types(self):
        calls = {"foo": [], "bar": []}

        @hp.param("foo")
        def foo(a, b: int = 2, c: float = 0.5, flag: bool = True):
            calls["foo"].append((a, b, c, flag))
            return a, b, c, flag

        @hp.param("bar")
        def bar(x=0):
            calls["bar"].append(x)
            return x

        argv_backup = sys.argv
        sys.argv = ["prog", "foo", "3", "--b", "4", "--c", "1.5", "--flag", "False"]
        try:
            result = hp.launch()
        finally:
            sys.argv = argv_backup

        self.assertEqual(result, ("3", 4, 1.5, False))
        self.assertEqual(calls["foo"][-1], ("3", 4, 1.5, False))
        self.assertFalse(calls["bar"])  # bar not executed

    def test_launch_collects_locals_and_globals(self):
        def local_runner():
            @hp.param("local_func")
            def local_func(y=2):
                return ("local", y)

            argv_backup = sys.argv
            sys.argv = ["prog", "local_func", "--y", "5"]
            try:
                return hp.launch()
            finally:
                sys.argv = argv_backup

        result = local_runner()
        self.assertEqual(result, ("local", 5))

        argv_backup = sys.argv
        sys.argv = ["prog", "global_func", "--x", "9"]
        try:
            result_global = hp.launch()
        finally:
            sys.argv = argv_backup
        self.assertEqual(result_global, ("global", 9))

    def test_help_from_docstring(self):
        @hp.param("doc_func")
        def doc_func(a, b=2):
            """Doc summary.

            Args:
                a: first arg
                b (int): second arg
            """
            return a, b

        parser = hp_cli._build_parser_for_func(doc_func)
        actions = {action.dest: action for action in parser._actions}
        self.assertEqual(actions["a"].help, "first arg")
        self.assertEqual(actions["b"].help, "second arg (default: 2)")

    def test_help_from_numpy_and_rest(self):
        @hp.param("numpy_style")
        def numpy_style(x, y=1):
            """NumPy style.

            Parameters
            ----------
            x : int
                the x value
            y : int
                the y value
            """
            return x, y

        @hp.param("rest_style")
        def rest_style(p, q=3):
            """
            :param p: first param
            :param q: second param
            """
            return p, q

        parser_numpy = hp_cli._build_parser_for_func(numpy_style)
        actions_numpy = {action.dest: action for action in parser_numpy._actions}
        self.assertEqual(actions_numpy["x"].help, "the x value")
        self.assertIn("y", actions_numpy)

        parser_rest = hp_cli._build_parser_for_func(rest_style)
        actions_rest = {action.dest: action for action in parser_rest._actions}
        self.assertEqual(actions_rest["p"].help, "first param")
        self.assertEqual(actions_rest["q"].help, "second param (default: 3)")
