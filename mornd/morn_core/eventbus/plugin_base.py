from dataclasses import dataclass

from morn_core.eventbus.hooks import HookRegistration


@dataclass
class PluginInfo:
    plugin_id: str
    name: str
    version: str
    description: str
    hooks: list[HookRegistration]


def register_plugin_hooks(plugin: PluginInfo, event_bus, hook_manager, state) -> None:
    for hook in plugin.hooks:
        hook_manager.register(hook)