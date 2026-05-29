import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.emotion.bond_tracker import BondTracker
from morn.contrib.security_advanced.apz_store import APZStore
from morn_core.evolution.audit import EvolutionLogger


@pytest.mark.asyncio
async def test_l2_l4_restart_persistence():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)

        async with MemoryStore(data_dir) as store:
            eid1 = await store.add_capsule({
                "entities": '["用户"]',
                "description": "用户说今天心情很好",
                "source": "chat",
            })
            eid2 = await store.add_capsule({
                "entities": '["用户"]',
                "description": "用户提到喜欢编程",
                "source": "chat",
            })
            await store.add_knowledge("用户", "喜欢", "编程", confidence=0.8)

        async with MemoryStore(data_dir) as store2:
            cap1 = await store2.get_capsule(eid1)
            assert cap1 is not None
            assert "心情很好" in cap1["description"]

            cap2 = await store2.get_capsule(eid2)
            assert cap2 is not None
            assert "喜欢编程" in cap2["description"]

            knowledge = await store2.query_knowledge(subject="用户")
            assert len(knowledge) >= 1
            assert any(k["object"] == "编程" for k in knowledge)

            count = await store2.count()
            assert count >= 2


@pytest.mark.asyncio
async def test_emotion_snapshot_restore():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)

        tracker1 = BondTracker({"initial_bond": 0.5})
        tracker1.set_data_dir(data_dir)
        tracker1._bond = 0.75
        tracker1.save()

        tracker2 = BondTracker({"initial_bond": 0.5})
        tracker2.set_data_dir(data_dir)
        tracker2.load()

        assert abs(tracker2.get_bond() - 0.75) < 0.05


@pytest.mark.asyncio
async def test_apz_key_rotation_old_data_undecryptable():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)

        store1 = APZStore(data_dir)
        eid = store1.write("secret message for APZ")
        assert store1.read(eid) == "secret message for APZ"

        store2 = APZStore(data_dir)
        with pytest.raises(Exception):
            store2.read(eid)


@pytest.mark.asyncio
async def test_evolution_log_restart_persistence():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)

        logger1 = EvolutionLogger(data_dir)
        logger1.log("test_module", "test_action", {"key": "value1"})
        logger1.log("test_module", "test_action_2", {"key": "value2"})

        logger2 = EvolutionLogger(data_dir)
        logs = logger2.get_log(limit=10)

        assert len(logs) >= 2
        assert any(e["action"] == "test_action" for e in logs)
        assert any(e["action"] == "test_action_2" for e in logs)
