# Morn API 参考

> 版本 0.1.0 · 基于 morn-core

---

## 内核（morn.kernel）

框架核心，零额外依赖。

### EventBus

中央事件总线。发布/订阅异步通信，所有插件通过此总线交互。

```python
from morn import EventBus

bus = EventBus()
await bus.start()

# 订阅
bus.subscribe("heartbeat.tick", callback, "my_listener")

# 发布
await bus.publish(Event(
    type="memory.capsule_written",
    payload={"event_id": "xxx"},
    source="memory_core",
    priority=Priority.MEDIUM,
))
```

| 方法 | 说明 |
|------|------|
| `start()` | 启动事件总线 |
| `stop()` | 停止事件总线 |
| `subscribe(type, callback, subscriber_id)` | 订阅事件类型 |
| `unsubscribe(subscriber_id)` | 取消订阅 |
| `publish(event)` | 发布事件 |
| `get_stats()` | 获取总线统计 |

### Event

```python
Event(type: str, payload: dict, source: str, priority: Priority)
```

### Priority

```python
Priority.HIGH    # 立即处理
Priority.MEDIUM  # 正常优先级
Priority.LOW     # 可延迟
```

### HookManager

钩子管理器，管理插件的 EventBus 订阅生命周期。

```python
from morn import HookManager, HookRegistration

manager = HookManager(event_bus)
manager.register(HookRegistration(
    plugin_id="my_plugin",
    event_type="heartbeat.minute",
    callback=on_minute,
    timeout=10.0,
))
```

### SecurityValidator

安全验证器，纯函数无状态，硬编码规则集。

```python
from morn import SecurityValidator

validator = SecurityValidator(config)
result = validator.validate(action, params)
# result.verdict == "allow" | "block"
```

### PluginLoader

插件加载生命周期管理。

```python
from morn import PluginLoader

loader = PluginLoader(event_bus, hook_manager)
await loader.load(MyPlugin)
await loader.unload("my_plugin")
loader.list_plugins()
```

### MornPlugin

插件抽象基类。所有插件继承此类。

```python
from morn import MornPlugin, PluginContext

class MyPlugin(MornPlugin):
    plugin_id = "my_plugin"
    name = "我的插件"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    
    async def on_load(self, context: PluginContext):
        pass
```

| 钩子方法 | 触发时机 |
|----------|---------|
| `on_load(context)` | 插件加载时 |
| `on_unload()` | 插件卸载时 |
| `on_event(event)` | 有订阅的事件时 |
| `on_heartbeat(tick)` | 每次心跳（如果 `needs_periodic_trigger=True`） |
| `on_chat(message)` | 有对话时 |

### PluginContext

```python
PluginContext(
    event_bus=...,
    hook_manager=...,
    config={},
    data_dir=None,
)
```

---

## SDK 服务层（morn.sdk）

通过 `from morn import ...` 直接访问。

### ChatEngine

对话引擎，云端大模型 + 本地兜底混合路由。

```python
from morn import ChatEngine

engine = ChatEngine(
    instance_name="my_morn",
    memory_store=store,
    config=config,
)
reply = await engine.chat("你好")
```

### MemoryStore

记忆存储，L1 工作记忆 + L2 情景记忆。

```python
from morn import MemoryStore

store = MemoryStore(data_dir, enable_encryption=True)
await store.__aenter__()
await store.add_capsule("event_1", "对话内容", entities=["创建者"])
capsule = await store.get_capsule("event_1")
results = await store.search("关键词")
```

### SecurityLayer

安全层，用户保护 + 外部边界检查。

```python
from morn import SecurityLayer

protection = UserProtection(event_bus=bus)
boundary = ExternalBoundary(data_dir, event_bus=bus)
```

### MornPresence

存在形式基类。所有对话界面通过此类接入。

```python
from morn import MornPresence

class MyPresence(MornPresence):
    name = "my_presence"
    
    async def start(self): ...
    async def stop(self): ...
    async def send_message(self, text): ...
```

---

## 内置插件（morn.plugins）

| 插件类 | plugin_id | 等级 | 说明 |
|--------|-----------|------|------|
| HealthMonitorPlugin | health_monitor | **S** | 系统健康监控 |
| IdentityAffirmerPlugin | identity_affirmer | A | 身份确认 |
| SelfPrunerPlugin | self_pruner | A | 自我瘦身 |
| BondTrackerPlugin | bond_tracker | A | 纽带追踪 |
| IntentDriftPlugin | intent_drift | A | 意图漂移检测 |
| AuditPlugin | audit | A | 安全审计 |
| ThinkingEvolutionPlugin | thinking_evolution | A | 思维风格进化 |
| MilestonePlugin | milestones | A | 里程碑追踪 |
| DreamEnginePlugin | dream_engine | **B** | 梦境引擎 |
| SelfReflectionPlugin | self_reflection | **B** | 自省循环 |
| HindsightPlugin | hindsight | **B** | 后见之明 |

---

## 高级组件（morn.contrib）

| 导入路径 | 说明 |
|----------|------|
| `morn.contrib.memory_advanced` | L3/L4 记忆、图谱、外部记忆 |
| `morn.contrib.security_advanced` | 动态权限、意图检测、APZ 隐私区 |

---

## CLI

```bash
morn --instance <name>        # 启动实例
morn-cli                      # 新入口（可选）
```
