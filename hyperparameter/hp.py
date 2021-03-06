import inspect
import json
import threading

from typing import Any, Dict
from typing import Callable


class Tracker:
    """
    tracker for python read/write operations
    """
    rlist = set()

    wlist = set()
    callback = None

    @staticmethod
    def reads():
        retval = list(Tracker.rlist)
        retval.sort()
        return retval

    @staticmethod
    def writes():
        retval = list(Tracker.wlist)
        retval.sort()
        return retval

    @staticmethod
    def all():
        retval = list(Tracker.rlist.union(Tracker.wlist))
        retval.sort()
        return retval

    @staticmethod
    def report():
        retvals = [
            'parameter reads: \n  {}'.format('\n  '.join(Tracker.reads())),
            'parameter writes: \n  {}'.format('\n  '.join(Tracker.writes())),
        ]
        return '\n'.join(retvals)

    @staticmethod
    def set_tracker(func: Callable[[Dict[str, Any]], None]):
        """ report hyper-parameter value to a tracker, for example, `mlflow.tracking`
        """
        Tracker.callback = func


class Accessor(dict):
    """
    Helper for accessing hyper-parameters.

    When reading an undefined parameter, accessor will:
    1. return false in `if` statement:
    >>> params = HyperParameter()
    >>> if (not params.undefined_int): print("parameter undefined")
    parameter undefined

    2. support default value for undefined parameter
    >>> params = HyperParameter()
    >>> params.undefined_int.getOrElse(10)
    10

    3. support to create nested parameter:
    >>> params = HyperParameter()
    >>> params.undefined_object.undefined_prop = 1
    >>> print(params)
    {'undefined_object': {'undefined_prop': 1}}
    """

    def __init__(self, root, path=None):
        self._root = root
        self._path = path

    def getOrElse(self, default: Any):
        """
        get value for the parameter, or get default value if parameter is not defined.
        """
        Tracker.rlist.add(self._path)
        value = self._root.get(self._path)
        return default if value is None else value

    def __getattr__(self, name: str) -> Any:
        if name in ['_path', '_root']:
            return self[name]

        if self._path:
            name = '{}.{}'.format(self._path, name)
        return Accessor(self._root, name)

    def __setattr__(self, name: str, value: Any):
        if name in ['_path', '_root']:
            return self.__setitem__(name, value)
        full_name = '{}.{}'.format(self._path,
                                   name) if self._path is not None else name
        Tracker.wlist.add(full_name)
        root = self._root
        for path in self._path.split('.'):
            root[path] = HyperParameter()
            root = root[path]
        root[name] = value
        return value

    def __str__(self):
        return ''

    def __bool__(self):
        return False

    def __call__(self, default: Any) -> Any:
        """
        shortcut for getOrElse
        """
        return self.getOrElse(default)

    __nonzero__ = __bool__


class HyperParameter(dict):
    '''
    HyperParameter is an extended dict with features for better parameter management.

    A HyperParameter can be create with:
    >>> hp = HyperParameter(param1=1, param2=2, obj1={'propA': 'A'})

    or

    >>> hp = HyperParameter(**{'param1': 1, 'param2': 2, 'obj1': {'propA': 'A'}})

    Once the HyperParameter object is created, you can access the values using the object-style api:
    >>> hp.param1
    1
    >>> hp.obj1.propA
    'A'

    or using the dict-style api (for legacy codes):
    >>> hp['param1']
    1
    >>> hp['obj1']['propA']
    'A'

    The object-style api also support creating or updating the parameters:
    >>> hp.a.b.c = 1

    which avoid to maintain the dict data manually like this:
    >>> hp = {}
    >>> if 'a' not in hp: hp['a'] = {}
    >>> if 'b' not in hp['a']: hp['a']['b'] = {}
    >>> hp['a']['b']['c'] = 1

    You can also create a parameter with a string name:
    >>> hp = HyperParameter()
    >>> hp.put('a.b.c', 1)
    '''

    def __init__(self, **kws):
        super(HyperParameter, self).__init__()
        self.update(kws)

    def update(self, kws):
        for k, v in kws.items():
            if isinstance(v, dict):
                if k in self and isinstance(self[k], dict):
                    vv = HyperParameter(**self[k])
                    vv.update(v)
                    v = vv
                else:
                    v = HyperParameter(**v)
            self[k] = v

    def put(self, name: str, value: Any):
        """
        put/update a parameter with string name

        Args:
            name (str): parameter name, 'obj.prop' is supported
            value (Any): parameter value

        Examples:
        >>> cfg = HyperParameter()
        >>> cfg.put('param1', 1)
        >>> cfg.put('obj1.propA', 'A')

        >>> cfg.param1
        1
        >>> cfg.obj1.propA
        'A'
        """
        path = name.split('.')
        obj = self
        for p in path[:-1]:
            if p not in obj or (not isinstance(obj[p], dict)):
                obj[p] = HyperParameter()
            obj = obj[p]
        Tracker.wlist.add(name)
        obj[path[-1]] = safe_numeric(value)

    def get(self, name: str) -> Any:
        """
        get a parameter by a string name

        Args:
            name (str): parameter name

        Returns:
            Any: parameter value

        Examples:
        >>> cfg = HyperParameter(a=1, b = {'c':2, 'd': 3})
        >>> cfg.get('a')
        1
        >>> cfg.get('b.c')
        2
        """
        path = name.split('.')
        obj = self
        for p in path[:-1]:
            if p not in obj:
                return None
            obj = obj[p]
        Tracker.rlist.add(name)
        return obj[path[-1]] if path[-1] in obj else None

    def __setitem__(self, key, value):
        if isinstance(value, dict):
            return dict.__setitem__(self, key, HyperParameter(**value))
        return dict.__setitem__(self, key, value)

    def __getattr__(self, name):
        """
        read parameter with object-style api
        """
        if name in self.keys():
            return self[name]
        else:
            if name in self.__dict__.keys():
                return self.__dict__[name]
            return Accessor(self, name)

    def __setattr__(self, name, value):
        """
        create/update parameter with object-style api
        """
        self[name] = value

    def __call__(self) -> Any:
        """
        Return a parameter accessor.

        Returns:
            Any: holder of current parameter

        Examples:
        >>> cfg = HyperParameter(a=1, b = {'c':2, 'd': 3})
        >>> cfg().a.getOrElse(2)
        1
        >>> cfg().b.c.getOrElse(2)
        2
        >>> cfg().b.d.getOrElse(2)
        3
        >>> cfg().b.e.getOrElse(2)
        2
        """

        return Accessor(self, None)

    @staticmethod
    def from_json(s):
        obj = json.loads(s)
        return HyperParameter(**obj)

    @staticmethod
    def from_json_file(path):
        with open(path) as f:
            obj = json.load(f)
            return HyperParameter(**obj)


class param_scope(HyperParameter):
    '''
    thread safe scoped hyper parameeter

    Examples:
    create a scoped HyperParameter
    >>> with param_scope(**{'a': 1, 'b': 2}) as cfg:
    ...     print(cfg.a)
    1

    read parameter in a function
    >>> def foo():
    ...    with param_scope() as cfg:
    ...        return cfg.a
    >>> with param_scope(**{'a': 1, 'b': 2}) as cfg:
    ...     foo() # foo should get cfg using a with statement
    1

    update some config only in new scope
    >>> with param_scope(**{'a': 1, 'b': 2}) as cfg:
    ...     cfg.b
    ...     with param_scope(**{'b': 3}) as newcfg:
    ...         newcfg.b
    2
    3
    '''
    tls = threading.local()

    def __init__(self, *args, **kws):
        if hasattr(param_scope.tls,
                   '_cfg_') and len(param_scope.tls._cfg_) > 0:
            self.update(param_scope.tls._cfg_[-1])
        self.update(kws)
        for newcfg in args:
            if '=' in newcfg:
                k, v = newcfg.split('=', 1)
                self.put(k, v)

    def __enter__(self):
        if not hasattr(param_scope.tls, '_cfg_'):
            param_scope.tls._cfg_ = []
        param_scope.tls._cfg_.append(self)
        return param_scope.tls._cfg_[-1]

    def __exit__(self, exc_type, exc_value, traceback):
        param_scope.tls._cfg_.pop()

    @staticmethod
    def init(params):
        """
        init param_scope for a new thread.
        """
        if not hasattr(param_scope.tls, '_cfg_'):
            param_scope.tls._cfg_ = []
            param_scope.tls._cfg_.append(params)


def auto_param(func):
    """
    Convert keyword arguments into hyperparameters

    Examples:

    >>> @auto_param
    ... def foo(a, b=2, c='c', d=None):
    ...     print(a, b, c, d)

    >>> foo(1)
    1 2 c None

    >>> with param_scope('foo.b=3'):
    ...     foo(2)
    2 3 c None
    """
    predef_kws = {}
    predef_val = {}

    namespace = func.__module__
    if namespace == '__main__':
        namespace = None
    if namespace is not None:
        namespace += '.{}'.format(func.__name__)
    else:
        namespace = func.__name__

    signature = inspect.signature(func)
    for k, v in signature.parameters.items():
        if v.default != v.empty:
            name = '{}.{}'.format(namespace, k)
            predef_kws[k] = name
            Tracker.rlist.add(name)
            predef_val[name] = v.default

    def wrapper(*arg, **kws):
        with param_scope() as hp:
            local_params = {}
            for k, v in predef_kws.items():
                if hp.get(v) is not None and k not in kws:
                    kws[k] = hp.get(v)
                    local_params[v] = hp.get(v)
                else:
                    local_params[v] = predef_val[v]
            if Tracker.callback is not None:
                Tracker.callback(local_params)
            return func(*arg, **kws)

    return wrapper


def safe_numeric(value):
    if isinstance(value, str):
        try:
            return int(value)
        except:
            pass
        try:
            return float(value)
        except:
            pass
    return value


if __name__ == "__main__":
    import doctest
    doctest.testmod(verbose=False)

    import unittest

    class TestHyperParameter(unittest.TestCase):

        def test_parameter_create(self):
            param1 = HyperParameter(a=1, b=2)
            self.assertEqual(param1.a, 1)
            self.assertEqual(param1.b, 2)

            param2 = HyperParameter(**{'a': 1, 'b': 2})
            self.assertEqual(param2.a, 1)
            self.assertEqual(param2.b, 2)

        def test_parameter_update_with_holder(self):
            param1 = HyperParameter()
            param1.a = 1
            param1.b = 2
            param1.c.b.a = 3
            self.assertDictEqual(param1, {
                'a': 1,
                'b': 2,
                'c': {
                    'b': {
                        'a': 3
                    }
                }
            })

        def test_parameter_update(self):
            param1 = HyperParameter()
            param1.put('c.b.a', 1)
            self.assertDictEqual(param1, {'c': {'b': {'a': 1}}})

        def test_parameter_patch(self):
            param1 = HyperParameter()
            param1.update({'a': 1, 'b': 2})
            self.assertEqual(param1.a, 1)
            self.assertEqual(param1.b, 2)

    class TestHolder(unittest.TestCase):

        def test_holder_as_bool(self):
            param1 = HyperParameter()
            self.assertFalse(param1.a.b)

            param1.a.b = False
            self.assertFalse(param1.a.b)

            param1.a.b = True
            self.assertTrue(param1.a.b)

    class TestParamScope(unittest.TestCase):

        def test_scope_create(self):
            with param_scope(a=1, b=2) as hp:
                self.assertEqual(hp.a, 1)
                self.assertEqual(hp.b, 2)

            with param_scope(**{'a': 1, 'b': 2}) as hp:
                self.assertEqual(hp.a, 1)
                self.assertEqual(hp.b, 2)

        def test_nested_scope(self):
            with param_scope(a=1, b=2) as hp1:
                self.assertEqual(hp1.a, 1)

                with param_scope(a=3) as hp2:
                    self.assertEqual(hp2.a, 3)

        def test_scope_with_function_call(self):

            def read_a():
                with param_scope() as hp:
                    return hp.a

            self.assertFalse(read_a())

            with param_scope(a=1):
                self.assertEqual(read_a(), 1)
            with param_scope(a=2):
                self.assertEqual(read_a(), 2)

            with param_scope(a=1):
                self.assertEqual(read_a(), 1)
                with param_scope(a=2):
                    self.assertEqual(read_a(), 2)
                self.assertEqual(read_a(), 1)

    unittest.main()
