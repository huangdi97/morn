import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.memory_advanced.external_memory import (
    ExternalMemoryAdapter,
    VoidAdapter,
    Mem0Adapter,
    LettaAdapter,
    ZepAdapter,
)


class TestVoidAdapter:
    def test_void_adapter_default_noop(self):
        adapter = ExternalMemoryAdapter()
        assert adapter.adapter_type == "void"
        assert adapter.store_memory({"test": "data"}) is True
        assert adapter.retrieve("test") == []
        assert adapter.connect() is True
        assert adapter.disconnect() is True

    def test_void_adapter_explicit(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "void"})
        assert adapter.adapter_type == "void"


class TestMem0Adapter:
    def test_mem0_store(self):
        adapter = ExternalMemoryAdapter({
            "adapter_type": "mem0",
            "mem0_config": {"api_key": "test-key", "endpoint": "http://localhost"},
        })
        adapter.connect()
        assert adapter.store_memory({"text": "hello"}) is True

    def test_mem0_retrieve(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "mem0"})
        adapter.connect()
        assert adapter.retrieve("test") == []

    def test_mem0_store_without_connect(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "mem0"})
        assert adapter.store_memory({"text": "hello"}) is False


class TestLettaAdapter:
    def test_letta_store(self):
        adapter = ExternalMemoryAdapter({
            "adapter_type": "letta",
            "letta_config": {"api_key": "test-key", "endpoint": "http://localhost"},
        })
        adapter.connect()
        assert adapter.store_memory({"text": "hello"}) is True

    def test_letta_retrieve(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "letta"})
        adapter.connect()
        assert adapter.retrieve("test") == []

    def test_letta_store_without_connect(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "letta"})
        assert adapter.store_memory({"text": "hello"}) is False


class TestZepAdapter:
    def test_zep_store(self):
        adapter = ExternalMemoryAdapter({
            "adapter_type": "zep",
            "zep_config": {"api_key": "test-key", "endpoint": "http://localhost"},
        })
        adapter.connect()
        assert adapter.store_memory({"text": "hello"}) is True

    def test_zep_retrieve(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "zep"})
        adapter.connect()
        assert adapter.retrieve("test") == []

    def test_zep_store_without_connect(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "zep"})
        assert adapter.store_memory({"text": "hello"}) is False


class TestSwitchAdapter:
    def test_switch_adapter(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "void"})
        assert adapter.adapter_type == "void"
        adapter.set_adapter("mem0")
        assert adapter.adapter_type == "mem0"
        assert isinstance(adapter._adapter, Mem0Adapter)

    def test_set_adapter_validates(self):
        adapter = ExternalMemoryAdapter()
        with pytest.raises(ValueError, match="unknown adapter type"):
            adapter.set_adapter("nonexistent")

    def test_unknown_adapter_type_falls_back(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "unknown"})
        assert adapter.adapter_type == "void"
        assert isinstance(adapter._adapter, VoidAdapter)


class TestWikiSearch:
    def test_wiki_search_no_path_returns_empty(self):
        adapter = ExternalMemoryAdapter()
        assert adapter.wiki_search("test") == []

    def test_wiki_search_path_not_exists(self):
        adapter = ExternalMemoryAdapter({"wiki_path": "/nonexistent/path"})
        assert adapter.wiki_search("test") == []

    def test_wiki_search_finds_content(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            wiki_file = Path(tmpdir) / "wiki.md"
            wiki_file.write_text("Morn 数字生命框架核心知识")
            adapter = ExternalMemoryAdapter({"wiki_path": str(wiki_file)})
            results = adapter.wiki_search("数字生命")
            assert len(results) == 1
            assert "数字生命" in results[0]["content"]


class TestObsidianSearch:
    def test_obsidian_search_no_path_returns_empty(self):
        adapter = ExternalMemoryAdapter()
        assert adapter.obsidian_search("test") == []

    def test_obsidian_search_path_not_exists(self):
        adapter = ExternalMemoryAdapter({"obsidian_vault_path": "/nonexistent/vault"})
        assert adapter.obsidian_search("test") == []

    def test_obsidian_search_finds_content(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            vault = Path(tmpdir) / "vault"
            vault.mkdir()
            note = vault / "note.md"
            note.write_text("Obsidian vault note content about Morn")
            adapter = ExternalMemoryAdapter({"obsidian_vault_path": str(vault)})
            results = adapter.obsidian_search("Morn")
            assert len(results) == 1
            assert "Morn" in results[0]["content"]

    def test_obsidian_search_no_match(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            vault = Path(tmpdir) / "vault"
            vault.mkdir()
            note = vault / "note.md"
            note.write_text("Some other content")
            adapter = ExternalMemoryAdapter({"obsidian_vault_path": str(vault)})
            results = adapter.obsidian_search("Morn")
            assert results == []


class TestConnectDisconnect:
    def test_connect_disconnect_lifecycle(self):
        adapter = ExternalMemoryAdapter({"adapter_type": "mem0"})
        assert adapter.connect() is True
        assert adapter.disconnect() is True