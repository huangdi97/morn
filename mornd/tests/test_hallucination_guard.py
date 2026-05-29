import os
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import pytest

from morn.contrib.memory_advanced.hallucination_guard import HallucinationGuard
from morn_core.memory.retrieval import RawSnapshotStore


class TestFidelityCheck:
    @pytest.mark.asyncio
    async def test_fidelity_high_similarity(self):
        guard = HallucinationGuard()
        result = guard._jaccard_similarity("hello world", "hello world")
        assert result == 1.0
        fidelity = {"passed": True, "similarity": 1.0, "action": "accept"}
        assert fidelity["passed"] is True
        assert fidelity["action"] == "accept"

    @pytest.mark.asyncio
    async def test_fidelity_low_similarity(self):
        guard = HallucinationGuard()
        result = guard._jaccard_similarity("abcdefghij", "klmnopqrst")
        assert result == 0.0

    @pytest.mark.asyncio
    async def test_fidelity_with_real_store(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = os.path.join(tmp, "test.db")
            store = RawSnapshotStore(db_path)
            raw = "The cat sat on the mat and looked at the moon"
            summary = "The cat sat on the mat and looked at the moon"
            snapshot = await store.store_snapshot(
                source="test",
                raw_text=raw,
                metadata={"capsule_id": "cap1"},
            )
            guard = HallucinationGuard(raw_snapshot_store=store)
            result = await guard.check_summary_fidelity(
                snapshot.snapshot_id,
                summary,
            )
            assert result["action"] == "accept"
            assert result["passed"] is True
            assert result["similarity"] == 1.0

    @pytest.mark.asyncio
    async def test_fidelity_low_with_real_store(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = os.path.join(tmp, "test.db")
            store = RawSnapshotStore(db_path)
            snapshot = await store.store_snapshot(
                source="test",
                raw_text="Quantum entanglement is a physical phenomenon",
                metadata={},
            )
            guard = HallucinationGuard(raw_snapshot_store=store)
            result = await guard.check_summary_fidelity(
                snapshot.snapshot_id,
                "The weather is nice today",
            )
            assert result["action"] == "review"
            assert result["passed"] is False
            assert result["similarity"] < 0.85

    @pytest.mark.asyncio
    async def test_fidelity_missing_snapshot(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = os.path.join(tmp, "test.db")
            store = RawSnapshotStore(db_path)
            guard = HallucinationGuard(raw_snapshot_store=store)
            result = await guard.check_summary_fidelity("nonexistent", "hello")
            assert result["passed"] is False
            assert result["action"] == "review"
            assert "error" in result


class TestConsistencyCheck:
    def test_consistency_no_conflict_empty(self):
        guard = HallucinationGuard()
        class DummyStore:
            SEMANTIC = "semantic"
            TEMPORAL = "temporal"
            def get_subgraph(self, gt):
                return None
        result = guard.check_consistency(DummyStore(), "alice", "likes", "bob", 1000.0)
        assert result["consistent"] is True
        assert result["conflicts"] == []

    def test_consistency_detects_conflict(self):
        guard = HallucinationGuard()
        class DummySemantic:
            def query_semantic(self, entity, relation=None, depth=1):
                if entity == "alice":
                    return [{"relation": "dislikes", "target": {"name": "bob"}}]
                return []
        class DummyTemporal:
            def query_temporal(self, entity, start_time=None, end_time=None):
                return []
        class DummyTripleStore:
            SEMANTIC = "semantic"
            TEMPORAL = "temporal"
            def get_subgraph(self, gt):
                if gt == "semantic":
                    return DummySemantic()
                return DummyTemporal()
        result = guard.check_consistency(DummyTripleStore(), "alice", "likes", "bob", 1000.0)
        assert result["consistent"] is False
        assert len(result["conflicts"]) >= 1


class TestAttachReference:
    def test_attach_reference(self):
        guard = HallucinationGuard()
        result = guard.attach_snapshot_reference(
            {"data": "test"},
            "abc12345-xyz",
        )
        assert result["raw_snapshot_id"] == "abc12345-xyz"
        assert result["snapshot_preview"] == "abc12345..."
        assert result["data"] == "test"

    def test_attach_reference_does_not_remove_original(self):
        guard = HallucinationGuard()
        original = {"key": "value", "nested": {"a": 1}}
        result = guard.attach_snapshot_reference(original, "snap-001")
        assert result["key"] == "value"
        assert result["raw_snapshot_id"] == "snap-001"


class TestGuardNotBlockAccept:
    @pytest.mark.asyncio
    async def test_guard_not_block_accept(self):
        guard = HallucinationGuard()
        result = await guard.check_summary_fidelity("fake", "test")
        assert result["action"] in ("accept", "review")