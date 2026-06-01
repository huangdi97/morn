import logging
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone, timedelta
from typing import Optional

import aiosqlite

from morn.core.rules import SecurityRule, add_rule, get_all_rules


@dataclass
class SafetyEventCapsule:
    event_id: str
    event_type: str
    action_text: str
    trigger_rule: str = ""
    creator_intervention: bool = False
    suggested_rule: Optional[str] = None
    verification_status: str = "pending"
    created_at: str = ""


class SafetyMemoryStore:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.db: Optional[aiosqlite.Connection] = None

    async def _ensure_db(self):
        if self.db is None:
            self.db = await aiosqlite.connect(str(self.db_path))
            self.db.row_factory = aiosqlite.Row
            await self.db.execute("PRAGMA journal_mode=WAL")
            await self._create_table()

    async def _create_table(self):
        await self.db.execute("""
            CREATE TABLE IF NOT EXISTS safety_events (
                event_id             TEXT PRIMARY KEY,
                event_type           TEXT NOT NULL,
                action_text          TEXT NOT NULL,
                trigger_rule         TEXT DEFAULT '',
                creator_intervention INTEGER DEFAULT 0,
                suggested_rule       TEXT,
                verification_status  TEXT DEFAULT 'pending',
                created_at           TEXT DEFAULT (datetime('now'))
            )
        """)
        await self.db.commit()

    async def store_event(self, capsule: SafetyEventCapsule) -> str:
        await self._ensure_db()
        event_id = capsule.event_id or str(uuid.uuid4())
        created_at = capsule.created_at or datetime.now(timezone.utc).isoformat()
        await self.db.execute("""
            INSERT INTO safety_events
                (event_id, event_type, action_text, trigger_rule,
                 creator_intervention, suggested_rule, verification_status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            event_id, capsule.event_type, capsule.action_text, capsule.trigger_rule,
            1 if capsule.creator_intervention else 0,
            capsule.suggested_rule, capsule.verification_status, created_at,
        ))
        await self.db.commit()
        return event_id

    async def get_events(self, event_type: Optional[str] = None, limit: int = 50) -> list[dict]:
        await self._ensure_db()
        if event_type:
            cursor = await self.db.execute(
                "SELECT * FROM safety_events WHERE event_type=? ORDER BY created_at DESC LIMIT ?",
                (event_type, limit))
        else:
            cursor = await self.db.execute(
                "SELECT * FROM safety_events ORDER BY created_at DESC LIMIT ?", (limit,))
        rows = await cursor.fetchall()
        return [dict(row) for row in rows]

    async def get_pending_rules(self) -> list[dict]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT * FROM safety_events WHERE verification_status='pending' ORDER BY created_at DESC")
        rows = await cursor.fetchall()
        return [dict(row) for row in rows]

    async def get_active_rules(self) -> list[dict]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT * FROM safety_events WHERE verification_status='active' ORDER BY created_at DESC")
        rows = await cursor.fetchall()
        return [dict(row) for row in rows]

    async def approve_rule(self, event_id: str) -> bool:
        await self._ensure_db()
        cursor = await self.db.execute(
            "UPDATE safety_events SET verification_status='active' WHERE event_id=?",
            (event_id,))
        await self.db.commit()
        return cursor.rowcount > 0

    async def reject_rule(self, event_id: str) -> bool:
        await self._ensure_db()
        cursor = await self.db.execute(
            "UPDATE safety_events SET verification_status='rejected' WHERE event_id=?",
            (event_id,))
        await self.db.commit()
        return cursor.rowcount > 0

    async def count_by_status(self) -> dict[str, int]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT verification_status, COUNT(*) as cnt FROM safety_events GROUP BY verification_status")
        rows = await cursor.fetchall()
        result = {"pending": 0, "sandbox": 0, "active": 0, "rejected": 0}
        for row in rows:
            result[row["verification_status"]] = row["cnt"]
        return result

    async def close(self):
        if self.db:
            await self.db.close()
            self.db = None


class RuleLearner:
    def __init__(self, safety_store: SafetyMemoryStore):
        self.safety_store = safety_store

    async def learn_from_event(self, event: SafetyEventCapsule) -> Optional[SecurityRule]:
        if not event.suggested_rule:
            return None

        existing_rules = get_all_rules()
        for rule in existing_rules:
            if rule.pattern == event.suggested_rule:
                return None

        rule_id = f"AUTO_{len(existing_rules) + 1:03d}"
        new_rule = SecurityRule(
            rule_id=rule_id,
            name=f"Learned from {event.event_type}",
            pattern=event.suggested_rule,
            category="learned",
            severity="medium",
            action_on_match="block",
            description=f"Auto-learned from event {event.event_id}: {event.action_text[:60]}",
        )
        return new_rule

    async def sync_to_validator(self, validator) -> int:
        active_events = await self.safety_store.get_active_rules()
        count = 0
        for evt in active_events:
            if evt.get("suggested_rule"):
                existing = get_all_rules()
                already_exists = any(
                    r.pattern == evt["suggested_rule"] for r in existing
                )
                if not already_exists:
                    rule_id = f"SYNC_{len(existing) + 1:03d}"
                    new_rule = SecurityRule(
                        rule_id=rule_id,
                        name=f"Synced from safety event",
                        pattern=evt["suggested_rule"],
                        category="learned",
                        severity="medium",
                        action_on_match="block",
                        description=f"Synced from event {evt['event_id']}: {evt['action_text'][:60]}",
                    )
                    add_rule(new_rule)
                    count += 1
        return count

    async def get_sandbox_rules(self, days: int = 7) -> list[dict]:
        events = await self.safety_store.get_events()
        cutoff = datetime.now(timezone.utc) - timedelta(days=days)
        result = []
        for evt in events:
            if evt["verification_status"] != "pending":
                continue
            try:
                created = datetime.fromisoformat(evt["created_at"])
            except (ValueError, TypeError):
                continue
            if created >= cutoff:
                result.append(evt)
        return result

    async def promote_sandbox_rules(self, days: int = 7):
        events = await self.safety_store.get_events()
        cutoff = datetime.now(timezone.utc) - timedelta(days=days)
        for evt in events:
            if evt["verification_status"] != "pending":
                continue
            try:
                created = datetime.fromisoformat(evt["created_at"])
            except (ValueError, TypeError):
                continue
            if created < cutoff:
                await self.safety_store.approve_rule(evt["event_id"])
