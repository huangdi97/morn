import json
import os
import sys
from datetime import datetime, timezone

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.memory.audit_agent import AuditAgent


def _make_snapshot_id(i):
    return f"snap_self_heal_{i:03d}"


def _make_event_id(i):
    return f"evt_self_heal_{i:03d}"


@pytest.mark.asyncio
async def test_self_heal_no_capsules_returns_zero(data_dir):
    async with MemoryStore(data_dir) as store:
        agent = AuditAgent(store)
        result = await agent.self_heal_scan()
        assert result == 0


@pytest.mark.asyncio
async def test_self_heal_matching_description_not_modified(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = _make_event_id(1)
        await store.add_capsule({
            "event_id": eid,
            "entities": "[]",
            "description": "创建者喜欢编程",
        })
        raw_meta = json.dumps({"capsule_id": eid})
        now_str = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")
        await store.db.execute(
            "INSERT INTO raw_snapshots (snapshot_id, source, raw_text, sha256_hash, timestamp, metadata) VALUES (?, ?, ?, ?, ?, ?)",
            (_make_snapshot_id(1), "chat", "创建者喜欢编程", "hash1", now_str, raw_meta),
        )
        await store.db.commit()
        agent = AuditAgent(store)
        result = await agent.self_heal_scan()
        assert result == 0


@pytest.mark.asyncio
async def test_self_heal_divergent_description_skips_without_llm(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = _make_event_id(2)
        await store.add_capsule({
            "event_id": eid,
            "entities": "[]",
            "description": "完全不相关的内容描述",
        })
        raw_meta = json.dumps({"capsule_id": eid})
        now_str = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")
        await store.db.execute(
            "INSERT INTO raw_snapshots (snapshot_id, source, raw_text, sha256_hash, timestamp, metadata) VALUES (?, ?, ?, ?, ?, ?)",
            (_make_snapshot_id(2), "chat", "创建者喜欢编程和阅读", "hash2", now_str, raw_meta),
        )
        await store.db.commit()
        agent = AuditAgent(store)
        result = await agent.self_heal_scan()
        assert result == 0


@pytest.mark.asyncio
async def test_self_heal_no_raw_text_adds_to_unhealable(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = _make_event_id(3)
        await store.add_capsule({
            "event_id": eid,
            "entities": "[]",
            "description": "一些内容",
        })
        raw_meta = json.dumps({"capsule_id": eid})
        now_str = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")
        await store.db.execute(
            "INSERT INTO raw_snapshots (snapshot_id, source, raw_text, sha256_hash, timestamp, metadata) VALUES (?, ?, ?, ?, ?, ?)",
            (_make_snapshot_id(3), "chat", "", "hash3", now_str, raw_meta),
        )
        await store.db.commit()
        agent = AuditAgent(store)
        result = await agent.self_heal_scan()
        assert result == 0
        unhealable = await agent.get_unhealable()
        assert len(unhealable) >= 1
        assert _make_snapshot_id(3) in unhealable


@pytest.mark.asyncio
async def test_get_unhealable_returns_correct_list(data_dir):
    async with MemoryStore(data_dir) as store:
        snap_ids = []
        for i in range(3):
            eid = _make_event_id(10 + i)
            await store.add_capsule({
                "event_id": eid,
                "entities": "[]",
                "description": f"内容{i}",
            })
            raw_meta = json.dumps({"capsule_id": eid})
            now_str = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")
            sid = _make_snapshot_id(10 + i)
            snap_ids.append(sid)
            await store.db.execute(
                "INSERT INTO raw_snapshots (snapshot_id, source, raw_text, sha256_hash, timestamp, metadata) VALUES (?, ?, ?, ?, ?, ?)",
                (sid, "chat", "", f"hash{10+i}", now_str, raw_meta),
            )
        await store.db.commit()
        agent = AuditAgent(store)
        await agent.self_heal_scan()
        unhealable = await agent.get_unhealable()
        for sid in snap_ids:
            assert sid in unhealable


@pytest.mark.asyncio
async def test_self_heal_disabled_returns_zero(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = _make_event_id(20)
        await store.add_capsule({
            "event_id": eid,
            "entities": "[]",
            "description": "任何内容",
        })
        raw_meta = json.dumps({"capsule_id": eid})
        now_str = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")
        await store.db.execute(
            "INSERT INTO raw_snapshots (snapshot_id, source, raw_text, sha256_hash, timestamp, metadata) VALUES (?, ?, ?, ?, ?, ?)",
            (_make_snapshot_id(20), "chat", "可能有问题的文本", "hash20", now_str, raw_meta),
        )
        await store.db.commit()
        agent = AuditAgent(store, config={"heal_enabled": False})
        result = await agent.self_heal_scan()
        assert result == 0
        unhealable = await agent.get_unhealable()
        assert len(unhealable) == 0