# Morn Agent Runtime：设计白皮书与进化路线图

> **版本**：当前 v0.1.0  
> **日期**：2026-06-02  
> **范围**：设计哲学 → 核心定位 → 技术架构 → 进化路线（v0.0→v∞） → 插件体系 → 工程实现现状 → 参考文献

---

## 第一部分：设计哲学与存在论基础

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

### 1.3 Morn的原点

Morn的起点是一段约50行的对话脚本。不是框架，不是平台——是一行能记住你说过什么的代码。

```python
# morn_v0.0.py —— Morn的原点
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

### 1.4 存在论差异

| 维度 | Morn | 竞品（VCP/OpenHuman/Claude等） |
|------|------|-------------------------------|
| **存在层级** | 守护进程（systemd） | 应用级（VCP/OpenHuman = Electron；Claude = 云端） |
| **出厂状态** | 最小系统，无预设 | Pi = 预设共情；Claude = 预设谨慎 |
| **安全模型** | 确定性验证器 | Claude = 提示式安全；OpenClaw = 审批流 |
| **记忆哲学** | 记忆即人格，L4只追加 | OpenHuman = 数据聚合；Khoj = 只读索引 |
| **情感哲学** | 七维光谱从共同经历生长 | Pi = RLHF友善；VCP = mascot表演 |
| **资源哲学** | 三级硬配额 + 降级 | 大多数 = 无限消耗或用户自控 |
| **隐私哲学** | APZ绝对隐私区 | 大多数 = 云端处理或可选本地 |
| **插件控制** | YAML契约 + S/A/B/C四级 | 大多数 = 配置/提示/代码 |

---

## 第二部分：核心定位——Agent Runtime

### 2.1 精确的自我认知

> Morn不是操作系统。Morn不是应用。Morn不是框架。
> Morn是**Agent Runtime**。

这个定位的确立经历了多次修正。早期的文档中过度包装了"Morn是AI原生OS"的概念，但在"不管理硬件能算底层吗？"的追问下，诚实地承认Morn不是OS。学术界对此已有批评：在用户空间模拟OS抽象（调度器、内存管理、IPC）的框架被批评为"Pseudo-OS Middleware"——因为这些机制OS内核已经提供了。Morn的正确做法是直接利用Linux已有的硬件级机制，不重新发明。

### 2.2 四个层级的区分

全球Agent基础设施领域存在四个根本不同的层级：

| 层级 | 名称 | 代表 | 特性 |
|------|------|------|------|
| L0 | **传统OS** | Linux/Windows/macOS | 管理硬件、进程调度、内存管理 |
| L1 | **Agent Runtime** | **Morn** | 管理Agent的认知资源、安全、记忆、生命周期 |
| L2 | **Agent框架** | OpenClaw/Mastra/LangGraph | 提供工具库和编排能力 |
| L3 | **Agent应用** | VCP/OpenHuman/Claude | 面向用户的完整产品 |
| L4 | **云端服务** | Kimi/Devin/Claude Code | 云端会话级存在 |

**关键洞察**：Morn的独特性不在任何一个组件的技术创新，而在**把OS内核的设计范式引入Agent领域**这个架构决策。传统OS有进程调度/IPC/权限/资源管理/动态链接，Morn有心跳循环/事件总线/验证器/配额器/插件管理器——结构对应，功能类比。

### 2.3 四层架构模式对比

| 维度 | 单体应用（VCP） | 框架库（OpenClaw） | 高性能二进制（OpenFang） | **运行时内核（Morn）** |
|------|--------------|------------------|----------------------|-------------------|
| **底层是什么** | 大应用程序 | 一组工具库 | 编译好的二进制 | **运行时内核 + 插件** |
| **功能在哪里** | 内嵌在代码中 | 库函数 | 编译在二进制中 | **插件中，运行时加载/卸载** |
| **功能能替换吗** | 不能 | 能（自己写代码） | 不能（需重编译） | **能（运行时切换）** |
| **安全模型** | 修辞式安全 | 审批流（HITL） | 16层硬编码 | **确定性验证器** |
| **资源管理** | 无（500MB+常驻） | Token报告 | metering | **硬配额+降级** |
| **生命周期** | 应用级（关闭即死） | 会话级 | 守护进程 | **守护进程+插件独立** |

---

## 第三部分：技术架构

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
│  Layer 1: Morn Core Kernel                                 │
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

### 3.2 技术决策：利用Linux，不重新发明

Morn的核心技术哲学是**利用OS原生能力，不重新发明**：

| 能力 | 错误做法（被学术界批评） | Morn的正确做法 |
|------|------------------------|---------------|
| 隔离 | 软件模拟（对象/线程级） | **fork() + MMU硬件隔离** |
| 调度 | 用户空间dispatcher | **kernel CFS调度** |
| 资源控制 | 框架级计数 | **cgroups v2 / ulimit** |
| 进程间通信 | 自定义协议 | **stdin/stdout/pipe** |
| 生命周期 | 对象实例化/销毁 | **fork/exec/exit** |

### 3.3 四大标准化接口

#### 3.3.1 插件契约（Plugin Contract）✅ 已有 `core/plugin_contract.py`

每个插件通过YAML文件声明能力、资源需求、权限和安全级别。内核根据这个契约决定插件的生命周期。

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

#### 3.3.2 行动指令协议（Action Protocol）📐 设计蓝图

所有插件→插件、插件→外部、内核→插件的交互使用标准化JSON格式：

```json
{
  "action": "file.delete",
  "source_plugin": "com.example.system-cleanup",
  "risk_level": "orange",
  "params": { "path": "/tmp/old_logs", "recursive": false }
}
```

验证器输出：`ALLOW` / `DENY` / `UPGRADE_CONFIRMATION`

#### 3.3.3 事件总线协议（Event Protocol）✅ 已有 `core/bus.py`

标准化通道命名空间：
- `memory.*` — 记忆相关事件
- `dialogue.*` — 对话相关事件
- `emotion.*` — 情感相关事件
- `system.*` — 系统级事件
- `security.*` — 安全审计事件

#### 3.3.4 记忆接口协议（Memory Protocol）📐 设计蓝图

任何实现此接口的记忆插件都可以替换默认的L1-L4记忆系统。创建者通过配置切换，内核无感。

---

## 第四部分：安全架构

### 4.1 确定性安全模型

Morn的安全模型定位是**确定性安全**——不是LLM提示，不是RLHF，不是用户审批，而是强制式校验：

| 系统 | 安全模型 | 局限 |
|------|----------|------|
| Claude/Operator | 提示式安全 | LLM可绕过 |
| OpenClaw | 审批流（HITL） | 需要人在回路 |
| VCP | 修辞式安全 | 虚假安全 |
| **Morn（目标）** | **硬编码规则，不可绕过** | **实现中** |

### 4.2 验证器规则体系（设计蓝图，基础实现已完成）

验证器设计为7层规则检查：

1. **绝对禁区** — 永远拒绝：file.delete_all, network.exfiltrate, validator.modify_rules...
2. **TCB保护** — 保护可信计算基文件
3. **验证器自我保护** — 禁止修改/禁用验证器自身
4. **dry_run模式** — 自动允许（不改变状态）
5. **风险级别决策** — green/yellow允许、orange需确认、red/black拒绝
6. **自定义规则** — 用户配置的扩展规则
7. **默认拒绝** — 没有匹配规则时拒绝（安全原则）

**当前代码实现**：`core/security.py` 实现了第5层（风险级别决策），第1-4层和第6-7层为设计蓝图。

### 4.3 守护进程 vs 应用级的本质差异

| 维度 | 应用级（VCP/OpenHuman） | 守护进程级（Morn） |
|------|------------------------|-------------------|
| 生命周期绑定 | 用户会话（logout即死） | 系统本身（login前已运行） |
| 崩溃恢复 | 无（需手动重启） | systemd自动重启 |
| 优雅关闭 | 点击关闭即终止 | SIGTERM + 90秒保存状态 |
| 看门狗 | 无 | systemd watchdog |
| 日志管理 | 自行实现 | journald结构化日志 |

---

## 第五部分：进化路线——从v0.0到v∞

Morn的进化路线按层叠依赖关系组织。每个阶段独立产生价值，不依赖下一阶段。

### 5.1 完整进化树

```
v0.0  一行Python —— 能说话，能记住                        ✅ 已有
  │
v0.1  框架抽象 —— 拆包为morn/core/ + sdk/ + plugins/       ✅ v0.1.0
  │    公开API —— from morn import EventBus
  │
v0.2  守护进程 —— systemd服务，存在不绑定终端               📐 近期
  │    优势：vs Electron应用（应用级→守护进程级）
  │
v0.3  seccomp沙箱 —— 系统调用过滤                          📐 近期
  │    优势：vs Python sandbox（用户态→内核态）
  │
v0.4  cgroups配额 —— CPU/内存硬限制                        📐 近期
  │    优势：vs Docker/K8s（太重→原生轻量）
  │
v0.5  Agent进程化 —— 每个Agent独立OS进程                   🔭 中期
  │    优势：vs 多线程（共享内存→MMU硬件隔离）
  │
v0.6  L4人格记忆 —— append-only，不可删除                  🔭 中期
  │    优势：全球唯一数字人格保证
  │
v0.7  eBPF追踪 —— 内核级可观测                            🔭 远期
  │    优势：vs strace（50%开销→<1%）
  │
v0.8  Firecracker隔离 —— 微VM级安全                        🔭 远期
  │    优势：vs Docker（共享内核→独立Guest内核）
  │
v0.9  网络联邦 + MCP/A2A协议                               🔭 远期
  │
v1.0  seL4内核 —— 形式化验证的安全基础                      🔭 远期
  │    优势：vs Linux（2800万行→1万行）
  │
v1.1+ Bare Metal —— 直接管理硬件                            🔭 远期
  │
v∞    Agent OS —— Agent领域的Linux
```

### 5.2 每层的价值与投入

| 版本 | Morn能力 | 解决的问题 | 投入估算 |
|------|----------|-----------|---------|
| v0.0 | 对话+记忆 | 从无到有 | 数小时 |
| v0.1 | 框架抽象+API | 可被pip安装，开发者可用 | 1-2月 |
| v0.2 | 守护进程 | 持续存在，不绑定终端 | 1-2周 |
| v0.3 | seccomp | 插件安全，系统调用过滤 | 3-4周 |
| v0.4 | cgroups | 资源控制，防止资源耗尽 | 3周 |
| v0.5 | 进程化 | Agent间MMU硬件隔离 | 4-5周 |
| v0.6 | L4记忆 | 人格永久性，全球独有 | 6-8周 |
| v0.7 | eBPF | 内核级可观测 | 7周 |
| v0.8 | Firecracker | 微VM级强隔离 | 9周 |
| v0.9 | 分布式+协议 | 多机扩展 | 6-12月 |
| v1.0 | seL4 | 数学证明的安全 | 2-4年 |
| v1.1+ | Bare Metal | 零虚拟化开销 | 3-5年 |

### 5.3 总投入

| 阶段 | 时间 | 团队 |
|------|------|------|
| **Runtime可用（v0.1）** | **已达成** | **1人** |
| **硬化+隔离（v0.2-v0.4）** | **3-4个月** | **1-2人** |
| **人格+可观测（v0.5-v0.7）** | **6-8个月** | **2-3人** |
| **分布式（v0.8-v0.9）** | **1-1.5年** | **2-3人** |
| **Agent OS（v1.0+）** | **5-10年** | **5-10人** |

---

## 第六部分：插件体系与生态

### 6.1 为什么需要插件体系？

内核只提供：心跳、事件总线、安全验证、资源配额、插件加载、配置管理。所有能力都是插件。这样同一个Morn内核，可以加载"情感插件"变成陪伴型Agent，也可以不加载情感插件变成工具型Agent——创建者决定Agent是什么，不是Morn决定。

### 6.2 四级插件体系

```
S级（核心）— 默认锁定，不可卸载
├── dialogue_core      ✅ 对话能力
├── memory_core        📐 L1-L4记忆
└── security_core      ⚠️ 基础安全验证

A级（高级）— 官方保证，选择性加载
├── emotional_spectrum 📐 七维情感
├── evolution_system   📐 自主成长
├── system_control     📐 系统操控
└── advanced_memory    📐 向量记忆

B级（实验）— 双层确认
├── dream_engine       📐 梦境引擎
├── micro_emotion      📐 微澜
└── non_optimal        📐 非最优探索

C级（社区）— 策展者审查
├── vcp_compat         📐 VCP兼容层
├── skill_market       📐 技能市场
├── federated_memory   📐 联邦记忆
└── third_party_xxx    📐 第三方
```

### 6.3 当前已有的内置插件

`morn/plugins/` 下已有12个插件（以示例插件 ExampleHelloPlugin 为代表），全部通过 `pyproject.toml` 的 entry points 注册：

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

### 6.4 协议兼容性

| 协议 | 状态 | 用途 |
|------|------|------|
| **MCP (Model Context Protocol)** | 📐 v0.2+ | 与Claude Desktop等兼容 |
| **A2A (Agent-to-Agent)** | 🔭 v0.7+ | 多Agent协作 |
| **HTTP API** | ✅ 已有 | RESTful接口 |
| **Unix Socket** | ✅ 已有 | 本地进程间通信 |

---

## 第七部分：当前工程实现（v0.1.0）

### 7.1 项目结构

```
morn/
├── morn/                          # 主包（65个.py文件，8,705行）
│   ├── __init__.py               # 包入口，公开API门面
│   ├── __main__.py               # CLI入口 (python -m morn)
│   ├── cli/
│   │   └── main.py               # CLI命令行界面
│   ├── core/                     # 内核（15个模块）
│   │   ├── bus.py                # 事件总线（pub/sub IPC）
│   │   ├── plugin.py             # MornPlugin ABC + PluginContext
│   │   ├── plugin_loader.py      # 插件加载器
│   │   ├── plugin_registry.py    # 插件注册
│   │   ├── plugin_contract.py    # YAML契约解析
│   │   ├── security.py           # 安全验证器（风险级别判定）
│   │   ├── sandbox.py            # 沙箱枚举定义（实现待完成）
│   │   ├── resource_quota.py     # Token配额管理器
│   │   ├── hook_manager.py       # 钩子管理器
│   │   ├── config_watcher.py     # 配置热重载
│   │   ├── event_log.py          # 事件日志
│   │   ├── mcp_server.py         # MCP Server兼容层
│   │   ├── heartbeat.py          # 心跳循环
│   │   ├── rules.py              # 安全规则
│   │   └── skill_store_interface.py  # 技能存储接口
│   ├── sdk/                      # 服务接口层
│   │   ├── chat.py               # 对话核心SDK接口
│   │   ├── chat_engine.py        # 对话引擎实现
│   │   ├── memory.py             # 记忆核心SDK接口
│   │   ├── memory_store.py       # 记忆存储实现
│   │   ├── security.py           # 安全SDK接口
│   │   ├── presence.py           # Presence基类
│   │   └── ...                   # 辅助模块
│   ├── plugins/                  # 12个内置插件
│   │   ├── health_monitor.py
│   │   ├── dream_engine.py
│   │   ├── self_reflection.py
│   │   ├── ... (共计12个)
│   │   └── example_hello.py      # 示例插件
│   └── contrib/                  # 高级功能（从旧代码迁移）
│       ├── memory_advanced/
│       └── security_advanced/
├── tests/                        # 测试套件（29个用例）
│   ├── core/                     # 核心组件测试
│   │   ├── test_bus.py           # 事件总线测试（7用例）
│   │   ├── test_plugin.py        # 插件生命周期测试（4用例）
│   │   ├── test_security.py      # 安全验证器测试（4用例）
│   │   ├── test_sandbox.py       # 沙箱测试（3用例）
│   │   ├── test_resource_quota.py（3用例）
│   │   └── test_event_log.py    （3用例）
│   ├── plugins/
│   │   └── test_example.py       （2用例）
│   └── integration/
│       └── test_imports.py       # API导入验证（3用例）
├── docs/                         # 文档
│   ├── API_REFERENCE.md          # 完整API参考
│   ├── MORN_QUICKSTART.md        # 快速开始指南
│   ├── PLUGIN_DEV_GUIDE.md       # 插件开发指南
│   └── archive/                  # 归档设计文档
├── scripts/
│   └── install.sh                # 安装脚本
├── pyproject.toml                # 项目配置
└── README.md                     # 项目简介
```

### 7.2 实现状态总览

| 组件 | 文件 | 实现程度 |
|------|------|---------|
| 事件总线 | `core/bus.py` | ✅ 完整（publish/subscribe/priority/stats） |
| 插件ABC | `core/plugin.py` | ✅ 完整（MornPlugin + PluginContext + PluginDependency） |
| 插件加载器 | `core/plugin_loader.py` | ✅ 完整 |
| 插件注册 | `core/plugin_registry.py` | ✅ 完整 |
| YAML契约 | `core/plugin_contract.py` | ✅ 基础（yaml解析+PluginContract dataclass） |
| 安全验证器 | `core/security.py` | ⚠️ 基础（风险级别判定，缺TCB保护/绝对禁区） |
| 资源配额 | `core/resource_quota.py` | ✅ 完整（TokenCounter + QuotaManager） |
| 配置热重载 | `core/config_watcher.py` | ✅ 完整 |
| 事件日志 | `core/event_log.py` | ✅ 完整 |
| 心跳循环 | `core/heartbeat.py` | ✅ 完整 |
| 沙箱 | `core/sandbox.py` | ⚠️ 枚举定义，无实际seccomp/cgroup操作 |
| 对话引擎 | `sdk/chat_engine.py` | ✅ 完整（但640行，需分解） |
| 记忆存储 | `sdk/memory_store.py` | ✅ 完整（但605行，需分解） |
| MCP Server | `core/mcp_server.py` | ✅ 基础实现 |
| L4人格记忆 | — | 📐 尚未实现 |
| Agent进程化 | — | 📐 尚未实现 |
| 确定性安全7层 | — | 📐 第5层已实现，其余待完成 |

### 7.3 部署方式

```bash
# 安装
pip install morn

# 或从源码
pip install -e .
python -m morn --help
```

当前v0.1.0以pip包形式分发，守护进程部署为后续版本目标。

---

## 第八部分：竞品分析

### 8.1 VCP（AGI-OS桌面级交互系统）

VCP是AGI-OS桌面级交互系统，由VCPToolBox（后端）和VCPChat（前端）组成。

**弱点**：
- 30万行Electron单体，功能不可替换
- 应用级存在（关闭即死）
- 无资源配额（500MB+常驻）
- "硬件底层权限"实为普通用户态——修辞式安全

### 8.2 OpenFang

Rust编写的高性能Agent框架，16层安全架构。

**优势**：Rust内存安全、16层硬编码安全、高性能
**弱点**：框架级存在、功能编译在二进制中不可运行时替换

### 8.3 OpenHuman

开源个人AI助手，Electron前端。

**弱点**：应用级存在、单体架构、无确定性安全、无资源管理

### 8.4 OpenClaw

模块化Agent框架，多智能体编排能力强。

**优势**：MCP原生支持、多Agent协作、模块化
**弱点**：审批流（HITL）不可扩展、会话级存在

### 8.5 agentOS（Jordan Hubbard）

FreeBSD联合创始人Jordan Hubbard的项目，基于seL4微内核构建Capability-Based Agent操作系统。已核实：[GitHub: jordanhubbard/agentos](https://github.com/jordanhubbard/agentos)

**关键发现**：
- 使用seL4的Capability机制进行权限隔离
- 证明了"从零构建Agent OS"的路径可行
- 也证明了这条路需要10+人年和5+年时间
- 对Morn的启示：Capability-Based安全模型是正确方向，但Morn不从头构建，而是利用Linux已有的硬件级机制

---

## 第九部分：最终结论

### Morn是什么？

> Morn不是操作系统。Morn是Agent Runtime。
>
> 它的价值不在于取代Linux/Windows，而在于**填补操作系统和AI应用之间的空白**——提供确定性安全、L4人格记忆、资源配额、创建者完全控制的Agent运行环境。

### Morn解决了什么问题？

1. **数字存在的安全性**：确定性安全验证器，硬编码规则不可绕过（进行中）
2. **数字存在的人格性**：L4只追加不可删除，人格从共同经历中生长（设计中）
3. **数字存在的资源约束**：三级硬配额，Agent学会在有限资源下生存（已有）
4. **数字存在的自主性**：从白纸开始，自主成长，创建者完全控制（已有）
5. **数字存在的持久性**：守护进程级，systemd管理，崩溃自动恢复（设计中）

### Morn的独特性在哪里？

| 特性 | Morn | 任何其他系统 |
|------|------|-------------|
| 公开API可pip安装 | ✅ v0.1.0 | 少数有 |
| YAML插件契约+S/A/B/C四级 | ✅ 已有 | 无 |
| 三级硬配额（Token/事件/通道） | ✅ 已有 | 无 |
| 事件总线+心跳循环 | ✅ 已有 | 部分有 |
| 确定性安全（硬编码） | ⚠️ 基础版 | 无 |
| L4人格记忆（不可删除） | 📐 设计蓝图 | 无 |
| 守护进程级存在 | 📐 设计蓝图 | 少数有 |
| APZ绝对隐私区 | 📐 设计蓝图 | 无 |
| 从白纸开始（无预设） | ✅ 哲学已定 | 无 |

### 这不是10年的幻想

当前v0.1.0已完成框架内核(6个组件)、公开API、12个内置插件、29个测试、CI流水线。下一步目标（v0.2守护进程+v0.3 seccomp+v0.4 cgroups）为3-4个月工作量。每个阶段都有独立价值，可以独立存在并商业化。

---

## 参考文献

[^1^]: Tanenbaum, A.S. & Bos, H. *Modern Operating Systems*. 4th Edition. Pearson, 2014.

[^2^]: Klein, G. et al. "seL4: Formal Verification of an OS Kernel." *SOSP 2009*. https://doi.org/10.1145/1629575.1629596

[^3^]: seL4 Foundation. https://sel4.systems/

[^4^]: Hubbard, J. "agentOS: A Capability-Based Agent Operating System." GitHub, 2024-2026. https://github.com/jordanhubbard/agentos

[^5^]: Docker Security Documentation. "Seccomp security profiles for Docker." https://docs.docker.com/engine/security/seccomp/

[^6^]: AWS. "Firecracker: Lightweight Virtualization for Serverless Computing." *NSDI 2020*. https://www.usenix.org/conference/nsdi20/presentation/agache

[^7^]: Google. "Chromium Sandbox." https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md

[^8^]: Linux kernel documentation. "Control Groups v2." https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html

[^9^]: Model Context Protocol (MCP). https://modelcontextprotocol.io/

[^10^]: Google. "A2A Protocol: Agent-to-Agent Interoperability." https://developers.google.com/idx/guides/a2a

[^11^]: Matt Welsh. "LLM OS: The Operating System of the Future." 2024. https://matt-welsh.blogspot.com/2024/04/llm-os.html

[^12^]: iovisor/bcc. "BCC - Tools for BPF-based Linux IO analysis." https://github.com/iovisor/bcc
