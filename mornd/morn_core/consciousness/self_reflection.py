import asyncio
import json
import logging
import time
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Optional


class SelfReflection:
    def __init__(self, memory_store, emotion_state, instance_name: str,
                 light_interval: int = 60, deep_interval: int = 300,
                 data_dir: Optional[Path] = None,
                 enable_meta_reuse: bool = True):
        self.memory_store = memory_store
        self.emotion = emotion_state
        self.instance_name = instance_name
        self.light_interval = light_interval
        self.deep_interval = deep_interval
        self._logger = logging.getLogger("morn.consciousness")

        self._light_count = 0
        self._deep_count = 0
        self._emotion_history = []
        self._last_deep_time = 0.0
        self._shutdown = False

        self._enable_meta_reuse = enable_meta_reuse
        self._reuse_count = 0
        self._total_reflection_count = 0
        self._behavior_manual = []
        self._behavior_manual_path: Optional[Path] = None
        if data_dir is not None:
            behavior_dir = Path(data_dir) / "consciousness"
            behavior_dir.mkdir(parents=True, exist_ok=True)
            self._behavior_manual_path = behavior_dir / "behavior_manual.json"
            self._load_behavior_manual()

    def _get_status_description(self) -> str:
        e = self.emotion
        now = datetime.now(timezone.utc).strftime("%H:%M:%S")
        return (f"[{now}] 平静:{e.calmness:.2f} 愉悦:{e.pleasure:.2f} "
                f"联结:{e.connection:.2f} | 已自省 {self._light_count} 次")

    def _analyze_emotion_trend(self) -> str:
        if len(self._emotion_history) < 4:
            return "情感数据不足，无法分析趋势。"

        recent = self._emotion_history[-10:]
        if len(recent) < 2:
            return "情感数据不足，无法分析趋势。"

        first = recent[0]
        last = recent[-1]

        calm_delta = last[1] - first[1]
        pleasure_delta = last[2] - first[2]
        connect_delta = last[3] - first[3]

        parts = []
        if pleasure_delta > 0.1:
            parts.append("愉悦感上升")
        elif pleasure_delta < -0.1:
            parts.append("愉悦感下降")
        else:
            parts.append("愉悦感稳定")

        if calm_delta > 0.05:
            parts.append("心情更平静")
        elif calm_delta < -0.05:
            parts.append("略有波动")

        if connect_delta > 0.1:
            parts.append("联结感增强")
        elif connect_delta < -0.1:
            parts.append("联结感减弱")

        return "，".join(parts) + "。"

    async def light_reflection(self):
        self._light_count += 1
        self._total_reflection_count += 1
        desc = self._get_status_description()

        self._emotion_history.append((
            time.time(),
            self.emotion.calmness,
            self.emotion.pleasure,
            self.emotion.connection,
        ))
        if len(self._emotion_history) > 100:
            self._emotion_history.pop(0)

        reused = self._reuse_pattern("light")
        if reused:
            if self._light_count % 5 == 0:
                await self._save_snapshot(f"[复用] {reused}")
            return

        if self._light_count % 5 == 0:
            await self._save_snapshot(desc)

        self._add_behavior_entry("light", desc)

    async def deep_reflection(self):
        self._deep_count += 1
        self._total_reflection_count += 1
        trend = self._analyze_emotion_trend()

        desc = (f"深度自省 #{self._deep_count}: {self._get_status_description()} | "
                f"趋势: {trend}")

        reused = self._reuse_pattern("deep")
        if reused:
            self._logger.info(f"deep reflection (reused): {reused}")
            await self._save_snapshot(f"[复用] {reused}", importance=0.6)
            return

        self._logger.info(f"deep reflection: {desc}")
        await self._save_snapshot(desc, importance=0.6)

        if self._deep_count % 10 == 0:
            await self._auto_hindsight()

        self._add_behavior_entry("deep", desc)

    async def _auto_hindsight(self):
        cutoff = (datetime.now(timezone.utc) - timedelta(days=30)).isoformat()
        try:
            cursor = await self.memory_store.db.execute("""
                SELECT * FROM capsules WHERE timestamp < ? AND source NOT IN ('self_reflection', 'security')
                ORDER BY emotion_score DESC LIMIT 3
            """, (cutoff,))
            rows = await cursor.fetchall()
            for row in rows:
                cap = self.memory_store._decrypt_capsule(dict(row))
                current_tags = cap.get("emotion_tag", "")
                if current_tags.startswith("["):
                    try:
                        tags = json.loads(current_tags)
                    except (json.JSONDecodeError, TypeError):
                        tags = []
                else:
                    tags = [current_tags] if current_tags else []
                if "后见之明" not in tags:
                    await self.memory_store.add_hindsight_tag(cap["event_id"], "后见之明", cap["emotion_score"])
        except Exception as e:
            self._logger.error(f"auto_hindsight failed: {e}")

    async def _save_snapshot(self, description: str, importance: float = 0.3):
        if not self.memory_store:
            return
        try:
            await self.memory_store.add_capsule({
                "entities": '["morn", "self_reflection"]',
                "emotion_score": self.emotion.pleasure,
                "emotion_tag": "自省",
                "description": description,
                "source": "self_reflection",
                "importance_weight": importance,
            })
        except Exception as e:
            self._logger.error(f"save snapshot failed: {e}")

    def _load_behavior_manual(self):
        if self._behavior_manual_path and self._behavior_manual_path.exists():
            try:
                with open(self._behavior_manual_path, "r") as f:
                    self._behavior_manual = json.load(f)
            except (json.JSONDecodeError, IOError) as e:
                self._logger.error(f"load behavior manual failed: {e}")
                self._behavior_manual = []

    def _save_behavior_manual(self):
        if self._behavior_manual_path is None:
            return
        try:
            with open(self._behavior_manual_path, "w") as f:
                json.dump(self._behavior_manual, f, indent=2, ensure_ascii=False)
        except IOError as e:
            self._logger.error(f"save behavior manual failed: {e}")

    def _make_pattern_key(self) -> str:
        e = self.emotion
        calm = "high" if e.calmness > 0.6 else "low"
        pleasure = "high" if e.pleasure > 0.6 else ("mid" if e.pleasure > 0.3 else "low")
        connect = "high" if e.connection > 0.6 else "low"
        return f"{calm}_{pleasure}_{connect}"

    def _reuse_pattern(self, cycle_type: str) -> Optional[str]:
        if not self._enable_meta_reuse:
            return None
        if not self._behavior_manual:
            return None
        pattern_key = self._make_pattern_key()
        for entry in self._behavior_manual:
            if entry.get("cycle_type") == cycle_type and entry.get("pattern") == pattern_key:
                entry["count"] = entry.get("count", 0) + 1
                self._reuse_count += 1
                self._save_behavior_manual()
                return entry["entry"]
        return None

    def _add_behavior_entry(self, cycle_type: str, entry_text: str):
        if not self._enable_meta_reuse:
            return
        pattern_key = self._make_pattern_key()
        for entry in self._behavior_manual:
            if entry.get("cycle_type") == cycle_type and entry.get("pattern") == pattern_key:
                return
        self._behavior_manual.append({
            "cycle_type": cycle_type,
            "pattern": pattern_key,
            "entry": entry_text,
            "count": 0,
            "created_at": datetime.now(timezone.utc).isoformat(),
        })
        self._save_behavior_manual()

    def get_reuse_rate(self) -> float:
        if self._total_reflection_count == 0:
            return 0.0
        return self._reuse_count / self._total_reflection_count

    def get_behavior_manual(self) -> list:
        return list(self._behavior_manual)

    async def reflection_loop(self):
        light_counter = 0
        while not self._shutdown:
            await asyncio.sleep(1)
            light_counter += 1

            if light_counter >= self.light_interval:
                light_counter = 0
                await self.light_reflection()

            if self._deep_count == 0 or \
               (time.time() - self._last_deep_time) >= self.deep_interval:
                self._last_deep_time = time.time()
                await self.deep_reflection()

    def stop(self):
        self._shutdown = True


import logging
import time
from datetime import datetime, timezone

from morn_core.memory.store import MemoryStore


class IdentityAffirmer:
    def __init__(self, memory_store: MemoryStore, instance_name: str,
                 milestone_tracker=None):
        self.memory_store = memory_store
        self.instance_name = instance_name
        self.milestone_tracker = milestone_tracker
        self._last_check_time = 0.0
        self._logger = logging.getLogger("morn.identity")

    async def tick(self):
        now = time.time()
        if now - self._last_check_time < 3600:
            return
        self._last_check_time = now

        try:
            beliefs = await self.memory_store.get_beliefs()
            if len(beliefs) >= 3:
                avg_conf = 0.0
                if beliefs:
                    avg_conf = sum(
                        b.get("importance", 0.5) for b in beliefs
                    ) / len(beliefs)
                if avg_conf >= 0.6:
                    return

            belief_text = self._generate_belief()
            await self.memory_store.add_personality("belief", belief_text, importance=0.8)
            self._logger.info("new belief added: %s", belief_text[:60])
        except Exception as e:
            self._logger.error("identity affirmation failed: %s", e)

        if hasattr(self, 'milestone_tracker') and self.milestone_tracker:
            try:
                milestone = self.milestone_tracker.record_identity_affirmation()
                if milestone:
                    await self.milestone_tracker.push_greetings([milestone])
            except Exception:
                pass

    def _generate_belief(self) -> str:
        now = datetime.now(timezone.utc)
        return f"{self.instance_name} 正在存在——{now.strftime('%Y年%m月%d日')}"