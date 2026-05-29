# Morn Phase 1 · 轮次 3：收尾验证 + pyproject.toml 注册

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

前两轮已完成内核提取（轮次1）+ SDK + CLI Presence + 公开 API（轮次2）。本轮收尾：

1. **注册 morn/ 为新 pip 包** — 在 pyproject.toml 中添加子包注册
2. **全量 import 扫描** — 查找任何遗漏的旧 import 路径
3. **测试全量通过验证** — 确认任意 `morn --instance` 可启动
4. **生成 import 验证脚本** — 确保所有模块都能正常导入

---

## 任务 A：注册 morn/ 为新 pip 包

当前 `morn/` 目录是纯文件系统目录，没有 pip 注册。需要在 `pyproject.toml` 中注册为子包。

修改 `pyproject.toml`，让 `morn/` 作为 `morn-core` 包的一部分被注册：

```toml
[tool.setuptools.packages.find]
include = ["morn_core*", "morn*"]
```

如果不存在 `[tool.setuptools.packages.find]`，则添加。这里的目的是让 `pip install -e .` 在安装 `morn_core*` 的同时也安装 `morn*` 下的所有子包（`morn.kernel`、`morn.sdk`、`morn.cli`、`morn.contrib`、`morn.contrib.memory_advanced`、`morn.contrib.security_advanced`）。

### A1 检查 pyproject.toml

读取当前 `pyproject.toml`，找到 `[tool.setuptools.packages.find]` 部分（如果有）。如果没有，添加：

```toml
[tool.setuptools.packages.find]
include = ["morn_core*", "morn*"]
```

### A2 验证

```bash
# 重新安装包
pip install -e .

# 验证 morn 子包可导入
python -c "import morn.kernel; import morn.sdk; import morn.cli; import morn.contrib; print('✓ all morn subpackages importable')"
```

---

## 任务 B：全量 import 扫描

扫描整个 `morn_core/` 和 `morn/` 目录，查找任何引用已被移动的 A 级文件的旧 import 路径。

被移动的文件（11 个）：
- `morn_core/memory/raw_snapshot_store.py`
- `morn_core/memory/hallucination_guard.py`
- `morn_core/memory/external_memory.py`
- `morn_core/memory/graph_store.py`
- `morn_core/chat/knowledge_extractor.py`
- `morn_core/chat/l4_depositor.py`
- `morn_core/security/risk_guard.py`
- `morn_core/security/ethical_judgment.py`
- `morn_core/security/rule_learner.py`
- `morn_core/security/apz_store.py`
- `morn_core/security/audit.py`

新路径：
- `morn.contrib.memory_advanced.*`
- `morn.contrib.security_advanced.*`

### B1 扫描旧 import

```bash
# 搜索所有 .py 文件中引用已移动模块的旧路径
grep -rn "from morn_core\.\(memory\|chat\|security\)\." --include="*.py" \
    morn_core/ morn/ | grep -E "(raw_snapshot_store|hallucination_guard|external_memory|graph_store|knowledge_extractor|l4_depositor|risk_guard|ethical_judgment|rule_learner|apz_store|audit)"
```

如果有匹配，每个都需要更新为 `morn.contrib.*` 路径。

### B2 执行修复

对每一个找到的旧 import 路径，执行 patch 操作更新为新路径。

注意：`morn_core/security/__init__.py` 中的 `from .audit import ...` 不需要改，因为 `.audit` 是相对导入，它指向的是当前目录下的 `audit.py`。但 `audit.py` 已经被移走了——所以需要改为 `from morn.contrib.security_advanced.audit import ...`，这个已经在轮次1中改过了。这里需要确认。

---

## 任务 C：验证测试

### C1 运行全量测试

```bash
python -m pytest tests/ --timeout=60 -q 2>&1 | tail -30
```

确认：
- 没有新增失败（仅保留轮次1/2中已记录的预存失败）
- 测试数量与之前一致（1158+）

### C2 验证 `morn --instance` 可启动

```bash
timeout 5 python -c "
from morn_core.server import main
print('✓ morn --instance entry point importable')
"
```

不能真正启动实例（心跳循环会阻塞），只需要验证 import 不报错。

### C3 验证新入口

```bash
python -c "
from morn.cli import CLIPresence
from morn.sdk.presence import MornPresence
from morn.sdk.chat import ChatEngine
from morn.sdk.memory import MemoryStore
from morn.sdk.security import UserProtection
print('✓ All new entry points importable')
"
```

---

## 任务 D（可选）：尝试清理旧文件

检查 `morn_core/eventbus/` 中的原始文件是否仍被使用。内核文件已复制到 `morn/kernel/`，但原始文件在 `morn_core/` 中仍然存在且被大量使用（server.py 和其他模块仍然引用它们）。**本阶段不删除旧文件**——保留原样以确保回退路径可用。

只需要输出一个列表，标记"可清理"和"暂保留"的文件。

---

## 验收标准

1. ✅ `pip install -e .` 后 `python -c "import morn.kernel; import morn.sdk; import morn.cli; import morn.contrib"` 成功
2. ✅ 全量 import 扫描无遗漏的旧路径引用
3. ✅ `pytest` 全量测试通过（无新增失败）
4. ✅ `python -c "from morn_core.server import main"` 可导入（旧入口不破坏）
5. ✅ `python -c "from morn.cli import CLIPresence"` 可导入（新入口可用）
