"""Token 双轨统计（ADR-003）。

云端路径：使用 API 返回的精确计数。
兜底路径：字符数 / 1.5 × 1.2 安全系数估算。
重试路径：每次独立计数（不累加去重）。
"""

import logging
import time

logger = logging.getLogger("morn.token_tracker")


class TokenTracker:
    def __init__(self):
        # 累计统计
        self._total_input: int = 0
        self._total_output: int = 0
        self._cloud_calls: int = 0
        self._local_calls: int = 0
        self._fallback_events: int = 0  # 本地兜底触发次数
        # 路径分布历史（最近 100 条）
        self._history: list[dict] = []
        self._max_history = 100

    # ── 记录路径 ──

    def record_cloud(
        self,
        input_tokens: int,
        output_tokens: int,
        model: str = "",
        was_fallback: bool = False,
    ) -> None:
        """记录一次云端调用。"""
        self._total_input += input_tokens
        self._total_output += output_tokens
        self._cloud_calls += 1
        if was_fallback:
            self._fallback_events += 1
        self._history.append({
            "path": "cloud",
            "model": model,
            "input": input_tokens,
            "output": output_tokens,
            "fallback": was_fallback,
            "ts": time.time(),
        })
        self._trim()

    def record_local(
        self,
        input_text: str,
        output_text: str,
        model: str = "",
    ) -> None:
        """记录一次本地调用。使用字符估算 × 1.2 安全系数。"""
        input_tokens = self._estimate(input_text)
        output_tokens = self._estimate(output_text)
        self._total_input += input_tokens
        self._total_output += output_tokens
        self._local_calls += 1
        self._history.append({
            "path": "local",
            "model": model,
            "input": input_tokens,
            "output": output_tokens,
            "estimated": True,
            "ts": time.time(),
        })
        self._trim()

    # ── 估算 ──

    @staticmethod
    def _estimate(text: str) -> int:
        """粗略 token 估算：字符数 ÷ 1.5 × 1.2 安全系数。"""
        if not text:
            return 0
        raw = len(text) / 1.5
        return max(1, int(raw * 1.2))

    # ── 查询 ──

    def get_summary(self) -> dict:
        return {
            "total_input_tokens": self._total_input,
            "total_output_tokens": self._total_output,
            "cloud_calls": self._cloud_calls,
            "local_calls": self._local_calls,
            "fallback_events": self._fallback_events,
            "history_count": len(self._history),
        }

    def get_path_distribution(self) -> dict:
        """返回云端 vs 本地 vs 兜底的比例。"""
        total = self._cloud_calls + self._local_calls
        if total == 0:
            return {"cloud_pct": 0, "local_pct": 0, "fallback_pct": 0}
        return {
            "cloud_pct": round(self._cloud_calls / total * 100, 1),
            "local_pct": round(self._local_calls / total * 100, 1),
            "fallback_pct": round(self._fallback_events / max(self._cloud_calls, 1) * 100, 1),
        }

    # ── 内部 ──

    def _trim(self) -> None:
        if len(self._history) > self._max_history:
            self._history = self._history[-self._max_history:]
