# Morn 插件开发指南

> 教你编写、打包、分发自己的 Morn 插件。

---

## 1. 基本结构

每个 Morn 插件是一个继承 `MornPlugin` 的 Python 类：

```python
from morn import MornPlugin, PluginContext, HookRegistration


class GreetingPlugin(MornPlugin):
    plugin_id = "auto_greeting"           # 唯一标识，全局唯一
    name = "自动问候"                       # 可读名称
    version = "0.1.0"                     # 语义化版本
    plugin_class = "B"                     # A / B / C（见下方等级说明）
    needs_periodic_trigger = True          # 是否需要周期性唤醒
    usage_hint = "low"                     # low / medium / high 资源提示
    required_permissions = ["memory.read"]  # 必要权限
    optional_permissions = []              # 可选权限（降级运行）
    
    async def on_load(self, context: PluginContext):
        """插件加载时调用。在此注册事件钩子。"""
        # context.event_bus    — 事件总线引用
        # context.hook_manager — 钩子管理器
        # context.config       — 插件配置字典
        # context.data_dir     — 实例数据目录
        
        async def on_hour(event):
            if context.config.get("greeting_enabled", True):
                # 发送问候
                pass
        
        context.hook_manager.register(HookRegistration(
            plugin_id=self.plugin_id,
            event_type="heartbeat.hour",
            callback=on_hour,
        ))
    
    async def on_unload(self):
        """插件卸载时调用。清理资源。"""
        pass
```

---

## 2. 插件等级

| 等级 | 说明 | 示例 |
|------|------|------|
| **S** | 核心插件，出厂默认加载 | 记忆核心、对话核心、安全核心 |
| **A** | 官方高级插件，确定性行为 | 情感系统、进化引擎 |
| **B** | 实验插件，允许不确定性 | 梦境引擎、非最优探索 |
| **C** | 第三方/远期插件 | 社区贡献 |

---

## 3. 可用钩子

| 钩子 | 触发时机 | 说明 |
|------|---------|------|
| `on_load(context)` | 插件被加载 | 注册事件钩子的入口 |
| `on_unload()` | 插件被卸载 | 清理资源 |
| `on_event(event)` | 订阅的事件发布 | 接收 Event 对象 |
| `on_heartbeat(tick)` | 每次心跳（1Hz） | 仅当 `needs_periodic_trigger=True` |
| `on_chat(message)` | 有对话发生 | 收到用户消息文本 |

---

## 4. 事件类型

常见事件：

| 事件类型 | 触发者 | payload 示例 |
|----------|--------|-------------|
| `heartbeat.tick` | 内核 | `{}` |
| `heartbeat.minute` | 内核 | `{}` |
| `heartbeat.hour` | 内核 | `{}` |
| `memory.capsule_written` | 记忆核心 | `{"event_id": "xxx"}` |
| `security.alert` | 安全核心 | `{"type": "...", "severity": "..."}` |
| `task.failed` | 任何插件 | `{"plugin": "xxx", "error": "..."}` |

---

## 5. 完整示例：自动问候插件

```python
"""自动问候插件——每整点通过 CLI 发送问候"""
import time
from morn import MornPlugin, PluginContext, HookRegistration


class AutoGreetingPlugin(MornPlugin):
    plugin_id = "auto_greeting"
    name = "自动问候"
    version = "0.1.0"
    plugin_class = "B"
    needs_periodic_trigger = True
    usage_hint = "low"
    
    def __init__(self):
        super().__init__()
        self._last_greeting_hour = -1
    
    async def on_load(self, context: PluginContext):
        async def on_tick(event):
            current_hour = time.localtime().tm_hour
            if current_hour != self._last_greeting_hour:
                self._last_greeting_hour = current_hour
                greetings = {
                    6: "早上好！新的一天开始了 ☀️",
                    12: "中午好，该吃午饭了 🍚",
                    18: "晚上好，辛苦了 🌆",
                    22: "夜深了，早点休息 🌙",
                }
                msg = greetings.get(current_hour, f"现在是 {current_hour} 点")
                print(f"👋 {msg}")
        
        context.hook_manager.register(HookRegistration(
            plugin_id=self.plugin_id,
            event_type="heartbeat.tick",
            callback=on_tick,
        ))
```

---

## 6. 测试

```python
import pytest
from morn import MornPlugin, PluginContext


class TestGreetingPlugin:
    @pytest.mark.asyncio
    async def test_plugin_loads(self):
        plugin = AutoGreetingPlugin()
        context = PluginContext()
        await plugin.on_load(context)
        assert plugin.is_loaded
        assert plugin.plugin_id == "auto_greeting"
```

---

## 13. 最佳实践

推荐命名约定：`morn-plugin-<name>`

```
morn-plugin-auto-greeting/
├── morn_plugin_auto_greeting/
│   ├── __init__.py
│   └── plugin.py
├── pyproject.toml
└── README.md
```

`pyproject.toml` 中声明依赖 `morn-core`：

```toml
[project]
name = "morn-plugin-auto-greeting"
version = "0.1.0"
dependencies = ["morn-core>=0.1.0"]
```

发布到 PyPI：`flit publish` 或 `twine upload dist/*`

---

## 8. 插件生命周期

### YAML 契约声明

每个插件通过类属性声明其 YAML 契约。以下字段映射到 `plugin.yaml`：

| 类属性 | YAML 字段 | 说明 |
|--------|-----------|------|
| `plugin_id` | `plugin_id` | 全局唯一标识 |
| `name` | `name` | 可读名称 |
| `version` | `version` | 语义化版本 |
| `plugin_class` | `class` | S/A/B/C 等级 |
| `dependencies` | `dependencies` | 插件依赖列表 |
| `required_permissions` | `required_permissions` | 必要权限 |
| `optional_permissions` | `optional_permissions` | 可选权限 |
| `needs_periodic_trigger` | `needs_periodic_trigger` | 是否需要周期性触发 |
| `usage_hint` | `usage_hint` | 资源消耗提示 |
| `health_check_interval` | `health_check_interval` | 健康检查间隔（秒） |
| `capabilities` | `capabilities` | MCP 能力列表 |

### 生命周期钩子

| 钩子 | 触发时机 | 超时影响 |
|------|---------|---------|
| `on_load(context)` | 插件加载时 | 超时 → 加载失败 |
| `on_unload()` | 插件卸载时 | 超时 → 强制卸载 |
| `on_event(event)` | 有订阅的事件时 | 超时 → 跳过该事件 |
| `on_heartbeat(tick)` | 每次心跳（1Hz） | 超时 → 跳过该心跳 |
| `on_chat(message)` | 有对话时 | 超时 → 跳过消息 |
| `health_check()` | 每 `health_check_interval` 秒 | 超时 → 标记为不健康 |

执行顺序：`on_load` → `health_check` (定期) → `on_event` / `on_heartbeat` / `on_chat` (按需) → `on_unload`

---

## 9. 安全边界

### 权限声明

插件必须声明它需要的权限，分为必要和可选两类：

```python
class MyPlugin(MornPlugin):
    required_permissions = ["memory.read", "memory.write"]   # 缺少则加载失败
    optional_permissions = ["network.http"]                   # 缺少可降级运行
    
    async def on_load(self, context):
        if self.require_permission("network.http"):
            # 具有该权限时的逻辑
            pass
```

### 分级沙箱

| 等级 | 沙箱级别 | 说明 |
|------|---------|------|
| **S** | `NONE` | 无限制，信任核心插件 |
| **A** | `RESTRICTED` | 受限文件系统 + 网络白名单 |
| **B** | `SANDBOXED` | 隔离文件系统 + 禁止网络 |
| **C** | `SANDBOXED` | 完全沙箱 + seccomp 过滤 |

### seccomp 禁止的系统调用

- `clone` / `fork` — 禁止创建子进程
- `execve` / `execveat` — 禁止执行外部程序
- `ptrace` — 禁止进程追踪
- `mount` / `umount` — 禁止挂载操作
- `reboot` / `swapon` — 禁止系统级操作
- 非 `O_RDONLY` 的 `open` — B/C 级禁止写文件

---

## 10. MCP 集成

### 插件自动注册为 MCP Server 端点

当插件加载时，`McpServerManager` 自动将插件注册为一个 MCP Server 端点。插件的 `capabilities` 列表中的每个条目映射为一个 MCP `tool`。

```python
capabilities = [
    {"name": "example.say_hello", "description": "返回问候语"},
    {"name": "example.count", "description": "计数"},
]
```

### 调用流程

```
外部 MCP 客户端 → McpServerManager → 查找目标插件 → 发布 Event → 插件 on_event 处理
```

插件通过订阅特定事件类型来响应 MCP 调用。事件类型与 `capabilities[].name` 一致。

---

## 11. 资源管理

### Token 配额

每个插件有一个 token 配额，由 `QuotaManager` 统一管理。配额按以下维度分配：

| 维度 | 默认配额 | 说明 |
|------|---------|------|
| 每日 token | 100,000 | 所有 LLM 调用合计 |
| 每分钟 token | 10,000 | 突发限制 |
| 内存 | 256 MB | 插件 RSS 上限 |

### 检查剩余配额

```python
quota = context.quota_manager.get_quota("my_plugin")
# quota.tokens_remaining, quota.memory_bytes_remaining
```

### 超限降级

当配额超限时，`QuotaManager` 发布 `resource.quota_exceeded` 事件，插件应：

1. 降低事件处理频率
2. 跳过非关键处理
3. 等待配额重置后恢复

---

## 12. 开发示例

完整代码参考 `morn/plugins/example_hello.py`。以下是从零编写插件的最小框架：

```python
"""我的第一个 Morn 插件"""
from morn.core.plugin import MornPlugin, PluginContext, PluginDependency
from morn.core.hooks import HookRegistration
from morn.core.bus import Event


class MyFirstPlugin(MornPlugin):
    plugin_id = "my_first_plugin"
    name = "我的第一个插件"
    version = "0.1.0"
    plugin_class = "C"
    dependencies = []
    required_permissions = []
    optional_permissions = []
    needs_periodic_trigger = False
    usage_hint = "low"
    health_check_interval = 60
    capabilities = [
        {"name": "my_plugin.hello", "description": "返回问候"},
    ]

    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        print(f"{self.plugin_id} loaded")

        context.hook_manager.register(HookRegistration(
            plugin_id=self.plugin_id,
            event_type="my_plugin.hello",
            callback=self.on_event,
            timeout=5.0,
        ))

    async def on_unload(self):
        print(f"{self.plugin_id} unloaded")
        await super().on_unload()

    async def on_event(self, event: Event):
        print(f"收到事件: {event.type} payload={event.payload}")

    async def health_check(self) -> bool:
        return True
```

---

- **不阻塞**：插件钩子应该是异步的，不要在钩子中执行耗时的同步操作
- **错误处理**：捕获所有异常，避免插件崩溃影响内核
- **权限最小化**：只声明需要的权限，不要申请 `memory.write` 如果你只需要 `memory.read`
- **向后兼容**：`version` 遵循语义化版本，大版本变更时考虑同时影响 `plugin_class`
