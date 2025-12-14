from __future__ import annotations

import os
import sys
import textwrap
from textwrap import dedent

try:
    import hyperparameter as hp
except ModuleNotFoundError:
    repo_root = os.path.abspath(
        os.path.join(os.path.dirname(__file__), os.pardir, os.pardir)
    )
    if repo_root not in sys.path:
        sys.path.insert(0, repo_root)
    

@hp.param
def greet(name: str = "world", enthusiasm: int = 1) -> None:
    """Print a greeting; values can be overridden via CLI or hp.scope."""
    suffix = "!" * max(1, enthusiasm)
    print(f"hello, {name}{suffix}")


def demo() -> None:
    use_color = sys.stdout.isatty()
    green = "\033[92m" if use_color else ""
    cyan = "\033[96m" if use_color else ""
    yellow = "\033[93m" if use_color else ""
    reset = "\033[0m" if use_color else ""

    default_code = dedent(
        """
        greet()
        """
    ).strip()
    scoped_code = dedent(
        """
        with hp.scope(**{"greet.name": "scope-user", "greet.enthusiasm": 3}):
            greet()
        """
    ).strip()
    nested_code = dedent(
        """
        with hp.scope(**{"greet.name": "outer", "greet.enthusiasm": 2}):
            greet()  # outer scope values
            with hp.scope(**{"greet.name": "inner"}):
                greet()  # inner overrides name only; enthusiasm inherited
        """
    ).strip()
    cli_code = "python -m hyperparameter.examples.quickstart -D greet.name=Alice --enthusiasm=3"

    print(f"{yellow}=== Function definition ==={reset}")
    print(
        textwrap.indent(
            dedent(
                """
            @hp.param
            def greet(name: str = "world", enthusiasm: int = 1) -> None:
                suffix = "!" * max(1, enthusiasm)
                print(f"hello, {name}{suffix}")
            """
            ).strip(),
            prefix=f"{cyan}",
        )
        + "\n"
        + reset
    )

    print(f"{yellow}=== Quickstart: default values ==={reset}")
    print(f"{cyan}{default_code}{reset}")
    greet()

    print(f"\n{yellow}=== Quickstart: scoped override ==={reset}")
    print(f"{cyan}{scoped_code}{reset}")
    with hp.scope(**{"greet.name": "scope-user", "greet.enthusiasm": 3}):
        greet()

    print(f"\n{yellow}=== Quickstart: nested scopes ==={reset}")
    print(f"{cyan}{nested_code}{reset}")
    with hp.scope(**{"greet.name": "outer", "greet.enthusiasm": 2}):
        greet()
        with hp.scope(**{"greet.name": "inner"}):
            greet()

    print(f"\n{yellow}=== Quickstart: CLI override ==={reset}")
    print(f"{cyan}{cli_code}{reset}")
    print("Run this command separately to see CLI overrides in action.")


if __name__ == "__main__":
    # No args: run the quick demo. With args: expose the @hp.param CLI.
    if len(sys.argv) == 1:
        demo()
    else:
        hp.launch(greet)
