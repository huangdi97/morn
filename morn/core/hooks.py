from dataclasses import dataclass
from typing import Callable, Optional

from .bus import EventBus, Priority


@dataclass
class HookRegistration:
    plugin_id: str
    event_type: str
    callback: Callable
    timeout: float = 0.5
    enabled: bool = True


class HookManager:
    def __init__(self, event_bus: EventBus):
        self._event_bus = event_bus
        self._registrations: dict[str, HookRegistration] = {}
        self._paused: set[str] = set()

    def register(self, hook: HookRegistration) -> None:
        key = f"{hook.plugin_id}:{hook.event_type}"
        self._registrations[key] = hook
        self._paused.discard(hook.plugin_id)

        def wrapped(event):
            if hook.plugin_id in self._paused or not hook.enabled:
                return
            return hook.callback(event)

        self._event_bus.subscribe(
            event_type=hook.event_type,
            callback=wrapped,
            subscriber_id=key,
        )

    def unregister(self, plugin_id: str, event_type: str) -> None:
        key = f"{plugin_id}:{event_type}"
        self._registrations.pop(key, None)
        self._event_bus.unsubscribe(event_type, key)

    def pause_plugin(self, plugin_id: str) -> None:
        self._paused.add(plugin_id)

    def resume_plugin(self, plugin_id: str) -> None:
        self._paused.discard(plugin_id)
