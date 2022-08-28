from hyperparameter import param_scope, set_tracker, Tracker
from model import sparse_lr_plot

from sklearn import datasets
from sklearn.preprocessing import StandardScaler

import mlflow


def mlflow_tracker(params):
    for k, v in params.items():
        mlflow.log_param(k, v)


set_tracker(mlflow_tracker)

X, y = datasets.load_digits(return_X_y=True)
X = StandardScaler().fit_transform(X)

# classify small against large digits
y = (y > 4).astype(int)


def run(args):
    # run the lr model with parameter from cmdline
    with param_scope(*args.define):  # set parameters according to cmd line
        sparse_lr_plot(X, y)


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser("example")
    parser.add_argument(
        "-D",
        "--define",
        nargs="*",
        default=[],
        action="extend",
        help="define a parameter `param_name=param_value`, supported parameter list: \n"
        + "\n  ".join(Tracker.all()),
    )
    args = parser.parse_args()
    run(args)

    print(Tracker.report())
