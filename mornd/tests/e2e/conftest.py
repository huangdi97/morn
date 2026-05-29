import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock

import pytest
import pytest_asyncio

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from morn_core.memory.store import MemoryStore
from morn_core.chat.engine import ChatEngine
from morn_core.presence.telegram_bot import TelegramBot


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_e2e_") as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def bot_config():
    return {
        "instance_type": "平衡型",
        "mode": "hybrid",
        "temperature": 0.7,
        "api_key": "fake-key",
        "api_base": "https://api.deepseek.com/v1",
        "model_name": "deepseek-chat",
        "ollama_model": "qwen2.5:1.5b",
        "ollama_host": "http://localhost:11434",
    }


@pytest_asyncio.fixture
async def memory_store(data_dir):
    async with MemoryStore(data_dir) as store:
        yield store


@pytest_asyncio.fixture
async def chat_engine(memory_store, bot_config, data_dir):
    config = {**bot_config, "_config_path": str(data_dir / "config.json")}
    engine = ChatEngine(
        instance_name="Morn",
        memory_store=memory_store,
        config=config,
    )
    engine._call_llm = AsyncMock(return_value="这是一个模拟回复。")
    return engine


@pytest_asyncio.fixture
async def bot(chat_engine, memory_store, data_dir):
    b = TelegramBot(
        token="fake:token",
        chat_engine=chat_engine,
        memory_store=memory_store,
        data_dir=data_dir,
        instance_name="test",
    )
    b._send_message = AsyncMock()
    return b