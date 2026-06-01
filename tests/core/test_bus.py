import asyncio


from morn.core.bus import Event, Priority


class TestEventBus:
    async def test_subscribe_and_publish(self, event_bus):
        received = []

        async def callback(event):
            received.append(event.payload)

        event_bus.subscribe("test.hello", callback, subscriber_id="test")
        await event_bus.publish(Event(
            type="test.hello",
            payload={"msg": "hello"},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1
        assert received[0]["msg"] == "hello"

    async def test_priority_order(self, event_bus):
        order = []

        async def high_cb(event):
            order.append("high")

        async def low_cb(event):
            order.append("low")

        event_bus.subscribe("test.priority", high_cb, subscriber_id="high")
        event_bus.subscribe("test.priority", low_cb, subscriber_id="low")
        await event_bus.publish(Event(
            type="test.priority",
            payload={},
            source="test",
            priority=Priority.HIGH,
        ))
        await asyncio.sleep(0.05)
        assert order == ["high", "low"]

    async def test_unsubscribe(self, event_bus):
        received = []

        async def callback(event):
            received.append(event.payload)

        event_bus.subscribe("test.unsub", callback, subscriber_id="unsub")
        event_bus.unsubscribe("test.unsub", "unsub")
        await event_bus.publish(Event(
            type="test.unsub",
            payload={"x": 1},
            source="test",
            priority=Priority.MEDIUM,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 0

    async def test_multiple_subscribers(self, event_bus):
        results = {1: [], 2: []}

        async def cb1(event):
            results[1].append(event.payload)

        async def cb2(event):
            results[2].append(event.payload)

        event_bus.subscribe("test.multi", cb1, subscriber_id="s1")
        event_bus.subscribe("test.multi", cb2, subscriber_id="s2")
        await event_bus.publish(Event(
            type="test.multi",
            payload={"n": 42},
            source="test",
            priority=Priority.LOW,
        ))
        await asyncio.sleep(0.05)
        assert results[1][0]["n"] == 42
        assert results[2][0]["n"] == 42

    async def test_get_stats(self, event_bus):
        for i in range(5):
            await event_bus.publish(Event(
                type="test.stats",
                payload={"i": i},
                source="test",
                priority=Priority.MEDIUM,
            ))
        await asyncio.sleep(0.05)
        stats = event_bus.get_stats()
        assert stats["published"] >= 5

    async def test_replay_events(self, tmp_path, event_bus):
        from morn.core.event_log import EventLog
        db_path = tmp_path / "test_replay.db"
        elog = EventLog(db_path)
        await elog.open()
        event_bus._event_log = elog

        await event_bus.publish(Event(
            type="test.replay",
            payload={"v": 1},
            source="test",
            priority=Priority.LOW,
        ))
        await event_bus.publish(Event(
            type="test.replay",
            payload={"v": 2},
            source="test",
            priority=Priority.LOW,
        ))
        await asyncio.sleep(0.05)

        replayed = []
        async def cb(event):
            replayed.append(event.payload)
        event_bus.subscribe("test.replay", cb, subscriber_id="replayer")

        count = await event_bus.replay_events(elog)
        await asyncio.sleep(0.05)
        assert count == 2
        assert len(replayed) == 2

        await elog.close()

    async def test_custom_event_type(self, event_bus):
        received = []

        async def cb(event):
            received.append(event.payload)

        event_bus.subscribe("my.custom.event", cb, subscriber_id="custom")
        await event_bus.publish(Event(
            type="my.custom.event",
            payload={"custom": True},
            source="test",
            priority=Priority.HIGH,
        ))
        await asyncio.sleep(0.05)
        assert len(received) == 1
        assert received[0]["custom"] is True