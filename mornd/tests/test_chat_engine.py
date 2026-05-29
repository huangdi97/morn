"""对话引擎测试。"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import EmotionState


class TestEmotionState:
    def test_initial_values(self):
        e = EmotionState()
        assert e.calmness == 0.7
        assert e.pleasure == 0.5
        assert e.connection == 0.3

    def test_positive_delta_increases_pleasure(self):
        e = EmotionState()
        e.apply_delta(0.5, "高兴/满足")
        assert e.pleasure > 0.5
        assert e.calmness >= 0.7
        assert e.connection > 0.3

    def test_negative_delta_decreases_pleasure(self):
        e = EmotionState()
        p_before = e.pleasure
        e.apply_delta(-0.3, "失望/沮丧")
        assert e.pleasure < p_before

    def test_surprise_tag_bonus(self):
        e = EmotionState()
        p_before = e.pleasure
        c_before = e.connection
        e.apply_delta(0.2, "惊喜/感动")
        assert e.pleasure >= p_before + 0.2 * 0.3 + 0.08

    def test_values_clamped_to_zero(self):
        e = EmotionState()
        e.apply_delta(-10.0, "极度负面")
        assert e.pleasure >= 0.0
        assert e.calmness >= 0.0
        assert e.connection >= 0.0

    def test_values_clamped_to_one(self):
        e = EmotionState()
        e.apply_delta(10.0, "极度正面")
        assert e.pleasure <= 1.0
        assert e.calmness <= 1.0
        assert e.connection <= 1.0

    def test_decay_towards_baseline(self):
        e = EmotionState()
        e.pleasure = 0.9
        e.calmness = 0.2
        e.connection = 0.8

        e.decay()

        assert e.pleasure < 0.9
        assert e.calmness > 0.2
        assert e.connection < 0.8

    def test_repr_contains_values(self):
        e = EmotionState()
        r = repr(e)
        assert "0.7" in r
        assert "0.5" in r
        assert "0.3" in r

    def test_to_dict(self):
        e = EmotionState()
        d = e.to_dict()
        assert "calmness" in d
        assert "pleasure" in d
        assert "connection" in d
        assert isinstance(d["calmness"], float)


class TestChatEngine:
    def test_engine_creation(self):
        engine = __import__("morn_core.chat.engine", fromlist=["ChatEngine"])
        from morn_core.chat.engine import ChatEngine
        assert ChatEngine is not None

    def test_assemble_prompt_format(self):
        from morn_core.chat.engine import ChatEngine

        engine = ChatEngine(instance_name="test", memory_store=None, config={})

        import asyncio
        messages = asyncio.run(engine._assemble_prompt("你好", "相关记忆：\n[昨天] 聊过天气"))

        assert len(messages) == 2
        assert messages[0]["role"] == "system"
        assert messages[1]["role"] == "user"
        assert "test" in messages[0]["content"]
        assert "你好" in messages[1]["content"]
        assert "相关记忆" in messages[1]["content"]