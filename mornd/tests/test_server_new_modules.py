import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))


class TestNewModuleImports:
    def test_bond_tracker_import(self):
        from morn_core.emotion.bond_tracker import BondTracker
        assert BondTracker is not None

    def test_retrieval_engine_import(self):
        from morn_core.memory.retrieval import RetrievalEngine, LayeredRetrievalEngine
        assert RetrievalEngine is not None
        assert LayeredRetrievalEngine is not None

    def test_dynamic_permissions_import(self):
        from morn.contrib.security_advanced.risk_guard import DynamicPermissions
        assert DynamicPermissions is not None

    def test_external_boundary_import(self):
        from morn_core.security.external_boundary import ExternalBoundary
        assert ExternalBoundary is not None

    def test_intent_drift_import(self):
        from morn.contrib.security_advanced.ethical_judgment import IntentDriftDetector
        assert IntentDriftDetector is not None

    def test_challenge_mode_import(self):
        from morn_core.consciousness.self_pruning import ChallengeMode
        assert ChallengeMode is not None

    def test_audit_agent_import(self):
        from morn_core.memory.audit_agent import AuditAgent
        assert AuditAgent is not None


class TestMornStateNewFields:
    def test_morn_state_has_new_fields(self):
        from morn_core.server import MornState
        state = MornState()
        assert hasattr(state, "bond_tracker")
        assert hasattr(state, "retrieval_engine")
        assert hasattr(state, "layered_retrieval")
        assert hasattr(state, "dynamic_permissions")
        assert hasattr(state, "external_boundary")
        assert hasattr(state, "intent_drift_detector")
        assert hasattr(state, "challenge_mode")
        assert hasattr(state, "audit_agent")

    def test_new_fields_default_to_none(self):
        from morn_core.server import MornState
        state = MornState()
        assert state.bond_tracker is None
        assert state.retrieval_engine is None
        assert state.layered_retrieval is None
        assert state.dynamic_permissions is None
        assert state.external_boundary is None
        assert state.intent_drift_detector is None
        assert state.challenge_mode is None
        assert state.audit_agent is None


class TestInitSubsystems:
    def test_init_bond_tracker_no_error(self):
        from morn_core.emotion.bond_tracker import BondTracker
        bt = BondTracker({})
        assert bt.get_bond() == 0.1
        assert bt.get_stage() == "初识期"

    def test_init_dynamic_permissions_no_error(self):
        from morn.contrib.security_advanced.risk_guard import DynamicPermissions
        with tempfile.TemporaryDirectory() as tmpdir:
            dp = DynamicPermissions(Path(tmpdir))
            assert dp.get_permission("chat") == "🟢"
            assert dp.get_permission("code_execute") == "🔴"

    def test_init_external_boundary_no_error(self):
        from morn_core.security.external_boundary import ExternalBoundary
        with tempfile.TemporaryDirectory() as tmpdir:
            eb = ExternalBoundary(Path(tmpdir))
            assert eb.check_inbound("tcp", 80, "1.2.3.4") is False

    def test_init_intent_drift_detector_no_error(self):
        from morn.contrib.security_advanced.ethical_judgment import IntentDriftDetector
        detector = IntentDriftDetector()
        assert detector.get_drift_score() == 0.0
        assert detector.get_alerts() == []


class TestConfigDisable:
    def test_config_defaults_have_new_modules_enabled(self):
        from morn_core.server import load_config
        with tempfile.TemporaryDirectory() as tmpdir:
            cfg = load_config(Path(tmpdir))
        assert cfg.get("bond_tracker_enabled") is True
        assert cfg.get("retrieval_engine_enabled") is True
        assert cfg.get("layered_retrieval_enabled") is True
        assert cfg.get("dynamic_permissions_enabled") is True
        assert cfg.get("external_boundary_enabled") is True
        assert cfg.get("intent_drift_detector_enabled") is True
        assert cfg.get("challenge_mode_enabled") is True
        assert cfg.get("audit_agent_enabled") is True


class TestBackgroundLoops:
    def test_bond_update_loop_exists(self):
        from morn_core.server import bond_update_loop
        import asyncio
        assert asyncio.iscoroutinefunction(bond_update_loop)

    def test_intent_drift_loop_exists(self):
        from morn_core.server import intent_drift_loop
        import asyncio
        assert asyncio.iscoroutinefunction(intent_drift_loop)

    def test_audit_loop_exists(self):
        from morn_core.server import audit_loop
        import asyncio
        assert asyncio.iscoroutinefunction(audit_loop)


class TestLoopShutdown:
    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_bond_loop_exits_on_shutdown(self, mock_log):
        from morn_core.server import bond_update_loop, MornState
        state = MornState()
        state.shutdown = True
        await bond_update_loop(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_drift_loop_exits_on_shutdown(self, mock_log):
        from morn_core.server import intent_drift_loop, MornState
        state = MornState()
        state.shutdown = True
        await intent_drift_loop(state)

    @pytest.mark.asyncio
    @patch("morn_core.server.logging.getLogger")
    async def test_audit_loop_exits_on_shutdown(self, mock_log):
        from morn_core.server import audit_loop, MornState
        state = MornState()
        state.shutdown = True
        await audit_loop(state)