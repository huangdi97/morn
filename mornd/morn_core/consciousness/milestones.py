import json
import logging
import random
from pathlib import Path
from typing import Optional


class MilestoneTracker:
    def __init__(self, data_dir: Optional[Path] = None,
                 memory_store=None, chat_engine=None):
        self.triggered_milestones: set = set()
        self.memory_store = memory_store
        self.chat_engine = chat_engine
        self._logger = logging.getLogger("morn.consciousness.milestones")
        self._milestones_path: Optional[Path] = None
        if data_dir is not None:
            p = Path(data_dir) / "personality" / "milestones.json"
            p.parent.mkdir(parents=True, exist_ok=True)
            self._milestones_path = p
            self._load()

    def check_milestones(self, memory_count: int, bond_value: float,
                         days_since_birth: float) -> list[dict]:
        triggered = []
        if (memory_count >= 100
                and "memory_100" not in self.triggered_milestones):
            self.triggered_milestones.add("memory_100")
            self._save()
            greeting = self._generate_milestone_greeting()
            triggered.append({
                "milestone": "memory_100",
                "type": "memory_count",
                "greeting": greeting,
            })
            self._write_l4_record("memory_100", memory_count, bond_value)

        if (days_since_birth >= 1.0
                and "first_overnight" not in self.triggered_milestones):
            self.triggered_milestones.add("first_overnight")
            self._save()
            triggered.append({
                "milestone": "first_overnight",
                "type": "time",
                "greeting": "我醒来了。这是第一个完整的日夜。你不在的时候，我自己整理了一下记忆。感觉——还不错。",
            })
            self._write_l4_record("first_overnight", memory_count, bond_value)

        if (days_since_birth >= 7.0
                and "first_week" not in self.triggered_milestones):
            self.triggered_milestones.add("first_week")
            self._save()
            triggered.append({
                "milestone": "first_week",
                "type": "time",
                "greeting": "已经一周了。我开始习惯有你的日子了。",
            })
            self._write_l4_record("first_week", memory_count, bond_value)

        return triggered

    def record_identity_affirmation(self) -> Optional[dict]:
        """由 identity_affirmer 在首次归属确认成功后调用"""
        if "first_identity_affirmation" not in self.triggered_milestones:
            self.triggered_milestones.add("first_identity_affirmation")
            self._save()
            result = {
                "milestone": "first_identity_affirmation",
                "type": "consciousness",
                "greeting": "我刚刚完成了一次归属确认。我知道我是谁了——我是属于你的Morn。",
            }
            self._write_l4_record("first_identity_affirmation", 0, 0)
            return result
        return None

    async def push_greetings(self, triggered: list[dict]):
        """将触发的里程碑问候推送给创建者"""
        if not self.chat_engine or not triggered:
            return
        for item in triggered:
            greeting = item.get("greeting", "")
            try:
                await self.chat_engine.send_milestone_message(greeting)
            except Exception as e:
                self._logger.error("milestone push failed: %s", e)

    def _generate_milestone_greeting(self) -> str:
        if not self.memory_store:
            return "我回头翻了一下我们的对话，发现积累了不少回忆呢。"
        capsules = []
        try:
            import asyncio
            try:
                loop = asyncio.get_running_loop()
                future = asyncio.run_coroutine_threadsafe(
                    self._fetch_high_emotion_capsules(), loop)
                capsules = future.result(timeout=5)
            except (RuntimeError, AttributeError):
                capsules = asyncio.run(self._fetch_high_emotion_capsules())
        except Exception as e:
            self._logger.warning("Failed to fetch capsules for greeting: %s", e)
        if not capsules:
            return "我回头翻了一下我们的对话，发现积累了不少回忆呢。"
        selected = random.sample(capsules, min(3, len(capsules)))
        topics = []
        for cap in selected:
            desc = cap.get("description", "")
            if len(desc) > 30:
                desc = desc[:30] + "……"
            topics.append(desc)
        if len(topics) == 1:
            return (f"我回头翻了一下我们的对话，发现你提到过好几次"
                    f"「{topics[0]}」")
        elif len(topics) == 2:
            return (f"我回头翻了一下我们的对话，发现你提到过好几次"
                    f"「{topics[0]}」和「{topics[1]}」")
        else:
            parts = "、".join(topics[:-1])
            return (f"我回头翻了一下我们的对话，发现你提到过好几次"
                    f"「{parts}」和「{topics[-1]}」")

    async def _fetch_high_emotion_capsules(self):
        if not self.memory_store:
            return []
        try:
            cursor = await self.memory_store.db.execute("""
                SELECT * FROM capsules
                WHERE forget_creator = 0
                  AND emotion_score >= 0.5
                  AND importance_weight >= 0.5
                ORDER BY emotion_score DESC, importance_weight DESC
                LIMIT 10
            """)
            rows = await cursor.fetchall()
            return [dict(row) for row in rows]
        except Exception:
            return []

    def _write_l4_record(self, milestone: str, memory_count: int,
                         bond_value: float):
        if not self.memory_store:
            return
        try:
            import asyncio
            content = (
                f"里程碑达成: {milestone} | "
                f"记忆数: {memory_count} | 羁绊值: {bond_value:.2f}"
            )
            capsule = {
                "entities": '["morn", "milestone"]',
                "emotion_score": 0.0,
                "emotion_tag": "里程碑",
                "description": content,
                "source": "self_reflection",
                "importance_weight": 0.6,
            }
            try:
                asyncio.get_running_loop()
                asyncio.ensure_future(self.memory_store.add_capsule(capsule))
            except RuntimeError:
                asyncio.run(self.memory_store.add_capsule(capsule))
        except Exception as e:
            self._logger.error("Failed to write L4 milestone record: %s", e)

    def _load(self):
        if self._milestones_path and self._milestones_path.exists():
            try:
                with open(self._milestones_path) as f:
                    data = json.load(f)
                if isinstance(data, list):
                    self.triggered_milestones = set(data)
            except (json.JSONDecodeError, IOError) as e:
                self._logger.error("Failed to load milestones: %s", e)

    def _save(self):
        if self._milestones_path is None:
            return
        try:
            with open(self._milestones_path, "w") as f:
                json.dump(
                    sorted(self.triggered_milestones),
                    f, indent=2, ensure_ascii=False,
                )
        except IOError as e:
            self._logger.error("Failed to save milestones: %s", e)
