"""后见之明插件——每小时回顾情感并触发反思"""
from morn.kernel.plugin import MornPlugin, PluginContext
from morn.kernel.hooks import HookRegistration
from morn.kernel.bus import Event, Priority


class HindsightPlugin(MornPlugin):
    plugin_id = "hindsight"
    name = "后见之明"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    usage_hint = "low"

    def __init__(self, state_ref=None):
        super().__init__()
        self._state_ref = state_ref

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        self._register_hooks()

    async def on_unload(self):
        await super().on_unload()

    async def _on_heartbeat_hour(self, event: Event) -> None:
        if not self._state_ref or not hasattr(self._state_ref, 'hindsight_engine') or not self._state_ref.hindsight_engine:
            return
        if not self._state_ref.chat_engine:
            return
        try:
            emotion = self._state_ref.chat_engine.emotion
            triggered = await self._state_ref.hindsight_engine.tick(emotion)
            if triggered:
                await self.context.event_bus.publish(Event(
                    type="hindsight.triggered",
                    payload={"count": len(triggered)},
                    source="hindsight_engine",
                    priority=Priority.LOW,
                ))
        except Exception as e:
            await self.context.event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "hindsight_engine", "error": str(e)},
                source="hindsight_engine",
                priority=Priority.HIGH,
            ))

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="hindsight",
            event_type="heartbeat.hour",
            callback=self._on_heartbeat_hour,
            timeout=30.0,
        ))