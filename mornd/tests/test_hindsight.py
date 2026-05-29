import json
import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.consciousness.hindsight import HindsightEngine


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir)


def make_emotion_state(pleasure=0.5, calmness=0.7, connection=0.3):
    state = MagicMock()
    state.pleasure = pleasure
    state.describe_state.return_value = f"pleasure={pleasure:.1f}"
    return state


@pytest.mark.asyncio
async def test_add_hindsight_mark(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.8,
            "emotion_tag": "高兴",
            "description": "test memory",
        })
        ok = await store.add_hindsight_mark(eid, "怀念", 0.3, "test context")
        assert ok is True
        marks = await store.get_hindsight_marks(eid)
        assert len(marks) == 1
        assert marks[0]["tag"] == "怀念"
        assert marks[0]["emotion_score"] == 0.3
        assert marks[0]["trigger_context"] == "test context"


@pytest.mark.asyncio
async def test_hindsight_marks_not_override_original(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.8,
            "emotion_tag": "高兴",
            "description": "test memory",
        })
        await store.add_hindsight_mark(eid, "怀念", 0.3, "ctx")
        cap = await store.get_capsule(eid)
        assert cap["emotion_tag"] == "高兴"
        assert cap["emotion_score"] == 0.8


@pytest.mark.asyncio
async def test_get_eligible_for_hindsight(data_dir):
    async with MemoryStore(data_dir) as store:
        eid1 = await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.8,
            "emotion_tag": "高兴",
            "description": "old memory",
            "timestamp": "2020-01-01T00:00:00.000000Z",
        })
        await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.4,
            "emotion_tag": "低落",
            "description": "low emotion",
            "timestamp": "2020-01-01T00:00:00.000000Z",
        })
        await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.8,
            "emotion_tag": "高兴",
            "description": "recent memory",
            "timestamp": "2099-01-01T00:00:00.000000Z",
        })
        eligible = await store.get_eligible_for_hindsight(threshold_days=30, min_emotion=0.5)
        ids = [e["id"] for e in eligible]
        assert eid1 in ids
        assert len(eligible) >= 1


@pytest.mark.asyncio
async def test_eligible_excludes_marked(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.8,
            "emotion_tag": "高兴",
            "description": "already marked",
            "timestamp": "2020-01-01T00:00:00.000000Z",
        })
        await store.add_hindsight_mark(eid, "怀念", 0.3, "ctx")
        eligible = await store.get_eligible_for_hindsight(30, 0.5)
        assert eid not in [e["id"] for e in eligible]


@pytest.mark.asyncio
async def test_hindsight_engine_scan(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.8,
            "emotion_tag": "高兴",
            "description": "old high emotion",
            "timestamp": "2020-01-01T00:00:00.000000Z",
        })
        engine = HindsightEngine(memory_store=store, config={"hindsight_enabled": True})
        emotion = make_emotion_state(pleasure=0.2)
        triggered = await engine.scan_and_apply(emotion)
        assert len(triggered) == 1
        t = triggered[0]
        assert t["original_score"] == 0.8
        assert t["new_score"] == 0.2
        assert t["new_tag"] in ("遗憾", "心疼过去的自己", "感伤", "理解")


@pytest.mark.asyncio
async def test_hindsight_no_diff(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": '["test"]',
            "emotion_score": 0.5,
            "emotion_tag": "平静",
            "description": "similar emotion",
            "timestamp": "2020-01-01T00:00:00.000000Z",
        })
        engine = HindsightEngine(memory_store=store, config={"hindsight_enabled": True})
        emotion = make_emotion_state(pleasure=0.6)
        triggered = await engine.scan_and_apply(emotion)
        assert len(triggered) == 0


@pytest.mark.asyncio
async def test_hindsight_db_column(data_dir):
    async with MemoryStore(data_dir) as store:
        cursor = await store.db.execute("PRAGMA table_info(capsules)")
        columns = {row["name"] for row in await cursor.fetchall()}
        assert "hindsight_marks" in columns
