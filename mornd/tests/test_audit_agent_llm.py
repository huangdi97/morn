import json
import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.memory.audit_agent import AuditAgent


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_audit_llm_") as tmpdir:
        yield Path(tmpdir)


def _make_llm_ok(verdict="pass", reason="verified by LLM"):
    async def llm_caller(messages):
        return json.dumps({"verdict": verdict, "reason": reason})
    return llm_caller


def _make_llm_extract_ok(triples=None):
    if triples is None:
        triples = [
            {"subject": "创建者", "relation": "喜欢", "object": "AI",
             "confidence": 0.9, "evidence": "创建者喜欢AI"}
        ]

    async def llm_caller(messages):
        return json.dumps(triples)
    return llm_caller


def _make_llm_fail():
    async def llm_caller(messages):
        raise RuntimeError("LLM unavailable")
    return llm_caller


@pytest.mark.asyncio
async def test_llm_audit_pass(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_ok("pass"))
        triple = {
            "subject": "创建者", "relation": "喜欢", "object": "编程",
            "evidence": "创建者喜欢编程", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "pass"
        assert result["reason"] == "verified by LLM"


@pytest.mark.asyncio
async def test_llm_audit_fail(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_ok("fail", "not supported"))
        triple = {
            "subject": "外星人", "relation": "入侵", "object": "地球",
            "evidence": "不相关", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "fail"


@pytest.mark.asyncio
async def test_llm_audit_uncertain(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_ok("uncertain", "partial match"))
        triple = {
            "subject": "创建者", "relation": "入侵", "object": "地球",
            "evidence": "创建者入侵", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "uncertain"


@pytest.mark.asyncio
async def test_llm_audit_fallback_to_rule_engine(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_fail())
        triple = {
            "subject": "创建者", "relation": "喜欢", "object": "编程",
            "evidence": "创建者喜欢编程", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "pass"
        assert "LLM" not in result.get("reason", "")


@pytest.mark.asyncio
async def test_llm_extract_triples(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_extract_ok())
        capsule = {"event_id": "evt_llm_001", "description": "创建者喜欢AI"}
        triples = await agent.extract_triples(capsule)
        assert len(triples) >= 1
        assert triples[0]["subject"] == "创建者"
        assert triples[0]["relation"] == "喜欢"
        assert triples[0]["object"] == "AI"


@pytest.mark.asyncio
async def test_llm_extract_fallback_to_regex(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_fail())
        capsule = {"event_id": "evt_fallback_001", "description": "创建者喜欢编程"}
        triples = await agent.extract_triples(capsule)
        assert len(triples) >= 1
        assert triples[0]["subject"] == "创建者"
        assert triples[0]["relation"] == "喜欢"
        assert triples[0]["object"] == "编程"


@pytest.mark.asyncio
async def test_llm_audit_disabled_uses_rule_engine(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_ok("pass"), config={"llm_audit_enabled": False})
        assert agent.llm_audit_enabled is False
        triple = {
            "subject": "创建者", "relation": "喜欢", "object": "编程",
            "evidence": "创建者喜欢编程", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "pass"
        assert "LLM" not in result.get("reason", "")


@pytest.mark.asyncio
async def test_llm_extract_disabled_uses_regex(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_extract_ok(), config={"llm_extract_enabled": False})
        assert agent.llm_extract_enabled is False
        capsule = {"event_id": "evt_disable_001", "description": "创建者是开发者"}
        triples = await agent.extract_triples(capsule)
        assert len(triples) >= 1
        assert triples[0]["relation"] == "是"


@pytest.mark.asyncio
async def test_llm_audit_invalid_verdict_normalized(data_dir):
    async with MemoryStore(data_dir) as store:
        async def bad_llm(messages):
            return '{"verdict": "invalid_status", "reason": "test"}'

        agent = AuditAgent(store, llm_caller=bad_llm)
        triple = {
            "subject": "创建者", "relation": "喜欢", "object": "编程",
            "evidence": "创建者喜欢编程", "source_text": "创建者喜欢编程",
        }
        result = await agent.audit(triple)
        assert result["verdict"] == "uncertain"


@pytest.mark.asyncio
async def test_llm_extract_empty_description_returns_empty(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store, llm_caller=_make_llm_extract_ok())
        triples = await agent._llm_extract("")
        assert triples == []