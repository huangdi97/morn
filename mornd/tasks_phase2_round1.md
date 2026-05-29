# Morn Phase 2 · 轮次 1：Plugin 基类完善 + 加载器基础设施

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

Phase 2 要实现完整的插件系统。轮次 1 建立基础设施——完善 Plugin 基类（与设计文档 §3.4 接口契约合并），创建插件加载器。

### 设计文档参考

设计文档 §3.4 定义了插件接口契约：

```yaml
plugin_id: "memory_core"           # 唯一标识
name: "记忆核心"                     # 可读名称
version: "0.1.0"                    # 语义化版本
dependencies:                       # 依赖声明
  - plugin: "security_core"
    min_version: "0.1.0"
required_permissions:               # 必要权限（缺失则无法启动）
  - "memory.read"
  - "memory.write"
optional_permissions:               # 可选权限（可降级运行）
  - "storage.archive"
needs_periodic_trigger: true        # 是否需要周期性触发
usage_hint: "medium"                # 资源提示: low / medium / high
```

插件的生命周期：加载 → 注册事件钩子 → 运行 → 卸载。

---

## 任务 A：完善 Plugin 基类

当前 `morn/kernel/plugin.py` 是一个基础的 `PluginInfo` dataclass。替换为完整的 `MornPlugin` 抽象基类，合并设计文档的接口契约：

```python
"""Morn 插件基类——所有插件继承此类"""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class PluginDependency:
    """插件依赖声明"""
    plugin: str          # 依赖的插件 ID
    min_version: str     # 最低版本
    optional: bool = False  # True = 可选依赖


class PluginContext:
    """插件运行时上下文——在 on_load 时注入
    
    包含插件需要的所有运行时资源：
    - event_bus: 事件总线引用
    - hook_manager: 钩子管理器
    - config: 插件配置字典
    - data_dir: 实例数据目录
    """
    
    def __init__(self, event_bus=None, hook_manager=None, config=None, data_dir=None):
        self.event_bus = event_bus
        self.hook_manager = hook_manager
        self.config = config or {}
        self.data_dir = data_dir


class MornPlugin(ABC):
    """Morn 插件抽象基类"""
    
    # —— 声明层（设计文档 §3.4 接口契约） ——
    plugin_id: str = ""                           # 唯一标识
    name: str = ""                                # 可读名称
    version: str = "0.1.0"                        # 语义化版本
    dependencies: list[PluginDependency] = []     # 依赖声明
    required_permissions: list[str] = []          # 必要权限
    optional_permissions: list[str] = []          # 可选权限
    needs_periodic_trigger: bool = False          # 是否需要周期性触发
    usage_hint: str = "low"                       # 资源提示: low / medium / high
    plugin_class: str = ""                        # S / A / B / C
    
    def __init__(self):
        self.context: Optional[PluginContext] = None
        self._loaded = False
    
    # —— 生命周期 ——
    @abstractmethod
    async def on_load(self, context: PluginContext):
        """插件加载时调用。接收 PluginContext，包含 event_bus 等运行时资源。"""
        self.context = context
        self._loaded = True
    
    async def on_unload(self):
        """插件卸载时调用。清理资源。"""
        self._loaded = False
        self.context = None
    
    # —— 事件钩子（可选） ——
    async def on_event(self, event):
        """收到订阅的事件时调用"""
        pass
    
    async def on_heartbeat(self, tick: int):
        """心跳触发（如果 needs_periodic_trigger=True）"""
        pass
    
    async def on_chat(self, message: str):
        """有对话时调用"""
        pass
    
    # —— 工具方法 ——
    @property
    def is_loaded(self) -> bool:
        return self._loaded
    
    def require_permission(self, permission: str) -> bool:
        """检查是否有某个权限"""
        return permission in self.required_permissions or \
               permission in self.optional_permissions
```

保存到 `morn/kernel/plugin.py`（覆盖现有文件。注意：当前 `plugin.py` 中有 `PluginInfo` dataclass 和 `register_plugin_hooks` 函数，需要在保留它们的基础上扩展，或者判断是否可以被新类替代）。

注意保留原文件中 `PluginInfo` 和 `register_plugin_hooks`——新代码应该引用 `MornPlugin`，但旧代码可能还在用 `PluginInfo`。

输出文件内容后确认：
- `PluginInfo` 保留（历史兼容）
- `register_plugin_hooks` 保留（历史兼容）
- 新增 `MornPlugin` 抽象基类
- 新增 `PluginDependency` dataclass
- 新增 `PluginContext` 类

---

## 任务 B：创建插件加载器

新建 `morn/kernel/plugin_loader.py`，实现插件扫描、版本校验、依赖解析、生命周期管理：

```python
"""插件加载器——扫描、验证、加载、卸载插件"""

import importlib
import inspect
import logging
import pkgutil
from pathlib import Path
from typing import Optional

from .plugin import MornPlugin, PluginContext


class PluginLoader:
    """插件加载器
    
    职责：
    - 扫描插件目录，发现插件类
    - 验证依赖和版本
    - 管理加载/卸载生命周期
    - 维护已加载插件的注册表
    """
    
    def __init__(self, event_bus=None, hook_manager=None):
        self._plugins: dict[str, MornPlugin] = {}  # plugin_id -> instance
        self._failed: list[tuple[str, str]] = []    # (plugin_id, error)
        self._event_bus = event_bus
        self._hook_manager = hook_manager
    
    def discover(self, search_paths: list[Path]) -> list[type[MornPlugin]]:
        """扫描目录，返回发现的插件类列表
        
        遍历 search_paths 中的每个目录，import 所有 .py 文件，
        找出继承 MornPlugin 的非抽象子类。
        """
        discovered = []
        # 实现：遍历目录 → importlib → inspect 找子类
        # 跳过以 _ 开头的文件
        return discovered
    
    async def load(self, plugin_class: type[MornPlugin], 
                   config: Optional[dict] = None,
                   data_dir: Optional[Path] = None) -> MornPlugin:
        """加载单个插件
        
        1. 实例化插件类
        2. 检查依赖是否满足
        3. 验证权限
        4. 创建 PluginContext
        5. 调用 on_load
        6. 注册到插件表
        """
        pass
    
    async def unload(self, plugin_id: str) -> bool:
        """卸载指定插件"""
        pass
    
    def get_plugin(self, plugin_id: str) -> Optional[MornPlugin]:
        """获取已加载的插件实例"""
        return self._plugins.get(plugin_id)
    
    def list_plugins(self) -> list[dict]:
        """列出所有已加载插件的信息"""
        return [
            {
                "plugin_id": p.plugin_id,
                "name": p.name,
                "version": p.version,
                "loaded": p.is_loaded,
                "class": p.plugin_class,
            }
            for p in self._plugins.values()
        ]
    
    def list_failed(self) -> list[tuple[str, str]]:
        """列出加载失败的插件"""
        return list(self._failed)
    
    async def load_all(self, plugin_classes: list[type[MornPlugin]],
                       default_config: Optional[dict] = None,
                       data_dir: Optional[Path] = None) -> int:
        """批量加载插件。返回成功加载的数量。"""
        success = 0
        for cls in plugin_classes:
            try:
                await self.load(cls, default_config, data_dir)
                success += 1
            except Exception as e:
                plugin_id = getattr(cls, 'plugin_id', cls.__name__)
                self._failed.append((plugin_id, str(e)))
                logging.getLogger("morn.plugin").error(f"Failed to load {plugin_id}: {e}")
        return success
    
    async def unload_all(self):
        """卸载所有插件"""
        for plugin_id in list(self._plugins.keys()):
            await self.unload(plugin_id)
```

先创建接口框架（方法体用 `pass` 或简单实现），让后续轮次可以填充具体逻辑。这一轮重点是**让接口定义和基本加载流程可用**。

---

## 任务 C：更新 kernel/__init__.py

在 `morn/kernel/__init__.py` 中添加新类的导出：

```python
from .plugin import MornPlugin, PluginDependency, PluginContext, PluginInfo, register_plugin_hooks
from .plugin_loader import PluginLoader
```

更新 `__all__` 列表，添加：
```python
"MornPlugin", "PluginDependency", "PluginContext", "PluginLoader",
```

---

## 任务 D：基础测试

创建测试文件 `tests/test_plugin_loader.py`，测试：

```python
import pytest
from morn.kernel.plugin import MornPlugin, PluginDependency, PluginContext
from morn.kernel.plugin_loader import PluginLoader


class TestMornPlugin:
    """验证 Plugin 基类的基本契约"""
    
    def test_abstract_class_cannot_instantiate(self):
        """MornPlugin 有 abstractmethod，不能直接实例化"""
        with pytest.raises(TypeError):
            MornPlugin()  # 应该报错，因为 on_load 是 abstractmethod
    
    def test_concrete_plugin_can_instantiate(self):
        """继承并实现 on_load 的插件可以实例化"""
        class TestPlugin(MornPlugin):
            plugin_id = "test_plugin"
            name = "测试插件"
            
            async def on_load(self, context):
                pass
        
        plugin = TestPlugin()
        assert plugin.plugin_id == "test_plugin"
        assert plugin.version == "0.1.0"
        assert plugin.is_loaded is False


class TestPluginLoader:
    """验证 PluginLoader 的基本操作"""
    
    @pytest.mark.asyncio
    async def test_load_simple_plugin(self):
        """能加载一个最简单的插件"""
        class SimplePlugin(MornPlugin):
            plugin_id = "simple"
            name = "简单插件"
            
            async def on_load(self, context):
                pass
        
        loader = PluginLoader()
        loaded = await loader.load(SimplePlugin)
        assert loaded is not None
        assert loaded.plugin_id == "simple"
        assert loaded.is_loaded
    
    @pytest.mark.asyncio
    async def test_load_and_unload(self):
        """加载后可以卸载"""
        class TempPlugin(MornPlugin):
            plugin_id = "temp"
            name = "临时插件"
            
            async def on_load(self, context):
                pass
        
        loader = PluginLoader()
        await loader.load(TempPlugin)
        assert loader.get_plugin("temp") is not None
        
        result = await loader.unload("temp")
        assert result is True
        assert loader.get_plugin("temp") is None
    
    @pytest.mark.asyncio
    async def test_list_plugins(self):
        """list_plugins 返回正确的信息"""
        class ListPlugin(MornPlugin):
            plugin_id = "listable"
            name = "可列表插件"
            
            async def on_load(self, context):
                pass
        
        loader = PluginLoader()
        await loader.load(ListPlugin)
        
        plugins = loader.list_plugins()
        assert len(plugins) >= 1
        listable = [p for p in plugins if p["plugin_id"] == "listable"]
        assert len(listable) == 1
        assert listable[0]["name"] == "可列表插件"
```

---

## 验收标准

1. ✅ `python -c "from morn.kernel.plugin import MornPlugin, PluginDependency, PluginContext"` 通过
2. ✅ `python -c "from morn.kernel.plugin_loader import PluginLoader"` 通过
3. ✅ `MornPlugin` 不能直接实例化（abstractmethod 保护）
4. ✅ 继承 `MornPlugin` 并实现 `on_load` 的插件可实例化
5. ✅ `PluginLoader.load()` 能加载并注册插件
6. ✅ `PluginLoader.unload()` 能卸载插件
7. ✅ `PluginLoader.list_plugins()` 返回正确信息
8. ✅ `pytest tests/test_plugin_loader.py -v` 全通过
9. ✅ 全量 pytest 无回归
