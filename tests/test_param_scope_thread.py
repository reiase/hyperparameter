"""
线程安全测试

测试模块：
1. TestThreadIsolation: 线程隔离
2. TestFrozenPropagation: frozen() 传播
3. TestMultipleThreads: 多线程并发
"""
from threading import Thread
from unittest import TestCase

from hyperparameter import param_scope


class TestThreadIsolation(TestCase):
    """线程隔离测试"""

    def _in_thread(self, key, expected_val):
        """在新线程中检查参数值"""
        ps = param_scope()
        if expected_val is None:
            with self.assertRaises(KeyError):
                getattr(ps, key)()
        else:
            self.assertEqual(getattr(ps, key)(), expected_val)

    def test_new_thread_isolated(self):
        """新线程不继承主线程的参数"""
        with param_scope(**{"a.b": 42}):
            t = Thread(target=self._in_thread, args=("a.b", None))
            t.start()
            t.join()

    def test_thread_local_modification(self):
        """线程内修改不影响其他线程"""
        results = []

        def worker(val):
            with param_scope(**{"x": val}):
                results.append(param_scope.x())

        threads = [Thread(target=worker, args=(i,)) for i in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(sorted(results), list(range(10)))


class TestFrozenPropagation(TestCase):
    """frozen() 传播测试"""

    def test_frozen_propagates_to_new_thread(self):
        """frozen() 传播到新线程"""
        with param_scope() as ps:
            param_scope.A.B = 1
            param_scope.frozen()

        result = []

        def target():
            result.append(param_scope.A.B())

        t = Thread(target=target)
        t.start()
        t.join()

        self.assertEqual(result[0], 1)

    def test_frozen_multiple_values(self):
        """frozen() 传播多个值"""
        with param_scope(**{"x": 1, "y": 2, "z": 3}):
            param_scope.frozen()

        results = {}

        def target():
            results["x"] = param_scope.x()
            results["y"] = param_scope.y()
            results["z"] = param_scope.z()

        t = Thread(target=target)
        t.start()
        t.join()

        self.assertEqual(results, {"x": 1, "y": 2, "z": 3})

    def test_frozen_update(self):
        """多次 frozen() 更新全局状态"""
        with param_scope(**{"val": 1}):
            param_scope.frozen()

        results = []

        def check():
            results.append(param_scope.val())

        t1 = Thread(target=check)
        t1.start()
        t1.join()

        with param_scope(**{"val": 2}):
            param_scope.frozen()

        t2 = Thread(target=check)
        t2.start()
        t2.join()

        self.assertEqual(results, [1, 2])


class TestMultipleThreads(TestCase):
    """多线程并发测试"""

    def test_concurrent_read(self):
        """并发读取"""
        with param_scope(**{"shared": 42}):
            param_scope.frozen()

        results = []
        errors = []

        def reader(expected):
            try:
                val = param_scope.shared()
                results.append(val == expected)
            except Exception as e:
                errors.append(str(e))

        threads = [Thread(target=reader, args=(42,)) for _ in range(20)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0)
        self.assertTrue(all(results))

    def test_concurrent_write_isolation(self):
        """并发写入隔离"""
        results = {}
        lock = __import__("threading").Lock()

        def writer(thread_id):
            with param_scope(**{"tid": thread_id}):
                for _ in range(100):
                    val = param_scope.tid()
                    if val != thread_id:
                        with lock:
                            results[thread_id] = False
                        return
                with lock:
                    results[thread_id] = True

        threads = [Thread(target=writer, args=(i,)) for i in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertTrue(all(results.values()))

    def test_nested_scope_in_thread(self):
        """线程中的嵌套作用域"""
        results = []

        def worker():
            with param_scope(**{"outer": 1}):
                results.append(param_scope.outer())
                with param_scope(**{"outer": 2, "inner": 3}):
                    results.append(param_scope.outer())
                    results.append(param_scope.inner())
                results.append(param_scope.outer())

        t = Thread(target=worker)
        t.start()
        t.join()

        self.assertEqual(results, [1, 2, 3, 1])


if __name__ == "__main__":
    import pytest
    pytest.main([__file__, "-v"])
