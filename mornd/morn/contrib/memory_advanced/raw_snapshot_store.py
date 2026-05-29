import hashlib
import json
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional

import aiosqlite


@dataclass
class RawSnapshot:
    snapshot_id: str
    source: str
    raw_text: str
    sha256_hash: str
    timestamp: str
    metadata: dict = field(default_factory=dict)


class RawSnapshotStore:
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
            CREATE TABLE IF NOT EXISTS raw_snapshots (
                snapshot_id     TEXT PRIMARY KEY,
                source          TEXT NOT NULL,
                raw_text        TEXT NOT NULL,
                sha256_hash     TEXT NOT NULL,
                timestamp       TEXT NOT NULL DEFAULT (datetime('now')),
                metadata        TEXT DEFAULT '{}'
            )
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_raw_snapshots_source ON raw_snapshots(source)
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_raw_snapshots_timestamp ON raw_snapshots(timestamp)
        """)
        await self.db.commit()

    @staticmethod
    def _compute_sha256(text: str) -> str:
        return hashlib.sha256(text.encode("utf-8")).hexdigest()

    async def store_snapshot(
        self, source: str, raw_text: str, metadata: Optional[dict] = None
    ) -> RawSnapshot:
        await self._ensure_db()
        snapshot_id = str(uuid.uuid4())
        sha256_hash = self._compute_sha256(raw_text)
        timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.") + \
            f"{datetime.now(timezone.utc).microsecond:06d}Z"
        metadata_json = json.dumps(metadata or {})
        await self.db.execute(
            "INSERT INTO raw_snapshots (snapshot_id, source, raw_text, sha256_hash, timestamp, metadata) VALUES (?, ?, ?, ?, ?, ?)",
            (snapshot_id, source, raw_text, sha256_hash, timestamp, metadata_json))
        await self.db.commit()
        return RawSnapshot(snapshot_id=snapshot_id, source=source, raw_text=raw_text,
                           sha256_hash=sha256_hash, timestamp=timestamp, metadata=metadata or {})

    async def get_snapshot(self, snapshot_id: str) -> Optional[RawSnapshot]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT * FROM raw_snapshots WHERE snapshot_id = ?", (snapshot_id,))
        row = await cursor.fetchone()
        return self._row_to_snapshot(row) if row else None

    async def get_snapshot_by_capsule(self, capsule_id: str) -> Optional[RawSnapshot]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT s.* FROM raw_snapshots s WHERE json_extract(s.metadata, '$.capsule_id') = ?",
            (capsule_id,))
        row = await cursor.fetchone()
        return self._row_to_snapshot(row) if row else None

    async def verify_integrity(self, snapshot_id: str) -> bool:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT raw_text, sha256_hash FROM raw_snapshots WHERE snapshot_id = ?",
            (snapshot_id,))
        row = await cursor.fetchone()
        if row is None:
            return False
        return row["sha256_hash"] == self._compute_sha256(row["raw_text"])

    async def search_raw(self, query: str, limit: int = 10) -> list[RawSnapshot]:
        await self._ensure_db()
        cursor = await self.db.execute(
            "SELECT * FROM raw_snapshots WHERE raw_text LIKE ? ORDER BY timestamp DESC LIMIT ?",
            (f"%{query}%", limit))
        rows = await cursor.fetchall()
        return [self._row_to_snapshot(row) for row in rows]

    @staticmethod
    def _row_to_snapshot(row: aiosqlite.Row) -> RawSnapshot:
        metadata = {}
        raw_metadata = row["metadata"]
        if raw_metadata:
            try:
                metadata = json.loads(raw_metadata)
            except (json.JSONDecodeError, TypeError):
                pass
        return RawSnapshot(snapshot_id=row["snapshot_id"], source=row["source"],
                           raw_text=row["raw_text"], sha256_hash=row["sha256_hash"],
                           timestamp=row["timestamp"], metadata=metadata)

    async def close(self):
        if self.db:
            await self.db.close()
            self.db = None