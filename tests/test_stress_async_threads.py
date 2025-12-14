"""
多线程异步模式压力测试

本测试文件专门用于测试Python下多线程+异步模式的正确性，
通过高并发场景验证参数隔离、上下文传递和异常恢复等功能。
"""
import asyncio
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from typing import List, Dict, Tuple, Set
import pytest

from hyperparameter import param_scope


class TestStressAsyncThreads:
    """多线程异步压力测试类"""

    @pytest.mark.asyncio
    async def test_stress_concurrent_async_tasks(self):
        """测试大量并发异步任务的参数隔离"""
        num_tasks = 1000
        results: List[Tuple[int, int]] = []

        async def worker(task_id: int):
            with param_scope(**{"TASK_ID": task_id}):
                # 模拟一些异步操作
                await asyncio.sleep(0.001)
                # 验证参数隔离
                val = param_scope.TASK_ID()
                results.append((task_id, val))
                return val

        # 创建大量并发任务
        tasks = [worker(i) for i in range(num_tasks)]
        await asyncio.gather(*tasks)

        # 验证所有任务都看到了正确的参数值
        assert len(results) == num_tasks
        result_dict = dict(results)
        for i in range(num_tasks):
            assert result_dict[i] == i, f"Task {i} saw wrong value: {result_dict[i]}"

    def test_stress_multi_thread_async(self):
        """测试多线程+异步的混合场景"""
        num_threads = 20
        tasks_per_thread = 50
        thread_results: List[List[Tuple[int, int, int]]] = []
        lock = threading.Lock()

        def thread_worker(thread_id: int):
            """每个线程运行自己的异步事件循环"""
            async def async_worker(task_id: int):
                with param_scope(**{"THREAD_ID": thread_id, "TASK_ID": task_id}):
                    await asyncio.sleep(0.001)
                    thread_val = param_scope.THREAD_ID()
                    task_val = param_scope.TASK_ID()
                    return (thread_id, task_id, thread_val, task_val)

            async def run_all():
                tasks = [async_worker(i) for i in range(tasks_per_thread)]
                results = await asyncio.gather(*tasks)
                with lock:
                    while len(thread_results) <= thread_id:
                        thread_results.append(None)
                    thread_results[thread_id] = results

            asyncio.run(run_all())

        # 启动多个线程
        threads = [threading.Thread(target=thread_worker, args=(i,)) for i in range(num_threads)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 验证结果
        assert len(thread_results) == num_threads
        for thread_id, results in enumerate(thread_results):
            assert len(results) == tasks_per_thread
            for task_id, result_tuple in enumerate(results):
                t_id, task_id_val, thread_val, task_val = result_tuple
                assert t_id == thread_id, f"Thread {thread_id} task {task_id} saw wrong thread_id: {t_id}"
                assert thread_val == thread_id, f"Thread {thread_id} task {task_id} saw wrong thread_val: {thread_val}"
                assert task_id_val == task_id, f"Thread {thread_id} task {task_id} saw wrong task_id: {task_id_val}"
                assert task_val == task_id, f"Thread {thread_id} task {task_id} saw wrong task_val: {task_val}"

    @pytest.mark.asyncio
    async def test_stress_nested_scopes_async(self):
        """测试嵌套作用域在异步环境下的正确性"""
        num_tasks = 500
        results: List[Tuple[int, int, int]] = []

        async def worker(task_id: int):
            # 外层作用域
            with param_scope(**{"OUTER": task_id * 10}):
                outer_val = param_scope.OUTER()
                
                # 内层作用域
                with param_scope(**{"INNER": task_id * 100}):
                    inner_val = param_scope.INNER()
                    outer_val_inside = param_scope.OUTER()
                    await asyncio.sleep(0.001)
                    
                    # 创建嵌套异步任务
                    async def nested():
                        with param_scope(**{"NESTED": task_id * 1000}):
                            await asyncio.sleep(0.001)
                            return (
                                param_scope.OUTER(),
                                param_scope.INNER(),
                                param_scope.NESTED()
                            )
                    
                    nested_vals = await nested()
                    results.append((outer_val, inner_val, outer_val_inside, *nested_vals))
                
                # 退出内层后应该恢复外层
                outer_val_after = param_scope.OUTER()
                results.append((outer_val, outer_val_after))

        tasks = [worker(i) for i in range(num_tasks)]
        await asyncio.gather(*tasks)

        # 验证嵌套作用域的正确性
        assert len(results) == num_tasks * 2  # 每个任务产生2个结果
        for i in range(num_tasks):
            # 第一个结果：嵌套作用域内
            outer, inner, outer_inside, outer_nested, inner_nested, nested = results[i * 2]
            assert outer == i * 10, f"Task {i}: outer value mismatch"
            assert inner == i * 100, f"Task {i}: inner value mismatch"
            assert outer_inside == i * 10, f"Task {i}: outer value inside inner scope mismatch"
            assert outer_nested == i * 10, f"Task {i}: outer value in nested task mismatch"
            assert inner_nested == i * 100, f"Task {i}: inner value in nested task mismatch"
            assert nested == i * 1000, f"Task {i}: nested value mismatch"
            
            # 第二个结果：退出内层后
            outer, outer_after = results[i * 2 + 1]
            assert outer == outer_after == i * 10, f"Task {i}: outer value not restored after inner exit"

    def test_stress_mixed_thread_async_isolation(self):
        """测试线程间和异步任务间的完全隔离"""
        num_threads = 30
        tasks_per_thread = 100
        all_results: Dict[int, List[Tuple[int, int]]] = {}
        lock = threading.Lock()

        def thread_worker(thread_id: int):
            async def async_worker(task_id: int):
                # 每个任务设置自己的参数
                with param_scope(**{"ID": thread_id * 10000 + task_id}):
                    await asyncio.sleep(0.0001)
                    val = param_scope.ID()
                    return (task_id, val)

            async def run_all():
                tasks = [async_worker(i) for i in range(tasks_per_thread)]
                results = await asyncio.gather(*tasks)
                with lock:
                    all_results[thread_id] = results

            asyncio.run(run_all())

        threads = [threading.Thread(target=thread_worker, args=(i,)) for i in range(num_threads)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 验证隔离性
        assert len(all_results) == num_threads
        for thread_id, results in all_results.items():
            assert len(results) == tasks_per_thread
            for task_id, val in results:
                expected = thread_id * 10000 + task_id
                assert val == expected, f"Thread {thread_id} task {task_id}: expected {expected}, got {val}"

    @pytest.mark.asyncio
    async def test_stress_concurrent_nested_async(self):
        """测试并发嵌套异步任务的参数隔离"""
        num_outer_tasks = 100
        num_inner_tasks_per_outer = 20
        results: List[Tuple[int, int, int]] = []

        async def outer_worker(outer_id: int):
            with param_scope(**{"OUTER_ID": outer_id}):
                async def inner_worker(inner_id: int):
                    with param_scope(**{"INNER_ID": inner_id}):
                        await asyncio.sleep(0.001)
                        return (
                            param_scope.OUTER_ID(),
                            param_scope.INNER_ID()
                        )
                
                inner_tasks = [inner_worker(i) for i in range(num_inner_tasks_per_outer)]
                inner_results = await asyncio.gather(*inner_tasks)
                
                for inner_id, (outer_val, inner_val) in enumerate(inner_results):
                    assert outer_val == outer_id, f"Outer task {outer_id} inner {inner_id}: outer value mismatch"
                    assert inner_val == inner_id, f"Outer task {outer_id} inner {inner_id}: inner value mismatch"
                    results.append((outer_id, inner_id, outer_val, inner_val))

        outer_tasks = [outer_worker(i) for i in range(num_outer_tasks)]
        await asyncio.gather(*outer_tasks)

        assert len(results) == num_outer_tasks * num_inner_tasks_per_outer

    def test_stress_exception_recovery(self):
        """测试异常情况下的参数恢复"""
        num_threads = 20
        tasks_per_thread = 50
        thread_results: List[bool] = []
        lock = threading.Lock()

        def thread_worker(thread_id: int):
            async def async_worker(task_id: int):
                try:
                    with param_scope(**{"ID": thread_id * 1000 + task_id}):
                        val1 = param_scope.ID()
                        # 嵌套作用域
                        try:
                            with param_scope(**{"ID": task_id}):
                                val2 = param_scope.ID()
                                # 模拟异常
                                if task_id % 10 == 0:
                                    raise ValueError(f"Test exception for task {task_id}")
                        except ValueError:
                            val3 = param_scope.ID()
                            return val1 == val3
                        val3 = param_scope.ID()
                        return val1 == val3
                except Exception:
                    return False

            async def run_all():
                tasks = [async_worker(i) for i in range(tasks_per_thread)]
                results = await asyncio.gather(*tasks)
                with lock:
                    thread_results.extend(results)

            asyncio.run(run_all())

        threads = [threading.Thread(target=thread_worker, args=(i,)) for i in range(num_threads)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 所有任务都应该成功恢复
        assert len(thread_results) == num_threads * tasks_per_thread, f"Expected {num_threads * tasks_per_thread} results, got {len(thread_results)}"
        assert all(thread_results), "Some tasks failed to recover after exception"

    @pytest.mark.asyncio
    async def test_stress_rapid_scope_switching(self):
        """测试快速作用域切换的正确性"""
        num_tasks = 1000
        results: List[int] = []

        async def worker(task_id: int):
            # 快速切换多个作用域
            for i in range(10):
                with param_scope(**{"VALUE": task_id * 10 + i}):
                    await asyncio.sleep(0.0001)
                    val = param_scope.VALUE()
                    results.append(val)
                    # 验证值正确
                    assert val == task_id * 10 + i, f"Task {task_id} iteration {i}: value mismatch"

        tasks = [worker(i) for i in range(num_tasks)]
        await asyncio.gather(*tasks)

        assert len(results) == num_tasks * 10

    def test_stress_thread_pool_with_async(self):
        """测试线程池+异步的混合场景"""
        num_threads = 10
        tasks_per_thread = 200
        all_results: Set[int] = set()
        lock = threading.Lock()

        def thread_worker(thread_id: int):
            async def async_worker(task_id: int):
                with param_scope(**{"ID": thread_id * 10000 + task_id}):
                    await asyncio.sleep(0.0001)
                    return param_scope.ID()

            async def run_all():
                tasks = [async_worker(i) for i in range(tasks_per_thread)]
                results = await asyncio.gather(*tasks)
                with lock:
                    all_results.update(results)

            asyncio.run(run_all())

        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(thread_worker, i) for i in range(num_threads)]
            for future in futures:
                future.result()

        # 验证所有值都唯一且正确
        assert len(all_results) == num_threads * tasks_per_thread
        expected_values = {i * 10000 + j for i in range(num_threads) for j in range(tasks_per_thread)}
        assert all_results == expected_values

    # @pytest.mark.asyncio
    # async def test_stress_frozen_propagation_async(self):
    #     """测试frozen参数在异步环境下的传播"""
    #     # 设置全局frozen值
    #     with param_scope(**{"GLOBAL": 9999}):
    #         param_scope.frozen()
    #
    #     num_tasks = 500
    #     results: List[int] = []
    #
    #     async def worker(task_id: int):
    #         # 应该继承frozen的值
    #         global_val = param_scope.GLOBAL()
    #         with param_scope(**{"LOCAL": task_id}):
    #             local_val = param_scope.LOCAL()
    #             # 创建嵌套任务
    #             async def nested():
    #                 # 嵌套任务也应该看到frozen值
    #                 nested_global = param_scope.GLOBAL()
    #                 return nested_global
    #             
    #             nested_global = await nested()
    #             results.append((global_val, local_val, nested_global))
    #             return global_val == 9999 and nested_global == 9999
    #
    #     tasks = [worker(i) for i in range(num_tasks)]
    #     success_flags = await asyncio.gather(*tasks)
    #
    #     # 验证所有任务都看到了frozen值
    #     assert all(success_flags), "Some tasks didn't see frozen value"
    #     assert len(results) == num_tasks
    #     for global_val, local_val, nested_global in results:
    #         assert global_val == 9999, "Global frozen value not propagated"
    #         assert nested_global == 9999, "Global frozen value not propagated to nested tasks"

    @pytest.mark.asyncio
    async def test_stress_high_concurrency(self):
        """高并发压力测试：大量任务同时运行"""
        num_tasks = 2000
        start_time = time.time()
        results: List[Tuple[int, int]] = []

        async def worker(task_id: int):
            with param_scope(**{"ID": task_id}):
                # 模拟一些计算
                await asyncio.sleep(0.0001)
                val = param_scope.ID()
                results.append((task_id, val))
                return val

        # 分批创建任务以避免内存问题
        batch_size = 500
        for batch_start in range(0, num_tasks, batch_size):
            batch_end = min(batch_start + batch_size, num_tasks)
            batch_tasks = [worker(i) for i in range(batch_start, batch_end)]
            await asyncio.gather(*batch_tasks)

        elapsed = time.time() - start_time
        print(f"\n高并发测试完成: {num_tasks} 个任务，耗时 {elapsed:.2f} 秒")

        # 验证结果
        assert len(results) == num_tasks
        result_dict = dict(results)
        for i in range(num_tasks):
            assert result_dict[i] == i, f"Task {i} saw wrong value"

    def test_stress_long_running_threads(self):
        """长时间运行的线程测试"""
        num_threads = 10
        iterations_per_thread = 1000
        duration_seconds = 5
        thread_results: List[int] = []
        lock = threading.Lock()
        stop_flag = threading.Event()

        def thread_worker(thread_id: int):
            async def async_iteration(iteration: int):
                with param_scope(**{"THREAD_ID": thread_id, "ITER": iteration}):
                    await asyncio.sleep(0.001)
                    t_id = param_scope.THREAD_ID()
                    it = param_scope.ITER()
                    if t_id != thread_id or it != iteration:
                        with lock:
                            thread_results.append(-1)  # 错误标记
                        return False
                    return True

            async def run_loop():
                iteration = 0
                start_time = time.time()
                while not stop_flag.is_set() and (time.time() - start_time) < duration_seconds:
                    success = await async_iteration(iteration)
                    if not success:
                        break
                    iteration += 1
                    if iteration >= iterations_per_thread:
                        break
                with lock:
                    thread_results.append(iteration)

            asyncio.run(run_loop())

        threads = [threading.Thread(target=thread_worker, args=(i,)) for i in range(num_threads)]
        for t in threads:
            t.start()
        
        # 等待指定时间或所有线程完成
        time.sleep(duration_seconds)
        stop_flag.set()
        
        for t in threads:
            t.join(timeout=1)

        # 验证结果
        assert len(thread_results) == num_threads
        # 检查是否有错误
        assert -1 not in thread_results, "Some iterations failed"
        # 所有线程应该至少完成了一些迭代
        assert all(count > 0 for count in thread_results), "Some threads didn't complete any iterations"

    @pytest.mark.asyncio
    async def test_stress_extreme_concurrency(self):
        """极端并发压力测试：大量线程+大量异步任务"""
        num_threads = 50
        tasks_per_thread = 200
        all_correct = []
        lock = threading.Lock()

        def thread_worker(thread_id: int):
            async def async_worker(task_id: int):
                # 多层嵌套作用域
                with param_scope(**{"THREAD": thread_id}):
                    with param_scope(**{"TASK": task_id}):
                        with param_scope(**{"COMBINED": thread_id * 100000 + task_id}):
                            await asyncio.sleep(0.0001)
                            # 验证所有层级的值
                            t = param_scope.THREAD()
                            task = param_scope.TASK()
                            combined = param_scope.COMBINED()
                            
                            # 创建嵌套异步任务验证隔离
                            async def nested():
                                with param_scope(**{"NESTED": task_id * 1000}):
                                    await asyncio.sleep(0.0001)
                                    return (
                                        param_scope.THREAD(),
                                        param_scope.TASK(),
                                        param_scope.COMBINED(),
                                        param_scope.NESTED()
                                    )
                            
                            nested_vals = await nested()
                            
                            correct = (
                                t == thread_id and
                                task == task_id and
                                combined == thread_id * 100000 + task_id and
                                nested_vals[0] == thread_id and
                                nested_vals[1] == task_id and
                                nested_vals[2] == thread_id * 100000 + task_id and
                                nested_vals[3] == task_id * 1000
                            )
                            return correct

            async def run_all():
                tasks = [async_worker(i) for i in range(tasks_per_thread)]
                results = await asyncio.gather(*tasks)
                with lock:
                    all_correct.extend(results)

            asyncio.run(run_all())

        threads = [threading.Thread(target=thread_worker, args=(i,)) for i in range(num_threads)]
        start_time = time.time()
        
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        
        elapsed = time.time() - start_time
        print(f"\n极端并发测试完成: {num_threads} 线程 × {tasks_per_thread} 任务 = {num_threads * tasks_per_thread} 总任务，耗时 {elapsed:.2f} 秒")

        # 验证所有任务都正确
        assert len(all_correct) == num_threads * tasks_per_thread
        assert all(all_correct), f"有 {sum(1 for x in all_correct if not x)} 个任务失败"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

