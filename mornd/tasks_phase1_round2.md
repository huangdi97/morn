# Morn Phase 1 · 轮次 2：SDK 接口层 + 公开 API + CLI Presence

## 编程原则（14条核心准则）

| # | 准则 | 说明 |
|---|------|------|
| 1 | **Think Before Coding** | 先想后写，改之前先理解整体结构 |
| 2 | **Simplicity First** | 简单优先，不加不必要的抽象层 |
| 3 | **Surgical Changes** | 一次一事，一个PR/commit只做一件事 |
| 4 | **Goal-Driven Execution** | 目标驱动，先明确"要什么"再"怎么写" |
| 5 | **架构优先，拒绝补丁** | 不堆补丁，架构不合理就重构 |
| 6 | **面向组件的构建** | 模块化，每个组件职责清晰 |
| 7 | **显式优于隐式** | 明确的数据流和依赖，不搞魔术 |
| 8 | **代码整洁与自文档化** | 代码即文档，命名和结构说明一切 |
| 9 | **单一职责** | 一个函数/类只做一件事 |
| 10 | **组合优于委托** | 组合模式 > 继承/委托 |
| 11 | **单一状态源** | 状态只在一个地方管理 |
| 12 | **避免语法糖** | 可读性 > 炫技 |
| 13 | **命名一致性** | 同一概念用同一命名 |
| 14 | **文件不超过300行** | 超了拆 |

## 低耦合原则
- 模块间只传ID，不传对象
- 不 import 其他模块的内部函数
- 依赖倒置：模块通过接口/ID通信，不直接耦合

## 执行规则
- 使用 opencode run 执行，禁止手写文件
- 每轮完成后跑全量测试确认无回归
- opencode 会自动修复测试失败，不需要人工干预
- 命令格式：`opencode run --file ./tasks.md -m deepseek/deepseek-v4-flash`

---

## 概述

上一轮已完成内核提取 + A/S 文件迁移。本轮要：
1. 创建 `morn/sdk/` 服务接口层，让 `from morn.sdk import ChatEngine` 能用
2. 完善 `morn/__init__.py` 为完整的公开 API 门面
3. 从 `morn_core/server.py` 提取 CLI 入口为 `morn/cli/`，作为默认 Presence

---

## 任务 A：创建 SDK 服务接口层

目标：让 `from morn.sdk import ChatEngine, MemoryStore, SecurityLayer` 可用。

这些接口层只是**引用现有实现**的轻量包装，不是重写。每个文件 10~30 行。

### A1 创建目录和文件

```
morn/sdk/
├── __init__.py
├── chat.py        ← 引用 morn_core/chat/engine 的实现
├── memory.py      ← 引用 morn_core/memory/store 的实现
├── security.py    ← 引用 morn_core/security 的实现
└── presence.py    ← Presence 基类（供 Telegram 等插件继承）
```

### A2 sdk/chat.py

```python
"""Morn 对话核心 SDK"""
from morn_core.chat.engine import ChatEngine, EmotionState

__all__ = ["ChatEngine", "EmotionState"]
```

### A3 sdk/memory.py

```python
"""Morn 记忆核心 SDK"""
from morn_core.memory.store import MemoryStore
from morn_core.memory.retrieval import RetrievalEngine, LayeredRetrievalEngine

__all__ = ["MemoryStore", "RetrievalEngine", "LayeredRetrievalEngine"]
```

### A4 sdk/security.py

```python
"""Morn 安全核心 SDK"""
from morn_core.security.user_protection import UserProtection
from morn_core.security.external_boundary import ExternalBoundary

__all__ = ["UserProtection", "ExternalBoundary"]
```

### A5 sdk/presence.py

Presence 基类——所有对话界面（CLI、Telegram、Web 等）通过这个基类接入：

```python
"""Morn Presence 基类——对话界面接入点"""
from abc import ABC, abstractmethod
from typing import Optional


class MornPresence(ABC):
    """存在形式基类。所有对话界面通过此类接入 Morn 实例。"""
    
    name: str = "base"
    
    @abstractmethod
    async def start(self) -> None: ...
    
    @abstractmethod
    async def stop(self) -> None: ...
    
    @abstractmethod
    async def send_message(self, text: str) -> None: ...
```

### A6 sdk/__init__.py

```python
"""Morn SDK — 服务接口层"""
from .chat import ChatEngine, EmotionState
from .memory import MemoryStore, RetrievalEngine, LayeredRetrievalEngine
from .security import UserProtection, ExternalBoundary
from .presence import MornPresence

__all__ = [
    "ChatEngine", "EmotionState",
    "MemoryStore", "RetrievalEngine", "LayeredRetrievalEngine",
    "UserProtection", "ExternalBoundary",
    "MornPresence",
]
```

---

## 任务 B：完善 `morn/__init__.py` 公开 API

当前 `morn/__init__.py` 是空的（上一轮创建但没写内容）。改为：

```python
"""Morn — 数字生命框架"""

__version__ = "0.1.0"

# 内核
from morn.kernel import (
    EventBus, Event, Priority,
    HookManager, HookRegistration,
    SecurityValidator,
)

# SDK（S 级服务接口）
from morn.sdk import (
    ChatEngine, MemoryStore, UserProtection,
    MornPresence,
)

# 可选高级组件（A 级）
from morn.contrib.memory_advanced import (
    RawSnapshotStore, ExternalMemoryAdapter,
    GraphStore, auto_extract,
)
from morn.contrib.security_advanced import (
    DynamicPermissions, IntentDriftDetector, APZStore,
)

# 内核导出
from morn.kernel import heartbeat_loop, memory_monitor

__all__ = [
    # 版本
    "__version__",
    # 内核
    "EventBus", "Event", "Priority",
    "HookManager", "HookRegistration",
    "SecurityValidator",
    "heartbeat_loop", "memory_monitor",
    # SDK（S 级）
    "ChatEngine", "MemoryStore", "UserProtection",
    "MornPresence",
    # 高级组件（A 级）
    "RawSnapshotStore", "ExternalMemoryAdapter",
    "GraphStore", "auto_extract",
    "DynamicPermissions", "IntentDriftDetector", "APZStore",
]
```

---

## 任务 C：提取 CLI Presence

从 `morn_core/server.py` 的 `cli_loop` 函数中提取独立的 CLI Presence 到 `morn/cli/`。

目标是让 `morn --instance xxx` 这个 CLI 入口点能从 `morn/cli/main.py` 启动，而不只是从 `morn_core/server.py`。

### C1 创建 `morn/cli/main.py`

复制 `morn_core/server.py` 中的 `cli_loop` 函数（约 55 行），以及它依赖的 `MornState` 的字段。包装为一个继承 `MornPresence` 的 `CLIPresence` 类：

```python
"""Morn CLI Presence — 默认对话界面"""
import asyncio
import sys
import time
from typing import Optional

from morn.sdk.presence import MornPresence


class CLIPresence(MornPresence):
    """CLI 对话界面。通过标准输入输出与创建者交互。"""
    
    name = "cli"
    
    def __init__(self, state):
        self.state = state
        self._running = False
    
    async def start(self):
        self._running = True
        # 启动 cli 循环
        
    async def stop(self):
        self._running = False
    
    async def send_message(self, text: str):
        print(text)
```

具体实现：将 `server.py` 中的 `cli_loop` 协程内容作为 `start()` 方法的核心逻辑。注意 `cli_loop` 依赖 `state.shutdown`、`state.memory_store`、`state.chat_engine`、`state.protection`、`state.last_interaction_time` 等字段——这些由 `MornState` 提供，`CLIPresence` 通过 `self.state` 访问。

### C2 创建 `morn/cli/__init__.py`

```python
"""Morn CLI Presence"""
from .main import CLIPresence

__all__ = ["CLIPresence"]
```

### C3 更新 `morn_core/server.py`

在 `server.py` 中，将原来内联的 `cli_loop` 协程替换为对 `CLIPresence` 的调用：

```python
from morn.cli import CLIPresence

# 在 tasks 注册处，将：
#   asyncio.create_task(cli_loop(state), name="morn-cli"),
# 改为：
#   cli = CLIPresence(state)
#   asyncio.create_task(cli.start(), name="morn-cli"),
```

注意：`cli_loop` 函数在 `server.py` 中是一个独立的协程函数，有 ~55 行代码。不要删除它（可能有其他地方引用），只需将原来的 `asyncio.create_task(cli_loop(state), name="morn-cli")` 替换为 CLIPresence 方式即可。`cli_loop` 函数本身保留在原处作为备用。

---

## 任务 D：更新 pyproject.toml（可选）

如果 `morn/cli/main.py` 需要成为 CLI 入口点，更新 `pyproject.toml`：

```toml
[project.scripts]
morn = "morn.cli.main:main"
```

但注意：原来的 `morn = "morn_core.server:main"` 还在运行中，不要破坏现有入口。可以加一个别名：

```toml
[project.scripts]
morn = "morn_core.server:main"      # 当前入口（保留）
morn-cli = "morn.cli.main:main"     # 新入口（可选）
```

---

## 验收标准

1. ✅ `python -c "from morn.sdk import ChatEngine, MemoryStore, UserProtection, MornPresence"` 成功
2. ✅ `python -c "from morn import EventBus, ChatEngine, MemoryStore, UserProtection"` 成功
3. ✅ `morn/cli/` 存在，含 `__init__.py` 和 `main.py`
4. ✅ `python -c "from morn.cli import CLIPresence"` 成功
5. ✅ `pytest` 全量测试通过（0 回归）
6. ✅ `morn --instance test_phase1_round2` 可正常启动（测试 CLI 入口不被破坏）
