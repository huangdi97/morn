import asyncio
import time
from unittest.mock import MagicMock, AsyncMock

import pytest

from morn_core.eventbus.bus import EventBus, Event, Priority
from morn_core.eventbus.hooks import HookManager, HookRegistration
from morn_core.eventbus.plugin_registry import register_all_plugin_hooks
from morn_core.eventbus.health_monitor import HealthMonitor


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


class MockPlugin:
    def __init__(self, name):
        self.name = name

    async def tick(self):
        pass

    async def diagnose(self):
        return {}

    async def evolve(self):
        return []


class MockState:
    def __init__(self):
        self.dream_engine = MockPlugin("dream")
        self.identity_affirmer = MockPlugin("identity")
        self.self_pruner = MockPlugin("pruner")
        self.bond_tracker = MockPlugin("bond")
        self.chat_engine = MagicMock()
        self.chat_engine.emotion = MagicMock()
        self.chat_engine.emotion.pleasure = 0.5
        self.intent_drift_detector = MockPlugin("drift")
        self.audit_agent = MockPlugin("audit")
        self.memory_store = MagicMock()
        self.memory_store.db = AsyncMock()
        cursor_mock = AsyncMock()
        cursor_mock.fetchall = AsyncMock(return_value=[])
        self.memory_store.db.execute = AsyncMock(return_value=cursor_mock)
        self.thinking_evolver = MockPlugin("thinking")
        self.milestone_tracker = MockPlugin("milestone")
        self.milestone_tracker.check_milestones = MagicMock(return_value=[])
        self.bond_tracker.get_bond = MagicMock(return_value=0.5)
        self.last_interaction_time = time.time()
        self.start_time = time.time()
        self.heartbeat_count = 10


class TestPluginRegistration:
    async def test_register_all_plugins_succeeds(self, event_bus, hook_manager):
        state = MockState()
        register_all_plugin_hooks(event_bus, hook_manager, state)

    async def test_register_dream_engine_hooks(self, event_bus, hook_manager):
        state = MockState()
        from morn_core.eventbus.plugin_registry import register_dream_engine_hooks
        register_dream_engine_hooks(event_bus, hook_manager, state)
        received = []
        async def listener(event):
            received.append(event)
        event_bus.subscribe("heartbeat.minute", listener, "test_listener")
        await event_bus.publish(Event(
            type="heartbeat.minute", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1

    async def test_plugin_not_initialized_skips_safely(self, event_bus, hook_manager):
        state = MockState()
        state.dream_engine = None
        from morn_core.eventbus.plugin_registry import register_dream_engine_hooks
        register_dream_engine_hooks(event_bus, hook_manager, state)


class TestHookManagerLifecycle:
    async def test_register_pause_resume_unregister(self, event_bus, hook_manager):
        received = []
        async def handler(event):
            received.append(event)

        hook_manager.register(HookRegistration(
            plugin_id="test_plugin", event_type="test.event",
            callback=handler,
        ))

        await event_bus.publish(Event(
            type="test.event", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1

        hook_manager.pause_plugin("test_plugin")
        await event_bus.publish(Event(
            type="test.event", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1

        hook_manager.resume_plugin("test_plugin")
        await event_bus.publish(Event(
            type="test.event", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 2

        hook_manager.unregister("test_plugin", "test.event")
        await event_bus.publish(Event(
            type="test.event", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 2


class TestMemoryCapsuleWritten:
    async def test_memory_store_publishes_capsule_written(self, event_bus, data_dir):
        from morn_core.memory.store import MemoryStore
        store = MemoryStore(data_dir, event_bus=event_bus)
        await store.__aenter__()

        received = []
        async def listener(event):
            received.append(event)

        event_bus.subscribe("memory.capsule_written", listener, "capsule_watcher")
        capsule_id = await store.add_capsule({
            "description": "test capsule",
            "source": "test",
        })
        await asyncio.sleep(0.1)
        assert len(received) >= 1
        assert received[0].type == "memory.capsule_written"
        assert received[0].payload["capsule_id"] == capsule_id

        await store.close()

    async def test_memory_store_payload_includes_session_and_trust(self, event_bus, data_dir):
        from morn_core.memory.store import MemoryStore
        store = MemoryStore(data_dir, event_bus=event_bus)
        await store.__aenter__()

        received = []
        async def listener(event):
            received.append(event)

        event_bus.subscribe("memory.capsule_written", listener, "capsule_watcher")
        await store.add_capsule({
            "description": "capsule with full metadata",
            "session_id": "sess_001",
            "trust_level": "htz",
            "source": "chat",
        })
        await asyncio.sleep(0.1)
        assert len(received) >= 1
        payload = received[0].payload
        assert payload["session_id"] == "sess_001"
        assert payload["trust_level"] == "htz"
        assert payload["source"] == "chat"

        await store.close()

    async def test_memory_store_no_event_bus_skips(self, data_dir):
        from morn_core.memory.store import MemoryStore
        store = MemoryStore(data_dir)
        await store.__aenter__()
        capsule_id = await store.add_capsule({
            "description": "test capsule without event bus",
            "source": "test",
        })
        assert capsule_id is not None
        await store.close()


class TestHealthMonitor:
    async def test_self_check_no_warnings(self, event_bus, hook_manager):
        state = MockState()
        monitor = HealthMonitor(event_bus, hook_manager, state)
        monitor.register_hooks(hook_manager)

        warnings = []
        async def listener(event):
            warnings.append(event)

        event_bus.subscribe("kernel.health_warning", listener, "warning_watcher")
        await monitor.self_check(Event(
            type="heartbeat.minute", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(warnings) == 0

    async def test_clock_jump_detection(self, event_bus, hook_manager):
        state = MockState()
        monitor = HealthMonitor(event_bus, hook_manager, state)
        warnings = []
        async def listener(event):
            warnings.append(event)
        event_bus.subscribe("kernel.clock_jump", listener, "warning_watcher")
    
        monitor._last_time = time.time() - 10
        await monitor.detect_clock_jump(Event(
            type="heartbeat.tick", payload={}, source="test",
            priority=Priority.HIGH,
        ))
        await asyncio.sleep(0.05)
        assert len(warnings) == 1
        assert warnings[0].type == "kernel.clock_jump"
        assert "clock_jump" in warnings[0].payload

    async def test_deep_queue_triggers_warning(self, event_bus, hook_manager):
        state = MockState()
        monitor = HealthMonitor(event_bus, hook_manager, state)
        warnings = []
        async def listener(event):
            warnings.append(event)
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
        await asyncio.sleep(0.05)
        assert len(warnings) >= 1
        payload = warnings[0].payload
        assert "warnings" in payload
        assert any("queue.med" in w for w in payload["warnings"])


class TestFullRegistration:
    async def test_register_all_plus_health_and_reflection(self, event_bus, hook_manager):
        state = MockState()
        register_all_plugin_hooks(event_bus, hook_manager, state)

        health = HealthMonitor(event_bus, hook_manager, state)
        health.register_hooks(hook_manager)

        reflected = []
        async def on_minute(event):
            reflected.append(event)

        hook_manager.register(HookRegistration(
            plugin_id="self_reflection",
            event_type="heartbeat.minute",
            callback=on_minute,
            timeout=15.0,
        ))

        await event_bus.publish(Event(
            type="heartbeat.minute", payload={}, source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(reflected) >= 1


class TestImportCompleteness:
    def test_plugin_registry_imports(self):
        from morn_core.eventbus.plugin_registry import (
            register_all_plugin_hooks,
            register_dream_engine_hooks,
            register_identity_hooks,
            register_self_pruning_hooks,
            register_bond_update_hooks,
            register_intent_drift_hooks,
            register_audit_hooks,
            register_thinking_evolution_hooks,
            register_milestone_hooks,
        )

    def test_plugin_base_imports(self):
        from morn_core.eventbus.plugin_base import PluginInfo, register_plugin_hooks

    def test_health_monitor_imports(self):
        from morn_core.eventbus.health_monitor import HealthMonitor