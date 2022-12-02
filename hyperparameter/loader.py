import warnings


def load(path: str):
    try:
        import toml
    except Exception as e:
        warnings.warn(
            "package toml is required by hyperparameter, please install toml with `pip install toml`"
        )
        raise e
    with open(path) as f:
        return toml.load(f)


def loads(config: str):
    try:
        import toml
    except Exception as e:
        warnings.warn(
            "package toml is required by hyperparameter, please install toml with `pip install toml`"
        )
        raise e
    return toml.loads(config)

def dumps(config) -> str:
    try:
        import toml
    except Exception as e:
        warnings.warn(
            "package toml is required by hyperparameter, please install toml with `pip install toml`"
        )
        raise e
    return toml.dumps(config)