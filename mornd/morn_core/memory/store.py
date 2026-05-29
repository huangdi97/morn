import asyncio
import json
import logging
import random
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

import aiosqlite

from morn_core.memory.crypto import MemoryCrypto
from morn.contrib.memory_advanced.raw_snapshot_store import RawSnapshotStore
from morn_core.memory.vector_store import VectorStore

try:
    from morn_core.eventbus.bus import Event, Priority
    _HAS_EVENTBUS = True
except ImportError:
    _HAS_EVENTBUS = False


_TRUST_MAP = {0: 'ltz', 1: 'mtz', 2: 'htz'}
_TRUST_MAP_REV = {'ltz': 0, 'mtz': 1, 'htz': 2}

DEFAULT_TRUST_LEVEL = 2


class MemoryStore:
    def __init__(self, data_dir: Path, enable_encryption: bool = False,
                 default_trust_level: int = DEFAULT_TRUST_LEVEL,
                 event_bus: Optional[object] = None):
        self.data_dir = Path(data_dir)
        self.db_path = self.data_dir / "memory.db"
        self.db: Optional[aiosqlite.Connection] = None
        self._crypto = MemoryCrypto(data_dir) if enable_encryption else None
        self.vector_store = VectorStore(data_dir)
        self.raw_snapshot_store: Optional[RawSnapshotStore] = None
        self.safety_memory: Optional[RawSnapshotStore] = None
        self.default_trust_level = default_trust_level
        self._event_bus = event_bus

    async def _fetch(self, sql: str, params: tuple = (), decrypt: bool = True) -> list[dict]:
        cursor = await self.db.execute(sql, params)
        rows = await cursor.fetchall()
        if decrypt:
            return [self._decrypt_capsule(dict(row)) for row in rows]
        return [dict(row) for row in rows]

    async def _execute(self, sql: str, params: tuple = ()) -> aiosqlite.Cursor:
        cursor = await self.db.execute(sql, params)
        await self.db.commit()
        return cursor

    async def __aenter__(self) -> "MemoryStore":
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self.db = await aiosqlite.connect(str(self.db_path))
        self.db.row_factory = aiosqlite.Row
        await self.db.execute("PRAGMA journal_mode=WAL")
        await self.db.execute("PRAGMA foreign_keys=ON")
        await self._create_schema()
        self.raw_snapshot_store = RawSnapshotStore(self.db_path)
        from morn.contrib.security_advanced.rule_learner import SafetyMemoryStore
        self.safety_memory = SafetyMemoryStore(self.db_path)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.db:
            await self.db.close()
            self.db = None

    async def _create_schema(self):
        await self.db.execute("""
            CREATE TABLE IF NOT EXISTS capsules (
                event_id        TEXT PRIMARY KEY,
                timestamp       TEXT NOT NULL,
                entities        TEXT NOT NULL DEFAULT '[]',
                emotion_score   REAL NOT NULL DEFAULT 0.0,
                emotion_tag     TEXT NOT NULL DEFAULT '',
                description     TEXT NOT NULL,
                importance_weight REAL NOT NULL DEFAULT 0.5,
                causal_edges    TEXT NOT NULL DEFAULT '[]',
                source          TEXT NOT NULL DEFAULT 'chat',
                forgotten       INTEGER NOT NULL DEFAULT 0,
                forget_creator  INTEGER NOT NULL DEFAULT 0,
                trust_level     TEXT NOT NULL DEFAULT 'htz',
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            )
        """)
        await self.db.execute("""
            CREATE VIRTUAL TABLE IF NOT EXISTS capsules_fts USING fts5(
                event_id UNINDEXED,
                description,
                entities,
                content='capsules',
                content_rowid='rowid'
            )
        """)
        await self.db.execute("""
            CREATE TRIGGER IF NOT EXISTS capsules_ai AFTER INSERT ON capsules
            BEGIN
                INSERT INTO capsules_fts(rowid, event_id, description, entities)
                VALUES (new.rowid, new.event_id, new.description, new.entities);
            END
        """)
        await self.db.execute("""
            CREATE TRIGGER IF NOT EXISTS capsules_ad AFTER DELETE ON capsules
            BEGIN
                INSERT INTO capsules_fts(capsules_fts, rowid, event_id, description, entities)
                VALUES ('delete', old.rowid, old.event_id, old.description, old.entities);
            END
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_capsules_emotion ON capsules(emotion_tag)
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_capsules_timestamp ON capsules(timestamp)
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_capsules_forget ON capsules(forget_creator)
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_capsules_timestamp_emotion ON capsules(timestamp, emotion_score)
        """)
        try:
            await self.db.execute("ALTER TABLE capsules ADD COLUMN hindsight_marks TEXT DEFAULT '[]'")
        except Exception:
            pass
        try:
            await self.db.execute("ALTER TABLE capsules ADD COLUMN raw_snapshot_id TEXT DEFAULT NULL")
        except Exception:
            pass
        await self.db.execute("""
            CREATE TABLE IF NOT EXISTS semantic_knowledge (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                subject     TEXT NOT NULL,
                relation    TEXT NOT NULL,
                object      TEXT NOT NULL,
                confidence  REAL NOT NULL DEFAULT 0.5,
                source      TEXT NOT NULL DEFAULT 'chat',
                source_event_id TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                verified_at TEXT,
                forgotten   INTEGER NOT NULL DEFAULT 0
            )
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_sk_subject_relation
            ON semantic_knowledge(subject, relation)
        """)
        await self.db.execute("""
            CREATE INDEX IF NOT EXISTS idx_sk_object
            ON semantic_knowledge(object)
        """)
        await self.db.execute("""
            CREATE UNIQUE INDEX IF NOT EXISTS idx_sk_unique
            ON semantic_knowledge(subject, relation, object)
        """)
        await self.db.execute("""
            CREATE TABLE IF NOT EXISTS personality_memory (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                category    TEXT NOT NULL,
                content     TEXT NOT NULL,
                importance  REAL NOT NULL DEFAULT 0.5,
                source      TEXT NOT NULL DEFAULT 'self_reflection',
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            )
        """)
        await self.db.execute("""
            CREATE TABLE IF NOT EXISTS archived_personality (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                archive_id  TEXT NOT NULL,
                category    TEXT NOT NULL,
                content     TEXT NOT NULL,
                importance  REAL NOT NULL DEFAULT 0.5,
                source      TEXT NOT NULL DEFAULT 'self_reflection',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                archived_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        """)
        await self.db.commit()

    @staticmethod
    def _make_event_id() -> str:
        now = datetime.now(timezone.utc)
        timestamp = now.strftime("%Y%m%d_%H%M%S")
        suffix = random.randint(100, 999)
        return f"evt_{timestamp}_{suffix}"

    def _decrypt_capsule(self, capsule: dict) -> dict:
        if capsule and self._crypto:
            capsule = dict(capsule)
            capsule["description"] = self._crypto.decrypt(capsule.get("description", ""))
        return capsule

    async def add_capsule(self, capsule: dict) -> str:
        if "description" not in capsule or not capsule["description"].strip():
            raise ValueError("description is required")

        event_id = capsule.get("event_id", self._make_event_id())
        now_ts = datetime.now(timezone.utc)
        timestamp = capsule.get("timestamp",
            now_ts.strftime("%Y-%m-%dT%H:%M:%S.") + f"{now_ts.microsecond:06d}Z")

        valid_levels = {'htz', 'mtz', 'ltz'}
        trust_level = capsule.get('trust_level', _TRUST_MAP.get(self.default_trust_level, 'htz'))
        if isinstance(trust_level, int):
            trust_level = _TRUST_MAP.get(trust_level, 'htz')
        if trust_level not in valid_levels:
            trust_level = 'htz'

        data = {
            "event_id": event_id,
            "timestamp": timestamp,
            "entities": capsule.get("entities", "[]"),
            "emotion_score": capsule.get("emotion_score", 0.0),
            "emotion_tag": capsule.get("emotion_tag", ""),
            "description": capsule["description"],
            "importance_weight": capsule.get("importance_weight", 0.5),
            "causal_edges": capsule.get("causal_edges", "[]"),
            "source": capsule.get("source", "chat"),
            "session_id": capsule.get("session_id", ""),
            "forgotten": capsule.get("forgotten", 0),
            "forget_creator": capsule.get("forget_creator", 0),
            "trust_level": trust_level,
            "raw_snapshot_id": capsule.get("raw_snapshot_id", None),
        }

        if self._crypto and data["description"]:
            data["description"] = self._crypto.encrypt(data["description"])

        for attempt in range(3):
            try:
                await self.db.execute("""
                    INSERT INTO capsules
                    (event_id, timestamp, entities, emotion_score, emotion_tag,
                     description, importance_weight, causal_edges, source,
                     forgotten, forget_creator, trust_level, raw_snapshot_id)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, (
                    data["event_id"], data["timestamp"], data["entities"],
                    data["emotion_score"], data["emotion_tag"],
                    data["description"], data["importance_weight"],
                    data["causal_edges"], data["source"],
                    data["forgotten"], data["forget_creator"],
                    data["trust_level"], data["raw_snapshot_id"],
                ))
                await self.db.commit()
                source = data["source"]
                if source in ("chat", "self_reflection", "milestone") and self.raw_snapshot_store:
                    raw_text = capsule["description"]
                    metadata = {"capsule_id": data["event_id"], "entities": data["entities"]}
                    snapshot = await self.raw_snapshot_store.store_snapshot(
                        source=source, raw_text=raw_text, metadata=metadata
                    )
                    data["raw_snapshot_id"] = snapshot.snapshot_id
                    try:
                        await self.db.execute(
                            "UPDATE capsules SET raw_snapshot_id=? WHERE event_id=?",
                            (snapshot.snapshot_id, data["event_id"]))
                        await self.db.commit()
                    except Exception:
                        pass
                try:
                    text = f"{data['entities']}: {data['description']}"
                    await self.vector_store.add_embedding(data["event_id"], text)
                except Exception as e:
                    logger = logging.getLogger("morn.memory")
                    logger.warning("vector embedding failed: %s", e)

                if self._event_bus and _HAS_EVENTBUS:
                    await self._publish_capsule_written(data["event_id"], data)
                return data["event_id"]
            except Exception as e:
                if attempt < 2:
                    logger = logging.getLogger("morn.memory")
                    logger.warning("add_capsule attempt %d failed: %s", attempt + 1, e)
                    data["event_id"] = self._make_event_id()
                    await asyncio.sleep(0.5)
                else:
                    logger = logging.getLogger("morn.memory")
                    logger.error("add_capsule failed after 3 attempts: %s", e)
                    raise RuntimeError(f"add_capsule failed after 3 retries: {e}")

    async def get_capsule(self, event_id: str) -> Optional[dict]:
        rows = await self._fetch("SELECT * FROM capsules WHERE event_id = ?", (event_id,))
        return rows[0] if rows else None

    @staticmethod
    def _trust_to_int(level: str) -> int:
        return _TRUST_MAP_REV.get(level, 0)

    @staticmethod
    def _int_to_trust(level: int) -> str:
        return _TRUST_MAP.get(level, 'ltz')

    async def set_trust_level(self, event_id: str, level: int) -> bool:
        if level not in _TRUST_MAP:
            raise ValueError(f"trust level must be 0, 1, or 2, got {level}")
        cursor = await self._execute(
            "UPDATE capsules SET trust_level=? WHERE event_id=?",
            (_TRUST_MAP[level], event_id))
        return cursor.rowcount > 0

    async def search_capsules(self, min_trust_level: int = 0, limit: int = 50) -> list[dict]:
        valid_strs = {_TRUST_MAP[v] for v in _TRUST_MAP if v >= min_trust_level}
        if not valid_strs:
            return []
        placeholders = ",".join("?" for _ in valid_strs)
        return await self._fetch(
            f"SELECT * FROM capsules WHERE trust_level IN ({placeholders}) AND forget_creator = 0 ORDER BY timestamp DESC LIMIT ?",
            list(valid_strs) + [limit])

    async def search_fts(self, query: str, limit: int = 10) -> list[dict]:
        results = []
        seen_ids = set()
        try:
            for row in await self._fetch("""
                SELECT c.* FROM capsules c JOIN capsules_fts f ON c.event_id = f.event_id
                WHERE capsules_fts MATCH ? AND c.forget_creator = 0 ORDER BY rank LIMIT ?
            """, (query, limit), decrypt=False):
                d = self._decrypt_capsule(row)
                if d["event_id"] not in seen_ids:
                    results.append(d)
                    seen_ids.add(d["event_id"])
        except aiosqlite.OperationalError:
            pass
        has_chinese = any(ord(c) > 127 for c in query)
        if self._crypto or has_chinese or len(results) == 0:
            try:
                for row in await self._fetch(
                    "SELECT * FROM capsules WHERE forget_creator = 0 ORDER BY timestamp DESC LIMIT ?",
                    (limit * 2,), decrypt=False):
                    d = self._decrypt_capsule(row)
                    desc = d.get("description", "")
                    if query.lower() in desc.lower() and d["event_id"] not in seen_ids:
                        results.append(d)
                        seen_ids.add(d["event_id"])
            except Exception:
                pass
        return results[:limit]

    async def semantic_search(self, query: str, limit: int = 5) -> list[dict]:
        try:
            event_ids = await self.vector_store.search_similar(query, limit)
            if not event_ids:
                return []
            capsules = []
            for eid in event_ids:
                cap = await self.get_capsule(eid)
                if cap and cap.get("forget_creator", 0) == 0:
                    capsules.append(cap)
            return capsules
        except Exception:
            return []

    async def search_by_entity(self, entity: str, limit: int = 10) -> list[dict]:
        return await self._fetch(
            "SELECT * FROM capsules WHERE entities LIKE ? AND forget_creator = 0 ORDER BY timestamp DESC LIMIT ?",
            (f'%"{entity}"%', limit))

    async def search_by_timerange(self, start: str, end: str, limit: int = 50) -> list[dict]:
        return await self._fetch(
            "SELECT * FROM capsules WHERE timestamp >= ? AND timestamp <= ? AND forget_creator = 0 ORDER BY timestamp DESC LIMIT ?",
            (start, end, limit))

    async def get_recent(self, limit: int = 20) -> list[dict]:
        return await self._fetch(
            "SELECT * FROM capsules WHERE forget_creator = 0 ORDER BY timestamp DESC LIMIT ?", (limit,))

    async def count(self) -> int:
        cursor = await self.db.execute("SELECT COUNT(*) FROM capsules WHERE forget_creator = 0")
        row = await cursor.fetchone()
        return row[0]

    async def forget(self, event_id: str) -> bool:
        cursor = await self._execute(
            "UPDATE capsules SET forget_creator=1 WHERE event_id=? AND forget_creator=0", (event_id,))
        return cursor.rowcount > 0

    async def unforget(self, event_id: str) -> bool:
        cursor = await self._execute(
            "UPDATE capsules SET forget_creator=0 WHERE event_id=? AND forget_creator=1", (event_id,))
        return cursor.rowcount > 0

    async def update_emotion(self, event_id: str, score: float, tag: str) -> bool:
        cursor = await self._execute(
            "UPDATE capsules SET emotion_score=?, emotion_tag=? WHERE event_id=?", (score, tag, event_id))
        return cursor.rowcount > 0

    async def add_emotion_tag(self, event_id: str, score: float,
                               tag: str) -> bool:
        cursor = await self.db.execute(
            "SELECT emotion_tag FROM capsules WHERE event_id=?", (event_id,))
        row = await cursor.fetchone()
        if row is None:
            return False
        old_tag = row["emotion_tag"]
        try:
            tags = json.loads(old_tag) if old_tag else []
            if not isinstance(tags, list):
                tags = [old_tag]
            tags.append(tag)
            new_tag = json.dumps(tags)
        except (json.JSONDecodeError, TypeError):
            new_tag = json.dumps([old_tag, tag]) if old_tag else json.dumps([tag])
        await self._execute(
            "UPDATE capsules SET emotion_tag=?, emotion_score=? WHERE event_id=?", (new_tag, score, event_id))
        return True

    async def get_capsules_by_emotion(self, tag: str, limit: int = 50) -> list[dict]:
        return await self._fetch(
            "SELECT * FROM capsules WHERE emotion_tag LIKE ? AND forget_creator = 0 ORDER BY timestamp DESC LIMIT ?",
            (f"%{tag}%", limit))

    async def cleanup_expired(self, retention_days: int = 30):
        await self._execute(
            "UPDATE capsules SET forgotten=1 WHERE timestamp < datetime('now', ?) AND source != 'evolution' AND forgotten = 0 AND forget_creator = 0",
            (f'-{retention_days} days',))

    async def vacuum(self):
        await self.db.execute("VACUUM")

    async def add_knowledge(self, subject: str, relation: str, object: str,
                        confidence: float = 0.5, source: str = "chat",
                        source_event_id: str = None, verified: bool = True) -> int:
        cursor = await self.db.execute(
            "SELECT id, confidence FROM semantic_knowledge WHERE subject = ? AND relation = ? AND object = ? AND forgotten = 0",
            (subject, relation, object))
        row = await cursor.fetchone()
        if row:
            new_conf = min(1.0, row["confidence"] + 0.1)
            await self._execute("UPDATE semantic_knowledge SET confidence = ?, verified_at = datetime('now') WHERE id = ?",
                                (new_conf, row["id"]))
            return row["id"]
        verified_at = datetime.now(timezone.utc).isoformat() if verified else None
        cursor = await self.db.execute(
            "INSERT INTO semantic_knowledge (subject, relation, object, confidence, source, source_event_id, verified_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (subject, relation, object, confidence, source, source_event_id, verified_at))
        await self.db.commit()
        return cursor.lastrowid

    async def query_knowledge(self, subject: str = None, relation: str = None,
                              object: str = None, min_confidence: float = 0.0) -> list[dict]:
        conditions, params = ["forgotten = 0", "confidence >= ?"], [min_confidence]
        for name, val in [("subject", subject), ("relation", relation), ("object", object)]:
            if val is not None:
                conditions.append(f"{name} = ?")
                params.append(val)
        return await self._fetch(
            f"SELECT * FROM semantic_knowledge WHERE {' AND '.join(conditions)}", params, decrypt=False)

    async def get_user_preferences(self) -> list[dict]:
        return await self.query_knowledge(subject="创建者", min_confidence=0.3)

    async def verify_knowledge(self, id: int, new_confidence: float):
        await self._execute(
            "UPDATE semantic_knowledge SET confidence = ?, verified_at = datetime('now') WHERE id = ?",
            (max(0.0, min(1.0, new_confidence)), id))

    async def forget_knowledge(self, id: int) -> bool:
        cursor = await self._execute("UPDATE semantic_knowledge SET forgotten=1 WHERE id=? AND forgotten=0", (id,))
        return cursor.rowcount > 0

    async def unforget_knowledge(self, id: int) -> bool:
        cursor = await self._execute("UPDATE semantic_knowledge SET forgotten=0 WHERE id=? AND forgotten=1", (id,))
        return cursor.rowcount > 0

    async def add_personality(self, category: str, content: str, importance: float = 0.5) -> int:
        cursor = await self.db.execute(
            "INSERT INTO personality_memory (category, content, importance) VALUES (?, ?, ?)",
            (category, content, importance))
        await self.db.commit()
        return cursor.lastrowid

    async def query_personality(self, category: str = None, limit: int = 20) -> list[dict]:
        if category:
            return await self._fetch(
                "SELECT * FROM personality_memory WHERE category = ? ORDER BY created_at DESC LIMIT ?",
                (category, limit), decrypt=False)
        return await self._fetch(
            "SELECT * FROM personality_memory ORDER BY created_at DESC LIMIT ?",
            (limit,), decrypt=False)

    async def get_identity(self) -> list[dict]:
        return await self.query_personality(category="identity")

    async def get_beliefs(self) -> list[dict]:
        return await self.query_personality(category="belief")

    async def add_hindsight_tag(self, event_id: str, tag: str, score: float) -> bool:
        return await self.add_emotion_tag(event_id, score, tag)

    async def add_hindsight_mark(self, memory_id: str, new_tag: str,
                                  new_emotion_score: float,
                                  trigger_context: str) -> bool:
        cursor = await self.db.execute(
            "SELECT hindsight_marks FROM capsules WHERE event_id=?", (memory_id,))
        row = await cursor.fetchone()
        if row is None:
            return False
        marks = json.loads(row["hindsight_marks"]) if row["hindsight_marks"] else []
        marks.append({
            "tag": new_tag,
            "emotion_score": new_emotion_score,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "trigger_context": trigger_context,
        })
        await self._execute("UPDATE capsules SET hindsight_marks=? WHERE event_id=?",
                            (json.dumps(marks), memory_id))
        return True

    async def get_hindsight_marks(self, memory_id: str) -> list[dict]:
        cursor = await self.db.execute(
            "SELECT hindsight_marks FROM capsules WHERE event_id=?", (memory_id,))
        row = await cursor.fetchone()
        if row is None:
            return []
        return json.loads(row["hindsight_marks"]) if row["hindsight_marks"] else []

    async def get_eligible_for_hindsight(
            self, threshold_days: int = 30, min_emotion: float = 0.5) -> list[dict]:
        rows = await self._fetch(
            "SELECT event_id, description, timestamp, emotion_tag, emotion_score FROM capsules "
            "WHERE timestamp <= datetime('now', ?) AND emotion_score >= ? AND forget_creator = 0 "
            "AND (hindsight_marks IS NULL OR hindsight_marks = '[]') ORDER BY timestamp ASC LIMIT 50",
            (f'-{threshold_days} days', min_emotion), decrypt=False)
        return [dict(
            id=r["event_id"], description=r["description"], timestamp=r["timestamp"],
            emotion_tag=r["emotion_tag"], emotion_score=r["emotion_score"],
        ) for r in rows]

    async def _publish_capsule_written(self, capsule_id: str, capsule_data: dict):
        if not self._event_bus:
            return
        logger = logging.getLogger("morn.memory")
        logger.debug("capsule_written published for %s", capsule_id)
        await self._event_bus.publish(Event(
            type="memory.capsule_written",
            payload={
                "capsule_id": capsule_id,
                "session_id": capsule_data.get("session_id", ""),
                "trust_level": capsule_data.get("trust_level", "mtz"),
                "source": capsule_data.get("source", "unknown"),
            },
            source="memory_core",
            priority=Priority.MEDIUM,
            timestamp=capsule_data.get("timestamp", time.time()),
        ))

    async def archive_personality(self, archive_id: str = None) -> int:
        """归档当前 L4 人格记忆到 archived_personality 表。

        将当前 personality_memory 全部复制到 archived_personality
        （标记为给定 archive_id），然后清空 personality_memory。
        返回归档的记录条数。

        旧 L4 归档后可作为"过去视角"只读引用，不参与当前
        L4 状态机更新。情感隔离（标注为 archived）。
        """
        if archive_id is None:
            from datetime import datetime as _dt
            archive_id = f"archive_{_dt.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}"

        rows = await self._fetch(
            "SELECT category, content, importance, source, created_at "
            "FROM personality_memory ORDER BY id", decrypt=False)
        if not rows:
            return 0

        import json
        values = []
        for r in rows:
            values.append(
                f"({json.dumps(archive_id)}, "
                f"{json.dumps(r['category'])}, "
                f"{json.dumps(r['content'])}, "
                f"{r['importance']}, "
                f"{json.dumps(r['source'])}, "
                f"{json.dumps(r['created_at'])})"
            )
        batch = ", ".join(values)
        await self.db.execute(
            f"INSERT INTO archived_personality "
            f"(archive_id, category, content, importance, source, created_at) "
            f"VALUES {batch}"
        )
        await self.db.execute("DELETE FROM personality_memory")
        await self.db.commit()
        return len(rows)

    async def get_archived_personalities(self, archive_id: str = None, limit: int = 20) -> list[dict]:
        """查询归档的人格记忆。不设 archive_id 则返回所有归档。"""
        if archive_id:
            return await self._fetch(
                "SELECT * FROM archived_personality "
                "WHERE archive_id = ? ORDER BY archived_at DESC, id ASC LIMIT ?",
                (archive_id, limit), decrypt=False)
        else:
            return await self._fetch(
                "SELECT * FROM archived_personality ORDER BY archived_at DESC, id ASC LIMIT ?",
                (limit,), decrypt=False)

    async def get_archive_ids(self) -> list[str]:
        """返回所有唯一的 archive_id（含记录数）。"""
        cursor = await self.db.execute(
            "SELECT archive_id, COUNT(*) as count, MAX(archived_at) as archived_at "
            "FROM archived_personality GROUP BY archive_id ORDER BY archived_at DESC"
        )
        rows = await cursor.fetchall()
        return [dict(r) for r in rows]

    async def close(self):
        if self.db:
            await self.db.close()
            self.db = None