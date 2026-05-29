import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.security_advanced.rule_learner import SafetyEventCapsule, SafetyMemoryStore
from morn.contrib.security_advanced.rule_learner import RuleLearner
from morn_core.security.rules import get_all_rules, reset_rules
from morn_core.security.rules import SecurityValidator


@pytest.fixture
def db_path():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir) / "memory.db"


@pytest.fixture(autouse=True)
def cleanup_rules():
    reset_rules()
    yield


@pytest.mark.asyncio
async def test_learn_from_event_with_suggestion(db_path):
    store = SafetyMemoryStore(str(db_path))
    learner = RuleLearner(store)
    event = SafetyEventCapsule(
        event_id="e1", event_type="block", action_text="dd if=/dev/zero of=/dev/sda",
        trigger_rule="DANGER_001",
        suggested_rule=r"dd\s+if=.*of=",
    )
    rule = await learner.learn_from_event(event)
    assert rule is not None
    assert rule.pattern == r"dd\s+if=.*of="
    assert rule.category == "learned"
    await store.close()


@pytest.mark.asyncio
async def test_learn_from_event_no_suggestion(db_path):
    store = SafetyMemoryStore(str(db_path))
    learner = RuleLearner(store)
    event = SafetyEventCapsule(
        event_id="e2", event_type="block", action_text="some action",
        trigger_rule="DANGER_001",
    )
    rule = await learner.learn_from_event(event)
    assert rule is None
    await store.close()


@pytest.mark.asyncio
async def test_learn_duplicate_skipped(db_path):
    store = SafetyMemoryStore(str(db_path))
    learner = RuleLearner(store)
    event = SafetyEventCapsule(
        event_id="e3", event_type="block", action_text="rm -rf",
        trigger_rule="DANGER_001",
        suggested_rule=r"rm\s+.*-rf",
    )
    rule = await learner.learn_from_event(event)
    assert rule is None
    await store.close()


@pytest.mark.asyncio
async def test_sync_to_validator(db_path):
    store = SafetyMemoryStore(str(db_path))
    learner = RuleLearner(store)
    await store.store_event(SafetyEventCapsule(
        event_id="e1", event_type="block", action_text="dd if=/dev/zero",
        verification_status="active", suggested_rule=r"dd\s+if=",
    ))
    count = await learner.sync_to_validator(SecurityValidator())
    assert count == 1
    all_rules = get_all_rules()
    patterns = [r.pattern for r in all_rules]
    assert r"dd\s+if=" in patterns
    await store.close()


@pytest.mark.asyncio
async def test_promote_sandbox_rules(db_path):
    from datetime import datetime, timezone, timedelta
    store = SafetyMemoryStore(str(db_path))
    learner = RuleLearner(store)
    old_time = (datetime.now(timezone.utc) - timedelta(days=10)).isoformat()
    await store.store_event(SafetyEventCapsule(
        event_id="old1", event_type="block", action_text="old action",
        verification_status="pending", created_at=old_time))
    await store.store_event(SafetyEventCapsule(
        event_id="new1", event_type="block", action_text="new action",
        verification_status="pending",
        created_at=datetime.now(timezone.utc).isoformat()))
    await learner.promote_sandbox_rules(days=7)
    events = await store.get_events()
    status_map = {e["event_id"]: e["verification_status"] for e in events}
    assert status_map.get("old1") == "active"
    assert status_map.get("new1") == "pending"
    await store.close()


@pytest.mark.asyncio
async def test_get_sandbox_rules(db_path):
    from datetime import datetime, timezone, timedelta
    store = SafetyMemoryStore(str(db_path))
    learner = RuleLearner(store)
    old_time = (datetime.now(timezone.utc) - timedelta(days=10)).isoformat()
    await store.store_event(SafetyEventCapsule(
        event_id="old1", event_type="block", action_text="old action",
        verification_status="pending", created_at=old_time))
    await store.store_event(SafetyEventCapsule(
        event_id="new1", event_type="block", action_text="new action",
        verification_status="pending",
        created_at=datetime.now(timezone.utc).isoformat()))
    sandbox = await learner.get_sandbox_rules(days=7)
    ids = [e["event_id"] for e in sandbox]
    assert "new1" in ids
    assert "old1" not in ids
    await store.close()