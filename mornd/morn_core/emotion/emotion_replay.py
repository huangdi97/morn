import csv
import io
from datetime import datetime, timezone, timedelta


class EmotionReplay:
    def __init__(self):
        self._events = []

    def add_event(self, dim: str, old_val: float, new_val: float,
                  delta: float, trigger: str):
        now = datetime.now(timezone.utc).isoformat()
        trigger_truncated = trigger[:100]
        self._events.append({
            "timestamp": now,
            "dimension": dim,
            "old_value": old_val,
            "new_value": new_val,
            "delta": delta,
            "trigger_event": trigger_truncated,
        })

    def get_replay(self, days: int = 7) -> list:
        cutoff = datetime.now(timezone.utc) - timedelta(days=days)
        result = []
        for ev in self._events:
            ev_time = datetime.fromisoformat(ev["timestamp"])
            if ev_time >= cutoff:
                result.append(ev)
        return result

    def export_csv(self, path: str):
        fieldnames = ["timestamp", "dimension", "old_value", "new_value",
                      "delta", "trigger_event"]
        with open(path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(self._events)

    def visualize(self, days: int = 7):
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
        from matplotlib.dates import DateFormatter

        events = self.get_replay(days)
        if not events:
            fig, ax = plt.subplots(figsize=(10, 5))
            ax.text(0.5, 0.5, "No emotion events in this period",
                    ha="center", va="center", transform=ax.transAxes)
            return fig

        fig, ax = plt.subplots(figsize=(10, 5))
        dims = sorted(set(e["dimension"] for e in events))
        for dim in dims:
            dim_events = [e for e in events if e["dimension"] == dim]
            timestamps = [datetime.fromisoformat(e["timestamp"]) for e in dim_events]
            values = [e["new_value"] for e in dim_events]
            ax.plot(timestamps, values, marker="o", linestyle="-",
                    label=dim, markersize=3)

        ax.set_ylabel("Value")
        ax.set_xlabel("Time")
        ax.set_title(f"Emotion Timeline (past {days} days)")
        ax.legend(loc="best")
        ax.set_ylim(0, 1)
        ax.xaxis.set_major_formatter(DateFormatter("%m-%d %H:%M"))
        fig.autofmt_xdate()
        return fig