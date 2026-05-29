import logging
import time

_POSITIVE_TAGS = ["怀念", "欣慰", "温暖回忆", "释然"]
_NEGATIVE_TAGS = ["遗憾", "心疼过去的自己", "感伤", "理解"]
_NEUTRAL_TAGS = ["回味", "重新理解", "不同视角"]


class HindsightEngine:
    def __init__(self, memory_store, chat_engine=None, config=None):
        self.memory_store = memory_store
        self.chat_engine = chat_engine
        self.enabled = config.get("hindsight_enabled", True) if config else True
        self.threshold_days = config.get("hindsight_threshold_days", 30)
        self.min_emotion = config.get("hindsight_min_emotion", 0.5)
        self._last_scan = 0.0
        self._logger = logging.getLogger("morn.consciousness.hindsight")
        self._scan_counter = 0

    async def scan_and_apply(self, current_emotion_state) -> list[dict]:
        if not self.enabled or not self.memory_store:
            return []
        eligible = await self.memory_store.get_eligible_for_hindsight(
            self.threshold_days, self.min_emotion)
        if not eligible:
            return []
        self._scan_counter += 1
        current_score = (
            current_emotion_state.pleasure
            if hasattr(current_emotion_state, 'pleasure')
            else 0.5
        )
        triggered = []
        for cap in eligible:
            original_score = cap.get("emotion_score", 0.5)
            diff = current_score - original_score
            if abs(diff) <= 0.3:
                continue
            if diff > 0.3:
                tags = _POSITIVE_TAGS
            elif diff < -0.3:
                tags = _NEGATIVE_TAGS
            else:
                tags = _NEUTRAL_TAGS
            tag_index = min(
                len(tags) - 1,
                max(0, int((current_score + original_score) / 2 * len(tags)))
            )
            tag_index = min(tag_index, len(tags) - 1)
            new_tag = tags[tag_index]
            emotion_desc = ""
            if hasattr(current_emotion_state, 'describe_state'):
                emotion_desc = current_emotion_state.describe_state()
            trigger_context = (
                f"hindsight_scan#{self._scan_counter}, "
                f"current_emotion: {emotion_desc or str(round(current_score, 2))}"
            )
            success = await self.memory_store.add_hindsight_mark(
                memory_id=cap["id"],
                new_tag=new_tag,
                new_emotion_score=current_score,
                trigger_context=trigger_context,
            )
            if success:
                triggered.append({
                    "memory_id": cap["id"],
                    "original_tag": cap.get("emotion_tag", ""),
                    "new_tag": new_tag,
                    "original_score": original_score,
                    "new_score": current_score,
                    "trigger_context": trigger_context,
                })
        return triggered

    async def tick(self, current_emotion_state, force=False) -> list[dict]:
        now = time.time()
        if not force and now - self._last_scan < 86400:
            return []
        self._last_scan = now
        return await self.scan_and_apply(current_emotion_state)
