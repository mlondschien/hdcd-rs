import pytest
from pathlib import Path
import numpy as np
from hdcdpython import hdcd

_IRIS_FILE = "iris.csv"
_IRIS_PATH = Path(__file__).resolve().parents[1] / "testdata" / _IRIS_FILE

@pytest.fixture()
def iris_dataset():
    return np.loadtxt("../testdata/iris.csv", skiprows=1, delimiter=",", usecols=(0, 1, 2, 3))

def test_hdcd(iris_dataset):
    result = hdcd(iris_dataset)
    np.testing.assert_array_equal(result, [50, 100])
