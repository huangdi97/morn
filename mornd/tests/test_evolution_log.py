import os
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.audit import EvolutionLogger


class TestEventLogging:
    def test_log_returns_event(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            event = logger.log("l1", "revise", {"template": "t1"})
            assert event["source"] == "l1"
            assert event["action"] == "revise"
            assert event["detail"]["template"] == "t1"

    def test_log_without_detail(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            event = logger.log("l0", "tune")
            assert event["source"] == "l0"
            assert event["action"] == "tune"
            assert event["detail"] == {}

    def test_log_appends_to_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log("l1", "revise")
            logger.log("l2", "optimize")
            log_path = os.path.join(tmp, "evolution", "evolution_log.jsonl")
            with open(log_path) as f:
                lines = f.readlines()
            assert len(lines) == 2


class TestGetLog:
    def test_get_log_returns_events(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log("l1", "revise")
            logger.log("l2", "optimize")
            log = logger.get_log()
            assert len(log) == 2

    def test_get_log_filter_by_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log("l1", "revise")
            logger.log("l2", "optimize")
            log = logger.get_log(source="l1")
            assert len(log) == 1
            assert log[0]["source"] == "l1"

    def test_get_log_respects_limit(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            for i in range(10):
                logger.log("l1", f"action_{i}")
            log = logger.get_log(limit=3)
            assert len(log) == 3

    def test_get_log_file_not_found(self):
        logger = EvolutionLogger()
        assert logger.get_log() == []


class TestStats:
    def test_get_stats_counts_by_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log("l1", "revise")
            logger.log("l1", "refine")
            logger.log("l2", "optimize")
            stats = logger.get_stats()
            assert stats["l1"] == 2
            assert stats["l2"] == 1

    def test_get_stats_empty(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            assert logger.get_stats() == {}

    def test_get_stats_no_file(self):
        logger = EvolutionLogger()
        assert logger.get_stats() == {}


class TestFileAutoCreate:
    def test_file_created_on_log(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            log_path = os.path.join(tmp, "evolution", "evolution_log.jsonl")
            assert not os.path.exists(log_path)
            logger.log("l1", "test")
            assert os.path.exists(log_path)

    def test_file_appended_multiple_times(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log("l1", "a")
            logger.log("l1", "b")
            logger.log("l1", "c")
            log = logger.get_log()
            assert len(log) == 3
