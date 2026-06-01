import json
import logging
import os
import time
from collections import defaultdict
from datetime import datetime, timezone, timedelta
from typing import Optional


class SelfPruner:
    def __init__(self, memory_store, skill_store=None, emotion_history_ref=None,
                 instance_name: str = "default",
                 max_capsules: int = 10000, max_skills: int = 50,
                 max_emotion_history: int = 1000,
                 retention_days: int = 30, enabled: bool = True):
        self.memory_store = memory_store
        self.skill_store = skill_store
        self.emotion_history_ref = emotion_history_ref
        self.instance_name = instance_name
        self.max_capsules = max_capsules
        self.max_skills = max_skills
        self.max_emotion_history = max_emotion_history
        self.retention_days = retention_days
        self.enabled = enabled
        self._logger = logging.getLogger("morn.consciousness")
        self._prune_log: list[dict] = []
        self._cleanup_proposals: dict[str, dict] = {}

    async def diagnose(self) -> dict:
        if not self.enabled:
            return {"enabled": False, "action": "skipped"}

        capsule_count = await self.memory_store.count() if self.memory_store else 0
        skill_count = 0
        if self.skill_store:
            skills = await self.skill_store.list_skills()
            skill_count = len(skills)
        emotion_count = len(self.emotion_history_ref) if self.emotion_history_ref else 0

        result = {
            "capsule_count": capsule_count,
            "skill_count": skill_count,
            "emotion_history_count": emotion_count,
            "capsule_excess": max(0, capsule_count - self.max_capsules),
            "skill_excess": max(0, skill_count - self.max_skills),
            "emotion_excess": max(0, emotion_count - self.max_emotion_history),
        }

        if result["capsule_excess"] > 0:
            deleted = await self.prune_memory()
            result["capsules_pruned"] = deleted
        if result["skill_excess"] > 0:
            deleted = await self.prune_skills()
            result["skills_pruned"] = deleted
        if result["emotion_excess"] > 0:
            deleted = await self.prune_emotion_history()
            result["emotion_pruned"] = deleted

        self._prune_log.append({
            "timestamp": datetime.now(timezone.utc).isoformat(),
            **result,
        })
        if len(self._prune_log) > 100:
            self._prune_log.pop(0)

        if result.get("capsules_pruned", 0) or result.get("skills_pruned", 0) or result.get("emotion_pruned", 0):
            self._logger.info(
                "prune complete: %d capsules, %d skills, %d emotion entries",
                result.get("capsules_pruned", 0),
                result.get("skills_pruned", 0),
                result.get("emotion_pruned", 0),
            )

        return result

    async def prune_memory(self, limit: Optional[int] = None) -> int:
        if not self.memory_store or not self.enabled:
            return 0
        target = self.max_capsules if limit is None else limit
        capsule_count = await self.memory_store.count()
        if capsule_count <= target:
            return 0

        excess = capsule_count - target
        cutoff = (datetime.now(timezone.utc) - timedelta(days=self.retention_days)).isoformat()

        cursor = await self.memory_store.db.execute("""
            SELECT event_id, importance_weight, source, timestamp FROM capsules
            WHERE forget_creator = 0
              AND timestamp < ?
              AND source NOT IN ('self_reflection', 'identity')
            ORDER BY importance_weight ASC, timestamp ASC
            LIMIT ?
        """, (cutoff, excess))
        rows = await cursor.fetchall()

        deleted = 0
        for row in rows:
            if row["source"] in ("self_reflection", "identity"):
                continue
            if row["importance_weight"] >= 0.8:
                continue
            success = await self.memory_store.forget(row["event_id"])
            if success:
                deleted += 1

        if deleted == 0 and excess > 0:
            cursor = await self.memory_store.db.execute("""
                SELECT event_id, importance_weight, source FROM capsules
                WHERE forget_creator = 0
                  AND source NOT IN ('self_reflection', 'identity')
                ORDER BY importance_weight ASC, timestamp ASC
                LIMIT ?
            """, (excess,))
            rows = await cursor.fetchall()
            for row in rows:
                if row["source"] in ("self_reflection", "identity"):
                    continue
                if row["importance_weight"] >= 0.8:
                    continue
                success = await self.memory_store.forget(row["event_id"])
                if success:
                    deleted += 1

        return deleted

    async def prune_skills(self, limit: Optional[int] = None) -> int:
        if not self.skill_store or not self.enabled:
            return 0
        target = self.max_skills if limit is None else limit
        skills = await self.skill_store.list_skills()
        if len(skills) <= target:
            return 0

        excess = len(skills) - target
        skills_sorted = sorted(skills, key=lambda s: (s.get("usage_count", 0), s.get("created_at", "")))
        to_delete = skills_sorted[:excess]

        deleted = 0
        for skill in to_delete:
            success = await self.skill_store.delete_skill(skill["id"])
            if success:
                deleted += 1
        return deleted

    async def prune_emotion_history(self, limit: Optional[int] = None) -> int:
        if self.emotion_history_ref is None or not self.enabled:
            return 0
        target = self.max_emotion_history if limit is None else limit
        current_len = len(self.emotion_history_ref)
        if current_len <= target:
            return 0
        excess = current_len - target
        del self.emotion_history_ref[:excess]
        return excess

    def get_prune_log(self) -> list[dict]:
        return list(self._prune_log)

    async def diagnose_memory_redundancy(self, memory_store=None, max_similar=3) -> list[dict]:
        store = memory_store or self.memory_store
        if not store or not self.enabled:
            return []
        cursor = await store.db.execute("""
            SELECT event_id, entities, description, timestamp, importance_weight, source
            FROM capsules
            WHERE source NOT IN ('self_reflection', 'identity')
              AND forget_creator = 0
            ORDER BY timestamp ASC
        """)
        rows = await cursor.fetchall()
        topic_groups = defaultdict(list)
        for row in rows:
            try:
                entities = json.loads(row["entities"]) if isinstance(row["entities"], str) else (row["entities"] or [])
            except (json.JSONDecodeError, TypeError):
                entities = []
            topic_key = tuple(sorted(entities)) if entities else "untagged"
            topic_groups[topic_key].append(dict(row))
        redundancies = []
        for topic_key, caps in topic_groups.items():
            if len(caps) >= max_similar:
                redundancies.append({
                    "topic": list(topic_key) if isinstance(topic_key, tuple) else topic_key,
                    "count": len(caps),
                    "capsule_ids": [c["event_id"] for c in caps],
                    "suggested_action": "merge_or_prune",
                })
        await self._write_meta_event("diagnose_memory_redundancy", {
            "redundant_topics": len(redundancies),
            "total_scanned": len(rows),
        })
        return redundancies

    async def diagnose_skill_redundancy(self, skill_manager=None, max_idle_days=90, min_success_rate=0.8) -> list[dict]:
        store = skill_manager or self.skill_store
        if not store or not self.enabled:
            return []
        skills = await store.list_skills()
        now = datetime.now(timezone.utc)
        cutoff = (now - timedelta(days=max_idle_days)).isoformat()
        redundancies = []
        for skill in skills:
            last_used = skill.get("last_used_at") or skill.get("created_at", "")
            success_rate = skill.get("success_rate", 1.0)
            if success_rate is None:
                success_rate = 1.0
            is_idle = bool(last_used) and last_used < cutoff
            is_low_success = success_rate < min_success_rate
            if is_idle or is_low_success:
                days_idle = 0
                if last_used:
                    try:
                        days_idle = (now - datetime.fromisoformat(last_used.replace("Z", "+00:00"))).days
                    except (ValueError, TypeError):
                        days_idle = max_idle_days
                redundancies.append({
                    "skill_id": skill["id"],
                    "skill_name": skill.get("name", "unknown"),
                    "days_idle": days_idle,
                    "success_rate": success_rate,
                    "suggested_action": "discard" if (is_idle and is_low_success) else "review",
                })
        await self._write_meta_event("diagnose_skill_redundancy", {
            "redundant_skills": len(redundancies),
            "total_scanned": len(skills),
        })
        return redundancies

    async def diagnose_code_bloat(self, source_dir=None, max_lines=1000) -> list[dict]:
        if not self.enabled:
            return []
        src = source_dir
        if src is None:
            src = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
        if not os.path.isdir(src):
            return []
        bloated = []
        total_scanned = 0
        for root, dirs, files in os.walk(src):
            dirs[:] = [d for d in dirs if not d.startswith("__pycache__") and not d.startswith(".")]
            for f in files:
                if f.endswith(".py"):
                    fp = os.path.join(root, f)
                    try:
                        with open(fp, "r", encoding="utf-8") as fh:
                            line_count = sum(1 for _ in fh)
                        total_scanned += 1
                        if line_count > max_lines:
                            bloated.append({
                                "file_path": fp,
                                "line_count": line_count,
                                "suggested_action": "refactor",
                            })
                    except Exception:
                        continue
        await self._write_meta_event("diagnose_code_bloat", {
            "bloated_files": len(bloated),
            "total_scanned": total_scanned,
        })
        return bloated

    async def generate_cleanup_proposal(self) -> dict:
        if not self.enabled:
            return {"enabled": False, "action": "skipped"}
        mem_red = await self.diagnose_memory_redundancy()
        skill_red = await self.diagnose_skill_redundancy()
        code_bloat = await self.diagnose_code_bloat()
        proposal_id = f"cleanup_{int(time.time())}_{self.instance_name}"
        proposal = {
            "proposal_id": proposal_id,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "status": "pending",
            "dimensions": {
                "memory_redundancy": mem_red,
                "skill_redundancy": skill_red,
                "code_bloat": code_bloat,
            },
            "summary": {
                "redundant_memory_topics": len(mem_red),
                "redundant_skills": len(skill_red),
                "bloated_files": len(code_bloat),
            },
        }
        self._cleanup_proposals[proposal_id] = proposal
        await self._write_meta_event("cleanup_proposal_generated", {
            "proposal_id": proposal_id,
            **proposal["summary"],
        })
        return proposal

    async def execute_cleanup(self, proposal_id, confirm=False) -> dict:
        if proposal_id not in self._cleanup_proposals:
            return {"success": False, "error": "proposal_not_found"}
        proposal = self._cleanup_proposals[proposal_id]
        if proposal["status"] != "pending":
            return {"success": False, "error": "proposal_already_executed"}
        if not confirm:
            return {"success": False, "error": "confirmation_required"}
        results = {"memory_pruned": 0, "skills_pruned": 0}
        for mem in proposal["dimensions"]["memory_redundancy"]:
            for cid in mem["capsule_ids"][2:]:
                if self.memory_store:
                    ok = await self.memory_store.forget(cid)
                    if ok:
                        results["memory_pruned"] += 1
        for sk in proposal["dimensions"]["skill_redundancy"]:
            if sk["suggested_action"] == "discard" and self.skill_store:
                ok = await self.skill_store.delete_skill(sk["skill_id"])
                if ok:
                    results["skills_pruned"] += 1
        proposal["status"] = "executed"
        await self._write_meta_event("cleanup_executed", {
            "proposal_id": proposal_id,
            **results,
        })
        return {"success": True, **results}

    async def _write_meta_event(self, event_type: str, data: dict):
        if not self.memory_store:
            return
        try:
            await self.memory_store.add_capsule({
                "description": f"self_pruning:{event_type} " + json.dumps(data),
                "source": "self_pruning",
                "importance_weight": 0.3,
                "timestamp": datetime.now(timezone.utc).isoformat(),
            })
        except Exception:
            self._logger.debug("failed to write meta event: %s", event_type)


import logging
import time
from datetime import datetime, timezone, timedelta
from typing import List, Optional


TEMPLATES = [
    "你之前决定{event}，结果{outcome}。现在回头看，你会改变什么？",
    "关于{topic}，我发现了一段与你当时说法不同的记忆。{quote}。你怎么看？",
    "你曾经{behavior}，当时你说{reason}。我注意到{observation}，想问问你的想法。",
]


class ChallengeMode:
    def __init__(self, memory_store, bond_tracker,
                 deep_dialogue_count: int = 0,
                 days_since_first: int = 0,
                 min_bond: float = 0.95,
                 min_deep_dialogue: int = 50,
                 min_days: int = 30,
                 cooldown_days: int = 7,
                 bond_cost: float = 0.02,
                 bond_reward: float = 0.01):
        self.memory_store = memory_store
        self.bond_tracker = bond_tracker
        self._deep_dialogue_count = deep_dialogue_count
        self._days_since_first = days_since_first
        self._min_bond = min_bond
        self._min_deep_dialogue = min_deep_dialogue
        self._min_days = min_days
        self._cooldown_days = cooldown_days
        self._bond_cost = bond_cost
        self._bond_reward = bond_reward
        self._last_challenge_time = 0.0
        self._logger = logging.getLogger("morn.consciousness")

    def is_unlocked(self) -> bool:
        if self.bond_tracker.get_bond() < self._min_bond:
            return False
        if self._deep_dialogue_count < self._min_deep_dialogue:
            return False
        if self._days_since_first < self._min_days:
            return False
        return True

    async def find_challenge_topics(self) -> List[dict]:
        topics = []
        cutoff = (datetime.now(timezone.utc) - timedelta(days=30)).isoformat()
        try:
            cursor = await self.memory_store.db.execute("""
                SELECT * FROM capsules
                WHERE timestamp < ? AND source NOT IN ('self_reflection', 'security')
                AND importance_weight >= 0.5
                ORDER BY emotion_score DESC LIMIT 5
            """, (cutoff,))
            rows = await cursor.fetchall()
            for row in rows:
                cap = dict(row)
                topics.append({
                    "template_index": 0,
                    "params": {
                        "event": cap.get("description", "某件事"),
                        "outcome": f"情感评分: {cap.get('emotion_score', 0):.1f}",
                    },
                })
        except Exception as e:
            self._logger.error(f"find_challenge_topics failed: {e}")
        return topics

    def generate_challenge(self, topic: dict) -> str:
        idx = topic.get("template_index", 0)
        if idx < 0 or idx >= len(TEMPLATES):
            idx = 0
        return TEMPLATES[idx].format(**topic["params"])

    def can_challenge_now(self) -> bool:
        if not self.is_unlocked():
            return False
        if self._last_challenge_time == 0:
            return True
        elapsed = time.time() - self._last_challenge_time
        return elapsed >= self._cooldown_days * 86400

    def trigger_challenge(self) -> float:
        bond = self.bond_tracker.get_bond()
        new_bond = max(0.0, bond - self._bond_cost)
        self.bond_tracker._bond = new_bond
        self._last_challenge_time = time.time()
        return new_bond

    def positive_response(self) -> float:
        bond = self.bond_tracker.get_bond()
        new_bond = min(1.0, bond + self._bond_reward)
        self.bond_tracker._bond = new_bond
        return new_bond

    def set_deep_dialogue_count(self, count: int):
        self._deep_dialogue_count = count

    def set_days_since_first(self, days: int):
        self._days_since_first = days