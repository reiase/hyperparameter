import numpy as np
from sklearn.linear_model import LogisticRegression
import matplotlib.pyplot as plt

from hyperparameter import auto_param, param_scope, set_tracker

MyLogisticRegression = auto_param(LogisticRegression)


@auto_param
def sparse_lr_plot(X, y, learning_rate=0.01, penalty="l1", C=0.01, tol=0.01):
    LR = MyLogisticRegression(C=C, penalty=penalty, tol=tol, solver="saga")

    LR.fit(X, y)
    coef = LR.coef_.ravel()
    sparsity_LR = np.mean(coef == 0) * 100
    print()
    plt.imshow(
        np.abs(coef.reshape(8, 8)),
        interpolation="nearest",
        cmap="binary",
        vmax=1,
        vmin=0,
    )
    plt.title(
        str(
            {
                "C": C,
                "penalty": penalty,
                "tol": tol,
                "sparsity": "%.2f%%" % (sparsity_LR),
            }
        )
    )
    plt.show()
