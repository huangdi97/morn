# Morn Phase 2 · 轮次 2：内置插件迁移（首批 3 个）

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

当前有 9 个内部插件通过 `morn/kernel/plugin_registry.py` 中的函数注册为 EventBus 钩子。它们本质上是插件但没继承 `MornPlugin`。本轮将首批 3 个迁移为标准 `MornPlugin` 子类。

### 迁移模式

每个插件从两段式（`register_xxx_hooks` 函数 + `server.py` 初始化）变为一段式（`MornPlugin` 子类，自包含加载 + 注册钩子）。

迁移后的插件将：
1. 继承 `MornPlugin`
2. 在 `on_load` 中注册自己的钩子（使用 `self.context.hook_manager.register()`）
3. 在 `on_unload` 中清理钩子
4. 声明自己的 `plugin_id`、`plugin_class`、`needs_periodic_trigger` 等字段

---

## 任务 A：迁移 HealthMonitor

当前 `health_monitor.py` 位于 `morn_core/eventbus/health_monitor.py`。它是一个通过 `server.py` 初始化的独立类，注册了自己的钩子。

新建 `morn/plugins/health_monitor.py`：

```python
"""HealthMonitor 插件——系统健康监控"""
from morn.kernel.plugin import MornPlugin, PluginContext, PluginDependency
from morn.kernel.hooks import HookRegistration
from morn.kernel.bus import Event, Priority


class HealthMonitorPlugin(MornPlugin):
    """健康监控插件
    
    监控系统关键指标：心跳、内存、磁盘。通过 EventBus 发布健康事件。
    """
    
    plugin_id = "health_monitor"
    name = "健康监控"
    version = "0.1.0"
    plugin_class = "S"
    needs_periodic_trigger = True
    usage_hint = "low"
    
    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        # 注册钩子——与原来 server.py 中 HealthMonitor.register_hooks() 相同
        self._register_hooks()
    
    async def on_unload(self):
        # 清理钩子
        if self.context and self.context.hook_manager:
            pass  # hook_manager 会在 unload 时统一清理
        await super().on_unload()
    
    def _register_hooks(self):
        """注册心跳/事件钩子"""
        # 从原 HealthMonitor 类中复制钩子注册逻辑
        pass
```

注意：这只是结构迁移。具体的钩子回调逻辑从原 `health_monitor.py` 和 `server.py` 中复制。**不要改变功能行为**，只改变组织方式。

### 具体实现要求

1. 复制 `morn_core/eventbus/health_monitor.py` 中的 `HealthMonitor` 类的完整逻辑到 `HealthMonitorPlugin` 中
2. `HealthMonitorPlugin.on_load()` 内部调用原 `HealthMonitor.register_hooks()` 的逻辑
3. 新建 `morn/plugins/__init__.py` — 默认空，后续添加导出

---

## 任务 B：迁移 DreamEngine

`dream_engine` 在 `plugin_registry.py` 中注册为 `register_dream_engine_hooks`。

新建 `morn/plugins/dream_engine.py`：

```python
"""梦境引擎插件——空闲时重组记忆生成梦境叙事"""
import time
from morn.kernel.plugin import MornPlugin, PluginContext, PluginDependency
from morn.kernel.hooks import HookRegistration
from morn.kernel.bus import Event, Priority


class DreamEnginePlugin(MornPlugin):
    plugin_id = "dream_engine"
    name = "梦境引擎"
    version = "0.1.0"
    plugin_class = "B"
    needs_periodic_trigger = True
    usage_hint = "low"
    
    async def on_load(self, context: PluginContext):
        await super().on_load(context)
        # 从 state.dream_engine 获取实例引用
        # 注册 heartbeat.minute 钩子
        
    async def on_unload(self):
        await super().on_unload()
```

具体实现：将 `plugin_registry.py` 中 `register_dream_engine_hooks` 函数的逻辑（从 `state.dream_engine` 获取引用并调用 `tick()`）封装为 `DreamEnginePlugin` 的 `on_heartbeat` 方法。

---

## 任务 C：迁移 SelfReflection

`self_reflection` 的钩子在 `server.py` 中直接注册（约 397-404 行）：

```python
if self_reflection:
    async def self_reflection_event_callback(event):
        await self_reflection.light_reflection()
    hook_manager.register(HookRegistration(
        plugin_id="self_reflection",
        event_type="heartbeat.minute",
        callback=self_reflection_event_callback,
        timeout=15.0,
    ))
```

新建 `morn/plugins/self_reflection.py`，封装这个逻辑。

---

## 任务 D：创建插件注册点

新建 `morn/plugins/__init__.py`，导出所有内置插件：

```python
"""Morn 内置插件"""
from .health_monitor import HealthMonitorPlugin
from .dream_engine import DreamEnginePlugin
from .self_reflection import SelfReflectionPlugin

__all__ = [
    "HealthMonitorPlugin",
    "DreamEnginePlugin",
    "SelfReflectionPlugin",
]
```

---

## 任务 E：更新 server.py 使用新插件

**注意：这是关键改动，需要谨慎处理。** `server.py` 当前通过两种方式注册钩子：

1. `register_all_plugin_hooks()` 批量注册 9 个插件钩子（在 plugin_registry.py 中）
2. `health_monitor.register_hooks()` 单独注册（在 server.py 约 373 行）
3. `hook_manager.register()` 直接注册（如 self_reflection，在 server.py 约 400 行）

迁移策略：
- `health_monitor` → 用 `PluginLoader.load(HealthMonitorPlugin)` 替换原来 `health.register_hooks()`
- `dream_engine` → 在 `register_all_plugin_hooks()` 中去掉对应的函数调用，改为在 `on_load` 中自注册
- `self_reflection` → 去掉 `server.py` 中直接的 `hook_manager.register()`，改为插件

**这一轮只做结构迁移，server.py 中暂时保留旧路径作为备用。** 新插件通过 `PluginLoader` 加载，旧钩子注册路径保持不动。当所有 9 个内置插件都迁移完成后，统一清理。

---

## 验收标准

1. ✅ `python -c "from morn.plugins import HealthMonitorPlugin, DreamEnginePlugin, SelfReflectionPlugin"` 通过
2. ✅ 每个插件实例化后满足 `isinstance(plugin, MornPlugin)` 和 `plugin.plugin_class` 正确
3. ✅ `morn/plugins/` 目录包含 4 个文件（`__init__.py` + 3 个插件）
4. ✅ `pytest` 全量测试通过（无新增失败）
5. ✅ 原 `health_monitor.py` 的钩子仍然能通过旧路径调用（向后兼容）
