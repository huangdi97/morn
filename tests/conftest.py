import asyncio
import pytest

from morn.core.bus import Event, EventBus, Priority


@pytest.fixture
async def event_bus():
    loop = asyncio.get_running_loop()
    bus = EventBus(loop)
    await bus.start()
    yield bus
    await bus.stop()