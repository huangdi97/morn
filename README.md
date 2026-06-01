<p align="center">
  <h1 align="center">Morn — 数字生命框架 / Agent OS</h1>
  <p align="center">开源 Agent 操作系统框架 · 零 AI 内核 · 事件驱动 · 插件生态 · 隐私优先</p>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/python-3.11%20%7C%203.12-blue" alt="Python">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
  <img src="https://img.shields.io/github/actions/workflow/status/huangdi97/morn/.github/workflows/ci.yml?branch=main" alt="CI">
</p>

---

Morn 是一个开源的 **Agent 操作系统框架**。它的内核不包含任何认知能力，出厂时只是一个能记住对话、能聊天、能保护隐私的最小系统。所有智能行为通过插件热插拔实现。

```python
from morn import EventBus, ChatEngine, MemoryStore

bus = EventBus()
store = MemoryStore("./my_morn")
chat = ChatEngine(store=store, bus=bus)
```

### 核心特点

- **零 AI 内核** — 内核无内置 AI 能力，所有认知通过插件注入
- **事件驱动** — 基于 EventBus 的发布/订阅架构，模块间松耦合
- **插件生态** — S/A/B/C 四级插件体系，支持热加载/卸载
- **隐私优先** — 本地记忆存储，数据加密，权限最小化

---

## 架构概览

Morn 采用三层架构设计：

| 层 | 说明 |
|----|------|
| **Kernel（内核）** | 零外部依赖。事件总线 (`EventBus`) + 记忆存储 (`MemoryStore`) + 对话引擎 (`ChatEngine`) + 安全层 (`SecurityLayer`) |
| **SDK** | 开发者工具包。提供 `MornPlugin` 基类、`PluginContext`、`HookManager`，用于构建插件和应用 |
| **Plugins（插件生态）** | 插件系统。内置 S/A/B 级插件 + 第三方插件，通过事件总线与内核交互 |

---

## 快速开始

### 安装

```bash
pip install morn
```

### 启动实例

```bash
morn --instance my-first
```

启动后自动进入 CLI 对话界面。输入 `/status` 查看状态，`/shutdown` 退出。

### 验证安装

```python
from morn import EventBus, ChatEngine, MemoryStore
print("Morn 已就绪")
```

### 运行插件

在 `config.json` 中启用内置插件：

```json
{
  "bond_tracker_enabled": true,
  "thinking_evolution_enabled": true
}
```

更多用法参见 [快速开始指南](docs/MORN_QUICKSTART.md)。

---

## 插件生态

### 已有插件

| 插件 | plugin_id | 等级 | 说明 |
|------|-----------|------|------|
| HealthMonitor | `health_monitor` | **S** | 系统健康监控 |
| IdentityAffirmer | `identity_affirmer` | A | 身份确认 |
| BondTracker | `bond_tracker` | A | 纽带追踪 |
| IntentDrift | `intent_drift` | A | 意图漂移检测 |
| ThinkingEvolution | `thinking_evolution` | A | 思维风格进化 |
| DreamEngine | `dream_engine` | **B** | 梦境引擎 |
| SelfReflection | `self_reflection` | **B** | 自省循环 |

### 第三方插件

- `morn-presence-telegram` — Telegram Presence 插件，通过 Telegram Bot 与 Morn 对话

### 开发自己的插件

插件开发非常简单：继承 `MornPlugin`，实现 `on_load`/`on_unload` 钩子，注册事件订阅即可。

```python
from morn import MornPlugin, PluginContext, HookRegistration

class MyPlugin(MornPlugin):
    plugin_id = "my_plugin"
    name = "我的插件"
    version = "0.1.0"
    plugin_class = "B"

    async def on_load(self, context: PluginContext):
        async def on_event(event):
            print(f"收到事件: {event.type}")

        context.hook_manager.register(HookRegistration(
            plugin_id=self.plugin_id,
            event_type="heartbeat.tick",
            callback=on_event,
        ))
```

详细教程见 [插件开发指南](docs/PLUGIN_DEV_GUIDE.md)。

---

## 文档索引

| 文档 | 说明 |
|------|------|
| [快速开始指南](docs/MORN_QUICKSTART.md) | 从零搭建你的第一个 Morn 实例 |
| [API 参考](docs/API_REFERENCE.md) | 完整类、函数、CLI 清单 |
| [插件开发指南](docs/PLUGIN_DEV_GUIDE.md) | 编写、打包、分发你自己的 Morn 插件 |

---

## 许可证

[MIT](LICENSE)