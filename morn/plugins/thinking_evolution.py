"""思维演化插件——长时间空闲时演化思维模式"""
import time
from morn.core.plugin import MornPlugin, PluginContext
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class ThinkingEvolutionPlugin(MornPlugin):
    plugin_id = "thinking_evolution"
    name = "思维演化"
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
        if not self._state_ref or not hasattr(self._state_ref, 'thinking_evolver') or not self._state_ref.thinking_evolver:
            return
        idle = time.time() - self._state_ref.last_interaction_time
        if idle < 3600:
            return
        try:
            ev_events = self._state_ref.thinking_evolver.evolve()
            if ev_events:
                await self.context.event_bus.publish(Event(
                    type="thinking.evolved",
                    payload={"events": ev_events},
                    source="thinking_evolver", priority=Priority.LOW,
                ))
        except Exception as e:
            await self.context.event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "thinking_evolver", "error": str(e)},
                source="thinking_evolver", priority=Priority.HIGH,
            ))

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="thinking_evolution", event_type="heartbeat.hour",
            callback=self._on_heartbeat_hour, timeout=30.0,
        ))