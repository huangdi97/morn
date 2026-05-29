import json
import os
import sys
from unittest.mock import AsyncMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))


class TestDailyChat:
    @pytest.mark.asyncio
    async def test_normal_chat_returns_non_empty(self, chat_engine):
        reply = await chat_engine.chat("今天天气不错")
        assert reply
        assert isinstance(reply, str)

    @pytest.mark.asyncio
    async def test_emotion_records_happiness(self, chat_engine):
        chat_engine._call_llm = AsyncMock()
        chat_engine._call_llm.side_effect = [
            "我为你感到高兴！",
            '{"delta_score": 0.3, "tag": "高兴/喜悦"}',
        ]
        initial_pleasure = chat_engine.emotion.pleasure
        await chat_engine.chat("我今天很开心")
        assert chat_engine.emotion.pleasure > initial_pleasure

    @pytest.mark.asyncio
    async def test_memory_records_birthday(self, chat_engine):
        await chat_engine.chat("我妈妈的生日是3月15号")
        recent = await chat_engine.memory_store.get_recent(limit=5)
        descriptions = [c["description"] for c in recent]
        assert any("生日" in d or "3月15" in d for d in descriptions)

    @pytest.mark.asyncio
    async def test_restraint_mode(self, chat_engine):
        await chat_engine.set_restraint_mode(True)
        assert chat_engine.restraint_mode
        reply = await chat_engine.chat("今天天气不错")
        assert reply == "嗯，我在。"

    @pytest.mark.asyncio
    async def test_wake_from_restraint(self, chat_engine):
        await chat_engine.set_restraint_mode(True)
        reply = await chat_engine.chat("好了")
        assert not chat_engine.restraint_mode
        assert reply != "嗯，我在。"