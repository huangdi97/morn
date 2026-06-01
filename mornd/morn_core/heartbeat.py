import asyncio
import gc
import os
import time

import psutil

from morn_core.eventbus.bus import Event, EventBus, Priority

# ── systemd watchdog (sd_notify) via ctypes ──
_HAS_SD_NOTIFY = False
try:
    import ctypes
    import ctypes.util

    _libsystemd_path = ctypes.util.find_library("systemd")
    if _libsystemd_path:
        _libsystemd = ctypes.CDLL(_libsystemd_path, use_errno=True)
        _libsystemd.sd_notify.argtypes = (ctypes.c_int, ctypes.c_char_p)
        _libsystemd.sd_notify.restype = ctypes.c_int
        _HAS_SD_NOTIFY = True
except Exception:
    pass


def _sd_notify(state: str) -> None:
    """Send a sd_notify(3) message.  Silent no-op if systemd is absent."""
    global _HAS_SD_NOTIFY
    if not _HAS_SD_NOTIFY:
        return
    try:
        _libsystemd.sd_notify(0, state.encode("utf-8"))
    except Exception:
        # Once we fail, stop trying for the rest of this process lifetime.
        _HAS_SD_NOTIFY = False


async def heartbeat_loop(state, event_bus: EventBus):
    while not state.shutdown:
        target_time = time.monotonic() + 1
        state.heartbeat_count += 1
        state.last_heartbeat = time.monotonic()

        # systemd watchdog ping every 10 seconds
        if state.heartbeat_count % 10 == 0:
            _sd_notify("WATCHDOG=1")

        await event_bus.publish(Event(
            type="heartbeat.tick",
            payload={"count": state.heartbeat_count},
            source="kernel",
            priority=Priority.HIGH,
        ))

        if state.heartbeat_count % 60 == 0:
            await event_bus.publish(Event(
                type="heartbeat.minute",
                payload={"count": state.heartbeat_count},
                source="kernel",
                priority=Priority.MEDIUM,
            ))

        if state.heartbeat_count % 3600 == 0:
            await event_bus.publish(Event(
                type="heartbeat.hour",
                payload={"count": state.heartbeat_count},
                source="kernel",
                priority=Priority.MEDIUM,
            ))

        if state.heartbeat_count % 60 == 0:
            state.log("heartbeat", f"#{state.heartbeat_count} alive")

        await asyncio.sleep(max(0, target_time - time.monotonic()))


async def memory_monitor(state):
    while True:
        target_time = time.monotonic() + 60
        if state.shutdown:
            break
        rss = psutil.Process(os.getpid()).memory_info().rss
        mem_mb = round(rss / 1024 / 1024, 2)
        state.mem_history.append((time.time(), mem_mb))
        if len(state.mem_history) > 1440:
            state.mem_history.pop(0)
        if len(state.mem_history) >= 4:
            recent = state.mem_history[-4:]
            if recent[0][1] < recent[1][1] < recent[2][1] < recent[3][1]:
                diffs_total = recent[3][1] - recent[0][1]
                if diffs_total > 30:
                    gc.collect()
                    state.log("memory_gc", f"forced gc after {diffs_total:.1f}MB growth")
        await asyncio.sleep(max(0, target_time - time.monotonic()))


async def wal_checkpoint(state):
    while True:
        target_time = time.monotonic() + 86400
        if state.shutdown:
            break
        await asyncio.sleep(max(0, target_time - time.monotonic()))
        if state.db is not None:
            await state.db.execute("PRAGMA wal_checkpoint(TRUNCATE)")
            state.log("wal", "checkpoint completed")