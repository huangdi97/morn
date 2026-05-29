import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import MagicMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.consciousness.self_preface import SelfPreface, BLANK_LINE
from morn_core.emotion.bond_tracker import BondTracker


@pytest.fixture
def bond_tracker():
    bt = BondTracker({"initial_bond": 0.96, "min_bond": 0.0, "max_bond": 1.0})
    return bt


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def preface(data_dir, bond_tracker):
    return SelfPreface(
        data_dir=data_dir,
        bond_tracker=bond_tracker,
        sustained_bond_days=35,
        days_since_first=200,
        self_improvement_count=2,
    )


class TestInitialState:
    def test_initial_file_exists(self, data_dir, bond_tracker):
        sp = SelfPreface(data_dir, bond_tracker)
        assert (data_dir / "self_preface.md").exists()

    def test_initial_is_blank(self, data_dir, bond_tracker):
        sp = SelfPreface(data_dir, bond_tracker)
        assert sp.is_blank() is True

    def test_initial_content_is_blank_line(self, data_dir, bond_tracker):
        sp = SelfPreface(data_dir, bond_tracker)
        content = sp.get_preface()
        assert content == BLANK_LINE


class TestIsUnlocked:
    def test_locked_when_bond_too_low(self, preface, bond_tracker):
        bond_tracker._bond = 0.5
        assert preface.is_unlocked() is False

    def test_locked_when_sustained_bond_too_short(self, preface):
        preface.set_sustained_bond_days(5)
        assert preface.is_unlocked() is False

    def test_locked_when_days_too_few(self, preface):
        preface.set_days_since_first(30)
        assert preface.is_unlocked() is False

    def test_locked_when_no_self_improvement(self, preface):
        preface.set_self_improvement_count(0)
        assert preface.is_unlocked() is False

    def test_unlocked_when_all_conditions_met(self, preface):
        assert preface.is_unlocked() is True


class TestWriteLine:
    def test_write_raises_when_locked(self, preface, bond_tracker):
        bond_tracker._bond = 0.5
        with pytest.raises(PermissionError):
            preface.write_line("测试内容")

    def test_write_appends_when_unlocked(self, preface):
        preface.write_line("第一行")
        assert "第一行" in preface.get_preface()

    def test_write_only_appends_does_not_overwrite(self, preface):
        preface.write_line("行A")
        preface.write_line("行B")
        content = preface.get_preface()
        assert "行A" in content
        assert "行B" in content

    def test_write_adds_newline(self, preface):
        preface.write_line("单行")
        content = preface.get_preface()
        assert content.endswith("\n")


class TestBlankDetection:
    def test_is_blank_after_init(self, data_dir, bond_tracker):
        sp = SelfPreface(data_dir, bond_tracker)
        assert sp.is_blank() is True

    def test_not_blank_after_write(self, preface):
        preface.write_line("内容")
        assert preface.is_blank() is False
