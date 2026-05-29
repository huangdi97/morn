"""情感delta解析测试。"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import ChatEngine, EmotionState


class TestParseEmotionDelta:
    def test_parse_single_delta(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:pleasure:+0.1]"
        )
        assert result == {"pleasure": 0.1}

    def test_parse_multiple_deltas(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:pleasure:+0.1,determination:-0.05]"
        )
        assert result == {"pleasure": 0.1, "determination": -0.05}

    def test_no_delta_marker(self):
        result = ChatEngine._parse_emotion_delta(
            "这是一个普通回复"
        )
        assert result == {}

    def test_empty_delta_marker(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:]"
        )
        assert result == {}

    def test_invalid_dimension_name(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:unknown:+0.1]"
        )
        assert result == {}

    def test_invalid_delta_value(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:pleasure:abc]"
        )
        assert result == {}

    def test_delta_out_of_range(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:pleasure:+2.0]"
        )
        assert result == {}

    def test_mixed_valid_and_invalid(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:pleasure:+0.1,unknown:+0.2]"
        )
        assert result == {"pleasure": 0.1}

    def test_parse_warmth_and_ripple(self):
        result = ChatEngine._parse_emotion_delta(
            "[emotion:warmth:+0.05,ripple:+0.08]"
        )
        assert result == {"warmth": 0.05, "ripple": 0.08}


class TestDeltaStateConsistency:
    def test_delta_applied_within_bounds(self):
        e = EmotionState()
        e.calmness = 0.5
        delta = ChatEngine._parse_emotion_delta(
            "[emotion:calmness:-0.6]"
        )
        e.calmness += delta.get("calmness", 0)
        e._clamp()
        assert 0.0 <= e.calmness <= 1.0

    def test_multiple_deltas_and_decay_evolution(self):
        e = EmotionState()
        e.pleasure = 0.5
        e.connection = 0.3
        e.warmth = 0.5

        deltas = [
            {"pleasure": 0.1, "connection": 0.05, "warmth": 0.08},
            {"pleasure": -0.05},
            {"connection": 0.03},
        ]
        for d in deltas:
            for dim, val in d.items():
                setattr(e, dim, getattr(e, dim) + val)
            e._clamp()
            e.decay()

        assert 0.0 <= e.pleasure <= 1.0
        assert 0.0 <= e.connection <= 1.0
        assert 0.0 <= e.warmth <= 1.0