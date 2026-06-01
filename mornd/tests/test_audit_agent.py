import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.memory.audit_agent import AuditAgent


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_audit_") as tmpdir:
        yield Path(tmpdir)


@pytest.mark.asyncio
async def test_extract_simple_triple(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_test_001", "description": "创建者喜欢编程"}
        triples = await agent.extract_triples(capsule)
        assert len(triples) >= 1
        t = triples[0]
        assert t["subject"] == "创建者"
        assert t["relation"] == "喜欢"
        assert t["object"] == "编程"


@pytest.mark.asyncio
async def test_extract_multiple_triples(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_test_002", "description": "创建者喜欢编程，Morn是AI"}
        triples = await agent.extract_triples(capsule)
        assert len(triples) >= 1


@pytest.mark.asyncio
async def test_extract_empty_description(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        triples = await agent.extract_triples({"event_id": "evt_empty", "description": ""})
        assert len(triples) == 0


@pytest.mark.asyncio
async def test_dedup_same_event(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_dedup", "description": "创建者喜欢编程"}
        first = await agent.extract_triples(capsule)
        second = await agent.extract_triples(capsule)
        assert len(first) >= 1
        assert len(second) == 0


@pytest.mark.asyncio
async def test_audit_pass(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        triple = {
            "subject": "创建者", "relation": "喜欢", "object": "编程",
            "evidence": "创建者喜欢编程", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "pass"


@pytest.mark.asyncio
async def test_audit_fail(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        triple = {
            "subject": "外星人", "relation": "入侵", "object": "地球",
            "evidence": "不相关", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "fail"


@pytest.mark.asyncio
async def test_audit_uncertain(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        triple = {
            "subject": "创建者", "relation": "入侵", "object": "地球",
            "evidence": "创建者入侵", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "uncertain"


@pytest.mark.asyncio
async def test_audit_no_source(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        triple = {
            "subject": "创建者", "relation": "喜欢", "object": "编程",
            "evidence": "", "source_text": "",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "fail"


@pytest.mark.asyncio
async def test_extract_and_deposit_pass(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_deposit_001", "description": "创建者喜欢咖啡"}
        count = await agent.extract_and_deposit(capsule)
        assert count >= 1
        rows = await store.query_knowledge(subject="创建者")
        matching = [r for r in rows if r["object"] == "咖啡"]
        assert len(matching) >= 1


@pytest.mark.asyncio
async def test_extract_and_deposit_uncertain_marked_pending(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_uncertain", "description": "Morn会学习"}
        count = await agent.extract_and_deposit(capsule)
        assert count >= 1


@pytest.mark.asyncio
async def test_extract_evidence_referencing(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_evid_001", "description": "创建者说天气很好"}
        triples = await agent.extract_triples(capsule)
        assert len(triples) >= 1
        for t in triples:
            assert "evidence" in t
            assert t["source_event_id"] == "evt_evid_001"


@pytest.mark.asyncio
async def test_extract_reset_tracking(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_reset", "description": "创建者喜欢旅行"}
        first = await agent.extract_triples(capsule)
        assert len(first) >= 1
        second = await agent.extract_triples(capsule)
        assert len(second) == 0
        agent.reset_extraction_tracking()
        third = await agent.extract_triples(capsule)
        assert len(third) >= 1


@pytest.mark.asyncio
async def test_integration_full_pipeline(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        capsule = {"event_id": "evt_full_001", "description": "创建者是开发者，Morn是助手"}
        count = await agent.extract_and_deposit(capsule)
        assert count >= 1
        knowledge = await store.query_knowledge()
        assert len(knowledge) >= 1
