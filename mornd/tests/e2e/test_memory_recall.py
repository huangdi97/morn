import json
import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from morn_core.memory.retrieval import RetrievalEngine


class TestMemoryRecall:
    @pytest.mark.asyncio
    async def test_short_term_recall(self, memory_store):
        eid = await memory_store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者说他最喜欢的颜色是蓝色",
            "emotion_score": 0.6,
        })
        result = await memory_store.get_capsule(eid)
        assert result is not None
        assert "蓝色" in result["description"]

    @pytest.mark.asyncio
    async def test_cross_conversation_recall(self, memory_store):
        await memory_store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者提到他养了一只猫叫咪咪",
            "emotion_score": 0.5,
        })
        engine = RetrievalEngine(memory_store)
        results = await engine.search(query="猫")
        assert len(results) > 0

    @pytest.mark.asyncio
    async def test_emotional_temperature_retrieval(self, memory_store):
        await memory_store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者说他今天非常开心",
            "emotion_score": 0.9,
            "emotion_tag": "高兴",
        })
        await memory_store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者说他今天上班迟到了",
            "emotion_score": 0.3,
            "emotion_tag": "沮丧",
        })
        engine = RetrievalEngine(memory_store)
        emotion_state = {"calmness": 0.3, "warmth": 0.8, "ripple": 0.6}
        results = await engine.search(
            query="今天",
            emotion_state=emotion_state,
        )
        assert len(results) > 0

    @pytest.mark.asyncio
    async def test_empty_retrieval(self, memory_store):
        engine = RetrievalEngine(memory_store)
        results = await engine.search(query="不存在的记忆")
        assert len(results) == 0

    @pytest.mark.asyncio
    async def test_search_fts_hit(self, memory_store):
        await memory_store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者正在学习西班牙语",
        })
        results = await memory_store.search_fts("西班牙")
        assert len(results) >= 1