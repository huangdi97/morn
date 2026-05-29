import asyncio
import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from morn_core.chat.engine import EmotionState
from morn_core.consciousness.self_reflection import SelfReflection


class TestConcurrency:
    @pytest.mark.asyncio
    async def test_concurrent_chat_no_conflict(self, chat_engine):
        tasks = [chat_engine.chat(f"消息{i}") for i in range(5)]
        results = await asyncio.gather(*tasks)
        assert len(results) == 5
        assert all(r for r in results)

    @pytest.mark.asyncio
    async def test_concurrent_memory_writes(self, memory_store):
        tasks = []
        for i in range(10):
            tasks.append(memory_store.add_capsule({
                "entities": '["test"]',
                "description": f"并发测试记忆#{i}",
                "source": "test",
            }))
        event_ids = await asyncio.gather(*tasks)
        assert len(event_ids) == 10
        assert all(eid for eid in event_ids)

    @pytest.mark.asyncio
    async def test_concurrent_reflection_and_chat(self, chat_engine, memory_store):
        reflection = SelfReflection(
            memory_store=memory_store,
            emotion_state=chat_engine.emotion,
            instance_name="Morn",
            light_interval=1,
            deep_interval=3,
        )

        async def run_reflection():
            for _ in range(3):
                await reflection.light_reflection()
                await asyncio.sleep(0.1)

        async def run_chat():
            for i in range(3):
                await chat_engine.chat(f"并发对话{i}")
                await asyncio.sleep(0.05)

        await asyncio.gather(run_reflection(), run_chat())
        assert reflection._light_count == 3

    @pytest.mark.asyncio
    async def test_message_flood(self, chat_engine):
        tasks = [chat_engine.chat(f"批量消息{j}") for j in range(20)]
        results = await asyncio.gather(*tasks)
        assert len(results) == 20
        assert all(r for r in results)

    @pytest.mark.asyncio
    async def test_concurrent_emotion_update(self):
        state = EmotionState()

        async def update_emotion(delta, tag):
            state.apply_delta(delta, tag)

        tasks = [
            update_emotion(0.3, "高兴"),
            update_emotion(-0.2, "失望"),
            update_emotion(0.5, "惊喜/感动"),
            update_emotion(-0.1, "低落"),
            update_emotion(0.2, "平静"),
        ]
        await asyncio.gather(*tasks)
        assert 0.0 <= state.pleasure <= 1.0
        assert 0.0 <= state.calmness <= 1.0
        assert 0.0 <= state.connection <= 1.0