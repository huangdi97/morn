import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.security_advanced.audit import SecurityAuditLog, AuditReplay


@pytest.fixture
def db_path():
    with tempfile.TemporaryDirectory(prefix="morn_test_") as tmpdir:
        yield Path(tmpdir) / "memory.db"


@pytest.mark.asyncio
async def test_log_event(db_path):
    log = SecurityAuditLog(str(db_path))
    eid = await log.log_event(
        event_type="block",
        action="rm -rf /",
        verdict="block",
        rule_id="DANGER_001",
        context={"cmd": "rm -rf /", "user": "test"},
    )
    assert eid is not None
    events = await log.get_events()
    assert len(events) == 1
    assert events[0].event_type == "block"
    assert events[0].action == "rm -rf /"
    assert events[0].verdict == "block"
    assert events[0].rule_id == "DANGER_001"
    assert events[0].context["cmd"] == "rm -rf /"
    await log.close()


@pytest.mark.asyncio
async def test_get_events_by_type(db_path):
    log = SecurityAuditLog(str(db_path))
    await log.log_event("block", "rm -rf /", "block", "DANGER_001")
    await log.log_event("warn", "suspicious access", "warn", "WARN_001")
    await log.log_event("block", "api key leak", "block", "PRIVACY_001")

    blocks = await log.get_events(event_type="block")
    assert len(blocks) == 2
    assert all(e.event_type == "block" for e in blocks)

    warns = await log.get_events(event_type="warn")
    assert len(warns) == 1
    await log.close()


@pytest.mark.asyncio
async def test_get_events_timerange(db_path):
    log = SecurityAuditLog(str(db_path))
    eid1 = await log.log_event("block", "action1", "block", "R1")
    eid2 = await log.log_event("warn", "action2", "warn", "R2")
    events = await log.get_events_by_timerange("2000-01-01", "2099-12-31")
    assert len(events) == 2
    events = await log.get_events_by_timerange("2000-01-01", "2000-01-02")
    assert len(events) == 0
    await log.close()


@pytest.mark.asyncio
async def test_get_stats(db_path):
    log = SecurityAuditLog(str(db_path))
    await log.log_event("block", "rm", "block", "DANGER_001")
    await log.log_event("block", "leak", "block", "PRIVACY_001")
    await log.log_event("warn", "suspicious", "warn", "WARN_001")
    await log.log_event("creator_override", "override", "allow", "OVERRIDE_001")

    stats = await log.get_stats()
    assert stats["total"] == 4
    assert stats["by_type"]["block"] == 2
    assert stats["by_type"]["warn"] == 1
    assert stats["by_type"]["creator_override"] == 1
    assert stats["by_rule"]["DANGER_001"] == 1
    await log.close()


@pytest.mark.asyncio
async def test_replay_chain(db_path):
    log = SecurityAuditLog(str(db_path))
    eid = await log.log_event("block", "rm -rf /", "block", "DANGER_001")
    chain = await log.get_replay_chain(eid)
    assert len(chain) >= 1
    assert chain[0].entry_id == eid
    await log.close()


@pytest.mark.asyncio
async def test_export_csv(db_path):
    log = SecurityAuditLog(str(db_path))
    await log.log_event("block", "rm -rf /", "block", "DANGER_001")
    replay = AuditReplay(log)
    csv_content = await replay.export_csv()
    assert "entry_id" in csv_content
    assert "block" in csv_content
    assert "rm -rf /" in csv_content
    assert "DANGER_001" in csv_content
    await log.close()


@pytest.mark.asyncio
async def test_export_timeline(db_path):
    log = SecurityAuditLog(str(db_path))
    await log.log_event("block", "rm -rf /", "block", "DANGER_001")
    replay = AuditReplay(log)
    timeline = await replay.export_timeline()
    assert "# Security Audit Timeline" in timeline
    assert "block" in timeline
    await log.close()


@pytest.mark.asyncio
async def test_generate_summary(db_path):
    log = SecurityAuditLog(str(db_path))
    await log.log_event("block", "rm -rf /", "block", "DANGER_001")
    await log.log_event("block", "leak", "block", "PRIVACY_001")
    await log.log_event("warn", "suspicious", "warn", "WARN_001")
    await log.log_event("creator_override", "override", "allow", "OVERRIDE_001")
    replay = AuditReplay(log)
    summary = await replay.generate_summary(days=7)
    assert "Total intercepted" in summary
    assert "Blocked: 2" in summary
    assert "Warned: 1" in summary
    assert "Creator interventions" in summary
    assert "Top 3 rules triggered" in summary
    assert "Trend" in summary
    await log.close()