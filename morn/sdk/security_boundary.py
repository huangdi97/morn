import asyncio
import json
import logging
from pathlib import Path
from datetime import datetime, timezone
from typing import Optional

from morn.core.bus import Event, Priority

logger = logging.getLogger("morn.boundary")


class ExternalBoundary:
    def __init__(self, data_dir: Path, event_bus=None):
        self.data_dir = Path(data_dir) / "security"
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self._config_file = self.data_dir / "external_boundary.json"
        self._allowed_outbound: list[dict] = []
        self._blocked_ports: list[int] = []
        self._monitor_enabled = True
        self._connection_log: list[dict] = []
        self._event_bus = event_bus
        self._load_config()

    async def _publish_alert(self, message: str):
        if self._event_bus:
            await self._event_bus.publish(Event(
                type="security.alert",
                payload={"source": "external_boundary", "message": message},
                source="external_boundary",
                priority=Priority.HIGH,
            ))

    def _load_config(self):
        if self._config_file.exists():
            try:
                data = json.loads(self._config_file.read_text())
                self._allowed_outbound = data.get("allowed_outbound", [])
                self._blocked_ports = data.get("blocked_ports", [])
                self._monitor_enabled = data.get("monitor_enabled", True)
            except (json.JSONDecodeError, KeyError) as exc:
                logger.warning("external_boundary config corrupt, using defaults: %s", exc)
        else:
            self._save_config()

    def _save_config(self):
        self._config_file.write_text(
            json.dumps({
                "allowed_outbound": self._allowed_outbound,
                "blocked_ports": self._blocked_ports,
                "monitor_enabled": self._monitor_enabled,
            }, ensure_ascii=False, indent=2)
        )

    def _log_connection(self, direction: str, protocol: str, endpoint: str, allowed: bool):
        if not self._monitor_enabled:
            return
        self._connection_log.append({
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "direction": direction,
            "protocol": protocol,
            "endpoint": endpoint,
            "allowed": allowed,
        })

    def check_inbound(self, protocol: str, port: int, source: str) -> bool:
        allowed = False
        self._log_connection("inbound", protocol, f"{source}:{port}", allowed)
        if not allowed:
            try:
                loop = asyncio.get_running_loop()
                if loop.is_running():
                    asyncio.create_task(self._publish_alert(
                        f"入站连接被拒绝: {protocol} {source}:{port}"
                    ))
            except RuntimeError:
                pass
        return allowed

    def register_allowed_outbound(self, service: str, endpoint: str):
        entry = {"service": service, "endpoint": endpoint}
        if entry not in self._allowed_outbound:
            self._allowed_outbound.append(entry)
            self._save_config()

    def get_connection_log(self) -> list[dict]:
        return list(self._connection_log)

    def validate_port(self, port: int) -> bool:
        if port in self._blocked_ports:
            return False
        if port < 0 or port > 65535:
            return False
        return True
