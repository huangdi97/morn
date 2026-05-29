import logging
import time
from datetime import datetime, timedelta, timezone

from morn_core.memory.store import MemoryStore


class DreamEngine:
    def __init__(self, memory_store: MemoryStore, apz_store):
        self.memory_store = memory_store
        self.apz_store = apz_store
        self._last_dream_time = 0.0
        self._logger = logging.getLogger("morn.dream")

    async def tick(self, idle_seconds: float):
        now = time.time()
        if idle_seconds < 1800:
            return
        if now - self._last_dream_time < 86400:
            return
        try:
            await self._generate_dream()
            self._last_dream_time = now
        except Exception as e:
            self._logger.error("dream generation failed: %s", e)

    async def _generate_dream(self):
        cutoff_old = (datetime.now(timezone.utc) - timedelta(days=90)).isoformat()
        cutoff_recent = (datetime.now(timezone.utc) - timedelta(days=7)).isoformat()
        capsules = await self.memory_store.search_by_timerange(cutoff_old, cutoff_recent, limit=10)
        capsules = [c for c in capsules if c.get("emotion_score", 0) >= 0.5]
        capsules.sort(key=lambda c: c.get("emotion_score", 0), reverse=True)
        top = capsules[:5]

        if not top:
            return

        narrative_parts = []
        for cap in top:
            ts = cap.get("timestamp", "")[:10]
            desc = cap.get("description", "")[:100]
            narrative_parts.append(f"[{ts}] {desc}")
        narrative = "梦境碎片:\n" + "\n".join(narrative_parts)

        if self.apz_store:
            await self.apz_store.write(
                narrative,
                source="dream",
                emotion_tag="dream",
            )
            self._logger.info("dream recorded: %d memories", len(top))