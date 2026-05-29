"""情感回放 EmotionReplay 测试。"""

import csv
import os
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.emotion.emotion_replay import EmotionReplay


class TestEventRecording:
    def test_add_event_and_retrieve(self):
        r = EmotionReplay()
        r.add_event("pleasure", 0.5, 0.6, 0.1, "positive interaction")
        events = r.get_replay(days=365)
        assert len(events) == 1
        assert events[0]["dimension"] == "pleasure"
        assert events[0]["old_value"] == 0.5
        assert events[0]["new_value"] == 0.6
        assert events[0]["delta"] == 0.1

    def test_multiple_events(self):
        r = EmotionReplay()
        r.add_event("calmness", 0.7, 0.5, -0.2, "stress")
        r.add_event("pleasure", 0.5, 0.8, 0.3, "good news")
        r.add_event("warmth", 0.5, 0.7, 0.2, "kind words")
        assert len(r._events) == 3


class TestReplayFiltering:
    def test_replay_filter_by_days(self):
        r = EmotionReplay()
        from datetime import datetime, timezone, timedelta
        old_event = {
            "timestamp": (datetime.now(timezone.utc) - timedelta(days=10)).isoformat(),
            "dimension": "pleasure", "old_value": 0.5, "new_value": 0.5,
            "delta": 0.0, "trigger_event": "old",
        }
        new_event = {
            "timestamp": (datetime.now(timezone.utc) - timedelta(hours=1)).isoformat(),
            "dimension": "pleasure", "old_value": 0.5, "new_value": 0.6,
            "delta": 0.1, "trigger_event": "recent",
        }
        r._events = [old_event, new_event]
        recent = r.get_replay(days=3)
        assert len(recent) == 1
        assert recent[0]["trigger_event"] == "recent"


class TestCsvExport:
    def test_export_csv(self):
        r = EmotionReplay()
        r.add_event("pleasure", 0.5, 0.6, 0.1, "nice chat")
        r.add_event("calmness", 0.7, 0.8, 0.1, "relaxing")
        with tempfile.NamedTemporaryFile(mode="w", suffix=".csv",
                                         delete=False) as tmp:
            csv_path = tmp.name
        try:
            r.export_csv(csv_path)
            with open(csv_path) as f:
                reader = csv.DictReader(f)
                rows = list(reader)
            assert len(rows) == 2
            assert rows[0]["dimension"] == "pleasure"
            assert rows[1]["dimension"] == "calmness"
        finally:
            os.unlink(csv_path)


class TestTriggerTruncation:
    def test_trigger_truncated_at_100_chars(self):
        r = EmotionReplay()
        long_trigger = "x" * 200
        r.add_event("pleasure", 0.5, 0.6, 0.1, long_trigger)
        assert len(r._events[0]["trigger_event"]) == 100

    def test_short_trigger_not_truncated(self):
        r = EmotionReplay()
        r.add_event("pleasure", 0.5, 0.6, 0.1, "short")
        assert r._events[0]["trigger_event"] == "short"


class TestVisualize:
    def test_visualize_returns_figure(self):
        r = EmotionReplay()
        r.add_event("pleasure", 0.5, 0.6, 0.1, "test")
        fig = r.visualize(days=365)
        assert fig is not None

    def test_visualize_empty_returns_figure(self):
        r = EmotionReplay()
        fig = r.visualize(days=7)
        assert fig is not None