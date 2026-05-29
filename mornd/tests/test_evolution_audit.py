import os
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.evolution.audit import EvolutionAuditor
from morn_core.evolution.audit import EvolutionLogger


class TestRecordChangeAndCount:
    def test_record_change_and_count(self):
        auditor = EvolutionAuditor()
        auditor.record_change("fast", {"component": "skill_a", "success": True})
        assert auditor._fast_count == 1
        assert auditor._slow_count == 0

    def test_record_change_multiple_fast(self):
        auditor = EvolutionAuditor()
        for _ in range(5):
            auditor.record_change("fast", {"component": "skill_a", "success": True})
        assert auditor._fast_count == 5

    def test_record_change_slow(self):
        auditor = EvolutionAuditor()
        auditor.record_change("slow", {"component": "skill_b", "success": False})
        assert auditor._slow_count == 1

    def test_record_change_disabled(self):
        auditor = EvolutionAuditor(config={"audit_enabled": False})
        auditor.record_change("fast", {"component": "skill_a", "success": True})
        assert auditor._fast_count == 0


class TestShouldGenerate:
    def test_should_generate_after_10_fast(self):
        auditor = EvolutionAuditor()
        for _ in range(9):
            auditor.record_change("fast", {"component": "x", "success": True})
        assert not auditor.should_generate_report()
        auditor.record_change("fast", {"component": "x", "success": True})
        assert auditor.should_generate_report()

    def test_should_generate_after_3_slow(self):
        auditor = EvolutionAuditor()
        for _ in range(2):
            auditor.record_change("slow", {"component": "x", "success": True})
        assert not auditor.should_generate_report()
        auditor.record_change("slow", {"component": "x", "success": True})
        assert auditor.should_generate_report()


class TestReportStructure:
    def test_report_structure(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(10):
                auditor.record_change("fast", {"component": "c1", "success": True})
            report = auditor.generate_report()
            assert "report_id" in report
            assert "generated_at" in report
            assert "period_summary" in report
            assert "effective_patterns" in report
            assert "ineffective_patterns" in report
            assert "recommendations" in report
            ps = report["period_summary"]
            assert "total_fast_changes" in ps
            assert "total_slow_changes" in ps
            assert "effective_changes" in ps
            assert "ineffective_changes" in ps
            assert "effectiveness_rate" in ps


class TestReportEffectivenessRate:
    def test_report_effectiveness_rate(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(8):
                auditor.record_change("fast", {"component": "c1", "success": True})
            for _ in range(2):
                auditor.record_change("fast", {"component": "c1", "success": False})
            report = auditor.generate_report()
            assert report["period_summary"]["effectiveness_rate"] == 0.8
            assert report["period_summary"]["effective_changes"] == 8
            assert report["period_summary"]["ineffective_changes"] == 2

    def test_report_effectiveness_rate_all_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(10):
                auditor.record_change("fast", {"component": "c1", "success": False})
            report = auditor.generate_report()
            assert report["period_summary"]["effectiveness_rate"] == 0.0

    def test_report_effectiveness_rate_all_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(10):
                auditor.record_change("fast", {"component": "c1", "success": True})
            report = auditor.generate_report()
            assert report["period_summary"]["effectiveness_rate"] == 1.0


class TestIneffectivePatternDetection:
    def test_ineffective_pattern_detection(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(3):
                auditor.record_change("fast", {"component": "bad_skill", "success": False})
            for _ in range(7):
                auditor.record_change("fast", {"component": "good_skill", "success": True})
            report = auditor.generate_report()
            ineff_patterns = report["ineffective_patterns"]
            assert any(p["component"] == "bad_skill" for p in ineff_patterns)
            assert not any(p["component"] == "good_skill" for p in ineff_patterns)

    def test_no_ineffective_patterns(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(10):
                auditor.record_change("fast", {"component": "good_skill", "success": True})
            report = auditor.generate_report()
            assert report["ineffective_patterns"] == []


class TestReportHistory:
    def test_get_report_history(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(10):
                auditor.record_change("fast", {"component": "c1", "success": True})
            report1 = auditor.generate_report()
            for _ in range(10):
                auditor.record_change("fast", {"component": "c1", "success": True})
            report2 = auditor.generate_report()
            history = auditor.get_report_history(limit=2)
            assert len(history) == 2
            assert history[0]["report_id"] == report1["report_id"]
            assert history[1]["report_id"] == report2["report_id"]

    def test_get_report_history_limit(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(3):
                for _ in range(10):
                    auditor.record_change("fast", {"component": "c1", "success": True})
                auditor.generate_report()
            history = auditor.get_report_history(limit=2)
            assert len(history) == 2

    def test_get_report_history_empty(self):
        auditor = EvolutionAuditor()
        assert auditor.get_report_history() == []


class TestSummaryText:
    def test_summary_text(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            auditor = EvolutionAuditor(evolution_logger=logger)
            for _ in range(10):
                auditor.record_change("fast", {"component": "c1", "success": True})
            auditor.generate_report()
            summary = auditor.get_summary_report()
            assert "报告 ID" in summary
            assert "有效率" in summary

    def test_summary_text_empty(self):
        auditor = EvolutionAuditor()
        assert auditor.get_summary_report() == "暂无审计报告"


class TestLoggerGetRecentChanges:
    def test_logger_get_recent_changes(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log_change("fast", {"component": "c1", "before": 1, "after": 2, "success": True})
            logger.log_change("slow", {"component": "c2", "before": "a", "after": "b", "success": False})
            logger.log("other", "irrelevant")
            changes = logger.get_recent_changes(count=10)
            assert len(changes) == 2
            assert changes[1]["cycle_type"] == "slow"
            assert changes[1]["success"] is False

    def test_logger_get_recent_changes_empty(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            assert logger.get_recent_changes() == []

    def test_logger_get_recent_changes_limit(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            for i in range(10):
                logger.log_change("fast", {"component": f"c{i}", "success": True})
            changes = logger.get_recent_changes(count=3)
            assert len(changes) == 3


class TestLoggerGetChangeStats:
    def test_logger_get_change_stats(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log_change("fast", {"component": "c1", "success": True})
            logger.log_change("fast", {"component": "c1", "success": True})
            logger.log_change("slow", {"component": "c2", "success": False})
            stats = logger.get_change_stats()
            assert stats["total_fast"] == 2
            assert stats["total_slow"] == 1
            assert stats["effective"] == 2
            assert stats["ineffective"] == 1
            assert stats["by_component"]["c1"]["success"] == 2
            assert stats["by_component"]["c2"]["fail"] == 1

    def test_logger_get_change_stats_empty(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            stats = logger.get_change_stats()
            assert stats["total_fast"] == 0
            assert stats["total_slow"] == 0
            assert stats["effective"] == 0
            assert stats["ineffective"] == 0

    def test_logger_get_change_stats_by_component(self):
        with tempfile.TemporaryDirectory() as tmp:
            logger = EvolutionLogger(tmp)
            logger.log_change("fast", {"component": "c1", "success": True})
            logger.log_change("fast", {"component": "c1", "success": False})
            logger.log_change("fast", {"component": "c2", "success": True})
            stats = logger.get_change_stats()
            assert stats["by_component"]["c1"]["success"] == 1
            assert stats["by_component"]["c1"]["fail"] == 1
            assert stats["by_component"]["c2"]["success"] == 1