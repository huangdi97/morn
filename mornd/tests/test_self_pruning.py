import os
import sys
import json
from unittest.mock import AsyncMock, MagicMock, PropertyMock, patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.consciousness.self_pruning import SelfPruner


@pytest.fixture
def mock_store():
    store = MagicMock()
    store.count = AsyncMock()
    store.count.return_value = 12000
    store.forget = AsyncMock()
    store.forget.return_value = True
    store.db = AsyncMock()
    return store


@pytest.fixture
def mock_skill_store():
    store = MagicMock()
    store.list_skills = AsyncMock()
    store.delete_skill = AsyncMock()
    store.delete_skill.return_value = True
    return store


@pytest.fixture
def emotion_history():
    return []


@pytest.fixture
def pruner(mock_store, mock_skill_store, emotion_history):
    return SelfPruner(
        memory_store=mock_store,
        skill_store=mock_skill_store,
        emotion_history_ref=emotion_history,
        instance_name="test_morn",
        max_capsules=10000,
        max_skills=50,
        max_emotion_history=1000,
        enabled=True,
    )


@pytest.fixture
def disabled_pruner(mock_store, mock_skill_store, emotion_history):
    return SelfPruner(
        memory_store=mock_store,
        skill_store=mock_skill_store,
        emotion_history_ref=emotion_history,
        instance_name="test_morn",
        max_capsules=10000,
        max_skills=50,
        max_emotion_history=1000,
        enabled=False,
    )


class TestDiagnose:
    @pytest.mark.asyncio
    async def test_diagnose_reports_excess(self, pruner, mock_store, mock_skill_store):
        mock_store.count.return_value = 12000
        mock_skill_store.list_skills.return_value = [{"id": i, "usage_count": 0, "created_at": ""} for i in range(60)]

        cursor_mock = AsyncMock()
        cursor_mock.fetchall.return_value = []
        mock_store.db.execute.return_value = cursor_mock

        result = await pruner.diagnose()

        assert result["capsule_excess"] == 2000
        assert result["skill_excess"] == 10

    @pytest.mark.asyncio
    async def test_diagnose_no_excess(self, pruner, mock_store, mock_skill_store):
        mock_store.count.return_value = 5000
        mock_skill_store.list_skills.return_value = [{"id": i, "usage_count": 0, "created_at": ""} for i in range(30)]

        result = await pruner.diagnose()

        assert result["capsule_excess"] == 0
        assert result["skill_excess"] == 0

    @pytest.mark.asyncio
    async def test_diagnose_prune_log(self, pruner, mock_store, mock_skill_store):
        mock_store.count.return_value = 5000
        mock_skill_store.list_skills.return_value = [{"id": i, "usage_count": 0, "created_at": ""} for i in range(30)]

        await pruner.diagnose()
        assert len(pruner.get_prune_log()) == 1

        await pruner.diagnose()
        assert len(pruner.get_prune_log()) == 2


class TestPruneMemory:
    @pytest.mark.asyncio
    async def test_prune_memory_under_threshold(self, pruner, mock_store):
        mock_store.count.return_value = 5000
        result = await pruner.prune_memory()
        assert result == 0

    @pytest.mark.asyncio
    async def test_prune_memory_removes_excess(self, pruner, mock_store):
        mock_store.count.return_value = 12000

        cursor_mock = AsyncMock()
        cursor_mock.fetchall.return_value = [
            {"event_id": f"evt_{i}", "importance_weight": 0.1, "source": "chat", "timestamp": "2024-01-01T00:00:00Z"}
            for i in range(2000)
        ]
        mock_store.db.execute.return_value = cursor_mock

        result = await pruner.prune_memory()
        assert result == 2000
        assert mock_store.forget.await_count == 2000

    @pytest.mark.asyncio
    async def test_prune_memory_skips_high_importance(self, pruner, mock_store):
        mock_store.count.return_value = 12000
        cursor_mock = AsyncMock()
        cursor_mock.fetchall.return_value = [
            {"event_id": "evt_high", "importance_weight": 0.9, "source": "chat", "timestamp": "2024-01-01T00:00:00Z"}
        ]
        mock_store.db.execute.return_value = cursor_mock

        mock_store.count.return_value = 12000
        result = await pruner.prune_memory()
        mock_store.forget.assert_not_called()
        assert result == 0


class TestPruneSkills:
    @pytest.mark.asyncio
    async def test_prune_skills_under_threshold(self, pruner, mock_skill_store):
        mock_skill_store.list_skills.return_value = [{"id": i, "usage_count": 10, "created_at": ""} for i in range(30)]
        result = await pruner.prune_skills()
        assert result == 0

    @pytest.mark.asyncio
    async def test_prune_skills_removes_least_used(self, pruner, mock_skill_store):
        mock_skill_store.list_skills.return_value = [
            {"id": i, "usage_count": i, "created_at": f"2024-01-{i+1:02d}"}
            for i in range(55)
        ]
        result = await pruner.prune_skills()
        assert result == 5
        assert mock_skill_store.delete_skill.await_count == 5


class TestPruneEmotionHistory:
    @pytest.mark.asyncio
    async def test_prune_emotion_history_under_threshold(self, pruner):
        result = await pruner.prune_emotion_history()
        assert result == 0

    @pytest.mark.asyncio
    async def test_prune_emotion_history_trims_excess(self, pruner, emotion_history):
        for i in range(1500):
            emotion_history.append((i, 0.5, 0.5, 0.5))
        result = await pruner.prune_emotion_history()
        assert result == 500
        assert len(emotion_history) == 1000


class TestThresholds:
    def test_default_thresholds(self, pruner):
        assert pruner.max_capsules == 10000
        assert pruner.max_skills == 50
        assert pruner.max_emotion_history == 1000

    def test_custom_thresholds(self):
        p = SelfPruner(
            memory_store=MagicMock(),
            max_capsules=500,
            max_skills=10,
            max_emotion_history=100,
        )
        assert p.max_capsules == 500
        assert p.max_skills == 10
        assert p.max_emotion_history == 100


class TestL4Protection:
    @pytest.mark.asyncio
    async def test_prune_memory_skips_self_reflection_source(self, pruner, mock_store):
        mock_store.count.return_value = 12000
        cursor_mock = AsyncMock()
        cursor_mock.fetchall.return_value = [
            {"event_id": "evt_l4", "importance_weight": 0.1, "source": "self_reflection", "timestamp": "2024-01-01T00:00:00Z"}
        ]
        mock_store.db.execute.return_value = cursor_mock
        result = await pruner.prune_memory()
        assert result == 0

    @pytest.mark.asyncio
    async def test_prune_memory_skips_identity_source(self, pruner, mock_store):
        mock_store.count.return_value = 12000
        cursor_mock = AsyncMock()
        cursor_mock.fetchall.return_value = [
            {"event_id": "evt_l4", "importance_weight": 0.1, "source": "identity", "timestamp": "2024-01-01T00:00:00Z"}
        ]
        mock_store.db.execute.return_value = cursor_mock
        result = await pruner.prune_memory()
        assert result == 0


class TestDisable:
    @pytest.mark.asyncio
    async def test_disabled_diagnose_skips(self, disabled_pruner):
        result = await disabled_pruner.diagnose()
        assert result["enabled"] is False
        assert result["action"] == "skipped"

    @pytest.mark.asyncio
    async def test_disabled_prune_memory_does_nothing(self, disabled_pruner):
        result = await disabled_pruner.prune_memory()
        assert result == 0

    @pytest.mark.asyncio
    async def test_disabled_prune_skills_does_nothing(self, disabled_pruner):
        result = await disabled_pruner.prune_skills()
        assert result == 0

    @pytest.mark.asyncio
    async def test_disabled_prune_emotion_history_does_nothing(self, disabled_pruner):
        result = await disabled_pruner.prune_emotion_history()
        assert result == 0


class TestPruneLog:
    @pytest.mark.asyncio
    async def test_prune_log_entries_accrue(self, pruner, mock_store, mock_skill_store):
        mock_store.count.return_value = 12000
        mock_skill_store.list_skills.return_value = [{"id": i, "usage_count": 0, "created_at": ""} for i in range(60)]
        cursor_mock = AsyncMock()
        cursor_mock.fetchall.return_value = [
            {"event_id": f"evt_{i}", "importance_weight": 0.1, "source": "chat", "timestamp": "2024-01-01T00:00:00Z"}
            for i in range(2000)
        ]
        mock_store.db.execute.return_value = cursor_mock

        await pruner.diagnose()
        log = pruner.get_prune_log()
        assert len(log) == 1
        entry = log[0]
        assert "capsules_pruned" in entry
        assert "timestamp" in entry

    @pytest.mark.asyncio
    async def test_prune_log_capped(self, pruner, mock_store, mock_skill_store):
        mock_store.count.return_value = 5000
        mock_skill_store.list_skills.return_value = [{"id": i, "usage_count": 0, "created_at": ""} for i in range(30)]
        for _ in range(120):
            await pruner.diagnose()
        assert len(pruner.get_prune_log()) == 100


# ── Upgrade tests (from test_self_pruning_upgrade.py) ──

@pytest.fixture
def upgrade_mock_store():
    store = MagicMock()
    store.count = AsyncMock(return_value=0)
    store.forget = AsyncMock(return_value=True)
    store.add_capsule = AsyncMock(return_value="meta_evt_1")
    store.db = AsyncMock()
    return store


@pytest.fixture
def upgrade_mock_skill_store():
    store = MagicMock()
    store.list_skills = AsyncMock(return_value=[])
    store.delete_skill = AsyncMock(return_value=True)
    return store


@pytest.fixture
def upgrade_pruner(upgrade_mock_store, upgrade_mock_skill_store):
    return SelfPruner(
        memory_store=upgrade_mock_store,
        skill_store=upgrade_mock_skill_store,
        emotion_history_ref=[],
        instance_name="test_upgrade",
        max_capsules=10000,
        max_skills=50,
        max_emotion_history=1000,
        enabled=True,
    )


@pytest.fixture
def upgrade_disabled_pruner(upgrade_mock_store, upgrade_mock_skill_store):
    return SelfPruner(
        memory_store=upgrade_mock_store,
        skill_store=upgrade_mock_skill_store,
        emotion_history_ref=[],
        instance_name="test_disabled",
        enabled=False,
    )


def _make_row(event_id, entities, description="test", importance=0.5, source="chat"):
    return {
        "event_id": event_id,
        "entities": json.dumps(entities),
        "description": description,
        "timestamp": "2025-01-01T00:00:00Z",
        "importance_weight": importance,
        "source": source,
    }


class TestDiagnoseMemoryRedundancy:
    @pytest.mark.asyncio
    async def test_detects_redundant_topics(self, upgrade_pruner, upgrade_mock_store):
        cursor = AsyncMock()
        cursor.fetchall.return_value = [
            _make_row("evt_1", ["ai", "safety"]),
            _make_row("evt_2", ["ai", "safety"]),
            _make_row("evt_3", ["ai", "safety"]),
            _make_row("evt_4", ["weather"]),
        ]
        upgrade_mock_store.db.execute.return_value = cursor

        result = await upgrade_pruner.diagnose_memory_redundancy(max_similar=3)

        assert len(result) == 1
        assert result[0]["topic"] == ["ai", "safety"]
        assert result[0]["count"] == 3
        assert result[0]["capsule_ids"] == ["evt_1", "evt_2", "evt_3"]

    @pytest.mark.asyncio
    async def test_no_redundancy_below_threshold(self, upgrade_pruner, upgrade_mock_store):
        cursor = AsyncMock()
        cursor.fetchall.return_value = [
            _make_row("evt_1", ["ai"]),
            _make_row("evt_2", ["safety"]),
        ]
        upgrade_mock_store.db.execute.return_value = cursor

        result = await upgrade_pruner.diagnose_memory_redundancy(max_similar=3)

        assert len(result) == 0

    @pytest.mark.asyncio
    async def test_untagged_capsules_grouped_together(self, upgrade_pruner, upgrade_mock_store):
        cursor = AsyncMock()
        cursor.fetchall.return_value = [
            _make_row("evt_1", [], description="first"),
            _make_row("evt_2", [], description="second"),
            _make_row("evt_3", [], description="third"),
        ]
        upgrade_mock_store.db.execute.return_value = cursor

        result = await upgrade_pruner.diagnose_memory_redundancy(max_similar=3)

        assert len(result) == 1
        assert result[0]["topic"] == "untagged"
        assert result[0]["count"] == 3


class TestDiagnoseSkillRedundancy:
    @pytest.mark.asyncio
    async def test_detects_idle_and_low_success_skills(self, upgrade_pruner, upgrade_mock_skill_store):
        upgrade_mock_skill_store.list_skills.return_value = [
            {"id": "sk_1", "name": "old_skill", "last_used_at": "2023-01-01T00:00:00Z", "success_rate": 0.5},
            {"id": "sk_2", "name": "good_skill", "last_used_at": "2026-05-01T00:00:00Z", "success_rate": 0.95},
        ]

        result = await upgrade_pruner.diagnose_skill_redundancy(max_idle_days=90, min_success_rate=0.8)

        assert len(result) == 1
        assert result[0]["skill_id"] == "sk_1"
        assert result[0]["suggested_action"] == "discard"

    @pytest.mark.asyncio
    async def test_no_redundant_skills(self, upgrade_pruner, upgrade_mock_skill_store):
        upgrade_mock_skill_store.list_skills.return_value = [
            {"id": "sk_1", "name": "recent_skill", "last_used_at": "2026-05-27T00:00:00Z", "success_rate": 0.95},
        ]

        result = await upgrade_pruner.diagnose_skill_redundancy(max_idle_days=90, min_success_rate=0.8)

        assert len(result) == 0

    @pytest.mark.asyncio
    async def test_empty_skill_store(self, upgrade_pruner, upgrade_mock_skill_store):
        upgrade_mock_skill_store.list_skills.return_value = []
        result = await upgrade_pruner.diagnose_skill_redundancy()
        assert len(result) == 0


class TestDiagnoseCodeBloat:
    @pytest.mark.asyncio
    async def test_detects_bloated_files(self, upgrade_pruner, tmp_path):
        bloated_file = tmp_path / "huge_module.py"
        with open(bloated_file, "w") as f:
            for _ in range(1500):
                f.write("x = 1\n")
        small_file = tmp_path / "tiny_module.py"
        with open(small_file, "w") as f:
            for _ in range(50):
                f.write("x = 1\n")

        result = await upgrade_pruner.diagnose_code_bloat(source_dir=str(tmp_path), max_lines=1000)

        assert len(result) == 1
        assert str(bloated_file) in result[0]["file_path"]
        assert result[0]["suggested_action"] == "refactor"

    @pytest.mark.asyncio
    async def test_no_bloat_below_threshold(self, upgrade_pruner, tmp_path):
        small_file = tmp_path / "tiny_module.py"
        with open(small_file, "w") as f:
            for _ in range(100):
                f.write("x = 1\n")

        result = await upgrade_pruner.diagnose_code_bloat(source_dir=str(tmp_path), max_lines=1000)

        assert len(result) == 0

    @pytest.mark.asyncio
    async def test_invalid_source_dir_returns_empty(self, upgrade_pruner):
        result = await upgrade_pruner.diagnose_code_bloat(source_dir="/nonexistent_path_xyz")
        assert len(result) == 0


class TestGenerateCleanupProposal:
    @pytest.mark.asyncio
    async def test_generates_proposal_with_all_dimensions(self, upgrade_pruner, upgrade_mock_store, upgrade_mock_skill_store):
        cursor = AsyncMock()
        cursor.fetchall.return_value = [
            _make_row("evt_1", ["ai"]),
            _make_row("evt_2", ["ai"]),
            _make_row("evt_3", ["ai"]),
        ]
        upgrade_mock_store.db.execute.return_value = cursor
        upgrade_mock_skill_store.list_skills.return_value = [
            {"id": "sk_1", "name": "old_skill", "last_used_at": "2023-01-01T00:00:00Z", "success_rate": 0.4},
        ]
        with patch.object(upgrade_pruner, "diagnose_code_bloat", return_value=[]):
            proposal = await upgrade_pruner.generate_cleanup_proposal()

        assert "proposal_id" in proposal
        assert proposal["status"] == "pending"
        assert proposal["summary"]["redundant_memory_topics"] == 1
        assert proposal["summary"]["redundant_skills"] == 1
        assert proposal["summary"]["bloated_files"] == 0
        assert "dimensions" in proposal

    @pytest.mark.asyncio
    async def test_disabled_pruner_skips(self, upgrade_disabled_pruner):
        result = await upgrade_disabled_pruner.generate_cleanup_proposal()
        assert result == {"enabled": False, "action": "skipped"}


class TestExecuteCleanup:
    @pytest.mark.asyncio
    async def test_executes_with_confirmation(self, upgrade_pruner, upgrade_mock_store, upgrade_mock_skill_store):
        cursor = AsyncMock()
        cursor.fetchall.return_value = [
            _make_row("evt_1", ["ai"]),
            _make_row("evt_2", ["ai"]),
            _make_row("evt_3", ["ai"]),
            _make_row("evt_4", ["ai"]),
        ]
        upgrade_mock_store.db.execute.return_value = cursor
        upgrade_mock_skill_store.list_skills.return_value = [
            {"id": "sk_1", "name": "old_skill", "last_used_at": "2023-01-01T00:00:00Z", "success_rate": 0.4},
        ]
        with patch.object(upgrade_pruner, "diagnose_code_bloat", return_value=[]):
            proposal = await upgrade_pruner.generate_cleanup_proposal()

        result = await upgrade_pruner.execute_cleanup(proposal["proposal_id"], confirm=True)

        assert result["success"] is True
        assert result["memory_pruned"] == 2
        assert result["skills_pruned"] == 1

    @pytest.mark.asyncio
    async def test_requires_confirmation(self, upgrade_pruner):
        proposal = await upgrade_pruner.generate_cleanup_proposal()

        result = await upgrade_pruner.execute_cleanup(proposal["proposal_id"], confirm=False)

        assert result["success"] is False
        assert result["error"] == "confirmation_required"

    @pytest.mark.asyncio
    async def test_proposal_not_found(self, upgrade_pruner):
        result = await upgrade_pruner.execute_cleanup("nonexistent_id", confirm=True)
        assert result["success"] is False
        assert result["error"] == "proposal_not_found"

    @pytest.mark.asyncio
    async def test_cannot_execute_twice(self, upgrade_pruner):
        with patch.object(upgrade_pruner, "diagnose_code_bloat", return_value=[]):
            proposal = await upgrade_pruner.generate_cleanup_proposal()

        await upgrade_pruner.execute_cleanup(proposal["proposal_id"], confirm=True)
        result = await upgrade_pruner.execute_cleanup(proposal["proposal_id"], confirm=True)

        assert result["success"] is False
        assert result["error"] == "proposal_already_executed"


class TestDisabledPruner:
    @pytest.mark.asyncio
    async def test_disabled_returns_empty_for_all_diagnoses(self, upgrade_disabled_pruner, upgrade_mock_store, upgrade_mock_skill_store):
        mem = await upgrade_disabled_pruner.diagnose_memory_redundancy()
        assert mem == []

        skill = await upgrade_disabled_pruner.diagnose_skill_redundancy()
        assert skill == []

        bloat = await upgrade_disabled_pruner.diagnose_code_bloat()
        assert bloat == []