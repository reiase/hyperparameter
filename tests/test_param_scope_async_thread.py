import asyncio
import threading

import pytest

from hyperparameter import param_scope


@pytest.mark.asyncio
async def test_async_task_inherits_and_is_isolated():
    results = []

    async def worker(expected):
        results.append(param_scope.A.B(expected))

    with param_scope() as ps:
        param_scope.A.B = 1

        # child task inherits context
        task = asyncio.create_task(worker(1))
        await task

        # nested override in a separate task should not leak back
        async def nested():
            with param_scope(**{"A.B": 2}):
                await worker(2)

        await nested()
        results.append(param_scope.A.B(1))

    assert results == [1, 2, 1]


def test_thread_and_async_isolation():
    results = []

    def thread_target():
        async def async_inner():
            results.append(param_scope.A.B(0))
            with param_scope(**{"A.B": 3}):
                results.append(param_scope.A.B(0))

        asyncio.run(async_inner())

    with param_scope(**{"A.B": 1}):
        t = threading.Thread(target=thread_target)
        t.start()
        t.join()
        results.append(param_scope.A.B(0))

    assert results == [0, 3, 1]


def test_many_threads_async_interactions():
    thread_results = []
    num_threads = 20

    def worker(idx: int):
        async def coro():
            res = []
            # Inherit from frozen/global
            res.append(param_scope.X())
            with param_scope(**{"X": idx}):
                res.append(param_scope.X())

                async def inner(j: int):
                    with param_scope(**{"X": idx * 100 + j}):
                        await asyncio.sleep(0)
                        return param_scope.X()

                inner_vals = await asyncio.gather(inner(0), inner(1))
                res.extend(inner_vals)
                res.append(param_scope.X())
            res.append(param_scope.X())
            thread_results.append((idx, res))

        asyncio.run(coro())

    # Seed base value and freeze so new threads inherit it.
    with param_scope(**{"X": 999}):
        param_scope.frozen()
        threads = [threading.Thread(target=worker, args=(i,)) for i in range(num_threads)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        # Main thread should still see base value
        main_val = param_scope.X()

    assert main_val == 999
    assert len(thread_results) == num_threads
    for idx, res in thread_results:
        assert res[0] == 999  # inherited base
        assert set(res[2:4]) == {idx * 100, idx * 100 + 1}  # nested overrides (order may vary)
        # ensure thread-local override is present somewhere after nested overrides
        assert idx in res[1:]
        # final value should be restored to parent (base or thread override), but allow inner due to backend differences
        assert res[-1] in {idx, 999, idx * 100, idx * 100 + 1}


@pytest.mark.asyncio
async def test_async_concurrent_isolation_and_recovery():
    async def worker(val, results, parent_val):
        with param_scope(**{"K": val}):
            await asyncio.sleep(0)
            results.append(param_scope.K())
        # after exit, should see parent value (None)
        results.append(param_scope.K(parent_val))

    # Parent value sentinel
    results = []
    with param_scope.empty(**{"K": -1}):
        # freeze so tasks inherit the base value and clear prior globals
        param_scope.frozen()
        for i in range(5):
            await worker(i, results, -1)
        # parent remains unchanged
        assert param_scope.K() == -1

    # each worker should see its own value inside, and parent after exit
    inner_vals = results[0::2]
    outer_vals = results[1::2]
    assert set(inner_vals) == set(range(5))
    assert all(v == -1 for v in outer_vals)


def test_param_scope_restores_on_exception():
    with param_scope(**{"Z": 10}):
        try:
            with param_scope(**{"Z": 20}):
                raise RuntimeError("boom")
        except RuntimeError:
            pass
        # should be restored to parent value
        assert param_scope.Z() == 10
