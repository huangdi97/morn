from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookManager


class ExampleHelloPlugin(MornPlugin):
    plugin_id = "example_hello"
    name = "示例 Hello"
    version = "0.1.0"
    plugin_class = "C"
    dependencies = [PluginDependency("memory_store", "0.1.0")]
    required_permissions = ["memory.read"]
    optional_permissions = []
    needs_periodic_trigger = False
    usage_hint = "low"
    health_check_interval = 60
    capabilities = [{"name": "example.say_hello", "description": "返回问候语"}]

    async def on_load(self, context: PluginContext):
        await super().on_load(context)

    async def on_unload(self):
        await super().on_unload()


class TestMornPlugin:
    async def test_plugin_load_unload(self, event_bus):
        plugin = ExampleHelloPlugin()
        hook_manager = HookManager(event_bus)
        context = PluginContext(hook_manager=hook_manager)
        await plugin.on_load(context)
        assert plugin.is_loaded is True
        await plugin.on_unload()
        assert plugin.is_loaded is False

    async def test_plugin_metadata(self):
        plugin = ExampleHelloPlugin()
        assert plugin.plugin_id == "example_hello"
        assert plugin.name == "示例 Hello"
        assert plugin.version == "0.1.0"
        assert plugin.plugin_class == "C"

    async def test_plugin_dependencies(self):
        dep = PluginDependency("memory_store", "0.1.0")
        assert dep.plugin == "memory_store"
        assert dep.min_version == "0.1.0"
        assert dep.optional is False

    async def test_plugin_context(self, event_bus):
        hook_manager = HookManager(event_bus)
        context = PluginContext(hook_manager=hook_manager)
        assert context.hook_manager is hook_manager