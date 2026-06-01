import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.action.multi_path import ActionRouter


class _MockCLI:
    def is_available(self):
        return True

    def execute(self, command, timeout=30, workdir=None):
        return {"success": True, "stdout": "mock output", "returncode": 0}

    def get_history(self):
        return []


class _MockAPI:
    def __init__(self, success=True):
        self._success = success

    def is_available(self):
        return True

    def call(self, method, url, headers=None, body=None, timeout=10):
        if self._success:
            return {"success": True, "status_code": 200, "body": "{}"}
        return {"success": False, "error": "mock failure", "status_code": 500}

    def get_history(self):
        return []


class _FailingCLI:
    def is_available(self):
        return True

    def execute(self, command, timeout=30, workdir=None):
        return {"success": False, "error": "mock failure", "returncode": 1}

    def get_history(self):
        return []


class _UnavailableCLI:
    def is_available(self):
        return False

    def execute(self, command, timeout=30, workdir=None):
        return {"success": False, "error": "unavailable", "returncode": -1}

    def get_history(self):
        return []


class _UnavailableAPI:
    def is_available(self):
        return False

    def call(self, method, url, headers=None, body=None, timeout=10):
        return {"success": False, "error": "unavailable", "status_code": 0}

    def get_history(self):
        return []


class TestActionRouterPaths:
    def test_auto_select_cli(self):
        router = ActionRouter(cli_executor=_MockCLI(), api_caller=_MockAPI())
        result = router.route("command", params={"command": "echo hello"})
        assert result["success"] is True
        assert result["path"] == "cli"

    def test_auto_select_api_when_cli_fails(self):
        router = ActionRouter(cli_executor=_FailingCLI(), api_caller=_MockAPI())
        result = router.route("api_call", params={"method": "GET", "url": "http://example.com"})
        assert result["success"] is True
        assert result["path"] == "api"

    def test_cli_fail_degrade_to_api(self):
        router = ActionRouter(cli_executor=_FailingCLI(), api_caller=_MockAPI())
        result = router.route("command", params={"command": "fail"})
        assert result["path"] == "api"

    def test_all_unavailable_returns_manual(self):
        router = ActionRouter(cli_executor=_UnavailableCLI(), api_caller=_UnavailableAPI())
        result = router.route("command", params={"command": "test"})
        assert result["success"] is False
        assert result["path"] == "manual"

    def test_get_available_paths(self):
        router = ActionRouter(cli_executor=_MockCLI(), api_caller=_MockAPI())
        paths = router.get_available_paths()
        assert "cli" in paths
        assert "api" in paths
        assert "manual" in paths

    def test_get_fallback_chain(self):
        router = ActionRouter()
        chain = router.get_fallback_chain()
        assert chain == ["cli", "api", "browser", "manual"]

    def test_force_path_cli(self):
        router = ActionRouter(
            config={"force_path": "cli"},
            cli_executor=_MockCLI(),
            api_caller=_MockAPI(),
        )
        result = router.route("api_call", params={"method": "GET", "url": "http://example.com"})
        assert result["path"] == "cli"


class TestActionRouterHistory:
    def test_history_success_rate_affects_selection(self):
        cli = _MockCLI()
        api = _MockAPI(success=False)
        router = ActionRouter(cli_executor=cli, api_caller=api)
        result = router.route("command", params={"command": "test"})
        assert result["path"] == "cli"

    def test_routing_records_history(self):
        router = ActionRouter(cli_executor=_MockCLI(), api_caller=_MockAPI())
        assert len(router.get_history()) == 0
        router.route("command", params={"command": "test"})
        assert len(router.get_history()) == 1
        assert router.get_history()[0]["path"] == "cli"

    def test_no_executor_configured(self):
        router = ActionRouter()
        result = router.route("command", params={"command": "test"})
        assert result["success"] is False
        assert result["path"] == "manual"

    def test_browser_path_not_available(self):
        router = ActionRouter(cli_executor=_MockCLI(), api_caller=_MockAPI())
        paths = router.get_available_paths()
        assert "browser" not in paths