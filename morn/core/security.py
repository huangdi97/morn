import asyncio
import hashlib
import json
import logging
import os
import re
import threading
import time
from dataclasses import dataclass, field
from typing import Optional

from .bus import EventBus, Event, Priority
from .rules import get_all_rules

logger = logging.getLogger("morn.security")


@dataclass
class ValidationResult:
    action: str
    reason: str
    suggested_level: str
    rule_id: Optional[str] = None


RISK_ORDER = {"green": 0, "yellow": 1, "orange": 2, "red": 3, "black": 4}


class SecurityValidator:
    def __init__(self, config: dict, event_bus: Optional[EventBus] = None):
        self._config = config
        self._event_bus = event_bus
        self._rules = get_all_rules()
        self._risk_levels = self._load_risk_config(config)
        self._stats = {"allowed": 0, "blocked": 0, "confirmed": 0}
        self._last_reload = time.time()
        self._config_path = None
        self._lock = threading.Lock()
        self._last_config_hash = None

    def _load_risk_config(self, config: dict) -> dict:
        return config.get("risk_levels", {})

    def set_config_path(self, path: str) -> None:
        self._config_path = path

    def validate(self, action_type: str, params: dict,
                 source_plugin: str, risk_level: str,
                 risk_preference: str) -> ValidationResult:
        risk_score = RISK_ORDER.get(risk_level, 1)
        pref_score = RISK_ORDER.get(risk_preference, 1)

        if risk_level == "black":
            with self._lock:
                self._stats["blocked"] += 1
            return ValidationResult("block", "绝对禁区：操作已被永久拦截", "black")

        params_str = json.dumps(params, ensure_ascii=False)
        for rule in self._rules:
            if re.search(rule.pattern, params_str, re.IGNORECASE):
                if rule.action_on_match == "block":
                    with self._lock:
                        self._stats["blocked"] += 1
                    return ValidationResult("block", rule.description, risk_level, rule.rule_id)

        plugin_permissions = self._config.get("plugin_permissions", {})
        allowed_actions = plugin_permissions.get(source_plugin, [])
        if allowed_actions and action_type not in allowed_actions:
            with self._lock:
                self._stats["blocked"] += 1
            return ValidationResult(
                "block",
                f"插件 {source_plugin} 无权执行 {action_type}",
                risk_level,
            )

        if risk_level == "green":
            with self._lock:
                self._stats["allowed"] += 1
            return ValidationResult("allow", "安全操作，自动放行", "green")

        if risk_level == "red":
            with self._lock:
                self._stats["blocked"] += 1
            return ValidationResult("block", f"高风险操作 ({risk_level})，已拦截", risk_level)

        if risk_level == "orange":
            if risk_score <= pref_score:
                with self._lock:
                    self._stats["confirmed"] += 1
                return ValidationResult("confirm", "中风险操作，需要创建者确认", risk_level)
            else:
                with self._lock:
                    self._stats["blocked"] += 1
                return ValidationResult("block", f"操作风险({risk_level})超出创建者偏好({risk_preference})", risk_level)

        with self._lock:
            self._stats["allowed"] += 1
        return ValidationResult("allow", f"低风险操作 ({risk_level})，已放行", risk_level)

    async def publish_alert(self, result: ValidationResult,
                            source_plugin: str,
                            risk_level: str) -> None:
        if self._event_bus:
            await self._event_bus.publish(Event(
                type="security.alert",
                payload={
                    "action": result.action,
                    "reason": result.reason,
                    "rule_id": result.rule_id,
                    "source_plugin": source_plugin,
                    "risk_level": risk_level,
                    "suggested_level": result.suggested_level,
                },
                source="security_validator",
                priority=Priority.HIGH,
            ))

    def reload_config(self) -> int:
        if not self._config_path:
            return 0
        try:
            with open(self._config_path) as f:
                content = f.read()
            new_hash = hashlib.md5(content.encode()).hexdigest()
            if new_hash == self._last_config_hash:
                return 0
            self._last_config_hash = new_hash
            cfg = json.loads(content)
            self._config.clear()
            self._config.update(cfg)
            self._rules = get_all_rules()
            self._risk_levels = self._load_risk_config(cfg)
            self._last_reload = time.time()
            logger.info("security config reloaded: %d rules", len(self._rules))
            return len(self._rules)
        except (OSError, json.JSONDecodeError) as e:
            logger.warning("config reload failed: %s", e)
            return 0

    def get_stats(self) -> dict:
        with self._lock:
            return dict(self._stats)


async def watch_config_reload(validator: SecurityValidator, interval: float = 5.0):
    last_mtime = 0
    while True:
        try:
            mtime = os.path.getmtime(validator._config_path)
            if mtime != last_mtime:
                last_mtime = mtime
                count = validator.reload_config()
                if count > 0 and validator._event_bus:
                    await validator._event_bus.publish(Event(
                        type="security.config_reloaded",
                        payload={"rules_loaded": count},
                        source="security_validator",
                        priority=Priority.MEDIUM,
                    ))
        except (OSError, AttributeError):
            pass
        await asyncio.sleep(interval)
