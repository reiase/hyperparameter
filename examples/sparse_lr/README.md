Sparse LR Examples
==================

This example shows l1 penalty and sparsity in logistic regression, based on `scikit-learn` example from https://scikit-learn.org/stable/auto_examples/linear_model/plot_logistic_l1_l2_sparsity.html#sphx-glr-auto-examples-linear-model-plot-logistic-l1-l2-sparsity-py.

`sparse_lr_train` (from `model.py`) classifies 8x8 images of digits into two classes: 0-4 against 5-9, 
and visualize the coefficients of the model for different penalty methods(l1 or l2) and C.

We use the `let` decorator to declare hyper-parameters for our algorithm:
``` python
@let(learning_rate=0.01, penalty='l1', C=0.01, tol=0.01)
def sparse_lr_train(X, y):
    C = local_param('C')
    penalty = local_param('penalty')
    tol = local_param('tol')
    print({'C': C, 'penalty': penalty, 'tol': tol})
    ...
```

Four hyper-parameter are defined for function `sparse_lr_train`: `learning_rate`, `penalty`, `C` and `tol`. 
There are two ways to control the hyper-parameters:
1. parameter scope (see detail in `example_1.py`):

``` python 
with param_scope('model.sparse_lr_train.C=0.1'):
    sparse_lr_train(X, y)
```

2. command line arguments (see detail in `example_2.py`):

``` python
def run(args):
    # run the lr model with parameter from cmdline
    with param_scope(*args.define):  # set parameters according to cmd line
        sparse_lr_train(X, y)
        ...


if __name__ == '__main__':
    # create cmd line arguments parser
    import argparse
    parser = argparse.ArgumentParser('example')
    parser.add_argument('-D', '--define', nargs='*', default=[])
    args = parser.parse_args()

    run(args)

```
