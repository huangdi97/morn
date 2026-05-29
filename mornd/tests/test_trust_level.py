import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore, DEFAULT_TRUST_LEVEL


@pytest.fixture
def data_dir():
    with tempfile.TemporaryDirectory(prefix="morn_trust_") as tmpdir:
        yield Path(tmpdir)


@pytest.mark.asyncio
async def test_default_trust_level(data_dir):
    async with MemoryStore(data_dir) as store:
        assert store.default_trust_level == DEFAULT_TRUST_LEVEL


@pytest.mark.asyncio
async def test_custom_default_trust_level(data_dir):
    async with MemoryStore(data_dir, default_trust_level=0) as store:
        assert store.default_trust_level == 0


@pytest.mark.asyncio
async def test_add_capsule_default_trust_is_htz(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "test default trust",
        })
        cap = await store.get_capsule(eid)
        assert cap["trust_level"] == "htz"


@pytest.mark.asyncio
async def test_add_capsule_with_int_trust_level(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "test int trust level",
            "trust_level": 0,
        })
        cap = await store.get_capsule(eid)
        assert cap["trust_level"] == "ltz"


@pytest.mark.asyncio
async def test_htz_can_be_deposited_to_l4(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "htz capsule",
            "trust_level": 2,
        })
        cap = await store.get_capsule(eid)
        assert cap["trust_level"] == "htz"
        int_level = store._trust_to_int(cap["trust_level"])
        assert int_level == 2


@pytest.mark.asyncio
async def test_mtz_can_be_deposited_to_l4(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "mtz capsule",
            "trust_level": 1,
        })
        cap = await store.get_capsule(eid)
        assert cap["trust_level"] == "mtz"
        int_level = store._trust_to_int(cap["trust_level"])
        assert int_level == 1


@pytest.mark.asyncio
async def test_ltz_cannot_be_deposited_to_l4(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "ltz capsule",
            "trust_level": 0,
        })
        cap = await store.get_capsule(eid)
        assert cap["trust_level"] == "ltz"
        int_level = store._trust_to_int(cap["trust_level"])
        assert int_level == 0
        assert int_level < 1


@pytest.mark.asyncio
async def test_set_trust_level(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": "[]",
            "description": "adjustable trust",
            "trust_level": 2,
        })
        assert await store.set_trust_level(eid, 0) is True
        cap = await store.get_capsule(eid)
        assert cap["trust_level"] == "ltz"


@pytest.mark.asyncio
async def test_set_trust_level_invalid(data_dir):
    async with MemoryStore(data_dir) as store:
        with pytest.raises(ValueError):
            await store.set_trust_level("evt_xxx", 999)


@pytest.mark.asyncio
async def test_set_trust_level_nonexistent(data_dir):
    async with MemoryStore(data_dir) as store:
        assert await store.set_trust_level("evt_nonexistent", 0) is False


@pytest.mark.asyncio
async def test_search_capsules_min_trust_filter(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({"entities": "[]", "description": "htz", "trust_level": 2})
        await store.add_capsule({"entities": "[]", "description": "mtz", "trust_level": 1})
        await store.add_capsule({"entities": "[]", "description": "ltz", "trust_level": 0})

        all_caps = await store.search_capsules(min_trust_level=0)
        assert len(all_caps) == 3

        htz_mtz = await store.search_capsules(min_trust_level=1)
        assert len(htz_mtz) == 2
        for c in htz_mtz:
            assert c["trust_level"] in ("htz", "mtz")

        htz_only = await store.search_capsules(min_trust_level=2)
        assert len(htz_only) == 1
        assert htz_only[0]["trust_level"] == "htz"
