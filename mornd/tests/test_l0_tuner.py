import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.l0_tuner import L0Tuner


class TestWeightUpdate:
    def test_initial_weights(self):
        tuner = L0Tuner()
        w = tuner.get_weights()
        assert w["recency"] == 0.4
        assert w["frequency"] == 0.3
        assert w["emotional"] == 0.2
        assert w["relevance"] == 0.1

    def test_tune_increases_emotional_weight(self):
        tuner = L0Tuner({"learning_rate": 0.1})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.5})
        w = tuner.get_weights()
        assert w["emotional"] > 0.2

    def test_tune_increases_frequency_weight(self):
        tuner = L0Tuner({"learning_rate": 0.1})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.5, "frequency_avg": 0.9})
        w = tuner.get_weights()
        assert w["frequency"] > 0.3

    def test_tune_low_emotional_decreases_weight(self):
        tuner = L0Tuner({"learning_rate": 0.1})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.1, "frequency_avg": 0.5})
        w = tuner.get_weights()
        assert w["emotional"] < 0.2

    def test_weights_normalized(self):
        tuner = L0Tuner({"learning_rate": 0.05})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.8, "frequency_avg": 0.7})
        w = tuner.get_weights()
        total = sum(w.values())
        assert abs(total - 1.0) < 1e-6

    def test_tune_not_enabled_does_nothing(self):
        tuner = L0Tuner({"enabled": False})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.9})
        w = tuner.get_weights()
        assert w["recency"] == 0.4
        assert w["frequency"] == 0.3
        assert w["emotional"] == 0.2


class TestDecayAdjustment:
    def test_initial_decay_params(self):
        tuner = L0Tuner()
        d = tuner.get_decay_params()
        assert d["base_decay"] == 0.1
        assert d["frequency_decay"] == 0.05
        assert d["emotional_boost"] == 0.2

    def test_high_emotion_raises_emotional_boost(self):
        tuner = L0Tuner({"learning_rate": 0.1})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.5})
        d = tuner.get_decay_params()
        assert d["emotional_boost"] > 0.2

    def test_low_emotion_lowers_emotional_boost(self):
        tuner = L0Tuner({"learning_rate": 0.1})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.1, "frequency_avg": 0.5})
        d = tuner.get_decay_params()
        assert d["emotional_boost"] < 0.2

    def test_high_frequency_raises_base_decay(self):
        tuner = L0Tuner({"learning_rate": 0.1})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 0.5, "frequency_avg": 0.9})
        d = tuner.get_decay_params()
        assert d["base_decay"] > 0.1


class TestConfigLoading:
    def test_default_config(self):
        tuner = L0Tuner()
        assert tuner.enabled is True
        assert tuner.learning_rate == 0.01
        assert tuner.min_weight == 0.1
        assert tuner.max_weight == 1.0

    def test_custom_config(self):
        tuner = L0Tuner({
            "enabled": False,
            "learning_rate": 0.5,
            "min_weight": 0.05,
            "max_weight": 2.0,
        })
        assert tuner.enabled is False
        assert tuner.learning_rate == 0.5
        assert tuner.min_weight == 0.05
        assert tuner.max_weight == 2.0

    def test_partial_config(self):
        tuner = L0Tuner({"learning_rate": 0.2})
        assert tuner.enabled is True
        assert tuner.learning_rate == 0.2
        assert tuner.min_weight == 0.1
        assert tuner.max_weight == 1.0

    def test_tune_interval_from_config(self):
        tuner = L0Tuner({"tune_interval": 5})
        assert tuner._tune_interval == 5


class TestBoundaryConditions:
    def test_tune_with_empty_stats(self):
        tuner = L0Tuner({"learning_rate": 0.1})
        for _ in range(10):
            tuner.tune({})
        w = tuner.get_weights()
        assert abs(sum(w.values()) - 1.0) < 1e-6

    def test_weights_clamped_min(self):
        tuner = L0Tuner({"min_weight": 0.3, "max_weight": 0.7, "learning_rate": 0.5})
        for _ in range(20):
            tuner.tune({"emotional_importance_avg": 0.1, "frequency_avg": 0.1})
        w = tuner.get_weights()
        assert abs(sum(w.values()) - 1.0) < 1e-6
        assert all(v <= 0.7 + 1e-6 for v in w.values())

    def test_weights_clamped_max(self):
        tuner = L0Tuner({"min_weight": 0.1, "max_weight": 0.5, "learning_rate": 0.5})
        for _ in range(20):
            tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.9})
        w = tuner.get_weights()
        assert abs(sum(w.values()) - 1.0) < 1e-6
        assert all(v <= 0.5 + 1e-6 for v in w.values())

    def test_clamp_directly(self):
        tuner = L0Tuner({"min_weight": 0.2, "max_weight": 0.8})
        assert tuner._clamp(0.5) == 0.5
        assert tuner._clamp(0.1) == 0.2
        assert tuner._clamp(1.0) == 0.8
        assert tuner._clamp(0.0, 0.1, 0.5) == 0.1
        assert tuner._clamp(0.9, 0.1, 0.5) == 0.5

    def test_decay_params_clamped(self):
        tuner = L0Tuner({"learning_rate": 10.0})
        for _ in range(10):
            tuner.tune({"emotional_importance_avg": 1.0, "frequency_avg": 1.0})
        d = tuner.get_decay_params()
        assert 0.0 <= d["emotional_boost"] <= 1.0
        assert 0.01 <= d["base_decay"] <= 1.0

    def test_tune_interval_skip(self):
        tuner = L0Tuner({"tune_interval": 10, "learning_rate": 0.1})
        tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.9})
        w_before = tuner.get_weights()
        assert tuner._access_count == 1
        tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.9})
        assert tuner._access_count == 2
        tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.9})
        assert tuner._access_count == 3
    def test_tune_interval_fires(self):
        tuner = L0Tuner({"tune_interval": 3, "learning_rate": 0.1})
        for _ in range(3):
            tuner.tune({"emotional_importance_avg": 0.9, "frequency_avg": 0.5})
        w = tuner.get_weights()
        assert w["emotional"] > 0.2
