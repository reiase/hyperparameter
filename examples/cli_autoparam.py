"""
Simple CLI starter using hyperparameter.auto_param + launch().

Usage:
  python examples/cli_autoparam.py greet --name Alice --times 2
  python examples/cli_autoparam.py calc --a 3 --b 7
  python examples/cli_autoparam.py greet -D greet.times=5  # override via -D
"""

from hyperparameter import auto_param, run_cli


@auto_param("greet")
def greet(name: str = "world", times: int = 1, excited: bool = False):
    """Print greeting messages.

    Args:
        name: Who to greet.
        times: How many times to repeat.
        excited: Add an exclamation mark if true.
    """
    msg = f"Hello, {name}"
    if excited:
        msg += "!"
    for _ in range(int(times)):
        print(msg)
    return msg


@auto_param("calc")
def calc(a: int = 1, b: int = 2):
    """Tiny calculator that prints sum and product.

    Args:
        a: First operand.
        b: Second operand.
    """
    s = int(a) + int(b)
    p = int(a) * int(b)
    print(f"a={a}, b={b}, sum={s}, product={p}")
    return s, p


if __name__ == "__main__":
    run_cli()
