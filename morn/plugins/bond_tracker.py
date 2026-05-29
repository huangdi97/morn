"""连接追踪插件——周期性更新情感连接深度"""
import time
from morn.core.plugin import MornPlugin, PluginContext
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class BondTrackerPlugin(MornPlugin):
    plugin_id = "bond_tracker"
    name = "连接追踪"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    usage_hint = "low"

    def __init__(self, state_ref=None):
        super().__init__()
        self._state_ref = state_ref
        self._counter = 0

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        self._register_hooks()

    async def on_unload(self):
        await super().on_unload()

    async def _on_heartbeat_minute(self, event: Event) -> None:
        self._counter += 1
        if self._counter < 5:
            return
        self._counter = 0
        if not self._state_ref or not self._state_ref.bond_tracker or not self._state_ref.chat_engine:
            return
        try:
            idle = time.time() - self._state_ref.last_interaction_time
            depth = min(self._state_ref.heartbeat_count / 100, 1.0)
            sentiment = self._state_ref.chat_engine.emotion.pleasure
            days = (time.time() - self._state_ref.start_time) / 86400
            self._state_ref.bond_tracker.update(depth, sentiment, days)
            self._state_ref.bond_tracker.save()
        except Exception as e:
            await self.context.event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "bond_tracker", "error": str(e)},
                source="bond_tracker", priority=Priority.HIGH,
            ))

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="bond_tracker", event_type="heartbeat.minute",
            callback=self._on_heartbeat_minute, timeout=10.0,
        ))