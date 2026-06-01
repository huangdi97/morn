import os
import sys
import tempfile
from pathlib import Path


sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.memory_advanced.graph_store import GraphStore


class TestGraphStore:
    def test_add_node(self):
        store = GraphStore()
        store.add_node("e1", "person", "Alice", aliases=["A", "Ali"])
        node = store.get_node("e1")
        assert node is not None
        assert node["entity_id"] == "e1"
        assert node["name"] == "Alice"
        assert node["aliases"] == ["A", "Ali"]

    def test_add_edge(self):
        store = GraphStore()
        store.add_node("e1", "person", "Alice")
        store.add_node("e2", "person", "Bob")
        store.add_edge("e1", "e2", "knows", weight=0.8, confidence=0.9)
        edges = store.get_edges("e1")
        assert len(edges) == 1
        assert edges[0]["relation_type"] == "knows"
        assert edges[0]["weight"] == 0.8

    def test_get_node_nonexistent(self):
        store = GraphStore()
        assert store.get_node("nonexistent") is None

    def test_get_edges_in_out_both(self):
        store = GraphStore()
        store.add_node("a", "t", "A")
        store.add_node("b", "t", "B")
        store.add_node("c", "t", "C")
        store.add_edge("a", "b", "rel1")
        store.add_edge("c", "a", "rel2")
        out_edges = store.get_edges("a", "out")
        assert len(out_edges) == 1
        assert out_edges[0]["target_id"] == "b"
        in_edges = store.get_edges("a", "in")
        assert len(in_edges) == 1
        assert in_edges[0]["source_id"] == "c"
        both_edges = store.get_edges("a", "both")
        assert len(both_edges) == 2

    def test_diffusion_one_hop(self):
        store = GraphStore()
        store.add_node("a", "t", "A")
        store.add_node("b", "t", "B")
        store.add_edge("a", "b", "rel", weight=0.9)
        result = store.diffusion("a", hops=1, min_weight=0.3)
        names = {r["name"] for r in result}
        assert "A" in names
        assert "B" in names

    def test_diffusion_two_hops(self):
        store = GraphStore()
        store.add_node("a", "t", "A")
        store.add_node("b", "t", "B")
        store.add_node("c", "t", "C")
        store.add_edge("a", "b", "rel", weight=0.9)
        store.add_edge("b", "c", "rel", weight=0.9)
        result = store.diffusion("a", hops=2, min_weight=0.3)
        names = {r["name"] for r in result}
        assert "A" in names
        assert "B" in names
        assert "C" in names

    def test_diffusion_weight_filter(self):
        store = GraphStore()
        store.add_node("a", "t", "A")
        store.add_node("b", "t", "B")
        store.add_node("c", "t", "C")
        store.add_edge("a", "b", "rel", weight=0.9)
        store.add_edge("a", "c", "rel", weight=0.1)
        result = store.diffusion("a", hops=1, min_weight=0.5)
        names = {r["name"] for r in result}
        assert "A" in names
        assert "B" in names
        assert "C" not in names

    def test_search_by_name(self):
        store = GraphStore()
        store.add_node("e1", "person", "Alice")
        store.add_node("e2", "person", "Bob")
        results = store.search("Alice")
        assert len(results) == 1
        assert results[0]["entity_id"] == "e1"

    def test_search_by_alias(self):
        store = GraphStore()
        store.add_node("e1", "person", "Alice", aliases=["Ali", "小爱"])
        results = store.search("小爱")
        assert len(results) == 1
        assert results[0]["entity_id"] == "e1"

    def test_count_nodes_and_edges(self):
        store = GraphStore()
        assert store.count_nodes() == 0
        assert store.count_edges() == 0
        store.add_node("a", "t", "A")
        store.add_node("b", "t", "B")
        assert store.count_nodes() == 2
        store.add_edge("a", "b", "rel")
        assert store.count_edges() == 1

    def test_persistence_csv(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            store = GraphStore(Path(tmpdir))
            store.add_node("e1", "person", "Alice", aliases=["Ali"])
            store.add_edge("e1", "e2", "knows", weight=0.8)

            store2 = GraphStore(Path(tmpdir))
            assert store2.count_nodes() == 1
            node = store2.get_node("e1")
            assert node is not None
            assert node["name"] == "Alice"
            assert node["aliases"] == ["Ali"]
            assert store2.count_edges() == 1