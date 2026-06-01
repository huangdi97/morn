"""配置热重载——基于 asyncio + stat 轮询检测配置文件变更"""

import asyncio
import logging
import os
import time
from pathlib import Path
from typing import Callable, Optional

logger = logging.getLogger("morn.config_watcher")


class ConfigWatcher:
    def __init__(
        self,
        config_path: Path,
        publish_callback: Callable,
        poll_interval: float = 5.0,
        debounce_seconds: float = 1.0,
    ):
        self._config_path = config_path
        self._publish = publish_callback
        self._poll_interval = poll_interval
        self._debounce_seconds = debounce_seconds
        self._running = False
        self._task: Optional[asyncio.Task] = None
        self._cached_config: dict = {}
        self._cached_mtime: float = 0.0
        self._last_change_time: float = 0.0
        self._debounce_task: Optional[asyncio.Task] = None

    async def start(self):
        self._running = True
        if self._config_path.exists():
            self._cached_config = self._read_yaml_safe()
            self._cached_mtime = self._config_path.stat().st_mtime
        self._task = asyncio.create_task(self._poll_loop())

    async def stop(self):
        self._running = False
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        if self._debounce_task:
            self._debounce_task.cancel()
            try:
                await self._debounce_task
            except asyncio.CancelledError:
                pass

    def get_config(self) -> dict:
        return dict(self._cached_config)

    async def _poll_loop(self):
        while self._running:
            await asyncio.sleep(self._poll_interval)
            if not self._config_path.exists():
                continue
            try:
                mtime = self._config_path.stat().st_mtime
            except OSError:
                continue
            if mtime > self._cached_mtime:
                self._cached_mtime = mtime
                self._last_change_time = time.time()
                if self._debounce_task is None or self._debounce_task.done():
                    self._debounce_task = asyncio.create_task(self._debounce_then_reload())

    async def _debounce_then_reload(self):
        while True:
            await asyncio.sleep(self._debounce_seconds)
            if time.time() - self._last_change_time >= self._debounce_seconds:
                break
        await self._reload()

    async def _reload(self):
        new_config = self._read_yaml_safe()
        if new_config is None:
            await self._publish(
                self._make_event("config.parse_error", {"path": str(self._config_path)})
            )
            return
        changed_keys = self._diff_keys(self._cached_config, new_config)
        if not changed_keys:
            return
        old_config = self._cached_config
        self._cached_config = new_config
        logger.info("config reloaded: changed keys %s", changed_keys)
        await self._publish(
            self._make_event(
                "config.reloaded",
                {
                    "changed_keys": list(changed_keys),
                    "path": str(self._config_path),
                },
            )
        )
        risk_level_old = old_config.get("risk_level")
        risk_level_new = new_config.get("risk_level")
        perms_old = old_config.get("permissions")
        perms_new = new_config.get("permissions")
        if risk_level_old != risk_level_new or perms_old != perms_new:
            logger.info("security-relevant config changed, publishing security.update")
            await self._publish(
                self._make_event("security.update", {"path": str(self._config_path)})
            )

    def _read_yaml_safe(self) -> Optional[dict]:
        try:
            import yaml
            with open(self._config_path, "r") as f:
                return yaml.safe_load(f) or {}
        except Exception as exc:
            logger.warning("failed to parse config yaml: %s", exc)
            return None

    def _diff_keys(self, old: dict, new: dict) -> set:
        all_keys = set(old.keys()) | set(new.keys())
        return {k for k in all_keys if old.get(k) != new.get(k)}

    def _make_event(self, event_type: str, payload: dict):
        from .bus import Event, Priority
        return Event(
            type=event_type,
            payload=payload,
            source="config_watcher",
            priority=Priority.HIGH,
        )