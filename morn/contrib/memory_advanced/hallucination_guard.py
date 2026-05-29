import logging

logger = logging.getLogger("morn.memory")


class HallucinationGuard:
    def __init__(self, raw_snapshot_store=None):
        self._raw_store = raw_snapshot_store

    @staticmethod
    def _jaccard_similarity(a: str, b: str) -> float:
        if not a and not b:
            return 1.0
        set_a = set(a)
        set_b = set(b)
        intersection = set_a & set_b
        union = set_a | set_b
        if not union:
            return 1.0
        return len(intersection) / len(union)

    async def check_summary_fidelity(self, raw_snapshot_id: str, summary_text: str) -> dict:
        if self._raw_store is None:
            return {"passed": True, "similarity": 1.0, "action": "accept"}

        snapshot = await self._raw_store.get_snapshot(raw_snapshot_id)
        if snapshot is None:
            return {"passed": False, "similarity": 0.0, "action": "review", "error": "snapshot_not_found"}

        similarity = self._jaccard_similarity(snapshot.raw_text, summary_text)
        if similarity < 0.85:
            return {"passed": False, "similarity": round(similarity, 4), "action": "review"}
        return {"passed": True, "similarity": round(similarity, 4), "action": "accept"}

    def check_consistency(self, triple_store, subject: str, relation: str,
                          object_: str, timestamp: float) -> dict:
        conflicts = []

        semantic_graph = None
        temporal_graph = None

        if hasattr(triple_store, "get_subgraph"):
            semantic_graph = triple_store.get_subgraph(type(triple_store).SEMANTIC if hasattr(type(triple_store), "SEMANTIC") else None)
            temporal_graph = triple_store.get_subgraph(type(triple_store).TEMPORAL if hasattr(type(triple_store), "TEMPORAL") else None)

        if semantic_graph:
            semantic_result = getattr(semantic_graph, "query_semantic", None)
            if semantic_result:
                existing = semantic_result(subject)
                for record in existing:
                    if record.get("relation") != relation:
                        conflicts.append({
                            "type": "semantic_contradiction",
                            "subject": subject,
                            "existing_relation": record.get("relation"),
                            "new_relation": relation,
                            "existing_target": record.get("target"),
                            "new_target": object_,
                        })

        if temporal_graph:
            temporal_result = getattr(temporal_graph, "query_temporal", None)
            if temporal_result:
                existing = temporal_result(subject, start_time=timestamp - 3600, end_time=timestamp + 3600)
                for record in existing:
                    edges = record.get("edges", [])
                    for edge in edges:
                        if edge.get("relation_type") != relation and edge.get("relation_type"):
                            conflicts.append({
                                "type": "temporal_contradiction",
                                "subject": subject,
                                "existing_relation": edge.get("relation_type"),
                                "new_relation": relation,
                                "timestamp_range": [timestamp - 3600, timestamp + 3600],
                            })

        if conflicts:
            return {"consistent": False, "conflicts": conflicts}
        return {"consistent": True, "conflicts": []}

    def attach_snapshot_reference(self, result: dict, raw_snapshot_id: str) -> dict:
        result["raw_snapshot_id"] = raw_snapshot_id
        result["snapshot_preview"] = raw_snapshot_id[:8] + "..."
        return result