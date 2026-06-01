import os
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.chat.engine import ChatEngine


@pytest.mark.asyncio
async def test_restraint_skips_active_greeting():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            engine = ChatEngine(
                instance_name="test",
                memory_store=store,
                config={"mode": "local"},
            )

            await engine.set_restraint_mode(True, duration="2h")

            reply = await engine.chat("你好")
            assert reply == "嗯，我在。"

            reply2 = await engine.chat("今天天气怎么样")
            assert reply2 == "嗯，我在。"


@pytest.mark.asyncio
async def test_restraint_auto_expiry():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            engine = ChatEngine(
                instance_name="test",
                memory_store=store,
                config={"mode": "local"},
            )

            await engine.set_restraint_mode(True, duration="1m")
            assert engine.restraint_mode is True

            engine.restraint_until = (datetime.now(timezone.utc) - timedelta(seconds=10)).isoformat()

            in_restraint = await engine._is_in_restraint()
            assert in_restraint is False
            assert engine.restraint_mode is False


@pytest.mark.asyncio
async def test_restraint_wake_word_exit():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            engine = ChatEngine(
                instance_name="test",
                memory_store=store,
                config={"mode": "local"},
            )

            await engine.set_restraint_mode(True)
            reply = await engine.chat("好了")
            assert engine.restraint_mode is False

            await engine.set_restraint_mode(True)
            reply2 = await engine.chat("可以说话了")
            assert engine.restraint_mode is False

            await engine.set_restraint_mode(True)
            reply3 = await engine.chat("醒醒")
            assert engine.restraint_mode is False

            await engine.set_restraint_mode(True)
            reply4 = await engine.chat("陪我说话")
            assert engine.restraint_mode is False


@pytest.mark.asyncio
async def test_restraint_non_wake_short_reply():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        data_dir = Path(tmpdir)
        async with MemoryStore(data_dir) as store:
            engine = ChatEngine(
                instance_name="test",
                memory_store=store,
                config={"mode": "local"},
            )

            await engine.set_restraint_mode(True)
            reply = await engine.chat("今天有什么新闻")
            assert reply == "嗯，我在。"

            reply2 = await engine.chat("讲个笑话")
            assert reply2 == "嗯，我在。"