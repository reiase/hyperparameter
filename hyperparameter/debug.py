import code
import io
from contextlib import redirect_stderr, redirect_stdout
from types import CodeType
from typing import Any


def register_debug_command(name, cmd=None):
    if cmd is None:

        def wrapper(cls):
            register_debug_command(name, cls)
            return cls

        return wrapper
    DebugCommand.REGISTER[name] = cmd


class DebugCommand:
    REGISTER = {}

    def help(self):
        pass

    def __str__(self) -> str:
        return self()

    def __repr__(self) -> str:
        return str(self)

    def __call__(self, *args: Any, **kwds: Any) -> Any:
        pass


@register_debug_command("help")
class HelpCommand(DebugCommand):
    def help(self):
        ret = "list of commands:\n"
        for k, v in DebugCommand.REGISTER.items():
            h = v()
            ret += f"== {k} ==\n"
            if isinstance(h, HelpCommand):
                ret += "print this help"
            else:
                ret += h.help()
            ret += "\n\n"
        return ret

    def __call__(self, *args: Any, **kwds: Any) -> Any:
        return self.help()


@register_debug_command("bt")
class BackTrace(DebugCommand):
    def help(self):
        return "print python and C stack"

    def __call__(self, *args: Any, **kwds: Any) -> Any:
        import traceback

        from hyperparameter.librbackend import backtrace

        bt = backtrace()
        py = "".join(traceback.format_stack())
        return f"{bt}\n{py}"

    def __str__(self) -> str:
        return self()


@register_debug_command("params")
class ParamsCommand(DebugCommand):
    def help(self):
        return "list of parameters"

    def __call__(self) -> Any:
        import json

        from hyperparameter import param_scope

        params = param_scope().storage().storage()
        return json.dumps(params)

    def __str__(self) -> str:
        return self()


@register_debug_command("exit")
class ExitCommand(DebugCommand):
    def help(self):
        return "exit debug server"


class DebugConsole(code.InteractiveConsole):
    def init(self):
        for k, v in DebugCommand.REGISTER.items():
            self.locals[k] = v()

    def resetoutput(self):
        out = self.output
        self.output = ""
        return out

    def write(self, data: str) -> None:
        self.output += data

    def runsource(
        self, source: str, filename: str = "<input>", symbol: str = "single"
    ) -> bool:
        try:
            code = self.compile(source, filename, symbol)
        except (OverflowError, SyntaxError, ValueError):
            # Case 1: wrong code
            self.showsyntaxerror(filename)
            self.resetbuffer()
            return self.resetoutput()

        if code is None:
            # Case 2: incomplete code
            return

        ret = self.runcode(code)
        self.resetbuffer()
        return ret

    def runcode(self, code: CodeType) -> None:
        try:
            with redirect_stderr(io.StringIO()) as err:
                with redirect_stdout(io.StringIO()) as out:
                    exec(code, self.locals)
            ret = err.getvalue() + out.getvalue()
            if len(ret) == 0:
                return None
            return ret

        except SystemExit:
            raise
        except:
            self.showtraceback()
            return self.resetoutput()

    def push(self, line: str) -> bool:
        if not hasattr(self, "output"):
            self.output = ""
        self.buffer.append(line)
        source = "\n".join(self.buffer)
        return self.runsource(source, self.filename)
