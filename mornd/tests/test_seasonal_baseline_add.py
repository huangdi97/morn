import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.emotion.bond_tracker import BondTracker


def _history(days: int, dim: str, value: float) -> list[dict]:
    return [{dim: value} for _ in range(days)]


class TestSeasonalBaseline:
    @pytest.mark.asyncio
    async def test_less_than_30_days_no_adjustment(self):
        b = BondTracker({})
        history = _history(29, "calmness", 0.9)
        result = await b.seasonal_baseline_adjust(emotion_history=history)
        assert result == {}

    @pytest.mark.asyncio
    async def test_30_days_deviation_adjusts_baseline(self):
        b = BondTracker({})
        history = _history(30, "calmness", 0.9)
        result = await b.seasonal_baseline_adjust(emotion_history=history)
        assert "calmness" in result
        assert result["calmness"] == 0.02

    @pytest.mark.asyncio
    async def test_adjustment_capped_at_30_percent(self):
        b = BondTracker({})
        b._bond = 0.5
        for _ in range(100):
            history = _history(30, "calmness", 0.95)
            result = await b.seasonal_baseline_adjust(emotion_history=history)
            if not result:
                break
        total_adj = sum(b._seasonal_adjustments.values())
        assert abs(total_adj) <= 0.21

    @pytest.mark.asyncio
    async def test_seasonal_adjustments_recorded_correctly(self):
        b = BondTracker({})
        history = _history(30, "warmth", 0.8)
        result = await b.seasonal_baseline_adjust(emotion_history=history)
        assert "warmth" in b._seasonal_adjustments
        adj = b._seasonal_adjustments["warmth"]
        assert adj == 0.02

    @pytest.mark.asyncio
    async def test_bond_value_includes_adjustment(self):
        b = BondTracker({"initial_bond": 0.1})
        initial = b.get_bond()
        history = _history(30, "calmness", 0.9)
        await b.seasonal_baseline_adjust(emotion_history=history)
        adjusted = b.get_bond()
        assert adjusted != initial

    @pytest.mark.asyncio
    async def test_baseline_negative_deviation(self):
        b = BondTracker({})
        history = _history(30, "calmness", 0.5)
        result = await b.seasonal_baseline_adjust(emotion_history=history)
        assert "calmness" in result
        assert result["calmness"] == -0.02

    @pytest.mark.asyncio
    async def test_no_adjustment_when_deviation_below_threshold(self):
        b = BondTracker({})
        history = _history(30, "calmness", 0.71)
        result = await b.seasonal_baseline_adjust(emotion_history=history)
        assert result == {}