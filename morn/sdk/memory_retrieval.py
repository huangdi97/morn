import asyncio
import json
import logging
import time
from collections import defaultdict
from typing import Optional

logger = logging.getLogger("morn.retrieval")


class RetrievalEngine:
    DEFAULT_CHANNELS = {
        "time": True, "entity": True, "keyword": True,
        "semantic": True, "graph": True, "causal": True,
    }

    def __init__(self, memory_store, vector_store=None, channels: dict = None):
        self.store = memory_store
        self.vector_store = vector_store or memory_store.vector_store
        self.channels = channels if channels is not None else dict(self.DEFAULT_CHANNELS)

    @staticmethod
    def _resolve_entity(entity):
        return entity if isinstance(entity, str) else entity.get("name", "")

    def _get_aliases(self, entity):
        return entity.get("aliases", []) if isinstance(entity, dict) else []

    async def search(self, query=None, entities=None, timerange=None,
                     k: float = 60.0, min_score: float = 0.01,
                     emotion_state: dict = None) -> list[tuple]:
        tasks = []
        channel_names = []
        for name, arg, fn in [
            ("keyword", query, lambda: self._run_keyword(query)),
            ("entity", entities, lambda: self._run_entity(entities)),
            ("time", timerange, lambda: self._run_time(timerange)),
            ("semantic", query, lambda: self._run_semantic(query)),
            ("graph", entities, lambda: self._run_graph(entities)),
            ("causal", entities, lambda: self._run_causal(entities)),
        ]:
            if self.channels.get(name) and arg:
                tasks.append(fn())
                channel_names.append(name)

        if not tasks:
            return []
        results = await asyncio.gather(*tasks, return_exceptions=True)
        rank_lists = [res for res in results if not isinstance(res, Exception)]
        adjusted_k = k
        if emotion_state is not None:
            adjustment = self._apply_emotional_temperature(emotion_state)
            overlap_count = sum(1 for _ in channel_names)
            if overlap_count > 0:
                avg_factor = sum(adjustment.values()) / len(adjustment) if adjustment else 1.0
                adjusted_k = k * avg_factor
                adjusted_k = max(max(k * 0.9, k - 10), min(k * 1.1, adjusted_k))
        return self._rrf_fuse(rank_lists, adjusted_k, min_score)

    async def _run_keyword(self, query: str) -> list[str]:
        return [c["event_id"] for c in await self.store.search_fts(query)]

    async def _run_entity(self, entities: list) -> list[str]:
        seen, ids = set(), []
        for ent in entities:
            for alias in [self._resolve_entity(ent)] + self._get_aliases(ent):
                for c in await self.store.search_by_entity(alias):
                    if c["event_id"] not in seen:
                        seen.add(c["event_id"])
                        ids.append(c["event_id"])
        return ids

    async def _run_time(self, timerange: tuple) -> list[str]:
        caps = await self.store.search_by_timerange(*timerange)
        return [c["event_id"] for c in caps]

    async def _run_semantic(self, query: str, top_k: int = 10) -> list[str]:
        if not self.vector_store or not self.vector_store._available:
            return []
        event_ids = await self.vector_store.search_similar(query, top_k)
        return list(event_ids)

    async def _run_graph(self, seed_entities: list, hops: int = 2) -> list[str]:
        seen, results, seed_ids = set(), [], set()
        for ent in seed_entities:
            for c in await self.store.search_by_entity(self._resolve_entity(ent)):
                seed_ids.add(c["event_id"])

        current = set(seed_ids)
        for hop in range(hops):
            if not current:
                break
            next_set = set()
            for eid in current:
                if eid in seen:
                    continue
                seen.add(eid)
                results.append(eid)
                cap = await self.store.get_capsule(eid)
                if not cap:
                    continue
                try:
                    edges = json.loads(cap.get("causal_edges", "[]"))
                except (json.JSONDecodeError, TypeError):
                    edges = []
                for edge in edges:
                    if isinstance(edge, list) and len(edge) >= 1:
                        target = edge[0]
                        weight = edge[1] if len(edge) > 1 else 1.0
                        if weight < 0.3 and hop >= 1:
                            continue
                        if target not in seen:
                            next_set.add(target)
            current = next_set

        return results

    async def _run_causal(self, seed_entities: list,
                          direction: str = "both") -> list[str]:
        results, seen = [], set()
        for ent in seed_entities:
            for c in await self.store.search_by_entity(self._resolve_entity(ent)):
                if c["event_id"] not in seen:
                    seen.add(c["event_id"])
                    results.append(c["event_id"])
                    await self._causal_trace(c["event_id"], direction, seen, results)
        return results

    async def _causal_trace(self, event_id: str, direction: str,
                            seen: set, results: list):
        if direction not in ("forward", "backward", "both"):
            direction = "both"
        cap = await self.store.get_capsule(event_id)
        if not cap:
            return
        try:
            edges = json.loads(cap.get("causal_edges", "[]"))
        except (json.JSONDecodeError, TypeError):
            edges = []

        if direction in ("forward", "both"):
            for edge in edges:
                if isinstance(edge, list) and len(edge) >= 1:
                    target = edge[0]
                    if target not in seen:
                        seen.add(target)
                        results.append(target)

        if direction in ("backward", "both"):
            cursor = await self.store.db.execute(
                "SELECT event_id FROM capsules WHERE instr(causal_edges, ?) > 0",
                (event_id,))
            rows = await cursor.fetchall()
            for row in rows:
                eid = row["event_id"]
                if eid not in seen:
                    seen.add(eid)
                    results.append(eid)

    @staticmethod
    def _apply_emotional_temperature(emotion_state: dict = None) -> dict:
        if not emotion_state:
            return {}
        adjustment = {"keyword": 1.0, "entity": 1.0, "semantic": 1.0, "graph": 1.0, "causal": 1.0}
        calmness = emotion_state.get("calmness", 0.0)
        warmth = emotion_state.get("warmth", 0.0)
        ripple = emotion_state.get("ripple", 0.0)
        if calmness > 0.7:
            adjustment["keyword"] += 0.05
            adjustment["semantic"] += 0.05
        if warmth > 0.7:
            adjustment["entity"] += 0.05
            adjustment["graph"] += 0.05
        if ripple > 0.5:
            adjustment["causal"] += 0.08
        for k in adjustment:
            adjustment[k] = max(0.9, min(1.1, adjustment[k]))
        return adjustment

    def _rrf_fuse(self, rank_lists: list[list], k: float,
                  min_score: float) -> list[tuple]:
        scores = defaultdict(float)
        for ranked in rank_lists:
            for rank, eid in enumerate(ranked):
                scores[eid] += 1.0 / (k + rank)

        sorted_items = sorted(scores.items(), key=lambda x: -x[1])
        return [(eid, score) for eid, score in sorted_items if score >= min_score]

    async def search_by_timerange(self, start: str, end: str) -> list[dict]:
        return await self.store.search_by_timerange(start, end)

    async def search_by_entity(self, entity: str) -> list[dict]:
        return await self.store.search_by_entity(entity)

    async def search_fts(self, query: str) -> list[dict]:
        return await self.store.search_fts(query)

    async def search_semantic(self, query: str, top_k: int = 5) -> list[dict]:
        ids = await self._run_semantic(query, top_k)
        results = []
        for eid in ids:
            cap = await self.store.get_capsule(eid)
            if cap:
                results.append(cap)
        return results

    async def graph_diffusion(self, seed_entity_id: str, hops: int = 2) -> list[str]:
        cap = await self.store.get_capsule(seed_entity_id)
        if not cap:
            return []
        try:
            entities = json.loads(cap.get("entities", "[]"))
        except (json.JSONDecodeError, TypeError):
            entities = [seed_entity_id]
        return await self._run_graph(entities, hops)

    async def causal_trace(self, entity_id: str, direction: str = "both") -> list[str]:
        cap = await self.store.get_capsule(entity_id)
        if not cap:
            return []
        try:
            entities = json.loads(cap.get("entities", "[]"))
        except (json.JSONDecodeError, TypeError):
            entities = [entity_id]
        return await self._run_causal(entities, direction)


class LayeredRetrievalEngine(RetrievalEngine):
    FAST_CHANNELS = {"keyword": True, "entity": True}
    SLOW_CHANNELS = {"semantic": True, "graph": True, "causal": True}

    def __init__(self, memory_store, vector_store=None,
                 channels: dict = None, mode: str = "balanced",
                 default_timeout: float = 2.0):
        super().__init__(memory_store, vector_store, channels)
        self.mode = mode
        self.default_timeout = default_timeout
        self._slow_results_cache = {}
        self._pending_tasks = {}

    def set_mode(self, mode: str):
        if mode not in ("fast", "balanced", "deep"):
            raise ValueError(f"unknown mode: {mode}")
        self.mode = mode

    @staticmethod
    def _filter_channels(base, exclude):
        channels = dict(base)
        for ch in exclude:
            channels[ch] = False
        return channels

    async def quick_search(self, query=None, entities=None) -> list[tuple]:
        engine = RetrievalEngine(self.store, self.vector_store,
                                 self._filter_channels(self.DEFAULT_CHANNELS, self.SLOW_CHANNELS))
        return await engine.search(query=query, entities=entities)

    async def deep_search(self, query=None, entities=None,
                          timeout: float = None) -> dict:
        timeout = timeout if timeout is not None else self.default_timeout
        if self.mode == "fast":
            return {"quick_results": await self.quick_search(query, entities), "pending_task_id": None}

        quick = await self.quick_search(query, entities)
        if self.mode == "deep":
            return {"quick_results": quick, "pending_task_id": None,
                    "slow_results": await self.search(query=query, entities=entities)}

        task_id = f"slow_{int(time.time() * 1000)}_{id(query)}_{id(str(entities))}"
        task = asyncio.create_task(self._run_slow_channels(query, entities))
        self._pending_tasks[task_id] = task
        try:
            slow_results = await asyncio.wait_for(task, timeout=timeout)
            self._slow_results_cache[task_id] = slow_results
            return {"quick_results": quick, "pending_task_id": task_id, "slow_results": slow_results}
        except asyncio.TimeoutError:
            return {"quick_results": quick, "pending_task_id": task_id}

    async def _run_slow_channels(self, query, entities):
        engine = RetrievalEngine(self.store, self.vector_store,
                                 self._filter_channels(self.DEFAULT_CHANNELS, self.FAST_CHANNELS))
        return await engine.search(query=query, entities=entities)

    async def get_slow_results(self, task_id: str, timeout: float = 1.0):
        if task_id in self._slow_results_cache:
            return self._slow_results_cache[task_id]

        task = self._pending_tasks.get(task_id)
        if task is None:
            return None

        try:
            results = await asyncio.wait_for(task, timeout=timeout)
            self._slow_results_cache[task_id] = results
            return results
        except asyncio.TimeoutError:
            return None

    async def search(self, query=None, entities=None,
                     timerange=None, k: float = 60.0,
                     min_score: float = 0.01,
                     emotion_state: dict = None) -> list[tuple]:
        if self.mode == "fast":
            return await self.quick_search(query, entities)
        return await super().search(query, entities, timerange, k, min_score, emotion_state=emotion_state)


from morn.contrib.memory_advanced.raw_snapshot_store import RawSnapshot, RawSnapshotStore
