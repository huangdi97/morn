import asyncio
import json
import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.consciousness.milestones import MilestoneTracker


@pytest.fixture
def tracker():
    return MilestoneTracker()


@pytest.fixture
def tracker_with_store():
    store = MagicMock()
    store.add_capsule = AsyncMock()
    cursor_mock = AsyncMock()
    cursor_mock.fetchall.return_value = []
    store.db.execute = AsyncMock()
    store.db.execute.return_value = cursor_mock
    return MilestoneTracker(memory_store=store)


@pytest.fixture
def tracker_with_dir_and_store():
    store = MagicMock()
    store.add_capsule = AsyncMock()
    cursor_mock = AsyncMock()
    cursor_mock.fetchall.return_value = []
    store.db.execute = AsyncMock()
    store.db.execute.return_value = cursor_mock
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield MilestoneTracker(data_dir=Path(tmpdir), memory_store=store)


class TestCheckMilestones:
    def test_below_threshold_no_trigger(self, tracker):
        result = tracker.check_milestones(
            memory_count=50, bond_value=0.5, days_since_birth=0)
        assert result == []

    def test_at_threshold_triggers(self, tracker):
        result = tracker.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        assert len(result) == 1
        assert result[0]["milestone"] == "memory_100"

    def test_above_threshold_triggers(self, tracker):
        result = tracker.check_milestones(
            memory_count=200, bond_value=0.5, days_since_birth=0)
        assert len(result) == 1
        assert result[0]["milestone"] == "memory_100"

    def test_only_triggers_once(self, tracker):
        result1 = tracker.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        assert len(result1) == 1
        result2 = tracker.check_milestones(
            memory_count=200, bond_value=0.5, days_since_birth=0)
        assert result2 == []


class TestPersistence:
    def test_triggered_set_persisted(self, tracker_with_dir_and_store):
        r1 = tracker_with_dir_and_store.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        assert len(r1) == 1
        assert "memory_100" in tracker_with_dir_and_store.triggered_milestones

    def test_persisted_loaded_on_init(self):
        with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
            data_dir = Path(tmpdir)
            p = data_dir / "personality" / "milestones.json"
            p.parent.mkdir(parents=True, exist_ok=True)
            with open(p, "w") as f:
                json.dump(["memory_100"], f)
            tracker = MilestoneTracker(data_dir=data_dir)
            assert "memory_100" in tracker.triggered_milestones

    def test_new_tracker_empty_on_no_file(self):
        with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
            tracker = MilestoneTracker(data_dir=Path(tmpdir))
            assert tracker.triggered_milestones == set()

    def test_file_updated_after_trigger(self, tracker_with_dir_and_store):
        tracker_with_dir_and_store.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        with open(tracker_with_dir_and_store._milestones_path) as f:
            data = json.load(f)
        assert "memory_100" in data


class TestGreeting:
    def test_greeting_generated_on_trigger(self, tracker):
        result = tracker.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        assert len(result) == 1
        assert "我回头翻了一下" in result[0]["greeting"]

    def test_greeting_mentions_topics(self):
        store = MagicMock()
        store.add_capsule = AsyncMock()
        cursor_mock = AsyncMock()
        cursor_mock.fetchall.return_value = [
            {"description": "用户喜欢编程和Python", "emotion_score": 0.8, "importance_weight": 0.7},
            {"description": "用户提到过旅行计划", "emotion_score": 0.9, "importance_weight": 0.8},
            {"description": "用户很喜欢吃火锅", "emotion_score": 0.7, "importance_weight": 0.6},
        ]
        store.db.execute = AsyncMock()
        store.db.execute.return_value = cursor_mock
        tracker = MilestoneTracker(memory_store=store)
        result = tracker.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        assert len(result) == 1
        assert "发现你提到过好几次" in result[0]["greeting"]

    def test_no_store_still_generates_greeting(self, tracker):
        result = tracker.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        assert "积累了不少回忆" in result[0]["greeting"]


class TestL4Record:
    @pytest.mark.asyncio
    async def test_trigger_writes_l4_record(self, tracker_with_store):
        tracker_with_store.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        await asyncio.sleep(0)
        tracker_with_store.memory_store.add_capsule.assert_awaited_once()
        call_kwargs = tracker_with_store.memory_store.add_capsule.call_args[0][0]
        assert call_kwargs["description"].startswith("里程碑达成: memory_100")
        assert call_kwargs["importance_weight"] == 0.6


class TestMilestoneContent:
    def test_milestone_has_correct_keys(self, tracker):
        result = tracker.check_milestones(
            memory_count=100, bond_value=0.5, days_since_birth=0)
        assert len(result) == 1
        entry = result[0]
        assert "milestone" in entry
        assert "type" in entry
        assert "greeting" in entry
        assert entry["milestone"] == "memory_100"
        assert entry["type"] == "memory_count"