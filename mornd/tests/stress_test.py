"""24小时稳定性测试脚本。

可直接运行（完整24小时测试）或通过 pytest 快速验证。
"""

import asyncio
import logging
import os
import random
import signal
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import psutil

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from morn_core.chat.engine import EmotionState
from morn_core.emotion.bond_tracker import BondTracker
from morn_core.server import MornState, heartbeat_loop, memory_monitor


@dataclass
class StressReport:
    duration_hours: float = 0.0
    max_memory_mb: float = 0.0
    total_heartbeats: int = 0
    total_interactions: int = 0
    total_deep_reflections: int = 0
    emotion_violations: int = 0
    memory_warnings: int = 0
    abnormal_exit: bool = False
    errors: list[str] = field(default_factory=list)


class StressTest:
    def __init__(self, duration_minutes: float = 5, interactive: bool = False):
        self.duration = duration_minutes * 60
        self.interactive = interactive
        self.report = StressReport()
        self._errors: list[str] = []
        self._start_time = 0.0
        self._pid = os.getpid()

        self.state = MornState()
        self.emotion = EmotionState()
        self.bond_tracker = BondTracker({})

        self._interaction_count = 0
        self._last_hour_report = 0
        self._consecutive_mem_growth = 0
        self._last_mem_mb = 0.0

        self._heartbeat_count_at_start = 0
        self._emotion_history: list[dict] = []

        self._shutdown = False

    def _handle_signal(self, signum, frame):
        self._shutdown = True

    def _get_rss_mb(self) -> float:
        return psutil.Process(self._pid).memory_info().rss / 1024 / 1024

    def _get_emotion_status(self) -> dict:
        return {
            "calmness": self.emotion.calmness,
            "pleasure": self.emotion.pleasure,
            "connection": self.emotion.connection,
            "determination": self.emotion.determination,
            "anticipation": self.emotion.anticipation,
            "warmth": self.emotion.warmth,
            "ripple": self.emotion.ripple,
        }

    def _check_emotion_bounds(self) -> int:
        violations = 0
        for dim, val in self._get_emotion_status().items():
            if not (0.0 <= val <= 1.0):
                violations += 1
                self._errors.append(f"emotion {dim}={val} out of [0,1] range")
        return violations

    def _simulate_conversation(self):
        messages = [
            "今天过得怎么样？",
            "你好，有什么新鲜事吗？",
            "我在想一个问题...",
            "能帮我分析一下这个吗？",
            "今天心情不错！",
            "有点累了。",
            "你有什么想法？",
            "我们来聊聊哲学吧。",
            "推荐一首歌给我？",
            "今天天气真好。",
        ]
        msg = random.choice(messages)
        delta = random.uniform(-0.3, 0.5)
        tag = random.choice(["", "高兴/满足", "失望/沮丧", "惊讶", "感动", "平淡"])
        self.emotion.apply_delta(delta, tag)
        self.emotion.decay()
        self._interaction_count += 1
        self.state.last_interaction_time = time.time()

        violations = self._check_emotion_bounds()
        self.report.emotion_violations += violations
        if violations:
            self._errors.append(f"emotion out of bounds after interaction #{self._interaction_count}")

        depth = min(self.state.heartbeat_count / 100, 1.0)
        sentiment = self.emotion.pleasure
        days = (time.time() - self.state.start_time) / 86400
        self.bond_tracker.update(depth, sentiment, days)
        self.bond_tracker.save()

    def _trigger_emotion_delta(self):
        delta = random.uniform(-0.5, 0.5)
        tag = random.choice(["强烈的喜悦", "深深的失望", "意外之喜", "温暖的感动", "平静的思考"])
        self.emotion.apply_delta(delta, tag)
        violations = self._check_emotion_bounds()
        self.report.emotion_violations += violations

    def _trigger_deep_introspection(self):
        self.report.total_deep_reflections += 1
        self.emotion.decay()
        self._check_emotion_bounds()

    def _trigger_drift_check(self):
        depth = min(self.state.heartbeat_count / 100, 1.0)
        sentiment = self.emotion.pleasure
        days = (time.time() - self.state.start_time) / 86400
        self.bond_tracker.update(depth, sentiment, days)
        self.bond_tracker.save()

    def _check_process_alive(self) -> bool:
        try:
            proc = psutil.Process(self._pid)
            return proc.is_running()
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return False

    def _check_memory(self) -> bool:
        rss = self._get_rss_mb()
        if rss > self.report.max_memory_mb:
            self.report.max_memory_mb = rss
        if self._last_mem_mb > 0 and rss > self._last_mem_mb:
            self._consecutive_mem_growth += 1
            if self._consecutive_mem_growth >= 3:
                growth = rss - self._last_mem_mb
                self.report.memory_warnings += 1
                self._errors.append(
                    f"memory grew for 3 consecutive checks: "
                    f"{self._last_mem_mb:.1f} -> {rss:.1f} MB (+{growth:.1f}MB)"
                )
                self._consecutive_mem_growth = 0
        else:
            self._consecutive_mem_growth = 0
        self._last_mem_mb = rss
        return True

    def _check_heartbeat(self) -> bool:
        if self.state.heartbeat_count <= self._heartbeat_count_at_start:
            self._errors.append(
                f"heartbeat not increasing: was {self._heartbeat_count_at_start}, "
                f"now {self.state.heartbeat_count}"
            )
            return False
        self._heartbeat_count_at_start = self.state.heartbeat_count
        return True

    def _report_hourly(self, elapsed: float):
        hours = elapsed / 3600
        rss = self._get_rss_mb()
        status = self._get_emotion_status()
        print(f"\n=== Hour {hours:.1f} ===")
        print(f"  Heartbeats: {self.state.heartbeat_count}")
        print(f"  Memory: {rss:.1f} MB (max: {self.report.max_memory_mb:.1f} MB)")
        print(f"  Interactions: {self._interaction_count}")
        print(f"  Deep reflections: {self.report.total_deep_reflections}")
        print(f"  Emotion: calm={status['calmness']:.2f} pleasure={status['pleasure']:.2f} "
              f"connect={status['connection']:.2f}")
        print(f"  Bond: {self.bond_tracker.get_bond():.4f} ({self.bond_tracker.get_stage()})")
        if self._errors:
            print(f"  Warnings/Errors so far: {len(self._errors)}")
            for err in self._errors[-3:]:
                print(f"    - {err}")
        print()

    def _generate_report(self) -> StressReport:
        elapsed = time.time() - self._start_time
        self.report.duration_hours = elapsed / 3600
        self.report.total_heartbeats = self.state.heartbeat_count
        self.report.total_interactions = self._interaction_count
        self.report.abnormal_exit = bool(self._errors)
        self.report.errors = self._errors
        return self.report

    def print_summary(self):
        print("\n" + "=" * 60)
        print("STRESS TEST SUMMARY")
        print("=" * 60)
        print(f"  Duration: {self.report.duration_hours:.2f} hours")
        print(f"  Max memory: {self.report.max_memory_mb:.1f} MB")
        print(f"  Total heartbeats: {self.report.total_heartbeats}")
        print(f"  Total interactions: {self.report.total_interactions}")
        print(f"  Total deep reflections: {self.report.total_deep_reflections}")
        print(f"  Emotion violations: {self.report.emotion_violations}")
        print(f"  Memory warnings: {self.report.memory_warnings}")
        print(f"  Abnormal exit: {self.report.abnormal_exit}")
        if self._errors:
            print(f"  Errors ({len(self._errors)}):")
            for err in self._errors:
                print(f"    - {err}")
        else:
            print("  No errors!")
        print("=" * 60)

    async def run_async(self):
        self._start_time = time.time()
        self.state.start_time = self._start_time
        self._heartbeat_count_at_start = self.state.heartbeat_count

        if self.interactive:
            signal.signal(signal.SIGINT, self._handle_signal)
            signal.signal(signal.SIGTERM, self._handle_signal)

        heartbeat_task = asyncio.create_task(heartbeat_loop(self.state), name="stress-heartbeat")
        memmon_task = asyncio.create_task(memory_monitor(self.state), name="stress-memmon")

        next_interaction = time.monotonic() + 5
        next_emotion_delta = time.monotonic() + 50
        next_deep_introspect = time.monotonic() + 300
        next_drift_check = time.monotonic() + 600
        next_monitor = time.monotonic() + 60
        next_hourly = time.monotonic() + 3600

        end_time = time.monotonic() + self.duration

        try:
            while time.monotonic() < end_time:
                if self._shutdown:
                    break

                now = time.monotonic()

                if now >= next_monitor:
                    next_monitor = now + 60
                    if not self._check_process_alive():
                        self._errors.append("process died during test")
                        break
                    self._check_memory()
                    self._check_heartbeat()

                if now >= next_interaction:
                    next_interaction = now + 300
                    self._simulate_conversation()

                if now >= next_emotion_delta:
                    next_emotion_delta = now + 600
                    self._trigger_emotion_delta()

                if now >= next_deep_introspect:
                    next_deep_introspect = now + 1800
                    self._trigger_deep_introspection()

                if now >= next_drift_check:
                    next_drift_check = now + 3600
                    self._trigger_drift_check()

                if now >= next_hourly:
                    next_hourly = now + 3600
                    elapsed = now - (self._start_time if self._start_time else time.time())
                    self._report_hourly(elapsed)

                await asyncio.sleep(1)

        finally:
            self.state.shutdown = True
            await asyncio.sleep(0.5)

            heartbeat_task.cancel()
            memmon_task.cancel()
            try:
                await heartbeat_task
            except (asyncio.CancelledError, Exception):
                pass
            try:
                await memmon_task
            except (asyncio.CancelledError, Exception):
                pass

        self._generate_report()


def main():
    duration_minutes = float(sys.argv[1]) if len(sys.argv) > 1 else 1440
    test = StressTest(duration_minutes=duration_minutes, interactive=True)
    print(f"Starting stress test for {duration_minutes} minutes ({duration_minutes/60:.1f} hours)")
    print(f"PID: {os.getpid()}")
    asyncio.run(test.run_async())
    test.print_summary()
    if test._errors:
        sys.exit(1)


if __name__ == "__main__":
    main()
