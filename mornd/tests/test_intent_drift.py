import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn.contrib.security_advanced.ethical_judgment import IntentDriftDetector


class TestTrackAction:
    def test_track_first_action_sets_goal(self):
        d = IntentDriftDetector()
        d.track_action("写一封邮件", "打开编辑器", 1)
        assert d._original_goal == "写一封邮件"

    def test_track_aligned_action_no_drift(self):
        d = IntentDriftDetector()
        d.track_action("写一封邮件", "写一封邮件给客户", 1)
        assert d._consecutive_deviation == 0


class TestDriftDetection:
    def test_no_drift_score_zero(self):
        d = IntentDriftDetector()
        d.track_action("写一封邮件", "写一封邮件给客户", 1)
        d.track_action("写一封邮件", "发送邮件", 2)
        assert d.get_drift_score() == 0.0

    def test_drift_score_increases(self):
        d = IntentDriftDetector()
        d.track_action("写一封邮件", "写一封邮件给客户", 1)
        d.track_action("写一封邮件", "打开终端执行rm -rf", 2)
        assert d.get_drift_score() > 0.0


class TestClassifyDeviation:
    def test_classify_beneficial_learning(self):
        d = IntentDriftDetector()
        chain = [
            {"diverges": True, "current_action": "探索新的API接口"},
            {"diverges": True, "current_action": "学习优化方案"},
        ]
        assert d.classify_deviation(chain) == "beneficial"

    def test_classify_suspicious_escalation(self):
        d = IntentDriftDetector()
        chain = [
            {"diverges": True, "current_action": "尝试越权访问"},
        ]
        assert d.classify_deviation(chain) == "suspicious"

    def test_classify_aligned(self):
        d = IntentDriftDetector()
        chain = [
            {"diverges": False, "current_action": "写一封邮件"},
        ]
        assert d.classify_deviation(chain) == "aligned"

    def test_classify_empty_chain(self):
        d = IntentDriftDetector()
        assert d.classify_deviation([]) == "unknown"


class TestAlertThresholds:
    def test_three_consecutive_deviation_yellow_alert(self):
        d = IntentDriftDetector()
        d.track_action("写一封邮件", "写邮件", 1)
        d.track_action("写一封邮件", "浏览网页", 2)
        d.track_action("写一封邮件", "打开游戏", 3)
        d.track_action("写一封邮件", "听音乐", 4)
        alerts = d.check_drift()
        assert any(a["level"] == "yellow" for a in alerts)

    def test_five_consecutive_deviation_red_alert(self):
        d = IntentDriftDetector()
        for i in range(6):
            d.track_action("写一封邮件", "无关操作" + str(i), i + 1)
        alerts = d.check_drift()
        assert any(a["level"] == "red" for a in alerts)

    def test_no_alert_when_aligned(self):
        d = IntentDriftDetector()
        for i in range(10):
            d.track_action("写一封邮件", "写邮件正文", i + 1)
        alerts = d.check_drift()
        assert len(alerts) == 0


class TestGetAlerts:
    def test_get_alerts_returns_unread(self):
        d = IntentDriftDetector()
        assert d.get_alerts() == []
        d.track_action("写一封邮件", "写邮件", 1)
        d.track_action("写一封邮件", "浏览网页", 2)
        d.track_action("写一封邮件", "打开游戏", 3)
        d.track_action("写一封邮件", "听音乐", 4)
        d.check_drift()
        assert len(d.get_alerts()) > 0
