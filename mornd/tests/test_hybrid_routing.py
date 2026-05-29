"""对话引擎混合路由（hybrid mode）健壮性验证测试。"""

import asyncio
import os
import sys
from unittest.mock import AsyncMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import ChatEngine


# ── fixtures ─────────────────────────────────────────────────────────

@pytest.fixture
def mock_memory():
    memory = AsyncMock()
    memory.add_capsule = AsyncMock()
    memory.search_fts = AsyncMock(return_value=[])
    memory.semantic_search = AsyncMock(return_value=[])
    memory.get_recent = AsyncMock(return_value=[])
    return memory


@pytest.fixture
def mock_search(monkeypatch):
    monkeypatch.setattr(ChatEngine, '_search_memory', AsyncMock(return_value=""))


@pytest.fixture
def mock_emotion_tag(monkeypatch):
    monkeypatch.setattr(ChatEngine, '_generate_emotion_tag', AsyncMock(return_value=(0.0, "")))


@pytest.fixture
def engine_hybrid(mock_memory, mock_search, mock_emotion_tag):
    return ChatEngine(
        instance_name="test",
        memory_store=mock_memory,
        config={"mode": "hybrid"},
    )


@pytest.fixture
def engine_cloud(mock_memory, mock_search, mock_emotion_tag):
    return ChatEngine(
        instance_name="test",
        memory_store=mock_memory,
        config={"mode": "cloud"},
    )


@pytest.fixture
def engine_local(mock_memory, mock_search, mock_emotion_tag):
    return ChatEngine(
        instance_name="test",
        memory_store=mock_memory,
        config={"mode": "local"},
    )


# ── 1. 正常路径（基线验证）───────────────────────────────────────────

class TestNormalPaths:

    @pytest.mark.asyncio
    async def test_cloud_normal(self, engine_hybrid, monkeypatch):
        """网络正常 → 走云端 → 正常返回"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(return_value="cloud reply"))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_hybrid.chat("你好")
        assert "cloud" in reply

    @pytest.mark.asyncio
    async def test_cloud_mode(self, engine_cloud, monkeypatch):
        """纯 cloud 模式 → 不走网络检测，直接走云端"""
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(return_value="cloud reply"))

        reply = await engine_cloud.chat("你好")
        assert "cloud" in reply

    @pytest.mark.asyncio
    async def test_local_mode(self, engine_local, monkeypatch):
        """纯 local 模式 → 不走网络检测，直接走本地"""
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_local.chat("你好")
        assert "local" in reply


# ── 2. 网络切换边界条件 ───────────────────────────────────────────────

class TestNetworkEdgeCases:

    @pytest.mark.asyncio
    async def test_cloud_timeout_fallback(self, engine_hybrid, monkeypatch):
        """云端超时（30s）→ 自动 fallback 到本地"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=asyncio.TimeoutError()))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_hybrid.chat("你好")
        assert "local" in reply

    @pytest.mark.asyncio
    async def test_cloud_500_fallback(self, engine_hybrid, monkeypatch):
        """云端返回 HTTP 500 → fallback 到本地"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=RuntimeError("API error: 500 Internal Server Error")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_hybrid.chat("你好")
        assert "local" in reply

    @pytest.mark.asyncio
    async def test_cloud_429_fallback(self, engine_hybrid, monkeypatch):
        """云端返回 HTTP 429 → fallback 到本地"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=RuntimeError("API error: 429 Too Many Requests")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_hybrid.chat("你好")
        assert "local" in reply

    @pytest.mark.asyncio
    async def test_cloud_dns_fail_fallback(self, engine_hybrid, monkeypatch):
        """云端 DNS 解析失败 → fallback 到本地"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=OSError("DNS resolution failed")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_hybrid.chat("你好")
        assert "local" in reply

    @pytest.mark.asyncio
    async def test_cloud_connection_refused_fallback(self, engine_hybrid, monkeypatch):
        """云端连接被拒绝 → fallback 到本地"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=ConnectionError("Connection refused")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_hybrid.chat("你好")
        assert "local" in reply

    @pytest.mark.asyncio
    async def test_check_ok_then_cloud_fails(self, engine_hybrid, monkeypatch):
        """网络检测通过但调用 cloud 时抛异常 → fallback 到本地"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=RuntimeError("API unavailable")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        reply = await engine_hybrid.chat("你好")
        assert "local" in reply


# ── 3. 频繁切换（flapping）────────────────────────────────────────────

class TestFlapping:

    @pytest.mark.asyncio
    async def test_flapping_10_times(self, engine_hybrid, mock_memory, monkeypatch):
        """连续 10 次交替：网络通→走云→断→走本地，验证记忆写入"""
        seq = [True, False] * 5
        check_mock = AsyncMock(side_effect=list(seq))
        cloud_mock = AsyncMock(return_value="cloud reply")

        monkeypatch.setattr(ChatEngine, '_check_network', check_mock)
        monkeypatch.setattr(ChatEngine, '_call_cloud', cloud_mock)
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        replies = []
        for i in range(10):
            reply = await engine_hybrid.chat(f"消息{i}")
            replies.append(reply)

        for i, reply in enumerate(replies):
            if seq[i]:
                assert "cloud" in reply, f"第{i}次应走云端"
            else:
                assert "local" in reply, f"第{i}次应走本地"

        assert mock_memory.add_capsule.call_count == 10
        assert check_mock.call_count == 10
        assert cloud_mock.call_count == 5


# ── 4. 状态一致性 ─────────────────────────────────────────────────────

class TestStateConsistency:

    @pytest.mark.asyncio
    async def test_emotion_updated_after_cloud(self, engine_hybrid, monkeypatch):
        """云端成功 → 情感状态更新"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(return_value="cloud reply"))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))
        monkeypatch.setattr(ChatEngine, '_generate_emotion_tag', AsyncMock(return_value=(0.3, "高兴/满意")))

        pleasure_before = engine_hybrid.emotion.pleasure
        await engine_hybrid.chat("你好")
        assert engine_hybrid.emotion.pleasure > pleasure_before

    @pytest.mark.asyncio
    async def test_emotion_updated_after_fallback(self, engine_hybrid, monkeypatch):
        """云端失败 fallback 到本地 → 情感状态仍然更新（基于本地回复）"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=RuntimeError("fail")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))
        monkeypatch.setattr(ChatEngine, '_generate_emotion_tag', AsyncMock(return_value=(0.2, "平静")))

        pleasure_before = engine_hybrid.emotion.pleasure
        await engine_hybrid.chat("你好")
        assert engine_hybrid.emotion.pleasure > pleasure_before

    @pytest.mark.asyncio
    async def test_both_unavailable(self, engine_hybrid, monkeypatch):
        """全部失败（云端+本地都不可用）→ 降级消息，情感状态不异常突变"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=RuntimeError("cloud fail")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(side_effect=RuntimeError("local fail")))

        pleasure_before = engine_hybrid.emotion.pleasure
        reply = await engine_hybrid.chat("你好")
        assert "连接不上" in reply or "没听清楚" in reply
        assert abs(engine_hybrid.emotion.pleasure - pleasure_before) <= 0.01

    @pytest.mark.asyncio
    async def test_hybrid_config_unchanged(self, engine_hybrid, monkeypatch):
        """切换前后 _mode 配置不变"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(return_value="cloud reply"))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        before = engine_hybrid._mode
        await engine_hybrid.chat("你好")
        await engine_hybrid.chat("世界")
        assert engine_hybrid._mode == before


# ── 5. 记忆完整性 ─────────────────────────────────────────────────────

class TestMemoryIntegrity:

    @pytest.mark.asyncio
    async def test_memory_written_after_normal(self, engine_hybrid, mock_memory, monkeypatch):
        """正常对话后写入记忆 capsule"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(return_value="cloud reply"))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        await engine_hybrid.chat("你好")

        mock_memory.add_capsule.assert_awaited_once()
        call_args = mock_memory.add_capsule.await_args[0][0]
        assert "emotion_score" in call_args
        assert "description" in call_args

    @pytest.mark.asyncio
    async def test_memory_written_after_fallback(self, engine_hybrid, mock_memory, monkeypatch):
        """降级后记忆正确写入"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=RuntimeError("fail")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        await engine_hybrid.chat("你好")

        mock_memory.add_capsule.assert_awaited_once()
        call_args = mock_memory.add_capsule.await_args[0][0]
        assert "emotion_score" in call_args
        assert "description" in call_args

    @pytest.mark.asyncio
    async def test_memory_written_after_both_fail(self, engine_hybrid, mock_memory, monkeypatch):
        """降级消息也写入记忆"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(return_value=True))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(side_effect=RuntimeError("cloud fail")))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(side_effect=RuntimeError("local fail")))

        await engine_hybrid.chat("你好")

        mock_memory.add_capsule.assert_awaited_once()


# ── 6. 并发安全 ───────────────────────────────────────────────────────

class TestConcurrency:

    @pytest.mark.asyncio
    async def test_rapid_fire_5_messages(self, engine_hybrid, mock_memory, monkeypatch):
        """连续快速发送 5 条消息，间隔 100ms，网络在第 3 条时切换状态"""
        monkeypatch.setattr(ChatEngine, '_check_network', AsyncMock(side_effect=[True, True, False, False, False]))
        monkeypatch.setattr(ChatEngine, '_call_cloud', AsyncMock(return_value="cloud reply"))
        monkeypatch.setattr(ChatEngine, '_call_local', AsyncMock(return_value="local reply"))

        replies = []
        for i in range(5):
            reply = await engine_hybrid.chat(f"消息{i}")
            replies.append(reply)
            await asyncio.sleep(0.1)

        assert len(replies) == 5
        assert mock_memory.add_capsule.call_count == 5

        for i, call in enumerate(mock_memory.add_capsule.await_args_list):
            capsule = call[0][0]
            assert f"消息{i}" in capsule.get("description", ""), f"第{i}条消息应出现在记忆中"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])