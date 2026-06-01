"""依恋系统 BondTracker 测试。"""

import os
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.emotion.bond_tracker import BondTracker


class TestBondInitial:
    def test_initial_bond_default(self):
        b = BondTracker({})
        assert b.get_bond() == 0.1

    def test_initial_bond_custom(self):
        b = BondTracker({"initial_bond": 0.5})
        assert b.get_bond() == 0.5

    def test_min_bond_default(self):
        b = BondTracker({})
        b._bond = -1.0
        b.update(0, 0, 0)
        assert b.get_bond() >= 0.0

    def test_max_bond_default(self):
        b = BondTracker({})
        b._bond = 2.0
        b.update(1.0, 1.0, 100)
        assert b.get_bond() <= 1.0


class TestBondUpdate:
    def test_update_growth(self):
        b = BondTracker({"growth_rate": 0.01})
        initial = b.get_bond()
        b.update(0.5, 0.7, 10)
        assert b.get_bond() > initial

    def test_update_decay_low_sentiment(self):
        b = BondTracker({"decay_rate": 0.002})
        b._bond = 0.3
        b.update(0, 0.1, 0)
        assert b.get_bond() < 0.3

    def test_update_clamp_to_min(self):
        b = BondTracker({"min_bond": 0.0, "decay_rate": 1.0})
        b._bond = 0.5
        b.update(0, 0, 0)
        assert b.get_bond() >= 0.0

    def test_update_clamp_to_max(self):
        b = BondTracker({"max_bond": 1.0, "growth_rate": 1.0})
        b._bond = 0.5
        b.update(1.0, 1.0, 1000)
        assert b.get_bond() <= 1.0


class TestBondStage:
    def test_stage_initial(self):
        b = BondTracker({"initial_bond": 0.1})
        assert b.get_stage() == "初识期"

    def test_stage_intimate(self):
        b = BondTracker({"initial_bond": 0.5})
        assert b.get_stage() == "亲近期"

    def test_stage_mature(self):
        b = BondTracker({"initial_bond": 0.8})
        assert b.get_stage() == "默契期"

    def test_stage_boundary_bottom(self):
        b = BondTracker({"initial_bond": 0.3})
        assert b.get_stage() == "亲近期"

    def test_stage_boundary_top(self):
        b = BondTracker({"initial_bond": 0.6999})
        assert b.get_stage() == "亲近期"

    def test_stage_mature_can_challenge(self):
        b = BondTracker({"initial_bond": 0.8})
        assert b.can_challenge() is True

    def test_stage_intimate_cannot_challenge(self):
        b = BondTracker({"initial_bond": 0.5})
        assert b.can_challenge() is False

    def test_stage_initial_cannot_challenge(self):
        b = BondTracker({"initial_bond": 0.1})
        assert b.can_challenge() is False


class TestBondPersistence:
    def test_save_and_load(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            b = BondTracker({"initial_bond": 0.3})
            b.set_data_dir(Path(tmpdir))
            b._bond = 0.75
            b.save()

            b2 = BondTracker({"initial_bond": 0.1})
            b2.set_data_dir(Path(tmpdir))
            b2.load()
            assert b2.get_bond() == 0.75

    def test_load_missing_file_uses_default(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            b = BondTracker({"initial_bond": 0.5})
            b.set_data_dir(Path(tmpdir))
            b.load()
            assert b.get_bond() == 0.5

    def test_load_corrupted_file_uses_default(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            person_dir = Path(tmpdir) / "personality"
            person_dir.mkdir(parents=True)
            with open(person_dir / "bond.json", "w") as f:
                f.write("not json")
            b = BondTracker({"initial_bond": 0.5})
            b.set_data_dir(Path(tmpdir))
            b.load()
            assert b.get_bond() == 0.5