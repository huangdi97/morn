"""自我瘦身插件——周期性诊断并清理冗余记忆和技能"""
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class SelfPrunerPlugin(MornPlugin):
    plugin_id = "self_pruner"
    name = "自我瘦身"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    usage_hint = "low"
    dependencies = [PluginDependency("memory_store", "0.1.0")]
    required_permissions = ["memory.read", "memory.write", "memory.delete"]
    optional_permissions = []
    health_check_interval = 120

    def __init__(self, state_ref=None):
        super().__init__()
        self._state_ref = state_ref
        self._counter = 0

    async def on_load(self, context):
        await super().on_load(context)
        async def on_minute(event):
            nonlocal self
            self._counter += 1
            if self._counter < 10:
                return
            self._counter = 0
            if not self._state_ref or not self._state_ref.self_pruner:
                return
            try:
                result = await self._state_ref.self_pruner.diagnose()
                if result.get("capsules_pruned", 0) or result.get("skills_pruned", 0) or result.get("emotion_pruned", 0):
                    await context.event_bus.publish(Event(
                        type="self_pruning.completed",
                        payload=result,
                        source="self_pruner", priority=Priority.LOW,
                    ))
            except Exception as e:
                await context.event_bus.publish(Event(
                    type="task.failed",
                    payload={"plugin": "self_pruner", "error": str(e)},
                    source="self_pruner", priority=Priority.HIGH,
                ))
        context.hook_manager.register(HookRegistration(
            plugin_id="self_pruner", event_type="heartbeat.minute",
            callback=on_minute, timeout=10.0,
        ))

    async def on_unload(self):
        await super().on_unload()