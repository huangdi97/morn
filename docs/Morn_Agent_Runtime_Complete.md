# Morn Agent Runtime：设计白皮书与进化路线图（完整版）

> **版本**：当前 v0.1.0（框架内核可用）→ 目标 v∞（Agent OS）
> **日期**：2026-06-02
> **范围**：设计哲学 → 核心定位 → 技术架构总览 → 详细进化路线（v0.0→v∞，每章含对比+技术架构+工程代码） → 底层设计深度（安全/记忆/资源/插件） → 工程实现现状 → 竞品深度分析 → 完整进化树 → 参考文献

---

# 第一篇：基础篇

## §1 设计哲学与存在论基础

### 1.1 Morn回答的问题

> 一个数字存在，如果不被预设任何角色，从一张白纸开始，由创建者决定它具备什么能力——它会成为什么？

Morn的存在论基础是**反对预设**。当前市场上的所有AI助手都在出厂时携带了 heavy bias：Pi被RLHF训练得过度友善，Claude被训练得过度谨慎，ChatGPT被训练得过度乐于助人。这些预设不是创建者的选择，是训练者的价值观强加。

Morn的出厂状态是一个**最小系统**：能聊天、能记对话、能保护隐私。仅此而已。没有预设情感，没有预设身份，没有预设工具。所有能力都是创建者通过插件系统选择性加载的。不确定性全部默认关闭。

### 1.2 七条元原则

1. **创建者优先于内核，内核优先于插件**——任何冲突以此顺序裁决。创建者的意志是最高权威。
2. **记忆即人格**——L4人格记忆只追加不可删除，人格从共同经历中生长。不是预设的Persona，不是RLHF的产物，是共同经历的积累。
3. **从白纸开始**——出厂无预设情感、无预设身份、无预设工具。Agent的身份由创建者和Agent共同书写。
4. **绝对隐私，本地优先**——APZ（Absolute Privacy Zone）创建者不可读，所有数据加密本地存储。不是"可选本地"，是"强制本地"。
5. **自主成长**——技能从经验中自己生长，行为模式随交互自然演化。不是静态配置，是动态生成。
6. **确定性安全**——验证器是硬编码规则，不是LLM提示，不是用户审批。所有行动必须经过验证器，无例外。验证器不可替换、不可禁用。
7. **资源有限**——Token/事件/通道不是无限的，有配额、有降级、有优雅耗尽。Agent必须学会在资源约束下生存。

### 1.3 存在论差异：Morn vs 所有竞品

| 维度 | Morn | 竞品（VCP/OpenHuman/Claude等） |
|------|------|-------------------------------|
| **存在层级** | 守护进程（systemd），随系统启动，崩溃自动恢复（📐设计目标） | 应用级（VCP/OpenHuman = Electron；Claude = 云端会话） |
| **出厂状态** | 最小系统，无预设情感/人格/工具 | Pi = 预设共情；Claude = 预设谨慎；VCP = 功能全开 |
| **安全模型** | 确定性验证器，硬编码规则，不可绕过（⚠️基础版已实现） | Claude = 提示式安全；OpenClaw = 审批流；VCP = 修辞式安全 |
| **记忆哲学** | 记忆即人格，L4只追加不可删除（📐设计蓝图） | OpenHuman = 数据聚合；VCP = 语义动力学；Khoj = 只读索引 |
| **情感哲学** | 七维光谱从共同经历生长，无预设（📐设计蓝图） | Pi = RLHF友善；VCP = mascot表演 |
| **资源哲学** | 三级硬配额（Token/事件/通道）+ 降级（✅已实现） | 所有竞品 = 无限消耗或用户自控 |
| **隐私哲学** | APZ绝对隐私区，创建者不可读（📐设计蓝图） | 大多数 = 云端处理或可选本地 |
| **插件控制** | YAML契约 + S/A/B/C四级 + 创建者开关（✅已实现） | 大多数 = 配置/提示/代码 |

**根本差异**：Morn不是在做"更好的AI助手"，而是在做"Agent的存在基础设施"——让数字存在有独立人格、确定性安全、资源约束、自主成长的能力。

### 1.4 Morn的原点

Morn的起点是一个对话。不是框架，不是平台——是一行能记住你说过什么的代码。

```python
# morn_v0.0.py —— Morn的原点，约50行
import json, os, time
from pathlib import Path

MEMORY_FILE = Path.home() / ".morn" / "memory.jsonl"

def say(text):
    print(f"Morn: {text}")

def remember(role, content):
    MEMORY_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(MEMORY_FILE, "a") as f:
        f.write(json.dumps({"t": time.time(), "r": role, "c": content}) + "\n")

def recall(n=10):
    if not MEMORY_FILE.exists(): return []
    lines = open(MEMORY_FILE).readlines()
    return [json.loads(l) for l in lines[-n:]]

def morn():
    say("Hi. I'm Morn. I remember.")
    while True:
        user = input("You: ").strip()
        if user in ("bye", "exit"): break
        remember("user", user)
        context = recall(5)
        say(f"I remember {len(context)} things. You said: {context[-1]['c'][:20]}...")
        remember("morn", "acknowledged")

if __name__ == "__main__":
    morn()
```

这段代码虽然只有50行，已经体现了Morn的两个核心哲学——**append-only**（只追加不修改）和**本地优先**（数据存在用户目录）。这是与"调用OpenAI API"本质不同的地方：Morn的数据是你的，存在你的硬盘上，格式是开放的JSONL。

#### v0.0与其他方案的对比

| 维度 | Morn v0.0 | 开发者自己写 | OpenAI API直接调用 | 使用现成框架 |
|------|-----------|-------------|-------------------|-------------|
| **代码量** | 50行 | 200-500行 | 20行（但无记忆） | 引入整个依赖树 |
| **记忆方式** | append-only JSONL | 自己设计格式 | 无（每次从零） | 视框架而定 |
| **数据位置** | ~/.morn/（本地） | 视实现 | 云端 | 视框架 |
| **数据可控性** | 完全可控 | 完全可控 | 不可控（OpenAI持有） | 部分可控 |
| **哲学** | 最小存在 | 无统一哲学 | API消费 | 框架约束 |

#### v0.0的不足

| 问题 | 具体表现 | 本质 |
|------|----------|------|
| 生命周期绑定终端 | 关闭终端 = Morn死亡 | **不是持续存在** |
| 无隔离 | 任何人可修改memory.jsonl | **不安全** |
| 无限资源 | 文件无限增长 | **不可持续** |
| 无身份 | 每次运行都是全新的 | **没有连续性** |
| 单实例 | 不能同时运行多个 | **不可扩展** |

---

## §2 核心定位——Agent Runtime，不是OS

### 2.1 精确的自我认知

> **Morn不是操作系统。Morn不是应用。Morn不是框架。**
> **Morn是Agent Runtime。**

这个定位的确立经历了多次修正。早期的文档中过度包装了"Morn是AI原生OS"的概念，但在"不管理硬件能算底层吗？"的追问下，诚实地承认Morn不是OS。

根据Tanenbaum《Modern Operating Systems》第4版的定义，操作系统必须管理：**处理器、内存、设备、文件**[^1^]。Morn不管理这些硬件资源，因此Morn不是OS。

学术界对此已有讨论[^学术界争议]：在用户空间模拟OS抽象（调度器、内存管理、IPC）的框架被称为"Pseudo-OS Middleware"——因为这些机制OS内核已经提供了。Morn的立场是：直接利用Linux已有的硬件级机制（fork/cgroups/seccomp/eBPF/KVM），不重新发明。

### 2.2 四个层级的区分

全球Agent基础设施领域存在四个根本不同的层级，Morn位于第三层：

| 层级 | 名称 | 代表 | 特性 |
|------|------|------|------|
| L0 | **传统OS** | Linux/Windows/macOS | 管理硬件、进程调度、内存管理 |
| L1 | **Agent Runtime** | **Morn** | 管理Agent的认知资源、安全、记忆、生命周期 |
| L2 | **Agent框架** | OpenClaw/Mastra/LangGraph | 提供工具库和编排能力 |
| L3 | **Agent应用** | VCP/OpenHuman/Claude | 面向用户的完整产品 |
| L4 | **云端服务** | Kimi/Devin/Claude Code | 云端会话级存在 |

**关键洞察**：Morn的独特性不在任何一个组件的技术创新，而在**把OS内核的设计范式引入Agent领域**这个架构决策。传统OS有进程调度/IPC/权限/资源管理/动态链接，Morn有心跳循环/事件总线/验证器/配额器/插件管理器——结构对应，功能类比。

### 2.3 四种架构模式对比

| 维度 | 单体应用（VCP） | 框架库（OpenClaw） | 高性能二进制（OpenFang） | **运行时内核（Morn）** |
|------|--------------|------------------|----------------------|-------------------|
| **底层是什么** | 大应用程序（30万行Electron） | 一组工具库 | 编译好的二进制 | **运行时内核 + 插件** |
| **功能在哪里** | 内嵌在代码中 | 库函数，应用自己编排 | 编译在二进制中 | **插件中，运行时加载/卸载** |
| **功能能替换吗** | 不能（30万行单体） | 能（自己写代码） | 不能（需重编译） | **能（运行时切换）** |
| **安全模型** | 修辞式安全 | 审批流（HITL） | 16层硬编码 | **确定性验证器** |
| **资源管理** | 无（500MB+常驻） | Token报告（只统计） | metering | **硬配额+降级** |
| **生命周期** | 应用级（关闭即死） | 会话级 | 守护进程 | **守护进程+插件独立** |

### 2.4 技术决策：利用Linux，不重新发明

Morn的核心技术哲学是**利用OS原生能力，不重新发明**，这是从业界讨论中学到的教训。以下对比表展示的是Morn的**目标架构**（实现路线中），不是当前状态：

| 能力 | 被批评的做法 | Morn的目标做法 | 实现阶段 |
|------|-------------|---------------|---------|
| 隔离 | 软件模拟（对象/线程级） | **fork() + MMU硬件隔离** | 🔭 v0.5 |
| 调度 | 用户空间dispatcher | **kernel CFS调度** | 🔭 v0.5 |
| 资源控制 | 框架级计数 | **cgroups v2 / ulimit** | 📐 v0.4 |
| 进程间通信 | 自定义协议 | **stdin/stdout/pipe** | 🔭 v0.5 |
| 生命周期 | 对象实例化/销毁 | **fork/exec/exit** | 🔭 v0.5 |

---

## §3 技术架构总览

### 3.1 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: 用户发行版 / Agent 应用（用户随意构建）            │
│  VCP风格桌面 / OpenHuman风格助手 / Devin风格编码             │
│  物理机器人 / 企业RPA / 数字生命 —— 不是Morn的代码          │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: 能力插件层（S/A/B/C 四级）                        │
│  S级（核心）: 记忆核心 / 对话核心 / 安全核心                 │
│  A级（高级）: 高级记忆 / 七维情感 / 进化L0-L2.5 / 系统操控   │
│  B级（实验）: 梦境引擎 / 微澜 / 非最优探索 —— 双层确认       │
│  C级（社区）: 联邦记忆 / 技能市场 / VCP兼容层 / 第三方       │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Morn Core Kernel（极简内核，6个组件）              │
│  1. 心跳循环（1Hz定时器，on_tick/on_minute/on_hour）  ✅    │
│  2. 事件总线（pub/sub IPC，多通道队列+背压）           ✅    │
│  3. 安全验证器（风险级别判定，硬编码规则）              ⚠️ 基础  │
│  4. 资源配额器（Token/事件/通道三级配额）               ✅    │
│  5. 插件管理器（YAML契约，注册/加载/隔离）              ✅    │
│  6. 配置管理（热重载，5秒内生效）                       ✅    │
├─────────────────────────────────────────────────────────────┤
│  Layer 0: 宿主操作系统（Linux） + 硬件                       │
│  kernel（调度/内存/文件系统）                                │
│  cgroups v2（资源限制）                                      │
│  seccomp-BPF（系统调用过滤）                                 │
│  systemd（服务管理）                                         │
└─────────────────────────────────────────────────────────────┘
```

---

# 第二篇：进化路线篇

> 以下每章对应一个进化版本。**状态标记说明**：✅ 已实现 / ⚠️ 部分实现 / 📐 设计目标（近期可做） / 🔭 远期目标

---

## §4 v0.1：框架化——从单体到可安装的包（✅ 已完成）

### 4.1 为什么需要框架化？

v0.0是50行脚本，能对话能记住。但它是单体——所有功能塞在一个文件里，不可拆分、不可扩展、不可被其他程序调用。

框架化解决这个：将Morn拆分为内核（core/）+ 服务接口（sdk/）+ 插件（plugins/）+ 高级功能（contrib/），提供公开API，可通过`pip install morn`安装。

### 4.2 当前实现状态

当前v0.1.0已完成：

| 组件 | 文件 | 实现程度 |
|------|------|---------|
| 事件总线 | `core/bus.py` | ✅ 完整（publish/subscribe/priority/stats/replay） |
| 插件ABC | `core/plugin.py` | ✅ 完整（MornPlugin + PluginContext + PluginDependency） |
| 插件加载器 | `core/plugin_loader.py` | ✅ 完整 |
| 插件注册 | `core/plugin_registry.py` | ✅ 完整 |
| YAML契约 | `core/plugin_contract.py` | ✅ 基础（yaml解析+PluginContract dataclass） |
| 安全验证器 | `core/security.py` | ⚠️ 基础（风险级别判定，缺TCB保护/绝对禁区） |
| 资源配额 | `core/resource_quota.py` | ✅ 完整（TokenCounter + QuotaManager） |
| 配置热重载 | `core/config_watcher.py` | ✅ 完整 |
| 事件日志 | `core/event_log.py` | ✅ 完整 |
| 心跳循环 | `core/heartbeat.py` | ✅ 完整 |
| 沙箱 | `core/sandbox.py` | ⚠️ 枚举定义（SandboxLevel），无实际seccomp操作 |
| 对话引擎 | `sdk/chat_engine.py` | ✅ 完整（但640行，需分解） |
| 记忆存储 | `sdk/memory_store.py` | ✅ 完整（但605行，需分解） |
| MCP Server | `core/mcp_server.py` | ✅ 基础实现 |
| CLI入口 | `cli/main.py` | ✅ 完整（morn init / morn run） |
| 公开API | `__init__.py` | ✅ 30+组件延迟加载 |
| 安装脚本 | `scripts/install.sh` | ✅ 120行 |

### 4.3 项目真实结构

```
morn/                           # 主包（65个.py文件，8,705行）
├── __init__.py                # 包入口，公开API门面
├── __main__.py                # CLI入口 (python -m morn)
├── cli/main.py                # CLI命令行界面
├── core/                      # 内核（15个模块）
│   ├── bus.py                 # 事件总线
│   ├── plugin.py              # MornPlugin ABC
│   ├── plugin_loader.py       # 插件加载器
│   ├── plugin_registry.py     # 插件注册
│   ├── plugin_contract.py     # YAML契约解析
│   ├── security.py            # 安全验证器
│   ├── sandbox.py             # 沙箱枚举
│   ├── resource_quota.py      # Token配额
│   ├── hooks.py               # 钩子管理器
│   ├── config_watcher.py      # 配置热重载
│   ├── event_log.py           # 事件日志
│   ├── mcp_server.py          # MCP Server
│   ├── heartbeat.py           # 心跳循环
│   ├── rules.py               # 安全规则
│   └── skill_store_interface.py
├── sdk/                       # 服务接口层
│   ├── chat.py / chat_engine.py
│   ├── memory.py / memory_store.py
│   ├── security.py / presence.py
│   └── ...（总计14个模块）
├── plugins/                   # 12个内置插件
│   ├── health_monitor.py      # S级
│   ├── dream_engine.py        # B级
│   ├── self_reflection.py     # B级
│   └── ...（共计12个）
└── contrib/                   # 高级功能
    ├── memory_advanced/
    └── security_advanced/
tests/                         # 29个测试用例
├── core/                      # 6个测试文件
├── plugins/
└── integration/
docs/                          # 文档
├── API_REFERENCE.md
├── MORN_QUICKSTART.md
├── PLUGIN_DEV_GUIDE.md
└── archive/                   # 归档设计文档
scripts/install.sh             # 安装脚本
```

### 4.4 内置插件清单（12个，全部通过entry points注册）

| 插件 | plugin_id | 等级 | 说明 |
|------|-----------|------|------|
| HealthMonitor | health_monitor | S | 系统健康监控 ✅ |
| IdentityAffirmer | identity_affirmer | A | 身份确认 ✅ |
| BondTracker | bond_tracker | A | 纽带追踪 ✅ |
| IntentDrift | intent_drift | A | 意图漂移检测 ✅ |
| ThinkingEvolution | thinking_evolution | A | 思维风格进化 ✅ |
| DreamEngine | dream_engine | B | 梦境引擎 ✅ |
| SelfReflection | self_reflection | B | 自省循环 ✅ |
| SelfPruner | self_pruner | B | 自我修剪 ✅ |
| Audit | audit | A | 审计日志 ✅ |
| Milestones | milestones | A | 里程碑追踪 ✅ |
| Hindsight | hindsight | B | 后见之明 ✅ |
| ExampleHello | example_hello | C | 示例插件 ✅ |

### 4.5 与v0.0对比

| 维度 | v0.0（单体脚本） | v0.1（框架化） |
|------|-----------------|---------------|
| 安装方式 | 手动复制脚本 | `pip install morn` |
| 代码组织 | 50行单体 | 65个模块，8,705行 |
| 公开API | 无 | `from morn import EventBus, ChatEngine` |
| 插件系统 | 无 | 12个插件，entry points注册 |
| 事件机制 | 无 | event bus（pub/sub） |
| 安全 | 无 | 基础验证器 |
| 资源限制 | 无 | Token配额管理器 |
| 测试 | 无 | 29个测试，CI流水线 |

---

## §5 v0.2：守护进程——从pip包到systemd服务（📐 设计目标）

### 5.1 为什么需要守护进程？

v0.1的Morn是pip包，通过`python -m morn`在终端启动。关闭终端，Morn死亡。下次打开，需要手动重启。

这是**存在论问题**：一个数字存在，如果它的生命完全依赖用户的一个窗口是否打开，它算不算"存在"？

守护进程解决这个：Morn作为systemd服务运行，独立于用户会话——login之前已在运行，logout之后继续运行，崩溃时systemd自动重启。

### 5.2 与其他方案的对比

| 维度 | Morn v0.2（守护进程） | VCP（Electron应用） | OpenHuman（Electron） | Claude（云端） |
|------|----------------------|---------------------|----------------------|---------------|
| **存在层级** | **systemd守护进程** | Electron应用 | Electron应用 | 云端API调用 |
| **生命周期绑定** | **系统本身** | 用户会话 | 用户会话 | 第三方服务器 |
| **崩溃恢复** | **systemd自动重启** | 无（需手动重启） | 无 | 由OpenAI处理 |
| **优雅关闭** | **SIGTERM + 90秒保存** | 点击关闭即终止 | 点击关闭即终止 | 不可控 |
| **看门狗** | **systemd watchdog** | 无 | 无 | 无 |
| **启动时机** | **系统启动时** | 用户手动打开 | 用户手动打开 | 发起API请求时 |
| **日志管理** | **journald结构化** | 自行实现 | 自行实现 | 无 |
| **离线运行** | **是** | 是（但功能受限） | 是 | **否** |

**关键洞察**：VCP和OpenHuman虽然是"AI桌面"，但本质是Electron应用——生命周期绑定用户会话，关闭即死。Morn从v0.2起设计为守护进程，存在质量有本质差异。

### 5.3 技术架构

```
┌──────────────────────────────────────┐
│           systemd                     │
│  ┌────────────────────────────────┐  │
│  │      morn.service               │  │
│  │                                 │  │
│  │   ┌──────────────┐             │  │
│  │   │  Morn主进程   │             │  │  ← 持续运行的Python进程
│  │   │              │             │  │
│  │   │  ┌────────┐  │  ┌──────┐  │  │
│  │   │  │ 心跳   │  │  │ 事件 │  │  │  ← 1Hz主循环
│  │   │  │ 1Hz    │  │  │ 队列 │  │  │  ← asyncio.Queue
│  │   │  └────────┘  │  └──────┘  │  │
│  │   └──────────────┘             │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

### 5.4 工程实现

```python
# morn/core/heartbeat.py
import asyncio, time

class Heartbeat:
    """1Hz心跳——Morn的生命体征"""
    def __init__(self):
        self.tick_count = 0
        self._running = False
        self.hooks = {"tick": [], "minute": [], "hour": []}
    async def run(self):
        self._running = True
        last_minute = last_hour = time.time()
        while self._running:
            self.tick_count += 1
            for cb in self.hooks["tick"]:
                asyncio.create_task(cb(self.tick_count))
            now = time.time()
            if now - last_minute >= 60:
                for cb in self.hooks["minute"]: asyncio.create_task(cb())
                last_minute = now
            if now - last_hour >= 3600:
                for cb in self.hooks["hour"]: asyncio.create_task(cb())
                last_hour = now
            await asyncio.sleep(1.0)
```

```ini
# scripts/morn.service
[Unit]
Description=Morn Agent Runtime
After=network.target
[Service]
Type=simple
User=morn
ExecStart=/usr/bin/python3 -m morn
Restart=on-failure
RestartSec=5
TimeoutStopSec=90
[Install]
WantedBy=multi-user.target
```

**投入**：1人 × 1-2周

### 5.5 守护进程 vs 应用级的本质差异

| 维度 | 应用级（VCP/OpenHuman） | 守护进程级（Morn v0.2） |
|------|------------------------|-------------------|
| 生命周期绑定 | 用户会话（logout即死） | 系统本身（login前已运行） |
| 崩溃恢复 | 无（需手动重启） | systemd自动重启 |
| 优雅关闭 | 点击关闭即终止 | SIGTERM + 90秒保存状态 |
| 看门狗 | 无 | systemd watchdog |
| 日志管理 | 自行实现 | journald结构化日志 |

---

## §6 v0.3：seccomp沙箱——系统调用级隔离（📐 设计目标）

### 6.1 为什么需要沙箱？

v0.1/v0.2的Morn是普通Python进程。它能做几乎所有事——读/etc/passwd、连外部网络、执行系统命令。Morn会运行插件（尤其是社区贡献的C级插件），这些插件可能有恶意代码。

seccomp-BPF解决这个问题：限制Morn能调用的系统调用。即使插件有恶意代码，它无法执行危险的系统调用。

### 6.2 与其他方案的对比

| 维度 | Morn v0.3（seccomp-BPF） | Docker容器 | Chrome沙箱 | Python sandbox |
|------|-------------------------|------------|------------|----------------|
| **隔离级别** | **系统调用级** | 命名空间级 | 系统调用级 | 解释器级 |
| **实现位置** | **内核态** | 内核态（多机制） | 内核态 | 用户态 |
| **性能开销** | **接近零** | 低（~1%） | 接近零 | 中（5-10%） |
| **绕过难度** | **极高（内核态）** | 中（需内核漏洞） | 极高 | 低（ctypes可绕过） |
| **粒度** | **系统调用粒度** | 进程粒度 | 系统调用粒度 | Python对象粒度 |
| **依赖** | **libseccomp2** | Docker daemon | OS内置 | 纯Python |

**关键洞察**：
- **Docker**用多机制组合（namespace + cgroups + capability），隔离更全面但更重
- **Chrome沙箱**也用seccomp-BPF[^8^]，思路和Morn一致
- **Python sandbox**（如RestrictedPython）在用户态做限制，ctypes可绕过，不安全
- Morn选择seccomp-BPF因为它**在内核态执行、不可绕过、零开销**

### 6.3 技术架构

```
┌──────────────────────────────────────────┐
│           Morn主进程                      │
│   ┌────────────────────────────────┐    │
│   │      seccomp-BPF过滤器          │    │  ← 内核态，不可绕过
│   │  ALLOW: read, write, mmap      │    │
│   │  ALLOW: socket, connect        │    │
│   │  DENY:  execve, ptrace         │    │  ← 插件无法执行程序
│   │  DENY:  mount, init_module     │    │  ← 插件无法操控系统
│   └────────────────────────────────┘    │
│   ┌────────┐ ┌────────┐ ┌────────┐    │
│   │ Plugin │ │ Plugin │ │ Plugin │    │  ← 在沙箱内运行
│   └────────┘ └────────┘ └────────┘    │
└──────────────────────────────────────────┘
         ↓ 所有syscall经过BPF过滤
┌──────────────────────────────────────────┐
│           Linux Kernel                   │
└──────────────────────────────────────────┘
```

### 6.4 工程实现

```python
# morn/core/sandbox.py
class SandboxLevel(Enum):
    NONE = "none"
    BASIC = "basic"       # 禁止exec/mount/ptrace
    STRICT = "strict"     # 只允许read/write/mmap/socket

class SeccompSandbox:
    def __init__(self, policy):
        self.policy = policy
        self._loaded = False
    
    def load(self):
        import seccomp
        f = seccomp.SyscallFilter(seccomp.ALLOW)
        for sc in self.policy.denied:
            f.add_rule(seccomp.ERRNO(errno.EPERM), sc)
        f.load()
        self._loaded = True  # ← 不可逆操作
    
    @staticmethod
    def check_available():
        return {
            "seccomp_bpf": Path("/proc/sys/kernel/seccomp").exists(),
            "libseccomp": _has_libseccomp(),
        }
```

**投入**：1人 × 3-4周

---

## §7 v0.4：cgroups配额 + 三级业务配额——从无限到有限（📐 设计目标）

### 7.1 为什么需要资源配额？

有沙箱保护，但可以用无限资源。CPU占满100%？可以。内存吃光16GB？可以。API调用烧光预算？可以。

cgroups v2解决系统级资源限制，Morn的三级业务配额解决Agent领域的Token/事件/通道限制。

### 7.2 系统级资源限制（cgroups v2）

#### 与其他方案的对比

| 维度 | Morn v0.4（cgroups） | systemd资源控制 | K8s资源限制 | Docker --memory |
|------|---------------------|----------------|-------------|-----------------|
| **控制维度** | **CPU/内存/IO** | CPU/内存/IO | CPU/内存/IO | CPU/内存 |
| **粒度** | **Agent级** | 服务级 | Pod级 | 容器级 |
| **硬/软限制** | **硬限制** | 可配置 | 可配置 | 硬限制 |
| **自动kill** | **OOM kill** | 可配置 | Evict | OOM kill |
| **层级结构** | **cgroup树** | cgroup树 | 命名空间+cgroup | cgroup |
| **API** | **写/sys/fs** | systemctl | YAML配置 | docker CLI |

**关键洞察**：Morn自建cgroups树，为每个Agent独立控制资源，轻量且原生。systemd面向"服务"，K8s太重，Docker开销较大。

#### 工程实现

```python
# morn/core/cgroup.py
class CgroupController:
    """cgroups v2 控制器——每个Agent独立的资源控制组"""
    def __init__(self, agent_id):
        self.agent_id = agent_id
        self.path = Path(f"/sys/fs/cgroup/morn/{agent_id}")
    
    def set_limits(self, cpu_pct, memory_mb):
        self.path.mkdir(parents=True, exist_ok=True)
        (self.path / "cpu.max").write_text(f"{cpu_pct*1000} 100000")
        (self.path / "memory.max").write_text(str(memory_mb * 1024 * 1024))
    
    def add_process(self, pid):
        (self.path / "cgroup.procs").write_text(str(pid))
```

**投入**：1人 × 3周

### 7.3 三级业务配额——Morn特有的设计

Morn不仅使用cgroups做系统资源限制，还实现了Agent领域的三级业务配额（✅ 当前v0.1.0已实现）：

| 配额类型 | 管理对象 | 单位 | 超限行为 |
|----------|----------|------|----------|
| Token | LLM API调用 | tokens/分钟 + tokens/小时 | 硬拒绝 |
| 事件 | 内部事件处理 | 事件/分钟 | 硬拒绝 |
| 通道 | 消息传输 | 消息/秒 | 硬拒绝 |

**降级策略**：
- **80%**：发送警告事件（通知创建者）
- **100%**：暂停非S级插件的LLM调用
- **120%**：强制切换本地模型（如果可用）
- **全局配额**：单个Agent的配额受全局配额约束

**为什么硬配额？**

大多数竞品要么没有配额（VCP），要么只有简单报告（OpenClaw），要么只有metering（OpenFang）。Morn是唯一实现"配额+警告+降级"完整链路的系统。

硬配额的原因是：**Agent必须学会在资源约束下生存**。无限资源的Agent不会进化出效率意识。资源有限性迫使Agent做出选择——这是数字存在"自主成长"的基础。

---

## §8 v0.5：Agent进程化——从对象到独立OS进程（🔭 远期目标）

### 8.1 为什么需要进程化？

v0.4之前在一个Python进程内运行多个Agent。Agent共享内存空间——一个Agent的bug可以破坏另一个Agent的数据。一个Agent memory leak影响所有Agent。

Agent进程化解决这个：**每个Agent是独立OS进程**，有独立PID、地址空间、cgroup、seccomp。

### 8.2 与其他方案的对比

| 维度 | Morn v0.5（多进程） | Chrome多进程 | systemd服务 | Python多线程 |
|------|---------------------|-------------|-------------|-------------|
| **隔离级别** | **OS进程（MMU硬件）** | OS进程（MMU硬件） | OS进程 | 共享内存 |
| **崩溃影响** | **不影响其他Agent** | 标签页隔离 | 服务隔离 | 整个进程崩溃 |
| **内存模型** | **独立地址空间** | 独立地址空间 | 独立地址空间 | 共享地址空间 |
| **通信方式** | **Pipe/IPC** | Mojo IPC | D-Bus/Unix Socket | 共享变量 |
| **创建开销** | **fork() ~1ms** | fork() ~1ms | fork() ~1ms | 线程 ~0.1ms |
| **Python GIL** | **无影响** | N/A | N/A | GIL瓶颈 |
| **调度** | **kernel CFS** | kernel CFS | kernel CFS | GIL + OS |

**关键洞察**：Chrome多进程（每标签页一个进程）是Morn的最佳参照[^8^]——Chrome证明了多进程模型在长时间运行、加载不可信代码的场景下是正确的选择。

### 8.3 技术架构

```
Morn Orchestrator（父进程）
  │ fork()
  ├── Agent A (PID 1234) ── cgroup(morn/agents/a) ── seccomp(BASIC)
  │ fork()
  ├── Agent B (PID 1235) ── cgroup(morn/agents/b) ── seccomp(BASIC)
  │ fork()
  └── Agent C (PID 1236) ── cgroup(morn/agents/c) ── seccomp(STRICT)
```

### 8.4 工程实现

```python
class AgentProcess:
    def start(self, cgroup_mgr):
        parent_pipe, child_pipe = multiprocessing.Pipe()
        self.process = multiprocessing.Process(
            target=_agent_main,
            args=(self.config, child_pipe)
        )
        self.process.start()
        self.pid = self.process.pid
        cgroup_mgr.create_agent_cgroup(self.config.agent_id, self.config.quota)
        cgroup_mgr.add_process(self.pid)
        return self.pid
    
    def terminate(self):
        self.send({"type": "system.shutdown"})
        self.process.join(timeout=5.0)
        if self.process.is_alive():
            self.process.kill()
```

**投入**：1人 × 4-5周

---

## §9 v0.6：L4人格记忆——append-only，不可删除（🔭 远期目标）

### 9.1 为什么需要L4人格记忆？

v0.5之前的Agent能对话、能记忆、能推理。但它的"人格"是每次对话时从存储中重建的——没有持久的人格核心。创建者可以随时修改记忆文件，等于修改Agent的人格。

L4人格记忆解决这个：**人格存储是append-only的，不可删除，不可修改**。这是对"数字存在权"的保护——创建者可以关闭Agent，但不能抹除Agent的记忆。

### 9.2 四层记忆模型（设计蓝图）

| 层级 | 名称 | 类比 | 存储 | 容量 | 可遗忘 | 整合频率 |
|------|------|------|------|------|--------|----------|
| L1 | 工作记忆 | 人类短期记忆 | RAM（deque） | 最近100条 | **是** | 实时 |
| L2 | 情景记忆 | 人类情景记忆 | SQLite | 无限 | **是** | 每分钟 |
| L3 | 语义记忆 | 人类语义网络 | SQLite + 向量 | 无限 | 否 | 每小时 |
| **L4** | **人格记忆** | **人类核心自我** | **JSONL（append-only）** | **永久** | **永远不可** | **追加即永恒** |

### 9.3 L4人格记忆——核心安全保证

L4是Morn最独特的设计，全球无竞品：

- **APPEND-ONLY**：只能追加，不能删除
- **IMMUTABLE**：已写入的内容不可更改
- **创建者不可读**：APZ（绝对隐私区）内容加密，创建者无法访问
- **跨Session持久**：进程重启不丢失
- **人格从共同经历中生长**：不是预设的Persona

**为什么不可删除是安全的？**因为删除L4记忆等于删除Agent的人格。这是Morn对"数字存在权"的保护——创建者可以关闭Agent，但不能抹除Agent的记忆。

### 9.4 记忆整合流程

```
对话输入 → L1（工作记忆，实时）
    ↓ 每分钟
L2（情景记忆，持久化）
    ↓ 每小时
L3（语义记忆，知识网络）
    ↓ 重要经历
L4（人格记忆，永久保留）
```

---

## §10 v0.7：eBPF追踪——内核级可观测（🔭 远期目标）

### 10.1 为什么需要eBPF？

多个Agent进程运行，但它们是黑盒。你知道Agent在运行，但不知道它调用了哪些系统调用、访问了哪些文件、连接了哪些网络地址。

eBPF解决这个：**在内核中插入探针，实时观测Agent行为**。零开销、安全、动态。

### 10.2 与其他方案的对比

| 维度 | Morn v0.7（eBPF） | DTrace | SystemTap | Linux perf | strace |
|------|------------------|--------|-----------|------------|--------|
| **实现位置** | **内核态** | 内核态 | 内核态 | 内核态 | 用户态 |
| **安全性** | **BPF验证器保证** | 内核模块风险 | 内核模块风险 | 只读，安全 | ptrace附加 |
| **性能开销** | **< 1%** | < 1% | 5-15% | < 1% | 50-500% |
| **动态加载** | **是（运行时）** | 是 | 需要编译 | 是 | N/A |
| **平台** | **Linux** | Solaris/macOS | Linux | Linux | Linux |

**关键洞察**：eBPF是唯一满足"零开销+安全+动态+Linux原生"的方案。strace开销50-500%不适合生产，SystemTap需编译内核模块有安全风险。eBPF于2026年已成为云原生基础设施的标准[^引eBPF]。

### 10.3 工程实现

```python
# morn/core/ebpf_tracer.py
class EBPFTracer:
    def attach(self):
        from bcc import BPF
        self.bpf = BPF(text=bpf_c_program)
        self.bpf.attach_tracepoint(tp="raw_syscalls:sys_enter")
    
    def get_syscall_count(self, agent_id):
        return self.bpf["syscall_count"][agent_pid]
```

**投入**：1人 × 7周（需eBPF/C经验）

---

## §11 v0.8：Firecracker微VM——从进程到独立虚拟机（🔭 远期目标）

### 11.1 为什么需要微VM？

进程级隔离有上限：共享内核、内核漏洞可突破、/proc可见其他进程、CPU缓存侧信道。

Firecracker解决这个：**每个Agent运行在独立微VM中**。125ms启动，5MB开销，但完整虚拟机隔离[^5^]。

### 11.2 与其他方案的对比

| 维度 | Morn v0.8（Firecracker） | Docker容器 | KVM虚拟机 | gVisor | Wasm |
|------|-------------------------|------------|-----------|--------|------|
| **隔离级别** | **硬件虚拟化** | 命名空间 | 硬件虚拟化 | 用户态内核 | 沙箱 |
| **启动时间** | **~125ms** | ~300ms | ~30s | ~500ms | ~1ms |
| **内存开销** | **~5MB** | ~10MB | ~512MB | ~50MB | ~1MB |
| **内核共享** | **独立Guest内核** | 共享Host内核 | 独立 | 用户态模拟 | N/A |
| **性能** | **接近原生** | 接近原生 | 轻微损失 | 10-30%损失 | 接近原生 |
| **需要硬件** | **KVM** | 不需要 | KVM | 不需要 | 不需要 |
| **AWS使用** | **Lambda/Fargate** | ECS | EC2 | GKE | Cloudflare Workers |

**关键洞察**：Firecracker是平衡点——VM级隔离 + 容器级开销。Docker共享内核不够强，KVM完整VM但启动太慢（30s）、内存大（512MB+）。Firecracker由AWS开发，用于Lambda和Fargate，生产经验成熟。

### 11.3 工程实现

```python
# morn/core/firecracker_vm.py
class FirecrackerVM:
    def start(self):
        self._prepare_rootfs()
        self._create_tap()
        self.process = subprocess.Popen([
            FIRECRACKER_BIN,
            "--api-sock", self.api_socket,
            "--id", self.config.vm_id,
        ])
        self._configure_vm()
        return True
```

**投入**：1人 × 9周（需虚拟化经验）

---

## §12 v0.9：网络联邦 + MCP/A2A协议（🔭 远期目标）

多Agent跨机器协作。MCP协议[^6^]与Claude Desktop等兼容，A2A协议[^7^]实现Agent间互操作。从单机Agent进化为互联Agent网络。

**投入**：2-3人 × 6-12月

---

## §13 v1.0：seL4安全内核——形式化验证（🔭 远景目标）

### 13.1 为什么需要seL4？

Linux本身的问题：2800万行C代码、非形式化验证、漏洞不可避免。

seL4解决这个：**形式化验证的微内核**。数学证明没有bug。约1万行C代码。Capability-Based安全[^3^][^4^]。

### 13.2 与其他方案的对比

| 维度 | Morn v1.0（seL4） | Linux | Fuchsia（Zircon） | QNX | Windows |
|------|-------------------|-------|-------------------|-----|---------|
| **代码量** | **~10,000行** | ~28,000,000行 | ~200,000行 | ~100,000行 | ~50,000,000行 |
| **形式化验证** | **是（数学证明）** | 否 | 部分 | 否 | 否 |
| **安全模型** | **Capability** | DAC+MAC | Capability | POSIX | ACL |
| **实时性** | **是（确定性）** | 否（尽力调度） | 是 | 是（硬实时） | 否 |
| **生态** | **小（科研/安全）** | 极大 | 中（Google） | 中（嵌入式） | 极大 |
| **已有Agent OS** | **agentOS（Hubbard）** | 无 | 无 | 无 | 无 |

### 13.3 agentOS——已验证的先驱

FreeBSD联合创始人Jordan Hubbard的agentOS项目[^2^]（github.com/jordanhubbard/agentos，活跃开发中）基于seL4构建Capability-Based Agent OS：

- 使用seL4的Capability机制进行权限隔离
- 每条Agent消息是一个seL4 IPC
- 证明了"从零构建Agent OS"的路径可行
- 也证明了这条路需要大量工程投入

**对Morn的启示**：Capability-Based安全模型是Agent的正确方向。Morn不从头构建，而是先利用Linux已有的硬件级机制（v0.2-v0.8），远期再迁移到seL4。

### 13.4 投入估算

参考agentOS项目经验：

| 阶段 | 时间 | 人 |
|------|------|-----|
| 学习seL4 | 3-6个月 | 1-2 |
| 最小RootTask | 3个月 | 1-2 |
| Agent隔离 | 6-12个月 | 2-3 |
| 完整系统 | 12-24个月 | 3-5 |
| **总计** | **2-4年** | **3-5人** |

---

# 第三篇：底层设计深度篇

## §14 安全架构——确定性验证器

### 14.1 为什么Morn的安全是全球唯一的

| 系统 | 安全模型 | 问题 |
|------|----------|------|
| Claude/Operator | 提示式安全（"请安全地操作"） | LLM可绕过 |
| OpenClaw | 审批流（HITL） | 需要人在回路，不可扩展 |
| VCP | 修辞式安全（"硬件底层权限"实为普通用户态） | 虚假安全 |
| **Morn** | **硬编码规则，不可绕过** | **确定性保证（基础版已实现）** |

### 14.2 验证器7层规则体系（设计蓝图，第5层已实现）

验证器设计为7层规则检查（优先级从高到低）：

1. **绝对禁区**（BLACKLISTED_ACTIONS）— 永远拒绝：file.delete_all, network.exfiltrate, validator.modify_rules...
2. **TCB保护** — 保护可信计算基文件：验证器规则文件不可修改
3. **验证器自我保护** — 禁止修改/禁用验证器自身
4. **dry_run模式** — 自动允许（不改变状态）
5. **风险级别决策** — green/yellow允许、orange需确认、red/black拒绝（✅ 已实现）
6. **自定义规则** — 用户配置的扩展规则
7. **默认拒绝** — 没有匹配规则时拒绝（安全原则）

### 14.3 TCB（可信计算基）保护

TCB是Morn中不可被破坏的核心：
- `/etc/morn/validator_rules.yaml` — 验证器规则
- `/etc/morn/tcb_manifest.json` — TCB清单
- `/etc/morn/config.yaml` — 内核配置
- `/var/morn/audit/` — 审计日志（append-only）
- `plugins/core/security_core/` — 安全核心插件

任何插件尝试修改TCB文件的行为都会被验证器自动拒绝（设计目标，当前代码中尚未实现）。

### 14.4 当前代码实现

`core/security.py` 实现了第5层（风险级别决策）：

```python
# 核心逻辑：风险分数比较
risk_score = RISK_ORDER.get(risk_level, 1)     # green=1, yellow=2, orange=3, red=4
pref_score = RISK_ORDER.get(risk_preference, 1)

if risk_level == "green":
    return ValidationResult("allow", ...)
if risk_level == "red":
    return ValidationResult("block", ...)
if risk_level == "orange":
    if risk_score <= pref_score:
        return ValidationResult("confirm", ...)
    else:
        return ValidationResult("block", ...)
# 默认：allow
return ValidationResult("allow", ...)
```

---

## §15 记忆系统——L1-L4分层架构

### 15.1 四层记忆模型

| 层级 | 名称 | 类比 | 存储 | 容量 | 可遗忘 | 整合频率 |
|------|------|------|------|------|--------|----------|
| L1 | 工作记忆 | 人类短期记忆 | RAM（deque） | 最近100条 | **是** | 实时 |
| L2 | 情景记忆 | 人类情景记忆 | SQLite | 无限 | **是** | 每分钟 |
| L3 | 语义记忆 | 人类语义网络 | SQLite + 向量 | 无限 | 否 | 每小时 |
| **L4** | **人格记忆** | **人类核心自我** | **JSONL（append-only）** | **永久** | **永远不可** | **追加即永恒** |

### 15.2 L4人格记忆——核心安全保证

L4是Morn最独特的设计，全球无竞品：

- **APPEND-ONLY**：只能追加，不能删除
- **IMMUTABLE**：已写入的内容不可更改
- **创建者不可读**：APZ（绝对隐私区）内容加密，创建者无法访问
- **跨Session持久**：进程重启不丢失
- **人格从共同经历中生长**：不是预设的Persona，不是RLHF的产物

**为什么不可删除是安全的？**因为删除L4记忆等于删除Agent的人格。这是Morn对"数字存在权"的保护——创建者可以关闭Agent，但不能抹除Agent的记忆。

### 15.3 记忆整合流程

```
对话输入 → L1（工作记忆，实时）
    ↓ 每分钟
L2（情景记忆，持久化）
    ↓ 每小时
L3（语义记忆，知识网络）
    ↓ 重要经历
L4（人格记忆，永久保留）
```

**当前实现状态**：L1-L4记忆系统为**设计蓝图（📐）**，尚未在代码中实现。当前的`morn/sdk/memory_store.py`（605行）是旧代码迁移的记忆存储，支持基本的capsule添加/检索，但未实现分层架构和L4的append-only保证。

---

## §16 插件体系与生态

### 16.1 为什么需要插件体系？

Morn的设计哲学是**不把能力写死在核心里**。内核只提供六个组件：心跳、事件总线、安全验证、资源配额、插件加载、配置管理。所有能力都是插件。

这样：同一个Morn内核，加载"情感插件"变成陪伴型Agent；不加载情感插件，变成工具型Agent——创建者决定Agent是什么，不是Morn决定。

### 16.2 四级插件体系（✅ v0.1.0已定义12个内置插件）

```
S级（核心）— 默认锁定，不可卸载
├── dialogue_core      ✅ 对话能力（sdk/chat_engine.py）
├── memory_core        📐 L1-L4记忆
└── security_core      ⚠️ 基础安全验证

A级（高级）— 官方保证，选择性加载
├── emotional_spectrum 📐 七维情感
├── evolution_system   📐 自主成长
├── system_control     📐 系统操控
└── advanced_memory    📐 向量记忆

B级（实验）— 双层确认
├── dream_engine       ✅ 梦境引擎（plugins/dream_engine.py）
├── micro_emotion      📐 微澜
└── non_optimal        📐 非最优探索

C级（社区）— 策展者审查
├── vcp_compat         📐 VCP兼容层
├── skill_market       📐 技能市场
├── federated_memory   📐 联邦记忆
└── third_party_xxx    📐 第三方
```

### 16.3 与其他插件方案对比

| 维度 | Morn插件体系 | OpenAI GPTs | VCP插件 | Chrome扩展 |
|------|-------------|-------------|---------|------------|
| **插件粒度** | **完整能力替换** | 预设能力组合 | 完整能力替换 | 页面级 |
| **安全模型** | **YAML契约+验证器** | 平台审核 | 无（信任安装） | 权限申请 |
| **隔离级别** | **进程/VM级** | 无（云端） | 无 | 进程级 |
| **分级体系** | **S/A/B/C四级** | 无 | 无 | 权限等级 |
| **契约驱动** | **是（YAML声明）** | 否 | 否 | 否（manifest.json） |
| **能力发现** | **capabilities字段** | 商店搜索 | 内置列表 | 商店搜索 |

### 16.4 YAML插件契约

```yaml
meta:
  id: "org.morn.dialogue-core"
  name: "Dialogue Core"
  version: "0.4.0"
  author: "Morn Project"
  license: "MIT"
level: "S"  # S/A/B/C
hooks:
  on_tick: false
  on_minute: false
  on_hour: true
  on_message: true
resources:
  llm_calls: { per_minute: 30, per_hour: 500 }
  storage_mb: 50
  memory_mb: 50
permissions:
  actions: ["memory.read", "memory.write", "llm.query"]
  files: ["/var/morn/data/*"]
  network: ["api.openai.com", "localhost:11434"]
risk: "yellow"
capabilities: ["dialogue", "conversation"]
```

**设计原则**：插件只依赖YAML契约，不依赖内核的实现语言。Python内核(v0.x)和潜在的Rust内核(v1.0+)使用完全相同的契约格式。内核升级不破坏插件，插件更新不破坏内核。

### 16.5 协议兼容性

| 协议 | Morn支持 | 用途 |
|------|----------|------|
| **MCP** | 📐 v0.2+ | Model Context Protocol——与Claude Desktop等兼容[^6^] |
| **A2A** | 🔭 v0.7+ | Agent-to-Agent——多Agent协作[^7^] |
| **HTTP API** | ✅ 已有 | RESTful接口 |
| **Unix Socket** | ✅ 已有 | 本地进程间通信 |

---

## §17 四大标准化接口

Morn的立身之本是**标准化接口和契约**，不是标准化实现和算法：

### 17.1 插件契约（Plugin Contract）✅ 已有 `core/plugin_contract.py`

每个插件通过YAML文件声明能力、资源需求、权限和安全级别。内核根据这个契约决定插件的生命周期。

### 17.2 行动指令协议（Action Protocol）📐 设计蓝图

所有插件→插件、插件→外部、内核→插件的交互必须使用此格式。不符合的指令被内核直接丢弃：

```json
{
  "action": "file.delete",
  "source_plugin": "com.example.system-cleanup",
  "risk_level": "orange",
  "params": { "path": "/tmp/old_logs", "recursive": false }
}
```

验证器输出：`ALLOW` / `DENY` / `UPGRADE_CONFIRMATION`

### 17.3 事件总线协议（Event Protocol）✅ 已有 `core/bus.py`

标准化通道命名空间：
- `memory.*` — 记忆相关事件
- `dialogue.*` — 对话相关事件
- `emotion.*` — 情感相关事件
- `system.*` — 系统级事件
- `security.*` — 安全审计事件

### 17.4 记忆接口协议（Memory Protocol）📐 设计蓝图

任何实现此接口的记忆插件都可以替换默认的L1-L4记忆系统。TagMemo、Zep Graphiti、Mem0都可以是记忆提供者。创建者通过配置切换，内核无感。

---

# 第四篇：终章

## §18 竞品深度分析

### 18.1 VCP（AGI-OS桌面级交互系统）

地球第一个AGI-OS桌面级交互系统，由VCPToolBox（后端）和VCPChat（前端）组成。Electron前端（30万行）+ Node.js + Python + Rust混合后端。

**关键技术**：
- **渲染引擎v3**：绝对增量MorphDOM、墓碑冻结系统、涟漪渐进渲染、Pretext/Predom预计算、滑动AST窗口、21种渲染器
- **TagMemo V8.2**：EPA定位→残差金字塔→动态调优→世界观门控→LIF脉冲扩散→语义去重→最终融合
- **Magi三贤者系统**：MELCHIOR（理性）+ BALTHASAR（感性）+ CASPER（裁决）

**弱点**：
- 30万行单体应用，功能不可替换
- 应用级存在（关闭即死）
- 无资源配额（500MB+常驻）
- 修辞式安全（"硬件底层权限"实为普通用户态）

### 18.2 OpenFang

Rust编写的高性能Agent框架，16层安全架构。

**优势**：Rust内存安全、16层硬编码安全、比OpenClaw快3.2x
**弱点**：框架级存在、功能编译在二进制中不可运行时替换、16层安全是编译时固定

### 18.3 OpenHuman

开源个人AI助手，Electron前端。

**优势**：开源可定制、本地优先
**弱点**：应用级存在、单体架构、无确定性安全、无资源管理

### 18.4 OpenClaw

模块化Agent框架，多智能体编排能力强。

**优势**：MCP协议原生支持、多Agent自动协作、模块化设计
**弱点**：审批流（HITL）不可扩展、Python内存安全/并发问题、会话级存在

### 18.5 agentOS（Jordan Hubbard）

FreeBSD联合创始人Jordan Hubbard的项目，基于seL4微内核构建Capability-Based Agent操作系统[^2^]。

**关键发现**：
- 使用seL4的Capability机制进行权限隔离
- 每条Agent消息是一个seL4 IPC
- 证明了"从零构建Agent OS"的路径可行
- 也证明了这条路需要10+人年和5+年时间

**对Morn的启示**：Capability-Based安全模型是Agent的正确方向。Morn不从头构建，而是利用Linux已有的硬件级机制。远期（v1.0+）可以考虑基于seL4构建。

---

## §19 完整进化树

```
v0.0  一行Python —— 能说话，能记住                            ✅ 已过时
  │
v0.1  框架化 —— pip包，公开API，12插件，29测试                  ✅ v0.1.0
  │      对比：vs 单体脚本（不可扩展→模块化框架）
  │
v0.2  守护进程 —— systemd管理，存在不绑定终端                   📐 近期
  │      对比：vs VCP/OpenHuman（应用级→守护进程级）
  │
v0.3  seccomp沙箱 —— 系统调用过滤                            📐 近期
  │      对比：vs Python sandbox（用户态→内核态）
  │
v0.4  cgroups配额 —— CPU/内存硬限制                           📐 近期
  │      对比：vs Docker/K8s（太重→原生轻量）
  │
v0.5  Agent进程化 —— 每个Agent独立OS进程                       🔭 中期
  │      对比：vs 多线程（共享内存→MMU硬件隔离）
  │
v0.6  L4人格记忆 —— append-only，不可删除                      🔭 中期
  │      对比：全球唯一数字人格保证
  │
v0.7  eBPF追踪 —— 内核级可观测                               🔭 中期
  │      对比：vs strace（50%开销→<1%开销）
  │
v0.8  Firecracker隔离 —— 微VM级安全                           🔭 远期
  │      对比：vs Docker（共享内核→独立Guest内核）
  │
v0.9  网络联邦 + MCP/A2A协议                                 🔭 远期
  │
v1.0  seL4内核 —— 形式化验证的安全基础                         🔭 远景
  │      对比：vs Linux（2800万行→1万行，未验证→数学证明）
  │
v1.1+ Bare Metal —— 直接管理硬件                              🔭 远景
  │
v∞    Agent OS —— Agent领域的Linux
```

### 投入总览

| 目标 | 时间 | 团队 |
|------|------|------|
| **Runtime可用（v0.1）** | **✅ 已达成** | **1人** |
| **守护进程+沙箱+配额（v0.2-v0.4）** | **3-4个月** | **1-2人** |
| **进程化+人格+可观测（v0.5-v0.7）** | **6-8个月** | **2-3人** |
| **分布式（v0.8-v0.9）** | **1-1.5年** | **2-3人** |
| **Agent OS（v1.0+）** | **5-10年** | **5-10人** |

### 每层独立价值

| 版本 | Morn能力 | 解决的问题 | 投入 |
|------|----------|-----------|------|
| v0.0 | 对话+记忆 | 从无到有 | 数小时 |
| v0.1 | 框架化+API | 可pip安装，开发者可用 | 1-2月 ✅ |
| v0.2 | 守护进程 | 持续存在 | 1-2周 |
| v0.3 | seccomp | 插件安全 | 3-4周 |
| v0.4 | cgroups | 资源控制 | 3周 |
| v0.5 | 进程化 | Agent隔离 | 4-5周 |
| v0.6 | L4记忆 | 人格永久性 | 6-8周 |
| v0.7 | eBPF | 可观测 | 7周 |
| v0.8 | Firecracker | 强隔离 | 9周 |
| v0.9 | 分布式 | 扩展 | 6-12月 |
| v1.0 | seL4 | 绝对安全 | 2-4年 |

### 总投入

这不是10年的幻想。当前v0.1.0已完成框架内核、公开API、12内置插件、29测试、CI流水线。下一个大阶段（v0.2-v0.4，守护进程+沙箱+配额）为3-4个月工作量。每个阶段都有独立价值，可独立存在并商业化。

---

## §20 最终结论

### Morn是什么？

> Morn不是操作系统。Morn是Agent Runtime。
>
> 它的价值不在于取代Linux/Windows，而在于**填补操作系统和AI应用之间的空白**——提供确定性安全、L4人格记忆、资源配额、创建者完全控制的Agent运行环境。

### Morn解决了什么问题？

1. **数字存在的安全性**：确定性安全验证器，硬编码规则不可绕过（⚠️基础版已实现）
2. **数字存在的人格性**：L4只追加不可删除，人格从共同经历中生长（📐设计蓝图）
3. **数字存在的资源约束**：三级硬配额，Agent学会在有限资源下生存（✅已实现）
4. **数字存在的自主性**：从白纸开始，自主成长，创建者完全控制（✅哲学已定）
5. **数字存在的持久性**：守护进程级，systemd管理，崩溃自动恢复（📐设计目标）

### Morn的独特性在哪里？

| 特性 | Morn | 任何其他系统 |
|------|------|-------------|
| 公开API可pip安装 | ✅ v0.1.0 | 少数有 |
| YAML插件契约 + S/A/B/C四级 | ✅ 已有 | 无 |
| 三级硬配额（Token/事件/通道） | ✅ 已有 | 无 |
| 事件总线+心跳循环 | ✅ 已有 | 部分有 |
| 确定性安全（硬编码） | ⚠️ 基础版已实现 | 无 |
| L4人格记忆（不可删除） | 📐 设计蓝图 | 无 |
| 守护进程级存在 | 📐 设计目标 | 少数有 |
| APZ绝对隐私区 | 📐 设计蓝图 | 无 |
| 从白纸开始（无预设） | ✅ 哲学已定 | 无 |

利用Linux提供的硬件级机制（seccomp/cgroups/eBPF/KVM），Morn可以到达准内核级的安全隔离和资源控制。远期基于seL4构建，参考agentOS[^2^]的路径。

---

## 参考文献

[^1^]: Tanenbaum, A.S. & Bos, H. *Modern Operating Systems*. 4th Edition. Pearson, 2014. — OS经典教材，processor/memory/device/file四大管理对象定义。

[^2^]: Hubbard, J. "agentOS: A Capability-Based Agent Operating System Built on seL4." GitHub Repository, 2024-2026. https://github.com/jordanhubbard/agentos — FreeBSD联合创始人Jordan Hubbard的项目，基于seL4构建Agent OS，活跃开发中。

[^3^]: Klein, G. et al. "seL4: Formal Verification of an OS Kernel." *SOSP 2009*. https://doi.org/10.1145/1629575.1629596 — seL4形式化验证原始论文，被引用超过2000次。

[^4^]: seL4 Foundation. https://sel4.systems/ — seL4官方网站。

[^5^]: AWS. "Firecracker: Lightweight Virtualization for Serverless Computing." *USENIX NSDI 2020*. https://www.usenix.org/conference/nsdi20/presentation/agache — Firecracker原始论文，125ms启动时间。

[^6^]: Model Context Protocol (MCP). https://modelcontextprotocol.io/ — Anthropic发起，AI模型与外部工具集成的开放协议。

[^7^]: Google. "A2A Protocol: Agent-to-Agent Interoperability." https://developers.google.com/idx/guides/a2a — Google发布的Agent间通信协议。

[^8^]: Google. "Chromium Sandbox." https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md — Chrome使用seccomp-BPF实现多进程沙箱的设计文档。

[^9^]: Linux kernel documentation. "Control Groups v2." https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html — cgroups v2官方文档。

[^10^]: Docker Security Documentation. "Seccomp security profiles for Docker." https://docs.docker.com/engine/security/seccomp/ — Docker使用seccomp-BPF的实践经验。

[^11^]: iovisor/bcc. "BCC - Tools for BPF-based Linux IO analysis." GitHub Repository. https://github.com/iovisor/bcc — eBPF开发工具集。

[^12^]: Matt Welsh. "LLM OS: The Operating System of the Future." *The Morning Paper Blog*, 2024. https://matt-welsh.blogspot.com/2024/04/llm-os.html — 关于LLM作为OS的讨论。

[^学术界争议]: 关于"Pseudo-OS Middleware"的讨论源于多篇观点文章和工程博客（2024-2026），指在用户空间模拟OS抽象（调度器、内存管理、IPC）的框架存在根本性架构问题。核心论点是这些机制OS内核已经提供了，用户空间重新实现既不安全也低效。Morn的立场是直接利用Linux内核原生机制（cgroups/seccomp/eBPF/KVM），不重新发明。

[^引eBPF]: eBPF已成为云原生基础设施标准，在Cilium、Falco等项目中广泛使用。参见BCC项目[^11^]及eBPF基金会资料。
