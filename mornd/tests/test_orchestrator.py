import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.orchestrator import (
    EvolutionOrchestrator, FastCycleScheduler, SlowCycleScheduler,
)


class TestFastCycle:
    def test_fast_cycle_register_and_tick(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        calls = []

        def my_optimize():
            calls.append(1)
            return "ok"

        scheduler.register("test_comp", my_optimize, min_samples=1)
        results = scheduler.tick()
        assert len(calls) == 1
        assert len(results) == 1
        assert results[0]["success"] is True
        assert results[0]["output"] == "ok"

    def test_fast_cycle_skips_without_samples(self):
        scheduler = FastCycleScheduler({"fast_interval": 0})
        calls = []

        def my_optimize():
            calls.append(1)

        scheduler.register("test_comp", my_optimize, min_samples=10)
        results = scheduler.tick()
        assert len(calls) == 1


class TestSlowCycle:
    def test_slow_cycle_requires_approval(self):
        scheduler = SlowCycleScheduler({"slow_interval": 0})
        calls = []

        def my_optimize():
            calls.append(1)

        scheduler.register("test_comp", my_optimize, min_samples=1)
        results = scheduler.tick()
        assert len(calls) == 0
        assert any("awaiting_approval" in str(r.get("error", "")) for r in results)

    def test_slow_cycle_creates_snapshot(self):
        scheduler = SlowCycleScheduler({"slow_interval": 0, "snapshot_before_change": True})
        calls = []

        def my_optimize():
            calls.append(1)
            return "done"

        scheduler.register("test_comp", my_optimize, min_samples=1, require_approval=False)
        results = scheduler.tick()
        assert len(calls) == 1
        assert scheduler.get_status()["snapshots_count"] >= 1
        assert results[0]["snapshot"] is not None


class TestOrchestrator:
    def test_orchestrator_get_status(self):
        orch = EvolutionOrchestrator()
        status = orch.get_status()
        assert "fast_enabled" in status
        assert "slow_enabled" in status
        assert "fast_cycle" in status
        assert "slow_cycle" in status

    def test_fast_and_slow_independent(self):
        orch = EvolutionOrchestrator({
            "fast_interval": 0,
            "slow_interval": 0,
        })
        fast_calls = []
        slow_calls = []

        def fast_fn():
            fast_calls.append(1)
            return "fast_done"

        def slow_fn():
            slow_calls.append(1)
            return "slow_done"

        orch.register_fast("fast_comp", fast_fn, min_samples=1)
        orch.register_slow("slow_comp", slow_fn, min_samples=1, require_approval=False)

        fast_results = orch.tick_fast()
        slow_results = orch.tick_slow()

        assert len(fast_calls) == 1
        assert len(slow_calls) == 1
        assert fast_results[0]["success"] is True
        assert slow_results[0]["success"] is True