# Classifier based non-parametric change point detection

Change point detection tries to identify times when the probability distribution of a
stochastic process or time series changes. Existing methods either assume a parametric
model for within-segment distributions or a based on ranks or distances, and thus fail
in scenarios with reasonably large dimensionality.

`changeforest` implements a classifier based algorithm that consistently estimates
change points without any parametric assumptions even in high-dimensional scenarios.
See [1] for details.

## Python

`changeforest` is available on [`PyPI`](https://pypi.org/project/changeforest/) and
[`conda-forge`](https://anaconda.org/conda-forge/changeforest). To install from
`conda-forge` (recommended), simply run
```bash
conda install -c conda-forge changeforest
```

The following example performs random forest based change point detection on the iris
dataset. This includes three classes _setosa_, _versicolor_ and _virginica_ with 50
observations each. We interpret this as a simulated time series with change points at
`t = 50, 100`.

```python
In [1]: from changeforest import changeforest
   ...: from sklearn.datasets import fetch_openml
   ...:
   ...: iris = fetch_openml(data_id=61)["frame"].drop(columns="class").to_numpy()
   ...: result = changeforest(iris, "random_forest", "bs")
   ...: result
Out[1]:
                    best_split max_gain p_value
(0, 150]                    50   96.212    0.01
 ¦--(0, 50]                 34    -4.65       1
 °--(50, 150]              100   51.557    0.01
     ¦--(50, 100]           80   -3.068       1
     °--(100, 150]         134   -2.063       1

In [2]: result.split_points()
Out[2]: [50, 100]
```

`changeforest` also implements methods `change_in_mean` and `knn`. While `random_forest`
and `knn` implement the `TwoStepSearch` optimizer as described in [1], for
`change_in_mean` the optimizer `GridSearch` is used. Both `random_forest` and `knn`
perform model selection via a pseudo-permutation test (see [1]). For `change_in_mean`
split candidates are kept whenever `max_gain > control.minimal_gain_to_split`.

The iris dataset
allows for rather simple classification due to large mean shifts between classes. As a
result, both `change_in_mean` and `knn` also correctly identify die true change points.

```python
In [3]: result = changeforest(iris, "change_in_mean", "bs")
   ...: result.split_points()
Out[3]: [50, 100]

In [4]: result = changeforest(iris, "knn", "bs")
   ...: result.split_points()
Out[4]: [50, 100]
```

`changeforest` returns a tree-like object with attributes `start`, `stop`, `best_split`, `max_gain`, `p_value`, `is_significant`, `optimizer_result`, `model_selection_result`, `left`, `right` and `segments`. These can be interesting to further investigate the output of the algorithm. Here we
plot the approximated gain curves of the first three segments:
```python
In [5]: import matplotlib.pyplot as plt
   ...: result = changeforest(iris, "change_in_mean", "bs")
   ...: plt.plot(range(150), result.optimizer_result.gain_results[-1].gain)
   ...: plt.plot(range(50), result.left.optimizer_result.gain_results[-1].gain)
   ...: plt.plot(range(50, 150), result.right.optimizer_result.gain_results[-1].gain)
   ...: plt.legend([f"approx. gain for {x}" for x in ["(0, 150]", "(0, 50]", "(50, 150"]])
   ...: plt.show()
```
<p align="center">
  <img src="docs/iris-approx-gains.png" />
</p>

One can clearly observe that the approx. gain curves are piecewise linear, with maxima
at the true underlying change points.

## References

[1] M. Londschien, S. Kovács and P. Bühlmann (2021), "Random Forests and other nonparametric classifiers for multivariate change point detection", working paper.