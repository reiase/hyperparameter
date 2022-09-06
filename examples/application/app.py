from hyperparameter import param_scope, auto_param


@auto_param
def main(a="default a", b="default b"):  # inline默认值
    print(f"a={a}, b={b}")
    with param_scope() as ps:
        print(f"params in main = {ps}")


if __name__ == "__main__":
    import argparse
    import json

    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=str, default=None)
    parser.add_argument("-D", "--define", nargs="*", default=[], action="extend")
    args = parser.parse_args()

    if args.config is not None:  # 加载配置文件
        with open(args.config) as f:
            cfg = json.load(f)
    else:
        cfg = {}

    with param_scope(**cfg):  # 配置文件的scope
        with param_scope(*args.define):  # 命令行参数的scope
            main()
