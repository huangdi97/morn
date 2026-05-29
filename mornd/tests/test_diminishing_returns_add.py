import os
import sys
import time

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.orchestrator import FastCycleScheduler


class TestDiminishingReturns:
    def test_10_improvements_below_1_percent_caps(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        calls = []

        def fn():
            calls.append(1)
            return "ok"

        scheduler.register("comp_a", fn, min_samples=1)
        scheduler.tick()

        for _ in range(9):
            scheduler._tasks["comp_a"]["last_execution"] = 0
            scheduler.tick()

        assert len(calls) == 10
        assert "comp_a" in scheduler._capped_components

    def test_capped_component_rejects_new_registration(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        calls = []

        def fn():
            calls.append(1)
            return "ok"

        scheduler.register("comp_b", fn, min_samples=1)
        for _ in range(10):
            scheduler._tasks["comp_b"]["last_execution"] = 0
            scheduler.tick()

        assert "comp_b" in scheduler._capped_components

        def fn2():
            calls.append(2)
            return "ok"

        scheduler.register("comp_b", fn2, min_samples=1)
        assert scheduler._tasks["comp_b"]["enabled"] is False
        assert scheduler._tasks["comp_b"]["fn"] is not fn2

    def test_unlock_after_90_days(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        calls = []

        def fn():
            calls.append(1)
            return "ok"

        scheduler.register("comp_c", fn, min_samples=1)
        for _ in range(10):
            scheduler._tasks["comp_c"]["last_execution"] = 0
            scheduler.tick()

        cap_info = scheduler._capped_components["comp_c"]
        cap_info["unlock_at"] = time.time() - 1

        def fn2():
            calls.append(2)
            return "ok"

        scheduler.register("comp_c", fn2, min_samples=1)
        assert "comp_c" not in scheduler._capped_components

    def test_non_capped_component_continues_optimizing(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        calls = []

        def fn():
            calls.append(1)
            return "ok"

        scheduler.register("comp_d", fn, min_samples=1)
        scheduler.tick()
        assert scheduler._tasks["comp_d"]["enabled"] is True
        assert "comp_d" not in scheduler._capped_components

    def test_get_capped_components_returns_list(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        calls = []

        def fn():
            calls.append(1)
            return "ok"

        scheduler.register("comp_e", fn, min_samples=1)
        for _ in range(10):
            scheduler._tasks["comp_e"]["last_execution"] = 0
            scheduler.tick()

        capped = scheduler.get_capped_components()
        assert "comp_e" in capped
        assert "capped_at" in capped["comp_e"]
        assert "unlock_at" in capped["comp_e"]

    def test_threshold_is_configurable(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        scheduler._diminishing_threshold = 0.5
        calls = []

        def fn():
            calls.append(1)
            return "ok"

        scheduler.register("comp_f", fn, min_samples=1)
        for _ in range(10):
            scheduler._tasks["comp_f"]["last_execution"] = 0
            scheduler.tick()

        capped = scheduler.get_capped_components()
        assert "comp_f" in capped