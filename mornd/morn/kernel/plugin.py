"""Morn 插件系统——PluginInfo（历史兼容）和 MornPlugin（抽象基类）"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional

from .hooks import HookRegistration


# ===== 历史兼容：PluginInfo =====

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


# ===== 新的插件系统 =====

@dataclass
class PluginDependency:
    plugin: str
    min_version: str
    optional: bool = False


class PluginContext:
    def __init__(self, event_bus=None, hook_manager=None, config=None, data_dir=None):
        self.event_bus = event_bus
        self.hook_manager = hook_manager
        self.config = config or {}
        self.data_dir = data_dir


class MornPlugin(ABC):
    plugin_id: str = ""
    name: str = ""
    version: str = "0.1.0"
    dependencies: list[PluginDependency] = []
    required_permissions: list[str] = []
    optional_permissions: list[str] = []
    needs_periodic_trigger: bool = False
    usage_hint: str = "low"
    plugin_class: str = ""

    def __init__(self):
        self.context: Optional[PluginContext] = None
        self._loaded = False

    @abstractmethod
    async def on_load(self, context: PluginContext):
        self.context = context
        self._loaded = True

    async def on_unload(self):
        self._loaded = False
        self.context = None

    async def on_event(self, event):
        pass

    async def on_heartbeat(self, tick: int):
        pass

    async def on_chat(self, message: str):
        pass

    @property
    def is_loaded(self) -> bool:
        return self._loaded

    def require_permission(self, permission: str) -> bool:
        return permission in self.required_permissions or \
               permission in self.optional_permissions
