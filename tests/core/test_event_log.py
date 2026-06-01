

from morn.core.bus import Event, Priority
from morn.core.event_log import EventLog


class TestEventLog:
    async def test_write_and_read(self, tmp_path):
        db_path = tmp_path / "test_events.db"
        elog = EventLog(db_path)
        await elog.open()

        event = Event(type="test.event", payload={"x": 1}, source="test", priority=Priority.LOW)
        await elog.append(event)

        events = await elog.replay_since(0)
        assert len(events) == 1
        assert events[0].type == "test.event"
        assert events[0].payload == {"x": 1}

        await elog.close()

    async def test_list_events(self, tmp_path):
        db_path = tmp_path / "test_list.db"
        elog = EventLog(db_path)
        await elog.open()

        for i in range(5):
            event = Event(type="test.list", payload={"i": i}, source="test", priority=Priority.MEDIUM)
            await elog.append(event)

        events = await elog.replay_since(0)
        assert len(events) == 5

        await elog.close()

    async def test_clear(self, tmp_path):
        db_path = tmp_path / "test_clear.db"
        elog = EventLog(db_path)
        await elog.open()

        event = Event(type="test.clear", payload={}, source="test", priority=Priority.LOW)
        await elog.append(event)

        import aiosqlite
        async with aiosqlite.connect(str(db_path)) as conn:
            await conn.execute("DELETE FROM event_log")
            await conn.commit()

        events = await elog.replay_since(0)
        assert len(events) == 0

        await elog.close()