import asyncio
import enum
import logging
import time
import uuid
from collections import deque
from dataclasses import dataclass, field
from typing import Callable, Optional

logger = logging.getLogger("morn.eventbus")


class Priority(enum.Enum):
    HIGH = 0
    MEDIUM = 1
    LOW = 2


@dataclass
class Event:
    type: str
    payload: dict
    source: str
    priority: Priority
    timestamp: float
    event_id: str

    def __init__(
        self,
        type: str,
        payload: dict,
        source: str,
        priority: Priority,
        timestamp: Optional[float] = None,
        event_id: Optional[str] = None,
    ):
        self.type = type
        self.payload = payload
        self.source = source
        self.priority = priority
        self.timestamp = timestamp if timestamp is not None else time.time()
        self.event_id = event_id if event_id is not None else uuid.uuid4().hex


@dataclass
class SubscriberInfo:
    callback: Callable
    subscriber_id: str
    priority_filter: Optional[Priority] = None
    consecutive_timeout_count: int = 0
    suspended: bool = False
    _outcomes: deque = field(default_factory=lambda: deque(maxlen=3))


@dataclass
class BusStats:
    published: int = 0
    consumed: int = 0
    dropped: int = 0
    timeouts: int = 0


class _Queue:
    def __init__(self):
        self._items: deque[Event] = deque()

    def put_nowait(self, event: Event) -> None:
        self._items.append(event)

    def get_nowait(self) -> Event:
        if not self._items:
            raise asyncio.QueueEmpty
        return self._items.popleft()

    def age_out(self, max_age: float = 60.0, keep: int = 100) -> int:
        now = time.time()
        old_len = len(self._items)
        valid = [e for e in self._items if now - e.timestamp <= max_age]
        if len(valid) > keep:
            valid = valid[-keep:]
        self._items = deque(valid)
        return old_len - len(self._items)

    def qsize(self) -> int:
        return len(self._items)

    def peek_oldest_timestamp(self) -> Optional[float]:
        if not self._items:
            return None
        return self._items[0].timestamp


class EventBus:
    def __init__(self, loop: asyncio.AbstractEventLoop, event_log=None):
        self._loop = loop
        self._event_log = event_log
        self._replaying = False
        self._queues = {
            Priority.HIGH: _Queue(),
            Priority.MEDIUM: _Queue(),
            Priority.LOW: _Queue(),
        }
        self._subscribers: dict[str, list[SubscriberInfo]] = {}
        self._running = False
        self._stats = BusStats()
        self._backpressure: dict[str, dict] = {}
        self._dispatch_task: Optional[asyncio.Task] = None
        self._age_out_task: Optional[asyncio.Task] = None
        self._wakeup = asyncio.Event()

    async def publish(self, event: Event) -> None:
        if not self._replaying and self._event_log:
            await self._event_log.append(event)
        self._queues[event.priority].put_nowait(event)
        self._stats.published += 1
        self._wakeup.set()

    def subscribe(
        self,
        event_type: str,
        callback: Callable,
        subscriber_id: str,
        priority_filter: Optional[Priority] = None,
    ) -> None:
        if event_type not in self._subscribers:
            self._subscribers[event_type] = []
        info = SubscriberInfo(
            callback=callback,
            subscriber_id=subscriber_id,
            priority_filter=priority_filter,
        )
        existing = [
            i
            for i in self._subscribers[event_type]
            if i.subscriber_id == subscriber_id
        ]
        if existing:
            idx = self._subscribers[event_type].index(existing[0])
            self._subscribers[event_type][idx] = info
        else:
            self._subscribers[event_type].append(info)

    def unsubscribe(self, event_type: str, subscriber_id: str) -> None:
        if event_type not in self._subscribers:
            return
        self._subscribers[event_type] = [
            i
            for i in self._subscribers[event_type]
            if i.subscriber_id != subscriber_id
        ]
        if not self._subscribers[event_type]:
            del self._subscribers[event_type]

    async def start(self) -> None:
        self._running = True
        self._dispatch_task = asyncio.create_task(self._dispatch_loop())
        self._age_out_task = asyncio.create_task(self._age_out_loop())

    async def stop(self) -> None:
        self._running = False
        if self._dispatch_task:
            self._dispatch_task.cancel()
            try:
                await self._dispatch_task
            except asyncio.CancelledError:
                pass
        if self._age_out_task:
            self._age_out_task.cancel()
            try:
                await self._age_out_task
            except asyncio.CancelledError:
                pass

    def get_stats(self) -> dict:
        return {
            "published": self._stats.published,
            "consumed": self._stats.consumed,
            "dropped": self._stats.dropped,
            "timeouts": self._stats.timeouts,
            "queue_depth_high": self._queues[Priority.HIGH].qsize(),
            "queue_depth_medium": self._queues[Priority.MEDIUM].qsize(),
            "queue_depth_low": self._queues[Priority.LOW].qsize(),
        }

    async def replay_events(self, event_log, after_rowid: int = 0) -> int:
        self._replaying = True
        try:
            events = await event_log.replay_since(after_rowid)
            for event in events:
                await self.publish(event)
            return len(events)
        finally:
            self._replaying = False

    async def _dispatch_loop(self) -> None:
        while self._running:
            event = None
            for priority in (Priority.HIGH, Priority.MEDIUM, Priority.LOW):
                try:
                    event = self._queues[priority].get_nowait()
                    break
                except asyncio.QueueEmpty:
                    continue
            if event is None:
                self._wakeup.clear()
                any_items = any(
                    self._queues[p].qsize() > 0
                    for p in (Priority.HIGH, Priority.MEDIUM, Priority.LOW)
                )
                if not any_items:
                    try:
                        await asyncio.wait_for(self._wakeup.wait(), timeout=0.1)
                    except asyncio.TimeoutError:
                        pass
                continue
            self._stats.consumed += 1
            await self._dispatch_event(event)

    async def _dispatch_event(self, event: Event) -> None:
        subscribers = self._subscribers.get(event.type, [])
        if not subscribers:
            return
        tasks = []
        for info in subscribers:
            if info.suspended:
                continue
            if info.priority_filter is not None and info.priority_filter != event.priority:
                continue
            tasks.append(self._call_subscriber(info, event))
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)

    async def _call_subscriber(self, info: SubscriberInfo, event: Event) -> None:
        try:
            await asyncio.wait_for(info.callback(event), timeout=0.5)
            info.consecutive_timeout_count = 0
            info._outcomes.append(False)
        except asyncio.TimeoutError:
            info.consecutive_timeout_count += 1
            info._outcomes.append(True)
            self._stats.timeouts += 1
            logger.warning(
                "subscriber %s timed out on %s (consecutive: %d)",
                info.subscriber_id,
                event.type,
                info.consecutive_timeout_count,
            )
            await self.publish(
                Event(
                    type="task.failed",
                    payload={
                        "subscriber_id": info.subscriber_id,
                        "event_type": event.type,
                        "reason": "timeout",
                    },
                    source="eventbus",
                    priority=Priority.HIGH,
                )
            )
            if sum(info._outcomes) >= 2:
                info.suspended = True
                logger.warning(
                    "subscriber %s suspended due to backpressure",
                    info.subscriber_id,
                )
                await self.publish(
                    Event(
                        type="event.dropped",
                        payload={
                            "subscriber_id": info.subscriber_id,
                            "event_type": event.type,
                            "reason": "subscriber_suspended",
                            "consecutive_timeouts": info.consecutive_timeout_count,
                        },
                        source="eventbus",
                        priority=Priority.HIGH,
                    )
                )
        except asyncio.CancelledError:
            info.consecutive_timeout_count += 1
            info._outcomes.append(True)
            self._stats.timeouts += 1
            await self.publish(
                Event(
                    type="task.failed",
                    payload={
                        "subscriber_id": info.subscriber_id,
                        "event_type": event.type,
                        "reason": "cancelled",
                    },
                    source="eventbus",
                    priority=Priority.HIGH,
                )
            )
        except Exception as e:
            logger.error(
                "subscriber %s error on %s: %s",
                info.subscriber_id,
                event.type,
                e,
            )

    async def _age_out_loop(self) -> None:
        while self._running:
            await asyncio.sleep(10)
            for priority in (Priority.HIGH, Priority.MEDIUM, Priority.LOW):
                q = self._queues[priority]
                oldest = q.peek_oldest_timestamp()
                if oldest is None:
                    continue
                if time.time() - oldest > 60:
                    dropped = q.age_out(max_age=60, keep=100)
                    if dropped > 0:
                        self._stats.dropped += dropped
                        logger.info(
                            "aged out %d events from %s queue",
                            dropped,
                            priority.name,
                        )
                        await self.publish(
                            Event(
                                type="event.dropped",
                                payload={
                                    "priority": priority.name,
                                    "count": dropped,
                                    "reason": "queue_backpressure",
                                },
                                source="eventbus",
                                priority=Priority.HIGH,
                            )
                        )