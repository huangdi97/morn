import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.chat.engine import EmotionState
from morn_core.emotion.bond_tracker import BondTracker
from morn_core.evolution.orchestrator import EvolutionOrchestrator
from morn_core.consciousness.health_report import HealthReport


def _dummy_optimize():
    return True


@pytest.mark.asyncio
async def test_health_report_generates():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            emotion = EmotionState()
            bond = BondTracker({"initial_bond": 0.5})
            evo = EvolutionOrchestrator()
            evo.register_fast("test_component", _dummy_optimize)

            report = HealthReport(
                memory_store=store,
                emotion_engine=emotion,
                evolution_orchestrator=evo,
                bond_tracker=bond,
                data_dir=data_dir,
            )

            result = await report.generate()
            assert isinstance(result, str)
            assert len(result) > 0


@pytest.mark.asyncio
async def test_health_report_contains_key_fields():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            emotion = EmotionState()
            bond = BondTracker({"initial_bond": 0.5})
            evo = EvolutionOrchestrator()
            evo.register_fast("test_component", _dummy_optimize)

            report = HealthReport(
                memory_store=store,
                emotion_engine=emotion,
                evolution_orchestrator=evo,
                bond_tracker=bond,
                data_dir=data_dir,
            )

            result = await report.generate()
            assert "心跳" in result or "运行时长" in result
            assert "记忆" in result or "胶囊" in result
            assert "情感" in result
            assert "Bond" in result or "bond" in result.lower()
            assert "技能" in result


@pytest.mark.asyncio
async def test_health_report_data_consistency():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            eid1 = await store.add_capsule({
                "entities": '["用户"]',
                "description": "用户说今天很开心",
                "source": "chat",
            })
            eid2 = await store.add_capsule({
                "entities": '["用户"]',
                "description": "用户完成了任务",
                "source": "chat",
            })

            emotion = EmotionState()
            emotion.pleasure = 0.85
            emotion.calmness = 0.75

            bond = BondTracker({"initial_bond": 0.6})
            evo = EvolutionOrchestrator()
            evo.register_fast("test_component", _dummy_optimize)

            report = HealthReport(
                memory_store=store,
                emotion_engine=emotion,
                evolution_orchestrator=evo,
                bond_tracker=bond,
                data_dir=data_dir,
            )

            result = await report.generate()

            capsule_count = await store.count()
            assert str(capsule_count) in result

            assert "pleasure=0.85" in result


@pytest.mark.asyncio
async def test_health_report_empty_state():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            emotion = EmotionState()
            bond = BondTracker({})
            evo = EvolutionOrchestrator()

            report = HealthReport(
                memory_store=store,
                emotion_engine=emotion,
                evolution_orchestrator=evo,
                bond_tracker=bond,
                data_dir=data_dir,
            )

            result = await report.generate()
            assert isinstance(result, str)
            assert len(result) > 0
