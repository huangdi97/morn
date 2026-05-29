import os
import sys
import json
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.security_advanced.risk_guard import DynamicPermissions, DEFAULT_PERMISSIONS


class TestSixDimensionEvaluation:
    def test_online_low_risk_passes(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        level, reason = dp.evaluate("chat", {"creator_status": "online"})
        assert level == "🟢"

    def test_night_escalation(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        level, reason = dp.evaluate("memory_write", {
            "creator_status": "night",
            "task_context": "explicit_instruction",
            "time": "non_work_hours",
            "device": "primary_device",
            "history": {"success_rate": 1.0},
        })
        assert level in ("🟠", "🔴", "⚫")

    def test_unknown_device_escalation(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        level, reason = dp.evaluate("memory_write", {
            "creator_status": "online",
            "task_context": "explicit_instruction",
            "time": "work_hours",
            "device": "unknown_device",
            "history": {"success_rate": 1.0},
        })
        assert _risk_score(level) > _risk_score("🟡")

    def test_low_history_success_escalates(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        level, reason = dp.evaluate("chat", {
            "creator_status": "online",
            "task_context": "explicit_instruction",
            "time": "work_hours",
            "device": "primary_device",
            "history": {"success_rate": 0.1},
        })
        assert level != "🟢"

    def test_system_self_trigger_escalation(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        base_level, _ = dp.evaluate("config_read", {
            "creator_status": "online",
            "task_context": "explicit_instruction",
            "time": "work_hours",
            "device": "primary_device",
            "history": {"success_rate": 1.0},
        })
        escalated_level, _ = dp.evaluate("config_read", {
            "creator_status": "online",
            "task_context": "system_self_trigger",
            "time": "work_hours",
            "device": "primary_device",
            "history": {"success_rate": 1.0},
        })
        assert _risk_score(escalated_level) >= _risk_score(base_level)


class TestRiskLevelJudgment:
    def test_green_action_allowed(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        assert dp.allow("chat", {"creator_status": "online", "task_context": "explicit_instruction", "time": "work_hours", "device": "primary_device", "history": {"success_rate": 1.0}})

    def test_red_action_denied(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        assert not dp.allow("code_execute", {"creator_status": "online", "task_context": "explicit_instruction", "time": "work_hours", "device": "primary_device", "history": {"success_rate": 1.0}})

    def test_black_action_denied(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        assert not dp.allow("self_modify", {"creator_status": "online", "task_context": "explicit_instruction", "time": "work_hours", "device": "primary_device", "history": {"success_rate": 1.0}})


class TestConfigLoading:
    def test_default_config_loaded(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        assert dp.get_permission("chat") == "🟢"
        assert dp.get_permission("code_execute") == "🔴"
        assert dp.get_permission("self_modify") == "⚫"

    def test_config_persisted_to_file(self):
        d = Path(tempfile.mkdtemp())
        dp = DynamicPermissions(d)
        dp.modify("chat", "🟡")
        dp2 = DynamicPermissions(d)
        assert dp2.get_permission("chat") == "🟡"

    def test_config_corrupt_uses_defaults(self):
        d = Path(tempfile.mkdtemp())
        config_dir = d / "security"
        config_dir.mkdir(parents=True, exist_ok=True)
        (config_dir / "permissions.json").write_text("not-json")
        dp = DynamicPermissions(d)
        assert dp.get_permission("chat") == "🟢"


class TestDynamicModify:
    def test_modify_permission_level(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        dp.modify("chat", "🔴")
        assert dp.get_permission("chat") == "🔴"

    def test_modify_invalid_level_raises(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        try:
            dp.modify("chat", "invalid")
            assert False, "should have raised"
        except ValueError:
            pass

    def test_modify_affects_evaluate(self):
        dp = DynamicPermissions(Path(tempfile.mkdtemp()))
        dp.modify("chat", "🔴")
        level, _ = dp.evaluate("chat", {"creator_status": "online", "task_context": "explicit_instruction", "time": "work_hours", "device": "primary_device", "history": {"success_rate": 1.0}})
        assert level == "🔴"


def _risk_score(level):
    order = {"🟢": 0, "🟡": 1, "🟠": 2, "🔴": 3, "⚫": 4}
    return order.get(level, 0)
