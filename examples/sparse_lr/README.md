Sparse LR Examples
==================

This example is based on `scikit-learn` example: [l1 penalty and sparsity in logistic regression](https://scikit-learn.org/stable/auto_examples/linear_model/plot_logistic_l1_l2_sparsity.html#sphx-glr-auto-examples-linear-model-plot-logistic-l1-l2-sparsity-py), which classifies 8x8 images of digits into two classes: 0-4 against 5-9, 
and visualize the coefficients of the model for different penalty methods(l1 or l2) and C.

The algorithm is defined in function `sparse_lr_plot` from `model.py`. We use the decorator `auto_param`  to declare hyper-parameters for our function:
``` python
@auto_param
def sparse_lr_plot(X, y, learning_rate=0.01, penalty='l1', C=0.01, tol=0.01):
    print({'C': C, 'penalty': penalty, 'tol': tol})
    ...
```

Four keyword arguments are defined for `sparse_lr_plot`: `learning_rate`, `penalty`, `C` and `tol`. `auto_param` will convert these arguments into hyper-parameters.

There are two ways to control the hyper-parameters:
1. parameter scope (see detail in `example_1.py`):

``` python 
with param_scope('model.sparse_lr_train.C=0.1'):
    sparse_lr_plot(X, y)
```

2. command line arguments (see detail in `example_2.py`):

``` python
def run(args):
    # run the lr model with parameter from cmdline
    with param_scope(*args.define):  # set parameters according to cmd line
        sparse_lr_plot(X, y)
        ...


if __name__ == '__main__':
    # create cmd line arguments parser
    import argparse
    parser = argparse.ArgumentParser('example')
    parser.add_argument('-D', '--define', nargs='*', default=[])
    args = parser.parse_args()

    run(args)

```
