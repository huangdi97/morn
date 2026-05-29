import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def db_path():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir) / "test.db"
