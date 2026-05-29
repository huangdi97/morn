from abc import ABC, abstractmethod
from typing import Optional

class SkillStoreInterface(ABC):
    """SkillStore 的抽象接口，让 kernel/skills.py 依赖接口而非具体实现"""

    @abstractmethod
    async def get_skill(self, skill_id: str) -> Optional[dict]: ...

    @abstractmethod
    async def list_skills(self, tags: Optional[list[str]] = None) -> list[dict]: ...
