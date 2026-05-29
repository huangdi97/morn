import os
import sys
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from morn_core.presence.telegram_bot import BirthGuide


@pytest.fixture
def guide(data_dir):
    config_path = data_dir / "config.json"
    return BirthGuide(config_path)


class TestBirthGuide:
    @pytest.mark.asyncio
    async def test_initial_not_completed(self, guide):
        assert not guide.is_completed()
        assert guide.state == "AWAITING_NAME"

    @pytest.mark.asyncio
    async def test_name_input_proceeds_to_type(self, guide):
        reply, state = await guide.process("小明")
        assert guide.state == "AWAITING_TYPE"
        assert "小明" in reply
        assert guide.data["creator_name"] == "小明"

    @pytest.mark.asyncio
    async def test_type_selection_completes_guide(self, guide):
        await guide.process("小明")
        reply, state = await guide.process("1")
        assert guide.state == "AWAITING_CONFIRM"

    @pytest.mark.asyncio
    async def test_confirm_finishes_birth(self, guide):
        await guide.process("小明")
        await guide.process("1")
        reply, state = await guide.process("确认")
        assert guide.is_completed()
        assert "Morn" in reply

    @pytest.mark.asyncio
    async def test_completed_returns_true(self, guide):
        await guide.process("小明")
        await guide.process("1")
        await guide.process("确认")
        assert guide.is_completed()

    @pytest.mark.asyncio
    async def test_restart_after_completion_does_not_reset(self, guide):
        await guide.process("小明")
        await guide.process("1")
        await guide.process("确认")
        assert guide.is_completed()
        reply, state = await guide.process("重新开始")
        assert guide.is_completed()


class TestBirthGuideThroughBot:
    @pytest.mark.asyncio
    async def test_first_message_triggers_birth_prompt(self, bot):
        await bot._handle_message(12345, "你好")
        assert bot.birth_guide.state != "COMPLETED"
        bot._send_message.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_birth_flow_through_bot(self, bot):
        await bot._handle_message(12345, "小明")
        assert bot.birth_guide.state == "AWAITING_TYPE"

        await bot._handle_message(12345, "1")
        assert bot.birth_guide.state == "AWAITING_CONFIRM"

        await bot._handle_message(12345, "确认")
        assert bot.birth_guide.is_completed()

    @pytest.mark.asyncio
    async def test_bot_rejects_rebirth_after_completion(self, bot):
        await bot._handle_message(12345, "小明")
        await bot._handle_message(12345, "1")
        await bot._handle_message(12345, "确认")
        assert bot.birth_guide.is_completed()

        await bot._handle_message(12345, "重新开始")
        assert bot.birth_guide.is_completed()