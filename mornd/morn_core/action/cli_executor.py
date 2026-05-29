import logging
import os
import platform
import shlex
import subprocess
import time
from typing import Optional

from morn_core.security.security_validator import SecurityValidator, ValidationResult
from morn_core.eventbus.bus import EventBus, Event, Priority


_DANGEROUS_PATTERNS = [
    "rm -rf /",
    "rm -rf /*",
    "dd if=",
    ":(){ :|:& };:",
    "chmod 777 /",
    "chmod -R 777 /",
    "sudo ",
    "> /dev/sda",
    "mkfs.",
    "fdisk",
]


class ExecError(dict):
    def __init__(self, error: str):
        super().__init__(success=False, error=error, returncode=-1)


class CLIExecutor:
    def __init__(self, config=None, validator: Optional[SecurityValidator] = None,
                 event_bus: Optional[EventBus] = None):
        self._config = config or {}
        self._validator = validator
        self._event_bus = event_bus
        self._logger = logging.getLogger("morn.cli")
        self._history = []
        self._available = None

    def is_available(self):
        if self._available is not None:
            return self._available
        try:
            subprocess.run(
                ["echo", "available"],
                capture_output=True,
                timeout=5,
            )
            self._available = True
        except Exception:
            self._available = False
        return self._available

    def get_shell(self):
        return os.environ.get("SHELL", platform.system() == "Windows" and "cmd.exe" or "/bin/sh")

    def validate(self, command):
        if not command or not command.strip():
            return False, "空命令"
        lower = command.lower().strip()
        for pattern in _DANGEROUS_PATTERNS:
            if pattern in lower:
                return False, f"危险命令被拦截: {pattern}"
        return True, ""

    async def async_execute(self, command: str, source_plugin: str = "cli",
                            risk_level: str = "yellow") -> dict:
        if self._validator:
            result = self._validator.validate(
                action_type="execute_command",
                params={"cmd": command},
                source_plugin=source_plugin,
                risk_level=risk_level,
                risk_preference=self._config.get("risk_preference", "yellow"),
            )
            if result.action == "block":
                if self._event_bus:
                    await self._validator.publish_alert(result, source_plugin, risk_level)
                return ExecError(f"blocked: {result.reason}")
            elif result.action == "confirm":
                if self._event_bus:
                    await self._event_bus.publish(Event(
                        type="security.confirm_required",
                        payload={"command": command, "reason": result.reason},
                        source="security_validator",
                        priority=Priority.HIGH,
                    ))
                return ExecError("pending_confirmation")

        for pattern in _DANGEROUS_PATTERNS:
            if pattern in command.lower():
                return ExecError("blocked_by_pattern")

        return self._do_execute(command)

    def execute(self, command, timeout=30, workdir=None):
        valid, reason = self.validate(command)
        if not valid:
            self._logger.warning("命令被拦截: %s (%s)", command, reason)
            return {"success": False, "error": reason, "returncode": -1}
        return self._do_execute(command, timeout=timeout, workdir=workdir)

    def _do_execute(self, command, timeout=30, workdir=None):
        start = time.time()
        try:
            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                timeout=timeout,
                cwd=workdir or os.getcwd(),
                text=True,
            )
            elapsed = time.time() - start
            entry = {
                "command": command,
                "success": result.returncode == 0,
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "elapsed": elapsed,
            }
            self._history.append(entry)
            self._logger.info("命令执行完成: %s (rc=%d, %.2fs)", command, result.returncode, elapsed)
            return entry
        except subprocess.TimeoutExpired:
            elapsed = time.time() - start
            entry = {
                "command": command,
                "success": False,
                "error": f"执行超时 ({timeout}s)",
                "returncode": -1,
                "elapsed": elapsed,
            }
            self._history.append(entry)
            self._logger.warning("命令执行超时: %s (%ds)", command, timeout)
            return entry
        except Exception as e:
            elapsed = time.time() - start
            entry = {
                "command": command,
                "success": False,
                "error": str(e),
                "returncode": -1,
                "elapsed": elapsed,
            }
            self._history.append(entry)
            self._logger.error("命令执行异常: %s (%s)", command, e)
            return entry

    def get_history(self):
        return list(self._history)
