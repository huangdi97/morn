import json
import logging
from pathlib import Path

logger = logging.getLogger("morn.bond")


class BondTracker:
    def __init__(self, config: dict):
        self._initial_bond = float(config.get("initial_bond", 0.1))
        self._min_bond = float(config.get("min_bond", 0.0))
        self._max_bond = float(config.get("max_bond", 1.0))
        self._growth_rate = float(config.get("growth_rate", 0.01))
        self._decay_rate = float(config.get("decay_rate", 0.002))
        self._bond = self._initial_bond
        self._data_dir = None
        self._personality_dir = None
        self._seasonal_adjustments: dict = {}

    def get_bond(self) -> float:
        total_adjust = sum(self._seasonal_adjustments.values())
        return max(self._min_bond, min(self._max_bond, self._bond + total_adjust))

    def update(self, interaction_depth: float, sentiment_score: float,
               days_since_first: float) -> float:
        growth = (interaction_depth * 0.5 + sentiment_score * 0.3
                  + min(days_since_first * 0.0005, 0.05))
        self._bond += growth * self._growth_rate
        if sentiment_score < 0.2:
            self._bond -= self._decay_rate
        elif sentiment_score < 0.4:
            self._bond -= self._decay_rate * 0.5
        self._bond = max(self._min_bond, min(self._max_bond, self._bond))
        return self._bond

    def get_stage(self) -> str:
        if self._bond < 0.3:
            return "初识期"
        if self._bond >= 0.7:
            return "默契期"
        return "亲近期"

    async def seasonal_baseline_adjust(self, emotion_history: list[dict] = None) -> dict:
        adjustments = {}
        if not emotion_history or len(emotion_history) < 30:
            return adjustments
        recent = emotion_history[-30:]
        dims = ["calmness", "pleasure", "warmth", "connection"]
        baselines = {"calmness": 0.7, "pleasure": 0.5, "connection": 0.3, "warmth": 0.5}
        for dim in dims:
            baseline = baselines.get(dim, 0.5)
            recent_vals = [e.get(dim, baseline) for e in recent if isinstance(e, dict)]
            if len(recent_vals) < 30:
                continue
            avg = sum(recent_vals) / len(recent_vals)
            deviation = avg - baseline
            if abs(deviation) < 0.02:
                continue
            direction = 1 if deviation > 0 else -1
            current_adj = self._seasonal_adjustments.get(dim, 0.0)
            capped_adj = abs(current_adj)
            max_allowed = baseline * 0.3
            if capped_adj >= max_allowed:
                continue
            delta = 0.02 * direction
            new_adj = current_adj + delta
            if abs(new_adj) > max_allowed:
                delta = (max_allowed if direction > 0 else -max_allowed) - current_adj
                new_adj = current_adj + delta
            if abs(delta) < 0.001:
                continue
            self._seasonal_adjustments[dim] = new_adj
            adjustments[dim] = round(delta, 4)
            logger.info("seasonal_baseline_adjust: %s adjusted by %.4f (total: %.4f)", dim, delta, new_adj)
        return adjustments

    def can_challenge(self) -> bool:
        return self._bond >= 0.7

    def set_data_dir(self, data_dir: Path):
        self._data_dir = Path(data_dir)
        self._personality_dir = self._data_dir / "personality"
        self._personality_dir.mkdir(parents=True, exist_ok=True)

    def _bond_path(self) -> Path:
        if self._personality_dir is None:
            return Path("bond.json")
        return self._personality_dir / "bond.json"

    def save(self):
        path = self._bond_path()
        data = {"bond": round(self._bond, 4)}
        with open(path, "w") as f:
            json.dump(data, f, indent=2)

    def load(self):
        path = self._bond_path()
        if not path.exists():
            self._bond = self._initial_bond
            return
        try:
            with open(path) as f:
                data = json.load(f)
            self._bond = float(data.get("bond", self._initial_bond))
        except (json.JSONDecodeError, IOError):
            self._bond = self._initial_bond