"""Telegram Bot 和诞生引导测试。"""

import json
import os
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.presence.telegram_bot import BirthGuide


def _create_temp_config(data: dict = None) -> tuple[Path, Path]:
    """创建临时 config.json 并返回 (路径, 目录)"""
    tmpdir = tempfile.TemporaryDirectory()
    config_path = Path(tmpdir.name) / "config.json"
    if data:
        with open(config_path, "w") as f:
            json.dump(data, f, indent=2)
    return config_path, tmpdir


class TestBirthGuide:
    """诞生引导状态机测试"""

    def test_initial_state_is_awaiting_name(self):
        """新创建的 BirthGuide 状态应为 AWAITING_NAME"""
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        assert guide.state == "AWAITING_NAME"
        tmpdir.cleanup()

    def test_empty_config_loads_initial(self):
        """config.json 不存在时从头开始"""
        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "config.json"
            guide = BirthGuide(config_path)
            assert guide.state == "AWAITING_NAME"

    def test_completed_config_loads_completed(self):
        """config.json 中 birth_completed=True 时加载为 COMPLETED"""
        config_path, tmpdir = _create_temp_config({
            "birth_completed": True,
            "creator_name": "小明",
            "instance_type": "陪伴型",
        })
        guide = BirthGuide(config_path)
        assert guide.state == "COMPLETED"
        assert guide.data["creator_name"] == "小明"
        assert guide.data["instance_type"] == "陪伴型"
        tmpdir.cleanup()

    def test_is_completed(self):
        """is_completed() 在 COMPLETED 状态返回 True"""
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        assert guide.is_completed() is False
        guide.state = "COMPLETED"
        assert guide.is_completed() is True
        tmpdir.cleanup()

    async def _process(self, guide, text):
        """辅助方法：调用 async process"""
        return await guide.process(text)

    def test_name_accepted(self):
        """输入有效名字后转移到 AWAITING_TYPE"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        
        reply, state = asyncio.run(guide.process("小明"))
        assert guide.state == "AWAITING_TYPE"
        assert guide.data["creator_name"] == "小明"
        assert "陪伴" in reply or "助手" in reply or "平衡" in reply
        tmpdir.cleanup()

    def test_name_too_long(self):
        """超过 20 字的名字被拒绝"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        
        reply, state = asyncio.run(guide.process("这是一个非常非常长的名字测试二十个字了吧"))
        assert guide.state == "AWAITING_NAME"  # 状态不变
        assert "有点长" in reply
        tmpdir.cleanup()

    def test_cancel_resets(self):
        """取消命令重置到 AWAITING_NAME"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        
        asyncio.run(guide.process("小明"))
        assert guide.state == "AWAITING_TYPE"
        
        reply, state = asyncio.run(guide.process("取消"))
        assert guide.state == "AWAITING_NAME"
        assert guide.data["creator_name"] == ""
        tmpdir.cleanup()

    def test_type_selection(self):
        """选择类型后转移到 AWAITING_CONFIRM"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        
        asyncio.run(guide.process("小明"))
        reply, state = asyncio.run(guide.process("1"))
        assert guide.state == "AWAITING_CONFIRM"
        assert guide.data["instance_type"] == "陪伴型"
        assert "确认" in reply
        tmpdir.cleanup()

    def test_invalid_type_keeps_state(self):
        """无效类型选择停留在当前状态"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        
        asyncio.run(guide.process("小明"))
        reply, state = asyncio.run(guide.process("5"))
        assert guide.state == "AWAITING_TYPE"  # 状态不变
        
        # 然后选一个有效的
        reply, state = asyncio.run(guide.process("2"))
        assert guide.state == "AWAITING_CONFIRM"
        assert guide.data["instance_type"] == "助手型"
        tmpdir.cleanup()

    def test_confirm_completes(self):
        """确认后转移到 COMPLETED"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        
        asyncio.run(guide.process("小明"))
        asyncio.run(guide.process("3"))
        reply, state = asyncio.run(guide.process("确认"))
        assert guide.state == "COMPLETED"
        assert guide.is_completed() is True
        assert "我都会记住" in reply
        assert "从现在开始" in reply
        tmpdir.cleanup()

    def test_restart_from_confirm(self):
        """在确认阶段说"重新开始"回到 AWAITING_NAME"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        
        asyncio.run(guide.process("小明"))
        asyncio.run(guide.process("1"))
        reply, state = asyncio.run(guide.process("重新开始"))
        assert guide.state == "AWAITING_NAME"
        assert guide.data["creator_name"] == ""
        tmpdir.cleanup()

    def test_completed_returns_none_reply(self):
        """COMPLETED 状态下 process 返回 (None, COMPLETED)"""
        import asyncio
        config_path, tmpdir = _create_temp_config()
        guide = BirthGuide(config_path)
        guide.state = "COMPLETED"
        
        reply, state = asyncio.run(guide.process("你好"))
        assert reply is None
        assert state == "COMPLETED"
        tmpdir.cleanup()

    def test_saves_to_config(self):
        """状态变更后应写入 config.json"""
        import asyncio
        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "config.json"
            guide = BirthGuide(config_path)
            
            asyncio.run(guide.process("小明"))
            
            # 检查 config.json 已写入
            with open(config_path) as f:
                cfg = json.load(f)
            assert cfg["guide_state"]["state"] == "AWAITING_TYPE"
            assert cfg["guide_state"]["creator_name"] == "小明"
        # tmpdir 自动清理

    def test_telegram_bot_import(self):
        """TelegramBot 类可导入"""
        from morn_core.presence.telegram_bot import TelegramBot
        assert TelegramBot is not None