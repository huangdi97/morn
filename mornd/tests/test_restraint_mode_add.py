import os
import sys
from datetime import datetime, timedelta, timezone

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import ChatEngine


def _make_engine():
    return ChatEngine(instance_name="test", memory_store=None, config={})


@pytest.mark.asyncio
async def test_default_restraint_mode_is_false():
    engine = _make_engine()
    assert engine.restraint_mode is False


@pytest.mark.asyncio
async def test_enable_restraint_mode_sets_true():
    engine = _make_engine()
    await engine.set_restraint_mode(True)
    assert engine.restraint_mode is True


@pytest.mark.asyncio
async def test_restraint_expires_after_duration():
    engine = _make_engine()
    await engine.set_restraint_mode(True, duration="1m")
    assert engine.restraint_mode is True
    assert await engine._is_in_restraint() is True


@pytest.mark.asyncio
async def test_wake_word_exits_restraint():
    engine = _make_engine()
    await engine.set_restraint_mode(True)
    reply = await engine.chat("好了")
    assert engine.restraint_mode is False
    assert engine.restraint_until is None


@pytest.mark.asyncio
async def test_wake_word_kan_shuo_hua_le_exits_restraint():
    engine = _make_engine()
    await engine.set_restraint_mode(True)
    reply = await engine.chat("可以说话了")
    assert engine.restraint_mode is False


@pytest.mark.asyncio
async def test_disable_restraint_mode():
    engine = _make_engine()
    await engine.set_restraint_mode(True)
    await engine.set_restraint_mode(False)
    assert engine.restraint_mode is False
    assert engine.restraint_until is None


@pytest.mark.asyncio
async def test_restraint_short_reply_for_non_wake():
    engine = _make_engine()
    await engine.set_restraint_mode(True)
    reply = await engine.chat("随便说点什么")
    assert reply == "嗯，我在。"


@pytest.mark.asyncio
async def test_restraint_exit_then_normal():
    engine = _make_engine()
    await engine.set_restraint_mode(True)
    await engine.set_restraint_mode(False)
    reply = await engine.chat("你好")
    assert reply != "嗯，我在。"