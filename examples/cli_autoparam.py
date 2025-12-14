"""
Simple CLI starter using hyperparameter.param + run_cli().

Usage:
  # Default args
  python examples/cli_autoparam.py greet --name Alice --times 2
  python examples/cli_autoparam.py calc --a 3 --b 7

  # Mix CLI args with -D overrides
  python examples/cli_autoparam.py calc --a 4 -D calc.b=10

  # -D can also drive values used inside other commands (e.g., foo.value for greet)
  python examples/cli_autoparam.py greet -D foo.value=42

  # Thread-safe: run_cli freezes the scope so threads spawned inside see the overrides.
"""

import threading

import hyperparameter as hp


@hp.param("foo")
def _foo(value=1):
    return value

@hp.param("greet")
def greet(name: str = "world", times: int = 1, excited: bool = False):
    """Print greeting messages; internal foo.value is also override-able via -D foo.value=..."""
    msg = f"Hello, {name}. foo={_foo()}"
    if excited:
        msg += "!"
    for _ in range(int(times)):
        print(msg)
    return msg


@hp.param("calc")
def calc(a: int = 1, b: int = 2):
    """Tiny calculator that prints sum and product."""
    s = int(a) + int(b)
    p = int(a) * int(b)
    print(f"a={a}, b={b}, sum={s}, product={p}")
    return s, p


@hp.param("worker")
def spawn_child(task: str = "noop"):
    """Show that threads see CLI / -D overrides after run_cli freezes hp.scope."""

    def child():
        print(f"[child] task={hp.scope.worker.task()}")

    t = threading.Thread(target=child)
    t.start()
    t.join()
    return task


if __name__ == "__main__":
    run_cli()
