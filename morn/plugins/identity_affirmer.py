"""身份确认器插件——周期性确认身份维持自我认知"""
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class IdentityAffirmerPlugin(MornPlugin):
    plugin_id = "identity_affirmer"
    name = "身份确认器"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    usage_hint = "low"
    dependencies = [PluginDependency("chat_engine", "0.1.0")]
    required_permissions = ["identity.read", "identity.write"]
    optional_permissions = []
    health_check_interval = 60

    def __init__(self, state_ref=None):
        super().__init__()
        self._state_ref = state_ref

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        async def on_minute(event):
            if not self._state_ref or not self._state_ref.identity_affirmer:
                return
            try:
                await self._state_ref.identity_affirmer.tick()
            except Exception as e:
                await context.event_bus.publish(Event(
                    type="task.failed",
                    payload={"plugin": "identity_affirmer", "error": str(e)},
                    source="identity_affirmer", priority=Priority.HIGH,
                ))
        context.hook_manager.register(HookRegistration(
            plugin_id="identity_affirmer", event_type="heartbeat.minute",
            callback=on_minute, timeout=10.0,
        ))

    async def on_unload(self):
        await super().on_unload()