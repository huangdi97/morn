class L0Tuner:
    def __init__(self, config=None):
        config = config or {}
        self.enabled = config.get("enabled", True)
        self.learning_rate = config.get("learning_rate", 0.01)
        self.min_weight = config.get("min_weight", 0.1)
        self.max_weight = config.get("max_weight", 1.0)
        self._tune_interval = config.get("tune_interval", 10)
        self._access_count = 0
        self._weights = [0.4, 0.3, 0.2, 0.1]
        self._decay_params = [0.1, 0.05, 0.2]

    def tune(self, memory_stats):
        if not self.enabled:
            return
        self._access_count += 1
        if self._access_count % self._tune_interval != 0:
            return

        emotional_avg = memory_stats.get("emotional_importance_avg", 0.5)
        frequency_avg = memory_stats.get("frequency_avg", 0.5)

        emotional_delta = (emotional_avg - 0.5) * self.learning_rate
        self._weights[2] = self._clamp(self._weights[2] + emotional_delta)

        freq_delta = (frequency_avg - 0.5) * self.learning_rate
        self._weights[1] = self._clamp(self._weights[1] + freq_delta)

        total = sum(self._weights)
        self._weights = [w / total for w in self._weights]

        self._decay_params[2] = self._clamp(
            self._decay_params[2] + emotional_delta * 0.5, 0.0, 1.0
        )
        self._decay_params[0] = self._clamp(
            self._decay_params[0] + freq_delta * 0.3, 0.01, 1.0
        )

    def _clamp(self, value, lo=None, hi=None):
        lo = self.min_weight if lo is None else lo
        hi = self.max_weight if hi is None else hi
        return max(lo, min(hi, value))

    def get_weights(self):
        clamped = [self._clamp(v) for v in self._weights]
        total = sum(clamped)
        if total > 0:
            clamped = [v / total for v in clamped]
        return dict(zip(
            ["recency", "frequency", "emotional", "relevance"],
            clamped
        ))

    def get_decay_params(self):
        return dict(zip(
            ["base_decay", "frequency_decay", "emotional_boost"],
            self._decay_params
        ))


import json
import time
import uuid
from pathlib import Path


class ThinkingStyleEvolver:
    def __init__(self, config=None):
        config = config or {}
        self.enabled = config.get("enabled", False)
        self.data_dir = config.get("data_dir")
        self._templates = []
        self._storage_path = None
        if self.data_dir:
            self._storage_path = Path(self.data_dir) / "evolution" / "thinking_styles.json"
            self._load()

    def register_thought(self, pattern, steps, conditions=None, name=None):
        if not self.enabled:
            return None
        template = {
            "template_id": str(uuid.uuid4()),
            "name": name or f"template_{len(self._templates) + 1}",
            "trigger_conditions": conditions or [],
            "reasoning_steps": steps if isinstance(steps, list) else [steps],
            "success_count": 0,
            "fail_count": 0,
            "priority": 1.0,
            "created_at": time.time(),
            "last_used": 0.0,
        }
        if pattern:
            template["trigger_conditions"].append(pattern)
        self._templates.append(template)
        self._save()
        return template["template_id"]

    def get_matching(self, context):
        if not self.enabled or not self._templates:
            return []
        scored = []
        for t in self._templates:
            score = 0.0
            for cond in t["trigger_conditions"]:
                if isinstance(cond, str):
                    if cond.lower() in context.lower():
                        score += 1.0
                elif callable(cond):
                    if cond(context):
                        score += 1.0
            if score > 0:
                scored.append((score * t["priority"], t))
        scored.sort(key=lambda x: -x[0])
        return [t for _, t in scored]

    def record_outcome(self, template_id, success):
        if not self.enabled:
            return
        for t in self._templates:
            if t["template_id"] == template_id:
                if success:
                    t["success_count"] += 1
                else:
                    t["fail_count"] += 1
                t["last_used"] = time.time()
                self._save()
                return

    def evolve(self):
        if not self.enabled or not self._templates:
            return []
        events = []
        events.extend(self._revise())
        events.extend(self._recombine())
        events.extend(self._refine())
        if events:
            self._save()
        return events

    def _revise(self):
        events = []
        snap = list(self._templates)
        for t in snap:
            if t["fail_count"] > 0 and t["fail_count"] >= t["success_count"]:
                revised = {
                    "template_id": str(uuid.uuid4()),
                    "name": t["name"] + "_revised",
                    "trigger_conditions": list(t["trigger_conditions"]),
                    "reasoning_steps": list(t["reasoning_steps"]),
                    "success_count": 0,
                    "fail_count": 0,
                    "priority": t["priority"] * 0.8,
                    "created_at": time.time(),
                    "last_used": 0.0,
                }
                self._templates.append(revised)
                events.append({
                    "action": "revise",
                    "source_id": t["template_id"],
                    "new_id": revised["template_id"],
                })
        return events

    def _recombine(self):
        events = []
        snap = list(self._templates)
        if len(snap) < 2:
            return events
        for i in range(len(snap)):
            for j in range(i + 1, len(snap)):
                a, b = snap[i], snap[j]
                common = set(a["trigger_conditions"]) & set(b["trigger_conditions"])
                if len(common) >= 1:
                    mid = len(a["reasoning_steps"]) // 2
                    new_steps = a["reasoning_steps"][:mid] + b["reasoning_steps"][mid:]
                    new = {
                        "template_id": str(uuid.uuid4()),
                        "name": f"{a['name']}_x_{b['name']}",
                        "trigger_conditions": list(common),
                        "reasoning_steps": new_steps,
                        "success_count": 0,
                        "fail_count": 0,
                        "priority": max(a["priority"], b["priority"]) * 0.9,
                        "created_at": time.time(),
                        "last_used": 0.0,
                    }
                    self._templates.append(new)
                    events.append({
                        "action": "recombine",
                        "source_ids": [a["template_id"], b["template_id"]],
                        "new_id": new["template_id"],
                    })
        return events

    def _refine(self):
        events = []
        snap = list(self._templates)
        for t in snap:
            if t["success_count"] >= 3:
                old = t["priority"]
                t["priority"] = min(t["priority"] * 1.5, 10.0)
                events.append({
                    "action": "refine",
                    "template_id": t["template_id"],
                    "priority_from": old,
                    "priority_to": t["priority"],
                })
        return events

    def get_templates(self):
        return list(self._templates)

    def _save(self):
        if not self._storage_path:
            return
        self._storage_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self._storage_path, "w") as f:
            json.dump(self._templates, f, indent=2)

    def _load(self):
        if self._storage_path and self._storage_path.exists():
            with open(self._storage_path) as f:
                self._templates = json.load(f)
