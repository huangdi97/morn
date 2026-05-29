import time

from morn_core.eventbus.bus import EventBus, Event, Priority
from morn_core.eventbus.hooks import HookManager, HookRegistration


class HealthMonitor:
    def __init__(self, event_bus: EventBus, hook_manager: HookManager, state):
        self._event_bus = event_bus
        self._state = state
        self._last_time = 0.0

    async def self_check(self, event: Event) -> None:
        warnings = []

        for priority, queue in [
            ("high", self._event_bus._queues[Priority.HIGH]),
            ("med", self._event_bus._queues[Priority.MEDIUM]),
            ("low", self._event_bus._queues[Priority.LOW]),
        ]:
            depth = queue.qsize()
            if depth > 50:
                warnings.append(f"queue.{priority}: {depth} events pending")

        try:
            import psutil
            rss = psutil.Process().memory_info().rss
            mem_mb = rss / 1024 / 1024
            if mem_mb > 500:
                warnings.append(f"memory: {mem_mb:.0f}MB")
        except Exception:
            pass

        if warnings:
            await self._event_bus.publish(Event(
                type="kernel.health_warning",
                payload={"warnings": warnings},
                source="health_monitor",
                priority=Priority.HIGH,
            ))

    async def detect_clock_jump(self, event: Event) -> None:
        now = time.time()
        if self._last_time > 0:
            diff = abs(now - self._last_time)
            if diff > 5.0:
                await self._event_bus.publish(Event(
                    type="kernel.clock_jump",
                    payload={"clock_jump": diff, "message": f"clock jump detected: {diff:.1f}s"},
                    source="health_monitor",
                    priority=Priority.HIGH,
                ))
        self._last_time = now

    def register_hooks(self, hook_manager: HookManager) -> None:
        hook_manager.register(HookRegistration(
            plugin_id="health_monitor", event_type="heartbeat.minute",
            callback=self.self_check, timeout=5.0,
        ))
        hook_manager.register(HookRegistration(
            plugin_id="health_monitor", event_type="heartbeat.tick",
            callback=self.detect_clock_jump, timeout=1.0,
        ))