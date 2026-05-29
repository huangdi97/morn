import csv
import logging
from enum import Enum
from pathlib import Path
from typing import Any, Optional

logger = logging.getLogger("morn.memory")


class GraphMode(Enum):
    SEMANTIC = "semantic"
    TEMPORAL = "temporal"
    CAUSAL = "causal"
    AUTO = "auto"


class Node:
    def __init__(self, entity_id: str, entity_type: str, name: str,
                 aliases: Optional[list[str]] = None,
                 metadata: Optional[dict[str, Any]] = None):
        self.entity_id = entity_id
        self.entity_type = entity_type
        self.name = name
        self.aliases = aliases or []
        self.metadata = metadata or {}


class Edge:
    def __init__(self, source_id: str, target_id: str, relation_type: str,
                 weight: float = 1.0, confidence: float = 1.0):
        self.source_id = source_id
        self.target_id = target_id
        self.relation_type = relation_type
        self.weight = weight
        self.confidence = confidence


class GraphStore:
    def __init__(self, data_dir: Optional[Path] = None):
        self._nodes: dict[str, Node] = {}
        self._edges: list[Edge] = []
        self._data_dir = Path(data_dir) if data_dir else None
        self._mode = GraphMode.SEMANTIC
        self._semantic: Optional[SemanticGraph] = None
        self._temporal: Optional[TemporalGraph] = None
        self._causal: Optional[CausalGraph] = None
        if self._data_dir:
            self._load()

    def _subgraph_dir(self, name: str) -> Path:
        return self._data_dir / "graph" / name if self._data_dir else None

    def _nodes_path(self) -> Path:
        return self._data_dir / "graph" / "nodes.csv"

    def _edges_path(self) -> Path:
        return self._data_dir / "graph" / "edges.csv"

    def _ensure_subgraphs(self):
        if self._semantic is not None:
            return
        self._semantic = SemanticGraph(self._subgraph_dir("semantic"))
        self._temporal = TemporalGraph(self._subgraph_dir("temporal"))
        self._causal = CausalGraph(self._subgraph_dir("causal"))

    def _load(self):
        nodes_path = self._nodes_path()
        edges_path = self._edges_path()
        if nodes_path.exists():
            with nodes_path.open(newline="", encoding="utf-8") as f:
                reader = csv.DictReader(f)
                for row in reader:
                    aliases = row.get("aliases", "")
                    metadata = row.get("metadata", "")
                    import json
                    node = Node(
                        entity_id=row["entity_id"],
                        entity_type=row["entity_type"],
                        name=row["name"],
                        aliases=json.loads(aliases) if aliases else [],
                        metadata=json.loads(metadata) if metadata else {},
                    )
                    self._nodes[node.entity_id] = node
        if edges_path.exists():
            with edges_path.open(newline="", encoding="utf-8") as f:
                reader = csv.DictReader(f)
                for row in reader:
                    edge = Edge(
                        source_id=row["source_id"],
                        target_id=row["target_id"],
                        relation_type=row["relation_type"],
                        weight=float(row["weight"]),
                        confidence=float(row["confidence"]),
                    )
                    self._edges.append(edge)

    def _save(self):
        if not self._data_dir:
            return
        graph_dir = self._data_dir / "graph"
        graph_dir.mkdir(parents=True, exist_ok=True)
        import json
        nodes_path = graph_dir / "nodes.csv"
        with nodes_path.open("w", newline="", encoding="utf-8") as f:
            writer = csv.writer(f)
            writer.writerow(["entity_id", "entity_type", "name", "aliases", "metadata"])
            for node in self._nodes.values():
                writer.writerow([
                    node.entity_id, node.entity_type, node.name,
                    json.dumps(node.aliases, ensure_ascii=False),
                    json.dumps(node.metadata, ensure_ascii=False),
                ])
        edges_path = graph_dir / "edges.csv"
        with edges_path.open("w", newline="", encoding="utf-8") as f:
            writer = csv.writer(f)
            writer.writerow(["source_id", "target_id", "relation_type", "weight", "confidence"])
            for edge in self._edges:
                writer.writerow([
                    edge.source_id, edge.target_id,
                    edge.relation_type, edge.weight, edge.confidence,
                ])

    def add_node(self, entity_id: str, entity_type: str, name: str,
                 aliases: Optional[list[str]] = None,
                 metadata: Optional[dict[str, Any]] = None):
        node = Node(entity_id, entity_type, name, aliases, metadata)
        self._nodes[node.entity_id] = node
        self._save()

    def add_edge(self, source_id: str, target_id: str, relation_type: str,
                 weight: float = 1.0, confidence: float = 1.0):
        edge = Edge(source_id, target_id, relation_type, weight, confidence)
        self._edges.append(edge)
        self._save()

    def get_node(self, entity_id: str) -> Optional[dict[str, Any]]:
        node = self._nodes.get(entity_id)
        if not node:
            return None
        return {
            "entity_id": node.entity_id,
            "entity_type": node.entity_type,
            "name": node.name,
            "aliases": node.aliases,
            "metadata": node.metadata,
        }

    def get_edges(self, entity_id: str, direction: str = "both") -> list[dict[str, Any]]:
        results = []
        for edge in self._edges:
            if direction in ("both", "out") and edge.source_id == entity_id:
                results.append({
                    "source_id": edge.source_id,
                    "target_id": edge.target_id,
                    "relation_type": edge.relation_type,
                    "weight": edge.weight,
                    "confidence": edge.confidence,
                    "direction": "out",
                })
            if direction in ("both", "in") and edge.target_id == entity_id:
                results.append({
                    "source_id": edge.source_id,
                    "target_id": edge.target_id,
                    "relation_type": edge.relation_type,
                    "weight": edge.weight,
                    "confidence": edge.confidence,
                    "direction": "in",
                })
        return results

    def diffusion(self, entity_id: str, hops: int = 2,
                  min_weight: float = 0.3) -> list[dict[str, Any]]:
        from collections import deque
        visited: set[str] = set()
        activated: list[dict[str, Any]] = []
        queue: deque[tuple[str, int]] = deque()
        queue.append((entity_id, 0))
        visited.add(entity_id)

        while queue:
            current, depth = queue.popleft()
            node_data = self.get_node(current)
            if node_data:
                activated.append(node_data)

            if depth >= hops:
                continue

            for edge in self._edges:
                neighbor = None
                if edge.source_id == current:
                    neighbor = edge.target_id
                elif edge.target_id == current:
                    neighbor = edge.source_id
                if neighbor is None or neighbor in visited:
                    continue
                if edge.weight < min_weight:
                    continue
                effective_hops = hops
                if edge.weight < 0.7:
                    effective_hops = 1
                if depth + 1 <= effective_hops:
                    visited.add(neighbor)
                    queue.append((neighbor, depth + 1))

        return activated

    def search(self, query: str) -> list[dict[str, Any]]:
        query_lower = query.lower()
        results = []
        for node in self._nodes.values():
            if query_lower in node.name.lower():
                results.append(self.get_node(node.entity_id))
                continue
            for alias in node.aliases:
                if query_lower in alias.lower():
                    results.append(self.get_node(node.entity_id))
                    break
        return results

    def count_nodes(self) -> int:
        return len(self._nodes)

    def count_edges(self) -> int:
        return len(self._edges)

    def change_graph_mode(self, mode: GraphMode):
        self._mode = mode

    def query(self, query_text: str, mode: Optional[GraphMode] = None) -> list[dict[str, Any]]:
        effective_mode = mode or self._mode
        if effective_mode == GraphMode.AUTO:
            effective_mode = self._auto_route(query_text)
        if effective_mode == GraphMode.SEMANTIC:
            return self.search(query_text)
        elif effective_mode == GraphMode.TEMPORAL:
            self._ensure_subgraphs()
            return self._temporal.query_temporal(query_text)
        elif effective_mode == GraphMode.CAUSAL:
            self._ensure_subgraphs()
            return self._causal.query_causal(query_text)
        return []

    def _auto_route(self, query_text: str) -> GraphMode:
        q = query_text.lower()
        semantic_kw = ["喜欢", "不喜欢", "是什么", "属于", "包含", "类别"]
        temporal_kw = ["什么时候", "之前", "之后", "开始", "持续多久"]
        causal_kw = ["为什么", "导致", "所以", "因为", "结果"]
        for kw in causal_kw:
            if kw in q:
                return GraphMode.CAUSAL
        for kw in temporal_kw:
            if kw in q:
                return GraphMode.TEMPORAL
        for kw in semantic_kw:
            if kw in q:
                return GraphMode.SEMANTIC
        return GraphMode.SEMANTIC

    def add_to_subgraph(self, entity_id: str, relation: str, target_id: str,
                        graph_type: GraphMode, **attrs):
        weight = attrs.get("weight", 1.0)
        confidence = attrs.get("confidence", 1.0)
        if graph_type == GraphMode.SEMANTIC:
            self.add_edge(entity_id, target_id, relation, weight, confidence)
        else:
            self._ensure_subgraphs()
            sub = self._temporal if graph_type == GraphMode.TEMPORAL else self._causal
            sub.add_edge(entity_id, target_id, relation, weight, confidence)

    def get_subgraph(self, graph_type: GraphMode):
        if graph_type == GraphMode.SEMANTIC:
            return self
        self._ensure_subgraphs()
        if graph_type == GraphMode.TEMPORAL:
            return self._temporal
        elif graph_type == GraphMode.CAUSAL:
            return self._causal
        return None

    def query_semantic(self, entity: str, relation: Optional[str] = None,
                       depth: int = 1) -> list[dict[str, Any]]:
        return SemanticGraph.query_semantic(self, entity, relation, depth)

    def query_temporal(self, entity: str, start_time: Optional[float] = None,
                       end_time: Optional[float] = None) -> list[dict[str, Any]]:
        self._ensure_subgraphs()
        return self._temporal.query_temporal(entity, start_time, end_time)

    def query_causal(self, entity: str, direction: str = "forward",
                     depth: int = 3) -> list[dict[str, Any]]:
        self._ensure_subgraphs()
        return self._causal.query_causal(entity, direction, depth)


class SemanticGraph(GraphStore):
    def __init__(self, data_dir: Optional[Path] = None):
        super().__init__(data_dir)

    def query_semantic(self, entity: str, relation: Optional[str] = None,
                       depth: int = 1) -> list[dict[str, Any]]:
        nodes = self.search(entity)
        if not nodes:
            return []
        results = []
        for node in nodes:
            eid = node["entity_id"]
            edges = self.get_edges(eid)
            for edge in edges:
                if relation and edge["relation_type"] != relation:
                    continue
                target = self.get_node(edge["target_id"])
                results.append({
                    "source": node,
                    "relation": edge["relation_type"],
                    "target": target,
                    "weight": edge["weight"],
                })
        if depth > 1:
            seen = {n["entity_id"] for n in nodes}
            current = nodes
            for _ in range(depth - 1):
                next_entities = []
                for n in current:
                    for edge in self.get_edges(n["entity_id"]):
                        tid = edge["target_id"]
                        if tid not in seen:
                            seen.add(tid)
                            t = self.get_node(tid)
                            if t:
                                next_entities.append(t)
                                results.append({
                                    "source": n,
                                    "relation": edge["relation_type"],
                                    "target": t,
                                    "weight": edge["weight"],
                                })
                current = next_entities
                if not current:
                    break
        return results


class TemporalGraph(GraphStore):
    def __init__(self, data_dir: Optional[Path] = None):
        super().__init__(data_dir)

    def add_event(self, entity_id: str, timestamp: float,
                  event_type: str = "event", **attrs):
        self.add_node(entity_id, "event", event_type,
                      metadata={"timestamp": timestamp, **attrs})

    def query_temporal(self, entity: str, start_time: Optional[float] = None,
                       end_time: Optional[float] = None) -> list[dict[str, Any]]:
        nodes = self.search(entity)
        if not nodes:
            return []
        results = []
        for node in nodes:
            ts = node.get("metadata", {}).get("timestamp")
            if start_time is not None and (ts is None or ts < start_time):
                continue
            if end_time is not None and (ts is None or ts > end_time):
                continue
            edges = self.get_edges(node["entity_id"])
            timeline_entry = {
                "entity": node,
                "edges": edges,
            }
            results.append(timeline_entry)
        results.sort(key=lambda r: r["entity"].get("metadata", {}).get("timestamp", 0))
        return results


class CausalGraph(GraphStore):
    def __init__(self, data_dir: Optional[Path] = None):
        super().__init__(data_dir)

    def query_causal(self, entity: str, direction: str = "forward",
                     depth: int = 3) -> list[dict[str, Any]]:
        nodes = self.search(entity)
        if not nodes:
            return []
        results = []
        visited = set()
        from collections import deque
        queue = deque()
        for n in nodes:
            queue.append((n["entity_id"], 0))
            visited.add(n["entity_id"])
        while queue:
            eid, d = queue.popleft()
            if d >= depth:
                continue
            edges = self.get_edges(eid)
            for edge in edges:
                if direction == "forward" and edge["direction"] != "out":
                    continue
                if direction == "backward" and edge["direction"] != "in":
                    continue
                neighbor_id = edge["target_id"] if edge["direction"] == "out" else edge["source_id"]
                if neighbor_id not in visited:
                    visited.add(neighbor_id)
                    neighbor = self.get_node(neighbor_id)
                    if neighbor:
                        results.append({
                            "source": self.get_node(eid),
                            "relation": edge["relation_type"],
                            "target": neighbor,
                            "depth": d + 1,
                        })
                        queue.append((neighbor_id, d + 1))
        return results
