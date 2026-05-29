import uuid
from datetime import datetime, timezone


class EvolutionAuditor:
    def __init__(self, evolution_logger=None, config=None):
        self.evolution_logger = evolution_logger
        self.enabled = config.get("audit_enabled", True) if config else True
        self.fast_cycle_threshold = 10
        self.slow_cycle_threshold = 3
        self._fast_count = 0
        self._slow_count = 0
        self._last_report_time = 0
        self._report_history: list[dict] = []

    def record_change(self, cycle_type: str, change_data: dict) -> None:
        if not self.enabled:
            return
        if cycle_type == "fast":
            self._fast_count += 1
        elif cycle_type == "slow":
            self._slow_count += 1
        if self.evolution_logger:
            self.evolution_logger.log_change(cycle_type, change_data)

    def should_generate_report(self) -> bool:
        if not self.enabled:
            return False
        return self._fast_count >= self.fast_cycle_threshold or self._slow_count >= self.slow_cycle_threshold

    def generate_report(self) -> dict:
        changes = []
        if self.evolution_logger:
            changes = self.evolution_logger.get_recent_changes(count=100)

        total_fast = sum(1 for c in changes if c.get("cycle_type") == "fast")
        total_slow = sum(1 for c in changes if c.get("cycle_type") == "slow")
        effective = [c for c in changes if c.get("success") is True]
        ineffective = [c for c in changes if c.get("success") is False]
        effective_count = len(effective)
        ineffective_count = len(ineffective)
        total_changes = effective_count + ineffective_count
        effectiveness_rate = effective_count / total_changes if total_changes > 0 else 0.0

        by_component = {}
        for c in effective:
            comp = c.get("component", "unknown")
            by_component.setdefault(comp, {"success_count": 0, "fail_count": 0})
            by_component[comp]["success_count"] += 1
        for c in ineffective:
            comp = c.get("component", "unknown")
            by_component.setdefault(comp, {"success_count": 0, "fail_count": 0})
            by_component[comp]["fail_count"] += 1

        effective_patterns = [
            {"component": comp, "success_count": stats["success_count"], "pattern": f"{comp} changes tend to succeed"}
            for comp, stats in by_component.items()
            if stats["success_count"] > 0 and stats["success_count"] >= stats["fail_count"]
        ]

        ineffective_patterns = []
        for comp, stats in by_component.items():
            if stats["fail_count"] > 0 and stats["fail_count"] > stats["success_count"]:
                ineffective_patterns.append({
                    "component": comp,
                    "fail_count": stats["fail_count"],
                    "common_features": f"Component '{comp}' has high failure rate",
                })

        recommendations = []
        for ip in ineffective_patterns:
            recommendations.append(f"建议降低组件 {ip['component']} 的改动频率")
        if effectiveness_rate < 0.5:
            recommendations.append("整体有效率偏低，建议暂停并回顾进化策略")
        if not recommendations:
            recommendations.append("当前优化方向良好，继续保持")

        report = {
            "report_id": str(uuid.uuid4()),
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "period_summary": {
                "total_fast_changes": total_fast,
                "total_slow_changes": total_slow,
                "effective_changes": effective_count,
                "ineffective_changes": ineffective_count,
                "effectiveness_rate": round(effectiveness_rate, 2),
            },
            "effective_patterns": effective_patterns,
            "ineffective_patterns": ineffective_patterns,
            "recommendations": recommendations,
        }

        self._report_history.append(report)
        if self._fast_count >= self.fast_cycle_threshold:
            self._fast_count = 0
        if self._slow_count >= self.slow_cycle_threshold:
            self._slow_count = 0
        self._last_report_time = 0

        return report

    def get_report_history(self, limit=10) -> list[dict]:
        return self._report_history[-limit:]

    def get_summary_report(self) -> str:
        if not self._report_history:
            return "暂无审计报告"
        latest = self._report_history[-1]
        ps = latest["period_summary"]
        lines = [
            f"报告 ID: {latest['report_id']}",
            f"生成时间: {latest['generated_at']}",
            f"快周期改动: {ps['total_fast_changes']} 次",
            f"慢周期改动: {ps['total_slow_changes']} 次",
            f"有效改动: {ps['effective_changes']} 次",
            f"无效改动: {ps['ineffective_changes']} 次",
            f"有效率: {ps['effectiveness_rate'] * 100:.0f}%",
        ]
        if latest["recommendations"]:
            lines.append("建议:")
            for r in latest["recommendations"]:
                lines.append(f"  - {r}")
        return "\n".join(lines)


import json
import time
from pathlib import Path


class EvolutionLogger:
    def __init__(self, data_dir=None):
        self._storage_path = None
        if data_dir:
            self._storage_path = Path(data_dir) / "evolution" / "evolution_log.jsonl"
            self._storage_path.parent.mkdir(parents=True, exist_ok=True)

    def log(self, source, action, detail=None):
        event = {
            "timestamp": time.time(),
            "source": source,
            "action": action,
            "detail": detail or {},
        }
        if self._storage_path:
            with open(self._storage_path, "a") as f:
                f.write(json.dumps(event, ensure_ascii=False) + "\n")
        return event

    def get_log(self, limit=50, source=None):
        events = []
        if not self._storage_path or not self._storage_path.exists():
            return events
        with open(self._storage_path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                event = json.loads(line)
                if source is None or event.get("source") == source:
                    events.append(event)
        return events[-limit:]

    def log_change(self, cycle_type: str, change_data: dict) -> dict:
        event = self.log(
            source="audit",
            action="change",
            detail={"cycle_type": cycle_type, **change_data},
        )
        return event

    def get_recent_changes(self, count=50) -> list[dict]:
        all_events = self.get_log(limit=count * 10)
        changes = []
        for event in all_events:
            detail = event.get("detail", {})
            if detail.get("cycle_type") in ("fast", "slow"):
                entry = {
                    "cycle_type": detail["cycle_type"],
                    "component": detail.get("component", "unknown"),
                    "before": detail.get("before"),
                    "after": detail.get("after"),
                    "success": detail.get("success"),
                    "timestamp": event.get("timestamp", 0),
                }
                changes.append(entry)
        return changes[-count:]

    def get_change_stats(self) -> dict:
        changes = self.get_recent_changes(count=10000)
        total_fast = sum(1 for c in changes if c["cycle_type"] == "fast")
        total_slow = sum(1 for c in changes if c["cycle_type"] == "slow")
        effective = sum(1 for c in changes if c["success"] is True)
        ineffective = sum(1 for c in changes if c["success"] is False)
        by_component = {}
        for c in changes:
            comp = c["component"]
            if comp not in by_component:
                by_component[comp] = {"success": 0, "fail": 0}
            if c["success"] is True:
                by_component[comp]["success"] += 1
            elif c["success"] is False:
                by_component[comp]["fail"] += 1
        return {
            "total_fast": total_fast,
            "total_slow": total_slow,
            "effective": effective,
            "ineffective": ineffective,
            "by_component": by_component,
        }

    def get_stats(self):
        stats = {}
        if not self._storage_path or not self._storage_path.exists():
            return stats
        with open(self._storage_path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                event = json.loads(line)
                src = event.get("source", "unknown")
                stats[src] = stats.get(src, 0) + 1
        return stats