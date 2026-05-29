# Morn Phase 1 · 轮次 1：内核提取 + A/S 文件迁移

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

## 任务概述

基于 Phase 0 架构审计报告，将 `morn_core/` 中属于内核的部分提取到 `morn/kernel/`，同时将 11 个 A 级代码文件从 S 级目录迁移到 `morn/contrib/` 下的正确位置。

**总文件数**：内核提取 ~8 个文件（复制），A/S 迁移 ~11 个文件（移动）
**并行性**：两组任务互不冲突，可在一轮内完成

---

## 任务 A：内核提取

将以下源文件**复制**到 `morn/kernel/` 目录。注意：是新目录 `morn/`（不是 `morn_core/`），与 `morn_core/` 同级。

目标目录结构：
```
/home/hermes/morn/mornd/morn/kernel/
├── __init__.py          ← 导出所有公共类
├── bus.py               ← 复制自 eventbus/bus.py
├── plugin.py            ← 复制自 eventbus/plugin_base.py
├── plugin_registry.py   ← 复制自 eventbus/plugin_registry.py
├── hooks.py             ← 复制自 eventbus/hooks.py
├── heartbeat.py         ← 复制自 heartbeat.py
├── security.py          ← 复制自 security/security_validator.py
└── skill_store_interface.py  ← 新建：SkillStore 抽象接口
```

### A1 复制 eventbus 文件

将 `morn_core/eventbus/bus.py` 复制到 `morn/kernel/bus.py`
将 `morn_core/eventbus/plugin_base.py` 复制到 `morn/kernel/plugin.py`
将 `morn_core/eventbus/plugin_registry.py` 复制到 `morn/kernel/plugin_registry.py`
将 `morn_core/eventbus/hooks.py` 复制到 `morn/kernel/hooks.py`

**注意**：复制进来的文件要修正内部 import 路径。原文件写 `from morn_core.eventbus.bus import ...` 的，在新位置改为相对导入 `from .bus import ...`。但**不要改 `from morn_core.xxx import` 的其他引用**——那些是给外部模块用的，不是这个文件自己的导入。

具体分析每个文件的导入：
- `plugin_base.py`：引用 `morn_core.eventbus.hooks` → 改为 `.hooks`
- `hooks.py`：引用 `morn_core.eventbus.bus` → 改为 `.bus`
- `plugin_registry.py`：引用 `morn_core.eventbus.bus` 和 `morn_core.eventbus.hooks` → 改为 `.bus` 和 `.hooks`
- `bus.py`：纯标准库依赖，无 `morn_core` 引用

### A2 复制 heartbeat.py

将 `morn_core/heartbeat.py` 复制到 `morn/kernel/heartbeat.py`
修正 `from morn_core.eventbus.bus import ...` → `from .bus import ...`

### A3 复制 security_validator.py

将 `morn_core/security/security_validator.py` 复制到 `morn/kernel/security.py`
修正 `from morn_core.eventbus.bus import ...` → `from .bus import ...`
修正 `from morn_core.security.rules import ...` → security_validator 引用了 security.rules，这是一个跨模块依赖。在 kernel 中，我们需要判断：security_validator 本身属于内核（设计文档也这么说），但它依赖的 rules.py 是 S 级安全核心的一部分。**保持原有导入不变**——`from morn_core.security.rules import ...` 不需要改成 kernel 内导入，因为在 kernel 中引用上层服务是允许的（内核依赖服务比服务依赖内核好）。或者换一种方法：把 security_validator 需要的规则接口抽象出来，但本次先保持简单。

### A4 新建 skill_store_interface.py

新建 `morn/kernel/skill_store_interface.py`，定义 SkillStore 的抽象接口，解决 `skills/manager.py` 依赖 `evolution/skill_lifecycle.SkillStore` 的循环依赖问题：

```python
from abc import ABC, abstractmethod
from typing import Optional

class SkillStoreInterface(ABC):
    """SkillStore 的抽象接口，让 kernel/skills.py 依赖接口而非具体实现"""
    
    @abstractmethod
    async def get_skill(self, skill_id: str) -> Optional[dict]: ...
    
    @abstractmethod
    async def list_skills(self, tags: Optional[list[str]] = None) -> list[dict]: ...
```

### A5 新建 kernel/__init__.py

```python
"""Morn 内核：事件驱动内核 + 插件管理 + 安全验证"""

from .bus import EventBus, Event, Priority, SubscriberInfo, BusStats
from .plugin import PluginBase
from .plugin_registry import PluginRegistry, register_all_plugin_hooks
from .hooks import HookManager, HookRegistration
from .heartbeat import heartbeat_loop, memory_monitor, wal_checkpoint
from .security import SecurityValidator, ValidationResult
from .skill_store_interface import SkillStoreInterface

__all__ = [
    "EventBus", "Event", "Priority", "SubscriberInfo", "BusStats",
    "PluginBase", "PluginRegistry", "register_all_plugin_hooks",
    "HookManager", "HookRegistration",
    "heartbeat_loop", "memory_monitor", "wal_checkpoint",
    "SecurityValidator", "ValidationResult",
    "SkillStoreInterface",
]
```

---

## 任务 B：11 个 A/S 级文件迁移

将以下 A 级代码文件从 S 级目录**移动**到 `morn/contrib/` 下的正确位置。同时**修正**所有引用这些文件的 import 路径。

目标是：
1. `morn/contrib/memory_advanced/` — 存放 A 级高级记忆组件
2. `morn/contrib/security_advanced/` — 存放 A 级高级安全组件

### B1 创建目标目录

```
morn/contrib/
├── __init__.py
├── memory_advanced/
│   ├── __init__.py
│   ├── raw_snapshot_store.py
│   ├── hallucination_guard.py
│   ├── external_memory.py
│   ├── graph_store.py
│   ├── knowledge_extractor.py
│   └── l4_depositor.py
└── security_advanced/
    ├── __init__.py
    ├── risk_guard.py
    ├── ethical_judgment.py
    ├── rule_learner.py
    ├── apz_store.py
    └── audit.py
```

### B2 从 memory/ 迁移到 memory_advanced/

移动以下文件（**物理移动**，不是复制）：
- `morn_core/memory/raw_snapshot_store.py` → `morn/contrib/memory_advanced/raw_snapshot_store.py`
- `morn_core/memory/hallucination_guard.py` → `morn/contrib/memory_advanced/hallucination_guard.py`
- `morn_core/memory/external_memory.py` → `morn/contrib/memory_advanced/external_memory.py`
- `morn_core/memory/graph_store.py` → `morn/contrib/memory_advanced/graph_store.py`

**修正 import**：检查被移动文件内部引用 `from morn_core.memory.xxx` 或 `from .xxx` 的——改为 `from morn_core.memory.xxx`（指向 S 级记忆核心，没被移动的那些文件）。以及检查**所有其他文件**中引用这些被移动文件的，改为 `from morn.contrib.memory_advanced.xxx`。

### B3 从 chat/ 迁移到 memory_advanced/

移动：
- `morn_core/chat/knowledge_extractor.py` → `morn/contrib/memory_advanced/knowledge_extractor.py`
- `morn_core/chat/l4_depositor.py` → `morn/contrib/memory_advanced/l4_depositor.py`

修正所有 import 路径。注意：`chat/engine.py` 引用了 `knowledge_extractor` 和 `l4_depositor`，需要改为 `from morn.contrib.memory_advanced.knowledge_extractor import ...`

### B4 从 security/ 迁移到 security_advanced/

移动：
- `morn_core/security/risk_guard.py` → `morn/contrib/security_advanced/risk_guard.py`
- `morn_core/security/ethical_judgment.py` → `morn/contrib/security_advanced/ethical_judgment.py`
- `morn_core/security/rule_learner.py` → `morn/contrib/security_advanced/rule_learner.py`
- `morn_core/security/apz_store.py` → `morn/contrib/security_advanced/apz_store.py`
- `morn_core/security/audit.py` → `morn/contrib/security_advanced/audit.py`

**关键依赖**：`security/rules.py` 引用了 `security/rule_learner.py` 的 `RuleLearner`。迁移后改为 `from morn.contrib.security_advanced.rule_learner import RuleLearner`。同时 `security/__init__.py` 的导出也要更新。

另一个关键依赖：`memory/store.py` 引用了 `morn_core.security.rule_learner.SafetyMemoryStore` → 改为 `from morn.contrib.security_advanced.rule_learner import SafetyMemoryStore`

### B5 创建 __init__.py 文件

`morn/contrib/__init__.py`：
```python
"""Morn 可选高级组件"""
```

`morn/contrib/memory_advanced/__init__.py`：
```python
"""A 级高级记忆组件"""
from .raw_snapshot_store import RawSnapshotStore
from .external_memory import ExternalMemory
from .graph_store import GraphStore
from .knowledge_extractor import KnowledgeExtractor
from .l4_depositor import check_and_deposit
```

`morn/contrib/security_advanced/__init__.py`：
```python
"""A 级高级安全组件"""
from .risk_guard import DynamicPermissions
from .ethical_judgment import IntentDriftDetector
from .apz_store import APZStore
```

---

## 任务 C：更新 server.py

`server.py` 中初始化了所有模块，包括被迁移的 A 级组件。需要：
1. 将 `from morn_core.security.xxx` 改 `from morn.contrib.security_advanced.xxx`
2. 将 `from morn_core.chat.knowledge_extractor` 等改为新路径
3. 在 import 区域添加内核的别名导入（文件名改变参考）：例如原来 `from morn_core.security.security_validator import ...`，server.py 仍然可以用旧路径——因为我们没删除旧文件。等所有轮次完成后再清理。

**注意**：server.py 的改动只做 import 修正 + 备份确认。不要重构 server.py 本身的初始化逻辑。

---

## 验收标准

1. ✅ `morn/kernel/` 目录存在，包含 8 个文件
2. ✅ `morn/contrib/memory_advanced/` 存在，含 6 个文件（raw_snapshot_store.py, hallucination_guard.py, external_memory.py, graph_store.py, knowledge_extractor.py, l4_depositor.py）
3. ✅ `morn/contrib/security_advanced/` 存在，含 5 个文件（risk_guard.py, ethical_judgment.py, rule_learner.py, apz_store.py, audit.py）
4. ✅ 旧文件 `morn_core/` 下的对应文件已被删除（物理移动完成）
5. ✅ `python -c "from morn.kernel import EventBus, PluginBase, SecurityValidator"` 成功
6. ✅ `pytest` 全量测试通过
7. ✅ `morn_core/memory/store.py` 仍能正常导入（未受影响）
