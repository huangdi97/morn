"""HealthMonitor 插件——系统健康监控"""
import time
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class HealthMonitorPlugin(MornPlugin):
    plugin_id = "health_monitor"
    name = "健康监控"
    version = "0.1.0"
    plugin_class = "S"
    needs_periodic_trigger = True
    usage_hint = "low"
    dependencies = []
    required_permissions = ["system.memory.read", "system.process.read"]
    optional_permissions = []
    health_check_interval = 30

    def __init__(self):
        super().__init__()
        self._last_time = 0.0

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        self._register_hooks()

    async def on_unload(self):
        await super().on_unload()

    async def _self_check(self, event: Event) -> None:
        warnings = []
        for priority, queue in [
            ("high", self.context.event_bus._queues[Priority.HIGH]),
            ("med", self.context.event_bus._queues[Priority.MEDIUM]),
            ("low", self.context.event_bus._queues[Priority.LOW]),
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
            await self.context.event_bus.publish(Event(
                type="kernel.health_warning",
                payload={"warnings": warnings},
                source="health_monitor",
                priority=Priority.HIGH,
            ))

    async def _detect_clock_jump(self, event: Event) -> None:
        now = time.time()
        if self._last_time > 0:
            diff = abs(now - self._last_time)
            if diff > 5.0:
                await self.context.event_bus.publish(Event(
                    type="kernel.clock_jump",
                    payload={"clock_jump": diff, "message": f"clock jump detected: {diff:.1f}s"},
                    source="health_monitor",
                    priority=Priority.HIGH,
                ))
        self._last_time = now

    def _register_hooks(self):
        hm = self.context.hook_manager
        hm.register(HookRegistration(
            plugin_id="health_monitor", event_type="heartbeat.minute",
            callback=self._self_check, timeout=5.0,
        ))
        hm.register(HookRegistration(
            plugin_id="health_monitor", event_type="heartbeat.tick",
            callback=self._detect_clock_jump, timeout=1.0,
        ))