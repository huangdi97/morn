import time
import copy
import logging
from typing import Any, Callable, Optional

logger = logging.getLogger("morn.evolution")


FAST_CYCLE_DEFAULTS = {
    "interval": 3600,
    "min_success_samples": 3,
    "rollback_on_fail": True,
    "max_concurrent": 2,
}

SLOW_CYCLE_DEFAULTS = {
    "interval": 86400,
    "min_success_samples": 10,
    "rollback_on_fail": True,
    "require_creator_approval": True,
    "snapshot_before_change": True,
}


class FastCycleScheduler:
    def __init__(self, config: Optional[dict] = None):
        config = config or {}
        self._interval = config.get("fast_interval", FAST_CYCLE_DEFAULTS["interval"])
        self._min_samples = config.get("fast_min_success_samples", FAST_CYCLE_DEFAULTS["min_success_samples"])
        self._rollback = config.get("fast_rollback_on_fail", FAST_CYCLE_DEFAULTS["rollback_on_fail"])
        self._max_concurrent = config.get("fast_max_concurrent", FAST_CYCLE_DEFAULTS["max_concurrent"])
        self._tasks: dict[str, dict] = {}
        self._last_tick: Optional[float] = None
        self._stats = {"ticks": 0, "executions": 0, "failures": 0}
        self._diminishing_threshold: float = 0.01
        self._diminishing_window: int = 10
        self._capped_components: dict = {}

    def register(self, component_name: str, optimize_fn: Callable,
                 min_samples: Optional[int] = None):
        if component_name in self._capped_components:
            info = self._capped_components[component_name]
            unlock_at = info.get("unlock_at", 0)
            if time.time() < unlock_at:
                logger.info("FastCycleScheduler: component '%s' is capped until %s, skipping registration",
                            component_name, unlock_at)
                return
            else:
                del self._capped_components[component_name]
        self._tasks[component_name] = {
            "fn": optimize_fn,
            "min_samples": min_samples or self._min_samples,
            "last_execution": 0.0,
            "execution_count": 0,
            "fail_count": 0,
            "enabled": True,
            "success_rates": [],
        }

    def tick(self) -> list[dict]:
        now = time.time()
        self._last_tick = now
        self._stats["ticks"] += 1
        results = []
        running = 0
        for name, task in self._tasks.items():
            if not task["enabled"]:
                continue
            if now - task["last_execution"] < self._interval:
                continue
            if running >= self._max_concurrent:
                break
            try:
                running += 1
                output = task["fn"]()
                task["execution_count"] += 1
                task["last_execution"] = now
                self._stats["executions"] += 1
                results.append({"component": name, "success": True, "output": output})
                self._check_diminishing_returns(name, 1.0 if output else 0.0)
            except Exception as e:
                task["fail_count"] += 1
                self._stats["failures"] += 1
                logger.warning("Fast cycle task %s failed: %s", name, e)
                results.append({"component": name, "success": False, "error": str(e)})
                self._check_diminishing_returns(name, 0.0)
        return results

    def _check_diminishing_returns(self, component_name: str, success_rate: float) -> bool:
        task = self._tasks.get(component_name)
        if not task:
            return False
        rates = task.setdefault("success_rates", [])
        rates.append(success_rate)
        if len(rates) > self._diminishing_window:
            rates.pop(0)
        if len(rates) < self._diminishing_window:
            return False
        improvements = []
        for i in range(1, len(rates)):
            improvements.append(abs(rates[i] - rates[i-1]))
        avg_improvement = sum(improvements) / len(improvements)
        if avg_improvement < self._diminishing_threshold:
            now_ts = time.time()
            self._capped_components[component_name] = {
                "capped_at": now_ts,
                "unlock_at": now_ts + 90 * 86400,
            }
            task["enabled"] = False
            logger.info("FastCycleScheduler: component '%s' capped due to diminishing returns (avg improvement: %.4f < %.4f)",
                        component_name, avg_improvement, self._diminishing_threshold)
            return True
        return False

    def get_capped_components(self) -> dict:
        return dict(self._capped_components)

    def get_status(self) -> dict:
        return {
            "interval": self._interval,
            "min_samples": self._min_samples,
            "max_concurrent": self._max_concurrent,
            "rollback_on_fail": self._rollback,
            "last_tick": self._last_tick,
            "stats": dict(self._stats),
            "tasks": {
                name: {
                    "last_execution": t["last_execution"],
                    "execution_count": t["execution_count"],
                    "fail_count": t["fail_count"],
                    "enabled": t["enabled"],
                }
                for name, t in self._tasks.items()
            },
            "capped_components": dict(self._capped_components),
        }


class SlowCycleScheduler:
    def __init__(self, config: Optional[dict] = None):
        config = config or {}
        self._interval = config.get("slow_interval", SLOW_CYCLE_DEFAULTS["interval"])
        self._min_samples = config.get("slow_min_success_samples", SLOW_CYCLE_DEFAULTS["min_success_samples"])
        self._rollback = config.get("slow_rollback_on_fail", SLOW_CYCLE_DEFAULTS["rollback_on_fail"])
        self._require_approval = config.get("require_creator_approval", SLOW_CYCLE_DEFAULTS["require_creator_approval"])
        self._snapshot = config.get("snapshot_before_change", SLOW_CYCLE_DEFAULTS["snapshot_before_change"])
        self._tasks: dict[str, dict] = {}
        self._last_tick: Optional[float] = None
        self._snapshots: list[dict] = []
        self._stats = {"ticks": 0, "executions": 0, "failures": 0, "pending_approvals": 0}
        self._diminishing_threshold: float = 0.01
        self._diminishing_window: int = 10
        self._capped_components: dict = {}

    def register(self, component_name: str, optimize_fn: Callable,
                 min_samples: Optional[int] = None,
                 require_approval: Optional[bool] = None):
        if component_name in self._capped_components:
            info = self._capped_components[component_name]
            unlock_at = info.get("unlock_at", 0)
            if time.time() < unlock_at:
                logger.info("SlowCycleScheduler: component '%s' is capped until %s, skipping registration",
                            component_name, unlock_at)
                return
            else:
                del self._capped_components[component_name]
        self._tasks[component_name] = {
            "fn": optimize_fn,
            "min_samples": min_samples or self._min_samples,
            "require_approval": require_approval if require_approval is not None else self._require_approval,
            "last_execution": 0.0,
            "execution_count": 0,
            "fail_count": 0,
            "enabled": True,
            "approved": not (require_approval if require_approval is not None else self._require_approval),
            "success_rates": [],
        }

    def approve(self, component_name: str):
        task = self._tasks.get(component_name)
        if task:
            task["approved"] = True
            self._stats["pending_approvals"] = max(0, self._stats["pending_approvals"] - 1)

    def _create_snapshot(self, component_name: str) -> dict:
        snap = {
            "component": component_name,
            "timestamp": time.time(),
            "state": f"snapshot_{component_name}_{time.time()}",
        }
        self._snapshots.append(snap)
        return snap

    def tick(self) -> list[dict]:
        now = time.time()
        self._last_tick = now
        self._stats["ticks"] += 1
        results = []
        for name, task in self._tasks.items():
            if not task["enabled"]:
                continue
            if now - task["last_execution"] < self._interval:
                continue
            if not task["approved"]:
                self._stats["pending_approvals"] += 1
                results.append({"component": name, "success": False,
                                "error": "awaiting_approval"})
                continue
            snapshot = None
            if self._snapshot:
                snapshot = self._create_snapshot(name)
            try:
                output = task["fn"]()
                task["execution_count"] += 1
                task["last_execution"] = now
                task["approved"] = False
                self._stats["executions"] += 1
                results.append({"component": name, "success": True, "output": output,
                                "snapshot": snapshot})
                self._check_diminishing_returns(name, 1.0 if output else 0.0)
            except Exception as e:
                task["fail_count"] += 1
                self._stats["failures"] += 1
                logger.warning("Slow cycle task %s failed: %s", name, e)
                results.append({"component": name, "success": False, "error": str(e),
                                "snapshot": snapshot})
                self._check_diminishing_returns(name, 0.0)
        return results

    def _check_diminishing_returns(self, component_name: str, success_rate: float) -> bool:
        task = self._tasks.get(component_name)
        if not task:
            return False
        rates = task.setdefault("success_rates", [])
        rates.append(success_rate)
        if len(rates) > self._diminishing_window:
            rates.pop(0)
        if len(rates) < self._diminishing_window:
            return False
        improvements = []
        for i in range(1, len(rates)):
            improvements.append(abs(rates[i] - rates[i-1]))
        avg_improvement = sum(improvements) / len(improvements)
        if avg_improvement < self._diminishing_threshold:
            now_ts = time.time()
            self._capped_components[component_name] = {
                "capped_at": now_ts,
                "unlock_at": now_ts + 90 * 86400,
            }
            task["enabled"] = False
            logger.info("SlowCycleScheduler: component '%s' capped due to diminishing returns (avg improvement: %.4f < %.4f)",
                        component_name, avg_improvement, self._diminishing_threshold)
            return True
        return False

    def get_capped_components(self) -> dict:
        return dict(self._capped_components)

    def get_status(self) -> dict:
        return {
            "interval": self._interval,
            "min_samples": self._min_samples,
            "rollback_on_fail": self._rollback,
            "require_creator_approval": self._require_approval,
            "snapshot_before_change": self._snapshot,
            "last_tick": self._last_tick,
            "snapshots_count": len(self._snapshots),
            "stats": dict(self._stats),
            "tasks": {
                name: {
                    "last_execution": t["last_execution"],
                    "execution_count": t["execution_count"],
                    "fail_count": t["fail_count"],
                    "enabled": t["enabled"],
                    "approved": t["approved"],
                }
                for name, t in self._tasks.items()
            },
            "capped_components": dict(self._capped_components),
        }


class EvolutionOrchestrator:
    def __init__(self, config: Optional[dict] = None):
        config = config or {}
        self.fast_cycle = FastCycleScheduler(config)
        self.slow_cycle = SlowCycleScheduler(config)
        self.fast_enabled = config.get("fast_cycle_enabled", True)
        self.slow_enabled = config.get("slow_cycle_enabled", True)

    def register_fast(self, component_name: str, optimize_fn: Callable,
                      min_samples: Optional[int] = None):
        self.fast_cycle.register(component_name, optimize_fn, min_samples)

    def register_slow(self, component_name: str, optimize_fn: Callable,
                      min_samples: Optional[int] = None,
                      require_approval: Optional[bool] = None):
        self.slow_cycle.register(component_name, optimize_fn, min_samples, require_approval)

    def tick_fast(self) -> list[dict]:
        if not self.fast_enabled:
            return []
        return self.fast_cycle.tick()

    def tick_slow(self) -> list[dict]:
        if not self.slow_enabled:
            return []
        return self.slow_cycle.tick()

    def get_status(self) -> dict:
        return {
            "fast_enabled": self.fast_enabled,
            "slow_enabled": self.slow_enabled,
            "fast_cycle": self.fast_cycle.get_status(),
            "slow_cycle": self.slow_cycle.get_status(),
        }

    def check_diminishing_returns(self, component_name: str, success_rate: float) -> bool:
        fast_capped = self.fast_cycle._check_diminishing_returns(component_name, success_rate)
        slow_capped = self.slow_cycle._check_diminishing_returns(component_name, success_rate)
        return fast_capped or slow_capped

    def get_capped_components(self) -> dict:
        merged = dict(self.fast_cycle.get_capped_components())
        for k, v in self.slow_cycle.get_capped_components().items():
            if k not in merged:
                merged[k] = v
        return merged
