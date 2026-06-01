import json
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional

import aiosqlite


@dataclass
class AuditEntry:
    entry_id: str
    timestamp: str
    event_type: str
    action: str
    verdict: str
    rule_id: str
    context: dict = field(default_factory=dict)
    creator_response: Optional[str] = None


class SecurityAuditLog:
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
            CREATE TABLE IF NOT EXISTS security_audit (
                entry_id         TEXT PRIMARY KEY,
                timestamp        TEXT DEFAULT (datetime('now')),
                event_type       TEXT NOT NULL,
                action           TEXT NOT NULL,
                verdict          TEXT NOT NULL,
                rule_id          TEXT DEFAULT '',
                context          TEXT DEFAULT '{}',
                creator_response TEXT
            )
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_audit_type ON security_audit(event_type)
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_audit_time ON security_audit(timestamp)
        """)
        await self.db.commit()

    async def log_event(self, event_type: str, action: str, verdict: str,
                        rule_id: str, context: Optional[dict] = None) -> str:
        await self._ensure_db()
        entry_id = str(uuid.uuid4())
        timestamp = datetime.now(timezone.utc).isoformat()
        context_json = json.dumps(context or {})
        await self.db.execute("""
            INSERT INTO security_audit
                (entry_id, timestamp, event_type, action, verdict, rule_id, context)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, (entry_id, timestamp, event_type, action, verdict, rule_id, context_json))
        await self.db.commit()
        return entry_id

    async def get_events(self, event_type: Optional[str] = None,
                         limit: int = 100, offset: int = 0) -> list[AuditEntry]:
        await self._ensure_db()
        if event_type:
            cursor = await self.db.execute(
                "SELECT * FROM security_audit WHERE event_type=? ORDER BY timestamp DESC LIMIT ? OFFSET ?",
                (event_type, limit, offset))
        else:
            cursor = await self.db.execute(
                "SELECT * FROM security_audit ORDER BY timestamp DESC LIMIT ? OFFSET ?",
                (limit, offset))
        rows = await cursor.fetchall()
        return [self._row_to_entry(row) for row in rows]

    async def get_events_by_timerange(self, start: str, end: str) -> list[AuditEntry]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT * FROM security_audit WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp DESC",
            (start, end))
        rows = await cursor.fetchall()
        return [self._row_to_entry(row) for row in rows]

    async def get_stats(self) -> dict:
        await self._ensure_db()
        cursor = await self.db.execute("SELECT COUNT(*) as cnt FROM security_audit")
        total = (await cursor.fetchone())["cnt"]
        cursor = await self.db.execute(
            "SELECT event_type, COUNT(*) as cnt FROM security_audit GROUP BY event_type")
        by_type = {row["event_type"]: row["cnt"] for row in await cursor.fetchall()}
        cursor = await self.db.execute(
            "SELECT rule_id, COUNT(*) as cnt FROM security_audit WHERE rule_id != '' GROUP BY rule_id ORDER BY cnt DESC")
        by_rule = {row["rule_id"]: row["cnt"] for row in await cursor.fetchall()}
        return {"total": total, "by_type": by_type, "by_rule": by_rule}

    async def get_replay_chain(self, event_id: str) -> list[AuditEntry]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT * FROM security_audit WHERE entry_id=?", (event_id,))
        target = await cursor.fetchone()
        if target is None:
            return []
        target_time = target["timestamp"]
        cursor = await self.db.execute(
            "SELECT * FROM security_audit WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp ASC",
            (target_time, target_time))
        all_at_time = await cursor.fetchall()
        cursor = await self.db.execute(
            "SELECT * FROM security_audit WHERE entry_id=?",
            (event_id,))
        return [self._row_to_entry(r) for r in all_at_time]

    async def close(self):
        if self.db:
            await self.db.close()
            self.db = None

    @staticmethod
    def _row_to_entry(row: aiosqlite.Row) -> AuditEntry:
        context = {}
        raw = row["context"]
        if raw:
            try:
                context = json.loads(raw)
            except (json.JSONDecodeError, TypeError):
                pass
        return AuditEntry(
            entry_id=row["entry_id"],
            timestamp=row["timestamp"],
            event_type=row["event_type"],
            action=row["action"],
            verdict=row["verdict"],
            rule_id=row["rule_id"],
            context=context,
            creator_response=row["creator_response"],
        )


import csv
import io
from datetime import datetime, timezone, timedelta


class AuditReplay:
    def __init__(self, audit_log: SecurityAuditLog):
        self._audit_log = audit_log

    async def export_csv(self, start_time: Optional[str] = None,
                         end_time: Optional[str] = None,
                         output_path: Optional[str] = None) -> str:
        if start_time and end_time:
            events = await self._audit_log.get_events_by_timerange(start_time, end_time)
        else:
            events = await self._audit_log.get_events()

        fieldnames = ["entry_id", "timestamp", "event_type", "action", "verdict", "rule_id"]
        if output_path:
            with open(output_path, "w", newline="") as f:
                writer = csv.DictWriter(f, fieldnames=fieldnames)
                writer.writeheader()
                for ev in events:
                    writer.writerow({k: getattr(ev, k, "") for k in fieldnames})
            return output_path
        else:
            buf = io.StringIO()
            writer = csv.DictWriter(buf, fieldnames=fieldnames)
            writer.writeheader()
            for ev in events:
                writer.writerow({k: getattr(ev, k, "") for k in fieldnames})
            return buf.getvalue()

    async def export_timeline(self, output_path: Optional[str] = None) -> str:
        events = await self._audit_log.get_events(limit=10000)
        groups: dict[str, list] = {}
        for ev in events:
            try:
                dt = datetime.fromisoformat(ev.timestamp)
                day = dt.strftime("%Y-%m-%d")
            except (ValueError, TypeError):
                day = "unknown"
            groups.setdefault(day, []).append(ev)

        lines = ["# Security Audit Timeline\n"]
        for day in sorted(groups.keys(), reverse=True):
            lines.append(f"## {day}\n")
            for ev in groups[day]:
                ts = ev.timestamp
                if len(ts) > 19:
                    ts = ts[:19]
                lines.append(
                    f"- **{ts}** | {ev.event_type} | {ev.action[:60]} | verdict={ev.verdict} | rule={ev.rule_id}\n"
                )
            lines.append("\n")

        content = "".join(lines)
        if output_path:
            with open(output_path, "w") as f:
                f.write(content)
            return output_path
        return content

    async def generate_summary(self, days: int = 7) -> str:
        cutoff = (datetime.now(timezone.utc) - timedelta(days=days)).isoformat()
        now = datetime.now(timezone.utc).isoformat()
        events = await self._audit_log.get_events_by_timerange(cutoff, now)
        if not events:
            return f"## Security Summary (past {days} days)\n\nNo security events in this period."

        total = len(events)
        blocks = sum(1 for e in events if e.event_type == "block")
        warns = sum(1 for e in events if e.event_type == "warn")
        overrides = sum(1 for e in events if e.event_type == "creator_override")
        rule_changes = sum(1 for e in events if e.event_type == "rule_change")
        creator_interventions = overrides + rule_changes

        rule_counts: dict[str, int] = {}
        for e in events:
            if e.rule_id:
                rule_counts[e.rule_id] = rule_counts.get(e.rule_id, 0) + 1
        top_rules = sorted(rule_counts.items(), key=lambda x: -x[1])[:3]

        if total > 0:
            mid = total // 2
            first_half = sum(1 for e in events[:mid] if e.event_type in ("block", "warn"))
            second_half = sum(1 for e in events[mid:] if e.event_type in ("block", "warn"))
            if second_half > first_half:
                trend = "上升"
            elif second_half < first_half:
                trend = "下降"
            else:
                trend = "稳定"
        else:
            trend = "稳定"

        lines = [
            f"## Security Summary (past {days} days)\n",
            "\n",
            f"- **Total intercepted**: {total}\n",
            f"  - Blocked: {blocks}\n",
            f"  - Warned: {warns}\n",
            f"- **Creator interventions**: {creator_interventions}\n",
            f"  - Overrides: {overrides}\n",
            f"  - Rule changes: {rule_changes}\n",
        ]
        if top_rules:
            lines.append("- **Top 3 rules triggered**:\n")
            for i, (rule_id, count) in enumerate(top_rules, 1):
                lines.append(f"  {i}. {rule_id} ({count}次)\n")
        lines.append(f"- **Trend**: {trend}\n")
        return "".join(lines)
