from hyperparameter.hp import param_scope, Tracker
from model import sparse_lr_plot
import numpy as np

from sklearn import datasets
from sklearn.preprocessing import StandardScaler

X, y = datasets.load_digits(return_X_y=True)

X = StandardScaler().fit_transform(X)

# classify small against large digits
y = (y > 4).astype(int)

print("parameter list: \n  {}".format("\n  ".join(Tracker.all())))

# run the lr model with default parameters
sparse_lr_plot(X, y)


# run the lr model with another parameter
with param_scope("model.sparse_lr_train.C=0.1"):
    sparse_lr_plot(X, y)
