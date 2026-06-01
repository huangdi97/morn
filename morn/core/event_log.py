"""事件溯源——append-only event log based on aiosqlite"""

import asyncio
import hashlib
import json
import logging
from pathlib import Path
from typing import Optional

from .bus import Event, Priority

logger = logging.getLogger("morn.event_log")


class EventLog:
    def __init__(self, db_path: Path):
        self._db_path = db_path
        self._db_path.parent.mkdir(parents=True, exist_ok=True)
        self._conn: Optional = None
        self._lock = asyncio.Lock()
        self._last_hash: Optional[str] = None

    async def open(self):
        import aiosqlite
        self._conn = await aiosqlite.connect(str(self._db_path))
        await self._conn.execute("PRAGMA journal_mode=WAL")
        await self._conn.execute("PRAGMA synchronous=NORMAL")
        await self._conn.execute("""
            CREATE TABLE IF NOT EXISTS event_log (
                rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                type TEXT NOT NULL,
                source TEXT NOT NULL,
                priority TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                timestamp REAL NOT NULL,
                prev_event_hash TEXT
            )
        """)
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT prev_event_hash FROM event_log ORDER BY rowid DESC LIMIT 1"
        )
        row = await cursor.fetchone()
        if row and row[0]:
            self._last_hash = row[0]
        await cursor.close()

    async def close(self):
        if self._conn:
            await self._conn.close()
            self._conn = None

    async def append(self, event: Event) -> None:
        if not self._conn:
            return
        prev_hash = self._last_hash or ""
        raw = f"{prev_hash}{event.event_id}{event.type}{event.source}{event.priority.name}{event.timestamp}"
        # TODO(v1.0): 替换为 BLAKE3（3-5x 快于 SHA256）。当前 v0.4 仅存 hash 备用，不做 Merkle 验证。
        current_hash = hashlib.sha256(raw.encode()).hexdigest()
        async with self._lock:
            await self._conn.execute(
                "INSERT INTO event_log (event_id, type, source, priority, payload_json, timestamp, prev_event_hash) "
                "VALUES (?, ?, ?, ?, ?, ?, ?)",
                (
                    event.event_id,
                    event.type,
                    event.source,
                    event.priority.name,
                    json.dumps(event.payload),
                    event.timestamp,
                    prev_hash if prev_hash else None,
                ),
            )
            await self._conn.commit()
            self._last_hash = current_hash

    async def replay_since(self, after_rowid: int) -> list[Event]:
        if not self._conn:
            return []
        cursor = await self._conn.execute(
            "SELECT event_id, type, source, priority, payload_json, timestamp "
            "FROM event_log WHERE rowid > ? ORDER BY rowid ASC",
            (after_rowid,),
        )
        rows = await cursor.fetchall()
        await cursor.close()
        return [self._row_to_event(r) for r in rows]

    async def get_last(self, count: int) -> list[Event]:
        if not self._conn:
            return []
        cursor = await self._conn.execute(
            "SELECT event_id, type, source, priority, payload_json, timestamp "
            "FROM event_log ORDER BY rowid DESC LIMIT ?",
            (count,),
        )
        rows = await cursor.fetchall()
        await cursor.close()
        events = [self._row_to_event(r) for r in rows]
        events.reverse()
        return events

    def _row_to_event(self, row) -> Event:
        event_id, etype, source, priority_str, payload_json, timestamp = row
        priority = next(p for p in Priority if p.name == priority_str)
        return Event(
            type=etype,
            payload=json.loads(payload_json),
            source=source,
            priority=priority,
            timestamp=timestamp,
            event_id=event_id,
        )
