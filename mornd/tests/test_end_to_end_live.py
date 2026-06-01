import asyncio
import json
import tempfile
import time
from pathlib import Path

import pytest

from morn_core.eventbus.bus import EventBus, Event, Priority
from morn_core.eventbus.hooks import HookManager
from morn_core.eventbus.health_monitor import HealthMonitor
from morn_core.security.user_protection import UserProtection
from morn_core.security.external_boundary import ExternalBoundary
from morn.contrib.security_advanced.ethical_judgment import EthicalJudgment
from morn_core.security.security_validator import SecurityValidator
from morn_core.action.cli_executor import CLIExecutor
from morn_core.heartbeat import heartbeat_loop


@pytest.fixture
async def event_bus():
    loop = asyncio.get_event_loop()
    bus = EventBus(loop)
    await bus.start()
    yield bus
    await bus.stop()


@pytest.fixture
def hook_manager(event_bus):
    return HookManager(event_bus)


class MockState:
    def __init__(self):
        self.heartbeat_count = 0
        self.last_heartbeat = 0.0
        self.shutdown = False
        self.last_interaction_time = time.time()
        self.start_time = time.time()
        self.mem_history = []
        self.db = None
        self.memory_store = None
        self.chat_engine = None

    def log(self, module, message):
        pass


class TestHeartbeatToEventBusLifecycle:
    async def test_heartbeat_publishes_tick_and_minute(self, event_bus):
        state = MockState()
        received = []
        async def listener(event):
            received.append(event.type)
        event_bus.subscribe("heartbeat.tick", listener, "tick_watcher")
        event_bus.subscribe("heartbeat.minute", listener, "minute_watcher")
        task = asyncio.create_task(heartbeat_loop(state, event_bus))
        await asyncio.sleep(2.5)
        state.shutdown = True
        await task
        assert "heartbeat.tick" in received


class TestSecurityValidatorIntegration:
    async def test_cli_executor_publishes_security_alert(self, event_bus):
        config = {"risk_preference": "yellow"}
        validator = SecurityValidator(config, event_bus=event_bus)
        executor = CLIExecutor(config, validator=validator, event_bus=event_bus)
        received = []
        async def listener(event):
            received.append(event)
        event_bus.subscribe("security.alert", listener, "alert_watcher")
        result = await executor.async_execute("echo hello", risk_level="red")
        await asyncio.sleep(0.1)
        assert result.get("error", "").startswith("blocked:")

    async def test_user_protection_publishes_alert(self, event_bus):
        protection = UserProtection(event_bus=event_bus)
        received = []
        async def listener(event):
            received.append(event)
        event_bus.subscribe("security.alert", listener, "alert_watcher")
        protection.filter("你怎么忍心离开我")
        await asyncio.sleep(0.1)
        assert len(received) >= 1
        assert received[0].payload["source"] == "user_protection"

    async def test_external_boundary_publishes_alert(self, event_bus):
        with tempfile.TemporaryDirectory() as tmpdir:
            boundary = ExternalBoundary(Path(tmpdir), event_bus=event_bus)
            received = []
            async def listener(event):
                received.append(event)
            event_bus.subscribe("security.alert", listener, "alert_watcher")
            boundary.check_inbound("tcp", 80, "1.2.3.4")
            await asyncio.sleep(0.1)
            assert len(received) >= 1
            assert received[0].payload["source"] == "external_boundary"

    async def test_ethical_judgment_publishes_alert(self, event_bus):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir), event_bus=event_bus)
            received = []
            async def listener(event):
                received.append(event)
            event_bus.subscribe("security.alert", listener, "alert_watcher")
            ej.propose("self_modify")
            await asyncio.sleep(0.1)
            assert len(received) >= 1
            assert received[0].payload["source"] == "ethical_judgment"


class TestMemoryCapsuleEventChain:
    async def test_memory_store_publishes_capsule_written(self, event_bus, data_dir):
        from morn_core.memory.store import MemoryStore
        store = MemoryStore(data_dir, event_bus=event_bus)
        await store.__aenter__()
        received = []
        async def listener(event):
            received.append(event)
        event_bus.subscribe("memory.capsule_written", listener, "capsule_watcher")
        capsule_id = await store.add_capsule({
            "description": "end-to-end test capsule",
            "source": "test",
        })
        await asyncio.sleep(0.1)
        assert len(received) >= 1
        assert received[0].type == "memory.capsule_written"
        assert received[0].payload["capsule_id"] == capsule_id
        await store.close()


class TestHealthMonitorSelfCheck:
    async def test_self_check_publishes_health_warning(self, event_bus, hook_manager):
        state = MockState()
        monitor = HealthMonitor(event_bus, hook_manager, state)
        received = []
        async def listener(event):
            received.append(event)
        event_bus.subscribe("kernel.health_warning", listener, "warning_watcher")
        for _ in range(55):
            event_bus._queues[Priority.MEDIUM].put_nowait(Event(
                type="test.backlog", payload={}, source="test",
                priority=Priority.MEDIUM,
            ))
        await monitor.self_check(Event(
            type="heartbeat.minute", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.1)
        assert len(received) >= 1
        assert received[0].type == "kernel.health_warning"


class TestHotReloadConfigChange:
    async def test_config_reload_triggers_event(self, event_bus):
        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "config.json"
            config = {"risk_preference": "yellow"}
            validator = SecurityValidator(config, event_bus=event_bus)
            config_path.write_text(json.dumps(config, ensure_ascii=False))
            validator.set_config_path(str(config_path))
            received = []
            async def listener(event):
                received.append(event)
            event_bus.subscribe("security.config_reloaded", listener, "reload_watcher")
            count = validator.reload_config()
            await asyncio.sleep(0.1)
            assert count > 0

    async def test_config_reload_noop_on_unchanged(self, event_bus):
        with tempfile.TemporaryDirectory() as tmpdir:
            config_path = Path(tmpdir) / "config.json"
            config = {"risk_preference": "yellow"}
            validator = SecurityValidator(config, event_bus=event_bus)
            config_path.write_text(json.dumps(config, ensure_ascii=False))
            validator.set_config_path(str(config_path))
            validator.reload_config()
            assert validator.reload_config() == 0
