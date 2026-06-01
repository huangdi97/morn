import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.l0_tuner import ThinkingStyleEvolver


class TestRegistration:
    def test_register_returns_id(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        tid = ev.register_thought("error", ["check log", "retry"])
        assert tid is not None
        assert isinstance(tid, str)

    def test_register_disabled_returns_none(self):
        ev = ThinkingStyleEvolver({"enabled": False})
        tid = ev.register_thought("error", ["check log"])
        assert tid is None

    def test_register_stores_template(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        ev.register_thought("timeout", ["ping", "retry"], conditions=["network"])
        templates = ev.get_templates()
        assert len(templates) == 1
        assert templates[0]["name"] == "template_1"
        assert "timeout" in templates[0]["trigger_conditions"]
        assert "network" in templates[0]["trigger_conditions"]


class TestMatching:
    def test_get_matching_returns_by_context(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        ev.register_thought("error", ["check log"])
        ev.register_thought("timeout", ["ping"])
        result = ev.get_matching("got an error")
        assert len(result) == 1
        assert "check log" in result[0]["reasoning_steps"]

    def test_get_matching_empty_when_no_match(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        ev.register_thought("error", ["check log"])
        result = ev.get_matching("everything is fine")
        assert len(result) == 0

    def test_get_matching_disabled_returns_empty(self):
        ev = ThinkingStyleEvolver({"enabled": False})
        ev.register_thought("error", ["check log"])
        result = ev.get_matching("got an error")
        assert result == []

    def test_get_matching_uses_priority(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        tid_a = ev.register_thought("error", ["step a"])
        tid_b = ev.register_thought("error", ["step b"])
        t_a = next(t for t in ev._templates if t["template_id"] == tid_a)
        t_a["priority"] = 5.0
        result = ev.get_matching("got an error")
        assert result[0]["priority"] == 5.0


class TestOutcomeRecording:
    def test_record_success(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        tid = ev.register_thought("error", ["check"])
        ev.record_outcome(tid, True)
        t = ev._templates[0]
        assert t["success_count"] == 1
        assert t["fail_count"] == 0

    def test_record_failure(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        tid = ev.register_thought("error", ["check"])
        ev.record_outcome(tid, False)
        t = ev._templates[0]
        assert t["fail_count"] == 1

    def test_record_disabled_does_nothing(self):
        ev = ThinkingStyleEvolver({"enabled": False})
        tid = ev.register_thought("error", ["check"])
        ev.record_outcome(tid, True)
        assert ev._templates == []

    def test_record_nonexistent_id_does_nothing(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        ev.register_thought("error", ["check"])
        ev.record_outcome("nonexistent", True)
        assert ev._templates[0]["success_count"] == 0


class TestEvolve:
    def test_revise_creates_revised_template(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        tid = ev.register_thought("error", ["check log"])
        ev.record_outcome(tid, False)
        ev.record_outcome(tid, False)
        ev.record_outcome(tid, True)
        events = ev.evolve()
        revise_events = [e for e in events if e["action"] == "revise"]
        assert len(revise_events) >= 1
        assert any("_revised" in t["name"] for t in ev._templates)

    def test_recombine_creates_new_template(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        ev.register_thought("error", ["check log"], conditions=["a"])
        ev.register_thought("timeout", ["ping"], conditions=["a"])
        events = ev.evolve()
        recombine_events = [e for e in events if e["action"] == "recombine"]
        assert len(recombine_events) >= 1

    def test_refine_boosts_priority(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        tid = ev.register_thought("error", ["check log"])
        ev._templates[0]["success_count"] = 3
        events = ev.evolve()
        refine_events = [e for e in events if e["action"] == "refine"]
        assert len(refine_events) == 1
        assert ev._templates[0]["priority"] > 1.0

    def test_evolve_disabled_returns_empty(self):
        ev = ThinkingStyleEvolver({"enabled": False})
        ev.register_thought("error", ["check"])
        result = ev.evolve()
        assert result == []

    def test_evolve_no_templates_returns_empty(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        result = ev.evolve()
        assert result == []

    def test_refine_priority_capped(self):
        ev = ThinkingStyleEvolver({"enabled": True})
        tid = ev.register_thought("error", ["check"])
        ev._templates[0]["success_count"] = 3
        ev._templates[0]["priority"] = 9.0
        ev.evolve()
        assert ev._templates[0]["priority"] == 10.0


class TestDisabledByDefault:
    def test_disabled_by_default(self):
        ev = ThinkingStyleEvolver()
        assert ev.enabled is False

    def test_disabled_register_returns_none(self):
        ev = ThinkingStyleEvolver()
        assert ev.register_thought("x", ["y"]) is None

    def test_disabled_get_matching_empty(self):
        ev = ThinkingStyleEvolver()
        assert ev.get_matching("x") == []

    def test_disabled_evolve_empty(self):
        ev = ThinkingStyleEvolver()
        assert ev.evolve() == []
