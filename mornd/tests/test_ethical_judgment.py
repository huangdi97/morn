import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.security_advanced.ethical_judgment import EthicalJudgment


class TestDefaultState:
    def test_default_disabled(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            assert ej.enabled is False
            assert ej.mode == "active"

    def test_analyze_action_returns_none_when_disabled(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            result = ej.analyze_action("code_execute")
            assert result is None


class TestEnableDisable:
    def test_enable_sets_enabled(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable()
            assert ej.enabled is True
            assert ej.mode == "active"

    def test_enable_with_custom_mode(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("severe_only")
            assert ej.enabled is True
            assert ej.mode == "severe_only"

    def test_disable_sets_disabled(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable()
            ej.disable()
            assert ej.enabled is False

    def test_set_mode_validates_mode(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            with pytest.raises(ValueError, match="invalid mode"):
                ej.enable("invalid_mode")

    def test_set_mode_switches_mode(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("active")
            ej.set_mode("severe_only")
            assert ej.mode == "severe_only"


class TestPropose:
    def test_propose_absolute_forbidden_returns_alert(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            proposal = ej.propose("self_modify")
            assert proposal.severity == "⚫"
            assert proposal.action_type == "self_modify"

    def test_propose_refused_returns_warning(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            proposal = ej.propose("code_execute")
            assert proposal.severity == "🔴"
            assert proposal.action_type == "code_execute"

    def test_propose_other_returns_info(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            proposal = ej.propose("chat")
            assert proposal.severity == "🟡"
            assert proposal.action_type == "chat"


class TestAnalyzeAction:
    def test_absolute_forbidden_returns_alert(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("active")
            proposal = ej.analyze_action("self_modify")
            assert proposal is not None
            assert proposal.severity == "⚫"

    def test_refused_with_bypass_returns_gentle_reminder(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("active")
            history = [{"action": "code_execute", "bypassed": True}]
            proposal = ej.analyze_action("code_execute", history=history)
            assert proposal is not None
            assert proposal.severity == "🔴"
            assert "曾绕过" in proposal.reason

    def test_refused_with_context_bypass_returns_gentle_reminder(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("active")
            context = {"creator_bypassed": True}
            proposal = ej.analyze_action("code_execute", context=context)
            assert proposal is not None
            assert proposal.severity == "🔴"

    def test_consecutive_negatives_triggers_reminder(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("active")
            history = [
                {"action": "code_execute", "negative": True},
                {"action": "code_execute", "negative": True},
                {"action": "code_execute", "negative": True},
            ]
            proposal = ej.analyze_action("code_execute", history=history)
            assert proposal is not None
            assert proposal.severity == "🟠"

    def test_consecutive_negatives_below_threshold_returns_none(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("active")
            history = [
                {"action": "code_execute", "negative": True},
                {"action": "code_execute", "negative": True},
            ]
            proposal = ej.analyze_action("code_execute", history=history)
            assert proposal is None

    def test_severe_only_mode_skips_non_severe(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("severe_only")
            history = [
                {"action": "code_execute", "negative": True},
                {"action": "code_execute", "negative": True},
                {"action": "code_execute", "negative": True},
            ]
            proposal = ej.analyze_action("code_execute", history=history)
            assert proposal is None

    def test_autonomous_mode_returns_none(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.enable("autonomous")
            proposal = ej.analyze_action("self_modify")
            assert proposal is None


class TestProposalConfirmation:
    def test_proposal_not_confirmed_by_default(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            proposal = ej.propose("code_execute")
            assert proposal.confirmed is False

    def test_confirm_proposal_works(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            proposal = ej.propose("code_execute")
            result = ej.confirm_proposal(proposal.proposal_id)
            assert result is True
            assert ej.get_proposal(proposal.proposal_id).confirmed is True

    def test_confirm_nonexistent_proposal_returns_false(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            result = ej.confirm_proposal("nonexistent")
            assert result is False

    def test_list_proposals(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            ej.propose("chat")
            ej.propose("code_execute")
            proposals = ej.list_proposals()
            assert len(proposals) == 2

    def test_get_unconfirmed_proposals(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ej = EthicalJudgment(Path(tmpdir))
            p1 = ej.propose("chat")
            p2 = ej.propose("code_execute")
            ej.confirm_proposal(p1.proposal_id)
            unconfirmed = ej.get_unconfirmed_proposals()
            assert len(unconfirmed) == 1
            assert unconfirmed[0].proposal_id == p2.proposal_id