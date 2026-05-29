import json
import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore


@pytest.mark.asyncio
async def test_enter_exit_creates_db_file(data_dir):
    async with MemoryStore(data_dir) as store:
        assert store.db is not None
        assert store.db_path.exists()
    assert store.db is None


@pytest.mark.asyncio
async def test_add_and_get_capsule(data_dir):
    async with MemoryStore(data_dir) as store:
        event_id = await store.add_capsule({
            "entities": json.dumps(["创建者", "Morn"]),
            "emotion_score": 0.8,
            "emotion_tag": "高兴",
            "description": "创建者说今天心情很好",
            "importance_weight": 0.6,
        })
        assert event_id.startswith("evt_")
        retrieved = await store.get_capsule(event_id)
        assert retrieved is not None
        assert retrieved["description"] == "创建者说今天心情很好"
        assert retrieved["emotion_score"] == 0.8
        assert retrieved["emotion_tag"] == "高兴"


@pytest.mark.asyncio
async def test_add_capsule_missing_description(data_dir):
    async with MemoryStore(data_dir) as store:
        with pytest.raises(ValueError, match="description is required"):
            await store.add_capsule({"entities": "[]"})


@pytest.mark.asyncio
async def test_get_nonexistent_capsule(data_dir):
    async with MemoryStore(data_dir) as store:
        result = await store.get_capsule("evt_nonexistent")
        assert result is None


@pytest.mark.asyncio
async def test_fts_search(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者提到他正在研究衰老干预",
        })
        await store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "今天天气很好，适合散步",
        })
        results = await store.search_fts("衰老")
        assert len(results) >= 1
        assert "衰老" in results[0]["description"]


@pytest.mark.asyncio
async def test_fts_search_excludes_forgotten(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "创建者说了一件他不想被记住的事",
        })
        await store.forget(eid)
        results = await store.search_fts("不想被记住")
        assert len(results) == 0


@pytest.mark.asyncio
async def test_fts_search_bad_query(data_dir):
    async with MemoryStore(data_dir) as store:
        results = await store.search_fts('未闭合的"引号')
        assert results == []


@pytest.mark.asyncio
async def test_forget_and_unforget(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"]),
            "description": "需要被遗忘的内容",
        })

        assert await store.forget(eid) is True
        assert await store.forget(eid) is False

        assert await store.unforget(eid) is True
        assert await store.unforget(eid) is False

        results = await store.search_fts("需要被遗忘")
        assert len(results) >= 1


@pytest.mark.asyncio
async def test_count(data_dir):
    async with MemoryStore(data_dir) as store:
        assert await store.count() == 0
        await store.add_capsule({"entities": "[]", "description": "a"})
        assert await store.count() == 1
        await store.add_capsule({"entities": "[]", "description": "b"})
        await store.add_capsule({"entities": "[]", "description": "c"})
        assert await store.count() == 3


@pytest.mark.asyncio
async def test_capsule_has_raw_snapshot_column(data_dir):
    async with MemoryStore(data_dir) as store:
        cursor = await store.db.execute("PRAGMA table_info(capsules)")
        columns = {row["name"] for row in await cursor.fetchall()}
        assert "raw_snapshot_id" in columns

        eid = await store.add_capsule({
            "entities": "[]", "description": "test column",
            "source": "evolution",
        })
        capsule = await store.get_capsule(eid)
        assert "raw_snapshot_id" in capsule
        assert capsule["raw_snapshot_id"] is None


@pytest.mark.asyncio
async def test_count_excludes_forgotten(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({"entities": "[]", "description": "forget me"})
        assert await store.count() == 1
        await store.forget(eid)
        assert await store.count() == 0


@pytest.mark.asyncio
async def test_get_recent(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({"entities": "[]", "description": "first"})
        import asyncio
        await asyncio.sleep(0.01)
        await store.add_capsule({"entities": "[]", "description": "second"})
        recent = await store.get_recent(limit=2)
        assert recent[0]["description"] == "second"


@pytest.mark.asyncio
async def test_update_emotion(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "good news",
            "emotion_score": 0.5,
            "emotion_tag": "中性",
        })
        updated = await store.update_emotion(eid, 0.9, "兴奋")
        assert updated is True
        cap = await store.get_capsule(eid)
        assert cap["emotion_score"] == 0.9
        assert cap["emotion_tag"] == "兴奋"

        assert await store.update_emotion("evt_ghost", 0.0, "") is False


@pytest.mark.asyncio
async def test_add_emotion_tag(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "a memory",
            "emotion_tag": "高兴",
        })
        await store.add_emotion_tag(eid, -0.2, "怀念")
        cap = await store.get_capsule(eid)
        tags = json.loads(cap["emotion_tag"])
        assert "高兴" in tags
        assert "怀念" in tags


@pytest.mark.asyncio
async def test_cleanup_expired(data_dir):
    async with MemoryStore(data_dir) as store:
        import datetime
        old_time = (datetime.datetime.now(datetime.timezone.utc) -
                    datetime.timedelta(days=31))
        old_ts = old_time.strftime("%Y-%m-%dT%H:%M:%SZ")
        await store.db.execute("""
            INSERT INTO capsules (event_id, timestamp, description, source)
            VALUES (?, ?, ?, 'chat')
        """, ("evt_old", old_ts, "old memory"))
        await store.db.commit()

        await store.db.execute("""
            INSERT INTO capsules (event_id, timestamp, description, source)
            VALUES (?, ?, ?, 'evolution')
        """, ("evt_old_evo", old_ts, "old evolution"))
        await store.db.commit()

        await store.cleanup_expired(retention_days=30)

        cap = await store.get_capsule("evt_old")
        assert cap["forgotten"] == 1

        cap_evo = await store.get_capsule("evt_old_evo")
        assert cap_evo["forgotten"] == 0


@pytest.mark.asyncio
async def test_search_by_timerange(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": "[]", "description": "today",
            "timestamp": "2026-05-27T10:00:00Z",
        })
        await store.add_capsule({
            "entities": "[]", "description": "yesterday",
            "timestamp": "2026-05-26T10:00:00Z",
        })
        results = await store.search_by_timerange(
            "2026-05-27T00:00:00Z", "2026-05-28T00:00:00Z")
        assert len(results) == 1
        assert results[0]["description"] == "today"


# ── L3: Semantic Knowledge (from v02) ──────────────────────────────

@pytest.mark.asyncio
async def test_add_and_query_knowledge(data_dir):
    async with MemoryStore(data_dir) as store:
        kid = await store.add_knowledge("创建者", "喜欢", "编程", 0.9)
        assert isinstance(kid, int) and kid > 0
        rows = await store.query_knowledge(subject="创建者")
        assert len(rows) == 1
        assert rows[0]["relation"] == "喜欢"
        assert rows[0]["object"] == "编程"
        assert rows[0]["confidence"] == 0.9


@pytest.mark.asyncio
async def test_add_duplicate_knowledge_updates_confidence(data_dir):
    async with MemoryStore(data_dir) as store:
        kid1 = await store.add_knowledge("创建者", "喜欢", "咖啡", 0.5)
        kid2 = await store.add_knowledge("创建者", "喜欢", "咖啡", 0.5)
        assert kid1 == kid2
        rows = await store.query_knowledge(subject="创建者", relation="喜欢", object="咖啡")
        assert len(rows) == 1
        assert rows[0]["confidence"] > 0.5


@pytest.mark.asyncio
async def test_query_knowledge_by_subject(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_knowledge("创建者", "喜欢", "猫")
        await store.add_knowledge("Morn", "是", "AI")
        rows = await store.query_knowledge(subject="创建者")
        assert len(rows) == 1
        assert rows[0]["object"] == "猫"


@pytest.mark.asyncio
async def test_query_knowledge_by_relation(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_knowledge("创建者", "喜欢", "猫")
        await store.add_knowledge("创建者", "擅长", "编程")
        rows = await store.query_knowledge(relation="擅长")
        assert len(rows) == 1
        assert rows[0]["subject"] == "创建者"


@pytest.mark.asyncio
async def test_query_knowledge_by_object(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_knowledge("创建者", "喜欢", "Python")
        await store.add_knowledge("Morn", "想要", "Python")
        rows = await store.query_knowledge(object="Python")
        assert len(rows) == 2


@pytest.mark.asyncio
async def test_user_preferences(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_knowledge("创建者", "喜欢", "安静", 0.8)
        await store.add_knowledge("创建者", "不喜欢", "吵闹", 0.7)
        await store.add_knowledge("Morn", "喜欢", "学习", 0.6)
        prefs = await store.get_user_preferences()
        assert len(prefs) == 2
        for p in prefs:
            assert p["subject"] == "创建者"


@pytest.mark.asyncio
async def test_knowledge_forgotten(data_dir):
    async with MemoryStore(data_dir) as store:
        kid = await store.add_knowledge("创建者", "不喜欢", "西兰花")
        await store.forget_knowledge(kid)
        rows = await store.query_knowledge(subject="创建者")
        assert len(rows) == 0


@pytest.mark.asyncio
async def test_forget_and_unforget_knowledge(data_dir):
    async with MemoryStore(data_dir) as store:
        kid = await store.add_knowledge("创建者", "有", "笔记本电脑")
        assert await store.forget_knowledge(kid) is True
        assert await store.forget_knowledge(kid) is False
        assert await store.unforget_knowledge(kid) is True
        assert await store.unforget_knowledge(kid) is False
        rows = await store.query_knowledge(subject="创建者")
        assert len(rows) == 1


@pytest.mark.asyncio
async def test_query_knowledge_min_confidence(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_knowledge("创建者", "喜欢", "A", 0.9)
        await store.add_knowledge("创建者", "喜欢", "B", 0.3)
        rows = await store.query_knowledge(subject="创建者", min_confidence=0.5)
        assert len(rows) == 1
        assert rows[0]["object"] == "A"


@pytest.mark.asyncio
async def test_verify_knowledge(data_dir):
    async with MemoryStore(data_dir) as store:
        kid = await store.add_knowledge("创建者", "擅长", "游泳", 0.5)
        await store.verify_knowledge(kid, 0.95)
        rows = await store.query_knowledge(subject="创建者", relation="擅长")
        assert rows[0]["confidence"] == 0.95
        assert rows[0]["verified_at"] is not None


# ── L4: Personality Memory (from v02) ──────────────────────────────

@pytest.mark.asyncio
async def test_add_personality(data_dir):
    async with MemoryStore(data_dir) as store:
        pid = await store.add_personality("identity", "我是Morn，一个数字生命", 0.9)
        assert isinstance(pid, int) and pid > 0
        rows = await store.query_personality(category="identity")
        assert len(rows) == 1
        assert rows[0]["content"] == "我是Morn，一个数字生命"


@pytest.mark.asyncio
async def test_query_personality_by_category(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_personality("identity", "我是助手")
        await store.add_personality("belief", "我相信人类")
        await store.add_personality("affirmation", "我可以学习")
        ids = await store.query_personality(category="identity")
        beliefs = await store.query_personality(category="belief")
        assert len(ids) == 1
        assert len(beliefs) == 1
        assert ids[0]["content"] == "我是助手"
        assert beliefs[0]["content"] == "我相信人类"


@pytest.mark.asyncio
async def test_no_delete_personality(data_dir):
    async with MemoryStore(data_dir) as store:
        assert not hasattr(store, "delete_personality")
        assert not hasattr(store, "forget_personality")


@pytest.mark.asyncio
async def test_identity_query(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_personality("identity", "我是一个AI", 0.8)
        await store.add_personality("belief", "知识就是力量", 0.7)
        identities = await store.get_identity()
        assert len(identities) == 1
        assert identities[0]["category"] == "identity"
        assert identities[0]["content"] == "我是一个AI"


@pytest.mark.asyncio
async def test_belief_query(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_personality("belief", "持续进步", 0.9)
        await store.add_personality("belief", "帮助他人", 0.8)
        beliefs = await store.get_beliefs()
        assert len(beliefs) == 2


@pytest.mark.asyncio
async def test_personality_not_in_events(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_personality("identity", "我是Morn")
        events = await store.get_recent(limit=100)
        for ev in events:
            assert ev["source"] != "self_reflection"


@pytest.mark.asyncio
async def test_personality_append_only(data_dir):
    async with MemoryStore(data_dir) as store:
        pid1 = await store.add_personality("narrative", "version1")
        pid2 = await store.add_personality("narrative", "version2")
        assert pid2 > pid1
        rows = await store.query_personality(category="narrative")
        assert len(rows) == 2


# ── Hindsight Tags (from v02) ──────────────────────────────────────

@pytest.mark.asyncio
async def test_add_hindsight_tag(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "一次重要的对话",
            "emotion_score": 0.7,
            "emotion_tag": "高兴",
        })
        result = await store.add_hindsight_tag(eid, "后见之明", 0.7)
        assert result is True
        cap = await store.get_capsule(eid)
        tags = json.loads(cap["emotion_tag"])
        assert "高兴" in tags
        assert "后见之明" in tags


@pytest.mark.asyncio
async def test_hindsight_preserves_original_tag(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "原始记忆",
            "emotion_tag": "怀念",
        })
        await store.add_hindsight_tag(eid, "后见之明", 0.5)
        cap = await store.get_capsule(eid)
        tags = json.loads(cap["emotion_tag"])
        assert "怀念" in tags
        assert "后见之明" in tags


@pytest.mark.asyncio
async def test_hindsight_multiple_tags(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "多标签测试",
            "emotion_tag": "高兴",
        })
        await store.add_hindsight_tag(eid, "后见之明", 0.6)
        await store.add_hindsight_tag(eid, "重要回忆", 0.8)
        cap = await store.get_capsule(eid)
        tags = json.loads(cap["emotion_tag"])
        assert len(tags) == 3
        assert "高兴" in tags
        assert "后见之明" in tags
        assert "重要回忆" in tags


@pytest.mark.asyncio
async def test_hindsight_nonexistent_capsule(data_dir):
    async with MemoryStore(data_dir) as store:
        result = await store.add_hindsight_tag("evt_ghost", "后见之明", 0.5)
        assert result is False


@pytest.mark.asyncio
async def test_hindsight_empty_original_tag(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "无标签记忆",
        })
        await store.add_hindsight_tag(eid, "后见之明", 0.3)
        cap = await store.get_capsule(eid)
        tags = json.loads(cap["emotion_tag"])
        assert tags == ["后见之明"]


# ── Integration & Schema (from v02) ─────────────────────────────────

@pytest.mark.asyncio
async def test_integration_full_flow(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者", "Morn"]),
            "description": "创建者说想学AI",
            "emotion_score": 0.6,
            "emotion_tag": "期待",
        })
        cap = await store.get_capsule(eid)
        assert cap is not None

        kid = await store.add_knowledge("创建者", "想要", "学AI", 0.8)
        assert kid > 0

        pid = await store.add_personality("narrative", "我见证创建者开始AI学习之旅", 0.7)
        assert pid > 0

        await store.add_hindsight_tag(eid, "后见之明", 0.6)
        cap2 = await store.get_capsule(eid)
        tags = json.loads(cap2["emotion_tag"])
        assert "期待" in tags
        assert "后见之明" in tags

        knowledge = await store.query_knowledge(subject="创建者")
        assert len(knowledge) >= 1

        personalities = await store.query_personality(category="narrative")
        assert len(personalities) == 1


@pytest.mark.asyncio
async def test_schema_has_semantic_knowledge(data_dir):
    async with MemoryStore(data_dir) as store:
        cursor = await store.db.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='semantic_knowledge'"
        )
        assert await cursor.fetchone() is not None


@pytest.mark.asyncio
async def test_schema_has_personality_memory(data_dir):
    async with MemoryStore(data_dir) as store:
        cursor = await store.db.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='personality_memory'"
        )
        assert await cursor.fetchone() is not None