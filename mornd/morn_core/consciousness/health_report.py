import logging
import time
from datetime import datetime, timezone

logger = logging.getLogger("morn.health")


class HealthReport:
    def __init__(self, memory_store, emotion_engine, evolution_orchestrator,
                 bond_tracker, data_dir):
        self.memory_store = memory_store
        self.emotion_engine = emotion_engine
        self.evolution_orchestrator = evolution_orchestrator
        self.bond_tracker = bond_tracker
        self.data_dir = data_dir

    async def generate(self) -> str:
        lines = []
        now = datetime.now(timezone.utc)
        uptime_seconds = None
        start_time = getattr(self.memory_store, "_server_start_time", None)
        if start_time:
            uptime_seconds = time.time() - start_time
        if uptime_seconds is not None:
            if uptime_seconds < 60:
                uptime_str = f"{uptime_seconds:.0f}秒"
            elif uptime_seconds < 3600:
                uptime_str = f"{uptime_seconds//60}分{uptime_seconds%60:.0f}秒"
            else:
                uptime_str = f"{uptime_seconds//3600}时{(uptime_seconds%3600)//60}分"
            lines.append(f"运行时长：{uptime_str}")
        else:
            lines.append("运行时长：未知")

        l2_count = 0
        l2_recent = 0
        try:
            cursor = await self.memory_store.db.execute(
                "SELECT COUNT(*) as cnt FROM capsules"
            )
            row = await cursor.fetchone()
            l2_count = row["cnt"] if row else 0
        except Exception:
            pass
        try:
            cursor = await self.memory_store.db.execute(
                "SELECT COUNT(*) as cnt FROM capsules WHERE timestamp >= datetime('now', '-30 days')"
            )
            row = await cursor.fetchone()
            l2_recent = row["cnt"] if row else 0
        except Exception:
            pass
        lines.append(f"L2 胶囊：{l2_count} 条（近30天写入 {l2_recent} 条）")

        l4_count = 0
        try:
            cursor = await self.memory_store.db.execute(
                "SELECT COUNT(*) as cnt FROM knowledge"
            )
            row = await cursor.fetchone()
            l4_count = row["cnt"] if row else 0
        except Exception:
            pass
        lines.append(f"L4 信念：{l4_count} 条")

        if self.emotion_engine:
            e = self.emotion_engine
            dims = ["calmness", "pleasure", "warmth", "connection"]
            parts = []
            for d in dims:
                val = getattr(e, d, 0.0)
                if val >= 0.7:
                    trend = "稳定偏高"
                elif val >= 0.4:
                    trend = "平稳"
                else:
                    trend = "偏低"
                parts.append(f"{d}={val:.2f}({trend})")
            lines.append(f"情感状态：{' | '.join(parts)}")
        else:
            lines.append("情感状态：不可用")

        if self.bond_tracker:
            bond_val = self.bond_tracker.get_bond()
            stage = self.bond_tracker.get_stage()
            lines.append(f"Bond 值：{bond_val:.4f}（{stage}）")
        else:
            lines.append("Bond 值：不可用")

        skill_count = 0
        skill_active = 0
        if self.evolution_orchestrator:
            try:
                fast_status = self.evolution_orchestrator.fast_cycle.get_status()
                skill_count = len(fast_status.get("tasks", {}))
                active_tasks = [t for t in fast_status.get("tasks", {}).values() if t.get("enabled")]
                skill_active = len(active_tasks)
            except Exception:
                pass
        lines.append(f"技能：{skill_count} 个（活跃 {skill_active} 个）")

        db_size = 0
        try:
            db_path = self.memory_store.db_path
            if db_path and db_path.exists():
                db_size = db_path.stat().st_size
        except Exception:
            pass
        if db_size > 0:
            if db_size > 1024 * 1024:
                size_str = f"{db_size / 1024 / 1024:.1f}MB"
            elif db_size > 1024:
                size_str = f"{db_size / 1024:.1f}KB"
            else:
                size_str = f"{db_size}B"
            lines.append(f"存储占用：{size_str}")

        return "健康报告\n" + "─" * 20 + "\n" + "\n".join(lines) + "\n" + "─" * 20 + f"\n报告时间：{now.strftime('%Y-%m-%d %H:%M:%S')}"
