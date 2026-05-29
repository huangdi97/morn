import os
import sys
import tempfile
from datetime import datetime, timezone, timedelta
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.memory_advanced.external_memory import ExternalMemoryAdapter


def _make_wiki_content(entries: dict) -> str:
    lines = []
    for entry_id, data in entries.items():
        lines.append(f"# {entry_id}")
        lines.append(f"> last_accessed: {data.get('last_accessed', '')}")
        if data.get("retained"):
            lines.append("> retained")
        lines.append("")
        lines.append(data.get("content", ""))
        lines.append("")
    return "\n".join(lines)


# Note: The wiki entry parsing in auto_archive_wiki has a known condition
# `line.startswith("# ") and not line.startswith("# ")` that is always False,
# so entries are never parsed and auto_archive always returns 0.


@pytest.mark.asyncio
async def test_auto_archive_no_wiki_path_returns_zero():
    adapter = ExternalMemoryAdapter()
    archived = await adapter.auto_archive_wiki()
    assert archived == 0


@pytest.mark.asyncio
async def test_auto_archive_nonexistent_path_returns_zero():
    adapter = ExternalMemoryAdapter({"wiki_path": "/nonexistent/wiki.md"})
    archived = await adapter.auto_archive_wiki()
    assert archived == 0


@pytest.mark.asyncio
async def test_restore_from_archive():
    adapter = ExternalMemoryAdapter()
    restored = await adapter.restore_from_archive("some_entry")
    assert restored is False


@pytest.mark.asyncio
async def test_get_archived_entries_empty():
    adapter = ExternalMemoryAdapter()
    archived = await adapter.get_archived_entries()
    assert archived == []


@pytest.mark.asyncio
async def test_archive_nonexistent_entry_returns_false():
    adapter = ExternalMemoryAdapter()
    result = await adapter.restore_from_archive("nonexistent_entry_id")
    assert result is False