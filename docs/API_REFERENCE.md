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
| `health_check()` | 定期健康检查 |

### MornPlugin 类属性

| 属性 | 类型 | 说明 |
|------|------|------|
| `plugin_id` | `str` | 唯一标识 |
| `name` | `str` | 可读名称 |
| `version` | `str` | 语义化版本 |
| `dependencies` | `list[PluginDependency]` | 依赖插件列表 |
| `required_permissions` | `list[str]` | 必要权限 |
| `optional_permissions` | `list[str]` | 可选权限 |
| `needs_periodic_trigger` | `bool` | 是否需要周期性触发 |
| `usage_hint` | `str` | 资源消耗提示 |
| `health_check_interval` | `int` | 健康检查间隔（秒） |
| `capabilities` | `list[dict]` | MCP 能力列表 |

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

## 核心组件（morn.core）

### EventLog

事件日志，记录所有通过 EventBus 发布的事件。

```python
from morn.core.event_log import EventLog

log = EventLog(event_bus, max_entries=1000)
await log.start()

# 查询
recent = log.recent(10)
by_type = log.filter_by_type("heartbeat.minute")
```

| 方法 | 说明 |
|------|------|
| `start()` | 开始监听事件总线 |
| `stop()` | 停止监听 |
| `recent(n)` | 返回最近 n 条事件 |
| `filter_by_type(type)` | 按类型过滤事件 |
| `clear()` | 清空日志 |

### PluginContract

插件 YAML 契约解析，将插件类属性映射为结构化契约。

```python
from morn.core.plugin_contract import PluginContract, parse_contract

contract = parse_contract(MyPlugin)
# contract.plugin_id, contract.dependencies, contract.permissions, ...
```

| 方法 | 说明 |
|------|------|
| `parse_contract(plugin_cls)` | 从插件类解析契约 |
| `contract.to_dict()` | 序列化为字典 |
| `contract.validate()` | 验证契约完整性 |

### ConfigWatcher

配置文件热重载监视器。

```python
from morn.core.config_watcher import ConfigWatcher

watcher = ConfigWatcher("path/to/config.yaml", callback=on_change)
await watcher.start()
```

| 方法 | 说明 |
|------|------|
| `start()` | 开始监视文件变更 |
| `stop()` | 停止监视 |
| `get_config()` | 获取当前配置快照 |

### MCPServer

MCP Server 管理器，自动将插件注册为 MCP 端点。

```python
from morn.core.mcp_server import MCPServer

mcp = MCPServer(enabled=True)
mcp.register_plugin(my_plugin)
await mcp.start()
```

| 方法 | 说明 |
|------|------|
| `start()` | 启动 MCP Server |
| `stop()` | 停止 MCP Server |
| `register_plugin(plugin)` | 手动注册插件端点 |
| `unregister_plugin(plugin_id)` | 注销插件端点 |

### Sandbox

进程级沙箱，支持分级隔离。

```python
from morn.core.sandbox import Sandbox, SandboxLevel

# SandboxLevel.NONE     — 无限制
# SandboxLevel.RESTRICTED — 受限文件系统 + 网络白名单
# SandboxLevel.SANDBOXED  — 隔离文件系统 + 禁止网络 + seccomp

sandbox = Sandbox(level=SandboxLevel.SANDBOXED)
sandbox.apply()  # 调用 seccomp 应用规则
```

| 方法 | 说明 |
|------|------|
| `apply()` | 应用沙箱规则 |
| `apply_filesystem_restrictions()` | 应用文件系统限制 |
| `apply_seccomp_filter()` | 应用 seccomp BPF 过滤 |

### ResourceQuota

资源配额管理，跟踪 token 和内存使用。

```python
from morn.core.resource_quota import TokenCounter, QuotaManager, QuotaExceeded

manager = QuotaManager()
counter = TokenCounter(limit=100000)

# 检查配额
quota = manager.get_quota("my_plugin")
# quota.tokens_remaining, quota.memory_bytes_remaining

# 消费 token
if counter.consume(500):
    print("token 足够")
else:
    raise QuotaExceeded("token 不足")
```

| 类 | 说明 |
|----|------|
| `TokenCounter` | Token 计数器，带速率限制 |
| `QuotaManager` | 全局配额管理器 |
| `QuotaExceeded` | 配额超限异常 |

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
