import logging
import re
from collections import defaultdict
from pathlib import Path
from typing import Optional

from morn_core.evolution.skill_lifecycle import SkillStore

logger = logging.getLogger("morn.skills")


class SkillManager:
    def __init__(self, skill_store: SkillStore, data_dir: Path):
        self.store = skill_store
        self.skills_dir = Path(data_dir) / "skills"
        self._pattern_counter: dict[str, int] = defaultdict(int)
        self._pattern_reply: dict[str, str] = {}

    @staticmethod
    def _normalize(text: str) -> str:
        cleaned = re.sub(r'[^\w\u4e00-\u9fff]', '', text)
        return cleaned.lower()[:30]

    async def initialize(self):
        self.skills_dir.mkdir(parents=True, exist_ok=True)
        loaded = await self.store.load_external_skills(self.skills_dir)
        if loaded:
            logger.info("Loaded %d external skills from %s", loaded, self.skills_dir)

    async def propose_skill(self, user_message: str, reply: str) -> Optional[str]:
        norm = self._normalize(user_message)
        if not norm:
            return None
        self._pattern_counter[norm] += 1
        self._pattern_reply[norm] = reply
        count = self._pattern_counter[norm]
        if count >= 3:
            return await self._auto_grow(norm, reply)
        return None

    async def _auto_grow(self, norm: str, reply: str) -> Optional[str]:
        existing = await self.store.search_skills(norm)
        for skill in existing:
            if norm in skill["name"] or any(norm in kw for kw in skill["trigger_keywords"]):
                self._pattern_counter[norm] = 0
                return None
        name = f"{norm}_回复"
        keywords = [norm]
        template = reply[:100]
        try:
            skill_id, _ = await self.store.add_skill(
                name=name,
                trigger_keywords=keywords,
                template=template,
                source="internal",
            )
            self._pattern_counter[norm] = 0
            logger.info("Auto-grown skill: %s (id=%d)", name, skill_id)
            return name
        except Exception as e:
            logger.warning("Auto-grow failed for %s: %s", norm, e)
            return None

    async def promote_skill(self, name: str, trigger_keywords: list[str],
                             template: str = "") -> int:
        sid, _ = await self.store.add_skill(
            name=name,
            trigger_keywords=trigger_keywords,
            template=template,
            source="internal",
        )
        return sid

    async def load_external(self, skills_dir: Optional[Path] = None) -> int:
        target = skills_dir or self.skills_dir
        return await self.store.load_external_skills(target)

    async def list_skills(self, source: Optional[str] = None) -> list[dict]:
        return await self.store.list_skills(source=source)

    async def get_matching(self, user_message: str) -> list[dict]:
        matches = await self.store.search_skills(user_message)
        result = []
        for skill in matches[:3]:
            await self.store.increment_usage(skill["id"])
            refreshed = await self.store.get_skill(skill["id"])
            if refreshed:
                result.append(refreshed)
        return result