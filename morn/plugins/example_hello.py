"""示例插件——展示 MornPlugin 完整生命周期

每个类属性对应一个 YAML 契约字段。
"""
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event


class ExampleHelloPlugin(MornPlugin):
    plugin_id = "example_hello"                                     # YAML: plugin_id
    name = "示例 Hello"                                              # YAML: name
    version = "0.1.0"                                               # YAML: version
    plugin_class = "C"                                              # YAML: class
    dependencies = [PluginDependency("memory_store", "0.1.0")]      # YAML: dependencies
    required_permissions = ["memory.read"]                          # YAML: required_permissions
    optional_permissions = []                                       # YAML: optional_permissions
    needs_periodic_trigger = False                                  # YAML: needs_periodic_trigger
    usage_hint = "low"                                              # YAML: usage_hint
    health_check_interval = 60                                      # YAML: health_check_interval
    capabilities = [                                                # YAML: capabilities -> MCP tools
        {"name": "example.say_hello", "description": "返回问候语"},
    ]

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        print(f"[ExampleHello] on_load: plugin_id={self.plugin_id}")

        context.hook_manager.register(HookRegistration(
            plugin_id=self.plugin_id,
            event_type="example.hello",
            callback=self.on_event,
            timeout=5.0,
        ))

    async def on_unload(self):
        print(f"[ExampleHello] on_unload: cleaning up")
        await super().on_unload()

    async def on_event(self, event: Event):
        if event.type == "example.hello":
            print(f"[ExampleHello] received event: {event.payload}")

    async def health_check(self) -> bool:
        return True
