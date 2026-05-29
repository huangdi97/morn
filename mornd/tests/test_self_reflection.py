import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import EmotionState
from morn_core.consciousness.self_reflection import SelfReflection
from morn_core.memory.store import MemoryStore


@pytest.fixture
def emotion():
    return EmotionState()


@pytest.fixture
def mock_store():
    store = MagicMock()
    store.add_capsule = AsyncMock()
    store.add_capsule.return_value = "evt_self_reflection_test"
    return store


@pytest.fixture
def reflection(emotion, mock_store):
    return SelfReflection(
        memory_store=mock_store,
        emotion_state=emotion,
        instance_name="test_morn",
        light_interval=60,
        deep_interval=300,
    )


class TestInitialState:
    def test_initial_state(self, reflection):
        assert reflection._light_count == 0
        assert reflection._deep_count == 0
        assert len(reflection._emotion_history) == 0
        assert reflection._shutdown is False

    def test_initial_emotion_history_empty(self, reflection):
        assert reflection._emotion_history == []


class TestLightReflection:
    @pytest.mark.asyncio
    async def test_light_reflection_increments_count(self, reflection):
        await reflection.light_reflection()
        assert reflection._light_count == 1

    @pytest.mark.asyncio
    async def test_light_reflection_records_emotion(self, reflection):
        await reflection.light_reflection()
        assert len(reflection._emotion_history) == 1
        ts, calm, pleasure, conn = reflection._emotion_history[0]
        assert calm == 0.7
        assert pleasure == 0.5
        assert conn == 0.3

    @pytest.mark.asyncio
    async def test_light_reflection_does_not_save_every_time(self, reflection, mock_store):
        for _ in range(4):
            await reflection.light_reflection()
        assert mock_store.add_capsule.await_count == 0

    @pytest.mark.asyncio
    async def test_light_reflection_every_5th_saves(self, reflection, mock_store):
        for _ in range(5):
            await reflection.light_reflection()
        mock_store.add_capsule.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_light_reflection_10th_saves_twice(self, reflection, mock_store):
        for _ in range(10):
            await reflection.light_reflection()
        assert mock_store.add_capsule.await_count == 2

    @pytest.mark.asyncio
    async def test_light_reflection_snapshot_has_correct_source(self, reflection, mock_store):
        for _ in range(5):
            await reflection.light_reflection()
        capsule = mock_store.add_capsule.await_args[0][0]
        assert capsule["source"] == "self_reflection"

    @pytest.mark.asyncio
    async def test_light_reflection_snapshot_has_correct_entities(self, reflection, mock_store):
        for _ in range(5):
            await reflection.light_reflection()
        capsule = mock_store.add_capsule.await_args[0][0]
        assert capsule["entities"] == '["morn", "self_reflection"]'

    @pytest.mark.asyncio
    async def test_light_reflection_snapshot_has_emotion_tag(self, reflection, mock_store):
        for _ in range(5):
            await reflection.light_reflection()
        capsule = mock_store.add_capsule.await_args[0][0]
        assert capsule["emotion_tag"] == "自省"

    @pytest.mark.asyncio
    async def test_light_reflection_emotion_history_capped(self, reflection):
        for _ in range(150):
            await reflection.light_reflection()
        assert len(reflection._emotion_history) == 100


class TestDeepReflection:
    @pytest.mark.asyncio
    async def test_deep_reflection_saves_memory(self, reflection, mock_store):
        await reflection.deep_reflection()
        mock_store.add_capsule.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_deep_reflection_memory_has_correct_source(self, reflection, mock_store):
        await reflection.deep_reflection()
        capsule = mock_store.add_capsule.await_args[0][0]
        assert capsule["source"] == "self_reflection"

    @pytest.mark.asyncio
    async def test_deep_reflection_increments_count(self, reflection):
        await reflection.deep_reflection()
        assert reflection._deep_count == 1

    @pytest.mark.asyncio
    async def test_deep_reflection_importance_is_0_6(self, reflection, mock_store):
        await reflection.deep_reflection()
        capsule = mock_store.add_capsule.await_args[0][0]
        assert capsule["importance_weight"] == 0.6


class TestEmotionTrend:
    def test_analyze_emotion_trend_insufficient_data(self, reflection):
        result = reflection._analyze_emotion_trend()
        assert "数据不足" in result

    def test_analyze_emotion_trend_rising(self, reflection):
        reflection._emotion_history = [
            (1.0, 0.7, 0.3, 0.3),
            (2.0, 0.7, 0.4, 0.3),
            (3.0, 0.7, 0.5, 0.3),
            (4.0, 0.7, 0.6, 0.5),
        ]
        result = reflection._analyze_emotion_trend()
        assert "愉悦感上升" in result
        assert "联结感增强" in result

    def test_analyze_emotion_trend_falling(self, reflection):
        reflection._emotion_history = [
            (1.0, 0.7, 0.8, 0.7),
            (2.0, 0.7, 0.6, 0.6),
            (3.0, 0.7, 0.4, 0.5),
            (4.0, 0.7, 0.2, 0.3),
        ]
        result = reflection._analyze_emotion_trend()
        assert "愉悦感下降" in result
        assert "联结感减弱" in result

    def test_analyze_emotion_trend_stable(self, reflection):
        reflection._emotion_history = [
            (1.0, 0.7, 0.5, 0.3),
            (2.0, 0.7, 0.51, 0.3),
            (3.0, 0.7, 0.49, 0.3),
            (4.0, 0.7, 0.5, 0.3),
        ]
        result = reflection._analyze_emotion_trend()
        assert "愉悦感稳定" in result


class TestStop:
    @pytest.mark.asyncio
    async def test_stop_sets_shutdown_flag(self, reflection):
        reflection.stop()
        assert reflection._shutdown is True

    @pytest.mark.asyncio
    async def test_stop_halts_loop(self, reflection):
        reflection.stop()
        import asyncio
        task = asyncio.create_task(reflection.reflection_loop())
        await asyncio.sleep(0.05)
        assert task.done() or task.cancelled()


class TestStatusDescription:
    def test_status_description_format(self, reflection):
        desc = reflection._get_status_description()
        assert "平静" in desc
        assert "愉悦" in desc
        assert "联结" in desc
        assert "0.70" in desc or "0.7" in desc
        assert "已自省 0 次" in desc

    def test_status_description_reflects_count(self, reflection):
        reflection._light_count = 5
        desc = reflection._get_status_description()
        assert "已自省 5 次" in desc


@pytest.mark.asyncio
async def test_integration_with_memory_store():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir, enable_encryption=False) as store:
            emotion = EmotionState()
            sr = SelfReflection(
                memory_store=store,
                emotion_state=emotion,
                instance_name="test_integration",
                light_interval=60,
                deep_interval=300,
            )
            await sr.deep_reflection()
            capsules = await store.get_recent(limit=10)
            assert len(capsules) >= 1
            c = capsules[0]
            assert c["source"] == "self_reflection"
            assert c["emotion_tag"] == "自省"
            assert c["emotion_score"] == 0.5