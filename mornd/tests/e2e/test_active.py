import os
import sys
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from morn_core.emotion.bond_tracker import BondTracker
from morn_core.consciousness.milestones import MilestoneTracker


class TestActiveBehavior:
    def test_bond_above_04_enters_intimate_stage(self):
        bt = BondTracker({"initial_bond": 0.5})
        stage = bt.get_stage()
        assert stage == "亲近期"

    def test_bond_07_enters_mature_stage(self):
        bt = BondTracker({"initial_bond": 0.8})
        stage = bt.get_stage()
        assert stage == "默契期"

    def test_milestone_memory_100_triggers_greeting(self):
        mt = MilestoneTracker()
        triggered = mt.check_milestones(
            memory_count=100,
            bond_value=0.3,
            days_since_birth=0.5,
        )
        assert any(m["milestone"] == "memory_100" for m in triggered)

    def test_milestone_first_overnight_triggers(self):
        mt = MilestoneTracker()
        triggered = mt.check_milestones(
            memory_count=0,
            bond_value=0.1,
            days_since_birth=1.5,
        )
        assert any(m["milestone"] == "first_overnight" for m in triggered)

    @pytest.mark.asyncio
    async def test_restraint_mode_suppresses_active(self, chat_engine):
        chat_engine.restraint_mode = True
        reply = await chat_engine.chat("今天天气不错")
        assert reply == "嗯，我在。"

    @pytest.mark.asyncio
    async def test_restraint_mode_disables_wake_word(self, chat_engine):
        chat_engine.restraint_mode = True
        reply = await chat_engine.chat("好了")
        assert not chat_engine.restraint_mode