import os
import sys

import pytest

ROOT_DIR = os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
)

sys.path.append(os.path.join(ROOT_DIR, "python", "tests"))

from testing_utils import run_polibs


@pytest.fixture
def runner():
    return type("Polibs", (), {"run": run_polibs})


@pytest.fixture(autouse=True)
def tests_dir():
    return os.path.join(ROOT_DIR, "rust", "tests-data")


@pytest.fixture(autouse=True)
def output_dir():
    return os.path.join(ROOT_DIR, "python", "tests", "output")
