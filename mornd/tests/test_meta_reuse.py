import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import EmotionState
from morn_core.consciousness.self_reflection import SelfReflection


@pytest.fixture
def emotion():
    return EmotionState()


@pytest.fixture
def mock_store():
    store = MagicMock()
    store.add_capsule = AsyncMock()
    store.add_capsule.return_value = "evt_test"
    return store


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def reflection_with_reuse(emotion, mock_store, data_dir):
    sr = SelfReflection(
        memory_store=mock_store,
        emotion_state=emotion,
        instance_name="test_reuse",
        light_interval=60,
        deep_interval=300,
        data_dir=data_dir,
        enable_meta_reuse=True,
    )
    return sr


class TestInitialState:
    def test_reuse_count_starts_at_zero(self, reflection_with_reuse):
        assert reflection_with_reuse._reuse_count == 0

    def test_total_reflection_count_starts_at_zero(self, reflection_with_reuse):
        assert reflection_with_reuse._total_reflection_count == 0

    def test_behavior_manual_is_empty(self, reflection_with_reuse):
        assert reflection_with_reuse._behavior_manual == []

    def test_reuse_rate_is_zero(self, reflection_with_reuse):
        assert reflection_with_reuse.get_reuse_rate() == 0.0


class TestReuseMatching:
    @pytest.mark.asyncio
    async def test_reuse_pattern_returns_none_when_empty(self, reflection_with_reuse):
        result = reflection_with_reuse._reuse_pattern("light")
        assert result is None

    @pytest.mark.asyncio
    async def test_reuse_pattern_returns_entry_on_match(self, reflection_with_reuse):
        reflection_with_reuse._behavior_manual.append({
            "cycle_type": "light",
            "pattern": "high_high_high",
            "entry": "标准条目",
            "count": 0,
            "created_at": "2024-01-01T00:00:00Z",
        })
        reflection_with_reuse.emotion.calmness = 0.7
        reflection_with_reuse.emotion.pleasure = 0.7
        reflection_with_reuse.emotion.connection = 0.7
        result = reflection_with_reuse._reuse_pattern("light")
        assert result == "标准条目"

    @pytest.mark.asyncio
    async def test_reuse_pattern_returns_none_on_mismatch(self, reflection_with_reuse):
        reflection_with_reuse._behavior_manual.append({
            "cycle_type": "light",
            "pattern": "high_high_high",
            "entry": "标准条目",
            "count": 0,
            "created_at": "2024-01-01T00:00:00Z",
        })
        reflection_with_reuse.emotion.calmness = 0.3
        reflection_with_reuse.emotion.pleasure = 0.2
        reflection_with_reuse.emotion.connection = 0.3
        result = reflection_with_reuse._reuse_pattern("light")
        assert result is None

    @pytest.mark.asyncio
    async def test_reuse_increments_count_on_match(self, reflection_with_reuse):
        reflection_with_reuse._behavior_manual.append({
            "cycle_type": "light",
            "pattern": "high_high_high",
            "entry": "标准条目",
            "count": 0,
            "created_at": "2024-01-01T00:00:00Z",
        })
        reflection_with_reuse.emotion.calmness = 0.7
        reflection_with_reuse.emotion.pleasure = 0.7
        reflection_with_reuse.emotion.connection = 0.7
        reflection_with_reuse._reuse_pattern("light")
        assert reflection_with_reuse._reuse_count == 1


class TestNoMatchCallsAddEntry:
    @pytest.mark.asyncio
    async def test_light_reflection_adds_entry(self, reflection_with_reuse, mock_store):
        await reflection_with_reuse.light_reflection()
        assert len(reflection_with_reuse._behavior_manual) == 1
        assert reflection_with_reuse._behavior_manual[0]["cycle_type"] == "light"

    @pytest.mark.asyncio
    async def test_deep_reflection_adds_entry(self, reflection_with_reuse, mock_store):
        await reflection_with_reuse.deep_reflection()
        assert len(reflection_with_reuse._behavior_manual) == 1
        assert reflection_with_reuse._behavior_manual[0]["cycle_type"] == "deep"

    @pytest.mark.asyncio
    async def test_light_reflection_skips_save_on_reuse(self, reflection_with_reuse, mock_store):
        reflection_with_reuse._behavior_manual.append({
            "cycle_type": "light",
            "pattern": "high_high_low",
            "entry": "复用条目",
            "count": 0,
            "created_at": "2024-01-01T00:00:00Z",
        })
        reflection_with_reuse.emotion.calmness = 0.7
        reflection_with_reuse.emotion.pleasure = 0.7
        reflection_with_reuse.emotion.connection = 0.3
        for _ in range(5):
            await reflection_with_reuse.light_reflection()
        assert mock_store.add_capsule.await_count == 1


class TestReuseRate:
    @pytest.mark.asyncio
    async def test_reuse_rate_tracks_correctly(self, reflection_with_reuse, mock_store):
        reflection_with_reuse._behavior_manual.append({
            "cycle_type": "light",
            "pattern": "high_high_low",
            "entry": "复用条目",
            "count": 0,
            "created_at": "2024-01-01T00:00:00Z",
        })
        reflection_with_reuse.emotion.calmness = 0.7
        reflection_with_reuse.emotion.pleasure = 0.7
        reflection_with_reuse.emotion.connection = 0.3
        for _ in range(3):
            await reflection_with_reuse.light_reflection()
        rate = reflection_with_reuse.get_reuse_rate()
        assert rate == 3 / 3


class TestDisabled:
    @pytest.mark.asyncio
    async def test_reuse_disabled_returns_none(self, emotion, mock_store, data_dir):
        sr = SelfReflection(
            memory_store=mock_store,
            emotion_state=emotion,
            instance_name="test_disabled",
            light_interval=60,
            deep_interval=300,
            data_dir=data_dir,
            enable_meta_reuse=False,
        )
        sr._behavior_manual.append({
            "cycle_type": "light",
            "pattern": "high_high_high",
            "entry": "条目",
            "count": 0,
            "created_at": "2024-01-01T00:00:00Z",
        })
        sr.emotion.calmness = 0.7
        sr.emotion.pleasure = 0.7
        sr.emotion.connection = 0.7
        result = sr._reuse_pattern("light")
        assert result is None

    @pytest.mark.asyncio
    async def test_reuse_disabled_no_entry_added(self, emotion, mock_store, data_dir):
        sr = SelfReflection(
            memory_store=mock_store,
            emotion_state=emotion,
            instance_name="test_disabled",
            light_interval=60,
            deep_interval=300,
            data_dir=data_dir,
            enable_meta_reuse=False,
        )
        await sr.light_reflection()
        assert len(sr._behavior_manual) == 0


class TestGetBehaviorManual:
    def test_returns_copy_of_manual(self, reflection_with_reuse):
        assert reflection_with_reuse.get_behavior_manual() == []

    def test_returns_entries(self, reflection_with_reuse):
        reflection_with_reuse._behavior_manual.append({"test": "data"})
        manual = reflection_with_reuse.get_behavior_manual()
        assert len(manual) == 1
        assert manual[0]["test"] == "data"
