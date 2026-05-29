import asyncio
import os
import sys
from unittest.mock import AsyncMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from morn_core.chat.engine import ChatEngine, EmotionState
from morn_core.server import MornState, heartbeat_loop


class TestFaultInjection:
    @pytest.mark.asyncio
    async def test_disk_write_failure_degrades_gracefully(self, chat_engine):
        chat_engine.memory_store.add_capsule = AsyncMock(side_effect=RuntimeError("disk full"))
        reply = await chat_engine.chat("你好")
        assert reply, "磁盘写入失败时仍应返回回复"

    @pytest.mark.asyncio
    async def test_empty_config_startup(self, memory_store, data_dir):
        config = {"_config_path": str(data_dir / "config.json")}
        engine = ChatEngine(instance_name="Morn", memory_store=memory_store, config=config)
        engine._call_llm = AsyncMock(return_value="默认回复。")
        reply = await engine.chat("你好")
        assert reply, "空配置应使用默认值启动"

    @pytest.mark.asyncio
    async def test_invalid_token_startup(self, chat_engine, memory_store, data_dir):
        from morn_core.presence.telegram_bot import TelegramBot
        bot = TelegramBot(
            token="invalid_token_no_dashes",
            chat_engine=chat_engine,
            memory_store=memory_store,
            data_dir=data_dir,
            instance_name="test",
        )
        bot._send_message = AsyncMock()
        try:
            await bot._handle_message(12345, "你好")
        except Exception:
            pytest.fail("无效 token 不应导致崩溃")

    @pytest.mark.asyncio
    async def test_memory_retrieval_failure_returns_empty(self, chat_engine):
        chat_engine.memory_store.search_fts = AsyncMock(side_effect=RuntimeError("db error"))
        chat_engine.memory_store.semantic_search = AsyncMock(side_effect=RuntimeError("db error"))
        chat_engine.memory_store.get_recent = AsyncMock(side_effect=RuntimeError("db error"))
        reply = await chat_engine.chat("测试记忆失败")
        assert reply, "记忆检索失败应返回正常回复"

    @pytest.mark.asyncio
    async def test_emotion_empty_state(self, memory_store, data_dir):
        config = {"_config_path": str(data_dir / "config.json")}
        engine = ChatEngine(instance_name="Morn", memory_store=memory_store, config=config)
        engine.emotion = EmotionState(initial={
            "calmness": 0.0, "pleasure": 0.0, "connection": 0.0,
            "determination": 0.0, "anticipation": 0.0,
            "warmth": 0.0, "ripple": 0.0,
        })
        engine._call_llm = AsyncMock(return_value="回复。")
        reply = await engine.chat("测试")
        assert reply, "情感全零时对话应正常返回"

    @pytest.mark.asyncio
    async def test_long_idle_heartbeat_still_alive(self):
        state = MornState()
        loop_task = asyncio.create_task(heartbeat_loop(state))
        await asyncio.sleep(1.5)
        loop_task.cancel()
        try:
            await loop_task
        except asyncio.CancelledError:
            pass
        assert state.heartbeat_count >= 1, "运行1.5秒后应有至少1次心跳"