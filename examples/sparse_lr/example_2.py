from hyperparameter.hp import param_scope, Tracker
from model import sparse_lr_train
import numpy as np

from sklearn import datasets
from sklearn.preprocessing import StandardScaler
import matplotlib.pyplot as plt

X, y = datasets.load_digits(return_X_y=True)

X = StandardScaler().fit_transform(X)

# classify small against large digits
y = (y > 4).astype(int)


def run(args):
    # run the lr model with parameter from cmdline
    with param_scope(*args.define):  # set parameters according to cmd line
        coef = sparse_lr_train(X, y)
        plt.imshow(
            np.abs(coef.reshape(8, 8)),
            interpolation="nearest",
            cmap="binary",
            vmax=1,
            vmin=0,
        )
        plt.show()


if __name__ == '__main__':
    # create cmd line arguments parser
    import argparse
    parser = argparse.ArgumentParser('example')
    parser.add_argument(
        '-D',
        '--define',
        nargs='*',
        default=[],
        help=
        'define a parameter `param_name=param_value`, supported parameter list: \n\n  {}'
        .format('\n  '.join(Tracker.all())))
    args = parser.parse_args()
    run(args)

    print(Tracker.report())
