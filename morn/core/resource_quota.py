"""Token 双轨制 + 硬配额管理"""

import logging
import time
from typing import Optional

logger = logging.getLogger("morn.quota")


class QuotaExceeded(Exception):
    def __init__(self, plugin_id: str, plugin_level: str, requested: int, remain: int):
        self.plugin_id = plugin_id
        self.plugin_level = plugin_level
        self.requested = requested
        self.remain = remain
        super().__init__(
            f"QuotaExceeded: plugin={plugin_id} level={plugin_level} "
            f"requested={requested} remain={remain}"
        )


class TokenCounter:
    def count_input(self, text: str, model_type: str = "cloud") -> int:
        if model_type == "cloud":
            try:
                return self._count_via_api(text)
            except NotImplementedError:
                return self._count_via_tiktoken(text, scale=1.2)
            except Exception:
                return self._count_via_tiktoken(text, scale=1.2)
        return self._count_via_tiktoken(text, scale=1.2)

    def count_output(self, text: str, model_type: str = "cloud") -> int:
        return self.count_input(text, model_type)

    def _count_via_api(self, text: str) -> int:
        """云端 API 精确计数。默认未实现（fallback 到 tiktoken 估算）。
        子类或全局初始化时可替换为具体 provider 的实现。
        例如：解析 DeepSeek API 响应的 usage.prompt_tokens 字段。
        """
        raise NotImplementedError(
            "API-based counting requires provider-specific implementation. "
            "Fallback to tiktoken estimation."
        )

    def _count_via_tiktoken(self, text: str, scale: float = 1.0) -> int:
        try:
            import tiktoken
            enc = tiktoken.get_encoding("cl100k_base")
            return int(len(enc.encode(text)) * scale)
        except ImportError:
            return int(len(text) * 1.5 * scale)


class QuotaManager:
    LEVEL_WEIGHTS = {
        "S": 0.40,
        "A": 0.30,
        "B": 0.15,
        "C": 0.15,
    }

    def __init__(self, global_budget: int, global_period: int = 60):
        self._global_budget = global_budget
        self._global_period = global_period
        self._buckets: dict[str, dict[str, float]] = {}
        self._start_time = time.monotonic()

    def _get_bucket(self, plugin_level: str) -> dict:
        if plugin_level not in self._buckets:
            weight = self.LEVEL_WEIGHTS.get(plugin_level, 0.0)
            capacity = int(self._global_budget * weight)
            self._buckets[plugin_level] = {
                "tokens": float(capacity),
                "capacity": float(capacity),
                "last_refill": time.monotonic(),
                "level": plugin_level,
            }
        return self._buckets[plugin_level]

    def _refill_bucket(self, bucket: dict):
        now = time.monotonic()
        elapsed = now - bucket["last_refill"]
        if elapsed >= self._global_period:
            bucket["tokens"] = bucket["capacity"]
            bucket["last_refill"] = now
        elif elapsed > 0:
            rate = bucket["capacity"] / self._global_period
            refill = rate * elapsed
            bucket["tokens"] = min(bucket["capacity"], bucket["tokens"] + refill)
            bucket["last_refill"] = now

    def check(self, plugin_level: str, token_count: int, plugin_id: str) -> bool:
        bucket = self._get_bucket(plugin_level)
        self._refill_bucket(bucket)
        return bucket["tokens"] >= token_count

    def consume(self, plugin_level: str, token_count: int, plugin_id: str):
        bucket = self._get_bucket(plugin_level)
        self._refill_bucket(bucket)
        if bucket["tokens"] < token_count:
            raise QuotaExceeded(
                plugin_id=plugin_id,
                plugin_level=plugin_level,
                requested=token_count,
                remain=int(bucket["tokens"]),
            )
        bucket["tokens"] -= token_count

    def adjust(self, plugin_level: str, estimated: int, actual: int, plugin_id: str):
        """API 响应后根据精确计数调整配额消耗。
        调用场景：chat_engine 先 consume(estimated)，收到 API 响应后
        用 adjust(estimated, actual) 修正差额。
        """
        diff = estimated - actual
        if diff > 0:
            bucket = self._get_bucket(plugin_level)
            bucket["tokens"] = min(bucket["capacity"], bucket["tokens"] + diff)

    def get_remain(self, plugin_level: str) -> int:
        bucket = self._get_bucket(plugin_level)
        self._refill_bucket(bucket)
        return int(bucket["tokens"])

    def can_borrow(self, plugin_level: str) -> bool:
        return plugin_level == "C"