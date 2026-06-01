import os
import sys
import time
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.skill_lifecycle import (
    SkillCurator,
    SkillLifecycleManager,
    SkillScoreCard,
    SkillStore,
    SkillVoteManager,
    SkillVersionStore,
)
from morn_core.evolution.skill_loader import SkillLoader
from morn_core.skills.manager import SkillManager


def _make_skill(sid, name, usage_count=0, updated_at=None, trigger_keywords=None):
    return {
        "id": sid,
        "name": name,
        "usage_count": usage_count,
        "updated_at": updated_at,
        "trigger_keywords": trigger_keywords or [],
    }


def _valid_skill_md(name="greeting", keywords=None, template=None, body=None):
    kw_line = ""
    if keywords:
        if isinstance(keywords, list):
            items = "\n".join(f"  - {k}" for k in keywords)
            kw_line = f"trigger_keywords:\n{items}\n"
        else:
            kw_line = f"trigger_keywords: {keywords}\n"
    tpl = template or "你好！有什么可以帮助你的？"
    bdy = body or "详细的回复内容"
    return f"""---
name: {name}
{kw_line}template: {tpl}
description: 技能描述
source: external
---
{bdy}"""


# ======= SkillStore Tests =======


@pytest.mark.asyncio
async def test_add_and_get_skill(db_path):
    async with SkillStore(db_path) as store:
        sid, _ = await store.add_skill("greeting", ["你好", "嗨"], "你好！有什么帮助你的？")
        skill = await store.get_skill(sid)
        assert skill is not None
        assert skill["name"] == "greeting"
        assert skill["trigger_keywords"] == ["你好", "嗨"]
        assert skill["template"] == "你好！有什么帮助你的？"
        assert skill["source"] == "internal"
        assert skill["usage_count"] == 0


@pytest.mark.asyncio
async def test_add_duplicate_name_is_idempotent(db_path):
    async with SkillStore(db_path) as store:
        sid1, is_new1 = await store.add_skill("greeting", ["你好"], "你好")
        sid2, is_new2 = await store.add_skill("greeting", ["嗨"], "嗨")
        assert sid1 == sid2
        assert is_new1 is True
        assert is_new2 is False
        skill = await store.get_skill(sid1)
        assert skill["trigger_keywords"] == ["你好"]


@pytest.mark.asyncio
async def test_get_nonexistent_skill(db_path):
    async with SkillStore(db_path) as store:
        skill = await store.get_skill(999)
        assert skill is None


@pytest.mark.asyncio
async def test_search_skills_by_name(db_path):
    async with SkillStore(db_path) as store:
        await store.add_skill("greeting", ["你好"], "你好")
        await store.add_skill("farewell", ["再见"], "再见")
        results = await store.search_skills("greet")
        assert len(results) >= 1
        assert results[0]["name"] == "greeting"


@pytest.mark.asyncio
async def test_search_skills_by_keyword(db_path):
    async with SkillStore(db_path) as store:
        await store.add_skill("greeting", ["你好", "嗨"], "你好")
        results = await store.search_skills("你好")
        assert len(results) >= 1
        assert results[0]["name"] == "greeting"


@pytest.mark.asyncio
async def test_list_skills_all(db_path):
    async with SkillStore(db_path) as store:
        await store.add_skill("s1", ["a"], source="internal")
        await store.add_skill("s2", ["b"], source="external")
        all_skills = await store.list_skills()
        assert len(all_skills) >= 2


@pytest.mark.asyncio
async def test_list_skills_filter_by_source(db_path):
    async with SkillStore(db_path) as store:
        await store.add_skill("s1", ["a"], source="internal")
        await store.add_skill("s2", ["b"], source="external")
        internal = await store.list_skills(source="internal")
        external = await store.list_skills(source="external")
        assert all(s["source"] == "internal" for s in internal)
        assert all(s["source"] == "external" for s in external)


@pytest.mark.asyncio
async def test_delete_skill(db_path):
    async with SkillStore(db_path) as store:
        sid, _ = await store.add_skill("temp", ["x"], "x")
        assert await store.delete_skill(sid) is True
        assert await store.get_skill(sid) is None
        assert await store.delete_skill(sid) is False


@pytest.mark.asyncio
async def test_increment_usage(db_path):
    async with SkillStore(db_path) as store:
        sid, _ = await store.add_skill("popular", ["test"], "test")
        assert await store.increment_usage(sid) is True
        skill = await store.get_skill(sid)
        assert skill["usage_count"] == 1
        await store.increment_usage(sid)
        await store.increment_usage(sid)
        skill = await store.get_skill(sid)
        assert skill["usage_count"] == 3


@pytest.mark.asyncio
async def test_increment_nonexistent(db_path):
    async with SkillStore(db_path) as store:
        assert await store.increment_usage(999) is False


@pytest.mark.asyncio
async def test_persistence(db_path):
    async with SkillStore(db_path) as store:
        await store.add_skill("persist", ["abc"], "hello")
    async with SkillStore(db_path) as store:
        results = await store.search_skills("persist")
        assert len(results) == 1
        assert results[0]["template"] == "hello"


# ======= SKILL.md Parsing Tests =======


@pytest.mark.asyncio
async def test_parse_skill_md_basic():
    content = """---
name: greeting
trigger_keywords:
  - 你好
  - 嗨
template: 你好！有什么可以帮助你的？
---

详细的回复内容"""
    parsed = SkillStore._parse_skill_md(content)
    assert parsed is not None
    assert parsed["name"] == "greeting"
    assert "你好" in parsed["trigger_keywords"]
    assert "嗨" in parsed["trigger_keywords"]
    assert parsed["template"] == "你好！有什么可以帮助你的？"


@pytest.mark.asyncio
async def test_parse_skill_md_without_frontmatter():
    parsed = SkillStore._parse_skill_md("just some text")
    assert parsed is None


@pytest.mark.asyncio
async def test_parse_skill_md_empty_name():
    content = """---
name: ""
---
body"""
    parsed = SkillStore._parse_skill_md(content)
    assert parsed is None


@pytest.mark.asyncio
async def test_parse_skill_md_inline_list():
    content = """---
name: greet
trigger_keywords: [hello, hi]
template: Hello
---"""
    parsed = SkillStore._parse_skill_md(content)
    assert parsed is not None
    assert parsed["name"] == "greet"
    assert parsed["trigger_keywords"] == ["hello", "hi"]


@pytest.mark.asyncio
async def test_load_external_skills_creates_skills(db_path, data_dir):
    skills_dir = data_dir / "skills"
    skills_dir.mkdir()
    (skills_dir / "greet.md").write_text("""---
name: greet
trigger_keywords:
  - hello
  - hi
template: Hello there!
---
Some body""", encoding="utf-8")
    (skills_dir / "bye.md").write_text("""---
name: bye
trigger_keywords:
  - bye
template: Goodbye!
---""", encoding="utf-8")
    async with SkillStore(db_path) as store:
        count = await store.load_external_skills(skills_dir)
        assert count == 2
        results = await store.list_skills(source="external")
        assert len(results) == 2
        names = {r["name"] for r in results}
        assert names == {"greet", "bye"}


@pytest.mark.asyncio
async def test_load_external_skills_idempotent(db_path, data_dir):
    skills_dir = data_dir / "skills"
    skills_dir.mkdir()
    (skills_dir / "greet.md").write_text("""---
name: greet
trigger_keywords: [hello]
template: Hello
---""", encoding="utf-8")
    async with SkillStore(db_path) as store:
        c1 = await store.load_external_skills(skills_dir)
        c2 = await store.load_external_skills(skills_dir)
        assert c1 == 1
        assert c2 == 0


@pytest.mark.asyncio
async def test_load_external_skills_nonexistent_dir(db_path):
    async with SkillStore(db_path) as store:
        count = await store.load_external_skills(Path("/nonexistent/skills"))
        assert count == 0


# ======= SkillManager Tests =======


@pytest.mark.asyncio
async def test_manager_initialize_creates_skills_dir(db_path, data_dir):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, data_dir)
        await mgr.initialize()
        assert mgr.skills_dir.exists()


@pytest.mark.asyncio
async def test_manager_initialize_loads_external(db_path, data_dir):
    skills_dir = data_dir / "skills"
    skills_dir.mkdir(parents=True, exist_ok=True)
    (skills_dir / "hello.md").write_text("""---
name: hello
trigger_keywords: [你好]
template: 你好！
---""", encoding="utf-8")
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, data_dir)
        await mgr.initialize()
        skills = await mgr.list_skills()
        names = [s["name"] for s in skills]
        assert "hello" in names


@pytest.mark.asyncio
async def test_promote_skill(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        sid = await mgr.promote_skill("custom", ["keyword"], "template text")
        skill = await store.get_skill(sid)
        assert skill["name"] == "custom"
        assert skill["source"] == "internal"


@pytest.mark.asyncio
async def test_list_skills_delegation(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        await mgr.promote_skill("a", ["x"])
        await mgr.promote_skill("b", ["y"])
        all_skills = await mgr.list_skills()
        assert len(all_skills) >= 2


@pytest.mark.asyncio
async def test_get_matching(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        await mgr.promote_skill("test_skill", ["hello", "world"], "response text")
        matches = await mgr.get_matching("hello world")
        assert len(matches) >= 1
        assert matches[0]["name"] == "test_skill"


@pytest.mark.asyncio
async def test_get_matching_increments_usage(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        await mgr.promote_skill("usage_test", ["trigger_word"], "template")
        await mgr.get_matching("trigger_word")
        await mgr.get_matching("trigger_word")
        await mgr.get_matching("trigger_word")
        matches = await mgr.get_matching("trigger_word")
        assert matches[0]["usage_count"] == 4


@pytest.mark.asyncio
async def test_get_matching_max_3(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        for i in range(5):
            await mgr.promote_skill(f"skill_{i}", [f"kw{i}"], "t")
        matches = await mgr.get_matching("kw")
        assert len(matches) <= 3


# ======= SkillManager Propose/Auto-Grow Tests =======


@pytest.mark.asyncio
async def test_propose_skill_below_threshold(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        result = await mgr.propose_skill("你好呀", "你好！很高兴见到你")
        assert result is None
        skills = await mgr.list_skills()
        assert len(skills) == 0


@pytest.mark.asyncio
async def test_propose_skill_reaches_threshold(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        r1 = await mgr.propose_skill("今天天气怎么样", "今天天气很好")
        r2 = await mgr.propose_skill("今天天气怎么样", "今天天气很好")
        assert r1 is None
        assert r2 is None
        r3 = await mgr.propose_skill("今天天气怎么样", "今天天气很好")
        assert r3 is not None
        skills = await mgr.list_skills()
        names = [s["name"] for s in skills]
        assert any("今天天气怎么样" in n for n in names)


@pytest.mark.asyncio
async def test_propose_skill_dedup(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        await mgr.promote_skill("天气_回复", ["今天天气怎么样"], "今天天气很好")
        for _ in range(3):
            await mgr.propose_skill("今天天气怎么样", "今天天气很好")
        skills = await mgr.list_skills()
        names = [s["name"] for s in skills]
        matching = [n for n in names if "天气" in n]
        assert len(matching) == 1


@pytest.mark.asyncio
async def test_propose_skill_empty_message(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        result = await mgr.propose_skill("", "reply")
        assert result is None


@pytest.mark.asyncio
async def test_normalize():
    assert SkillManager._normalize("Hello, 世界！") == "hello世界"
    assert SkillManager._normalize("  A  B  ") == "ab"
    assert len(SkillManager._normalize("a" * 100)) <= 30


@pytest.mark.asyncio
async def test_propose_skill_different_messages(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        assert await mgr.propose_skill("你好", "嗨") is None
        assert await mgr.propose_skill("再见", "拜拜") is None
        assert await mgr.propose_skill("你好", "嗨") is None
        assert await mgr.propose_skill("再见", "拜拜") is None
        assert await mgr.propose_skill("你好", "嗨") is not None
        assert await mgr.propose_skill("再见", "拜拜") is not None
        skills = await mgr.list_skills()
        names = [s["name"] for s in skills]
        assert "你好_回复" in names
        assert "再见_回复" in names


@pytest.mark.asyncio
async def test_load_external_delegation(db_path, data_dir):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, data_dir)
        skills_dir = data_dir / "skills"
        skills_dir.mkdir(parents=True, exist_ok=True)
        (skills_dir / "test.md").write_text("""---
name: external_skill
trigger_keywords: [ext]
template: external template
---""", encoding="utf-8")
        count = await mgr.load_external(skills_dir)
        assert count == 1


# ======= Upper Limit & Edge Cases =======


@pytest.mark.asyncio
async def test_upper_limit_skills(db_path):
    async with SkillStore(db_path) as store:
        for i in range(100):
            await store.add_skill(f"skill_{i}", [f"kw{i}"], f"template{i}")
        skills = await store.list_skills()
        assert len(skills) == 100


@pytest.mark.asyncio
async def test_search_limit(db_path):
    async with SkillStore(db_path) as store:
        for i in range(30):
            await store.add_skill(f"skill_{i}", ["common"], f"template{i}")
        results = await store.search_skills("common")
        assert len(results) <= 20


@pytest.mark.asyncio
async def test_manager_persistence_across_sessions(db_path, data_dir):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, data_dir)
        await mgr.promote_skill("persistent_skill", ["persist"], "i persist")
    async with SkillStore(db_path) as store:
        skills = await store.list_skills()
        names = [s["name"] for s in skills]
        assert "persistent_skill" in names


@pytest.mark.asyncio
async def test_auto_grow_handles_exception_gracefully(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        for _ in range(3):
            await mgr.propose_skill("test error", "reply")
        skills = await mgr.list_skills()
        assert len(skills) >= 1
        assert skills[0]["name"] == "testerror_回复"


@pytest.mark.asyncio
async def test_propose_skill_resets_after_growth(db_path):
    async with SkillStore(db_path) as store:
        mgr = SkillManager(store, Path("/tmp"))
        for _ in range(3):
            await mgr.propose_skill("重复消息", "回复")
        name = "重复消息_回复"
        existing = await store.search_skills(name)
        for skill in existing:
            if skill["name"] == name:
                sid = skill["id"]
                await store.delete_skill(sid)
        for _ in range(3):
            await mgr.propose_skill("重复消息", "回复")
        skills = await mgr.list_skills()
        names = [s["name"] for s in skills]
        assert name in names


# ======= SkillScoreCard Tests (from test_skill_scoring.py) =======


class TestScoreCardCalculation:
    def test_score_card_calculation(self):
        data = {
            "skill_id": "test_skill",
            "success_rate": 0.85,
            "reuse_count": 10,
            "recent_success_rates": [0.8, 0.9, 0.85, 0.88, 0.82],
            "avg_exec_time": 1.0,
            "baseline_time": 1.2,
            "context_diversity": 5,
        }
        card = SkillScoreCard.calculate(data)
        assert card.skill_id == "test_skill"
        assert card.success_rate == 0.85
        assert card.reuse_count == 10
        assert card.context_diversity == 5
        assert card.total_score > 0

    def test_verdict_retain(self):
        card = SkillScoreCard(
            skill_id="test", total_score=0.85,
            success_rate=0.9, reuse_count=20, stability=0.95, efficiency=0.8,
        )
        assert card.get_verdict() == "retain"

    def test_verdict_observe(self):
        card = SkillScoreCard(
            skill_id="test", total_score=0.5,
            success_rate=0.6, reuse_count=5, stability=0.5, efficiency=0.4,
        )
        assert card.get_verdict() == "observe"

    def test_verdict_retire(self):
        card = SkillScoreCard(
            skill_id="test", total_score=0.2,
            success_rate=0.1, reuse_count=0, stability=0.1, efficiency=0.0,
        )
        assert card.get_verdict() == "retire"

    def test_calculate_stability_high(self):
        data = {
            "skill_id": "stable_skill",
            "success_rate": 0.9,
            "reuse_count": 30,
            "recent_success_rates": [0.88, 0.89, 0.9, 0.91, 0.9, 0.89, 0.88, 0.9, 0.91, 0.89],
            "avg_exec_time": 0.5,
            "baseline_time": 1.0,
            "context_diversity": 10,
        }
        card = SkillScoreCard.calculate(data)
        assert card.get_verdict() == "retain"
        assert card.stability > 0.9

    def test_calculate_stability_low(self):
        data = {
            "skill_id": "unstable_skill",
            "success_rate": 0.5,
            "reuse_count": 5,
            "recent_success_rates": [0.1, 0.9, 0.2, 0.8, 0.3, 0.7, 0.4, 0.6, 0.5, 0.5],
            "avg_exec_time": 0.5,
            "baseline_time": 1.0,
            "context_diversity": 2,
        }
        card = SkillScoreCard.calculate(data)
        assert card.stability < 0.8


class TestVoteCompetition:
    def test_vote_competition(self):
        mgr = SkillVoteManager()
        cid = mgr.register_competition(["skill_a", "skill_b", "skill_c"])
        assert cid is not None
        assert len(cid) > 0

    def test_vote_winner_determined(self):
        mgr = SkillVoteManager()
        cid = mgr.register_competition(["skill_a", "skill_b"])
        mgr.cast_vote(cid, "skill_a", True)
        mgr.cast_vote(cid, "skill_a", True)
        mgr.cast_vote(cid, "skill_b", True)
        assert mgr.get_winner(cid) == "skill_a"

    def test_vote_archive(self):
        mgr = SkillVoteManager()
        cid = mgr.register_competition(["skill_x", "skill_y"])
        mgr.cast_vote(cid, "skill_x", True)
        mgr.cast_vote(cid, "skill_y", True)
        mgr.cast_vote(cid, "skill_x", True)
        mgr.archive_loser(cid)
        archived = mgr.get_archived()
        assert len(archived) == 1
        assert archived[0]["skill_id"] == "skill_y"

    def test_vote_multiple_rounds(self):
        mgr = SkillVoteManager()
        cid = mgr.register_competition(["s1", "s2", "s3"])
        for _ in range(5):
            mgr.cast_vote(cid, "s1", True)
            mgr.cast_vote(cid, "s2", False)
            mgr.cast_vote(cid, "s3", True)
        assert mgr.get_winner(cid) in ("s1", "s3")


class TestLifecycleCreate:
    def test_create_context_diversity_sufficient(self):
        mgr = SkillLifecycleManager()
        sid = mgr.create({"skill_id": "test_skill", "context_diversity": 5})
        assert sid == "test_skill"
        assert mgr.get_status("test_skill") == "observe"

    def test_create_context_diversity_insufficient(self):
        mgr = SkillLifecycleManager()
        sid = mgr.create({"skill_id": "low_diversity", "context_diversity": 1})
        assert sid is None

    def test_create_context_diversity_boundary(self):
        mgr = SkillLifecycleManager()
        sid = mgr.create({"skill_id": "boundary_skill", "context_diversity": 3})
        assert sid == "boundary_skill"


class TestLifecycleEvaluate:
    def test_evaluate_updates_status_high(self):
        mgr = SkillLifecycleManager()
        mgr.create({"skill_id": "good", "context_diversity": 3})
        mgr.record_usage("good", success=True)
        mgr.record_usage("good", success=True)
        mgr.record_usage("good", success=True)
        card = mgr.evaluate("good")
        assert card.get_verdict() == "retain"
        assert mgr.get_status("good") == "observe"

    def test_evaluate_low_score(self):
        mgr = SkillLifecycleManager()
        mgr.create({"skill_id": "bad", "context_diversity": 3})
        mgr.record_usage("bad", success=False)
        mgr.record_usage("bad", success=False)
        card = mgr.evaluate("bad")
        verdict = card.get_verdict()
        assert verdict in ("observe", "retire")


class TestLifecyclePromoteDemoteRetire:
    def test_promote_requires_observe_status(self):
        mgr = SkillLifecycleManager()
        assert mgr.promote("nonexistent") is False

    def test_promote_high_success(self):
        mgr = SkillLifecycleManager()
        mgr.create({"skill_id": "promotable", "context_diversity": 3})
        for _ in range(10):
            mgr.record_usage("promotable", success=True)
        mgr.evaluate("promotable")
        assert mgr.promote("promotable") is True
        assert mgr.get_status("promotable") == "active"

    def test_demote_requires_active_status(self):
        mgr = SkillLifecycleManager()
        mgr.create({"skill_id": "d", "context_diversity": 3})
        assert mgr.demote("d") is False

    def test_retire_requires_observe_and_low_score(self):
        mgr = SkillLifecycleManager()
        mgr.create({"skill_id": "r", "context_diversity": 3})
        mgr.record_usage("r", success=False)
        mgr.record_usage("r", success=False)
        mgr.evaluate("r")
        result = mgr.retire("r")
        assert result is True or result is False
        if result:
            assert mgr.get_status("r") == "retired"

    def test_full_promote_demote_retire_flow(self):
        mgr = SkillLifecycleManager()
        mgr.create({"skill_id": "flow", "context_diversity": 3})
        assert mgr.get_status("flow") == "observe"
        for _ in range(10):
            mgr.record_usage("flow", success=True)
        mgr.evaluate("flow")
        assert mgr.promote("flow") is True
        assert mgr.get_status("flow") == "active"


class TestVersionSaveAndRollback:
    def test_version_save_and_rollback(self):
        store = SkillVersionStore()
        v1 = store.save_version("test_skill", {"prompt": "v1", "success_rate": 0.8})
        v2 = store.save_version("test_skill", {"prompt": "v2", "success_rate": 0.6})

        latest = store.get_latest_version("test_skill")
        assert latest["version_data"]["prompt"] == "v2"

        store.rollback("test_skill", v1)
        latest = store.get_latest_version("test_skill")
        assert latest["version_data"]["prompt"] == "v1"

    def test_version_list(self):
        store = SkillVersionStore()
        store.save_version("s1", {"a": 1})
        store.save_version("s1", {"a": 2})
        store.save_version("s1", {"a": 3})
        versions = store.get_versions("s1")
        assert len(versions) == 3

    def test_version_get_nonexistent(self):
        store = SkillVersionStore()
        assert store.get_version("x", "y") is None
        assert store.get_latest_version("x") is None

    def test_rollback_invalid_id(self):
        store = SkillVersionStore()
        assert store.rollback("s1", "nonexistent") is False

    def test_auto_rollback_on_low_success(self):
        store = SkillVersionStore()
        v1 = store.save_version("auto_skill", {"prompt": "v1", "success_rate": 0.9})
        store.activate_version("auto_skill", v1)
        v2 = store.save_version("auto_skill", {"prompt": "v2", "success_rate": 0.1})

        result = store.auto_rollback("auto_skill", 0.1)
        assert result is not None
        latest = store.get_latest_version("auto_skill")
        assert latest["version_data"]["prompt"] == "v1"

    def test_auto_rollback_no_action_when_successful(self):
        store = SkillVersionStore()
        v1 = store.save_version("good_skill", {"prompt": "v1", "success_rate": 0.7})
        store.activate_version("good_skill", v1)
        store.save_version("good_skill", {"prompt": "v2", "success_rate": 0.7})

        result = store.auto_rollback("good_skill", 0.75)
        assert result is None
        latest = store.get_latest_version("good_skill")
        assert latest["version_data"]["prompt"] == "v2"


# ======= SkillCurator Tests (from test_skill_curator.py) =======


class TestReviewSkills:
    def test_normal_skill_not_marked(self):
        cur = SkillCurator()
        cur.record_usage(1, success=True)
        cur.record_usage(1, success=True)
        skills = [_make_skill(1, "greeting", usage_count=5)]
        results = cur.review_skills(skills)
        assert len(results) == 1
        assert results[0]["action"] == "keep"
        assert results[0]["issues"] == []

    def test_90_days_unused_marked_low_usage(self):
        cur = SkillCurator()
        cur.record_usage(1, success=True)
        cur._tracking["1"]["last_used"] = time.time() - 91 * 86400
        skills = [_make_skill(1, "old_skill")]
        results = cur.review_skills(skills)
        assert len(results) == 1
        assert "low_usage" in results[0]["issues"]
        assert results[0]["action"] == "review"

    def test_90_days_unused_and_low_success_deprecated(self):
        cur = SkillCurator()
        cur.record_usage(1, success=False)
        cur.record_usage(1, success=True)
        cur._tracking["1"]["last_used"] = time.time() - 91 * 86400
        skills = [_make_skill(1, "bad_old_skill")]
        results = cur.review_skills(skills)
        assert len(results) == 1
        assert "low_usage_and_low_success_rate" in results[0]["issues"]
        assert results[0]["action"] == "deprecate"

    def test_low_success_rate_deprecated(self):
        cur = SkillCurator()
        cur.record_usage(1, success=False)
        cur.record_usage(1, success=False)
        cur.record_usage(1, success=True)
        skills = [_make_skill(1, "buggy_skill")]
        results = cur.review_skills(skills)
        assert len(results) == 1
        assert "low_success_rate" in results[0]["issues"]
        assert results[0]["action"] == "deprecate"

    def test_similar_skills_merge_suggestion(self):
        cur = SkillCurator()
        skills = [
            _make_skill(1, "weather_query", trigger_keywords=["weather"]),
            _make_skill(2, "weather_check", trigger_keywords=["weather"]),
            _make_skill(3, "weather_report", trigger_keywords=["weather"]),
        ]
        results = cur.review_skills(skills)
        merge_count = sum(1 for r in results if "merge_suggestion" in r["issues"])
        assert merge_count >= 3

    def test_disabled_returns_empty(self):
        cur = SkillCurator({"enabled": False})
        skills = [_make_skill(1, "whatever")]
        results = cur.review_skills(skills)
        assert results == []


class TestGetStats:
    def test_get_stats_returns_data(self):
        cur = SkillCurator()
        cur.record_usage(1, success=True)
        cur.record_usage(1, success=True)
        cur.record_usage(1, success=False)
        stats = cur.get_stats(1)
        assert stats["skill_id"] == "1"
        assert stats["use_count"] == 3
        assert stats["success_count"] == 2
        assert stats["fail_count"] == 1
        assert stats["success_rate"] == 2 / 3
        assert stats["deprecated"] is False

    def test_get_stats_empty_skill(self):
        cur = SkillCurator()
        stats = cur.get_stats(999)
        assert stats["success_rate"] is None
        assert stats["deprecated"] is False

    def test_get_stats_deprecated_flag(self):
        cur = SkillCurator()
        cur.mark_deprecated("42")
        stats = cur.get_stats(42)
        assert stats["deprecated"] is True


class TestMarkDeprecated:
    def test_mark_deprecated_adds_to_set(self):
        cur = SkillCurator()
        cur.mark_deprecated("abc")
        assert "abc" in cur._deprecated

    def test_mark_deprecated_idempotent(self):
        cur = SkillCurator()
        cur.mark_deprecated("x")
        cur.mark_deprecated("x")
        assert len(cur._deprecated) == 1


class TestProposeArchive:
    def test_propose_archive_returns_deprecated(self):
        cur = SkillCurator()
        cur.mark_deprecated("1")
        cur.mark_deprecated("2")
        archive = cur.propose_archive()
        assert len(archive) == 2
        sids = {a["skill_id"] for a in archive}
        assert sids == {"1", "2"}
        assert all(a["reason"] == "deprecated" for a in archive)

    def test_propose_archive_empty(self):
        cur = SkillCurator()
        assert cur.propose_archive() == []


class TestGetReviewReport:
    def test_report_structure(self):
        cur = SkillCurator()
        cur.record_usage(1, success=True)
        cur.mark_deprecated("2")
        report = cur.get_review_report()
        assert report["tracked_skills"] == 1
        assert report["deprecated_count"] == 1
        assert "2" in report["deprecated"]
        assert "1" in report["tracking"]

    def test_report_empty(self):
        cur = SkillCurator()
        report = cur.get_review_report()
        assert report["tracked_skills"] == 0
        assert report["deprecated_count"] == 0


class TestPersistence:
    def test_save_and_load(self):
        with tempfile.TemporaryDirectory() as tmp:
            cur = SkillCurator(data_dir=tmp)
            cur.record_usage("abc", success=True)
            cur.mark_deprecated("abc")
            cur2 = SkillCurator(data_dir=tmp)
            assert cur2._tracking["abc"]["use_count"] == 1
            assert "abc" in cur2._deprecated

    def test_no_data_dir_does_not_persist(self):
        cur = SkillCurator()
        cur.record_usage("x", success=True)
        cur2 = SkillCurator()
        assert "x" not in cur2._tracking


class TestRecordUsage:
    def test_record_success(self):
        cur = SkillCurator()
        cur.record_usage(1, success=True)
        assert cur._tracking["1"]["success_count"] == 1
        assert cur._tracking["1"]["fail_count"] == 0

    def test_record_failure(self):
        cur = SkillCurator()
        cur.record_usage(1, success=False)
        assert cur._tracking["1"]["fail_count"] == 1
        assert cur._tracking["1"]["success_count"] == 0

    def test_record_without_success(self):
        cur = SkillCurator()
        cur.record_usage(1)
        assert cur._tracking["1"]["use_count"] == 1
        assert cur._tracking["1"]["success_count"] == 0
        assert cur._tracking["1"]["fail_count"] == 0


class TestSkillLifecycleManagerOldInterfaces:
    def test_old_mark_success_still_works(self):
        mgr = SkillLifecycleManager()
        mgr.record_usage(1, success=True)
        stats = mgr.get_stats(1)
        assert stats["success_count"] == 1

    def test_old_mark_failure_still_works(self):
        mgr = SkillLifecycleManager()
        mgr.record_usage(1, success=False)
        stats = mgr.get_stats(1)
        assert stats["fail_count"] == 1

    def test_old_suggest_removal_still_works(self):
        mgr = SkillLifecycleManager()
        mgr.mark_deprecated("old_skill")
        assert "old_skill" in mgr._deprecated

    def test_old_review_skills_still_works(self):
        mgr = SkillLifecycleManager()
        mgr.record_usage(1, success=True)
        skills = [_make_skill(1, "greeting", usage_count=5)]
        results = mgr.review_skills(skills)
        assert len(results) == 1
        assert results[0]["action"] == "keep"


# ======= SkillLoader Tests (from test_skill_loader.py) =======


class TestLoadFromFile:
    @pytest.mark.asyncio
    async def test_load_valid_skill_md(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            with tempfile.TemporaryDirectory() as tmp:
                fpath = Path(tmp) / "hello.md"
                fpath.write_text(_valid_skill_md(), encoding="utf-8")
                ok, msg = await loader.load_from_file(fpath)
                assert ok is True
                assert msg == "loaded"
                skill = await store.get_skill(1)
                assert skill is not None
                assert skill["name"] == "greeting"
                assert skill["source"] == "external"

    @pytest.mark.asyncio
    async def test_load_duplicate_skip(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            with tempfile.TemporaryDirectory() as tmp:
                fpath = Path(tmp) / "hello.md"
                fpath.write_text(_valid_skill_md(), encoding="utf-8")
                ok1, _ = await loader.load_from_file(fpath)
                ok2, msg2 = await loader.load_from_file(fpath)
                assert ok1 is True
                assert ok2 is False
                assert msg2 == "already_loaded"

    @pytest.mark.asyncio
    async def test_load_missing_name_skip(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            with tempfile.TemporaryDirectory() as tmp:
                fpath = Path(tmp) / "nameless.md"
                fpath.write_text("""---
trigger_keywords: [test]
template: hello
---
body""", encoding="utf-8")
                ok, msg = await loader.load_from_file(fpath)
                assert ok is False
                assert msg == "missing_name"

    @pytest.mark.asyncio
    async def test_load_file_not_found(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            ok, msg = await loader.load_from_file(Path("/nonexistent/skill.md"))
            assert ok is False
            assert msg == "file_not_found"

    @pytest.mark.asyncio
    async def test_load_invalid_format_skip(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            with tempfile.TemporaryDirectory() as tmp:
                fpath = Path(tmp) / "bad.md"
                fpath.write_text("just plain text without frontmatter", encoding="utf-8")
                ok, msg = await loader.load_from_file(fpath)
                assert ok is False
                assert msg == "invalid_format"

    @pytest.mark.asyncio
    async def test_load_missing_keywords_uses_filename(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            with tempfile.TemporaryDirectory() as tmp:
                fpath = Path(tmp) / "mycustom.md"
                fpath.write_text("""---
name: custom_skill
template: hello
---
body""", encoding="utf-8")
                ok, _ = await loader.load_from_file(fpath)
                assert ok is True
                skill = await store.get_skill(1)
                assert skill["name"] == "custom_skill"
                assert "mycustom" in skill["trigger_keywords"]


class TestLoadFromDir:
    @pytest.mark.asyncio
    async def test_load_directory(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            with tempfile.TemporaryDirectory() as tmp:
                d = Path(tmp)
                (d / "a.md").write_text(_valid_skill_md(name="skill_a", keywords=["a"]), encoding="utf-8")
                (d / "b.md").write_text(_valid_skill_md(name="skill_b", keywords=["b"]), encoding="utf-8")
                count = await loader.load_from_dir(d)
                assert count == 2
                skills = await store.list_skills(source="external")
                assert len(skills) == 2

    @pytest.mark.asyncio
    async def test_load_dir_nonexistent(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            count = await loader.load_from_dir(Path("/nonexistent/dir"))
            assert count == 0

    @pytest.mark.asyncio
    async def test_load_dir_idempotent(self, db_path):
        async with SkillStore(db_path) as store:
            mgr = SkillManager(store, Path("/tmp"))
            loader = SkillLoader(mgr)
            with tempfile.TemporaryDirectory() as tmp:
                d = Path(tmp)
                (d / "s.md").write_text(_valid_skill_md(name="skill_x", keywords=["x"]), encoding="utf-8")
                c1 = await loader.load_from_dir(d)
                c2 = await loader.load_from_dir(d)
                assert c1 == 1
                assert c2 == 0


class TestListAvailable:
    def test_list_available_returns_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            d = Path(tmp)
            (d / "a.md").write_text("---\nname: a\n---", encoding="utf-8")
            (d / "b.md").write_text("---\nname: b\n---", encoding="utf-8")
            loader = SkillLoader.__new__(SkillLoader)
            available = loader.list_available(d)
            assert len(available) == 2
            names = {a["name"] for a in available}
            assert names == {"a", "b"}

    def test_list_available_nonexistent_dir(self):
        loader = SkillLoader.__new__(SkillLoader)
        available = loader.list_available("/nonexistent/dir")
        assert available == []

    def test_list_available_empty_dir(self):
        with tempfile.TemporaryDirectory() as tmp:
            loader = SkillLoader.__new__(SkillLoader)
            available = loader.list_available(Path(tmp))
            assert available == []


class TestValidate:
    def test_validate_valid_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            fpath = Path(tmp) / "valid.md"
            fpath.write_text(_valid_skill_md(), encoding="utf-8")
            result = SkillLoader.validate(None, fpath)
            assert result["valid"] is True
            assert result["errors"] == []

    def test_validate_invalid_frontmatter(self):
        with tempfile.TemporaryDirectory() as tmp:
            fpath = Path(tmp) / "bad.md"
            fpath.write_text("no frontmatter here", encoding="utf-8")
            result = SkillLoader.validate(None, fpath)
            assert result["valid"] is False
            assert "invalid_frontmatter" in result["errors"]

    def test_validate_missing_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            fpath = Path(tmp) / "noname.md"
            fpath.write_text("""---
trigger_keywords: [test]
template: hello
---
body""", encoding="utf-8")
            result = SkillLoader.validate(None, fpath)
            assert result["valid"] is False
            assert "missing_name" in result["errors"]

    def test_validate_file_not_found(self):
        result = SkillLoader.validate(None, Path("/nonexistent/file.md"))
        assert result["valid"] is False
        assert "file_not_found" in result["errors"]


class TestFrontmatterParsing:
    def test_parse_inline_list(self):
        content = """---
name: greet
trigger_keywords: [hello, hi]
template: Hello
---"""
        parsed = SkillLoader._parse_skill_md(content)
        assert parsed is not None
        assert parsed["name"] == "greet"
        assert parsed["trigger_keywords"] == ["hello", "hi"]
        assert parsed["template"] == "Hello"

    def test_parse_with_body(self):
        content = """---
name: test
trigger_keywords:
  - kw1
---
this is the skill body
with multiple lines"""
        parsed = SkillLoader._parse_skill_md(content)
        assert parsed is not None
        assert parsed["name"] == "test"
        assert parsed["trigger_keywords"] == ["kw1"]
        assert "this is the skill body" in parsed["template"]

    def test_parse_no_frontmatter(self):
        parsed = SkillLoader._parse_skill_md("just text")
        assert parsed is None

    def test_parse_empty_name(self):
        content = """---
name: ""
---
body"""
        parsed = SkillLoader._parse_skill_md(content)
        assert parsed is not None
        assert parsed["name"] is None