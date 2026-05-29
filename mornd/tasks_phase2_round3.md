# Morn Phase 2 · 轮次 3：内置插件迁移（剩余 8 个）

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
| 8 | **代码整洁与自文档化** | 代码即名词，命名和结构说明一切 |
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

上一轮完成了首批 3 个内置插件迁移（HealthMonitorPlugin、DreamEnginePlugin、SelfReflectionPlugin）。本轮将 `morn/kernel/plugin_registry.py` 中剩余的 8 个插件全部迁移为标准 MornPlugin 子类。

这 8 个插件有高度一致的模板模式，迁移工作量小——每个 ~30-50 行。

---

## 迁移模板

每个插件都遵循这个模板——即上一轮已建立的模式：

```python
"""插件名称"""
from morn.kernel.plugin import MornPlugin, PluginContext
from morn.kernel.hooks import HookRegistration
from morn.kernel.bus import Event, Priority


class XxxPlugin(MornPlugin):
    plugin_id = "xxx"
    name = "中文名"
    version = "0.1.0"
    plugin_class = "A"       # 根据原插件定位
    needs_periodic_trigger = True
    usage_hint = "low"
    
    def __init__(self):
        super().__init__()
        self._counter = 0      # 如果原插件有 N 次一触发
    
    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        self._register_hooks()
    
    def _register_hooks(self):
        """从原 register_xxx_hooks 函数复制逻辑"""
        # 复制原 callback 函数的逻辑
        # 使用 self.context.hook_manager.register(...)
    
    async def on_unload(self):
        await super().on_unload()
```

### 原注册函数到插件的映射规则

每个 `register_xxx_hooks(event_bus, hook_manager, state)` 函数中：
- `event_bus` → `self.context.event_bus`
- `hook_manager` → `self.context.hook_manager`
- `state` → 需要通过 `self.context` 获取，但 plugin_registry 中的插件是在 server.py 中通过 `register_all_plugin_hooks(event_bus, hook_manager, state)` 注册的。由于 `state` 不是 `PluginContext` 的标准部分，插件会通过一个 `state_ref` 属性引用 server.py 的 MornState 实例。

**关键设计决策**：这些插件通过 `plugin_registry.register_all_plugin_hooks(event_bus, hook_manager, state)` 接收 `state` 引用。新插件应该在 `on_load` 时从 `context.config` 获取 `state` 引用，或者通过构造函数参数传入。为了保持简单，采用构造参数方式：

```python
class XxxPlugin(MornPlugin):
    def __init__(self, state_ref=None):
        super().__init__()
        self._state_ref = state_ref
```

---

## 任务 A：迁移 IdentityAffirmerPlugin

从 `register_identity_hooks`（plugin_registry.py:40-56）迁移。

原逻辑：每 `heartbeat.minute` 触发 → 检查 `state.identity_affirmer` → 调用 `tick()` → 错误发布 `task.failed`

```python
class IdentityAffirmerPlugin(MornPlugin):
    plugin_id = "identity_affirmer"
    name = "身份确认器"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    
    def __init__(self, state_ref=None):
        super().__init__()
        self._state_ref = state_ref
    
    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        async def on_minute(event):
            if not self._state_ref or not self._state_ref.identity_affirmer:
                return
            try:
                await self._state_ref.identity_affirmer.tick()
            except Exception as e:
                await context.event_bus.publish(Event(
                    type="task.failed",
                    payload={"plugin": "identity_affirmer", "error": str(e)},
                    source="identity_affirmer", priority=Priority.HIGH,
                ))
        context.hook_manager.register(HookRegistration(
            plugin_id="identity_affirmer", event_type="heartbeat.minute",
            callback=on_minute, timeout=10.0,
        ))
```

保存为 `morn/plugins/identity_affirmer.py`。

---

## 任务 B：迁移 SelfPrunerPlugin

从 `register_self_pruning_hooks`（plugin_registry.py:59-88）迁移。

原逻辑：每 10 个 `heartbeat.minute` 触发一次 → 调用 `state.self_pruner.diagnose()` → 如有关联数据发布 `self_pruning.completed`。

```python
class SelfPrunerPlugin(MornPlugin):
    plugin_id = "self_pruner"
    name = "自我瘦身"
    version = "0.1.0"
    plugin_class = "A"
    needs_periodic_trigger = True
    
    def __init__(self, state_ref=None):
        super().__init__()
        self._state_ref = state_ref
        self._counter = 0
    
    async def on_load(self, context):
        await super().on_load(context)
        async def on_minute(event):
            nonlocal self
            self._counter += 1
            if self._counter < 10:
                return
            self._counter = 0
            if not self._state_ref or not self._state_ref.self_pruner:
                return
            try:
                result = await self._state_ref.self_pruner.diagnose()
                if result.get("capsules_pruned", 0) or result.get("skills_pruned", 0) or result.get("emotion_pruned", 0):
                    await context.event_bus.publish(Event(
                        type="self_pruning.completed",
                        payload=result,
                        source="self_pruner", priority=Priority.LOW,
                    ))
            except Exception as e:
                await context.event_bus.publish(Event(
                    type="task.failed",
                    payload={"plugin": "self_pruner", "error": str(e)},
                    source="self_pruner", priority=Priority.HIGH,
                ))
        context.hook_manager.register(HookRegistration(
            plugin_id="self_pruner", event_type="heartbeat.minute",
            callback=on_minute, timeout=10.0,
        ))
```

保存为 `morn/plugins/self_pruner.py`。

---

## 任务 C：迁移 BondTrackerPlugin

从 `register_bond_update_hooks`（plugin_registry.py:91-119）迁移。

原逻辑：每 5 个 `heartbeat.minute` → 更新连接深度。

保存为 `morn/plugins/bond_tracker.py`。

---

## 任务 D：迁移 IntentDriftPlugin

从 `register_intent_drift_hooks`（plugin_registry.py:122-151）迁移。

原逻辑：每 10 个 `heartbeat.minute` → `check_drift()` → 如有警报发布 `security.alert`。

保存为 `morn/plugins/intent_drift.py`。

---

## 任务 E：迁移 AuditPlugin

从 `register_audit_hooks`（plugin_registry.py:154-189）迁移。

原逻辑：每 10 个 `heartbeat.minute` → 查询最近 capsule → `audit_agent.extract_and_deposit()`。

保存为 `morn/plugins/audit.py`。

---

## 任务 F：迁移 ThinkingEvolutionPlugin

从 `register_thinking_evolution_hooks`（plugin_registry.py:192-217）迁移。

原逻辑：每 `heartbeat.hour` → 如果空闲 1 小时+ → `evolve()`。

保存为 `morn/plugins/thinking_evolution.py`。

---

## 任务 G：迁移 MilestonePlugin

从 `register_milestone_hooks`（plugin_registry.py:220-250）迁移。

原逻辑：每 5 个 `heartbeat.minute` → `check_milestones()` → 如有触发 `push_greetings()`。

保存为 `morn/plugins/milestones.py`。

---

## 任务 H：迁移 HindsightPlugin

从 `register_hindsight_hooks`（plugin_registry.py:253-282）迁移。

原逻辑：每 `heartbeat.hour` → `hindsight_engine.tick(emotion)`。

保存为 `morn/plugins/hindsight.py`。

---

## 任务 I：更新 __init__.py

更新 `morn/plugins/__init__.py`，导出全部 8 个新插件：

```python
"""Morn 内置插件"""
from .health_monitor import HealthMonitorPlugin
from .dream_engine import DreamEnginePlugin
from .self_reflection import SelfReflectionPlugin
from .identity_affirmer import IdentityAffirmerPlugin
from .self_pruner import SelfPrunerPlugin
from .bond_tracker import BondTrackerPlugin
from .intent_drift import IntentDriftPlugin
from .audit import AuditPlugin
from .thinking_evolution import ThinkingEvolutionPlugin
from .milestones import MilestonePlugin
from .hindsight import HindsightPlugin

__all__ = [
    "HealthMonitorPlugin",
    "DreamEnginePlugin",
    "SelfReflectionPlugin",
    "IdentityAffirmerPlugin",
    "SelfPrunerPlugin",
    "BondTrackerPlugin",
    "IntentDriftPlugin",
    "AuditPlugin",
    "ThinkingEvolutionPlugin",
    "MilestonePlugin",
    "HindsightPlugin",
]
```

---

## 验收标准

1. ✅ `python -c "from morn.plugins import *"` 导入全部 11 个插件（3 + 8）
2. ✅ 每个插件实例化后 `isinstance(p, MornPlugin)` 为 True
3. ✅ 每个插件的 `plugin_id`、`name`、`plugin_class`、`needs_periodic_trigger` 正确设置
4. ✅ `morn/plugins/` 目录包含 12 个文件（__init__.py + 11 个插件）
5. ✅ 原 `plugin_registry.py` 未修改（向后兼容）
6. ✅ `pytest` 全量测试通过（无新增失败）
