# Morn v0.4 插件事件总线迁移

## 编程原则

### 14条核心准则
1. **Think Before Coding** — 先想后写，改之前先理解整体结构
2. **Simplicity First** — 简单优先，不加不必要的抽象层
3. **Surgical Changes** — 一次一事，一个commit只做一件事
4. **Goal-Driven Execution** — 目标驱动，先明确"要什么"再"怎么写"
5. **架构优先，拒绝补丁** — 不堆补丁，架构不合理就重构
6. **面向组件的构建** — 模块化，每个组件职责清晰
7. **显式优于隐式** — 明确的数据流和依赖，不搞魔术
8. **代码整洁与自文档化** — 代码即文档，命名和结构说明一切
9. **单一职责** — 一个函数/类只做一件事
10. **组合优于委托** — 组合模式 > 继承/委托
11. **单一状态源** — 状态只在一个地方管理
12. **避免语法糖** — 可读性 > 炫技
13. **命名一致性** — 同一概念用同一命名
14. **文件不超过300行** — 超了拆

### 低耦合原则
- 模块间只传ID，不传对象
- 不 import 其他模块的内部函数
- 依赖倒置：模块通过事件ID通信，不直接耦合

### 执行方式
- 新文件用 opencode run 创建
- 每轮结束后跑测试确认无回归
- 所有文件修改通过 opencode run 执行

---

## 任务 1：统一插件注册接口

**文件**：`morn_core/eventbus/plugin_base.py`（新建）

定义一个统一接口，所有插件通过这个接口注册到事件总线：

```python
@dataclass
class PluginInfo:
    plugin_id: str           # 唯一标识，如 "dream_engine"
    name: str                # 可读名称
    version: str             # 语义化版本
    description: str         # 简短描述
    hooks: list[HookRegistration]  # 注册的 Hook 列表

def register_plugin_hooks(plugin: PluginInfo, event_bus: EventBus, hook_manager: HookManager, state) -> None:
    """统一注册插件的所有 Hook"""
    for hook in plugin.hooks:
        hook_manager.register(hook)
```

这个接口做两件事：
1. 定义一个 `PluginInfo` 数据类，包含插件的元信息和 Hook 列表
2. 提供一个 `register_plugin_hooks()` 函数，遍历 Hook 列表并注册到 HookManager

---

## 任务 2：迁移 plugin loops 到事件驱动

**文件**：修改以下文件中的循环函数，改为事件订阅模式

现有的 8 个后台循环都在 `morn_core/server.py` 中：

| 循环函数 | 间隔 | 对应事件 | 插件ID |
|---------|------|---------|--------|
| dream_loop | 60s | heartbeat.minute | dream_engine |
| identity_loop | 60s | heartbeat.minute | identity_affirmer |
| self_pruning_loop | 600s | heartbeat.hour (每10次) | self_pruner |
| bond_update_loop | 300s | heartbeat.minute (每5次) | bond_tracker |
| intent_drift_loop | 600s | heartbeat.hour (每10次) | intent_drift |
| audit_loop | 600s | heartbeat.hour (每10次) | audit_agent |
| thinking_evolution_loop | 3600s | heartbeat.hour | thinking_evolver |
| milestone_loop | 300s | heartbeat.minute (每5次) | milestones |

### 改造方式

每个循环函数从「独立 while 循环」改为「事件驱动的 callback」。以 dream_loop 为例：

**改造前（server.py）：**
```python
async def dream_loop(state: MornState):
    while True:
        target_time = time.monotonic() + 60
        if state.shutdown:
            break
        if state.dream_engine:
            idle = time.time() - state.last_interaction_time
            try:
                await state.dream_engine.tick(idle)
            except Exception as e:
                logging.getLogger("morn").warning("dream tick failed: %s", e)
        await asyncio.sleep(max(0, target_time - time.monotonic()))
```

**改造后：**
```python
def register_dream_engine_hooks(event_bus, hook_manager, state):
    """Dream Engine 注册自己的事件 Hook"""
    async def on_minute(event):
        if state.dream_engine:
            idle = time.time() - state.last_interaction_time
            try:
                await state.dream_engine.tick(idle)
            except Exception as e:
                logging.getLogger("morn").warning("dream tick failed: %s", e)
                # 发布 task.failed 事件
                await event_bus.publish(Event(
                    type="task.failed",
                    payload={"plugin": "dream_engine", "error": str(e)},
                    source="dream_engine",
                    priority=Priority.HIGH,
                    timestamp=time.time(),
                ))

    hook_manager.register(HookRegistration(
        plugin_id="dream_engine",
        event_type="heartbeat.minute",
        callback=on_minute,
        timeout=10.0,  # dream engine 可能需要更长时间
    ))
```

### 对每个循环的改造要求

每个循环需要在原模块文件（或原位置）中导出一个 `register_<plugin_id>_hooks()` 函数，接收 `event_bus, hook_manager, state` 三个参数。

对于定时逻辑中带有「每 N 次执行一次」的（如 self_pruning_loop 每 600 秒而非每 60 秒），使用计数器变量：

```python
_prune_counter = 0

async def on_minute(event):
    nonlocal _prune_counter
    _prune_counter += 1
    if _prune_counter < 10:  # 每 10 次 heartbeat.minute = 每 600s
        return
    _prune_counter = 0
    # 实际执行逻辑
```

---

## 任务 3：Memory publish capsule_written 事件

**文件**：修改 `morn_core/memory/store.py`

在 MemoryStore 的写入方法中（事件胶囊创建/保存时），如果创建者传入 event_bus 引用，则 publish `memory.capsule_written` 事件。

```python
class MemoryStore:
    def __init__(self, ..., event_bus: Optional[EventBus] = None):
        self._event_bus = event_bus
        ...

    async def _publish_capsule_event(self, capsule_id: str, capsule_data: dict):
        if self._event_bus:
            await self._event_bus.publish(Event(
                type="memory.capsule_written",
                payload={"capsule_id": capsule_id, **capsule_data},
                source="memory_core",
                priority=Priority.MEDIUM,
                timestamp=time.time(),
            ))
```

在写入胶囊的方法中调用 `_publish_capsule_event()`。

---

## 任务 4：Server 重构 — 移除独立循环，改为插件注册

**文件**：修改 `morn_core/server.py`

### 改动内容

1. 移除 8 个独立的后台循环函数的函数定义（dream_loop, identity_loop, self_pruning_loop, bond_update_loop, intent_drift_loop, audit_loop, thinking_evolution_loop, milestone_loop）
2. 在 `main()` 中，初始化完成后调用各个插件的 `register_*_hooks()` 函数
3. 保留原有的 `heartbeat_loop`、`memory_monitor`、`wal_checkpoint`、`cli_loop`（这些不是插件——它们是内核组件）
4. 循环对应的注册逻辑可以放在原模块中，或者在 server.py 末尾定义，或者在新建的 `morn_core/eventbus/plugin_registry.py` 中统一注册

### 推荐方案

创建一个 `morn_core/eventbus/plugin_registry.py`：

```python
"""统一的插件注册入口，将所有插件的 Hook 注册到事件总线"""

from morn_core.eventbus.bus import EventBus, Event, Priority
from morn_core.eventbus.hooks import HookManager, HookRegistration

def register_all_plugin_hooks(event_bus: EventBus, hook_manager: HookManager, state) -> None:
    """注册所有插件的 Hook"""
    register_dream_engine_hooks(event_bus, hook_manager, state)
    register_identity_hooks(event_bus, hook_manager, state)
    register_self_pruning_hooks(event_bus, hook_manager, state)
    register_bond_update_hooks(event_bus, hook_manager, state)
    register_intent_drift_hooks(event_bus, hook_manager, state)
    register_audit_hooks(event_bus, hook_manager, state)
    register_thinking_evolution_hooks(event_bus, hook_manager, state)
    register_milestone_hooks(event_bus, hook_manager, state)


def register_dream_engine_hooks(event_bus, hook_manager, state):
    """Dream Engine: 每分钟触发一次"""
    async def on_minute(event):
        if not state.dream_engine:
            return
        idle = time.time() - state.last_interaction_time
        try:
            await state.dream_engine.tick(idle)
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "dream_engine", "error": str(e)},
                source="dream_engine", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="dream_engine", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_identity_hooks(event_bus, hook_manager, state):
    """身份确认: 每分钟触发一次"""
    async def on_minute(event):
        if not state.identity_affirmer:
            return
        try:
            await state.identity_affirmer.tick()
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "identity_affirmer", "error": str(e)},
                source="identity_affirmer", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="identity_affirmer", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_self_pruning_hooks(event_bus, hook_manager, state):
    """自我瘦身: 每 600 秒 (10次 heartbeat.minute)"""
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 10:
            return
        counter = 0
        if not state.self_pruner:
            return
        try:
            result = await state.self_pruner.diagnose()
            if result.get("capsules_pruned", 0) or result.get("skills_pruned", 0) or result.get("emotion_pruned", 0):
                await event_bus.publish(Event(
                    type="self_pruning.completed",
                    payload=result,
                    source="self_pruner", priority=Priority.LOW,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "self_pruner", "error": str(e)},
                source="self_pruner", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="self_pruner", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_bond_update_hooks(event_bus, hook_manager, state):
    """关系深度: 每 300 秒 (5次 heartbeat.minute)"""
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 5:
            return
        counter = 0
        if not state.bond_tracker or not state.chat_engine:
            return
        try:
            idle = time.time() - state.last_interaction_time
            depth = min(state.heartbeat_count / 100, 1.0)
            sentiment = state.chat_engine.emotion.pleasure
            days = (time.time() - state.start_time) / 86400
            new_bond = state.bond_tracker.update(depth, sentiment, days)
            state.bond_tracker.save()
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "bond_tracker", "error": str(e)},
                source="bond_tracker", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="bond_tracker", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_intent_drift_hooks(event_bus, hook_manager, state):
    """意图漂移检测: 每 600 秒"""
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 10:
            return
        counter = 0
        if not state.intent_drift_detector:
            return
        try:
            alerts = state.intent_drift_detector.check_drift()
            for alert in alerts:
                await event_bus.publish(Event(
                    type="security.alert",
                    payload=alert,
                    source="intent_drift_detector", priority=Priority.HIGH,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "intent_drift_detector", "error": str(e)},
                source="intent_drift_detector", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="intent_drift", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_audit_hooks(event_bus, hook_manager, state):
    """审计 Agent: 每 600 秒"""
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 10:
            return
        counter = 0
        if not state.audit_agent or not state.memory_store:
            return
        try:
            cursor = await state.memory_store.db.execute(
                "SELECT * FROM capsules WHERE source NOT IN ('audit_agent', 'self_reflection') ORDER BY timestamp DESC LIMIT 10"
            )
            rows = await cursor.fetchall()
            for row in rows:
                cap = dict(row)
                count = await state.audit_agent.extract_and_deposit(cap)
                if count:
                    await event_bus.publish(Event(
                        type="audit.triples_extracted",
                        payload={"capsule_id": cap.get("event_id"), "count": count},
                        source="audit_agent", priority=Priority.LOW,
                    ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "audit_agent", "error": str(e)},
                source="audit_agent", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="audit_agent", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))


def register_thinking_evolution_hooks(event_bus, hook_manager, state):
    """思维进化: 每 3600 秒触发一次，仅在 idle 状态"""
    async def on_hour(event):
        if not hasattr(state, 'thinking_evolver') or not state.thinking_evolver:
            return
        idle = time.time() - state.last_interaction_time
        if idle < 3600:
            return  # 非 idle 状态跳过
        try:
            ev_events = state.thinking_evolver.evolve()
            if ev_events:
                await event_bus.publish(Event(
                    type="thinking.evolved",
                    payload={"events": ev_events},
                    source="thinking_evolver", priority=Priority.LOW,
                ))
        except Exception as e:
            await event_bus.publish(Event(
                type="task.failed",
                payload={"plugin": "thinking_evolver", "error": str(e)},
                source="thinking_evolver", priority=Priority.HIGH,
            ))

    hook_manager.register(HookRegistration(
        plugin_id="thinking_evolver", event_type="heartbeat.hour",
        callback=on_hour, timeout=30.0,
    ))


def register_milestone_hooks(event_bus, hook_manager, state):
    """里程碑: 每 300 秒"""
    counter = 0

    async def on_minute(event):
        nonlocal counter
        counter += 1
        if counter < 5:
            return
        counter = 0
        # milestone logic from original milestone_loop (read from server.py)
        ...

    hook_manager.register(HookRegistration(
        plugin_id="milestones", event_type="heartbeat.minute",
        callback=on_minute, timeout=10.0,
    ))
```

---

## 任务 5：Health Monitor（ADR-006）

**文件**：`morn_core/eventbus/health_monitor.py`（新建）

实现内核健康监控的三层机制：

```python
class HealthMonitor:
    """内核健康监控：自检测 + 时钟跳变检测 + systemd watchdog"""

    def __init__(self, event_bus: EventBus, hook_manager: HookManager, state):
        self._event_bus = event_bus
        self._state = state

    async def self_check(self, event: Event) -> None:
        """每 60 秒执行一次自我 tick"""
        warnings = []

        # 1. 检查事件队列深度
        for priority, queue in [
            ("high", self._event_bus._queues[Priority.HIGH]),
            ("med", self._event_bus._queues[Priority.MEDIUM]),
            ("low", self._event_bus._queues[Priority.LOW]),
        ]:
            depth = queue.qsize()
            if depth > 50:
                warnings.append(f"queue.{priority}: {depth} events pending")

        # 2. 检查内存使用
        try:
            import psutil
            rss = psutil.Process().memory_info().rss
            mem_mb = rss / 1024 / 1024
            if mem_mb > 500:
                warnings.append(f"memory: {mem_mb:.0f}MB")
        except Exception:
            pass

        # 3. 发布健康警告
        if warnings:
            await self._event_bus.publish(Event(
                type="kernel.health_warning",
                payload={"warnings": warnings},
                source="health_monitor",
                priority=Priority.HIGH,
            ))

    async def detect_clock_jump(self, event: Event) -> None:
        """检测时钟跳变（比较 monotonic 与 RTC）"""
        import time
        now = time.time()
        # 检查 time.time() 是否突然变化
        if hasattr(self, '_last_time'):
            diff = abs(now - self._last_time)
            if diff > 5.0:
                await self._event_bus.publish(Event(
                    type="kernel.health_warning",
                    payload={"clock_jump": diff, "message": f"clock jump detected: {diff:.1f}s"},
                    source="health_monitor",
                    priority=Priority.HIGH,
                ))
        self._last_time = now
```

HealthMonitor 通过 HookManager 注册到 `heartbeat.minute` 事件（每 60 秒自我检查）和 `heartbeat.tick` 事件（每秒检测时钟跳变）。

---

## 任务 6：测试

**文件**：`tests/test_eventbus_integration.py`（新建）

1. **插件注册测试**：验证 `register_all_plugin_hooks()` 成功注册所有插件，不抛出异常
2. **HookManager 生命周期**：注册 → 暂停 → 恢复 → 取消注册，每一步验证状态正确
3. **Memory capsule_written 事件**：写入胶囊后验证 `memory.capsule_written` 被发布
4. **HealthMonitor 自检测**：模拟深队列，验证 `kernel.health_warning` 被发布
5. **Server 集成测试**：验证 server.py 初始化时注册了所有插件 Hook，不再创建独立的循环任务
6. **插件未初始化场景**：验证 `state.dream_engine = None` 时 on_minute 安全跳过

---

## 验收标准

1. ✅ `pytest tests/test_eventbus_integration.py -v` 全部通过
2. ✅ `pytest tests/test_eventbus.py -v` 仍全部通过（不破坏已有功能）
3. ✅ `pytest tests/ -x -q --timeout=120` 无回归（或已跑通的子集通过）
4. ✅ server.py 不再包含 dream_loop / identity_loop / self_pruning_loop / bond_update_loop / intent_drift_loop / audit_loop / thinking_evolution_loop / milestone_loop 的独立 while 循环
5. ✅ 所有插件通过 `register_all_plugin_hooks()` 注册到 HookManager
6. ✅ MemoryStore 写入胶囊时 publish `memory.capsule_written` 事件
7. ✅ HealthMonitor 每 60 秒自检测，触发 warning 时发布 `kernel.health_warning`
8. ✅ `import` 完整性：所有新的 hook 注册函数可以被成功导入
