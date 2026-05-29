# Morn Phase 3：文档 + PyPI 发布

## 概述

最后阶段，将 Morn 框架的开发者体验做完。产出三部份：

1. **开发者文档** — Quickstart + API 参考 + 插件开发指南 + 部署指南
2. **依赖拆分** — 将重型运行时依赖（aiogram, chromadb, ollama）从核心包移至可选 extras
3. **PyPI 发布准备工作** — 检查所有条件，但不真正发布到 PyPI（需确认）

---

## 任务 A：开发者文档 QUICKSTART.md

创建 `MORN_QUICKSTART.md`（不放在 morn/ 下，放在 mornd/ 根目录作为独立文档）。纯文档任务，直接手动创建。

标题：`Morn 数字生命框架 · 快速开始`

内容结构：

1. **安装**：`pip install -e .`（后续改为 `pip install morn`）
2. **3 行创建第一个数字生命**：
```python
from morn import EventBus, ChatEngine, MemoryStore, SecurityLayer

bus = EventBus()
store = MemoryStore("./my_morn")
security = SecurityLayer()
chat = ChatEngine(store=store, security=security)
chat.attach(bus)
```
3. **启动完整实例**：`morn --instance my_first_morn`
4. **通过 Telegram 对话**：配置 `config.json` 中的 telegram_token
5. **下一步**：加载高级插件、配置情感、部署到服务器

注意：这篇文档用 .md 格式，输出纯文字，不要用代码块之外的复杂格式。

---

## 任务 B：API 参考文档 API_REFERENCE.md

新建 `API_REFERENCE.md`，列出 Morn 公开 API：

### 内核（morn.kernel）

| 类/函数 | 说明 |
|---------|------|
| EventBus | 中央事件总线，发布/订阅 |
| Event | 事件对象 |
| Priority | 优先级枚举 (HIGH/MEDIUM/LOW) |
| HookManager | 钩子管理器 |
| HookRegistration | 钩子注册记录 |
| SecurityValidator | 安全验证器（纯函数无状态） |
| PluginLoader | 插件加载器 |
| MornPlugin | 插件抽象基类 |
| PluginContext | 插件运行时上下文 |

### SDK 服务层（morn.sdk / morn）

| 类 | 说明 |
|----|------|
| ChatEngine | 对话引擎（云端 + 本地混合路由） |
| MemoryStore | 记忆存储（L1+L2） |
| SecurityLayer | 安全层（用户保护 + 外部边界） |
| MornPresence | 存在形式基类 |
| CLIPresence | CLI 对话界面 |

### 内置插件（morn.plugins）

列出全部 11 个插件及其 plugin_id / class 等级。

---

## 任务 C：插件开发指南 PLUGIN_DEV_GUIDE.md

新建 `PLUGIN_DEV_GUIDE.md`，教第三方开发者如何写一个 Morn 插件。

内容：

1. **继承 MornPlugin**
2. **声明插件元数据**（plugin_id, name, version, plugin_class, permissions）
3. **实现 on_load** — 接收 PluginContext，注册钩子
4. **实现 on_heartbeat / on_event / on_chat**
5. **打包分发**：`morn-plugin-xxx` 命名约定，pip 安装
6. **完整示例**：写一个"自动问候"插件

示例代码：

```python
from morn import MornPlugin, PluginContext, HookRegistration

class GreetingPlugin(MornPlugin):
    plugin_id = "auto_greeting"
    name = "自动问候"
    version = "0.1.0"
    plugin_class = "B"
    needs_periodic_trigger = True
    required_permissions = ["memory.read"]
    
    async def on_load(self, context: PluginContext):
        async def on_hour(event):
            if context.config.get("greeting_enabled", True):
                print("👋 你好，创建者！")
        context.hook_manager.register(HookRegistration(
            plugin_id=self.plugin_id,
            event_type="heartbeat.hour",
            callback=on_hour,
        ))
```

---

## 任务 D：依赖拆分（pyproject.toml）

当前 `pyproject.toml` 将所有依赖列为必装。需要把重型外部依赖拆为可选：

**当前依赖问题：**
```toml
dependencies = [
    "aiohttp>=3.9",
    "aiogram>=3.0",    # ← Telegram bot 依赖（不是每个人都要）
    "aiosqlite>=0.20",
    "chromadb>=0.5.0", # ← 向量数据库（S 级最小系统需要？）
    "ollama>=0.3",     # ← 本地 LLM（不是每个人都要）
    "pydantic>=2.0",
    "pycryptodome>=3.20",
    "psutil>=5.9",
    "matplotlib>=3.8", # ← 绘图（不是每个人都要）
]
```

**拆分方案：**

```toml
# 核心依赖（所有实例必须）
dependencies = [
    "aiohttp>=3.9",
    "aiosqlite>=0.20",
    "pydantic>=2.0",
    "pycryptodome>=3.20",
    "psutil>=5.9",
]

# 可选 extras
[project.optional-dependencies]
telegram = ["aiogram>=3.0"]         # Telegram Presence
local-llm = ["ollama>=0.3"]         # 本地 Ollama 推理
vector = ["chromadb>=0.5.0"]        # ChromaDB 向量检索
all = ["morn-core[telegram,local-llm,vector]"]  # 一键安装全部
```

注意：只修改 pyproject.toml 的 `[project]` 部分的依赖列表，`[project.scripts]` 和其他部分不动。

---

## 任务 E：验证 + 清理

**E1** 确认所有文档排版正确，README.md 更新：
在 README.md 添加指向新文档的链接和安装说明更新。

**E2** 验证 `pip install -e .` 后：
- `python -c "import morn; print(morn.__version__)"` 可用
- `python -c "from morn import EventBus, ChatEngine, MemoryStore"` 可用

**E3** 列出所有`morn_core/` 下旧 EventBus 原文件的清理建议（不删除，仅标记）。

---

## 验收标准

1. ✅ `MORN_QUICKSTART.md` 存在，内容完整
2. ✅ `API_REFERENCE.md` 存在，列出所有公开 API
3. ✅ `PLUGIN_DEV_GUIDE.md` 存在，包含完整开发示例
4. ✅ `pyproject.toml` 依赖已拆分（重型依赖移到 extras）
5. ✅ `pip install -e .` 后核心导入可用
6. ✅ `pip install -e ".[telegram]"` 后 Telegram 依赖可选安装
