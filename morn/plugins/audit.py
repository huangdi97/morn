"""审计插件——周期性从胶囊中提取知识三元组"""
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class AuditPlugin(MornPlugin):
    plugin_id = "audit"
    name = "审计"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    usage_hint = "low"
    dependencies = [PluginDependency("memory_store", "0.1.0")]
    required_permissions = ["memory.read", "audit.write"]
    optional_permissions = []
    health_check_interval = 120

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
        if self._counter < 10:
            return
        self._counter = 0
        if not self._state_ref or not self._state_ref.audit_agent or not self._state_ref.memory_store:
            return
        try:
            cursor = await self._state_ref.memory_store.db.execute(
                "SELECT * FROM capsules WHERE source NOT IN ('audit_agent', 'self_reflection') ORDER BY timestamp DESC LIMIT 10"
            )
            rows = await cursor.fetchall()
            for row in rows:
                cap = dict(row)
                count = await self._state_ref.audit_agent.extract_and_deposit(cap)
                if count:
                    await self.context.event_bus.publish(Event(
                        type="audit.triples_extracted",
                        payload={"capsule_id": cap.get("event_id"), "count": count},
                        source="audit_agent", priority=Priority.LOW,
                    ))
        except Exception as e:
            await self.context.event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "audit_agent", "error": str(e)},
                source="audit_agent", priority=Priority.HIGH,
            ))

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="audit", event_type="heartbeat.minute",
            callback=self._on_heartbeat_minute, timeout=10.0,
        ))