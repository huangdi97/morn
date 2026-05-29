import logging
from typing import Optional

from morn_core.security.rules import SecurityValidator


logger = logging.getLogger("morn.security")


def log_security_event(action: str, result: dict) -> None:
    logger.warning(
        "安全事件 | action=%s | verdict=%s | reason=%s | rule=%s",
        action, result.get("verdict"), result.get("reason"), result.get("rule"),
    )


class ActionRouter:
    PATHS = ["cli", "api", "browser", "manual"]

    def __init__(self, config=None, cli_executor=None, api_caller=None):
        self._config = config or {}
        self._forced_path = self._config.get("force_path", None)
        self._cli = cli_executor
        self._api = api_caller
        self._logger = logging.getLogger("morn.router")
        self._history = []

    def execute(self, action: str, context: Optional[dict] = None) -> dict:
        result = SecurityValidator().validate(action, context)
        if result["verdict"] == "block":
            log_security_event(action, result)
            return {"status": "blocked", "reason": result["reason"]}
        return {"status": "allowed", "action": action}

    def get_available_paths(self):
        available = []
        for path in self.PATHS:
            check = self._get_path_available(path)
            if check:
                available.append(path)
        return available

    def _get_path_available(self, path):
        if path == "cli":
            return self._cli and self._cli.is_available()
        if path == "api":
            return self._api and self._api.is_available()
        if path == "browser":
            return False
        if path == "manual":
            return True
        return False

    def get_fallback_chain(self, action_type=None):
        return self.PATHS[:]

    def _get_path_delay(self, path):
        delays = {"cli": 0.01, "api": 0.3, "browser": 2.0, "manual": 999}
        return delays.get(path, 999)

    def _get_path_success_rate(self, path):
        entries = [h for h in self._history if h.get("path") == path]
        if not entries:
            return 1.0
        successes = sum(1 for e in entries if e.get("success"))
        return successes / len(entries)

    def route(self, action_type, params=None, context=None):
        if self._forced_path and self._forced_path in self.PATHS:
            return self._execute_path(self._forced_path, action_type, params, context)

        candidates = []
        for path in self.PATHS:
            if not self._get_path_available(path):
                continue
            delay = self._get_path_delay(path)
            rate = self._get_path_success_rate(path)
            candidates.append((path, delay, rate))

        candidates.sort(key=lambda x: (x[1], -x[2]))

        for path, delay, rate in candidates:
            result = self._execute_path(path, action_type, params, context)
            if result.get("success"):
                return result

        return {
            "success": False,
            "path": "manual",
            "error": "所有路径均不可行，请人工处理",
            "action_type": action_type,
        }

    def _execute_path(self, path, action_type, params, context):
        if path == "cli":
            if not self._cli:
                return {"success": False, "path": "cli", "error": "CLI 执行器未配置"}
            command = params.get("command", "") if params else ""
            result = self._cli.execute(command)
            entry = {"path": "cli", "success": result["success"], "result": result}
            self._history.append(entry)
            return {"success": result["success"], "path": "cli", "result": result, "action_type": action_type}

        if path == "api":
            if not self._api:
                return {"success": False, "path": "api", "error": "API 调用器未配置"}
            method = params.get("method", "GET") if params else "GET"
            url = params.get("url", "") if params else ""
            headers = params.get("headers", {}) if params else {}
            body = params.get("body", None) if params else None
            result = self._api.call(method, url, headers=headers, body=body)
            entry = {"path": "api", "success": result["success"], "result": result}
            self._history.append(entry)
            return {"success": result["success"], "path": "api", "result": result, "action_type": action_type}

        if path == "browser":
            entry = {"path": "browser", "success": False, "result": {"error": "浏览器自动化(v1.0+ 能力)"}}
            self._history.append(entry)
            return {"success": False, "path": "browser", "error": "浏览器自动化是 v1.0+ 能力", "action_type": action_type}

        if path == "manual":
            return {"success": False, "path": "manual", "error": "已降级到人工建议", "action_type": action_type}

        return {"success": False, "path": path, "error": f"未知路径: {path}"}

    def get_history(self):
        return list(self._history)
