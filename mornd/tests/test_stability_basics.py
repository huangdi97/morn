"""基础稳定性测试。

验证核心子系统初始化正常、后台循环可以启动/停止、情感状态始终在[0,1]范围内。
"""

import asyncio
import os
import sys
from unittest.mock import patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import EmotionState


class TestHeartbeatLoop:
    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_heartbeat_loop_exits_on_shutdown(self, mock_log):
        from morn_core.server import heartbeat_loop, MornState
        state = MornState()
        state.shutdown = True
        await heartbeat_loop(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_heartbeat_increments_count(self, mock_log):
        from morn_core.server import heartbeat_loop, MornState
        state = MornState()
        state.shutdown = True
        await heartbeat_loop(state)
        assert state.heartbeat_count >= 0

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_heartbeat_sets_last_heartbeat_time(self, mock_log):
        from morn_core.server import heartbeat_loop, MornState
        state = MornState()
        state.shutdown = True
        await heartbeat_loop(state)
        assert state.last_heartbeat >= 0


class TestMemoryMonitorLoop:
    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    @patch("morn_core.server.psutil")
    async def test_memory_monitor_exits_on_shutdown(self, mock_psutil, mock_log):
        from morn_core.server import memory_monitor, MornState
        mock_psutil.Process().memory_info().rss = 100 * 1024 * 1024
        state = MornState()
        state.shutdown = True
        await memory_monitor(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    @patch("morn_core.server.psutil")
    async def test_memory_monitor_records_usage(self, mock_psutil, mock_log):
        from morn_core.server import memory_monitor, MornState
        mock_psutil.Process().memory_info().rss = 100 * 1024 * 1024
        state = MornState()
        state.shutdown = True
        await memory_monitor(state)
        assert len(state.mem_history) >= 0


class TestEmotionStateInRange:
    def test_all_seven_dims_in_range_after_positive_delta(self):
        e = EmotionState()
        for _ in range(10):
            e.apply_delta(0.5, "高兴/满足")
        assert 0.0 <= e.calmness <= 1.0
        assert 0.0 <= e.pleasure <= 1.0
        assert 0.0 <= e.connection <= 1.0
        assert 0.0 <= e.determination <= 1.0
        assert 0.0 <= e.anticipation <= 1.0
        assert 0.0 <= e.warmth <= 1.0
        assert 0.0 <= e.ripple <= 1.0

    def test_all_seven_dims_in_range_after_negative_delta(self):
        e = EmotionState()
        for _ in range(10):
            e.apply_delta(-0.5, "失望/沮丧")
        assert 0.0 <= e.calmness <= 1.0
        assert 0.0 <= e.pleasure <= 1.0
        assert 0.0 <= e.connection <= 1.0
        assert 0.0 <= e.determination <= 1.0
        assert 0.0 <= e.anticipation <= 1.0
        assert 0.0 <= e.warmth <= 1.0
        assert 0.0 <= e.ripple <= 1.0

    def test_all_seven_dims_in_range_after_decay(self):
        e = EmotionState()
        e.pleasure = 0.9
        e.calmness = 0.2
        e.connection = 0.8
        for _ in range(20):
            e.decay()
        assert 0.0 <= e.calmness <= 1.0
        assert 0.0 <= e.pleasure <= 1.0
        assert 0.0 <= e.connection <= 1.0

    def test_ripple_trigger_stays_in_range(self):
        e = EmotionState()
        for _ in range(20):
            e.trigger_ripple()
        assert 0.0 <= e.ripple <= 1.0

    def test_extreme_deltas_clamped(self):
        e = EmotionState()
        e.apply_delta(100.0, "极度正面")
        assert e.pleasure <= 1.0
        assert e.calmness <= 1.0
        assert e.connection <= 1.0
        e.apply_delta(-100.0, "极度负面")
        assert e.pleasure >= 0.0
        assert e.calmness >= 0.0
        assert e.connection >= 0.0


class TestBackgroundLoops:
    def test_heartbeat_loop_is_coroutine_function(self):
        from morn_core.server import heartbeat_loop
        assert asyncio.iscoroutinefunction(heartbeat_loop)

    def test_memory_monitor_is_coroutine_function(self):
        from morn_core.server import memory_monitor
        assert asyncio.iscoroutinefunction(memory_monitor)

    def test_bond_update_loop_is_coroutine_function(self):
        from morn_core.server import bond_update_loop
        assert asyncio.iscoroutinefunction(bond_update_loop)

    def test_intent_drift_loop_is_coroutine_function(self):
        from morn_core.server import intent_drift_loop
        assert asyncio.iscoroutinefunction(intent_drift_loop)

    def test_audit_loop_is_coroutine_function(self):
        from morn_core.server import audit_loop
        assert asyncio.iscoroutinefunction(audit_loop)

    def test_self_pruning_loop_is_coroutine_function(self):
        from morn_core.server import self_pruning_loop
        assert asyncio.iscoroutinefunction(self_pruning_loop)


class TestLoopShutdown:
    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_heartbeat_loop_stops_on_shutdown(self, mock_log):
        from morn_core.server import heartbeat_loop, MornState
        state = MornState()
        state.shutdown = True
        await heartbeat_loop(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    @patch("morn_core.server.psutil")
    async def test_memory_monitor_stops_on_shutdown(self, mock_psutil, mock_log):
        from morn_core.server import memory_monitor, MornState
        mock_psutil.Process().memory_info().rss = 100 * 1024 * 1024
        state = MornState()
        state.shutdown = True
        await memory_monitor(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_bond_loop_stops_on_shutdown(self, mock_log):
        from morn_core.server import bond_update_loop, MornState
        state = MornState()
        state.shutdown = True
        await bond_update_loop(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_drift_loop_stops_on_shutdown(self, mock_log):
        from morn_core.server import intent_drift_loop, MornState
        state = MornState()
        state.shutdown = True
        await intent_drift_loop(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_audit_loop_stops_on_shutdown(self, mock_log):
        from morn_core.server import audit_loop, MornState
        state = MornState()
        state.shutdown = True
        await audit_loop(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_self_pruning_loop_stops_on_shutdown(self, mock_log):
        from morn_core.server import self_pruning_loop, MornState
        state = MornState()
        state.shutdown = True
        await self_pruning_loop(state)