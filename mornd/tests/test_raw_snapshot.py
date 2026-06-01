import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.retrieval import RawSnapshotStore
from morn_core.memory.store import MemoryStore


@pytest.fixture
def db_path():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir) / "memory.db"


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir)


@pytest.mark.asyncio
async def test_store_and_retrieve(db_path):
    store = RawSnapshotStore(db_path)
    snapshot = await store.store_snapshot(
        source="chat",
        raw_text="创建者说今天心情很好",
        metadata={"key": "value"},
    )
    assert snapshot.snapshot_id is not None
    assert snapshot.source == "chat"
    assert snapshot.raw_text == "创建者说今天心情很好"
    assert snapshot.metadata == {"key": "value"}

    retrieved = await store.get_snapshot(snapshot.snapshot_id)
    assert retrieved is not None
    assert retrieved.snapshot_id == snapshot.snapshot_id
    assert retrieved.raw_text == "创建者说今天心情很好"
    await store.close()


@pytest.mark.asyncio
async def test_sha256_integrity(db_path):
    store = RawSnapshotStore(db_path)
    snapshot = await store.store_snapshot(
        source="chat",
        raw_text="test integrity",
    )
    assert snapshot.sha256_hash is not None
    assert await store.verify_integrity(snapshot.snapshot_id) is True
    await store.close()


@pytest.mark.asyncio
async def test_immutable(db_path):
    store = RawSnapshotStore(db_path)
    assert not hasattr(store, "update_snapshot")
    assert not hasattr(store, "delete_snapshot")
    await store.close()


@pytest.mark.asyncio
async def test_search_raw(db_path):
    store = RawSnapshotStore(db_path)
    await store.store_snapshot(source="chat", raw_text="今天天气很好")
    await store.store_snapshot(source="chat", raw_text="衰老干预研究进展")
    await store.store_snapshot(source="chat", raw_text="散步有益健康")

    results = await store.search_raw("衰老", limit=10)
    assert len(results) >= 1
    assert "衰老" in results[0].raw_text

    results = await store.search_raw("不存在的内容", limit=10)
    assert len(results) == 0
    await store.close()


@pytest.mark.asyncio
async def test_capsule_auto_create(data_dir):
    async with MemoryStore(data_dir) as store:
        event_id = await store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者提到他正在研究衰老干预",
            "source": "chat",
        })

        snapshot = await store.raw_snapshot_store.get_snapshot_by_capsule(event_id)
        assert snapshot is not None
        assert snapshot.raw_text == "创建者提到他正在研究衰老干预"
        assert snapshot.source == "chat"
    await store.raw_snapshot_store.close()


@pytest.mark.asyncio
async def test_capsule_has_snapshot_id(data_dir):
    async with MemoryStore(data_dir) as store:
        event_id = await store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者说了一个重要的想法",
            "source": "chat",
        })

        capsule = await store.get_capsule(event_id)
        assert capsule is not None
        assert capsule.get("raw_snapshot_id") is not None
    await store.raw_snapshot_store.close()


@pytest.mark.asyncio
async def test_integrity_fail_detected(db_path):
    store = RawSnapshotStore(db_path)
    snapshot = await store.store_snapshot(
        source="chat",
        raw_text="original text content",
    )

    assert await store.verify_integrity(snapshot.snapshot_id) is True

    await store.db.execute(
        "UPDATE raw_snapshots SET raw_text=? WHERE snapshot_id=?",
        ("tampered text", snapshot.snapshot_id),
    )
    await store.db.commit()

    assert await store.verify_integrity(snapshot.snapshot_id) is False
    await store.close()


@pytest.mark.asyncio
async def test_get_snapshot_by_capsule_nonexistent(db_path):
    store = RawSnapshotStore(db_path)
    result = await store.get_snapshot_by_capsule("nonexistent")
    assert result is None
    await store.close()


@pytest.mark.asyncio
async def test_get_snapshot_nonexistent(db_path):
    store = RawSnapshotStore(db_path)
    result = await store.get_snapshot("nonexistent-uuid")
    assert result is None
    await store.close()


@pytest.mark.asyncio
async def test_different_sources_auto_create(data_dir):
    async with MemoryStore(data_dir) as store:
        for source in ("chat", "self_reflection", "milestone"):
            eid = await store.add_capsule({
                "entities": "[]",
                "description": f"test from {source}",
                "source": source,
            })
            snap = await store.raw_snapshot_store.get_snapshot_by_capsule(eid)
            assert snap is not None, f"no snapshot for source={source}"
            assert snap.source == source

        other_eid = await store.add_capsule({
            "entities": "[]",
            "description": "other source",
            "source": "evolution",
        })
        other_snap = await store.raw_snapshot_store.get_snapshot_by_capsule(other_eid)
        assert other_snap is None
    await store.raw_snapshot_store.close()