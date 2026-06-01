import json
import logging
import os
import time
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Any, Optional

logger = logging.getLogger("morn.external_memory")


class BaseMemoryAdapter(ABC):
    @abstractmethod
    def store(self, memory_data: Any) -> bool:
        ...

    @abstractmethod
    def retrieve(self, query: str, limit: int = 10) -> list[dict]:
        ...

    @abstractmethod
    def connect(self) -> bool:
        ...

    @abstractmethod
    def disconnect(self) -> bool:
        ...


class VoidAdapter(BaseMemoryAdapter):
    def store(self, memory_data: Any) -> bool:
        return True

    def retrieve(self, query: str, limit: int = 10) -> list[dict]:
        return []

    def connect(self) -> bool:
        return True

    def disconnect(self) -> bool:
        return True


class Mem0Adapter(BaseMemoryAdapter):
    def __init__(self, config: Optional[dict] = None):
        self.config = config or {}
        self._connected = False

    def store(self, memory_data: Any) -> bool:
        if not self._connected:
            return False
        logger.info("Mem0Adapter.store: %s", str(memory_data)[:80])
        return True

    def retrieve(self, query: str, limit: int = 10) -> list[dict]:
        if not self._connected:
            return []
        return []

    def connect(self) -> bool:
        self._connected = True
        logger.info("Mem0Adapter connected (endpoint=%s)", self.config.get("endpoint", "default"))
        return True

    def disconnect(self) -> bool:
        self._connected = False
        return True


class LettaAdapter(BaseMemoryAdapter):
    def __init__(self, config: Optional[dict] = None):
        self.config = config or {}
        self._connected = False

    def store(self, memory_data: Any) -> bool:
        if not self._connected:
            return False
        logger.info("LettaAdapter.store: %s", str(memory_data)[:80])
        return True

    def retrieve(self, query: str, limit: int = 10) -> list[dict]:
        if not self._connected:
            return []
        return []

    def connect(self) -> bool:
        self._connected = True
        logger.info("LettaAdapter connected (endpoint=%s)", self.config.get("endpoint", "default"))
        return True

    def disconnect(self) -> bool:
        self._connected = False
        return True


class ZepAdapter(BaseMemoryAdapter):
    def __init__(self, config: Optional[dict] = None):
        self.config = config or {}
        self._connected = False

    def store(self, memory_data: Any) -> bool:
        if not self._connected:
            return False
        logger.info("ZepAdapter.store: %s", str(memory_data)[:80])
        return True

    def retrieve(self, query: str, limit: int = 10) -> list[dict]:
        if not self._connected:
            return []
        return []

    def connect(self) -> bool:
        self._connected = True
        logger.info("ZepAdapter connected (endpoint=%s)", self.config.get("endpoint", "default"))
        return True

    def disconnect(self) -> bool:
        self._connected = False
        return True


ADAPTER_REGISTRY = {
    "void": VoidAdapter,
    "mem0": Mem0Adapter,
    "letta": LettaAdapter,
    "zep": ZepAdapter,
}


class ExternalMemoryAdapter:
    def __init__(self, config: Optional[dict] = None):
        self.config = config or {}
        self.adapter_type = self.config.get("adapter_type", "void")
        self._create_adapter()

    def _create_adapter(self):
        adapter_cls = ADAPTER_REGISTRY.get(self.adapter_type)
        if adapter_cls is None:
            logger.warning("unknown adapter_type '%s', falling back to void", self.adapter_type)
            adapter_cls = VoidAdapter
            self.adapter_type = "void"
        adapter_config = self.config.get(f"{self.adapter_type}_config", {})
        if adapter_cls is VoidAdapter:
            self._adapter = adapter_cls()
        else:
            self._adapter = adapter_cls(adapter_config)

    def set_adapter(self, adapter_type: str, adapter_config: Optional[dict] = None):
        if adapter_type not in ADAPTER_REGISTRY:
            raise ValueError(f"unknown adapter type: {adapter_type}, must be one of {list(ADAPTER_REGISTRY.keys())}")
        self.adapter_type = adapter_type
        self.config["adapter_type"] = adapter_type
        if adapter_config:
            self.config[f"{adapter_type}_config"] = adapter_config
        self._create_adapter()

    def store_memory(self, memory_data: Any) -> bool:
        return self._adapter.store(memory_data)

    def retrieve(self, query: str, limit: int = 10) -> list[dict]:
        return self._adapter.retrieve(query, limit)

    def connect(self) -> bool:
        return self._adapter.connect()

    def disconnect(self) -> bool:
        return self._adapter.disconnect()

    def wiki_search(self, query: str) -> list[dict]:
        wiki_path = self.config.get("wiki_path")
        if not wiki_path:
            return []
        path = Path(wiki_path)
        if not path.exists():
            logger.warning("wiki_search: path '%s' does not exist", wiki_path)
            return []
        results = []
        if path.is_file():
            content = path.read_text(encoding="utf-8", errors="ignore")
            if query.lower() in content.lower():
                results.append({"source": str(path), "content": content[:500]})
        return results

    def obsidian_search(self, query: str) -> list[dict]:
        vault_path = self.config.get("obsidian_vault_path")
        if not vault_path:
            return []
        path = Path(vault_path)
        if not path.exists():
            logger.warning("obsidian_search: vault path '%s' does not exist", vault_path)
            return []
        results = []
        if path.is_dir():
            for f in path.rglob("*.md"):
                try:
                    content = f.read_text(encoding="utf-8", errors="ignore")
                    if query.lower() in content.lower():
                        results.append({"source": str(f), "content": content[:500]})
                except (OSError, IOError):
                    continue
        return results

    async def auto_archive_wiki(self, days_threshold: int = 90) -> int:
        wiki_path = self.config.get("wiki_path")
        if not wiki_path:
            logger.warning("auto_archive_wiki: no wiki_path configured")
            return 0
        path = Path(wiki_path)
        if not path.exists():
            return 0
        from datetime import datetime, timezone, timedelta
        cutoff = datetime.now(timezone.utc) - timedelta(days=days_threshold)
        archived = 0
        try:
            from morn.sdk.trash_bin import TrashBin
            trash = TrashBin()
            if path.is_file():
                entries = {}
                content = path.read_text(encoding="utf-8", errors="ignore")
                lines = content.split("\n")
                current_id = None
                current_data = {}
                for line in lines:
                    if line.startswith("# "):
                        if current_id and current_data:
                            entries[current_id] = current_data
                        current_id = line[2:].strip()
                        current_data = {"content": "", "last_accessed": "", "retained": False}
                    elif current_id:
                        if line.startswith("> last_accessed: "):
                            current_data["last_accessed"] = line[18:].strip()
                        elif line.startswith("> retained"):
                            current_data["retained"] = True
                        else:
                            current_data["content"] += line + "\n"
                if current_id and current_data:
                    entries[current_id] = current_data
                for entry_id, data in entries.items():
                    if data.get("retained"):
                        continue
                    last_acc = data.get("last_accessed", "")
                    if last_acc:
                        try:
                            acc_time = datetime.fromisoformat(last_acc)
                            if acc_time.tzinfo is None:
                                acc_time = acc_time.replace(tzinfo=timezone.utc)
                        except (ValueError, TypeError):
                            acc_time = datetime.now(timezone.utc)
                    else:
                        acc_time = datetime.now(timezone.utc)
                    if acc_time < cutoff:
                        try:
                            trash.move_to_trash(
                                data_id=f"wiki_{entry_id}",
                                data={"entry_id": entry_id, "content": data["content"]},
                                data_type="wiki",
                                reason=f"auto-archive: not accessed for {days_threshold} days"
                            )
                            archived += 1
                        except (ValueError, Exception) as e:
                            logger.warning("auto_archive_wiki: failed to archive '%s': %s", entry_id, e)
        except Exception as e:
            logger.warning("auto_archive_wiki: failed: %s", e)
        return archived

    async def restore_from_archive(self, entry_id: str) -> bool:
        try:
            from morn.sdk.trash_bin import TrashBin
            trash = TrashBin()
            result = trash.restore_from_trash(f"wiki_{entry_id}")
            return result is not None
        except Exception as e:
            logger.warning("restore_from_archive: failed for '%s': %s", entry_id, e)
            return False

    async def get_archived_entries(self) -> list:
        try:
            from morn.sdk.trash_bin import TrashBin
            trash = TrashBin()
            return trash.list_contents(data_type="wiki")
        except Exception as e:
            logger.warning("get_archived_entries: failed: %s", e)
            return []