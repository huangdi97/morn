"""Morn — 数字生命框架"""

__version__ = "0.1.0"

# —— 内核：直接导入（零外部依赖，无循环风险） ——
from morn.core import (
    EventBus, Event, Priority,
    HookManager, HookRegistration,
    PluginLoader, SecurityValidator,
    heartbeat_loop, memory_monitor,
)

# —— SDK + 高级组件：延迟导入 ——

def __getattr__(name):
    """延迟导入：from morn import ChatEngine → lazy import from morn.sdk"""
    if name in ("MornPlugin", "PluginContext", "PluginDependency"):
        from morn.core.plugin import (
            MornPlugin as _MornPlugin,
            PluginContext as _PluginContext,
            PluginDependency as _PluginDependency,
        )
        _lazy = {
            "MornPlugin": _MornPlugin,
            "PluginContext": _PluginContext,
            "PluginDependency": _PluginDependency,
        }
        return _lazy[name]
    if name in ("ChatEngine", "MemoryStore", "UserProtection", "MornPresence"):
        from morn.sdk import (
            ChatEngine as _ChatEngine,
            MemoryStore as _MemoryStore,
            UserProtection as _UserProtection,
            MornPresence as _MornPresence,
        )
        _lazy = {
            "ChatEngine": _ChatEngine,
            "MemoryStore": _MemoryStore,
            "UserProtection": _UserProtection,
            "MornPresence": _MornPresence,
        }
        return _lazy[name]
    if name in ("RawSnapshotStore", "ExternalMemoryAdapter", "GraphStore", "auto_extract"):
        from morn.contrib.memory_advanced import (
            RawSnapshotStore as _RawSnapshotStore,
            ExternalMemoryAdapter as _ExternalMemoryAdapter,
            GraphStore as _GraphStore,
            auto_extract as _auto_extract,
        )
        _lazy = {
            "RawSnapshotStore": _RawSnapshotStore,
            "ExternalMemoryAdapter": _ExternalMemoryAdapter,
            "GraphStore": _GraphStore,
            "auto_extract": _auto_extract,
        }
        return _lazy[name]
    if name in ("DynamicPermissions", "IntentDriftDetector", "APZStore"):
        from morn.contrib.security_advanced import (
            DynamicPermissions as _DynamicPermissions,
            IntentDriftDetector as _IntentDriftDetector,
            APZStore as _APZStore,
        )
        _lazy = {
            "DynamicPermissions": _DynamicPermissions,
            "IntentDriftDetector": _IntentDriftDetector,
            "APZStore": _APZStore,
        }
        return _lazy[name]
    raise AttributeError(f"module 'morn' has no attribute {name!r}")

__all__ = [
    "__version__",
    # 内核
    "EventBus", "Event", "Priority",
    "HookManager", "HookRegistration",
    "PluginLoader", "SecurityValidator",
    "MornPlugin", "PluginContext", "PluginDependency",
    "heartbeat_loop", "memory_monitor",
    # SDK（延迟加载）
    "ChatEngine", "MemoryStore", "UserProtection",
    "MornPresence",
    # 高级记忆（延迟加载）
    "RawSnapshotStore", "ExternalMemoryAdapter",
    "GraphStore", "auto_extract",
    # 高级安全（延迟加载）
    "DynamicPermissions", "IntentDriftDetector", "APZStore",
]
