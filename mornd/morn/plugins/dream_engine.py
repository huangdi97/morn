"""梦境引擎插件——空闲时重组记忆生成梦境叙事"""
import time
from morn.kernel.plugin import MornPlugin, PluginContext
from morn.kernel.hooks import HookRegistration
from morn.kernel.bus import Event, Priority


class DreamEnginePlugin(MornPlugin):
    plugin_id = "dream_engine"
    name = "梦境引擎"
    version = "0.1.0"
    plugin_class = "B"
    needs_periodic_trigger = True
    usage_hint = "low"

    def __init__(self):
        super().__init__()
        self.dream_engine = None

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        self._register_hooks()

    async def on_unload(self):
        await super().on_unload()

    async def _on_heartbeat_minute(self, event: Event) -> None:
        if not self.dream_engine:
            return
        idle = time.time() - self.context.config.get("_last_interaction_time", time.time())
        try:
            await self.dream_engine.tick(idle)
        except Exception as e:
            await self.context.event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "dream_engine", "error": str(e)},
                source="dream_engine", priority=Priority.HIGH,
            ))

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="dream_engine", event_type="heartbeat.minute",
            callback=self._on_heartbeat_minute, timeout=10.0,
        ))