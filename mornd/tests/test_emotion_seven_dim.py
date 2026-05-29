"""7维情感系统测试。"""

import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import EmotionState


class TestSevenDimInitialValues:
    def test_all_seven_dims_defaults(self):
        e = EmotionState()
        assert e.calmness == 0.7
        assert e.pleasure == 0.5
        assert e.connection == 0.3
        assert e.determination == 0.6
        assert e.anticipation == 0.4
        assert e.warmth == 0.5
        assert e.ripple == 0.2


class TestSevenDimDelta:
    def test_positive_delta_increases_warmth(self):
        e = EmotionState()
        e.apply_delta(0.5, "亲切")
        assert e.warmth > 0.5

    def test_positive_delta_increases_ripple(self):
        e = EmotionState()
        e.apply_delta(0.5, "意外")
        assert e.ripple > 0.2

    def test_negative_delta_decreases_warmth(self):
        e = EmotionState()
        w_before = e.warmth
        e.apply_delta(-0.3, "冷淡")
        assert e.warmth < w_before

    def test_negative_delta_decreases_ripple(self):
        e = EmotionState()
        r_before = e.ripple
        e.apply_delta(-0.3, "平淡")
        assert e.ripple < r_before

    def test_delta_coefficients_warmth_positive(self):
        e = EmotionState()
        e.warmth = 0.5
        e.apply_delta(0.2, "")
        assert e.warmth == pytest.approx(0.5 + 0.2 * 0.15, abs=0.001)

    def test_delta_coefficients_warmth_negative(self):
        e = EmotionState()
        e.warmth = 0.5
        e.apply_delta(-0.2, "")
        assert e.warmth == pytest.approx(0.5 + (-0.2) * 0.1, abs=0.001)

    def test_delta_coefficients_ripple_positive(self):
        e = EmotionState()
        e.ripple = 0.2
        e.apply_delta(0.2, "")
        assert e.ripple == pytest.approx(0.2 + 0.2 * 0.05, abs=0.001)

    def test_delta_coefficients_ripple_negative(self):
        e = EmotionState()
        e.ripple = 0.2
        e.apply_delta(-0.2, "")
        assert e.ripple == pytest.approx(0.2 + (-0.2) * 0.08, abs=0.001)


class TestRippleSpecialRules:
    def test_trigger_ripple_increases_by_0_08(self):
        e = EmotionState()
        r_before = e.ripple
        e.trigger_ripple()
        assert e.ripple == r_before + 0.08

    def test_ripple_above_0_5_reduces_calmness_on_decay(self):
        e = EmotionState()
        e.ripple = 0.6
        c_before = e.calmness
        e.decay()
        assert e.calmness < c_before - 0.04

    def test_ripple_below_0_5_does_not_affect_calmness_on_decay(self):
        e = EmotionState()
        e.ripple = 0.4
        e.calmness = 0.7
        e.decay()
        assert e.calmness >= 0.7

    def test_ripple_decays_slowest(self):
        e = EmotionState()
        e.ripple = 1.0
        e.warmth = 1.0
        e.anticipation = 1.0
        for _ in range(5):
            e.decay()
        assert e.ripple > e.warmth
        assert e.ripple > e.anticipation

    def test_ripple_not_in_describe_state(self):
        e = EmotionState()
        desc = e.describe_state()
        assert "微澜" not in desc
        assert "ripple" not in desc


class TestSevenDimDecay:
    def test_warmth_decays_towards_baseline(self):
        e = EmotionState()
        e.warmth = 0.9
        e.decay()
        assert e.warmth < 0.9

    def test_ripple_decays_towards_baseline(self):
        e = EmotionState()
        e.ripple = 0.9
        e.decay()
        assert e.ripple < 0.9

    def test_all_seven_decay_rates(self):
        e = EmotionState()
        e.calmness = 0.0
        e.pleasure = 0.0
        e.connection = 0.0
        e.determination = 0.0
        e.anticipation = 0.0
        e.warmth = 0.0
        e.ripple = 0.0
        for _ in range(10):
            e.decay()
        assert e.calmness > e.pleasure
        assert e.pleasure > e.determination
        assert e.determination > e.anticipation


class TestSevenDimClamping:
    def test_warmth_clamped_to_zero(self):
        e = EmotionState()
        e.warmth = -0.5
        e._clamp()
        assert e.warmth == 0.0

    def test_warmth_clamped_to_one(self):
        e = EmotionState()
        e.warmth = 1.5
        e._clamp()
        assert e.warmth == 1.0

    def test_ripple_clamped_to_zero(self):
        e = EmotionState()
        e.ripple = -0.5
        e._clamp()
        assert e.ripple == 0.0

    def test_ripple_clamped_to_one(self):
        e = EmotionState()
        e.ripple = 1.5
        e._clamp()
        assert e.ripple == 1.0


class TestSevenDimBackwardCompat:
    def test_from_dict_old_three_dim(self):
        old_data = {"calmness": 0.8, "pleasure": 0.6, "connection": 0.4}
        e = EmotionState.from_dict(old_data)
        assert e.calmness == 0.8
        assert e.pleasure == 0.6
        assert e.connection == 0.4
        assert e.determination == 0.6
        assert e.anticipation == 0.4
        assert e.warmth == 0.5
        assert e.ripple == 0.2

    def test_from_dict_old_five_dim(self):
        old_data = {
            "calmness": 0.2, "pleasure": 0.3, "connection": 0.5,
            "determination": 0.7, "anticipation": 0.8,
        }
        e = EmotionState.from_dict(old_data)
        assert e.calmness == 0.2
        assert e.pleasure == 0.3
        assert e.connection == 0.5
        assert e.determination == 0.7
        assert e.anticipation == 0.8
        assert e.warmth == 0.5
        assert e.ripple == 0.2

    def test_to_dict_includes_all_seven(self):
        e = EmotionState()
        d = e.to_dict()
        assert "warmth" in d
        assert "ripple" in d
        assert len(d) == 7


class TestSevenDimRepr:
    def test_repr_stays_three_dim(self):
        e = EmotionState()
        r = repr(e)
        assert "0.7" in r
        assert "0.5" in r
        assert "0.3" in r
        assert "warmth" not in r
        assert "ripple" not in r

    def test_seven_dimension_repr(self):
        e = EmotionState()
        r = e.seven_dimension_repr()
        assert "warmth=" in r
        assert "ripple=" in r
        assert "calmness=" in r


class TestDescribeStateWithWarmth:
    def test_describe_state_contains_warmth(self):
        e = EmotionState()
        desc = e.describe_state()
        assert "温暖" in desc or "温暖" in desc