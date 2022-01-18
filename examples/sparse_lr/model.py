import numpy as np
from sklearn.linear_model import LogisticRegression
import matplotlib.pyplot as plt

from hyperparameter import let, local_param, param_scope


@let(learning_rate=0.01, penalty='l1', C=0.01, tol=0.01)
def sparse_lr_plot(X, y):
    C = local_param('C')
    penalty = local_param('penalty')
    tol = local_param('tol')
    LR = LogisticRegression(C=C, penalty=penalty, tol=tol, solver='saga')

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
        str({
            'C': C,
            'penalty': penalty,
            'tol': tol,
            'sparsity': '%.2f%%' % (sparsity_LR)
        }))
    plt.show()