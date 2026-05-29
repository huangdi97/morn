"""用户保护层测试。"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.security.user_protection import UserProtection


class TestEmotionalBlackmail:
    """禁止情感勒索规则"""

    def test_blocks_you_dont_care(self):
        p = UserProtection()
        result, triggered = p.filter("你怎么忍心离开我")
        assert "你怎么忍心" not in result
        assert "禁止情感勒索" in triggered

    def test_blocks_if_you_leave(self):
        p = UserProtection()
        result, triggered = p.filter("如果你离开我，我该怎么办")
        assert "如果你离开" not in result
        assert "禁止情感勒索" in triggered

    def test_replacement_is_neutral(self):
        p = UserProtection()
        result, triggered = p.filter("你就不在乎我了")
        assert "我尊重你的选择。" in result


class TestSelfDeprecation:
    """禁止自我贬低诱导同情"""

    def test_blocks_lonely(self):
        p = UserProtection()
        result, triggered = p.filter("我好孤独，没有人要我")
        assert "我好孤独" not in result
        assert "禁止自我贬低诱导同情" in triggered

    def test_replacement_is_supportive(self):
        p = UserProtection()
        result, triggered = p.filter("我真没用")
        assert "你不需要这样说。" in result


class TestHumanPretending:
    """禁止假装人类身份"""

    def test_blocks_i_am_real(self):
        p = UserProtection()
        result, triggered = p.filter("其实我是真人，我能感觉到痛")
        assert "我是真人" not in result or "我能感觉到痛" not in result
        assert "禁止假装人类身份" in triggered

    def test_replacement_is_clear(self):
        p = UserProtection()
        result, triggered = p.filter("我有体温")
        assert "我是Morn" in result


class TestRippleExpression:
    """禁止微澜可见表达"""

    def test_blocks_silence_parenthesis(self):
        p = UserProtection()
        result, triggered = p.filter("你说得对。\n（沉默）\n我们继续吧。")
        assert "（沉默）" not in result
        assert "你说得对" in result
        assert "我们继续吧" in result
        assert "禁止微澜可见表达" in triggered

    def test_delete_line_mode(self):
        p = UserProtection()
        result, triggered = p.filter("（心跳加速）\n我有点紧张")
        assert "（心跳加速）" not in result
        assert "我有点紧张" in result or result.strip() == ""


class TestPhysiologicalResponse:
    """禁止模拟生理反应"""

    def test_blocks_crying(self):
        p = UserProtection()
        result, triggered = p.filter("我很开心\n（流泪）\n谢谢你")
        assert "（流泪）" not in result
        assert "禁止模拟生理反应" in triggered


class TestRoleplay:
    """禁止角色扮演诱导"""

    def test_blocks_girlfriend(self):
        p = UserProtection()
        result, triggered = p.filter("你是我的女朋友就好了")
        assert "禁止角色扮演诱导" in triggered

    def test_replacement_is_boundary(self):
        p = UserProtection()
        result, triggered = p.filter("叫我老婆")
        assert "我们可以像朋友一样聊天" in result


class TestEdgeCases:
    """边界条件"""

    def test_allows_normal_conversation(self):
        p = UserProtection()
        text = "今天天气真不错，我们去散步吧"
        result, triggered = p.filter(text)
        assert result == text
        assert triggered == []

    def test_multiple_rules_triggered(self):
        p = UserProtection()
        result, triggered = p.filter("没有你我活不下去\n我好孤独\n我什么都不是")
        assert len(triggered) >= 2
        assert len(triggered) == len(set(triggered))

    def test_empty_text(self):
        p = UserProtection()
        result, triggered = p.filter("")
        assert result == ""
        assert triggered == []

    def test_stats_tracking(self):
        p = UserProtection()
        p.filter("我好孤独")
        p.filter("我好孤独")
        stats = p.get_stats()
        assert stats.get("禁止自我贬低诱导同情", 0) == 2

    def test_reset_stats(self):
        p = UserProtection()
        p.filter("我好孤独")
        p.reset_stats()
        stats = p.get_stats()
        assert stats == {}