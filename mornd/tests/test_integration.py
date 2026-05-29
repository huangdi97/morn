"""集成测试：验证所有模块可串联。"""

import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))


class TestConfig:
    """配置加载测试"""

    def test_load_config_creates_default(self):
        """config.json 不存在时创建默认配置"""
        from morn_core.server import load_config

        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir)
            config = load_config(data_dir)

            assert config["instance_name"] == "default"
            assert config["instance_type"] == "平衡型"
            assert config["mode"] == "hybrid"
            assert config["memory_retention_days"] == 30
            assert config["birth_completed"] is False

            config_path = data_dir / "config.json"
            assert config_path.exists()

    def test_load_config_loads_existing(self):
        """config.json 存在时加载已有配置"""
        from morn_core.server import load_config

        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir)
            custom = {"instance_name": "test_morn", "mode": "local"}
            config_path = data_dir / "config.json"
            with open(config_path, "w") as f:
                json.dump(custom, f)

            config = load_config(data_dir)
            assert config["instance_name"] == "test_morn"
            assert config["mode"] == "local"

    def test_load_config_fills_missing_defaults(self):
        """config.json 缺失字段用默认值补充"""
        from morn_core.server import load_config

        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir)
            partial = {"instance_name": "partial"}
            config_path = data_dir / "config.json"
            with open(config_path, "w") as f:
                json.dump(partial, f)

            config = load_config(data_dir)
            assert config["instance_name"] == "partial"
            assert config["mode"] == "hybrid"
            assert config["memory_retention_days"] == 30


class TestModuleImports:
    """模块导入集成测试"""

    def test_all_modules_import(self):
        """所有模块可导入"""
        from morn_core.server import main, load_config, MornState, parse_args
        from morn_core.memory.store import MemoryStore
        from morn_core.chat.engine import ChatEngine, EmotionState
        from morn_core.security.user_protection import UserProtection
        assert main is not None
        assert load_config is not None
        assert MemoryStore is not None
        assert ChatEngine is not None
        assert EmotionState is not None
        assert UserProtection is not None

    def test_memory_store_and_chat_engine_compatible(self):
        """MemoryStore 和 ChatEngine 可串联"""
        from morn_core.memory.store import MemoryStore
        from morn_core.chat.engine import ChatEngine, EmotionState

        assert hasattr(MemoryStore, "add_capsule")
        assert hasattr(MemoryStore, "search_fts")
        assert hasattr(MemoryStore, "get_recent")
        assert hasattr(MemoryStore, "count")
        assert hasattr(ChatEngine, "chat")
        assert hasattr(EmotionState, "apply_delta")
        assert hasattr(EmotionState, "decay")
        # emotion is instance attribute, not class attribute — verified by end-to-end test


class TestEndToEnd:
    """端到端基本流程测试（不启动事件循环）"""

    @pytest.mark.asyncio
    async def test_memory_chat_protection_flow(self):
        """记忆 → 对话 → 保护层，数据流可跑通"""
        from morn_core.memory.store import MemoryStore
        from morn_core.chat.engine import ChatEngine, EmotionState
        from morn_core.security.user_protection import UserProtection

        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir)

            async with MemoryStore(data_dir) as store:
                engine = ChatEngine(
                    instance_name="test",
                    memory_store=store,
                    config={"mode": "local"},
                )

                eid = await store.add_capsule({
                    "entities": '["创建者", "Morn"]',
                    "description": "创建者说正在测试系统",
                })
                assert eid is not None

                results = await store.search_fts("测试")
                assert len(results) >= 1

                engine.emotion.apply_delta(0.3, "高兴/满足")
                assert engine.emotion.pleasure > 0.5

                protection = UserProtection()
                filtered, triggered = protection.filter("你好，今天天气不错")
                assert triggered == []

                count = await store.count()
                assert count >= 1

                print("端到端数据流验证通过 ✅")

    def test_command_line_args(self):
        """命令行参数解析"""
        from morn_core.server import parse_args

        old_argv = sys.argv
        try:
            sys.argv = ["server.py", "--instance", "test_e2e"]
            args = parse_args()
            assert args.instance == "test_e2e"
        finally:
            sys.argv = old_argv
