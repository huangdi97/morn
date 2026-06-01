import json
import time
from pathlib import Path


class HarnessOptimizer:
    def __init__(self, config=None):
        config = config or {}
        self.enabled = config.get("enabled", False)
        self.data_dir = config.get("data_dir")
        self._metrics = {
            "prompt_quality": {"total": 0, "success": 0, "fail": 0},
            "tool_efficiency": {"total_calls": 0, "total_time": 0.0, "success": 0, "fail": 0},
            "memory_params": {"total_retrievals": 0, "adopted": 0, "ignored": 0},
        }
        self._log = []
        self._storage_path = None
        if self.data_dir:
            self._storage_path = Path(self.data_dir) / "evolution" / "harness_log.json"
            self._load()

    def collect_metrics(self, source, data):
        if not self.enabled:
            return
        if source == "prompt":
            self._metrics["prompt_quality"]["total"] += 1
            if data.get("success"):
                self._metrics["prompt_quality"]["success"] += 1
            else:
                self._metrics["prompt_quality"]["fail"] += 1
        elif source == "tool":
            self._metrics["tool_efficiency"]["total_calls"] += 1
            self._metrics["tool_efficiency"]["total_time"] += data.get("duration", 0.0)
            if data.get("success"):
                self._metrics["tool_efficiency"]["success"] += 1
            else:
                self._metrics["tool_efficiency"]["fail"] += 1
        elif source == "memory":
            self._metrics["memory_params"]["total_retrievals"] += 1
            if data.get("adopted"):
                self._metrics["memory_params"]["adopted"] += 1
            else:
                self._metrics["memory_params"]["ignored"] += 1
        self._save()

    def diagnose(self):
        if not self.enabled:
            return []
        issues = []
        pq = self._metrics["prompt_quality"]
        if pq["total"] >= 5:
            rate = pq["success"] / pq["total"]
            if rate < 0.5:
                issues.append({
                    "type": "prompt_quality",
                    "severity": "high" if rate < 0.3 else "medium",
                    "metric": "success_rate",
                    "value": rate,
                    "detail": "提示词命中率偏低",
                })

        te = self._metrics["tool_efficiency"]
        if te["total_calls"] >= 5:
            fail_rate = te["fail"] / te["total_calls"]
            if fail_rate > 0.3:
                issues.append({
                    "type": "tool_efficiency",
                    "severity": "high" if fail_rate > 0.5 else "medium",
                    "metric": "fail_rate",
                    "value": fail_rate,
                    "detail": "工具调用失败率偏高",
                })
            avg_time = te["total_time"] / te["total_calls"]
            if avg_time > 5.0:
                issues.append({
                    "type": "tool_efficiency",
                    "severity": "low",
                    "metric": "avg_duration",
                    "value": avg_time,
                    "detail": "工具平均耗时偏高",
                })

        mp = self._metrics["memory_params"]
        if mp["total_retrievals"] >= 5:
            adopt_rate = mp["adopted"] / mp["total_retrievals"]
            if adopt_rate < 0.3:
                issues.append({
                    "type": "memory_params",
                    "severity": "medium",
                    "metric": "adoption_rate",
                    "value": adopt_rate,
                    "detail": "记忆检索采纳率偏低",
                })
        return issues

    def optimize(self, target):
        if not self.enabled:
            return []
        suggestions = []
        if target == "prompt" or target == "all":
            pq = self._metrics["prompt_quality"]
            if pq["total"] >= 5:
                rate = pq["success"] / pq["total"]
                if rate < 0.5:
                    suggestions.append({
                        "target": "prompt",
                        "action": "reduce_system_prompt_length",
                        "params": {"max_length": 2000},
                        "reason": "提示词命中率不足50%",
                    })
                    suggestions.append({
                        "target": "prompt",
                        "action": "increase_example_count",
                        "params": {"example_count": 3},
                        "reason": "增加示例提升命中率",
                    })
        if target == "tool" or target == "all":
            te = self._metrics["tool_efficiency"]
            if te["total_calls"] >= 5:
                fail_rate = te["fail"] / te["total_calls"]
                if fail_rate > 0.3:
                    suggestions.append({
                        "target": "tool",
                        "action": "increase_retry_count",
                        "params": {"max_retries": 3},
                        "reason": "工具失败率高",
                    })
                    suggestions.append({
                        "target": "tool",
                        "action": "increase_timeout",
                        "params": {"timeout": 30},
                        "reason": "工具超时率高",
                    })
        if target == "memory" or target == "all":
            mp = self._metrics["memory_params"]
            if mp["total_retrievals"] >= 5:
                rate = mp["adopted"] / mp["total_retrievals"]
                if rate < 0.3:
                    suggestions.append({
                        "target": "memory",
                        "action": "increase_recency_weight",
                        "params": {"recency_weight": 0.5},
                        "reason": "记忆采纳率低，增加时效性权重",
                    })
                    suggestions.append({
                        "target": "memory",
                        "action": "reduce_result_count",
                        "params": {"max_results": 3},
                        "reason": "减少检索数量提高精度",
                    })
        return suggestions

    def get_metrics(self):
        return dict(self._metrics)

    def _save(self):
        if not self._storage_path:
            return
        payload = {
            "timestamp": time.time(),
            "metrics": self._metrics,
            "log": self._log[-100:],
        }
        self._storage_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self._storage_path, "w") as f:
            json.dump(payload, f, indent=2)

    def _load(self):
        if self._storage_path and self._storage_path.exists():
            with open(self._storage_path) as f:
                data = json.load(f)
                self._metrics = data.get("metrics", self._metrics)
                self._log = data.get("log", [])