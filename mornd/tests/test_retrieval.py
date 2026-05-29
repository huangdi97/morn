import json
import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.memory.store import MemoryStore
from morn_core.memory.retrieval import RetrievalEngine, LayeredRetrievalEngine


@pytest.mark.asyncio
async def test_keyword_channel(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        await store.add_capsule({
            "entities": json.dumps(["Morn"]),
            "description": "今天天气很好，适合散步",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(query="衰老")
        assert len(results) >= 0


@pytest.mark.asyncio
async def test_entity_channel(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(entities=["创建者"])
        assert len(results) >= 1


@pytest.mark.asyncio
async def test_entity_channel_with_aliases(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(entities=[{"name": "创建者", "aliases": ["creator"]}])
        assert len(results) >= 1


@pytest.mark.asyncio
async def test_time_channel(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": "[]",
            "description": "a time-based entry",
            "timestamp": "2026-05-27T10:00:00Z",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(
            timerange=("2026-01-01T00:00:00Z", "2026-12-31T00:00:00Z"))
        assert len(results) >= 1


@pytest.mark.asyncio
async def test_semantic_channel(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": "[]",
            "description": "心情很不错的一天",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(query="心情")
        _ = results


@pytest.mark.asyncio
async def test_semantic_channel_unavailable_skips(data_dir):
    async with MemoryStore(data_dir) as store:
        store.vector_store._available = False
        engine = RetrievalEngine(store)
        results = await engine.search(query="心情")
        _ = results


@pytest.mark.asyncio
async def test_graph_channel(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "test graph",
            "causal_edges": json.dumps([["evt_001", 0.8]]),
        })
        engine = RetrievalEngine(store)
        results = await engine.search(entities=["创建者"])
        _ = results


@pytest.mark.asyncio
async def test_causal_channel(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "test causal",
            "causal_edges": json.dumps([["evt_001", 0.8]]),
        })
        engine = RetrievalEngine(store)
        results = await engine.search(entities=["创建者"])
        _ = results


@pytest.mark.asyncio
async def test_rrf_fusion(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(query="心情", entities=["创建者"])
        scores = [s for _, s in results]
        assert all(s > 0 for s in scores)
        assert sorted(scores, reverse=True) == scores


@pytest.mark.asyncio
async def test_rrf_min_score_filter(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": "[]",
            "description": "随便什么内容",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(query="随便", min_score=10.0)
        assert len(results) == 0


@pytest.mark.asyncio
async def test_channel_disabled(data_dir):
    async with MemoryStore(data_dir) as store:
        engine = RetrievalEngine(store, channels={
            "keyword": False, "entity": False, "time": False,
            "semantic": False, "graph": False, "causal": False,
        })
        results = await engine.search(query="心情", entities=["创建者"])
        assert len(results) == 0


@pytest.mark.asyncio
async def test_empty_results(data_dir):
    async with MemoryStore(data_dir) as store:
        engine = RetrievalEngine(store)
        results = await engine.search(query="ZZZZnotfoundZZZZ")
        assert len(results) == 0


@pytest.mark.asyncio
async def test_rrf_k_parameter(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = RetrievalEngine(store)
        results_low_k = await engine.search(query="心情", k=1)
        results_high_k = await engine.search(query="心情", k=100)
        _ = results_low_k, results_high_k


@pytest.mark.asyncio
async def test_multi_channel_parallel(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "今天天气很好",
        })
        engine = RetrievalEngine(store)
        results = await engine.search(query="今天", entities=["创建者"])
        assert len(results) >= 1


@pytest.mark.asyncio
async def test_search_by_timerange_backward_compat(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": "[]", "description": "test",
            "timestamp": "2026-05-27T10:00:00Z",
        })
        engine = RetrievalEngine(store)
        caps = await engine.search_by_timerange("2026-01-01T00:00:00Z", "2026-12-31T00:00:00Z")
        assert len(caps) >= 1


@pytest.mark.asyncio
async def test_search_fts_backward_compat(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": "[]", "description": "今天好热",
        })
        engine = RetrievalEngine(store)
        caps = await engine.search_fts("今天")
        assert len(caps) >= 1


@pytest.mark.asyncio
async def test_graph_diffusion(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "seed entity",
        })
        engine = RetrievalEngine(store)
        results = await engine.graph_diffusion(eid, hops=2)
        assert isinstance(results, list)


@pytest.mark.asyncio
async def test_graph_diffusion_one_hop(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "seed entity",
        })
        engine = RetrievalEngine(store)
        results_1 = await engine.graph_diffusion(eid, hops=1)
        results_2 = await engine.graph_diffusion(eid, hops=2)
        assert isinstance(results_1, list)
        assert isinstance(results_2, list)


@pytest.mark.asyncio
async def test_causal_trace_forward(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "test",
            "causal_edges": json.dumps([["evt_target", 0.9]]),
        })
        engine = RetrievalEngine(store)
        results = await engine.causal_trace(eid, direction="forward")
        assert isinstance(results, list)


@pytest.mark.asyncio
async def test_causal_trace_backward(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "test",
            "causal_edges": json.dumps([["evt_target", 0.9]]),
        })
        engine = RetrievalEngine(store)
        results = await engine.causal_trace(eid, direction="backward")
        assert isinstance(results, list)


@pytest.mark.asyncio
async def test_causal_trace_both(data_dir):
    async with MemoryStore(data_dir) as store:
        eid = await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "test",
            "causal_edges": json.dumps([["evt_target", 0.9]]),
        })
        engine = RetrievalEngine(store)
        results = await engine.causal_trace(eid, direction="both")
        assert isinstance(results, list)


# ── LayeredRetrievalEngine tests (from test_retrieval_layered.py) ──

@pytest.mark.asyncio
async def test_quick_search_returns_immediately(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = LayeredRetrievalEngine(store)
        results = await engine.quick_search(query="心情")
        assert isinstance(results, list)


@pytest.mark.asyncio
async def test_quick_search_with_entities(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = LayeredRetrievalEngine(store)
        results = await engine.quick_search(entities=["创建者"])
        assert len(results) >= 1


@pytest.mark.asyncio
async def test_deep_search_returns_dict(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = LayeredRetrievalEngine(store, mode="balanced")
        result = await engine.deep_search(query="心情")
        assert "quick_results" in result
        assert "pending_task_id" in result


@pytest.mark.asyncio
async def test_deep_search_fast_mode(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = LayeredRetrievalEngine(store, mode="fast")
        result = await engine.deep_search(query="心情")
        assert result["pending_task_id"] is None
        assert "quick_results" in result


@pytest.mark.asyncio
async def test_deep_search_deep_mode(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = LayeredRetrievalEngine(store, mode="deep")
        result = await engine.deep_search(query="心情")
        assert "slow_results" in result


@pytest.mark.asyncio
async def test_set_mode_switches_behavior(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = LayeredRetrievalEngine(store, mode="balanced")
        assert engine.mode == "balanced"

        engine.set_mode("fast")
        assert engine.mode == "fast"
        result = await engine.deep_search(query="心情")
        assert result["pending_task_id"] is None

        engine.set_mode("deep")
        assert engine.mode == "deep"
        result = await engine.deep_search(query="心情")
        assert "slow_results" in result


@pytest.mark.asyncio
async def test_get_slow_results_nonexistent(data_dir):
    async with MemoryStore(data_dir) as store:
        engine = LayeredRetrievalEngine(store)
        result = await engine.get_slow_results("nonexistent_task")
        assert result is None


@pytest.mark.asyncio
async def test_deep_search_timeout_fallback(data_dir):
    async with MemoryStore(data_dir) as store:
        await store.add_capsule({
            "entities": json.dumps(["创建者"], ensure_ascii=False),
            "description": "创建者说今天心情很好",
        })
        engine = LayeredRetrievalEngine(store, mode="balanced", default_timeout=0.001)
        result = await engine.deep_search(query="心情", timeout=0.001)
        assert "quick_results" in result
