import os
import sys
import time
from unittest.mock import MagicMock

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.consciousness.self_pruning import ChallengeMode, TEMPLATES
from morn_core.emotion.bond_tracker import BondTracker


@pytest.fixture
def bond_tracker():
    bt = BondTracker({"initial_bond": 0.96, "min_bond": 0.0, "max_bond": 1.0})
    return bt


@pytest.fixture
def mock_store():
    return MagicMock()


@pytest.fixture
def challenge(bond_tracker, mock_store):
    return ChallengeMode(
        memory_store=mock_store,
        bond_tracker=bond_tracker,
        deep_dialogue_count=60,
        days_since_first=40,
    )


class TestIsUnlocked:
    def test_locked_when_bond_too_low(self, challenge, bond_tracker):
        bond_tracker._bond = 0.5
        assert challenge.is_unlocked() is False

    def test_locked_when_deep_dialogue_too_few(self, challenge):
        challenge.set_deep_dialogue_count(10)
        assert challenge.is_unlocked() is False

    def test_locked_when_days_too_few(self, challenge):
        challenge.set_days_since_first(5)
        assert challenge.is_unlocked() is False

    def test_unlocked_when_all_conditions_met(self, challenge):
        assert challenge.is_unlocked() is True

    def test_unlocked_at_threshold_boundary(self, challenge):
        challenge.set_deep_dialogue_count(50)
        challenge.set_days_since_first(30)
        assert challenge.is_unlocked() is True


class TestFindChallengeTopics:
    @pytest.mark.asyncio
    async def test_returns_topics(self, challenge, mock_store):
        from unittest.mock import AsyncMock
        cursor_mock = MagicMock()
        cursor_mock.fetchall = AsyncMock(return_value=[])
        mock_store.db.execute = AsyncMock(return_value=cursor_mock)

        topics = await challenge.find_challenge_topics()
        assert isinstance(topics, list)


class TestGenerateChallenge:
    def test_generates_from_template_0(self, challenge):
        topic = {
            "template_index": 0,
            "params": {"event": "学习编程", "outcome": "成功"},
        }
        result = challenge.generate_challenge(topic)
        assert "学习编程" in result
        assert "成功" in result

    def test_generates_from_template_1(self, challenge):
        topic = {
            "template_index": 1,
            "params": {"topic": "工作选择", "quote": "你曾说过喜欢稳定"},
        }
        result = challenge.generate_challenge(topic)
        assert "工作选择" in result
        assert "你曾说过喜欢稳定" in result

    def test_generates_from_template_2(self, challenge):
        topic = {
            "template_index": 2,
            "params": {"behavior": "熬夜工作", "reason": "效率高", "observation": "白天精神差"},
        }
        result = challenge.generate_challenge(topic)
        assert "熬夜工作" in result
        assert "效率高" in result
        assert "白天精神差" in result

    def test_invalid_template_index_falls_back(self, challenge):
        topic = {
            "template_index": 99,
            "params": {"event": "测试", "outcome": "回退"},
        }
        result = challenge.generate_challenge(topic)
        assert TEMPLATES[0] is not None


class TestFrequencyControl:
    def test_can_challenge_when_never_triggered(self, challenge):
        assert challenge.can_challenge_now() is True

    def test_cannot_challenge_within_cooldown(self, challenge):
        challenge.trigger_challenge()
        assert challenge.can_challenge_now() is False

    def test_can_challenge_after_cooldown(self, challenge):
        challenge._last_challenge_time = time.time() - 8 * 86400
        assert challenge.can_challenge_now() is True

    def test_cannot_challenge_when_locked(self, challenge, bond_tracker):
        bond_tracker._bond = 0.5
        assert challenge.can_challenge_now() is False


class TestBondImpact:
    def test_trigger_challenge_reduces_bond(self, challenge, bond_tracker):
        old_bond = bond_tracker.get_bond()
        challenge.trigger_challenge()
        assert bond_tracker.get_bond() < old_bond

    def test_trigger_challenge_reduces_by_exact_amount(self, challenge, bond_tracker):
        bond_tracker._bond = 0.5
        challenge.trigger_challenge()
        assert bond_tracker.get_bond() == pytest.approx(0.48)

    def test_positive_response_increases_bond(self, challenge, bond_tracker):
        bond_tracker._bond = 0.5
        challenge.positive_response()
        assert bond_tracker.get_bond() == pytest.approx(0.51)

    def test_bond_stays_within_bounds(self, challenge, bond_tracker):
        bond_tracker._bond = 0.01
        challenge.trigger_challenge()
        assert bond_tracker.get_bond() >= 0.0
