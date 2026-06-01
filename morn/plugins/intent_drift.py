"""意图漂移检测插件——周期性检测安全行为偏差"""
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event, Priority


class IntentDriftPlugin(MornPlugin):
    plugin_id = "intent_drift"
    name = "意图漂移检测"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    usage_hint = "low"
    dependencies = [PluginDependency("chat_engine", "0.1.0")]
    required_permissions = ["security.read", "intent.read"]
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
        if self._counter < 10:
            return
        self._counter = 0
        if not self._state_ref or not self._state_ref.intent_drift_detector:
            return
        try:
            alerts = self._state_ref.intent_drift_detector.check_drift()
            for alert in alerts:
                await self.context.event_bus.publish(Event(
                    type="security.alert",
                    payload=alert,
                    source="intent_drift_detector", priority=Priority.HIGH,
                ))
        except Exception as e:
            await self.context.event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "intent_drift_detector", "error": str(e)},
                source="intent_drift_detector", priority=Priority.HIGH,
            ))

    def _register_hooks(self):
        self.context.hook_manager.register(HookRegistration(
            plugin_id="intent_drift", event_type="heartbeat.minute",
            callback=self._on_heartbeat_minute, timeout=10.0,
        ))