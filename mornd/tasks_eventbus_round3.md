# Morn v0.4 第三轮：Memory 事件 + 插件剩余集成

## 编程原则（同前）

---

## 任务 1：MemoryStore EventBus 集成

**文件**：`morn_core/memory/store.py`

**改动：**
1. MemoryStore.__init__ 新增可选 `event_bus` 参数
2. 所有创建/保存事件胶囊的方法末尾调用 `_publish_capsule_written(capsule_id, data)`
3. 只发布胶囊写入事件，不发布读取/删除事件

```python
class MemoryStore:
    def __init__(self, data_dir, enable_encryption=False, event_bus=None):
        ...
        self._event_bus = event_bus

    async def _publish_capsule_written(self, capsule_id: str, capsule_data: dict):
        if self._event_bus is None:
            return
        await self._event_bus.publish(Event(
            type="memory.capsule_written",
            payload={
                "capsule_id": capsule_id,
                "session_id": capsule_data.get("session_id", ""),
                "trust_level": capsule_data.get("trust_level", "mtz"),
                "source": capsule_data.get("source", "unknown"),
            },
            source="memory_core",
            priority=Priority.MEDIUM,
        ))
```

**胶囊写入方法**（搜索 `async def` 中涉及 capsule 创建/保存的方法）：
- `add_capsule` 或 `save_capsule` — 写入时调用
- `import_capsules` — 批量导入时调用
- `_store_capsule` — 内部保存时调用

不发布读取操作。不发布删除操作。

---

## 任务 2：Server 集成 MemoryStore event_bus + HealthMonitor

**文件**：`morn_core/server.py`

**改动：**
1. 创建 MemoryStore 时传入 `event_bus`
2. 创建 HealthMonitor 并注册其 Hook
3. 创建 SelfReflection 后也注册其 Hook（自省循环应改为事件驱动）

```python
# 在 main() 中，创建 event_bus 和 memory_store 后：
state.memory_store = MemoryStore(data_dir, enable_encryption, event_bus=event_bus)

# HealthMonitor 注册：
from morn_core.eventbus.health_monitor import HealthMonitor
health = HealthMonitor(event_bus, state)
hook_manager.register(HookRegistration(
    plugin_id="health_monitor",
    event_type="heartbeat.minute",
    callback=health.self_check,
    timeout=5.0,
))
hook_manager.register(HookRegistration(
    plugin_id="health_monitor",
    event_type="heartbeat.tick",
    callback=health.detect_clock_jump,
    timeout=1.0,
))

# SelfReflection 注册（自省循环也改为事件驱动）：
hook_manager.register(HookRegistration(
    plugin_id="self_reflection",
    event_type="heartbeat.minute",
    callback=self_reflection_event_callback,
    timeout=15.0,
))
```

---

## 任务 3：测试

**文件**：`tests/test_eventbus_integration.py`（新建，如果不存在则创建；如果存在则追加）

测试覆盖：
1. **Memory 事件**：写入胶囊 → `memory.capsule_written` 被发布
2. **HealthMonitor 自检测**：EventBus 深队列 → `kernel.health_warning` 被发布
3. **全量注册**：`register_all_plugin_hooks` + HealthMonitor + SelfReflection 全部注册，不抛出异常
4. **MemoryStore 空 event_bus**：event_bus=None 时不崩溃，不发布事件

---

## 验收标准

1. ✅ `pytest tests/test_eventbus_integration.py -v` 全部通过
2. ✅ `pytest tests/test_memory_store.py -x -q --timeout=60` 无回归
3. ✅ `pytest tests/ -x -q --timeout=120`（或子集）无回归
4. ✅ server.py 创建 MemoryStore 时传入 event_bus
5. ✅ server.py 注册 HealthMonitor 到 HookManager
6. ✅ 写入胶囊后 debug 日志记录 "capsule_written published"
