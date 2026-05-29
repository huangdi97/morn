import os
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.harness import HarnessOptimizer


class TestMetricsCollection:
    def test_collect_prompt_metrics(self):
        opt = HarnessOptimizer({"enabled": True})
        opt.collect_metrics("prompt", {"success": True})
        m = opt.get_metrics()
        assert m["prompt_quality"]["total"] == 1
        assert m["prompt_quality"]["success"] == 1

    def test_collect_tool_metrics(self):
        opt = HarnessOptimizer({"enabled": True})
        opt.collect_metrics("tool", {"success": False, "duration": 2.5})
        m = opt.get_metrics()
        assert m["tool_efficiency"]["total_calls"] == 1
        assert m["tool_efficiency"]["fail"] == 1
        assert m["tool_efficiency"]["total_time"] == 2.5

    def test_collect_memory_metrics(self):
        opt = HarnessOptimizer({"enabled": True})
        opt.collect_metrics("memory", {"adopted": True})
        m = opt.get_metrics()
        assert m["memory_params"]["total_retrievals"] == 1
        assert m["memory_params"]["adopted"] == 1

    def test_collect_disabled_does_nothing(self):
        opt = HarnessOptimizer({"enabled": False})
        opt.collect_metrics("prompt", {"success": True})
        m = opt.get_metrics()
        assert m["prompt_quality"]["total"] == 0


class TestDiagnose:
    def test_diagnose_prompt_issue(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("prompt", {"success": False})
        issues = opt.diagnose()
        assert any(i["type"] == "prompt_quality" for i in issues)

    def test_diagnose_tool_fail_rate(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("tool", {"success": False, "duration": 0.5})
        issues = opt.diagnose()
        assert any(i["type"] == "tool_efficiency" for i in issues)

    def test_diagnose_tool_avg_duration(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("tool", {"success": True, "duration": 6.0})
        issues = opt.diagnose()
        assert any(i["metric"] == "avg_duration" for i in issues)

    def test_diagnose_memory_issue(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("memory", {"adopted": False})
        issues = opt.diagnose()
        assert any(i["type"] == "memory_params" for i in issues)

    def test_diagnose_no_issues(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("prompt", {"success": True})
            opt.collect_metrics("tool", {"success": True, "duration": 0.1})
            opt.collect_metrics("memory", {"adopted": True})
        issues = opt.diagnose()
        assert len(issues) == 0

    def test_diagnose_disabled_returns_empty(self):
        opt = HarnessOptimizer({"enabled": False})
        assert opt.diagnose() == []

    def test_diagnose_insufficient_data(self):
        opt = HarnessOptimizer({"enabled": True})
        opt.collect_metrics("prompt", {"success": False})
        assert opt.diagnose() == []


class TestOptimize:
    def test_optimize_prompt(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("prompt", {"success": False})
        sugg = opt.optimize("prompt")
        assert len(sugg) > 0
        assert all(s["target"] == "prompt" for s in sugg)

    def test_optimize_tool(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("tool", {"success": False, "duration": 0.5})
        sugg = opt.optimize("tool")
        assert len(sugg) > 0
        assert all(s["target"] == "tool" for s in sugg)

    def test_optimize_memory(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("memory", {"adopted": False})
        sugg = opt.optimize("memory")
        assert len(sugg) > 0
        assert all(s["target"] == "memory" for s in sugg)

    def test_optimize_all(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("prompt", {"success": False})
            opt.collect_metrics("tool", {"success": False, "duration": 6.0})
            opt.collect_metrics("memory", {"adopted": False})
        sugg = opt.optimize("all")
        assert len(sugg) >= 6

    def test_optimize_disabled_returns_empty(self):
        opt = HarnessOptimizer({"enabled": False})
        assert opt.optimize("all") == []

    def test_optimize_no_issues_empty(self):
        opt = HarnessOptimizer({"enabled": True})
        for _ in range(10):
            opt.collect_metrics("prompt", {"success": True})
        sugg = opt.optimize("prompt")
        assert sugg == []


class TestDisabledByDefault:
    def test_disabled_by_default(self):
        opt = HarnessOptimizer()
        assert opt.enabled is False

    def test_disabled_collect_noop(self):
        opt = HarnessOptimizer()
        opt.collect_metrics("prompt", {"success": True})
        assert opt._metrics["prompt_quality"]["total"] == 0


class TestPersistence:
    def test_save_and_load(self):
        with tempfile.TemporaryDirectory() as tmp:
            opt = HarnessOptimizer({"enabled": True, "data_dir": tmp})
            opt.collect_metrics("prompt", {"success": True})
            opt2 = HarnessOptimizer({"enabled": True, "data_dir": tmp})
            m = opt2.get_metrics()
            assert m["prompt_quality"]["total"] == 1
