import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.memory_advanced.graph_store import (
    GraphStore, SemanticGraph, TemporalGraph, CausalGraph, GraphMode,
)


class TestSemanticQuery:
    def test_semantic_query(self):
        store = GraphStore()
        store.add_node("coffee", "beverage", "咖啡")
        store.add_node("creator", "person", "创建者")
        store.add_node("drink", "category", "饮品")
        store.add_edge("creator", "coffee", "喜欢")
        store.add_edge("coffee", "drink", "属于")
        sub = store.get_subgraph(GraphMode.SEMANTIC)
        results = sub.query_semantic("创建者")
        assert len(results) >= 1
        relations = {r["relation"] for r in results}
        assert "喜欢" in relations


class TestTemporalQuery:
    def test_temporal_query(self):
        store = GraphStore()
        sub = store.get_subgraph(GraphMode.TEMPORAL)
        sub.add_node("e1", "event", "创建者做决定",
                     metadata={"timestamp": 100.0})
        sub.add_node("e2", "event", "创建者反思",
                     metadata={"timestamp": 200.0})
        sub.add_edge("e1", "e2", "在...之后")
        results = sub.query_temporal("创建者")
        assert len(results) >= 2


class TestCausalQuery:
    def test_causal_query(self):
        store = GraphStore()
        sub = store.get_subgraph(GraphMode.CAUSAL)
        sub.add_node("late", "event", "熬夜")
        sub.add_node("bad_state", "event", "状态差")
        sub.add_node("remind", "event", "主动提醒休息")
        sub.add_edge("late", "bad_state", "导致")
        sub.add_edge("bad_state", "remind", "导致")
        results = sub.query_causal("熬夜", direction="forward", depth=3)
        assert len(results) >= 1


class TestAutoRoute:
    def test_auto_route_semantic(self):
        store = GraphStore()
        store.add_node("creator", "person", "创建者")
        store.add_node("coffee", "beverage", "咖啡")
        store.add_edge("creator", "coffee", "喜欢")
        store.change_graph_mode(GraphMode.AUTO)
        results = store.query("创建者")
        assert len(results) >= 1

    def test_auto_route_temporal(self):
        store = GraphStore()
        sub = store.get_subgraph(GraphMode.TEMPORAL)
        sub.add_node("e1", "event", "事件A", metadata={"timestamp": 1.0})
        store.change_graph_mode(GraphMode.AUTO)
        results = store.query("什么时候发生")
        assert isinstance(results, list)

    def test_auto_route_causal(self):
        store = GraphStore()
        sub = store.get_subgraph(GraphMode.CAUSAL)
        sub.add_node("cause", "event", "原因")
        sub.add_node("effect", "event", "结果")
        sub.add_edge("cause", "effect", "导致")
        store.change_graph_mode(GraphMode.AUTO)
        results = store.query("为什么会这样")
        assert isinstance(results, list)


class TestCrossGraphEntity:
    def test_cross_graph_entity(self):
        store = GraphStore()
        store.add_node("e1", "test", "公共实体")
        store.add_to_subgraph("e1", "发生在", "e2", GraphMode.TEMPORAL)
        store.add_to_subgraph("e1", "导致", "e3", GraphMode.CAUSAL)
        sem_results = store.get_subgraph(GraphMode.SEMANTIC).get_node("e1")
        tmp_edges = store.get_subgraph(GraphMode.TEMPORAL).get_edges("e1")
        cau_edges = store.get_subgraph(GraphMode.CAUSAL).get_edges("e1")
        assert sem_results is not None
        assert len(tmp_edges) == 1
        assert len(cau_edges) == 1


class TestExistingApiCompatible:
    def test_existing_api_compatible(self):
        store = GraphStore()
        store.add_node("e1", "person", "Alice")
        store.add_node("e2", "person", "Bob")
        store.add_edge("e1", "e2", "knows", weight=0.8, confidence=0.9)
        node = store.get_node("e1")
        assert node is not None
        assert node["name"] == "Alice"
        edges = store.get_edges("e1")
        assert len(edges) == 1
        assert edges[0]["relation_type"] == "knows"
        results = store.search("Alice")
        assert len(results) == 1
        assert store.count_nodes() == 2
        assert store.count_edges() == 1

    def test_mode_switch(self):
        store = GraphStore()
        store.change_graph_mode(GraphMode.TEMPORAL)
        assert store._mode == GraphMode.TEMPORAL
        store.change_graph_mode(GraphMode.AUTO)
        assert store._mode == GraphMode.AUTO

    def test_add_to_subgraph(self):
        store = GraphStore()
        store.add_to_subgraph("e1", "导致", "e2", GraphMode.CAUSAL)
        sub = store.get_subgraph(GraphMode.CAUSAL)
        edges = sub.get_edges("e1")
        assert len(edges) == 1
        assert edges[0]["relation_type"] == "导致"