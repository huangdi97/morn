import asyncio
import os
import sys
from datetime import datetime, timezone, timedelta
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.trash_bin import TrashBin, PROTECTED_TYPES, TRASHABLE_TYPES


@pytest.fixture
def trash_bin():
    return TrashBin()


@pytest.fixture
def trash_bin_with_store():
    store = MagicMock()
    store.add_capsule = AsyncMock()
    return TrashBin(memory_store=store)


class TestMoveToTrash:
    def test_move_l2_to_trash(self, trash_bin):
        trash_bin.move_to_trash("evt_001", {"description": "test"}, "l2", "user request")
        assert trash_bin.count() == 1

    def test_move_l3_to_trash(self, trash_bin):
        trash_bin.move_to_trash("k_001", {"content": "knowledge"}, "l3", "wrong fact")
        assert trash_bin.count() == 1

    def test_move_wiki_to_trash(self, trash_bin):
        trash_bin.move_to_trash("wiki_001", "wiki content", "wiki", "outdated")
        assert trash_bin.count() == 1

    def test_protected_l4_blocked(self, trash_bin):
        with pytest.raises(ValueError, match="protected"):
            trash_bin.move_to_trash("l4_001", {"content": "identity"}, "l4", "test")

    def test_protected_apz_blocked(self, trash_bin):
        with pytest.raises(ValueError, match="protected"):
            trash_bin.move_to_trash("apz_001", "dream", "apz", "test")

    def test_protected_evolution_log_blocked(self, trash_bin):
        with pytest.raises(ValueError, match="protected"):
            trash_bin.move_to_trash("log_001", "event", "evolution_log", "test")

    def test_unknown_type_raises(self, trash_bin):
        with pytest.raises(ValueError, match="Unknown"):
            trash_bin.move_to_trash("x_001", "data", "unknown_type", "test")

    def test_duplicate_move_raises(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "data", "l2", "test")
        with pytest.raises(ValueError, match="already in trash"):
            trash_bin.move_to_trash("evt_001", "data", "l2", "test")


class TestRestoreFromTrash:
    def test_restore_returns_data(self, trash_bin):
        data = {"description": "hello world"}
        trash_bin.move_to_trash("evt_001", data, "l2", "test")
        restored = trash_bin.restore_from_trash("evt_001")
        assert restored == data

    def test_restore_sets_restored_at(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "data", "l2", "test")
        trash_bin.restore_from_trash("evt_001")
        assert trash_bin._items["evt_001"]["restored_at"] is not None

    def test_restore_nonexistent_returns_none(self, trash_bin):
        assert trash_bin.restore_from_trash("nonexistent") is None


class TestListContents:
    def test_list_empty(self, trash_bin):
        assert trash_bin.list_contents() == []

    def test_list_all(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "hello world", "l2", "test")
        trash_bin.move_to_trash("k_001", "some knowledge", "l3", "test")
        contents = trash_bin.list_contents()
        assert len(contents) == 2

    def test_list_filtered_by_type(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "hello", "l2", "test")
        trash_bin.move_to_trash("k_001", "knowledge", "l3", "test")
        contents = trash_bin.list_contents(data_type="l3")
        assert len(contents) == 1
        assert contents[0][0] == "k_001"

    def test_list_excludes_restored(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "hello", "l2", "test")
        trash_bin.restore_from_trash("evt_001")
        assert trash_bin.list_contents() == []

    def test_list_preview_included(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "a" * 100, "l2", "test")
        contents = trash_bin.list_contents()
        assert len(contents[0][4]) <= 63


class TestEmptyTrash:
    def test_empty_requires_confirm(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "data", "l2", "test")
        with pytest.raises(PermissionError, match="confirmation"):
            trash_bin.empty_trash()

    def test_empty_force_clears(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "data", "l2", "test")
        trash_bin.move_to_trash("evt_002", "data", "l3", "test")
        count = trash_bin.empty_trash(force=True)
        assert count == 2
        assert trash_bin.count() == 0

    def test_empty_with_require_confirm_disabled(self):
        tb = TrashBin(require_confirm=False)
        tb.move_to_trash("evt_001", "data", "l2", "test")
        count = tb.empty_trash()
        assert count == 1


class TestCount:
    def test_count_zero_initially(self, trash_bin):
        assert trash_bin.count() == 0

    def test_count_after_moves(self, trash_bin):
        trash_bin.move_to_trash("a", "data", "l2", "test")
        trash_bin.move_to_trash("b", "data", "l3", "test")
        assert trash_bin.count() == 2

    def test_count_after_restore_does_not_decrement(self, trash_bin):
        trash_bin.move_to_trash("a", "data", "l2", "test")
        trash_bin.restore_from_trash("a")
        assert trash_bin.count() == 1


class TestAutoExpire:
    def test_expired_items_removed(self):
        tb = TrashBin(retention_days=30)
        old_time = (datetime.now(timezone.utc) - timedelta(days=31)).isoformat()
        tb._items["old_evt"] = {
            "data": "old", "data_type": "l2", "reason": "test",
            "trashed_at": old_time, "restored_at": None,
        }
        tb._items["recent_evt"] = {
            "data": "recent", "data_type": "l2", "reason": "test",
            "trashed_at": datetime.now(timezone.utc).isoformat(), "restored_at": None,
        }
        count = tb.auto_expire()
        assert count == 1
        assert "old_evt" not in tb._items
        assert "recent_evt" in tb._items

    def test_auto_expire_disabled(self):
        tb = TrashBin(auto_expire_enabled=False)
        old_time = (datetime.now(timezone.utc) - timedelta(days=31)).isoformat()
        tb._items["old_evt"] = {
            "data": "old", "data_type": "l2", "reason": "test",
            "trashed_at": old_time, "restored_at": None,
        }
        count = tb.auto_expire()
        assert count == 0

    def test_notify_before_expiry_logs(self):
        tb = TrashBin(notify_before_expiry=True)
        near_expiry = (datetime.now(timezone.utc) - timedelta(days=29, hours=23)).isoformat()
        tb._items["near_evt"] = {
            "data": "near", "data_type": "l2", "reason": "test",
            "trashed_at": near_expiry, "restored_at": None,
        }
        with patch.object(tb._logger, "info") as mock_info:
            tb.auto_expire()
            mock_info.assert_called_once()


class TestSearch:
    def test_search_by_preview(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "hello world test", "l2", "test")
        results = trash_bin.search("world")
        assert len(results) == 1

    def test_search_by_id(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "hello", "l2", "test")
        results = trash_bin.search("evt_001")
        assert len(results) == 1

    def test_search_no_match(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "hello", "l2", "test")
        results = trash_bin.search("xyz")
        assert results == []

    def test_search_empty_query(self, trash_bin):
        assert trash_bin.search("") == []

    def test_search_excludes_restored(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "hello", "l2", "test")
        trash_bin.restore_from_trash("evt_001")
        results = trash_bin.search("hello")
        assert results == []


class TestGetSummary:
    def test_summary_empty(self, trash_bin):
        assert "为空" in trash_bin.get_summary()

    def test_summary_with_items(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "data1", "l2", "test1")
        trash_bin.move_to_trash("evt_002", "data2", "l3", "test2")
        summary = trash_bin.get_summary()
        assert "2 条" in summary

    def test_summary_all_restored(self, trash_bin):
        trash_bin.move_to_trash("evt_001", "data", "l2", "test")
        trash_bin.restore_from_trash("evt_001")
        assert "所有数据已被恢复" in trash_bin.get_summary()


class TestConfig:
    def test_default_config(self):
        tb = TrashBin()
        assert tb.retention_days == 30
        assert tb.auto_expire_enabled is True
        assert tb.notify_before_expiry is True
        assert tb.require_confirm is True

    def test_custom_config(self):
        tb = TrashBin(retention_days=7, auto_expire_enabled=False,
                      notify_before_expiry=False, require_confirm=False)
        assert tb.retention_days == 7
        assert tb.auto_expire_enabled is False
        assert tb.notify_before_expiry is False
        assert tb.require_confirm is False


class TestIntegrationWithStore:
    @pytest.mark.asyncio
    async def test_move_to_trash_writes_log(self, trash_bin_with_store):
        trash_bin_with_store.move_to_trash("evt_001", {"desc": "test"}, "l2", "user request")
        await asyncio.sleep(0)
        trash_bin_with_store.memory_store.add_capsule.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_restore_writes_log(self, trash_bin_with_store):
        trash_bin_with_store._items["evt_001"] = {
            "data": "data", "data_type": "l2", "trashed_at": "2024-01-01T00:00:00",
            "reason": "test", "restored_at": None,
        }
        trash_bin_with_store.restore_from_trash("evt_001")
        await asyncio.sleep(0)
        assert trash_bin_with_store.memory_store.add_capsule.await_count == 1

    @pytest.mark.asyncio
    async def test_empty_trash_writes_log(self, trash_bin_with_store):
        trash_bin_with_store._items["evt_001"] = {
            "data": "data", "data_type": "l2", "trashed_at": "2024-01-01T00:00:00",
            "reason": "test", "restored_at": None,
        }
        trash_bin_with_store.empty_trash(force=True)
        await asyncio.sleep(0)
        assert trash_bin_with_store.memory_store.add_capsule.await_count == 1