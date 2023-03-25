from setuptools import setup
from setuptools_rust import RustExtension, Binding

setup(
    rust_extensions=[
        RustExtension("hyperparameter.rbackend", binding=Binding.PyO3, debug=False)
    ],
)
