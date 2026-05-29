import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import ConfigManager


class TestConfigManager:
    @pytest.mark.asyncio
    async def test_detect_mode_switch_local(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"mode": "hybrid"}, None)
        matched, reply = await cm.detect_and_apply("改成纯本地模式")
        assert matched
        assert "已切换到纯本地模式" in reply
        assert cm.config["mode"] == "local"

    @pytest.mark.asyncio
    async def test_detect_mode_switch_cloud(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"mode": "hybrid"}, None)
        matched, reply = await cm.detect_and_apply("用云端")
        assert matched
        assert cm.config["mode"] == "cloud"

    @pytest.mark.asyncio
    async def test_detect_mode_switch_hybrid(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"mode": "local"}, None)
        matched, reply = await cm.detect_and_apply("混合模式")
        assert matched
        assert cm.config["mode"] == "hybrid"

    @pytest.mark.asyncio
    async def test_detect_temperature(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"temperature": 0.7}, None)
        matched, reply = await cm.detect_and_apply("温度调到0.3")
        assert matched
        assert cm.config["temperature"] == 0.3

    @pytest.mark.asyncio
    async def test_detect_temperature_out_of_range(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"temperature": 0.7}, None)
        matched, reply = await cm.detect_and_apply("温度调到10")
        assert matched
        assert "0.0 到 2.0" in reply
        assert cm.config["temperature"] == 0.7

    @pytest.mark.asyncio
    async def test_detect_rename(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"instance_name": "Morn"}, None)
        matched, reply = await cm.detect_and_apply("改名叫小美")
        assert matched
        assert cm.config["instance_name"] == "小美"

    @pytest.mark.asyncio
    async def test_detect_type_change(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"instance_type": "平衡型"}, None)
        matched, reply = await cm.detect_and_apply("改成陪伴型")
        assert matched
        assert cm.config["instance_type"] == "陪伴型"

    @pytest.mark.asyncio
    async def test_no_match_normal_conversation(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"mode": "hybrid"}, None)
        matched, reply = await cm.detect_and_apply("今天天气不错")
        assert not matched

    @pytest.mark.asyncio
    async def test_no_match_casual_mention(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"mode": "hybrid"}, None)
        matched, reply = await cm.detect_and_apply("本地人推荐的美食")
        assert not matched

    @pytest.mark.asyncio
    async def test_persistence(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump({"mode": "hybrid", "temperature": 0.7, "instance_name": "Morn", "instance_type": "平衡型"}, f)
            config_path = Path(f.name)

        try:
            with open(config_path) as f:
                config = json.load(f)
            cm = ConfigManager(config_path, config, None)
            await cm.detect_and_apply("改成纯本地模式")
            assert config["mode"] == "local"

            with open(config_path) as f:
                reloaded = json.load(f)
            assert reloaded["mode"] == "local"
            assert reloaded["temperature"] == 0.7
        finally:
            os.unlink(config_path)

    def test_config_manager_init(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"mode": "hybrid"}, None)
        assert cm.config_path == Path("/tmp/test_config.json")
        assert cm.config["mode"] == "hybrid"

    @pytest.mark.asyncio
    async def test_name_too_long(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"instance_name": "Morn"}, None)
        long_name = "长" * 20
        matched, reply = await cm.detect_and_apply("改名叫" + long_name)
        assert matched
        assert len(cm.config["instance_name"]) == 16

    @pytest.mark.asyncio
    async def test_mode_values_preserved(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump({"mode": "hybrid", "temperature": 0.3, "instance_name": "test", "instance_type": "助手型"}, f)
            config_path = Path(f.name)

        try:
            with open(config_path) as f:
                config = json.load(f)
            cm = ConfigManager(config_path, config, None)
            await cm.detect_and_apply("切换到纯云端模式")
            assert config["mode"] == "cloud"
            assert config["temperature"] == 0.3
            assert config["instance_name"] == "test"
            assert config["instance_type"] == "助手型"
        finally:
            os.unlink(config_path)

    def test_integration_with_engine(self):
        from morn_core.chat.engine import ChatEngine

        engine = ChatEngine(instance_name="test", memory_store=None, config={})
        assert hasattr(engine, "config_manager")
        assert engine.config_manager is None

    @pytest.mark.asyncio
    async def test_temperature_edge_low(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"temperature": 0.7}, None)
        matched, reply = await cm.detect_and_apply("温度设为0.0")
        assert matched
        assert cm.config["temperature"] == 0.0

    @pytest.mark.asyncio
    async def test_temperature_edge_high(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"temperature": 0.7}, None)
        matched, reply = await cm.detect_and_apply("温度设为2.0")
        assert matched
        assert cm.config["temperature"] == 2.0

    @pytest.mark.asyncio
    async def test_local_switch_rejected_when_ollama_down(self):
        cm = ConfigManager(Path("/tmp/test_config.json"), {"mode": "hybrid", "ollama_host": "http://localhost:11434", "ollama_model": "qwen2.5:1.5b"}, None)
        matched, reply = await cm.detect_and_apply("改成纯本地模式")
        assert matched
        # engine=None 时跳过实际检测，走正常切换流程
        assert "已切换到纯本地模式" in reply