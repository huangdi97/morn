"""里程碑插件——周期性检查并庆祝成长里程碑"""
import time
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class MilestonePlugin(MornPlugin):
    plugin_id = "milestones"
    name = "里程碑"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    usage_hint = "low"
    dependencies = [PluginDependency("memory_store", "0.1.0"), PluginDependency("bond_tracker", "0.1.0")]
    required_permissions = ["memory.read", "milestone.write"]
    optional_permissions = []
    health_check_interval = 60

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
        if not self._state_ref or not hasattr(self._state_ref, 'milestone_tracker') or not self._state_ref.milestone_tracker:
            return
        try:
            mem_count = await self._state_ref.memory_store.count() if self._state_ref.memory_store else 0
            bond = self._state_ref.bond_tracker.get_bond() if self._state_ref.bond_tracker else 0.0
            days = (time.time() - self._state_ref.start_time) / 86400
            triggered = self._state_ref.milestone_tracker.check_milestones(
                memory_count=mem_count, bond_value=bond, days_since_birth=days
            )
            if triggered:
                await self._state_ref.milestone_tracker.push_greetings(triggered)
        except Exception as e:
            await self.context.event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "milestones", "error": str(e)},
                source="milestones", priority=Priority.HIGH,
            ))

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="milestones", event_type="heartbeat.minute",
            callback=self._on_heartbeat_minute, timeout=10.0,
        ))