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
    __main__.__dict__.clear()
    __main__.__dict__.update(
        {
            "__name__": "__main__",
            "__file__": filename,
            "__builtins__": builtins,
        }
    )

    import io

    with io.open_code(filename) as fp:
        statement = "exec(compile(%r, %r, 'exec'))" % (fp.read(), (filename))
    import linecache

    linecache.checkcache()
    exec(compile(statement, "<string>", "exec"), __main__.__dict__, __main__.__dict__)


def main():
    import getopt

    opts, args = getopt.getopt(sys.argv[1:], "ho:p:vc:", ["help", "option="])

    if not args:
        print(_usage)
        sys.exit(2)

    options = []
    cfgfile = ".hparams"
    profile = None
    verbose = False

    for opt, optarg in opts:
        if opt in ["-h", "--help"]:
            print(_usage)
            sys.exit()
        elif opt in ["-o", "--option"]:
            options.append(optarg)
        elif opt in ["-c", "--config"]:
            cfgfile = optarg
        elif opt in ["-p", "--profile"]:
            profile = optarg
        elif opt in ["-v", "--verbose"]:
            verbose = True

    sys.argv[:] = args
    mainpyfile = args[0]
    mainpyfile = os.path.realpath(mainpyfile)
    sys.path[0] = os.path.dirname(mainpyfile)

    rawcfg = {}
    if os.path.exists(cfgfile):
        import hyperparameter.loader

        rawcfg = hyperparameter.loader.load(cfgfile)
    else:
        cfgfile = None
    if profile is not None:
        if profile in rawcfg:
            cfg = rawcfg[profile]
        else:
            raise Exception(f"profile: {profile} not found in {cfgfile}")
    else:
        cfg = rawcfg
    if verbose:
        print(
            f"start with hyperparameters from config file: {cfgfile} with profile: {profile}"
        )
        print("==== start verbose ====")
        print(f"  parameters from config file:")
        print(f"  \t{hyperparameter.loader.dumps(rawcfg)}".replace("\n", "\n  \t"))
        print(f"  hyperparameter options: {options}")
        print(f"  command line args: {args}")

    with param_scope(**cfg):
        with param_scope(*options) as ps:
            if verbose:
                import json

                print("  hyperparameters in use:")
                config_inuse = json.loads(json.dumps(ps))
                print(
                    f"  \t{hyperparameter.loader.dumps(config_inuse)}".replace(
                        "\n", "\n  \t"
                    )
                )
                print("---- end verbose ----")
            runscript(mainpyfile)


if __name__ == "__main__":
    main()
