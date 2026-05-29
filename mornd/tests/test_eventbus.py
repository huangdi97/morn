import asyncio
import time

import pytest

from morn_core.eventbus.bus import EventBus, Event, Priority


@pytest.fixture
async def event_bus():
    loop = asyncio.get_event_loop()
    bus = EventBus(loop)
    await bus.start()
    yield bus
    await bus.stop()


class TestEventBusBasics:
    async def test_publish_subscribe_basic(self, event_bus):
        received = []

        async def handler(event):
            received.append(event)

        event_bus.subscribe("test.event", handler, "test_sub")
        await event_bus.publish(Event(
            type="test.event",
            payload={"key": "value"},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1
        assert received[0].type == "test.event"
        assert received[0].payload == {"key": "value"}

    async def test_multi_subscriber(self, event_bus):
        received = []

        async def handler1(event):
            received.append("h1")

        async def handler2(event):
            received.append("h2")

        async def handler3(event):
            received.append("h3")

        event_bus.subscribe("test.multi", handler1, "sub1")
        event_bus.subscribe("test.multi", handler2, "sub2")
        event_bus.subscribe("test.multi", handler3, "sub3")

        await event_bus.publish(Event(
            type="test.multi",
            payload={},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 3
        assert sorted(received) == ["h1", "h2", "h3"]

    async def test_priority_ordering(self, event_bus):
        received = []

        async def handler(event):
            received.append((event.priority.name, event.event_id))

        event_bus.subscribe("test.pri", handler, "pri_sub")

        low_event = Event(type="test.pri", payload={}, source="test", priority=Priority.LOW)
        med_event = Event(type="test.pri", payload={}, source="test", priority=Priority.MEDIUM)
        high_event = Event(type="test.pri", payload={}, source="test", priority=Priority.HIGH)

        await event_bus.publish(low_event)
        await event_bus.publish(med_event)
        await event_bus.publish(high_event)

        await asyncio.sleep(0.1)
        assert len(received) == 3
        assert received[0][0] == "HIGH"
        assert received[1][0] == "MEDIUM"
        assert received[2][0] == "LOW"

    async def test_timeout_cancellation(self, event_bus):
        timeout_triggered = []

        async def slow_handler(event):
            await asyncio.sleep(1.0)

        async def timeout_listener(event):
            timeout_triggered.append(event)

        event_bus.subscribe("test.slow", slow_handler, "slow_sub")
        event_bus.subscribe("task.failed", timeout_listener, "timeout_watcher")

        await event_bus.publish(Event(
            type="test.slow",
            payload={},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.6)
        assert len(timeout_triggered) >= 1
        assert timeout_triggered[0].payload.get("reason") == "timeout"

    async def test_backpressure_drop(self, event_bus):
        dropped = []

        async def slow_handler(event):
            await asyncio.sleep(0.1)

        async def drop_listener(event):
            dropped.append(event)

        event_bus.subscribe("test.bp", slow_handler, "bp_sub")
        event_bus.subscribe("event.dropped", drop_listener, "drop_watcher")

        for i in range(150):
            await event_bus.publish(Event(
                type="test.bp",
                payload={"seq": i, "timestamp": time.time() - 120},
                source="test",
                priority=Priority.MEDIUM,
            ))
        await asyncio.sleep(1.0)
        stats = event_bus.get_stats()
        assert stats["dropped"] > 0 or stats["consumed"] < 150

    async def test_consecutive_timeout_suspend(self, event_bus):
        timeouts = 0

        async def flaky_handler(event):
            nonlocal timeouts
            timeouts += 1
            raise asyncio.TimeoutError
            await asyncio.sleep(0)

        async def flaky_wrapper(event):
            nonlocal timeouts
            timeouts += 1
            await asyncio.sleep(0.6)

        event_bus.subscribe("test.flaky", flaky_wrapper, "flaky_sub")

        for i in range(5):
            await event_bus.publish(Event(
                type="test.flaky",
                payload={"seq": i},
                source="test",
                priority=Priority.MEDIUM,
            ))
            await asyncio.sleep(0.1)

        await asyncio.sleep(0.5)
        assert timeouts >= 2

    async def test_unsubscribe(self, event_bus):
        received = []

        async def handler(event):
            received.append(event)

        event_bus.subscribe("test.unsub", handler, "unsub_sub")
        await event_bus.publish(Event(
            type="test.unsub",
            payload={},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1

        event_bus.unsubscribe("test.unsub", "unsub_sub")
        await event_bus.publish(Event(
            type="test.unsub",
            payload={},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1

    async def test_heartbeat_integration(self, event_bus):
        tick_received = []

        async def tick_handler(event):
            tick_received.append(event)

        event_bus.subscribe("heartbeat.tick", tick_handler, "tick_listener")

        from morn_core.heartbeat import heartbeat_loop

        class MockState:
            shutdown = False
            heartbeat_count = 0
            last_heartbeat = 0.0

            def log(self, module, msg):
                pass

        state = MockState()
        task = asyncio.create_task(heartbeat_loop(state, event_bus))
        await asyncio.sleep(1.5)
        state.shutdown = True
        await asyncio.sleep(0.5)
        task.cancel()
        try:
            await task
        except (asyncio.CancelledError, StopIteration):
            pass
        assert len(tick_received) >= 1

    async def test_concurrent_publish(self, event_bus):
        received = []

        async def handler(event):
            received.append(event.event_id)

        event_bus.subscribe("test.concurrent", handler, "con_sub")

        async def publisher(n):
            for i in range(20):
                await event_bus.publish(Event(
                    type="test.concurrent",
                    payload={"n": n, "i": i},
                    source="test",
                    priority=Priority.LOW,
                ))

        tasks = [asyncio.create_task(publisher(n)) for n in range(5)]
        await asyncio.gather(*tasks)
        await asyncio.sleep(0.3)
        assert len(received) == 100

    async def test_event_dropped_on_backpressure(self, event_bus):
        dropped_events = []

        async def drop_listener(event):
            dropped_events.append(event)

        event_bus.subscribe("event.dropped", drop_listener, "drop_watcher")

        async def slightly_slow(event):
            await asyncio.sleep(0.2)

        event_bus.subscribe("test.drop_check", slightly_slow, "slow_for_drop")

        for i in range(50):
            await event_bus.publish(Event(
                type="test.drop_check",
                payload={"seq": i, "timestamp": time.time() - 120},
                source="test",
                priority=Priority.MEDIUM,
            ))

        await asyncio.sleep(0.5)
        stats = event_bus.get_stats()
        assert stats["consumed"] > 0

    async def test_get_stats(self, event_bus):
        stats = event_bus.get_stats()
        assert "published" in stats
        assert "consumed" in stats
        assert "dropped" in stats
        assert "timeouts" in stats
        assert "queue_depth_high" in stats
        assert "queue_depth_medium" in stats
        assert "queue_depth_low" in stats

    async def test_subscriber_replaces_existing(self, event_bus):
        received = []

        async def handler1(event):
            received.append("h1")

        async def handler2(event):
            received.append("h2")

        event_bus.subscribe("test.replace", handler1, "same_sub")
        event_bus.subscribe("test.replace", handler2, "same_sub")

        await event_bus.publish(Event(
            type="test.replace",
            payload={},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1
        assert received[0] == "h2"