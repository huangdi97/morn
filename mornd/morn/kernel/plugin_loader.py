"""插件加载器——扫描、验证、加载、卸载插件"""

import importlib
import inspect
import logging
from pathlib import Path
from typing import Optional

from .plugin import MornPlugin, PluginContext


class PluginLoader:
    def __init__(self, event_bus=None, hook_manager=None):
        self._plugins: dict[str, MornPlugin] = {}
        self._failed: list[tuple[str, str]] = []
        self._event_bus = event_bus
        self._hook_manager = hook_manager

    def discover(self, search_paths: list[Path]) -> list[type[MornPlugin]]:
        discovered = []
        for search_path in search_paths:
            if not search_path.is_dir():
                continue
            for entry in search_path.iterdir():
                if entry.suffix != ".py" or entry.name.startswith("_"):
                    continue
                spec = importlib.util.spec_from_file_location(entry.stem, entry)
                if spec is None or spec.loader is None:
                    continue
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                for name, obj in inspect.getmembers(module, inspect.isclass):
                    if (issubclass(obj, MornPlugin) and obj is not MornPlugin
                            and not inspect.isabstract(obj)):
                        discovered.append(obj)
        return discovered

    async def load(self, plugin_class: type[MornPlugin],
                   config: Optional[dict] = None,
                   data_dir: Optional[Path] = None) -> MornPlugin:
        plugin_id = getattr(plugin_class, "plugin_id", plugin_class.__name__)
        if plugin_id in self._plugins:
            raise ValueError(f"Plugin '{plugin_id}' is already loaded")

        plugin = plugin_class()
        context = PluginContext(
            event_bus=self._event_bus,
            hook_manager=self._hook_manager,
            config=config,
            data_dir=data_dir,
        )
        await plugin.on_load(context)
        plugin._loaded = True
        self._plugins[plugin_id] = plugin
        return plugin

    async def unload(self, plugin_id: str) -> bool:
        plugin = self._plugins.pop(plugin_id, None)
        if plugin is None:
            return False
        await plugin.on_unload()
        return True

    def get_plugin(self, plugin_id: str) -> Optional[MornPlugin]:
        return self._plugins.get(plugin_id)

    def list_plugins(self) -> list[dict]:
        return [
            {
                "plugin_id": p.plugin_id,
                "name": p.name,
                "version": p.version,
                "loaded": p.is_loaded,
                "class": p.plugin_class,
            }
            for p in self._plugins.values()
        ]

    def list_failed(self) -> list[tuple[str, str]]:
        return list(self._failed)

    async def load_all(self, plugin_classes: list[type[MornPlugin]],
                       default_config: Optional[dict] = None,
                       data_dir: Optional[Path] = None) -> int:
        success = 0
        for cls in plugin_classes:
            try:
                await self.load(cls, default_config, data_dir)
                success += 1
            except Exception as e:
                plugin_id = getattr(cls, "plugin_id", cls.__name__)
                self._failed.append((plugin_id, str(e)))
                logging.getLogger("morn.plugin").error(f"Failed to load {plugin_id}: {e}")
        return success

    async def unload_all(self):
        for plugin_id in list(self._plugins.keys()):
            await self.unload(plugin_id)
