import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.retrieval import RetrievalEngine


class TestEmotionalTemperature:
    def test_none_emotion_returns_empty(self):
        result = RetrievalEngine._apply_emotional_temperature(None)
        assert result == {}

    def test_all_zero_emotion_returns_default_weights(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": 0.0, "warmth": 0.0, "ripple": 0.0}
        )
        assert result == {"keyword": 1.0, "entity": 1.0, "semantic": 1.0,
                          "graph": 1.0, "causal": 1.0}

    def test_high_calmness_boosts_keyword_and_semantic(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": 0.8, "warmth": 0.0, "ripple": 0.0}
        )
        assert result["keyword"] == 1.05
        assert result["semantic"] == 1.05
        assert result["entity"] == 1.0
        assert result["graph"] == 1.0
        assert result["causal"] == 1.0

    def test_high_warmth_boosts_entity_and_graph(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": 0.0, "warmth": 0.8, "ripple": 0.0}
        )
        assert result["entity"] == 1.05
        assert result["graph"] == 1.05
        assert result["keyword"] == 1.0
        assert result["semantic"] == 1.0
        assert result["causal"] == 1.0

    def test_high_ripple_boosts_causal(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": 0.0, "warmth": 0.0, "ripple": 0.6}
        )
        assert result["causal"] == 1.08
        assert result["keyword"] == 1.0

    def test_multiple_conditions_simultaneous(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": 0.8, "warmth": 0.8, "ripple": 0.6}
        )
        assert result["keyword"] == 1.05
        assert result["semantic"] == 1.05
        assert result["entity"] == 1.05
        assert result["graph"] == 1.05
        assert result["causal"] == 1.08

    def test_all_channels_clamped_within_09_11(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": 0.9, "warmth": 0.9, "ripple": 0.9}
        )
        for k, v in result.items():
            assert 0.9 <= v <= 1.1, f"{k}={v} out of range"

    def test_extreme_values_are_clipped(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": 2.0, "warmth": 2.0, "ripple": 2.0}
        )
        for k, v in result.items():
            assert 0.9 <= v <= 1.1, f"{k}={v} out of range"

    def test_negative_values_handled(self):
        result = RetrievalEngine._apply_emotional_temperature(
            {"calmness": -0.5, "warmth": -0.5, "ripple": -0.5}
        )
        assert result == {"keyword": 1.0, "entity": 1.0, "semantic": 1.0,
                          "graph": 1.0, "causal": 1.0}