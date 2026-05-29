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

## 7. 打包分发

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

## 8. 最佳实践

- **不阻塞**：插件钩子应该是异步的，不要在钩子中执行耗时的同步操作
- **错误处理**：捕获所有异常，避免插件崩溃影响内核
- **权限最小化**：只声明需要的权限，不要申请 `memory.write` 如果你只需要 `memory.read`
- **向后兼容**：`version` 遵循语义化版本，大版本变更时考虑同时影响 `plugin_class`
