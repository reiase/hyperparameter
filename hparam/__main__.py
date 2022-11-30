import io
import os
import sys
from hyperparameter import param_scope


_usage = """\
usage: python -m hparam [-o option] [-m module | pyfile] [arg] ...

Examples:

    python -m hparam -o opt1=1 -o opt2=b myscript.py arg1 arg2
"""



def runscript(filename):
    # The script has to run in __main__ namespace (or imports from
    # __main__ will break).
    #
    # So we clear up the __main__ and set several special variables
    # (this gets rid of pdb's globals and cleans old variables on restarts).
    import __main__
    # import ipdb; ipdb.set_trace()

    def canonic(filename):
        """Return canonical form of filename.

        For real filenames, the canonical form is a case-normalized (on
        case insensitive filesystems) absolute path.  'Filenames' with
        angle brackets, such as "<stdin>", generated in interactive
        mode, are returned unchanged.
        """
        if filename == "<" + filename[1:-1] + ">":
            return filename
        import os
        canonic = os.path.abspath(filename)
        canonic = os.path.normcase(canonic)
        return canonic

    builtins = __builtins__
    # __main__.__dict__.clear()
    __main__.__dict__.update(
        {
            "__name__": "__main__",
            "__file__": filename,
            "__builtins__": builtins,
        }
    )

    import io
    with io.open_code(filename) as fp:
        statement = "exec(compile(%r, %r, 'exec'))" % (fp.read(), canonic(filename))
    
    exec(statement)


def main():
    import getopt

    opts, args = getopt.getopt(sys.argv[1:], "ho:", ["help", "option="])

    if not args:
        print(_usage)
        sys.exit(2)

    options = []

    for opt, optarg in opts:
        if opt in ["-h", "--help"]:
            print(_usage)
            sys.exit()
        elif opt in ["-o", "--option"]:
            options.append(optarg)

    sys.argv[:] = args
    mainpyfile = args[0]
    mainpyfile = os.path.realpath(mainpyfile)
    sys.path[0] = os.path.dirname(mainpyfile)
    print(f"start with hyperparameters: {options}, and args: {args}")
    with param_scope(*options):
        runscript(mainpyfile)
    


if __name__ == "__main__":
    main()
