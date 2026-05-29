# Morn v0.4 事件总线实现

## 编程原则

### 14条核心准则
1. **Think Before Coding** — 先想后写，改之前先理解整体结构
2. **Simplicity First** — 简单优先，不加不必要的抽象层
3. **Surgical Changes** — 一次一事，一个PR/commit只做一件事
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
- 依赖倒置：模块通过接口/ID通信，不直接耦合

### 执行方式
- 新文件用 opencode 创建，修改已有文件也走 opencode
- 每轮结束后跑全量测试确认无回归
- 不要手写代码——所有修改通过 opencode run 执行

---

## 任务 1：EventBus 核心类

**文件**：`morn_core/eventbus/bus.py`（新建目录和文件）

实现一个异步事件总线，核心类 `EventBus`：

```python
class Priority(enum.Enum):
    HIGH = 0    # kernel.health_warning, task.failed, security.alert
    MEDIUM = 1  # memory.capsule_written, emotion.tick, emotion.on_event
    LOW = 2     # evolution.grow_skill, self_reflection.completed, event.dropped

@dataclass
class Event:
    type: str                # 事件类型标识，如 "memory.capsule_written"
    payload: dict            # 事件数据
    source: str              # 发布者标识，如 "memory_core"
    priority: Priority       # 优先级
    timestamp: float         # time.time() 创建时间戳
    event_id: str            # uuid4 hex，唯一标识
```

### EventBus 接口

```python
class EventBus:
    def __init__(self, loop: asyncio.AbstractEventLoop):
        # 三个独立的 asyncio.Queue，每个通道一个
        self._queues = {Priority.HIGH: asyncio.Queue(),
                        Priority.MEDIUM: asyncio.Queue(),
                        Priority.LOW: asyncio.Queue()}
        self._subscribers: dict[str, list[SubscriberInfo]] = {}  # event_type → listeners
        self._running = False
        self._stats = BusStats()
        self._backpressure = {}  # subscriber_id → 背压状态

    async def publish(self, event: Event) -> None:
        """将事件放入对应优先级的队列"""
        ...

    def subscribe(self, event_type: str, callback: Callable, subscriber_id: str,
                  priority_filter: Optional[Priority] = None) -> None:
        """注册事件处理函数"""
        ...

    def unsubscribe(self, event_type: str, subscriber_id: str) -> None:
        """取消注册"""
        ...

    async def start(self) -> None:
        """启动调度器——从队列中取出事件并派发给订阅者"""
        ...

    async def stop(self) -> None:
        """停止调度器"""
        ...

    def get_stats(self) -> dict:
        """返回统计信息：各队列深度、发布总数、消费总数、丢弃数"""
        ...
```

### 调度逻辑

- 高优先级队列优先出队，高队列空后出中队列，中队列空后出低队列
- 同优先级内严格 FIFO
- 每个事件发布给所有订阅了该 event_type 的 callback
- 每个 callback 包装在 `asyncio.wait_for` 中（timeout 默认 500ms），超时后 cancel task 并发布 `task.failed` 事件（不阻塞其他订阅者）

### 超时与背压（ADR-001）

- `SubscriberInfo` 包含 `consecutive_timeout_count` 字段
- 每次 callback 超时 → count++；正常完成 → count = 0
- 连续 3 个心跳周期中有 2 次超时（背压触发条件）→ 发布 `event.dropped` 事件并**不再分发新事件给该 subscriber** 直到其恢复
- 每个队列独立监控：若队列积压 60 秒以上（对比发布和消费速率）→ 丢弃该通道的旧事件，**仅保留最新 100 条**
- 丢弃时发布 `event.dropped` 事件（含通道 ID、丢弃数量）

### 系统预置事件类型

```
kernel.health_warning    → HIGH  内核自检测发现问题
task.failed              → HIGH  某个 Hook 超时或被取消
security.alert           → HIGH  安全验证器拦截到越权操作
event.dropped            → HIGH  事件因背压被丢弃
memory.capsule_written   → MEDIUM 新事件胶囊写入
emotion.tick             → MEDIUM 情感状态机定时更新
emotion.on_event         → MEDIUM 情感状态机事件触发
evolution.grow_skill     → LOW   技能自生长
self_reflection.completed → LOW  自省循环完成
```

---

## 任务 2：Heartbeat 集成事件总线

**文件**：重构 `morn_core/heartbeat.py`

当前 heartbeat.py 是一个简单的 1Hz while 循环，没有任何事件机制。

### 重构后行为

```python
async def heartbeat_loop(state, event_bus: EventBus):
    while not state.shutdown:
        target_time = time.monotonic() + 1
        state.heartbeat_count += 1
        state.last_heartbeat = time.monotonic()

        # 发布心跳事件——插件通过 subscribe("heartbeat.tick") 接收
        await event_bus.publish(Event(
            type="heartbeat.tick",
            payload={"count": state.heartbeat_count},
            source="kernel",
            priority=Priority.HIGH,
            timestamp=time.time(),
        ))

        if state.heartbeat_count % 60 == 0:
            await event_bus.publish(Event(
                type="heartbeat.minute",
                payload={"count": state.heartbeat_count},
                source="kernel",
                priority=Priority.MEDIUM,
                timestamp=time.time(),
            ))

        if state.heartbeat_count % 3600 == 0:
            await event_bus.publish(Event(
                type="heartbeat.hour",
                payload={"count": state.heartbeat_count},
                source="kernel",
                priority=Priority.MEDIUM,
                timestamp=time.time(),
            ))

        # 原来日志逻辑保留
        if state.heartbeat_count % 60 == 0:
            state.log("heartbeat", f"#{state.heartbeat_count} alive")

        await asyncio.sleep(max(0, target_time - time.monotonic()))
```

- `memory_monitor` 和 `wal_checkpoint` 保持不变——它们不是事件驱动的，而是独立的异步协程
- 所有现有的 `on_tick` / `on_minute` / `on_hour` 替换为事件订阅机制

### 事件类型新增

```
heartbeat.tick       → HIGH  每秒触发
heartbeat.minute     → MEDIUM 每60秒触发
heartbeat.hour       → MEDIUM 每3600秒触发
```

---

## 任务 3：Server 集成 EventBus

**文件**：修改 `morn_core/server.py`

### 改动内容

1. 在 `async def main()` 中创建 `EventBus` 实例
2. 启动时调用 `event_bus.start()`
3. 停止时调用 `event_bus.stop()`
4. 将 event_bus 实例传递给需要发布或订阅事件的组件
5. `heartbeat_loop` 的调用签名增加 `event_bus` 参数
6. 原有直接 import 调用逐步改为事件驱动（第一阶段: 只集成 EventBus 框架 + heartbeat，不改插件注册方式）

### 具体代码变化

```python
from morn_core.eventbus.bus import EventBus, Event, Priority

async def main():
    loop = asyncio.get_event_loop()
    event_bus = EventBus(loop)

    # 启动事件总线
    await event_bus.start()

    # 启动心跳（传入 event_bus）
    tasks = [
        asyncio.create_task(heartbeat_loop(state, event_bus)),
        asyncio.create_task(memory_monitor(state)),
        asyncio.create_task(wal_checkpoint(state)),
        # 其他现有任务保持不变...
    ]
```

---

## 任务 4：测试

**文件**：`tests/test_eventbus.py`

覆盖以下场景：

1. **发布订阅基本功能**：发布一个事件，验证订阅者收到正确数据
2. **多订阅者**：同一事件类型有 3 个订阅者，全部收到
3. **优先级出队**：高优先级事件在中/低优先级事件之后发布，但先被消费
4. **超时取消**：订阅者 callback 耗时 > 500ms，被取消并发布 `task.failed` 事件
5. **背压丢弃**：模拟消费慢，队列积压超过阈值，验证旧事件被丢弃、保留最新 100 条
6. **连续超时降频**：模拟 3 周期 2 次超时，验证订阅者被暂停分发
7. **取消订阅**：unsubscribe 后不再收到事件
8. **心跳集成**：验证 heartbeat_loop 发布 `heartbeat.tick` 事件，订阅者收到
9. **背压事件通知**：事件被丢弃时，验证 `event.dropped` 被发布
10. **并发安全**：多个协程同时 publish，无竞争条件

---

## 任务 5：Heartbeat Hook 注册机制（v0.4 基础框架）

**文件**：`morn_core/eventbus/hooks.py`（新建）

### 接口

```python
@dataclass
class HookRegistration:
    plugin_id: str
    event_type: str          # 要订阅的事件类型，如 "heartbeat.tick"
    callback: Callable       # async def callback(event: Event) → None
    timeout: float = 0.5    # 默认 500ms
    enabled: bool = True

class HookManager:
    """管理插件的 Hook 注册——包装 EventBus.subscribe 并添加降频逻辑"""

    def __init__(self, event_bus: EventBus):
        self._event_bus = event_bus
        self._registrations: dict[str, HookRegistration] = {}

    def register(self, hook: HookRegistration) -> None:
        """注册一个 Hook。如果已有同 plugin_id+event_type 的注册则替换。"""
        ...

    def unregister(self, plugin_id: str, event_type: str) -> None:
        """取消注册"""
        ...

    def get_due_frequency(self, plugin_id: str) -> str:
        """返回插件当前被分配的触发频率: 'tick', 'minute', 'hour'
           根据该插件的超时历史动态调整（背压逻辑）"""
        ...

    def pause_plugin(self, plugin_id: str) -> None:
        """暂停某插件所有 Hook"""
        ...

    def resume_plugin(self, plugin_id: str) -> None:
        """恢复某插件所有 Hook"""
        ...
```

### 说明

- `HookManager` 包装 `EventBus.subscribe`，在高频事件（如 `heartbeat.tick`）的 callback 中嵌入超时检测逻辑
- 维护每个插件的 `consecutive_timeout_count`，提供给 `get_due_frequency()` 做降频决策
- 被暂停的插件 Hook 不再分发事件，但注册信息保留
- 这是 v0.4 插件管理器基础框架的一部分——当前版本不实现动态加载，只实现 Hook 注册和调度

---

## 验收标准

1. ✅ `pytest tests/test_eventbus.py -v` 全部通过（至少 10 个测试）
2. ✅ `pytest tests/ -x -q` 全量测试无回归（当前 1118 个）
3. ✅ 新文件不超过 300 行（bus.py + hooks.py 各自 ≤300）
4. ✅ 心跳仍正常工作（`heartbeat_loop` 每秒一次，`heartbeat.minute` 每 60 秒一次）
5. ✅ EventBus 的 start/stop 生命周期与 MornState.shutdown 协调
6. ✅ 背压丢弃事件时日志有记录
