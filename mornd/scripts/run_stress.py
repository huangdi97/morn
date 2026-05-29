#!/usr/bin/env python3
"""
压力测试自动化运行脚本。

用法：
    python scripts/run_stress.py                    # 默认 5 分钟快速测试
    python scripts/run_stress.py --duration 72      # 72 小时完整测试
    python scripts/run_stress.py --duration 1 --output /tmp/stress_results.json  # 自定义输出
"""

import argparse
import asyncio
import json
import logging
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from tests.stress_test import StressTest, StressReport

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[
        logging.FileHandler(Path(__file__).parent.parent / "logs" / "stress_test.log"),
        logging.StreamHandler(),
    ],
)
logger = logging.getLogger("stress_runner")

DEFAULT_RESULTS_DIR = Path(__file__).parent.parent / "stress_results"


def load_baseline(results_dir: Path) -> dict:
    baseline_file = results_dir / "baseline.json"
    if baseline_file.exists():
        with open(baseline_file) as f:
            return json.load(f)
    return {}


def save_baseline(report: dict, results_dir: Path):
    results_dir.mkdir(parents=True, exist_ok=True)
    with open(results_dir / "baseline.json", "w") as f:
        json.dump(report, f, indent=2, default=str)


def compare_with_baseline(current: dict, baseline: dict) -> list[str]:
    alerts = []
    if not baseline:
        return ["首次运行，无基线数据可对比"]

    current_mem = current.get("max_memory_mb", 0)
    baseline_mem = baseline.get("max_memory_mb", 0)
    if baseline_mem > 0 and current_mem > baseline_mem * 1.2:
        alerts.append(f"内存增长超过基线20%: 当前{current_mem:.1f}MB vs 基线{baseline_mem:.1f}MB")

    current_errors = len(current.get("errors", []))
    baseline_errors = len(baseline.get("errors", []))
    if current_errors > baseline_errors + 2:
        alerts.append(f"错误数异常增长: 当前{current_errors} vs 基线{baseline_errors}")

    current_ev = current.get("emotion_violations", 0)
    baseline_ev = baseline.get("emotion_violations", 0)
    if current_ev > baseline_ev + 1:
        alerts.append(f"情感违规增加: 当前{current_ev} vs 基线{baseline_ev}")

    return alerts


async def main():
    parser = argparse.ArgumentParser(description="Morn 压力测试自动化运行")
    parser.add_argument("--duration", type=float, default=0.5,
                        help="测试时长（分钟），默认 0.5（30秒快速验证）")
    parser.add_argument("--output", type=str, default=None,
                        help="结果输出路径（JSON）")
    parser.add_argument("--save-baseline", action="store_true", default=True,
                        help="保存结果为基线（默认开启）")
    parser.add_argument("--interactive", action="store_true", default=False,
                        help="启用交互模式")
    args = parser.parse_args()

    results_dir = DEFAULT_RESULTS_DIR
    logger.info(f"启动压力测试，时长 {args.duration} 分钟")

    baseline = load_baseline(results_dir)
    if baseline:
        logger.info(f"已加载基线: {baseline.get('duration_hours', 0):.2f}h 运行")

    st = StressTest(duration_minutes=args.duration, interactive=args.interactive)
    await st.run_async()

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "duration_minutes": args.duration,
        "duration_hours": st.report.duration_hours,
        "max_memory_mb": st.report.max_memory_mb,
        "total_heartbeats": st.report.total_heartbeats,
        "total_interactions": st.report.total_interactions,
        "emotion_violations": st.report.emotion_violations,
        "memory_warnings": st.report.memory_warnings,
        "abnormal_exit": st.report.abnormal_exit,
        "errors": st.report.errors[:10],
    }

    alerts = compare_with_baseline(report, baseline)
    report["alerts"] = alerts

    output_path = args.output or str(results_dir / f"stress_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json")
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(report, f, indent=2, default=str)

    logger.info(f"结果已保存到 {output_path}")

    if args.save_baseline:
        save_baseline(report, results_dir)
        logger.info("基线已更新")

    if alerts:
        logger.warning("告警:")
        for a in alerts:
            logger.warning(f"  - {a}")
    else:
        logger.info("无异常告警")

    print(f"\n{'='*50}")
    print(f"压力测试完成: {args.duration} 分钟")
    print(f"  心跳数: {st.report.total_heartbeats}")
    print(f"  最大内存: {st.report.max_memory_mb:.1f} MB")
    print(f"  错误数: {len(st.report.errors)}")
    print(f"  情感违规: {st.report.emotion_violations}")
    print(f"  异常退出: {'是' if st.report.abnormal_exit else '否'}")
    print(f"  告警: {len(alerts)}")
    print(f"{'='*50}")

    if st.report.abnormal_exit or alerts:
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
