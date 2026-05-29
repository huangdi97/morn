"""五维情感系统测试。"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import EmotionState


class TestFiveDimInitialValues:
    def test_new_dimensions_have_defaults(self):
        e = EmotionState()
        assert e.determination == 0.6
        assert e.anticipation == 0.4

    def test_three_dimensions_unchanged(self):
        e = EmotionState()
        assert e.calmness == 0.7
        assert e.pleasure == 0.5
        assert e.connection == 0.3


class TestFiveDimDeltaApplication:
    def test_positive_delta_increases_determination(self):
        e = EmotionState()
        e.apply_delta(0.5, "高兴/满足")
        assert e.determination > 0.6

    def test_positive_delta_increases_anticipation(self):
        e = EmotionState()
        e.apply_delta(0.5, "高兴/满足")
        assert e.anticipation > 0.4

    def test_negative_delta_decreases_determination(self):
        e = EmotionState()
        d_before = e.determination
        e.apply_delta(-0.3, "失望/沮丧")
        assert e.determination < d_before

    def test_negative_delta_decreases_anticipation(self):
        e = EmotionState()
        a_before = e.anticipation
        e.apply_delta(-0.3, "失望/沮丧")
        assert e.anticipation < a_before


class TestFiveDimSurpriseBonus:
    def test_surprise_tag_boosts_all_five_dims(self):
        e = EmotionState()
        e.calmness = 0.5
        e.pleasure = 0.5
        e.connection = 0.5
        e.determination = 0.5
        e.anticipation = 0.5
        e.apply_delta(0.0, "惊喜/感动")
        assert e.calmness == 0.6
        assert e.pleasure == 0.6
        assert e.connection == 0.6
        assert e.determination == 0.6
        assert e.anticipation == 0.6


class TestFiveDimDecay:
    def test_determination_decays_towards_baseline(self):
        e = EmotionState()
        e.determination = 0.9
        e.decay()
        assert e.determination < 0.9

    def test_anticipation_decays_towards_baseline(self):
        e = EmotionState()
        e.anticipation = 0.0
        e.decay()
        assert e.anticipation > 0.0

    def test_determination_decays_faster_than_anticipation(self):
        e = EmotionState()
        e.determination = 0.9
        e.anticipation = 0.9
        for _ in range(10):
            e.decay()
        assert e.determination < e.anticipation

    def test_anticipation_decays_faster_than_connection(self):
        e = EmotionState()
        e.anticipation = 0.9
        e.connection = 0.9
        for _ in range(20):
            e.decay()
        assert e.anticipation < e.connection


class TestFiveDimClamping:
    def test_determination_clamped_to_zero(self):
        e = EmotionState()
        e.determination = -0.5
        e._clamp()
        assert e.determination == 0.0

    def test_anticipation_clamped_to_one(self):
        e = EmotionState()
        e.anticipation = 1.5
        e._clamp()
        assert e.anticipation == 1.0


class TestFiveDimRepr:
    def test_five_dimension_repr_contains_all_values(self):
        e = EmotionState()
        r = e.five_dimension_repr()
        assert f"{e.calmness:.2f}" in r
        assert f"{e.pleasure:.2f}" in r
        assert f"{e.connection:.2f}" in r
        assert f"{e.determination:.2f}" in r
        assert f"{e.anticipation:.2f}" in r


class TestToDict:
    def test_to_dict_includes_new_dimensions(self):
        e = EmotionState()
        d = e.to_dict()
        assert "determination" in d
        assert "anticipation" in d
        assert isinstance(d["determination"], float)
        assert isinstance(d["anticipation"], float)


class TestBackwardCompatible:
    def test_from_dict_old_three_dim(self):
        old_data = {"calmness": 0.8, "pleasure": 0.6, "connection": 0.4}
        e = EmotionState.from_dict(old_data)
        assert e.calmness == 0.8
        assert e.pleasure == 0.6
        assert e.connection == 0.4
        assert e.determination == 0.6
        assert e.anticipation == 0.4

    def test_from_dict_five_dim_overrides_all(self):
        data = {
            "calmness": 0.2,
            "pleasure": 0.3,
            "connection": 0.5,
            "determination": 0.7,
            "anticipation": 0.8,
        }
        e = EmotionState.from_dict(data)
        assert e.calmness == 0.2
        assert e.pleasure == 0.3
        assert e.connection == 0.5
        assert e.determination == 0.7
        assert e.anticipation == 0.8
