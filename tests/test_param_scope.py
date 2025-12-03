from unittest import TestCase

from hyperparameter import param_scope


class TestParamScopeCreate(TestCase):
    def test_param_scope_create_from_empty(self):
        ps = param_scope()

    def test_param_scope_create_from_kwargs(self):
        ps = param_scope(a=1, b=2)
        assert ps.a | 0 == 1
        assert ps.b | 0 == 2

    def test_param_scope_create_from_args(self):
        ps = param_scope("a=1", "b=2")
        assert ps.a | 0 == 1
        assert ps.b | 0 == 2

    def test_param_scope_create_with_long_name(self):
        ps = param_scope("a.b.c=1")
        assert ps.a.b.c | 0 == 1

    def test_param_scope_create_from_dict(self):
        ps = param_scope(**{"a.b.c": 1, "A.B.C": 2})
        assert ps.a.b.c | 0 == 1
        assert ps.A.B.C | 0 == 2


class TestParamScopeDirectAccess(TestCase):
    def test_param_scope_undefined_short_name(self):
        assert param_scope.a | 0 == 0
        assert param_scope.a(1) == 1
        assert param_scope().a(1) == 1

    def test_param_scope_undefined_with_long_name(self):
        assert param_scope.a.b.c | 0 == 0
        assert param_scope.a.b.c(1) == 1
        assert param_scope().a.b.c(1) == 1

    def test_param_scope_direct_write(self):
        with param_scope():
            param_scope.a = 1
            param_scope().b = 2

            assert param_scope.a() == 1
            assert param_scope.b() == None

            ps = param_scope()
            ps.b = 2
            assert ps.b() == 2
            with ps:
                param_scope.b() == 2

        # check for param leak
        assert param_scope.a() == None
        assert param_scope.b() == None


class TestParamScopeWith(TestCase):
    def test_with_param_scope(self):
        with param_scope() as ps:
            assert ps.a | 1 == 1
        with param_scope(a=1) as ps:
            assert ps.a | 0 == 1
        with param_scope("a=1") as ps:
            assert ps.a | 0 == 1
        with param_scope(**{"a": 1}) as ps:
            assert ps.a | 0 == 1

    def test_nested_param_scope(self):
        with param_scope() as ps1:
            assert ps1.a | "empty" == "empty"
            with param_scope(a="non-empty") as ps2:
                assert ps2.a | "empty" == "non-empty"
                with param_scope() as ps3:
                    with param_scope() as ps4:
                        assert ps2.a | "empty" == "non-empty"
            assert ps1.a | "empty" == "empty"
        assert param_scope.a | "empty" == "empty"


class TestParamScopeGetOrElse(TestCase):
    def test_param_scope_default_int(self):
        with param_scope(a=1, b="1", c="1.12", d="not int", e=False) as ps:
            assert ps.a | 0 == 1
            assert ps.b | 1 == 1
            assert ps.c | 1 == 1.12
            assert ps.d | 1 == "not int"
            assert ps.e | 1 == 0

    def test_param_scope_default_float(self):
        with param_scope(a=1, b="1", c="1.12", d="not int", e=False) as ps:
            assert ps.a | 0.0 == 1
            assert ps.b | 1.0 == 1
            assert ps.c | 1.0 == 1.12
            assert ps.d | 1.0 == "not int"
            assert ps.e | 1.0 == 0

    def test_param_scope_default_str(self):
        with param_scope(a=1, b="1", c="1.12", d="not int", e=False) as ps:
            assert ps.a | "0" == "1"
            assert ps.b | "1" == "1"
            assert ps.c | "1" == "1.12"
            assert ps.d | "1" == "not int"
            assert ps.e | "1" == "False"

    def test_param_scope_default_bool(self):
        with param_scope(a=1, b="1", c="1.12", d="not int", e=False) as ps:
            assert ps.a | False == True
            assert ps.b | False == True
            assert ps.c | False == False
            assert ps.d | False == False
            assert ps.e | False == False


class TestParamScopeBool(TestCase):
    def test_param_scope_bool_truthy(self):
        with param_scope(a=True, b=0, c="false") as ps:
            assert bool(ps.a) is True
            assert bool(ps.b) is False
            assert bool(ps.c) is True

    def test_param_scope_bool_missing(self):
        ps = param_scope()
        assert bool(ps.missing) is False


class TestParamScopeClear(TestCase):
    def test_clear_on_empty(self):
        # Should not raise when clearing an empty storage
        ps = param_scope.empty()
        ps.clear()
