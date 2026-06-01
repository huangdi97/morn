import logging
from datetime import datetime, timezone, timedelta


PROTECTED_TYPES = frozenset({'l4', 'apz', 'evolution_log'})
TRASHABLE_TYPES = frozenset({'l2', 'l3', 'wiki'})


class TrashBin:
    def __init__(self, memory_store=None, retention_days=30,
                 auto_expire_enabled=True, notify_before_expiry=True,
                 require_confirm=True):
        self._items = {}
        self.memory_store = memory_store
        self.retention_days = retention_days
        self.auto_expire_enabled = auto_expire_enabled
        self.notify_before_expiry = notify_before_expiry
        self.require_confirm = require_confirm
        self._logger = logging.getLogger("morn.memory.trash_bin")

    def move_to_trash(self, data_id, data, data_type, reason):
        if data_type in PROTECTED_TYPES:
            raise ValueError(
                f"Cannot delete {data_type}: this data type is protected and cannot be trashed")
        if data_type not in TRASHABLE_TYPES:
            raise ValueError(f"Unknown data type: {data_type}")
        if data_id in self._items:
            raise ValueError(f"Data {data_id} is already in trash")

        self._items[data_id] = {
            "data": data,
            "data_type": data_type,
            "trashed_at": datetime.now(timezone.utc).isoformat(),
            "reason": reason,
            "restored_at": None,
        }
        self._write_log("move_to_trash", data_id, data_type, reason)
        return True

    def restore_from_trash(self, data_id):
        item = self._items.get(data_id)
        if item is None:
            return None
        item["restored_at"] = datetime.now(timezone.utc).isoformat()
        self._write_log("restore_from_trash", data_id, item["data_type"], item["reason"])
        return item["data"]

    def list_contents(self, data_type=None):
        result = []
        now = datetime.now(timezone.utc)
        for data_id, item in self._items.items():
            if data_type is not None and item["data_type"] != data_type:
                continue
            if item["restored_at"] is not None:
                continue
            preview = self._make_preview(item["data"])
            result.append([
                data_id,
                item["data_type"],
                item["trashed_at"],
                item["reason"],
                preview,
            ])
        return result

    def empty_trash(self, force=False):
        if self.require_confirm and not force:
            raise PermissionError(
                "Trash requires confirmation before emptying. Use force=True to bypass.")
        count = len(self._items)
        self._items.clear()
        self._write_log("empty_trash", "all", "all", f"emptied {count} items")
        return count

    def count(self):
        return len(self._items)

    def auto_expire(self):
        if not self.auto_expire_enabled:
            return 0
        now = datetime.now(timezone.utc)
        cutoff = now - timedelta(days=self.retention_days)
        to_remove = []
        notification_items = []
        for data_id, item in self._items.items():
            try:
                trashed_at = datetime.fromisoformat(item["trashed_at"])
            except (ValueError, TypeError):
                trashed_at = now
            if trashed_at < cutoff:
                to_remove.append(data_id)
            elif self.notify_before_expiry:
                expires_at = trashed_at + timedelta(days=self.retention_days)
                remaining = (expires_at - now).total_seconds()
                if 0 < remaining <= 86400:
                    notification_items.append((data_id, item))
        for data_id in to_remove:
            item = self._items.pop(data_id, None)
            if item:
                self._write_log("auto_expire", data_id, item["data_type"],
                                f"auto-expired after {self.retention_days} days")
        if notification_items:
            self._logger.info(
                "TrashBin: %d items expire within 24 hours", len(notification_items))
        return len(to_remove)

    def search(self, query):
        if not query:
            return []
        query_lower = query.lower()
        results = []
        for data_id, item in self._items.items():
            if item["restored_at"] is not None:
                continue
            preview = self._make_preview(item["data"])
            if query_lower in preview.lower() or query_lower in data_id.lower():
                results.append({
                    "id": data_id,
                    "data_type": item["data_type"],
                    "trashed_at": item["trashed_at"],
                    "reason": item["reason"],
                    "preview": preview,
                })
        return results

    def get_summary(self):
        if not self._items:
            return "垃圾桶为空。"
        active = [v for v in self._items.values() if v["restored_at"] is None]
        if not active:
            return "垃圾桶中所有数据已被恢复。"
        trashed_times = []
        for item in active:
            try:
                t = datetime.fromisoformat(item["trashed_at"])
            except (ValueError, TypeError):
                continue
            trashed_times.append(t)
        if not trashed_times:
            return f"垃圾桶中有 {len(active)} 条数据。"
        earliest = min(trashed_times)
        latest = max(trashed_times)
        earliest_str = earliest.strftime("%Y-%m-%d %H:%M")
        latest_str = latest.strftime("%Y-%m-%d %H:%M")
        return (
            f"垃圾桶中有 {len(active)} 条数据，"
            f"最早的是 {earliest_str}，"
            f"最晚的是 {latest_str}。"
        )

    @staticmethod
    def _make_preview(data, max_len=60):
        if isinstance(data, dict):
            text = str(data.get("description", data.get("content", str(data))))
        elif isinstance(data, str):
            text = data
        else:
            text = str(data)
        if len(text) > max_len:
            text = text[:max_len] + "..."
        return text

    def _write_log(self, action, data_id, data_type, reason):
        if not self.memory_store:
            return
        try:
            import asyncio
            description = (
                f"垃圾桶操作: {action} | ID: {data_id} | "
                f"类型: {data_type} | 原因: {reason}"
            )
            capsule = {
                "entities": '["morn", "trash_bin"]',
                "emotion_score": 0.0,
                "emotion_tag": "系统事件",
                "description": description,
                "source": "self_reflection",
                "importance_weight": 0.3,
            }
            try:
                asyncio.get_running_loop()
                asyncio.ensure_future(self.memory_store.add_capsule(capsule))
            except RuntimeError:
                asyncio.run(self.memory_store.add_capsule(capsule))
        except Exception as e:
            self._logger.error("Failed to write trash log: %s", e)
