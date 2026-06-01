"""自我反思插件——定期进行轻量自省并记录情感快照"""
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event


class SelfReflectionPlugin(MornPlugin):
    plugin_id = "self_reflection"
    name = "自我反思"
    version = "0.1.0"
    plugin_class = "B"
    needs_periodic_trigger = True
    usage_hint = "low"
    dependencies = [PluginDependency("memory_store", "0.1.0"), PluginDependency("chat_engine", "0.1.0")]
    required_permissions = ["memory.read", "emotion.read"]
    optional_permissions = ["memory.write"]
    health_check_interval = 60

    def __init__(self):
        super().__init__()
        self.self_reflection = None

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        self._register_hooks()

    async def on_unload(self):
        await super().on_unload()

    async def _on_heartbeat_minute(self, event: Event) -> None:
        if self.self_reflection:
            await self.self_reflection.light_reflection()

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="self_reflection",
            event_type="heartbeat.minute",
            callback=self._on_heartbeat_minute,
            timeout=15.0,
        ))