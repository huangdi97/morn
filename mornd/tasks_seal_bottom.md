# Morn v0.4 封底轮：底层完整闭环

## 编程原则（同前）

---

## 任务 1：安全组件全接入 EventBus

**文件**：`morn_core/security/user_protection.py` + `morn_core/security/external_boundary.py` + `morn_core/security/ethical_judgment.py`

**改动**：这三个安全组件在检测到违规时 publish `security.alert` 事件。

### user_protection.py

在 `UserProtection` 的校验方法中添加 EventBus 发布：

```python
class UserProtection:
    def __init__(self, ..., event_bus=None):
        self._event_bus = event_bus

    async def _publish_alert(self, message: str):
        if self._event_bus:
            await self._event_bus.publish(Event(
                type="security.alert",
                payload={"source": "user_protection", "message": message},
                source="user_protection",
                priority=Priority.HIGH,
            ))
```

在拦截到诱导性情感行为的逻辑末尾调用 `_publish_alert()`。

### external_boundary.py

类似地，在 `ExternalBoundary` 的出站白名单校验失败时发布 `security.alert`。

### ethical_judgment.py

在 `EthicalJudgment` 判定违规时 publish `security.alert`。

---

## 任务 2：systemd service 补 watchdog（ADR-006）

**文件**：`/etc/systemd/system/morn.service`（如不存在则新建配置到 `~/.config/systemd/user/`）

在 morn.service 的 `[Service]` 段添加：

```
WatchdogSec=30
```

并在 `morn_core/server.py` 的 `main()` 启动后添加 sd_notify：

```python
try:
    import systemd.daemon
    systemd.daemon.notify("READY=1")
    # 启动 watchdog 通知协程
    async def _watchdog_ping():
        while True:
            await asyncio.sleep(10)
            systemd.daemon.notify("WATCHDOG=1")
    tasks.append(asyncio.create_task(_watchdog_ping(), name="morn-watchdog"))
except ImportError:
    pass  # 非 systemd 环境（开发/测试）静默跳过
```

---

## 任务 3：Server 配置文件集中化

**文件**：`morn_core/server.py` 的 `load_config()`

给 config.json 新增安全相关的默认字段：

```python
default_config = {
    # ... 现有字段 ...
    "risk_preference": "yellow",      # 创建者风险偏好：green/yellow/orange/red
    "risk_cooling_period": 30,        # 操作冷却期（分钟）
    "security_hot_reload": True,      # 是否启用安全热重载
    "watchdog_enabled": True,         # 是否启用 systemd watchdog
}
```

这样所有安全配置汇聚到一个位置，SecurityValidator 和 CLIExecutor 自动继承。

---

## 任务 4：端到端验证

**文件**：`tests/test_end_to_end_live.py`（新建）

模拟一次完整的 Morn 启动 → 心跳 → 事件发布 → 安全验证链路：

```python
class TestEndToEndLive:
    """验证底层所有组件协作的完整链路"""

    async def test_heartbeat_to_eventbus_lifecycle(self):
        """心跳 → EventBus 事件 → Hook 触发"""
        ...

    async def test_security_validator_integration(self):
        """CLIExecutor → SecurityValidator → security.alert"""
        ...

    async def test_memory_capsule_event_chain(self):
        """MemoryStore 写入 → memory.capsule_written → 下游订阅者"""
        ...

    async def test_health_monitor_self_check(self):
        """HealthMonitor → 发布 kernel.health_warning"""
        ...

    async def test_hot_reload_config_change(self):
        """修改 config.json → 5 秒内 SecurityValidator 自动加载"""
        ...
```

每个测试启动一个迷你 EventBus + 对应组件，不依赖完整 Morn 实例（保持独立快速）。

---

## 验收标准

1. ✅ UserProtection 拦截时发布 security.alert
2. ✅ ExternalBoundary 拦截时发布 security.alert
3. ✅ EthicalJudgment 判违规时发布 security.alert
4. ✅ morn.service 有 WatchdogSec=30（或至少配置可被应用）
5. ✅ server.py 的 sd_notify 启动兼容非 systemd 环境
6. ✅ config.json 新增风险偏好/冷却期等字段
7. ✅ 端到端链路测试全部通过
8. ✅ 之前 120 个测试全部仍通过
9. ✅ server.py import 正常
