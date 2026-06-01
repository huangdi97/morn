"""插件加载器——扫描、验证、加载、卸载插件"""

import asyncio
import importlib
import inspect
import logging
from pathlib import Path
from typing import Optional

from .mcp_server import MCPServer
from .plugin import MornPlugin, PluginContext
from .plugin_contract import PluginContract, parse_contract
from .sandbox import get_sandbox_for

logger = logging.getLogger("morn.plugin")


class PluginLoader:
    def __init__(self, event_bus=None, hook_manager=None, mcp_server: Optional[MCPServer] = None,
                 contract_dir: Optional[Path] = None,
                 quota_manager: Optional["QuotaManager"] = None):
        self._plugins: dict[str, MornPlugin] = {}
        self._failed: list[tuple[str, str]] = []
        self._event_bus = event_bus
        self._hook_manager = hook_manager
        self._mcp_server = mcp_server
        self._contract_dir = Path(contract_dir) if contract_dir else None
        self._contracts: dict[str, PluginContract] = {}
        self._health_check_task: Optional[asyncio.Task] = None
        self._ping_failures: dict[str, int] = {}
        self._health_check_running = False
        self._quota_manager = quota_manager

    def discover_from_packages(self) -> list[type[MornPlugin]]:
        import importlib.metadata

        discovered = []
        eps = importlib.metadata.entry_points(group="morn.plugins")
        for ep in eps:
            try:
                plugin_class = ep.load()
                if (inspect.isclass(plugin_class) and issubclass(plugin_class, MornPlugin)
                        and not inspect.isabstract(plugin_class)):
                    discovered.append(plugin_class)
            except Exception:
                self._failed.append((ep.name, "failed to load entry point"))
        return discovered

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
        plugin_level = getattr(plugin, "plugin_class", "S")
        context = PluginContext(
            event_bus=self._event_bus,
            hook_manager=self._hook_manager,
            config=config,
            data_dir=data_dir,
            quota_manager=self._quota_manager,
        )

        sandbox = get_sandbox_for(plugin_level)
        if sandbox:
            try:
                async with sandbox:
                    await plugin.on_load(context)
            except PermissionError:
                self._failed.append((plugin_id, "seccomp violation during load"))
                if self._event_bus:
                    from .bus import Event, Priority
                    await self._event_bus.publish(
                        Event(
                            type="plugin.health_failed",
                            payload={"plugin_id": plugin_id, "reason": "seccomp_violation_during_load"},
                            source="plugin_loader",
                            priority=Priority.HIGH,
                        )
                    )
                raise
        else:
            await plugin.on_load(context)

        plugin._loaded = True
        self._plugins[plugin_id] = plugin

        contract = self._load_contract(plugin_id)
        if contract:
            self._contracts[plugin_id] = contract

        if self._mcp_server:
            self._mcp_server.register_plugin(plugin)

        return plugin

    async def unload(self, plugin_id: str) -> bool:
        plugin = self._plugins.pop(plugin_id, None)
        if plugin is None:
            return False
        self._ping_failures.pop(plugin_id, None)
        self._contracts.pop(plugin_id, None)
        if self._mcp_server:
            self._mcp_server.unregister_plugin(plugin_id)
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

    def _load_contract(self, plugin_id: str) -> Optional[PluginContract]:
        if self._contract_dir is None:
            return None
        candidates = [
            self._contract_dir / f"{plugin_id}.yaml",
            self._contract_dir / f"{plugin_id}.yml",
        ]
        for path in candidates:
            if path.exists():
                try:
                    return parse_contract(str(path))
                except Exception as e:
                    logger.warning("failed to parse contract %s: %s", path, e)
                    return None
        return None

    def get_contract(self, plugin_id: str) -> Optional[PluginContract]:
        return self._contracts.get(plugin_id)

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

    def start_health_checks(self):
        self._health_check_running = True
        if self._health_check_task is None or self._health_check_task.done():
            self._health_check_task = asyncio.create_task(self._health_check_loop())

    def stop_health_checks(self):
        self._health_check_running = False
        if self._health_check_task:
            self._health_check_task.cancel()
            self._health_check_task = None

    async def _health_check_loop(self):
        while self._health_check_running:
            await asyncio.sleep(10)
            for plugin_id, plugin in list(self._plugins.items()):
                if plugin.health_check_interval <= 0:
                    continue
                try:
                    ok = await asyncio.wait_for(plugin.health_check(), timeout=5.0)
                except (asyncio.TimeoutError, Exception):
                    ok = False
                if ok:
                    self._ping_failures.pop(plugin_id, None)
                else:
                    self._ping_failures[plugin_id] = self._ping_failures.get(plugin_id, 0) + 1
                    count = self._ping_failures[plugin_id]
                    logger.warning(
                        "health check failed for %s (%d/3)", plugin_id, count
                    )
                    if count >= 3:
                        logger.error("unloading %s after 3 failed health checks", plugin_id)
                        await self.unload(plugin_id)
                        if self._event_bus:
                            from .bus import Event, Priority
                            await self._event_bus.publish(
                                Event(
                                    type="plugin.health_failed",
                                    payload={"plugin_id": plugin_id, "reason": "3_consecutive_failures"},
                                    source="plugin_loader",
                                    priority=Priority.HIGH,
                                )
                            )