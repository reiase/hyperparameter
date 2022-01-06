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

print('parameter list: \n  {}'.format('\n  '.join(Tracker.all())))

# run the lr model with default parameters
coef = sparse_lr_train(X, y)

plt.imshow(
    np.abs(coef.reshape(8, 8)),
    interpolation="nearest",
    cmap="binary",
    vmax=1,
    vmin=0,
)
plt.show()

# run the lr model with another parameter
with param_scope('model.sparse_lr_train.C=0.1'):
    coef = sparse_lr_train(X, y)
    plt.imshow(
        np.abs(coef.reshape(8, 8)),
        interpolation="nearest",
        cmap="binary",
        vmax=1,
        vmin=0,
    )
    plt.show()