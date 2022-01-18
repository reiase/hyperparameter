from setuptools import setup, find_packages
from setuptools.command.install import install


with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="hyperparameter",
    version="0.1.4",
    description="A hyper-parameter library for researchers, data scientists and machine learning engineers.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Reiase",
    author_email="reiase@gmail.com",
    url="https://github.com/reiase/hyperparameter",
    classifiers=[
        "Programming Language :: Python :: 3",
        "Operating System :: OS Independent",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
    ],
    packages=find_packages(),
    license="http://www.apache.org/licenses/LICENSE-2.0",
)
