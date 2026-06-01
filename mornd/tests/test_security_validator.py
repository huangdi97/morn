import asyncio
import json
import os
import sys
import tempfile
import threading

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.security.security_validator import SecurityValidator, ValidationResult, watch_config_reload
from morn_core.eventbus.bus import EventBus
from morn_core.action.cli_executor import CLIExecutor


@pytest.fixture
async def event_bus():
    loop = asyncio.get_event_loop()
    bus = EventBus(loop)
    await bus.start()
    yield bus
    await bus.stop()


class TestSecurityValidatorValidate:
    def test_allow_green(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        r = v.validate("chat", {"text": "hello"}, "core", "green", "yellow")
        assert r.action == "allow"

    def test_allow_yellow(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        r = v.validate("chat", {"text": "hello"}, "core", "yellow", "yellow")
        assert r.action == "allow"

    def test_block_red(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        r = v.validate("execute", {"cmd": "danger"}, "core", "red", "yellow")
        assert r.action == "block"

    def test_block_black(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        r = v.validate("execute", {"cmd": "evil"}, "core", "black", "yellow")
        assert r.action == "block"

    def test_confirm_orange_high_pref(self):
        v = SecurityValidator({"risk_preference": "orange"})
        r = v.validate("config_write", {"key": "value"}, "core", "orange", "orange")
        assert r.action == "confirm"

    def test_block_orange_exceeds_pref(self):
        v = SecurityValidator({"risk_preference": "green"})
        r = v.validate("config_write", {"key": "value"}, "core", "orange", "green")
        assert r.action == "block"

    def test_blacklist_danger_001(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        r = v.validate("execute_command", {"cmd": "rm -rf /home"}, "cli", "yellow", "yellow")
        assert r.action == "block"
        assert r.rule_id == "DANGER_001"

    def test_blacklist_api_key(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        r = v.validate("send", {"content": "my api_key is sk-abc123"}, "core", "yellow", "yellow")
        assert r.action == "block"

    def test_acl_blocked(self):
        v = SecurityValidator({
            "risk_preference": "yellow",
            "plugin_permissions": {"chat_plugin": ["chat"]},
        })
        r = v.validate("execute_command", {"cmd": "ls"}, "chat_plugin", "yellow", "yellow")
        assert r.action == "block"
        assert "无权" in r.reason

    def test_acl_allowed(self):
        v = SecurityValidator({
            "risk_preference": "yellow",
            "plugin_permissions": {"chat_plugin": ["chat", "execute_command"]},
        })
        r = v.validate("execute_command", {"cmd": "ls"}, "chat_plugin", "yellow", "yellow")
        assert r.action == "allow"

    def test_acl_empty_permissions_skip(self):
        v = SecurityValidator({
            "risk_preference": "yellow",
            "plugin_permissions": {},
        })
        r = v.validate("execute_command", {"cmd": "ls"}, "unknown_plugin", "yellow", "yellow")
        assert r.action == "allow"

    def test_validation_result_dataclass(self):
        r = ValidationResult("allow", "测试原因", "green", "RULE_001")
        assert r.action == "allow"
        assert r.reason == "测试原因"
        assert r.suggested_level == "green"
        assert r.rule_id == "RULE_001"

    def test_validation_result_default_rule_id(self):
        r = ValidationResult("block", "原因", "red")
        assert r.rule_id is None

    def test_risk_level_defaults_to_yellow(self):
        v = SecurityValidator({})
        r = v.validate("chat", {"text": "hi"}, "core", "unknown_level", "yellow")
        assert r.action == "allow"


class TestSecurityValidatorStats:
    def test_stats_initial(self):
        v = SecurityValidator({})
        s = v.get_stats()
        assert s == {"allowed": 0, "blocked": 0, "confirmed": 0}

    def test_stats_allowed(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        v.validate("chat", {"text": "hello"}, "core", "green", "yellow")
        s = v.get_stats()
        assert s["allowed"] == 1

    def test_stats_blocked(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        v.validate("exec", {"cmd": "evil"}, "core", "red", "yellow")
        s = v.get_stats()
        assert s["blocked"] == 1

    def test_stats_confirmed(self):
        v = SecurityValidator({"risk_preference": "orange"})
        v.validate("write", {"file": "test"}, "core", "orange", "orange")
        s = v.get_stats()
        assert s["confirmed"] == 1

    def test_stats_multiple_actions(self):
        v = SecurityValidator({"risk_preference": "orange"})
        v.validate("chat", {}, "core", "green", "orange")
        v.validate("chat", {}, "core", "green", "orange")
        v.validate("exec", {}, "core", "red", "orange")
        v.validate("write", {}, "core", "orange", "orange")
        s = v.get_stats()
        assert s["allowed"] == 2
        assert s["blocked"] == 1
        assert s["confirmed"] == 1


class TestSecurityValidatorEventBus:
    async def test_publish_alert(self, event_bus):
        received = []

        async def handler(event):
            received.append(event)

        event_bus.subscribe("security.alert", handler, "alert_test")

        v = SecurityValidator({"risk_preference": "yellow"}, event_bus=event_bus)
        result = ValidationResult("block", "测试拦截", "red", "DANGER_001")
        await v.publish_alert(result, "test_plugin", "red")
        await asyncio.sleep(0.05)
        assert len(received) == 1
        assert received[0].payload["action"] == "block"
        assert received[0].payload["rule_id"] == "DANGER_001"
        assert received[0].payload["source_plugin"] == "test_plugin"

    async def test_alert_no_eventbus(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        result = ValidationResult("block", "test", "red")
        await v.publish_alert(result, "plugin", "red")

    async def test_config_reloaded_event(self, event_bus):
        received = []

        async def handler(event):
            received.append(event)

        event_bus.subscribe("security.config_reloaded", handler, "config_test")

        v = SecurityValidator({"risk_preference": "yellow"}, event_bus=event_bus)
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump({"risk_preference": "orange"}, f)
            config_path = f.name

        v.set_config_path(config_path)
        count = v.reload_config()
        assert count > 0

        os.unlink(config_path)

    async def test_security_alert_from_cli_executor(self, event_bus):
        received = []

        async def handler(event):
            received.append(event)

        event_bus.subscribe("security.alert", handler, "cli_alert_test")

        v = SecurityValidator({"risk_preference": "yellow"}, event_bus=event_bus)
        e = CLIExecutor({"risk_preference": "yellow"}, validator=v, event_bus=event_bus)

        result = await e.async_execute("echo hello", risk_level="red")
        assert result["success"] is False
        assert "blocked" in str(result.get("error", ""))
        await asyncio.sleep(0.05)
        assert len(received) >= 1

    async def test_confirm_required_event(self, event_bus):
        received = []

        async def handler(event):
            received.append(event)

        event_bus.subscribe("security.confirm_required", handler, "confirm_test")

        v = SecurityValidator({"risk_preference": "orange"}, event_bus=event_bus)
        e = CLIExecutor({"risk_preference": "orange"}, validator=v, event_bus=event_bus)

        result = await e.async_execute("echo hello", risk_level="orange")
        assert "pending_confirmation" in str(result.get("error", ""))
        await asyncio.sleep(0.05)
        assert len(received) >= 1


class TestHotReload:
    def test_set_config_path(self):
        v = SecurityValidator({})
        v.set_config_path("/tmp/test_config.json")
        assert v._config_path == "/tmp/test_config.json"

    def test_reload_config_same_content(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump({"risk_preference": "yellow"}, f)
            path = f.name

        v.set_config_path(path)
        count1 = v.reload_config()
        count2 = v.reload_config()
        assert count1 > 0
        assert count2 == 0

        os.unlink(path)

    def test_reload_config_changed(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump({"risk_preference": "green"}, f)
            path = f.name

        v.set_config_path(path)
        count1 = v.reload_config()

        with open(path, "w") as f:
            json.dump({"risk_preference": "orange"}, f)
        count2 = v.reload_config()
        assert count2 > 0

        os.unlink(path)

    def test_reload_config_none_path(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        count = v.reload_config()
        assert count == 0

    async def test_watch_config_reload_detects_change(self, event_bus):
        reloaded = []

        async def handler(event):
            reloaded.append(event)

        event_bus.subscribe("security.config_reloaded", handler, "reload_test")

        v = SecurityValidator({"risk_preference": "yellow"}, event_bus=event_bus)
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json.dump({"risk_preference": "yellow"}, f)
            path = f.name

        v.set_config_path(path)
        v.reload_config()

        watch_task = asyncio.create_task(watch_config_reload(v, interval=0.1))

        with open(path, "w") as f:
            json.dump({"risk_preference": "orange"}, f)

        await asyncio.sleep(0.3)
        watch_task.cancel()
        try:
            await watch_task
        except (asyncio.CancelledError, StopIteration):
            pass

        assert len(reloaded) >= 1
        assert reloaded[0].payload["rules_loaded"] > 0

        os.unlink(path)

    def test_reload_no_path(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        count = v.reload_config()
        assert count == 0


class TestCLIExecutorIntegration:
    async def test_async_execute_no_validator(self):
        e = CLIExecutor({})
        result = await e.async_execute("echo hello")
        assert result["success"] is True

    async def test_async_execute_blocked_by_validator(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        e = CLIExecutor({"risk_preference": "yellow"}, validator=v)
        result = await e.async_execute("echo hello", risk_level="red")
        assert result["success"] is False
        assert "blocked" in str(result.get("error", ""))

    async def test_async_execute_dangerous_pattern_backup(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        e = CLIExecutor({"risk_preference": "yellow"}, validator=v)
        result = await e.async_execute("rm -rf /", risk_level="green")
        assert result["success"] is False

    async def test_async_execute_confirm_then_blocked_by_pattern(self):
        v = SecurityValidator({"risk_preference": "orange"})
        e = CLIExecutor({"risk_preference": "orange"}, validator=v)
        result = await e.async_execute("rm -rf /", risk_level="orange")
        assert result["success"] is False

    async def test_async_execute_safe_command(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        e = CLIExecutor({"risk_preference": "yellow"}, validator=v)
        result = await e.async_execute("echo hello_world", risk_level="green")
        assert result["success"] is True

    def test_sync_execute_unchanged(self):
        e = CLIExecutor({})
        result = e.execute("echo hello")
        assert result["success"] is True


class TestConcurrency:
    def test_concurrent_validate_stats_consistent(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        n = 100
        errors = []

        def do_validate(risk_level):
            try:
                v.validate("test", {}, "core", risk_level, "yellow")
            except Exception as e:
                errors.append(e)

        threads = []
        for i in range(n):
            level = "green" if i % 3 == 0 else ("red" if i % 3 == 1 else "yellow")
            t = threading.Thread(target=do_validate, args=(level,))
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

        assert len(errors) == 0
        s = v.get_stats()
        expected_allowed = sum(1 for i in range(n) if i % 3 != 1)
        expected_blocked = sum(1 for i in range(n) if i % 3 == 1)
        assert s["allowed"] == expected_allowed
        assert s["blocked"] == expected_blocked

    def test_get_stats_thread_safe(self):
        v = SecurityValidator({"risk_preference": "yellow"})
        results = []

        def read_stats():
            results.append(v.get_stats())

        threads = [threading.Thread(target=read_stats) for _ in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        assert len(results) == 10
        for r in results:
            assert "allowed" in r
            assert "blocked" in r
            assert "confirmed" in r

    def test_concurrent_validate_different_configs(self):
        def test_with_config(pref):
            v = SecurityValidator({"risk_preference": pref})
            r = v.validate("test", {}, "core", "orange", pref)
            # orange risk > green/yellow preference → block
            # orange risk == orange preference → confirm
            if pref == "orange":
                assert r.action == "confirm", f"expected confirm for pref={pref}, got {r.action}"
            else:
                assert r.action == "block", f"expected block for pref={pref}, got {r.action}"

        threads = []
        for pref in ["green", "yellow", "orange"]:
            t = threading.Thread(target=test_with_config, args=(pref,))
            threads.append(t)
            t.start()

        for t in threads:
            t.join()
