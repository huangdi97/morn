import json
import logging
import re
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import aiosqlite

logger = logging.getLogger("morn.evolution")


@dataclass
class SkillScoreCard:
    skill_id: str
    success_rate: float = 0.0
    reuse_count: int = 0
    stability: float = 0.0
    efficiency: float = 0.0
    context_diversity: int = 0
    total_score: float = 0.0

    @classmethod
    def calculate(cls, skill_data: dict) -> "SkillScoreCard":
        sid = skill_data.get("skill_id", "")
        success_rate = skill_data.get("success_rate", 0.0)
        reuse_count = skill_data.get("reuse_count", 0)
        recent_rates = skill_data.get("recent_success_rates", [])
        avg_exec_time = skill_data.get("avg_exec_time", 1.0)
        baseline_time = skill_data.get("baseline_time", 1.0)
        context_diversity = skill_data.get("context_diversity", 0)

        if recent_rates:
            mean = sum(recent_rates) / len(recent_rates)
            variance = sum((r - mean) ** 2 for r in recent_rates) / len(recent_rates)
            std = variance ** 0.5
            stability = max(0.0, 1.0 - std)
        else:
            stability = 1.0

        if baseline_time > 0 and avg_exec_time > 0:
            efficiency = min(1.0, baseline_time / avg_exec_time)
        else:
            efficiency = 1.0

        normalized_reuse = min(1.0, reuse_count / 50.0)

        total = (
            success_rate * 0.30
            + normalized_reuse * 0.25
            + stability * 0.25
            + efficiency * 0.20
        )

        return cls(
            skill_id=sid,
            success_rate=success_rate,
            reuse_count=reuse_count,
            stability=round(stability, 4),
            efficiency=round(efficiency, 4),
            context_diversity=context_diversity,
            total_score=round(total, 4),
        )

    def get_verdict(self) -> str:
        if self.total_score >= 0.7:
            return "retain"
        elif self.total_score >= 0.3:
            return "observe"
        else:
            return "retire"


class SkillVoteManager:
    def __init__(self):
        self._competitions: dict[str, dict] = {}
        self._archived: list[dict] = []

    def register_competition(self, skill_ids: list[str]) -> str:
        cid = str(uuid.uuid4())
        self._competitions[cid] = {
            "skill_ids": list(skill_ids),
            "votes": {sid: 0 for sid in skill_ids},
            "total_rounds": 0,
        }
        return cid

    def cast_vote(self, competition_id: str, skill_id: str, success: bool) -> None:
        comp = self._competitions.get(competition_id)
        if comp is None:
            raise ValueError(f"Competition {competition_id} not found")
        if skill_id not in comp["votes"]:
            raise ValueError(f"Skill {skill_id} not in competition")
        if success:
            comp["votes"][skill_id] += 1
        comp["total_rounds"] += 1

    def get_winner(self, competition_id: str) -> str:
        comp = self._competitions.get(competition_id)
        if comp is None:
            raise ValueError(f"Competition {competition_id} not found")
        if not comp["votes"]:
            raise ValueError("No votes cast yet")
        winner = max(comp["votes"], key=comp["votes"].get)
        return winner

    def archive_loser(self, competition_id: str) -> None:
        comp = self._competitions.get(competition_id)
        if comp is None:
            raise ValueError(f"Competition {competition_id} not found")
        winner_id = self.get_winner(competition_id)
        for sid in comp["skill_ids"]:
            if sid != winner_id:
                self._archived.append({
                    "competition_id": competition_id,
                    "skill_id": sid,
                    "votes": comp["votes"].get(sid, 0),
                    "winner_id": winner_id,
                })
        comp["_archived"] = True

    def get_archived(self) -> list[dict]:
        return list(self._archived)


class SkillVersionStore:
    def __init__(self, db_path=None):
        self._versions: dict[str, list[dict]] = {}
        self._latest: dict[str, str] = {}
        self._active: dict[str, str] = {}
        self._db_path = db_path

    def save_version(self, skill_id: str, version_data: dict) -> str:
        vid = str(uuid.uuid4())
        entry = {
            "version_id": vid,
            "skill_id": skill_id,
            "version_data": json.dumps(version_data, ensure_ascii=False),
            "created_at": time.time(),
            "is_active": 0,
            "success_rate": version_data.get("success_rate", 0.0),
        }
        if skill_id not in self._versions:
            self._versions[skill_id] = []
        self._versions[skill_id].append(entry)
        self._latest[skill_id] = vid
        return vid

    def get_version(self, skill_id: str, version_id: str) -> Optional[dict]:
        versions = self._versions.get(skill_id, [])
        for v in versions:
            if v["version_id"] == version_id:
                result = dict(v)
                result["version_data"] = json.loads(result["version_data"])
                return result
        return None

    def get_latest_version(self, skill_id: str) -> Optional[dict]:
        vid = self._latest.get(skill_id)
        if vid is None:
            return None
        return self.get_version(skill_id, vid)

    def rollback(self, skill_id: str, version_id: str) -> bool:
        target = self.get_version(skill_id, version_id)
        if target is None:
            return False
        self._latest[skill_id] = version_id
        self._active[skill_id] = version_id
        return True

    def get_versions(self, skill_id: str) -> list[dict]:
        raw = self._versions.get(skill_id, [])
        results = []
        for v in raw:
            entry = dict(v)
            entry["version_data"] = json.loads(entry["version_data"])
            results.append(entry)
        return results

    def activate_version(self, skill_id: str, version_id: str) -> bool:
        versions = self._versions.get(skill_id, [])
        for v in versions:
            if v["version_id"] == version_id:
                v["is_active"] = 1
                self._active[skill_id] = version_id
                self._latest[skill_id] = version_id
                return True
        return False

    def get_active_version(self, skill_id: str) -> Optional[dict]:
        vid = self._active.get(skill_id)
        if vid is None:
            return None
        return self.get_version(skill_id, vid)

    def auto_rollback(self, skill_id: str, current_success_rate: float) -> Optional[str]:
        active = self.get_active_version(skill_id)
        if active is None:
            return None
        if current_success_rate >= active["success_rate"] - 0.05:
            return None
        versions = self.get_versions(skill_id)
        if len(versions) < 2:
            return None
        prev = versions[-2]
        prev_vid = prev["version_id"]
        self.rollback(skill_id, prev_vid)
        logger.info("Auto-rolled back %s to version %s (success_rate %.2f < %.2f)",
                     skill_id, prev_vid, current_success_rate, active["success_rate"])
        return prev_vid


class SkillStore:
    def __init__(self, db_path: Path):
        self.db_path = db_path
        self.db: Optional[aiosqlite.Connection] = None

    async def __aenter__(self) -> "SkillStore":
        self.db = await aiosqlite.connect(str(self.db_path))
        self.db.row_factory = aiosqlite.Row
        await self.db.execute("PRAGMA journal_mode=WAL")
        await self._create_schema()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.db:
            await self.db.close()
            self.db = None

    async def _create_schema(self):
        await self.db.execute("""
            CREATE TABLE IF NOT EXISTS skills (
                id               INTEGER PRIMARY KEY AUTOINCREMENT,
                name             TEXT UNIQUE NOT NULL,
                trigger_keywords TEXT NOT NULL DEFAULT '[]',
                template         TEXT NOT NULL DEFAULT '',
                source           TEXT NOT NULL DEFAULT 'internal',
                usage_count      INTEGER NOT NULL DEFAULT 0,
                created_at       TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
            )
        """)
        await self.db.commit()

    async def add_skill(self, name: str, trigger_keywords: list[str],
                         template: str = "", source: str = "internal") -> tuple[int, bool]:
        cursor = await self.db.execute(
            "SELECT id FROM skills WHERE name = ?", (name,))
        row = await cursor.fetchone()
        if row:
            return row["id"], False
        kw_json = json.dumps(trigger_keywords, ensure_ascii=False)
        cursor = await self.db.execute("""
            INSERT INTO skills (name, trigger_keywords, template, source)
            VALUES (?, ?, ?, ?)
        """, (name, kw_json, template, source))
        await self.db.commit()
        return cursor.lastrowid, True

    async def get_skill(self, skill_id: int) -> Optional[dict]:
        cursor = await self.db.execute(
            "SELECT * FROM skills WHERE id = ?", (skill_id,))
        row = await cursor.fetchone()
        if row is None:
            return None
        d = dict(row)
        d["trigger_keywords"] = json.loads(d["trigger_keywords"])
        return d

    async def search_skills(self, query: str) -> list[dict]:
        words = [w for w in query.split() if w]
        if not words:
            return []
        conditions = []
        params = []
        for word in words:
            like = f"%{word}%"
            conditions.append("(name LIKE ? OR trigger_keywords LIKE ?)")
            params.extend([like, like])
        sql = f"SELECT * FROM skills WHERE {' OR '.join(conditions)} ORDER BY usage_count DESC LIMIT 20"
        cursor = await self.db.execute(sql, params)
        rows = await cursor.fetchall()
        results = []
        for row in rows:
            d = dict(row)
            d["trigger_keywords"] = json.loads(d["trigger_keywords"])
            results.append(d)
        return results

    async def list_skills(self, source: Optional[str] = None) -> list[dict]:
        if source:
            cursor = await self.db.execute(
                "SELECT * FROM skills WHERE source = ? ORDER BY created_at DESC",
                (source,))
        else:
            cursor = await self.db.execute(
                "SELECT * FROM skills ORDER BY created_at DESC")
        rows = await cursor.fetchall()
        results = []
        for row in rows:
            d = dict(row)
            d["trigger_keywords"] = json.loads(d["trigger_keywords"])
            results.append(d)
        return results

    async def delete_skill(self, skill_id: int) -> bool:
        cursor = await self.db.execute(
            "DELETE FROM skills WHERE id = ?", (skill_id,))
        await self.db.commit()
        return cursor.rowcount > 0

    async def increment_usage(self, skill_id: int) -> bool:
        cursor = await self.db.execute("""
            UPDATE skills
            SET usage_count = usage_count + 1, updated_at = datetime('now')
            WHERE id = ?
        """, (skill_id,))
        await self.db.commit()
        return cursor.rowcount > 0

    @staticmethod
    def _parse_skill_md(content: str) -> Optional[dict]:
        m = re.match(r'^---\s*\n(.*?)\n---[\s\n]*(.*)', content, re.DOTALL)
        if not m:
            m = re.match(r'^---\s*\n(.*?)\n---\s*$', content, re.DOTALL)
            if not m:
                return None
            frontmatter_text = m.group(1)
            body = ""
        else:
            frontmatter_text = m.group(1)
            body = m.group(2).strip()
        frontmatter = {}
        current_key = None
        for line in frontmatter_text.strip().split('\n'):
            stripped = line.strip()
            if not stripped:
                continue
            if ':' in stripped and not stripped.startswith('- '):
                key, _, value = stripped.partition(':')
                current_key = key.strip()
                value = value.strip()
                if value:
                    if value.startswith('[') and value.endswith(']'):
                        try:
                            value = json.loads(value)
                        except json.JSONDecodeError:
                            value = [v.strip().strip('"').strip("'") for v in value[1:-1].split(',') if v.strip()]
                    frontmatter[current_key] = value
                else:
                    frontmatter[current_key] = []
            elif stripped.startswith('- ') and current_key:
                item = stripped[2:].strip().strip('"').strip("'")
                if not isinstance(frontmatter.get(current_key), list):
                    frontmatter[current_key] = []
                frontmatter[current_key].append(item)
        name = frontmatter.get("name", "")
        if isinstance(name, str):
            name = name.strip().strip('"').strip("'")
        if not name:
            return None
        trigger_keywords = frontmatter.get("trigger_keywords", [])
        if isinstance(trigger_keywords, str):
            trigger_keywords = [trigger_keywords]
        template = frontmatter.get("template", body)
        if isinstance(template, str):
            template = template.strip()
        return {
            "name": name,
            "trigger_keywords": trigger_keywords,
            "template": template,
        }

    async def load_external_skills(self, skills_dir: Path) -> int:
        if not skills_dir.exists():
            return 0
        count = 0
        for md_file in sorted(skills_dir.glob("*.md")):
            content = md_file.read_text(encoding="utf-8")
            parsed = self._parse_skill_md(content)
            if parsed is None:
                logger.warning("SKILL.md parse failed: %s", md_file)
                continue
            try:
                _, is_new = await self.add_skill(
                    name=parsed["name"],
                    trigger_keywords=parsed["trigger_keywords"],
                    template=parsed["template"],
                    source="external",
                )
                if is_new:
                    count += 1
            except Exception as e:
                logger.warning("Failed to load skill %s: %s", md_file, e)
        return count

    async def close(self):
        if self.db:
            await self.db.close()
            self.db = None


class SkillCurator:
    def __init__(self, config=None, data_dir=None, evolution_logger=None):
        config = config or {}
        self.enabled = config.get("enabled", True)
        self._evolution_logger = evolution_logger
        self._tracking = {}
        self._deprecated = set()
        self._storage_path = None
        if data_dir:
            self._storage_path = Path(data_dir) / "evolution" / "curator_tracking.json"
            self._load()

    def record_usage(self, skill_id, success=None):
        sid = str(skill_id)
        if sid not in self._tracking:
            self._tracking[sid] = {
                "use_count": 0,
                "success_count": 0,
                "fail_count": 0,
                "last_used": 0.0,
            }
        t = self._tracking[sid]
        t["use_count"] += 1
        t["last_used"] = time.time()
        if success is True:
            t["success_count"] += 1
        elif success is False:
            t["fail_count"] += 1
        self._save()

    def get_stats(self, skill_id):
        sid = str(skill_id)
        t = self._tracking.get(sid, {})
        use_count = t.get("use_count", 0)
        success_count = t.get("success_count", 0)
        fail_count = t.get("fail_count", 0)
        total = success_count + fail_count
        success_rate = success_count / total if total > 0 else None
        return {
            "skill_id": sid,
            "use_count": use_count,
            "success_count": success_count,
            "fail_count": fail_count,
            "success_rate": success_rate,
            "last_used": t.get("last_used", 0),
            "deprecated": sid in self._deprecated,
        }

    def mark_deprecated(self, skill_id):
        sid = str(skill_id)
        self._deprecated.add(sid)
        if self._evolution_logger:
            self._evolution_logger.log("skill_curator", "mark_deprecated", {"skill_id": sid})
        self._save()

    def propose_archive(self):
        archived = []
        for sid in list(self._deprecated):
            stats = self.get_stats(sid)
            archived.append({
                "skill_id": sid,
                "reason": "deprecated",
                "stats": stats,
            })
        return archived

    def get_review_report(self):
        return {
            "tracked_skills": len(self._tracking),
            "deprecated_count": len(self._deprecated),
            "deprecated": sorted(self._deprecated),
            "tracking": {k: dict(v) for k, v in self._tracking.items()},
        }

    def review_skills(self, skills_list):
        if not self.enabled:
            return []
        results = []
        name_groups = {}
        for s in skills_list:
            sid = str(s.get("id", s.get("name", "")))
            name = s.get("name", sid)
            usage_count = s.get("usage_count", 0)
            success_rate = self._get_success_rate(sid)
            days_since_use = self._calc_days_since_use(sid, s.get("updated_at"))
            issues = []
            action = "keep"
            typical_tags = set()

            if days_since_use is not None and days_since_use >= 90:
                if success_rate is not None and success_rate < 0.8:
                    issues.append("low_usage_and_low_success_rate")
                    action = "deprecate"
                else:
                    issues.append("low_usage")
                    if action == "keep":
                        action = "review"

            if success_rate is not None and success_rate < 0.6:
                issues.append("low_success_rate")
                action = "deprecate"

            if sid in self._deprecated:
                if action != "deprecate":
                    issues.append("already_deprecated")
                    action = "deprecated"

            if s.get("trigger_keywords"):
                typical_tags.update(kw.lower().strip() for kw in s["trigger_keywords"] if isinstance(kw, str))
            base = name.lower().replace("_", " ").replace("-", " ").strip()
            if base:
                typical_tags.add(base.split()[0] if base.split() else base)

            for tag in typical_tags:
                if tag:
                    name_groups.setdefault(tag, []).append({"sid": sid, "name": name, "result_index": len(results)})

            results.append({
                "skill_id": sid,
                "name": name,
                "usage_count": usage_count,
                "success_rate": success_rate,
                "days_since_use": days_since_use,
                "issues": issues,
                "action": action,
            })

        for tag, group in name_groups.items():
            if len(group) >= 3:
                for entry in group:
                    idx = entry["result_index"]
                    r = results[idx]
                    if "merge_suggestion" not in r["issues"]:
                        r["issues"].append("merge_suggestion")
                        if r["action"] in ("keep", "review"):
                            r["action"] = "merge"

        if self._evolution_logger:
            for r in results:
                if r["action"] != "keep":
                    self._evolution_logger.log("skill_curator", f"review:{r['action']}", {
                        "skill_id": r["skill_id"],
                        "name": r["name"],
                        "issues": r["issues"],
                    })

        return results

    def _get_success_rate(self, skill_id):
        sid = str(skill_id)
        t = self._tracking.get(sid)
        if not t:
            return None
        total = t.get("success_count", 0) + t.get("fail_count", 0)
        if total == 0:
            return None
        return t["success_count"] / total

    def _calc_days_since_use(self, skill_id, fallback_updated_at=None):
        sid = str(skill_id)
        t = self._tracking.get(sid)
        ts = None
        if t and t.get("last_used", 0) > 0:
            ts = t["last_used"]
        if ts is None and fallback_updated_at:
            try:
                ts = self._parse_datetime(fallback_updated_at)
            except (ValueError, TypeError, AttributeError):
                pass
        if ts is None:
            return None
        return (time.time() - ts) / 86400

    @staticmethod
    def _parse_datetime(dt_str):
        from datetime import datetime
        for fmt in ("%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d"):
            try:
                return datetime.strptime(dt_str, fmt).timestamp()
            except ValueError:
                continue
        return time.time()

    def _save(self):
        if not self._storage_path:
            return
        self._storage_path.parent.mkdir(parents=True, exist_ok=True)
        data = {
            "tracking": self._tracking,
            "deprecated": sorted(self._deprecated),
        }
        with open(self._storage_path, "w") as f:
            json.dump(data, f, indent=2)

    def _load(self):
        if self._storage_path and self._storage_path.exists():
            with open(self._storage_path) as f:
                data = json.load(f)
                self._tracking = data.get("tracking", {})
                self._deprecated = set(data.get("deprecated", []))


class SkillLifecycleManager(SkillCurator):
    def __init__(self, config=None, data_dir=None, evolution_logger=None):
        super().__init__(config, data_dir, evolution_logger)
        self._scores: dict[str, SkillScoreCard] = {}
        self._status: dict[str, str] = {}
        self._consecutive_low: dict[str, int] = {}
        self._vote_mgr = SkillVoteManager()

    def create(self, skill_data: dict) -> Optional[str]:
        sid = skill_data.get("skill_id", "")
        if not sid:
            return None
        context_diversity = skill_data.get("context_diversity", 0)
        if context_diversity < 3:
            return None
        self._tracking[sid] = self._tracking.get(sid, {
            "use_count": 0,
            "success_count": 0,
            "fail_count": 0,
            "last_used": 0.0,
        })
        self._status[sid] = "observe"
        self._consecutive_low[sid] = 0
        return sid

    def evaluate(self, skill_id: str) -> SkillScoreCard:
        stats = self.get_stats(skill_id)
        skill_data = {
            "skill_id": str(skill_id),
            "success_rate": stats.get("success_rate", 0.0) or 0.0,
            "reuse_count": stats.get("use_count", 0),
            "recent_success_rates": [],
            "avg_exec_time": 1.0,
            "baseline_time": 1.0,
            "context_diversity": 1,
        }
        card = SkillScoreCard.calculate(skill_data)
        self._scores[str(skill_id)] = card
        verdict = card.get_verdict()
        if verdict == "retain":
            if self._status.get(str(skill_id)) not in ("active",):
                self._status[str(skill_id)] = "observe"
        elif verdict == "observe":
            if self._status.get(str(skill_id)) != "retired":
                self._status[str(skill_id)] = "observe"
        elif verdict == "retire":
            if self._status.get(str(skill_id)) == "observe":
                self._status[str(skill_id)] = "retired"
        return card

    def promote(self, skill_id: str) -> bool:
        sid = str(skill_id)
        if self._status.get(sid) != "observe":
            return False
        card = self._scores.get(sid)
        if card and card.success_rate >= 0.7:
            self._status[sid] = "active"
            if self._evolution_logger:
                self._evolution_logger.log("skill_lifecycle", "promote", {"skill_id": sid})
            return True
        return False

    def demote(self, skill_id: str) -> bool:
        sid = str(skill_id)
        if self._status.get(sid) != "active":
            return False
        card = self._scores.get(sid)
        if card and card.total_score < 0.5:
            self._consecutive_low[sid] = self._consecutive_low.get(sid, 0) + 1
            if self._consecutive_low[sid] >= 3:
                self._status[sid] = "observe"
                if self._evolution_logger:
                    self._evolution_logger.log("skill_lifecycle", "demote", {"skill_id": sid})
                return True
        else:
            self._consecutive_low[sid] = 0
        return False

    def retire(self, skill_id: str) -> bool:
        sid = str(skill_id)
        if self._status.get(sid) != "observe":
            return False
        card = self._scores.get(sid)
        if card and card.get_verdict() == "retire":
            self._status[sid] = "retired"
            self._deprecated.add(sid)
            if self._evolution_logger:
                self._evolution_logger.log("skill_lifecycle", "retire", {"skill_id": sid})
            self._save()
            return True
        return False

    def get_status(self, skill_id: str) -> str:
        return self._status.get(str(skill_id), "unknown")

    def get_vote_manager(self) -> SkillVoteManager:
        return self._vote_mgr