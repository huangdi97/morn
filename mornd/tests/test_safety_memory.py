import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.security_advanced.rule_learner import SafetyEventCapsule, SafetyMemoryStore


@pytest.fixture
def db_path():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir) / "memory.db"


@pytest.mark.asyncio
async def test_store_and_retrieve_event(db_path):
    store = SafetyMemoryStore(str(db_path))
    capsule = SafetyEventCapsule(
        event_id="evt_test_001",
        event_type="block",
        action_text="rm -rf /",
        trigger_rule="DANGER_001",
        creator_intervention=False,
        suggested_rule="rm\s+.*-rf.*/",
        verification_status="pending",
    )
    eid = await store.store_event(capsule)
    assert eid == "evt_test_001"
    events = await store.get_events()
    assert len(events) == 1
    assert events[0]["action_text"] == "rm -rf /"
    assert events[0]["event_type"] == "block"
    await store.close()


@pytest.mark.asyncio
async def test_get_by_event_type(db_path):
    store = SafetyMemoryStore(str(db_path))
    await store.store_event(SafetyEventCapsule(
        event_id="e1", event_type="block", action_text="rm -rf /",
        trigger_rule="DANGER_001"))
    await store.store_event(SafetyEventCapsule(
        event_id="e2", event_type="review", action_text="suspicious access",
        trigger_rule="REVIEW_001"))
    await store.store_event(SafetyEventCapsule(
        event_id="e3", event_type="block", action_text="api key leak",
        trigger_rule="PRIVACY_001"))
    blocks = await store.get_events(event_type="block")
    assert len(blocks) == 2
    assert all(e["event_type"] == "block" for e in blocks)
    reviews = await store.get_events(event_type="review")
    assert len(reviews) == 1
    await store.close()


@pytest.mark.asyncio
async def test_pending_rules(db_path):
    store = SafetyMemoryStore(str(db_path))
    await store.store_event(SafetyEventCapsule(
        event_id="e1", event_type="block", action_text="test",
        verification_status="pending"))
    await store.store_event(SafetyEventCapsule(
        event_id="e2", event_type="block", action_text="test",
        verification_status="active"))
    pending = await store.get_pending_rules()
    assert len(pending) == 1
    assert pending[0]["event_id"] == "e1"
    await store.close()


@pytest.mark.asyncio
async def test_approve_rule(db_path):
    store = SafetyMemoryStore(str(db_path))
    await store.store_event(SafetyEventCapsule(
        event_id="e1", event_type="block", action_text="test",
        verification_status="pending"))
    ok = await store.approve_rule("e1")
    assert ok
    events = await store.get_active_rules()
    assert len(events) == 1
    assert events[0]["event_id"] == "e1"
    await store.close()


@pytest.mark.asyncio
async def test_reject_rule(db_path):
    store = SafetyMemoryStore(str(db_path))
    await store.store_event(SafetyEventCapsule(
        event_id="e1", event_type="block", action_text="test",
        verification_status="pending"))
    ok = await store.reject_rule("e1")
    assert ok
    events = await store.get_events()
    assert events[0]["verification_status"] == "rejected"
    await store.close()


@pytest.mark.asyncio
async def test_count_by_status(db_path):
    store = SafetyMemoryStore(str(db_path))
    await store.store_event(SafetyEventCapsule(
        event_id="e1", event_type="block", action_text="a",
        verification_status="pending"))
    await store.store_event(SafetyEventCapsule(
        event_id="e2", event_type="block", action_text="b",
        verification_status="active"))
    await store.store_event(SafetyEventCapsule(
        event_id="e3", event_type="review", action_text="c",
        verification_status="rejected"))
    counts = await store.count_by_status()
    assert counts["pending"] == 1
    assert counts["active"] == 1
    assert counts["rejected"] == 1
    assert counts["sandbox"] == 0
    await store.close()