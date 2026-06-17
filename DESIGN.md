# Morn

> 你的桌面 AI 创作系统
> 设计总纲 · 2026年6月 · v8.1 最终版（1.0 上线版 + 代码健康）

---

## 目录

1. [什么是 Morn](#一what-is-morn)
2. [市场验证与定位](#二市场验证与定位)
3. [核心架构](#三核心架构)
4. [创作台（Studio）](#四创作台studio)
5. [工作台（Workbench）](#五工作台workbench)
6. [管理台（Console）](#六管理台console)
7. [渠道与连接](#七渠道与连接)
8. [Morn Hub（生态接口）](#八morn-hub生态接口)
9. [安全模型](#九安全模型)
10. [开箱即用体验](#十开箱即用体验)
11. [插件系统（Plugin System）](#十一插件系统plugin-system)
12. [执行路线图](#十二执行路线图)
13. [缺口分析](#十三缺口分析)
14. [代码质量要求](#十四代码质量要求)
15. [竞品全景](#十五竞品全景)
16. [参考来源](#十六参考来源)
17. [附录 A：Tauri API 接口定义](#附录-atauri-api-接口定义)
18. [附录 B：数据模型与存储](#附录-b数据模型与存储)
19. [附录 C：UI 交互流程](#附录-cui-交互流程)

---

## 一、What is Morn

Morn 是一个跑在 Windows 桌面的 AI 创作操作系统。

### 核心理念：自下而上的组装

```
一句话 → 从 Hub 获取原子组件（任何类型）→ 组装成 Agent
                                                    ↓
                                       多 Agent 自由连线 → 团队
                                                    ↓
                               Agent / 团队 / 原子组件 → 发布到 Hub
                                                    ↓
                                       别人拿来再组合 → 生态滚起来
```

Morn 不给用户造笼子，给用户造工具。WorkBuddy 给你 10 个模板，Morn 给你一把扳手让你自己焊。

### 架构核心：三台中心，Hub 是接口

Morn 的架构核心是三个「台」，不是 Hub：

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│ Workbench│◄──►│  Studio  │◄──►│ Console  │
│  工作台   │    │  创作台   │    │  管理台   │
└─────┬────┘    └────┬─────┘    └────┬─────┘
      │              │               │
      └──────────────┼───────────────┘
                     │
            ┌────────┴────────┐
            │   Morn Hub      │  ← 开放生态接口
            │  不是架构中心    │     是连接三台到外部世界的接口
            └─────────────────┘
```

三台（Workbench/Studio/Console）是架构核心，共享同一套基础设施（COO/Registry/Security）。Hub 不是第四台，是三台各自的一个功能入口。详见 §2.4。

### 最终目标

> **Morn — 你的桌面 AI 创作系统。**
>
> 不是「一人公司的工具」，而是你可以用 AI 在桌面上创造任何东西的平台。
>
> - 你是普通用户 → 装别人做好的 Agent → 开箱即用
> - 你是一个创始人 → 组一个虚拟团队 → 一人公司
> - 你是创作者 → 自己设计工作流/Agent → 发布到 Hub
> - 你是开发者 → 扩展组件类型、注册新能力 → 平台进化
>
> Morn 本身是一个空桌面，用户往上面搭什么 = 用户的创作，
> 搭出来的东西可以自己用、可以卖、可以组合。
> 一人公司是第一个 killer use case，但不是终点。

**三个关键差异：**

| | 传统平台（WorkBuddy/Coze/Dify） | Morn |
|---|---|---|
| 组件类型 | 平台固定，等发版 | **可扩展，任何人可注册新类型** |
| Agent 组合 | 预设模板 | **用户自由连线** |
| 市场位置 | 顶层（卖成品） | **贯穿所有层（原子→Agent→团队）** |

三台一体：

```
🛠 工作台（Workbench）— 日常使用
   下指令，COO 拆任务、派活、跟进度、拿结果
   可以操控整个电脑：文件、应用、系统设置、桌面、浏览器
   后台任务、定时任务、多任务并行

🎨 创作台（Studio）— 组件创作
   从底层搭起：工具、知识、技能、人格、记忆、模型
   任意组合成 Agent → 组合成团队 → 发布到工作台或市场
   画布拖拽 + 代码编写 + 即时测试

📋 管理台（Console）— 运营管理
   管理所有组件和 Agent 的拓扑连接
   看成本、看绩效、做审批、调治理
   系统监控、安全事件、市场数据
```

三台共享同一套底层：同一个 COO、同一个 Registry、同一个知识库、同一个安全体系。三台之间无缝切换——从工作台说"帮我改一下 data-agent 的 prompt"→自动切到创作台。

---

## 二、市场验证与定位

2026年6月，桌面 AI Agent 市场已进入"超级应用大爆发"阶段。基于 200+ 来源的深度调研，Morn 的定位和方向获得了充分的行业验证。

### 2.1 "一人公司"时代已经到来

一人公司不是未来概念，是正在发生的现实。

**Medvi：$20K 启动 → $1.8B 估值，2 个员工**
- Matthew Gallagher，41岁，自学编程，2024年9月用 $20,000 启动
- 第 14 个月营收预计 $1.8B，只有 2 个员工（他和兄弟）
- 使用 ChatGPT、Claude、AI 工具做全部工作
- NYT、Forbes、Inc. 头条报道 — 一人公司运动最重要的 Proof Point

**硬数据：**
- 美国一人公司人数：**29.8M**
- 百万美元营收企业中一人创始人占比：**38%**
- 一人技术栈月成本：**$300-500**（替代 10-20 人团队，成本下降 90%）
- 营业利润率：**60-80%**（vs 传统 10-20%）
- Sam Altman 预测：**第一个一人 $10B 公司很快会出现**

### 2.2 2026 年桌面 AI 工作台市场全景

**六大设计范式已清晰成型：**

| 范式 | 描述 | 代表产品 |
|------|------|---------|
| **桌面 AI 工作台** | 预装即用，零模型配置 | Marvis, WorkBuddy, Claude Cowork, Cherry Studio, Kimi Work |
| **虚拟公司/团队** | Agent 以公司/团队形式组织 | Paperclip(70K⭐), ChatDev(33K⭐), MetaGPT(40K⭐) |
| **编排平台** | 可视化工作流编排 | Dify(120K⭐), n8n, CrewAI, LangGraph |
| **AI 员工平台** | 按"员工"而非"工具"销售 | Lindy, Sintra, ServiceNow |
| **Agent 即 OS** | Agent 成为操作系统第一公民 | Microsoft Project Solara, Win11 Agent Workspace |
| **Agent 协议层** | 三大互操作协议 | MCP(工具), A2A(Agent), ADP(训练数据) |

**关键商业产品对 Morn 的启示：**

| 产品 | 亮点 | 对 Morn 的启示 |
|------|------|---------------|
| **Marvis** (腾讯.05) | 6 Agent + 虚拟办公室 + 离线隐私 | 最直接对标：预装多Agent模式已验证 |
| **Claude Cowork** (Anthropic.01) | AI同事桌面，多步骤知识工作 | 从 Chatbot 到 Cowork 的范式转移 |
| **OpenAI Super App** (规划) | ChatGPT+Codex+Atlas 三合一 | 业界最大玩家验证了 Morn 的"All-in-One"哲学 |
| **Kimi Work** (月之暗面.05) | 300 Agent Swarm 并行 | 大规模 Agent 并行的可行性验证 |
| **Paperclip** (70K⭐) | 组织架构 + Company 商店 | 完全验证了 Morn"团队蓝图+市场"方向 |
| **Microsoft Project Solara** | AOSP 新 OS，Agent = 进程 | Morn 的"Agent=进程"隐喻获行业最大玩家背书 |

### 2.3 一人公司的标准技术栈

一人公司月均 **$300-500** 的工具投入，替代 10-20 人团队：

| 层 | 工具 | 月费 | 替代角色 |
|----|------|------|---------|
| 🧠 开发 | Claude Code, Cursor, GitHub Copilot | $20-200 | 3-5 个工程师 |
| 📣 营销 | ChatGPT, Claude, Midjourney | $50-100 | 营销团队 |
| ⚙️ 自动化 | n8n, Zapier AI, Make | $20-50 | 运营人员 |
| 💰 财务/法务 | QuickBooks AI, LawPal | $30-50 | 会计+法务 |
| 🤝 客服 | Lindy, Intercom AI, Bland.ai | $50-100 | 客服团队 |
| 📊 分析 | Cursor, ChatGPT Advanced Data | $10-50 | 数据分析师 |

**Morn 的定位：一人公司的桌面操作系统**——覆盖上述全套能力，并且是唯一本地+开源+可组装的选择。

### 2.4 Morn 的战略定位

**Morn = 桌面 AI 创作系统**

一人公司是它的第一个 killer use case，但不是终点。

```
普通用户装 Agent → 开箱即用
创始人组团队    → 一人公司
创作者卖设计   → 市场生态
开发者扩展类型  → 平台进化
```

```
一人公司需要什么    →    Morn 提供什么
────────────────────────────────────
一个聪明的大脑     →    Agent 聊天+执行
能自己写代码       →    Code Agent（Rust 后端）
能自己管文件       →    文件系统 Agent
能自己分析数据     →    工具系统+LLM 推理
能自己营销        →    可接入 Web/API
能自己客服        →    多渠道通信
所有东西在一个地方  →    桌面 All-in-One
```

### 2.5 核心差异化优势

1. **「自下而上组装」哲学** — 从原子组件组装 Agent 再组织成团队，无竞品做到
2. **三层市场**（组件/Agent/蓝图）— 无人做到
3. **Tauri 原生桌面** — 比 Electron 方案体积小 10 倍，性能更好
4. **Rust 内存安全** — 对比 OpenClaw 安全危机（2026.06 4 个连锁漏洞，18 万+企业受影响），天然更安全
5. **本地优先 + 隐私** — 数据不出机器，符合 EU AI Act 等监管要求
6. **开源 + 自托管** — 不被锁定
7. **天然兼容 SKILL.md** — 可直接吃掉 20,000+ 现有技能生态

### 2.6 市场与融资数据

| 指标 | 数据 | 来源 |
|------|------|------|
| 2026年 Agent 市场规模 | $10.9B | Grand View |
| 2030年预测 | $50.31B (CAGR 45.8%) | Grand View |
| Top 25 Agent 公司融资(2025-2026) | $25B+ | AgentMarketCap |
| 全球 AI VC 总额 | $258.7B (占全部 VC 的 61%) | Stanford HAI |
| 企业平均预期 Agent ROI | 171% | Gartner |
| 12 个月内回本比例 | 41% | Gartner |

---

## 三、核心架构

### 3.1 四层组合模型

```
Layer 0: 基础设施
  LLM 模型 / 数据源 / 操作系统 / 网络 / MCP

      ↓ 组合

Layer 1: 原子组件（可扩展类型系统）
  内置类型（7 种）：
    工具（Tool）— 单一操作，get_kline、web_search、send_msg
    知识（Knowledge）— 静态信息，股票代码库、公司档案
    技能（Skill）— 流程模板，技术分析流程、报告生成
    人格（Persona）— 思维模型 + 行为定义，analyst、writer
    记忆（Memory）— 存储配置：短期/长期/经验/工作记忆
    模型（Model）— LLM 选择 + 参数 + 成本档位
    渠道（Channel）— Telegram、企微、钉钉、飞书

  7 种是内置基础，不是天花板。
  任何人可以通过市场发布新的组件类型（§8 市场生态）。
  新类型安装后，在 Studio 里像原生类型一样使用。

      ↓ 组合

Layer 2: Agent（一车头多挂车）
  任意组件通过标准接口连接
  data-agent = tools(get_kline, calc_macd) + knowledge(stock_db)
               + skill(technical_analysis) + persona(analyst)
  
  也可以没有车头：纯工具链
  Timer → get_kline → calc_macd → write_file（无 LLM 参与）

      ↓ 组合

Layer 3: 团队（多 Agent + 工作流）
  股票研究团队 = [data-agent, search-agent, fin-plot, chat-agent]
  协作模式：链式 / 并行 / 会诊 / 招标 / 接力 / 师徒
```

### 3.2 标准组件接口

所有组件都实现这组接口，确保任意组件可以自由连接——像卡车的第五轮耦合：

```rust
// 所有组件的基础
trait Component {
    fn id(&self) -> &str;
    fn type_name(&self) -> &str;
    fn init(&mut self) -> Result<()>;
    fn run(&mut self) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn health_check(&self) -> HealthStatus;
}

// IO 端口（组件之间的连接方式）
trait IOComponent: Component {
    fn ports(&self) -> Vec<Port>;           // 暴露哪些端口
    fn send(&mut self, port: &str, data: Data) -> Result<()>;
    fn recv(&mut self, port: &str) -> Result<Option<Data>>;
}

// 事件总线（松耦合通信）
trait EventBus {
    fn publish(&self, event: Event);
    fn subscribe(&self, event_type: &str, handler: fn(Event));
}

// 安全声明
trait SecureComponent: Component {
    fn required_permissions(&self) -> Vec<Permission>;
}
```

### 3.3 系统架构

```
┌──────────────────────────────────────────────────────────────────┐
│                       Morn Desktop                               │
│              Tauri (Rust) · Windows 原生 .exe                    │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  接入层                                                          │
│  桌面端UI | CLI | 企微 | 钉钉 | 飞书 | 小程序 | PWA | API       │
│  Channel Adapter : 统一消息格式转换                               │
│                                                                  │
│  ┌── COO (Supervisor) ─────────────────────────────────────┐    │
│  │  1. Intent Parser (NL → 结构化意图, 基于 LLM)           │    │
│  │  2. Planner (意图分解为 DAG 计划, 查 Registry)          │    │
│  │  3. Router(匹配有效组件, 信任评分加权)                   │    │
│  │  4. Scheduler (DAG 调度: 并行/串行/重试/超时/审批)      │    │
│  │  COO 决策: 常规自动 / 需确认给建议 / 高风险必批 / 可学习 │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌── Registry (能力注册中心) ────────────────────────────────┐  │
│  │  所有工具/知识/技能/人格/记忆/模型/Agent/团队的统一注册表    │  │
│  │  信任评分(基于成功率+响应速度+用户反馈)                     │  │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌── Task Engine (执行引擎) ───────────────────────────────┐    │
│  │  DAG 调度 | 状态管理 | 超时/重试 | 审批点 | 子进程隔离  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌── Dual-LLM 安全 ───────────────────────────────────────┐    │
│  │  主 LLM 正常执行 / 副 LLM 安全检查 / 6 个检查点        │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  SQLite (agents/capabilities/tasks/executions/decisions/bindings)│
└──────────────────────────────────────────────────────────────────┘
```

### 3.4 进程模型

```
Windows 开机 → 注册表启动 Morn.exe → 系统托盘常驻

一个 .exe，多线程：
├─ 主线程：Tauri UI + 事件循环
├─ COO 线程：Supervisor 推理循环
├─ 执行线程池：Task Engine 并行子任务
│   └── 每个子任务独立子进程（Agent 崩溃隔离）
├─ 渠道线程池：IM 消息收发
└─ 托盘线程：系统通知 + 右键菜单
```

### 3.5 可扩展组件类型系统

Morn 的组件架构支持"从底层搭起"，且**组件类型本身也是可扩展的**。

#### 内置类型 vs 可注册类型

```
内置7种（core 自带）：
  memory / tools / llm / channels / persona / skills / security

可注册类型（任何人可发布）：

  ┌ 用户从市场安装「视觉识别组件类型」
  │ → 系统多了新类型: vision_model
  │ → Studio 自动显示新类型面板
  │ → 像原生类型一样创建/配置/使用
  │ → 可被其他 Agent 引用，可分享到市场
  │
  └ 用户安装「区块链连接器」
    → 系统多了: chain_connector
    → 没人预见过这个类型，但它和其他组件一样可组合
```

#### 组件类型注册机制

不硬编码类型，用 ComponentTypeDef 描述新类型：

```rust
// 安装一个 vision_model 组件类型时注册的定义
ComponentTypeDef {
  type_name: "vision_model",          // 新类型的名称
  interfaces: ["predict", "embed"],    // 必须实现的接口
  config_schema: { /* JSON Schema */ },// 配置项 schema
  implements: [],                      // 依赖的其他类型
  author: "user_abc",
  version: "1.0.0",
}
```

```
系统启动时 → 加载内置7种类型
用户从市场装新类型 → 注册到 TypeRegistry
Studio 自动更新面板 → 新类型可用
用户创建 vision_model 实例 → 配置 → 像内置类型一样使用
```

#### 原子组件清单（内置 + 示例扩展）

| 组件类型 | 类型来源 | 可选实例 | 组合规则 |
|---------|---------|---------|---------|
| 🧠 记忆 | 内置 | 工作记忆/情景记忆/语义记忆/长程经验/图谱记忆/闪存 | 每Agent至少1种，最多3种 |
| 🛠 工具 | 内置 | web_search/file_ops/code_exec/get_kline/calc_macd/... | 至少1个，无上限 |
| ⚙ LLM | 内置 | 本地GGUF/云端GPT/DeepSeek/Claude/混合(Hybrid) | 至少1个，最多3个 |
| 📡 渠道 | 内置 | Telegram/WeChat/钉钉/飞书/QQ/CLI/桌面 | 可选，0-N，默认桌面 |
| 👤 人格 | 内置 | 52预置(分析师/写手/研究员/...) + 自定义 | 可选，0-1个 |
| 🛠 技能 | 内置 | 技术分析/报告生成/代码审查/数据清理/... | 可选，0-N |
| 🔐 安全策略 | 内置 | L1(自由)/L2(需审批)/L3(仅通知)/L4(硬拦截) | 默认L2，可调 |
| 👁 vision_model | 市场扩展 | 图像分类/目标检测/OCR/人脸识别 | 可选，0-N |
| 🔗 chain_connector | 市场扩展 | 以太坊/Solana/BNB Chain/Polkadot | 可选，0-N |
| ... | 市场扩展 | 任何人可注册 | 自由组合 |

**组合规则：**

```
【必选】每 Agent 至少1记忆 + 1工具 + 1 LLM，缺失自动补默认值
【冲突】同层不可选2种；本地LLM+云端工具=不可用（需联网）
【约束】记忆层数×活跃Agent ≤ 5；工具数 ≤ 15/会话
【兼容】端口不匹配→自动插入 Transformer 中间件
```

**三种组合方式（§3.1 一句话构建的底层基础设施）：**

| 方式 | 入口 | 适用 |
|------|------|------|
| 一句话描述 | "建一个每天跟踪行业新闻的助手" → COO推断组件 → 确认 | 新手 |
| 引导式构建 | Step 1-5：记忆→工具→LLM→人格→渠道 | 中度用户 |
| 拖拽画布 | 从左侧组件库拖组件到画布，端口连线 | 高阶用户 |

### 3.6 行业基础设施兼容性

Morn 的内置系统天然对齐行业三大协议：

| 协议 | 定位 | Morn 兼容性 |
|------|------|------------|
| **MCP** (Model Context Protocol) | Agent↔工具/数据（事实标准） | 内置工具系统 = 自带 MCP 能力，无需用户配置 |
| **A2A** (Agent-to-Agent) | Agent↔Agent 协调 | 子 Agent 调度 + 事件总线 = 自带 A2A 能力 |
| **SKILL.md** | Agent 技能互操作标准 | 技能格式为 Markdown，**天然兼容**，可吃掉 20K+ 技能 |

### 3.7 Agent 记忆系统与可观测性

Morn 的记忆系统在行业内具有差异化优势：

| 框架 | ⭐ Stars | 基准 (LongMemEval) | 最佳场景 |
|------|---------|-------------------|---------|
| Mem0 | ~55K | 49.0% | 快速集成、通用 |
| Letta | ~15K | 63.8% | 复杂长期任务 |
| Zep | ~10K | — | 企业级 SOC2 合规 |
| **Morn (Hermes memory)** | — | — | **桌面本地 + 四层记忆架构** |

**Agent 可观测性**：Morn 的 Console/Debug 功能 = 内置可观测性，桌面 Agent 调试目前无竞品做好。代表行业平台包括 Langfuse、AgentOps、Braintrust 等。

### 3.8 Agent 失败模式防御

行业现状：**88% 的 Agent 项目无法投产**，71% 组织使用但仅 11% 到生产。

九大失败模式——Morn 应内置防御：
1. Rate limit 错误 → 重试 + 退避策略
2. 上下文窗口溢出 → Token 预算管理
3. 工具调用循环 → 最大重试次数+异常终止
4. 幻觉级联 → Dual-LLM 交叉验证
5. 权限不足 → 清晰的分层授权
6. 数据格式不匹配 → 类型校验 + Transformer 中间件
7. 超时 → 超时 + 后台任务队列
8. 成本失控 → 成本预算 + 自动降级
9. 安全逃逸 → 宪法规则 + 安全检查点

**Morn 的差异化**：可靠性作为核心卖点——竞品都在拼功能，没人强调可靠性。

---

## 四、创作台（Studio）

### 4.1 一句话构建 Agent

创作台不要求用户理解任何概念。最简单的方式就是说一句话。

```
你: "创建一个生物学虚拟研究 Agent"

COO 理解你的需求:
1. 领域识别 → "生物学研究"
2. 角色推断 → "虚拟研究员"
3. 能力推断 → 需要文献检索、数据分析、实验设计、论文撰写
4. 工具推断 → PubMed 搜索、序列分析、统计工具、图表生成
5. 知识推断 → 生物学数据库、实验方法、论文模板
6. 人格推断 → 学术风格、严谨、数据驱动

COO 自动生成完整 Agent 定义:
{
  name: "bio-research-agent",
  
  tools: [
    search_pubmed(return: papers),
    analyze_sequence(return: alignment),
    statistical_analysis(return: results),
    generate_chart(return: figures),
    write_paper(return: draft)
  ],
  
  knowledge: [
    ncbi_databases,
    biology_terminology,
    experimental_methods,
    citation_format
  ],
  
  skills: [
    literature_review(流程: search→filter→summarize),
    experimental_design(流程: hypothesis→design→validate),
    data_analysis(流程: clean→analyze→visualize→interpret)
  ],
  
  persona: {
    name: "生物学研究员",
    style: "academic",
    temperature: 0.3,
    principles: [
      "以实验数据为基础",
      "引文必须可追溯",
      "区分相关与因果",
      "不确定时标注置信度"
    ]
  },
  
  memory: {
    short_term: "当前研究上下文 (100条)",
    long_term: "项目知识库 (向量检索, 语义Top-5)",
    episodic: "实验记录 (每次分析自动保存)"
  },
  
  model: { provider: "deepseek", tier: "balanced" }
}

COO: "生物学虚拟研究 Agent 已生成。
      能力:
      - 搜索 PubMed 文献并自动摘要
      - 基因/蛋白序列对比分析
      - 实验数据统计与可视化
      - 论文草稿生成（含引文格式）
      
      要调整什么吗？[直接保存] [改工具] [改人格] [详细预览]"
```

**生成后的三个选择：**

```
你:
  "直接保存" → 一键入职，Agent 可以在工作台用了
  "帮我加一个 RNA 序列分析工具" → 在 COO 生成的版本上加
  "看看细节" → 打开完整组件编辑器，每个组件可调
```

**底层逻辑：**

```
COO 不是硬编码了"生物学 Agent"的配置。
它是基于 Registry 现有组件+市场可用组件动态推理的：
1. 查 Registry → 是否已有 biology_search, sequence_align 等工具
2. 查市场 → 是否有生物学知识库、生物学人设
3. 如有 → 直接引用；如无 → 创建骨架（端口定义空实现，用户后续补齐）
4. 组合成完整 Agent → 展示给用户

这意味着系统越用越聪明：
  第一次说"生物学 Agent" → COO 推断工具（可能不全）
  用户手动加了 RNA 分析工具 → Registry 记下了
  第二次有人建生物学 Agent → COO 自动加上 RNA 分析

一句话构建也支持团队：
  "创建一个生物学研究团队"
  → COO 自动生成: 文献Agent + 数据分析Agent + 实验设计Agent + 论文写作Agent
  → 定义协作模式: 文献∥实验∥数据 → 分析 → 论文
  → 直接使用

也支持工作流：
  "创建一个生物学文献追踪工作流"
  → COO 自动生成: 每日定时 → PubMed搜索(关键词) → 去重 → AI摘要 → 推送
  → 直接运行
```

**一句话构建 vs 模板 vs 自定义：**

```
一句话
  ↓ (不满意或想改)
选择模板编辑
  ↓ (模板不够灵活)
完全自定义
  ↓ (最终成品)
发布到市场让别人也能用

三个层次，同一入口。用户从哪层开始取决于经验和需求。
新手从一句话开始，高手从自定义开始，中间从模板开始。
```

### 4.2 创作哪些组件

用户在创作台可以独立创建、修改、测试以下组件类型：

| 组件 | 可编辑内容 |
|------|-----------|
| 🧰 工具（Tool） | 名称、端口定义、实现代码（Python/Rust/Shell）、权限声明 |
| 📚 知识（Knowledge） | 数据源选择、处理方式（向量/结构化/全文）、更新策略 |
| 🛠 技能（Skill） | 画布编排多个工具 + LLM 步骤、端口定义 |
| 👤 人格（Persona） | 核心思维模型、温度/风格/主动度、5 层 Prompt、约束规则 |
| 🧠 记忆（Memory） | 短期容量/过期、长期检索/遗忘、工作记忆、经验记忆 |
| ⚙ 模型（Model） | Provider/Model/参数/回退/成本档位 |
| 🤖 Agent | 从以上组件任意组合 |
| 🔗 管道 | 无 Agent 的纯组件链（Timer→get_kline→write_file）|

### 4.3 Agent 人格的深度设计

人格不是几个温度参数的组合。参考 Auto-Company 的 14 个专家人格设计，Morn 的人格组件包含：

```
Persona "analyst" 的定义：

1. 核心思维模型（5-7 条原则）
   - "以数据为基础，不是观点"
   - "先看大盘再看个股"
   - "技术面和基本面相互验证"

2. 决策框架
   - 收到分析请求时：先看趋势 → 算指标 → 查基本面 → 综合判断

3. 反模式
   - 不提供投资建议
   - 不基于单一指标下结论

4. 可量化参数
   temperature: 0.3, 风格: professional,
   详细度: concise, 主动度: 0.6

5. 5 层 Prompt
   L1 核心身份 / L2 技能指令 / L3 格式模板 / L4 约束规则 / L5 对话风格

6. 沟通风格
   "以图表开头，用数据说话"
```

参考论文：
- Persona to Personalization Survey (Chen et al., TMLR 2024) — 三层人格架构
- PERSONA: Composable Persona Vectors (2025) — 人格代数组合
- Character-LLM (Shao et al., EMNLP 2023) — 人格微调

### 4.4 组合画布

从左侧组件库拖组件到画布，从端口画线连接。端口类型不匹配时自动提示，可插入 Transformer 中间件。

```
画布节点类型（参考 Dify 143k⭐ + MetaAgent ICML 2025）：

- LLM（模型选择 + 系统 Prompt）
- Agent（人格 + LLM + 工具 + 知识 + 技能）
- 工具（类型化 IO 端口，MCP 兼容）
- 知识（RAG 管线、文件上传、向量库）
- 技能（方法论包：方法+参考+模板+脚本）
- 代码（沙箱化 Python/JS 执行）
- 路由器（条件分支）
- 循环（迭代 + 收敛检查）
- 触发器（定时 / 事件 / 手动）
```

### 4.5 测试面板

```
测试 Agent 时显示完整执行日志：
[1] 人格注入: 增强 Prompt (0.02s)
[2] 知识检索: stock_db → MACD=金叉 (0.15s)
[3] LLM 调用: deepseek (1.2s, 890 tokens)
[4] 工具调用: get_kline (2.1s)
[5] 工具调用: calc_macd (0.8s)
[6] LLM 调用: deepseek (1.5s, 1200 tokens)
---
总耗时: 5.68s | 总 Token: 2090 | 总成本: ¥0.02

点击每行查看该组件的完整输入输出，可编辑重跑。
```

### 4.6 协作组合模式（7 种）

参考 Multi-Agent Orchestration Survey (2026) 和 MetaGPT (ICLR 2024)：

| 模式 | 流程 | 适用 |
|------|------|------|
| 链式 | A → B → C | 数据处理流水线 |
| 主管-工人 | COO → [Agent A, B, C] | 标准任务分发 |
| 广播-监听 | 事件 → 多 Agent 各自反应 | 监控、情报收集 |
| 投票集成 | 多 Agent 独立评估 → 汇总选优 | 风险评估、翻译 |
| 路由-分类 | 输入 → 分类器 → 不同后续路径 | 客服分流 |
| Agent 即工具 | 一个 Agent 可以当作另一个 Agent 的工具 | 递归组合 |
| 共享黑板 | 多 Agent 读写同一空间 | 协同创作 |

### 4.7 Agent 构建指导

不是所有用户都知道怎么搭一个好 Agent。创作台提供"引导式构建"模式——从模板开始、按步骤配置、预览效果。

**预置 Agent 模板：**

| 模板 | 预置内容 | 用户可改 | 典型场景 |
|------|---------|---------|---------|
| 数据分析师 | tools(get_kline, calc_macd, calc_rsi) + knowledge(stock_db) + persona(analyst) | 增减工具、改人格参数 | 股票分析、数据查询 |
| 研究员 | tools(web_search, news_fetch, summarize) + knowledge(domain_terms) + persona(researcher) | 换数据源、改报告格式 | 行业调研、竞品分析 |
| 写作者 | tools(draft, review, format, send_msg) + persona(writer) | 改风格、加翻译技能 | 报告撰写、内容创作 |
| 程序员 | tools(read_file, write_file, exec_code, git_op) + persona(coder) | 加测试技能、改语言偏好 | 代码开发、脚本编写 |
| 翻译官 | tools(detect_lang, translate, proofread) + knowledge(bilingual_dict) + persona(translator) | 加专业领域词库 | 文档翻译 |
| 系统管家 | tools(launch_app, read_file, browse_web, send_msg) + persona(assistant) | 增减可操控的应用 | 日常电脑辅助 |
| 审查员 | tools(read_file, diff, lint_check, security_scan) + persona(reviewer) + knowledge(coding_standards) | 加自定义规则 | 代码审查、文档校对 |
| 客服 | tools(search_kb, classify_intent, escalate, send_msg) + knowledge(faq_db) + persona(assistant) | 加企业知识库 | 自动回复 |

**自定义 Agent 构建流程：**

```
从空白或模板开始 →
1. 命名 + 选择人格（从市场选或自建）
2. 添加工具（从市场选或自建）
3. 绑定知识库（从市场选或导入文件）
4. 配置记忆（容量/过期/检索方式）
5. 选择模型（成本档位/Provider）
6. 编写/调整 Prompt（5 层）
7. 在测试面板对话测试
8. 调整 → 再测试 → 直到满意
9. 发布到工作台（自己用）或市场（别人买）

每一步都有 COO 提供的实时建议：
"这个 Agent 没有配置知识库，要加一个吗？"
"推荐搭配 calc_macd 工具，很多分析类 Agent 都在用"
"当前人格 temperature 0.6，建议降到 0.3 会更精准"
```

### 4.8 多 Agent 团队构建

单 Agent 做不了的事，交给团队。

#### 核心理念：用户自组织，不是平台预设

Morn 的团队不是预设的固定模板，而是**用户在 Studio 里自由连线**。

与 WorkBuddy 的区别：

| | WorkBuddy 团队模式 | Morn 用户自组织 |
|---|---|---|
| **拓扑由谁定** | 腾讯产品经理 | **用户** |
| **扩展性** | 加模板等腾讯发版 | 用户随时搭新组合 |
| **复杂度上限** | 模板数量 | 无上限（任意图） |
| **和 Studio 的关系** | 独立的「团队」功能 | **Studio 的自然延伸** |

#### 一句话构建团队

用户说「帮我建一个研究团队」，COO 自动：

```
1. create_agent_from_nl("一个专注深度研究的AI分析师")    → Agent A
2. create_agent_from_nl("一个技术写手，擅长把论文变博客")  → Agent B
3. create_agent_from_nl("一个审核员，检查事实准确性")     → Agent C
4. 用户拖线: A.output → B.context, B.output → C.context
5. 用户说"研究量子计算现状" → A研究 → B写 → C审 → 交付
```

#### 团队组成来源

用户可以从 **任意来源** 获取团队零件：

```
来源1：自己从原子组件搭建（Studio → Agent → 连线）
来源2：从市场下载别人做好的 Agent
来源3：从市场下载别人做好的团队蓝图
来源4：混合——自己的 Agent + 别人的 Agent + 别人的组件
```

**关键：团队中的每个 member 本身也是可独立使用的 Agent。**

#### 预置团队模板（起点，不是终点）

预置模板是给用户的起点，不是天花板。用户拿到模板后可以随意增删改连。

| 模板 | 成员 | 协作模式 | 典型场景 |
|------|------|---------|---------|
| 股票研究团队 | data-agent + search-agent + fin-plot + chat-agent | 链式: data∥search→plot+chat | 完整股票分析 |
| 软件开发团队 | pm-agent + architect-agent + coder-agent + qa-agent | 会诊→链式 | 从需求到交付 |
| 内容生产团队 | research-agent + writer-agent + editor-agent + publisher-agent | 链式+审查点 | 周报/文章/报告 |
| 市场调研团队 | search-agent(多源) + analyst-agent + report-agent | 并行+汇总 | 行业/竞品分析 |
| 风控团队 | data-agent + rule-agent + analyst-agent + alert-agent | 投票+广播 | 投资风控 |
| 客服团队 | classifier-agent + handler-agent + knowledge-agent + escalate-agent | 路由→处理 | 自动客服 |
| 监控团队 | timer-agent + check-agent(多个) + alert-agent + report-agent | 广播+汇总 | 7x24 监控 |

**自定义团队构建流程：**

```
从空白或模板开始 →
1. 选择团队成员（已有 Agent + 新 Agent）
2. 选择协作模式（链式/并行/会诊/招标/接力/师徒/黑板）
3. 在画布上连线（端口匹配）
4. 定义通信协议：
   - 事件触发（A 完成 → B 开始）
   - 共享内存（多 Agent 读写同一上下文）
   - 直接消息（Agent 之间可以对话）
5. 设置共识机制：
   - 投票（多数决）
   - CEO 决断（指定一个 Agent 为 leader）
   - Munger 否决（指定一个 Agent 有否决权）
   - 自动汇总（COO 收集全部输出后合成）
6. 测试团队协作（完整任务测试）
7. 调整 → 再测试 → 直到满意
8. 保存为团队模板 → 发布到工作台或市场

每种协作模式的连线方式不同：
┌── 链式 ──────┐    ┌── 并行 ──────┐    ┌── 会诊 ──────┐
│ A→B→C        │    │ A ──┐       │    │ A → 独立意见  │
│ 输出连输入    │    │ B ──┼→ D   │    │ B → 独立意见→COO│
│              │    │ C ──┘       │    │ C → 独立意见  │
└──────────────┘    └──────────────┘    └──────────────┘
```

### 4.9 工作流构建（预定义 + 自定义）

工作流不仅限于 Agent 团队，也可以包含定时触发器、条件分支、人工审批点、外部 API 调用——完整的业务流程。

**预定义工作流模板：**

参考 Auto-Company 6 套标准工作流 + MetaGPT 角色化流程：

| 工作流 | 步骤 | 适用场景 |
|--------|------|---------|
| 任务执行 | 接收指令 → 拆解 → 分配 → 执行 → 汇总 → 交付 | 通用任务 |
| 深度分析 | 收集数据 → 多维度分析(技术/基本面/消息) → 综合判断 → 报告 | 投资/研究 |
| 新闻监控 | 定时触发 → 多源搜索 → 去重 → 摘要 → 推送 | 信息监控 |
| 报告生成 | 数据采集 → 分析 → 模板填充 → 审校 → 格式化 → 分发 | 日报/周报 |
| 代码交付 | 需求→设计→编码→测试→审查→合并→部署 | 软件开发 |
| 产品发布 | QA→DevOps→营销→销售→运营→CEO 确认 | 产品上线 |
| 决策评估 | 调研→CEO→逆向思考→产品→CTO→CFO | 项目立项 |
| 定时巡检 | 每日检查 → 条件判断 → 正常/告警 → 日志 | 系统运维 |

**自定义工作流构建：**

用户在工作流画布上拖拽节点，自由编排。

```
画布上可用的节点类型（比 Agent 组合画布更丰富）：

流程节点:
├── 触发（Timer / 事件 / Webhook / 手动）
├── Agent（调用已部署的 Agent）
├── 工具（直接调用单个工具）
├── 代码（沙箱化运行脚本）
├── LLM（直接 LLM 调用）
├── 知识检索（RAG 查询）

控制节点:
├── 条件（if X then A else B）
├── 循环（for each / while）
├── 等待（pause until condition / timer）
├── 并行（fork-join）
├── 路由（switch-case）
├── 聚合（merge 多路结果）

交互节点:
├── 人工审批（暂停，发通知到桌面/IM，等确认）
├── 人工输入（暂停，向用户提问）
├── 通知（推送消息到指定渠道）

节点之间的连线规则：
- 输出 → 输入（类型匹配）
- 事件 → 触发（松耦合）
- 广播（一个输出到多个输入）
- 条件 → 分支（true/false）

保存为工作流模板 → 在工作台表现为"快速指令"
  "用股票分析工作流查一下茅台"
  → COO 找到对应模板 → 填入参数 → 执行
```

**工作流的组合嵌套：**

```
一个工作流可以调用另一个工作流作为子步骤。
工作流 A（日报生成）:
  └ 步骤 3: 调用 工作流 B（股票分析）作为数据来源

工作流模板支持版本管理，用户可 fork 别人的模板修改。
```

---

## 五、工作台（Workbench）

工作台是用户日常待的地方。用户说"帮我分析茅台"，COO 自己决定怎么干——直接回答、调工具、派 Agent、组团队、套工作流模板，还是甚至跳创作台现做一个。用户看到的就是结果。

### 5.1 交互流程

```
用户输入 → COO 理解 → COO 判断 → 执行 → 反馈 → 确认

Step 1: 输入（所有指令类型在同一输入框）
  "帮我分析茅台"              → 任务
  "打开 Chrome 搜索 Morn"    → 电脑操控
  "每天早上 9 点发日报"       → 定时任务
  "帮我改 data-agent 的 prompt" → 跳创作台
  "看看今天花了多少"          → 跳管理台

Step 2: COO Intent Parser（基于 LLM）
  将自然语言转为结构化意图

Step 3: COO 判断执行路径

  COO 不是"查 Registry 派 Agent"这么简单。
  它动态评估任务复杂度，选择最优的执行层级：

  ┌──────────────────────────────────────────────────────────────┐
  │  COO 的执行层级决策树                                         │
  │                                                               │
  │  用户: "帮我分析茅台"                                         │
  │                                                               │
  │  ① 我能直接回答？                                            │
  │     → "茅台是白酒龙头，600519，今日收盘..."（直接 LLM 知识）  │
  │     仅需 LLM，无需工具/Agent，0 成本                          │
  │                                                               │
  │  ② 不需要 Agent，调一个工具就行？                             │
  │     → get_kline("600519") → 拿到数据 → 回答                  │
  │     单工具调用，不涉及 Agent 生命周期                         │
  │                                                               │
  │  ③ 需要一个现成 Agent？                                      │
  │     → 查 Registry: data-agent 可用（信任92分）               │
  │     → 派 data-agent 执行 → 返回结果                           │
  │     单 Agent，标准 ReAct 循环                                  │
  │                                                               │
  │  ④ 需要临时组一个团队？                                      │
  │     → 任务复杂：需要数据+搜索+分析+报告                      │
  │     → 动态组队：data-agent ∥ search-agent → analyst → report │
  │     → 多 Agent 并行+链式，COO 协调汇总                        │
  │                                                               │
  │  ⑤ 有现成的工作流模板？                                      │
  │     → 查工作流库：有"股票深度分析"模板                       │
  │     → 直接套模板：数据采集 → 技术分析 → 基本面 → 综合报告   │
  │     模板执行，带预设审批点和输出格式                          │
  │                                                               │
  │  ⑥ 以上都不行？                                              │
  │     → COO 说："目前没有分析茅台的能力，是否建一个？"          │
  │     → 跳创作台 → 一句话构建"股票分析 Agent"                   │
  │     → 建好后返回工作台继续                                    │
  │                                                               │
  │  用户不需要知道 COO 选了哪条路径。                            │
  │  COO 自动选最优：最快 + 最省 + 最准的平衡。                   │
  │  用户看到的就是结果。                                          │
  └──────────────────────────────────────────────────────────────┘

  COO 的选择依据：
  ┌──────────────┬──────────────────┬──────────────┐
  │ 层级          │ 触发条件          │ 成本/延迟     │
  ├──────────────┼──────────────────┼──────────────┤
  │ ① 直接回答   │ 简单知识查询      │ ¥0.001/0.5s  │
  │ ② 单工具     │ 单一数据获取      │ ¥0.003/1s    │
  │ ③ 单 Agent   │ 需要专业分析      │ ¥0.02/5s     │
  │ ④ 临时团队   │ 多维度复杂任务    │ ¥0.05/15s    │
  │ ⑤ 工作流模板 │ 有标准流程可套用  │ ¥0.03/10s    │
  │ ⑥ 跳创作台   │ 完全新领域        │ 一次性       │
  └──────────────┴──────────────────┴──────────────┘

  用户也可以强制指定：
  "帮我分析茅台，用数据团队" → 直接跳到 ④，不走决策树
  "帮我分析茅台，直接说"    → 跳过所有工具，只用 LLM 知识

Step 4: 执行
  <3 秒: 直接回答，不需确认
  3-30 秒: 显示计划 → 确认 → 实时进度
  >30 秒: 后台执行 → 完成后通知

Step 5: 反馈（桌面端/IM 多渠道同步）

Step 6: 确认（是否保存/发送/继续）
```

### 5.2 COO 的三种工作模式

```
🏃 主动模式（默认）
  低风险直接执行，高风险展示计划等确认

🛡 安全模式（高风险自动切换）
  每步展示要做什么、影响什么，确认才执行

🤖 自动化模式（定时/触发）
  按计划自动执行，完成后推送报告
```

### 5.3 COO 决策协议

参考 Auto-Company 决策框架 + Greywall Learning Mode：

```
决策三挡：
🟢 运营级（信任>60 + 低风险 + 无歧义）
  自动执行，仅推送通知
🟡 战术级（涉及参数选择/Agent推荐）
  COO 给建议，CEO 确认
🔴 战略级（涉及资金/安全/配置变更）
  必须 CEO 决策

COO 学习机制：
  用户说"以后不用问我"→自动下调阈值
  用户纠正→上调阈值
  规则存 decision_rules 表，可对话修改
```

### 5.4 接管整个电脑

参考 Marvis 1+5 OS 操控 + OSWorld (NeurIPS 2024) + AssistGUI (CVPR 2024) + ACU Survey (2026)：

```
文件系统: 浏览/读/写/移/删(需审批)/压缩/搜索
应用管理: 启动/关闭/列出/安装(谨慎)
系统设置: 壁纸/音量/网络/电源(非关键)
桌面操控: 鼠标/键盘/剪贴板/截图/窗口切换
浏览器: 导航/表单填充/内容提取/多标签

安全授权（每个 Agent 独立配置）：
  L1 沙箱层: 工作区文件 / 注册 API / 不可执行 shell
  L2 本地层: 全盘只读 / 应用控制 / 桌面截图
  L3 系统层: 全盘读写 / shell 执行 / 系统设置（需审批）

感知融合（参考 GUI Agents Survey ACL 2025）：
  像素截图（VLM）+ 无障碍树（结构化）+ OCR（文本锚定）
```

---

## 六、管理台（Console）

### 6.1 导航视图

| 视图 | 功能 |
|------|------|
| 📊 仪表盘 | 系统总览：任务/成本/组件/告警 |
| 🤖 组件拓扑 | 图形化所有组件连接，拖拽断连/重连/替换 |
| 💰 成本中心 | 按组件/Agent/模型拆分的费用，预算管理，每日趋势 |
| ⚙ 治理 | 安全策略例外、API Key、渠道绑定、信任阈值、审批队列 |
| 🔐 安全 | Dual-LLM 拦截记录、宪法状态、审计日志 |
| 🏪 市场 | 上架商品、下载统计、创作者收益 |
| 💻 系统 | CPU/内存/磁盘、Morn 进程管理、日志搜索 |

### 6.2 信任评分

参考 mission-control (5.2k⭐) 的四层评估 + Agent 市场论文 (Liu et al. 2026)：

```
trust_score = output_quality * 0.3 + execution_success * 0.3
            + latency_score * 0.2 + user_feedback * 0.2

四层评估：
  输出评估: 内容质量、格式合规
  追踪评估: 调用链完整性、错误率
  组件评估: 各子组件分别评分
  漂移评估: 性能随时间变化趋势
```

### 6.3 成本中心

```
按 Agent/工具/模型拆分成本
预算设置 + 超限行为（切低成本模型/暂停非必要/通知）
每日/每月趋势图
每笔调用记录可追溯
```

---

## 七、渠道与连接

### 7.1 全渠道列表

| 渠道 | 角色 | 阶段 |
|------|------|------|
| ⭐ Windows 桌面端 (Tauri) | 主战场 | Phase 0 |
| ⭐ 网页端 (PWA) | 无安装入口 | Phase 0 |
| 🔥 企业微信 | 国内 IM 远程 | Phase 1 |
| 🔥 钉钉 | 国内 IM 远程 | Phase 1 |
| 🔥 飞书 | 国内 IM 远程 | Phase 1 |
| ⭐ 微信小程序 | 移动轻量入口 | Phase 1 |
| ⭐ REST API + Webhook | 开发者集成 | Phase 1 |
| 🔶 邮件 (SMTP) | 报告交付 | Phase 1 |
| 🔶 微信公众号 | 推送+公告 | Phase 2 |
| 🔶 QQ 机器人 | 年轻用户 | Phase 2 |
| 🔶 Telegram | 海外用户 | Phase 2 |
| 🔶 浏览器扩展 | 上下文 AI | Phase 2 |

### 7.2 微信生态分层

```
官方安全层（推荐）:
  企业微信机器人（官方 API，完整对话+任务管理）
  微信公众号（服务号消息接口，推送+查询）
  微信小程序（扫码配对+移动入口）

捷径方案（快速实现）:
  PushPlus / Server酱（一个 API 推多平台）

技术向（进阶用户）:
  Gewechat (iPad 协议) / WeChatFerry (Win HOOK)
```

参考：QwenPaw 15k⭐ 15+ 渠道架构、CowAgent 41k⭐ 多渠道验证

---

## 八、Morn Hub（生态接口）

### 8.1 定位：不是市场，是开放生态接口

```
Morn Hub — 不是「市场」(Marketplace)，不是「商店」(Store)
          是 Agent 界的 npm + App Store + GitHub 混合

          可免费发布，也可付费
          核心价值：可发现性 > 交易
          生态繁荣比商业收入更重要
```

**Hub 不是架构中心。** 三台（Workbench/Studio/Console）是中心，Hub 是三台各自连接到外部世界的开放接口。

### 8.2 Hub 与三台的关系

```
Workbench 视角：         Studio 视角：          Console 视角：
┌──────────┐            ┌──────────┐           ┌──────────┐
│ 从 Hub   │            │ 发布到   │           │ 管理 Hub │
│ 安装 Agent│           │ Hub      │           │ 数据    │
│ → 直接用  │            │ → 别人安装│           │ 收益/统计│
│ 搜索 Agent│           │ 定价/免费 │           │ 审核/下架│
└──────────┘            └──────────┘           └──────────┘
```

### 8.3 Hub 可发布的内容

| 层级 | 内容 | 价格模式 |
|------|------|---------|
| 🧩 组件类型定义 | vision_model、chain_connector 等新类型 | 免费 ~ ¥99 |
| 🧱 原子组件 | 工具、知识、技能、人格、记忆、模型、渠道 | 免费 ~ ¥0.01/次 |
| 🎨 设计模式/蓝图 | Agent 设计稿、画布布局、工作流模式、Prompt 模板 | 免费 ~ ¥5 |
| 🤖 Agent | 预制 Agent | 免费 ~ ¥0.05/次 |
| 👥 团队蓝图 | 多 Agent 协作拓扑 | 免费 ~ ¥0.20/次 |

**免费和付费并存，免费是生态基础，付费给创作者选择。** 免费商品在搜索和推荐中与付费商品平权。

### 8.4 Hub 的核心机制

```
发现：搜索 / 分类 / 评分 / 评价 / 下载量 / 信任分
发布：Studio → 一键发布 → 填写描述/定价/截图
安装：Hub → 一键安装 → 注册到本地 Registry → 可用了
支付：Stripe（国际）+ 支付宝（国内）— 按次/月订阅/买断
审核：自动扫描恶意内容 + 人工审核标记 — 防止低质量泛滥
版本：创作者推送更新 → 用户收到通知 → 选择更新/回滚
收益：付费商品 → 平台抽成（10-20%）→ 创作者提现
```

### 8.5 Hub 不是「市场」的意义

| | 传统市场 | Morn Hub |
|--|---------|----------|
| 核心 | 交易 | 发现 + 分享 |
| 免费 | 次要促销手段 | **一等公民，与付费平权** |
| 付费 | 必须 | 可选 |
| 安装 | 购买后 | 免费一键，付费购买后 |
| 发布门槛 | 审核严 | 低门槛，任何人都能发 |
| 内容 | 成品 | 从组件类型到团队蓝图全都有 |

### 8.6 竞品教训

**GPT Store（3M+ GPTs）的教训：**
- 低质量内容泛滥 — "SEO for prompts"、关键词填充、夸大描述
- 创作者无法盈利 — 平台不支付创作者
- 发现机制差 — 搜索和推荐不好用
- → Hub 必须有审核 + 支付 + 好的发现机制

**Paperclip Company Store 的教训：**
- 70K⭐ 但真实用户反馈：「建的网站是坏的，营销数据是幻觉的」
- 「零人类公司」是强大叙事但不可靠
- → Morn 的差异化：做真能用的，不做营销话术

### 8.7 与现有系统关系

```rust
// 从 Storage 层
pub struct HubListing {
    pub id: String,
    pub item_type: String,     // "component_type" | "component" | "blueprint" | "agent" | "team"
    pub name: String,
    pub description: String,
    pub price: Option<f64>,    // None = 免费
    pub price_model: String,   // "free" | "per_use" | "subscription" | "buyout"
    pub author: String,
    pub rating: f64,
    pub downloads: u64,
    pub version: String,
    pub requires: Vec<String>, // 依赖的 Hub 包 ID
    pub verified: bool,        // 审核通过标志
    pub created_at: i64,
    pub updated_at: i64,
}

// 已有的 Gateway trait 扩展
pub trait PaymentGateway {
    fn create_payment(&self, amount: f64, currency: &str) -> Result<Payment, String>;
    fn verify_payment(&self, payment_id: &str) -> Result<bool, String>;
    fn refund(&self, payment_id: &str) -> Result<bool, String>;
    fn payout_to_creator(&self, creator_id: &str, amount: f64) -> Result<String, String>;
}
```

---

## 九、安全模型

### 9.1 四层宪法

```
L1（硬编码）: 格式化磁盘/删系统文件/访问其他进程内存 → 不可绕过
L2（需审批）: 写工作区外目录/未注册域名/shell 命令 → CEO 确认
L3（需通知）: 读工作区文件/调用注册 API/沙箱代码 → 推送通知
L4（自由）: 对话/查数据/搜索 → 不限制
```

参考：Greywall 三层安全边界 + archestra 3.8k⭐ Dual-LLM

### 9.2 Dual-LLM 安全

```
用户输入 → 主 LLM（正常推理）
         → 副 LLM（不同模型，保守配置，检查注入风险）
               → 无风险 → 正常执行
               → 发现风险 → 阻断，标记 HIGH，等用户审批

6 个安全检查点（参考 archestra）：
  认证 → 参数验证 → 内容净化 → 权限检查 → 审计日志 → 路由放行
```

### 9.3 市场安全背景：OpenClaw 安全危机

2026 年 6 月，OpenClaw 连续爆出 4 个连锁漏洞，18 万+ 企业部署受影响：

| 漏洞类型 | 描述 | Morn 防御 |
|---------|------|---------|
| 提示注入 | 执行攻击者控制的代码 | Rust 类型安全+输入清洗 |
| 数据泄露 | 链接预览零点击 | 本地优先+无网络默认 |
| 权限提升 | Agent 获取系统权限 | Tauri 沙箱+权限系统 |
| 技能供应链 | ClawHub 恶意技能 | 技能签名验证 |

**这对 Morn 是差异化武器**——Tauri 沙箱架构 + Rust 内存安全 + 本地优先 vs OpenClaw 的 Python 架构 + 云端技能市场。所有基于 OpenClaw 的产品（含 Marvis、WorkBuddy）共享同样的攻击面。

### 9.4 影子 Agent（Shadow AI）

- 员工私自部署 AI Agent 在个人电脑上，拥有公司凭证
- 平均每次数据泄露成本 **$308,000**
- 安全公司已开始提供「影子 Agent 狩猎」服务
- Morn 走企业路线需有 Agent 注册+审批+审计功能

### 9.5 监管合规

| 框架 | 时间 | 对 Agent 的影响 |
|------|------|-------------|
| **EU AI Act** | 2026.08 生效 | 高风险 Agent 需合规评估 |
| **NIST AI Agent Standards** | 2026.02 启动 | Agent 安全/互操作/可信标准 |
| **WEF Agent Playbook** | 2026.05 | Agent 入职应像新员工 |
| **ISO 42001** | 已生效 | AI 管理系统标准 |

**Morn 优势**：本地优先架构天然符合隐私合规（数据不出本地），可作为合规卖点。

### 9.6 五大安全威胁模型

| 威胁 | 严重度 | 典型案例 | Morn 防御 |
|------|--------|---------|---------|
| 提示注入 | 🔴 极高 | OpenClaw CVE | Rust 类型安全+输入清洗 |
| 数据泄露 | 🔴 极高 | 零点击漏洞 | 本地优先+无网络默认 |
| 权限提升 | 🟡 高 | Agent 获取系统权限 | Tauri 沙箱+权限系统 |
| 技能供应链 | 🟡 高 | ClawHub 恶意技能 | 技能签名验证 |
| 影子 Agent | 🟡 高 | 员工私自在电脑部署 | 有管控的桌面平台 |

---

## 十、开箱即用体验

> 本节为新章节。市场调研验证：所有成功竞品的共同特征是「打开即用」，这是 Morn 当前最大的产品差距。

### 10.1 核心差距

```
Marvis/WorkBuddy 的做法（打开即用）：
下载 → 安装 → 打开 → 说句话 → 活干完了
                        ↑
                    用户不需要知道 Agent 是什么

Morn 之前的做法：
下载 → 安装 → 打开 → 看到空白的 Studio →
"先创建一个 Agent... 选组件... 配技能..." →
用户：？？？
```

Morn 覆盖了 Marvis + WorkBuddy 约 80% 的技术能力，缺的是包装。

### 10.2 差距分析（不是代码差距，是产品包装差距）

```
                    Marvis/WorkBuddy                Morn
                    ────────────────                ────
首次打开体验     → 办公室/聊天框，Agent已就位   → 空窗口，Studio概念
模型来源         → 内置混元，免费Token           → 无默认模型，需配置
Agent 从哪里来   → 出厂预装6个，开箱即用         → 需用户自己创建
用户说什么       → "帮我整理文件"                → 不知道能说什么
AI 在干什么      → 办公室动画，可见               → 黑盒，看不见
怎么扩展         → 技能市场，一键安装             → Studio，需了解Agent概念
远程控制         → 微信/手机APP                   → 有通道但没接
安全模式         → 一键切换隐私模式               → 有dual_llm但没暴露给用户
```

### 10.3 行业共识

行业已验证的 11 个跨领域模式——Morn 的差距和优势所在：

| # | 模式 | 验证来源 | Morn 差距 |
|---|------|---------|-----------|
| 1 | ✅ 零配置开箱即用 | 全部竞品 | 🔴 最大差距 |
| 2 | ✅ 预装多Agent > 单Agent | Marvis 6, Cherry 300, Kimi 300 | 🔴 空壳 |
| 3 | ✅ Cowork 取代 Chatbot | Claude Cowork, Eigent | 🔴 聊天界面 |
| 4 | ✅ 本地优先 + 隐私保护 | Marvis 离线, OpenHuman | ✅ 天然符合 |
| 5 | ✅ 市场/商店生态 | AgentExchange, skills.sh | ✅ 三层市场是差异化 |
| 6 | ✅ Agent 沙箱与安全治理 | MXC, OpenShell | 🟡 需补 |
| 7 | ✅ 协议标准化 (MCP/A2A) | 行业标准 | 🟡 需补 |
| 8 | ✅ 可靠性 > 功能 | Klarna 教训/88%失败率 | 🟢 Morn 机会 |
| 9 | ✅ 混合 AI 人+Agent | Klarna 证明纯AI替代不行 | ✅ 定位正确 |
| 10 | ✅ 云电脑 vs 本地优先 | Coze 3.0 云 vs Morn 本地 | ✅ 战略差异点 |

### 10.4 三种引导状态

#### 状态 A：无 API Key（首次使用）

```
┌─────────────────────────────────────┐
│  👋 欢迎使用 Morn                   │
│                                     │
│  Morn 是一个桌面 AI 创作系统         │
│  你需要配置一个 AI 模型才能开始      │
│                                     │
│  ┌─────────────────────────────┐    │
│  │ 🔑 我有 API Key              │    │
│  │    → 打开设置页              │    │
│  └─────────────────────────────┘    │
│                                     │
│  ┌─────────────────────────────┐    │
│  │ ⚡ 使用内置中转（推荐）       │    │
│  │    → 一键配置，免费额度用完   │    │
│  │      后可自行更换            │    │
│  └─────────────────────────────┘    │
│                                     │
│  或者先逛逛：                        │
│  🏪 Bot Store · 📖 Studio · ⚙️ 设置  │
└─────────────────────────────────────┘
```

#### 状态 B：有 Key，初次聊天

```
┌─────────────────────────────────────┐
│  👋 你好，我是 Morn                 │
│  你的桌面 AI 系统已经就绪 ✅        │
│                                     │
│  试试说：                           │
│  ┌─────────────────────────────┐    │
│  │ 📄 "帮我写一份周报"          │    │
│  │ 💻 "查一下电脑配置"          │    │
│  │ 🔍 "搜索 AI Agent 最新消息"  │    │
│  │ 📊 "分析这组数据"            │    │
│  └─────────────────────────────┘    │
│                                     │
│  或者去 Store 安装预置 Bot          │
│  [输入框...]                        │
└─────────────────────────────────────┘
```

#### 状态 C：调用失败

```
┌─────────────────────────────────────┐
│  ⚠️ API 调用失败                    │
│                                     │
│  当前模型的连接出了问题              │
│  ┌─────────────────────────────┐    │
│  │ 🔑 检查 API Key → 设置      │    │
│  │ 🔄 切换模型 Provider        │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

### 10.5 预装 Agent（出厂自带）

利用 Studio 现有的 Agent 构建能力和注册的工具模块，预装以下 Agent：

| Agent | 能力来源 | 用户说 |
|-------|---------|--------|
| **万能助手** | `core/supervisor` + `core/pipeline` + `component/tool/*` | "帮我查一下..." |
| **文件助手** | `computer/fs_ops` + `component/tool/file_ops` | "整理桌面文件" |
| **系统助手** | `computer/sys_ops` + `computer/desktop_ops` | "电脑还剩多少内存" |
| **写作助手** | `core/pipeline` + `component/skill/builtins` + LLM | "写份周报" |

预装方式：在 Tauri 启动时调用 `Supervisor::create_agent_from_nl()` 或直接加载预设的 `NLAgentDef`，和用户自己在 Studio 造 Agent 是同一套路径。

---

## 十一、插件系统（Plugin System）

### 11.1 设计目标

Morn 的插件系统不等于「Agent 的技能」。Agent 的技能是 Agent 进程内可调用的工具/知识/人格；而 **Morn 插件扩展 Morn 桌面应用本身的能力**——UI 主题、通信渠道、工具注册、前端面板、协议适配——所有用户能感知到的「Morn 的形态和能力」都应该是可插拔的。

三条原则：
- **系统即插件** — Morn 内置功能（默认主题、内置渠道、预置工具）也是插件，只是出厂预装
- **增量叠加** — 插件不能修改核心代码，只能注册到已有插槽
- **用户可控** — 用户可以安装/启用/停用/卸载任意插件，不影响系统稳定性

### 11.2 插件类型

| 类型 | 注册到 | 热加载 | 说明 |
|------|--------|--------|------|
| `theme` | CSS 变量覆盖 | ✅ | 更换整个 UI 配色和视觉效果 |
| `channel` | ChannelRegistry | ✅ | 新增 IM/社交/语音渠道 |
| `tool` | ToolRegistry | ❌（需重启） | 新增 Agent 可用工具 |
| `knowledge` | 知识源注册表 | ✅ | 新增外部知识源（本地文件/Confluence/GitHub Wiki） |
| `ui-panel` | 前端插槽系统 | ✅ | 在 Studio/Console 中新增界面面板 |
| `protocol` | 协议注册表 | ❌ | 新增通信协议（A2A/MCP/BLE/HTTP） |

### 11.3 插件规范

**目录布局**（安装到 `~/.hermes/plugins/<id>/`）：

```
~/.hermes/plugins/morn-theme-cyber/
├── plugin.json          # 清单文件（必选）
├── themes/
│   └── variables.css    # CSS 变量覆盖
├── assets/
│   └── preview.png      # 预览图（可选）
└── README.md            # 说明文档（可选）
```

**plugin.json 格式**：

```json
{
  "id": "morn-theme-cyber",
  "name": "赛博朋克主题",
  "version": "1.0.0",
  "type": "theme",
  "entry": "themes/variables.css",
  "hooks": ["ui:css-variables"],
  "description": "暗色霓虹风格，青色发光 + 扫描线效果",
  "author": "Morn Team",
  "license": "MIT",
  "requires": "morn >= 0.2.0",
  "permissions": ["ui:override-styles"],
  "settings": {
    "accent": "#00f0ff",
    "dark": true
  }
}
```

### 11.4 Plugin trait（Rust 后端）

```rust
/// 后端插件 trait：渠道/工具/协议类型的插件实现此 trait
#[async_trait]
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn version(&self) -> &str;

    /// 插件初始化（扫描后调用）
    async fn load(&self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// 插件启用（注册到目标 Registry）
    async fn activate(&self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// 插件停用（从 Registry 移除）
    async fn deactivate(&self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// 插件卸载（清理资源）
    async fn unload(&self) -> Result<(), PluginError>;

    /// 声明支持的 hook 点
    fn hooks(&self) -> Vec<&str>;
}

/// 前端插件 trait：仅供 JS/TS 端使用
pub trait UIPlugin {
    fn id(&self) -> &str;
    fn plugin_type(&self) -> UIPluginType;  // Theme | Panel
    fn entry(&self) -> &str;  // CSS 或 JS 文件路径
}
```

**类型枚举**：

```rust
pub enum PluginType {
    Theme,
    Channel,
    Tool,
    Knowledge,
    UiPanel,
    Protocol,
}
```

### 11.5 PluginManager

```rust
pub struct PluginManager {
    /// 已加载的所有插件
    loaded: HashMap<String, Box<dyn Plugin>>,
    /// 已激活的插件 ID 集合
    active: HashSet<String>,
    /// 扫描目录（~/.hermes/plugins/）
    root: PathBuf,
    /// 类型级联索引
    by_type: HashMap<PluginType, Vec<String>>,
}
```

**生命周期方法**：

| 方法 | 行为 |
|------|------|
| `scan()` | 遍历 `~/.hermes/plugins/` 读取各目录的 `plugin.json` |
| `load(id)` | 创建 Plugin 实例，调用 `load()` |
| `activate(id)` | 注册到对应 Registry（对于 theme：前端注入 CSS） |
| `deactivate(id)` | 从对应 Registry 移除（对于 theme：移除 CSS link） |
| `uninstall(id)` | 停用 + 卸载 + 删除目录 |
| `list()` | 列出全部可用插件及其状态 |
| `get_for_type(t)` | 获取指定类型的所有已激活插件 |

### 11.6 主题插件（最快闭环）

主题是插件系统中最简单的类型——**一个 CSS 文件就够了**。

**工作原理**：
1. 安装时 PluginManager 读取 `themes/variables.css`
2. 激活时在 `<head>` 中动态插入 `<link rel="stylesheet">`
3. CSS 文件覆盖 `:root` 的 CSS 自定义属性（所有颜色/字号/间距变量）
4. 停用时移除该 `<link>` 标签，恢复默认

**当前内置主题**：

| 主题 ID | 名称 | 风格 | 状态 |
|---------|------|------|------|
| `morn-theme-default` | Morn 默认 | 暗色极简 | 内置（不可卸载） |
| `morn-theme-cyber` | 赛博朋克 | 霓虹青 + 扫描线 | 内置（可选） |
| `morn-theme-glass` | 玻璃拟态 | 毛玻璃 + 轻盈阴影 | 规划 |

### 11.7 前端插槽系统

对于 `ui-panel` 类型的插件，前端需要定义可扩展的插槽位置：

```typescript
// 插槽定义（在 Router 或布局组件中声明的挂载点）
export const SLOTS = {
  'sidebar:top':       '侧边栏顶部',
  'sidebar:bottom':    '侧边栏底部',
  'console:tab':       '管理台新增 Tab 页',
  'studio:panel':      '创作台新增面板',
  'chat:toolbar':      '聊天栏扩展按钮',
  'settings:section':  '设置页新增分组',
} as const;

type SlotId = keyof typeof SLOTS;

interface SlotRegistration {
  id: string;
  slot: SlotId;
  component: React.ComponentType;
  label: string;
  icon?: string;
  order?: number;
}
```

### 11.8 与现有系统关系

```
┌─────────────────────────────────────────────────────────┐
│                    PluginManager                         │
│  ┌─────────┐ ┌──────────┐ ┌───────┐ ┌──────┐ ┌───────┐ │
│  │ theme   │ │ channel  │ │ tool  │ │panel │ │proto  │ │
│  └────┬────┘ └────┬─────┘ └───┬───┘ └──┬───┘ └───┬───┘ │
│       │           │           │        │          │      │
└───────┼───────────┼───────────┼────────┼──────────┼──────┘
        │           │           │        │          │
        ▼           ▼           ▼        ▼          ▼
   <head> CSS  ChannelRegistry  ToolRegistry  React   Protocol
   <link>      │              │         Slot     Registry
               │              │         Render
               ▼              ▼
         Telegram/Discord   CodeToolExecutor
```

- 渠道插件：现有 `ChannelRegistry` + `MessageAdapter`，只需新增 `impl Channel for DiscordChannel` 并注册
- 工具插件：现有 `ToolRegistry` + `Tool trait`，插件提供 `Box<dyn Tool>` 实例
- 主题插件：无需 backend trait，前端直接 `<link>` 加载 CSS 文件

### 11.9 安全模型

| 维度 | 策略 |
|------|------|
| 权限声明 | plugin.json 中通过 `permissions` 字段声明所需能力，用户安装时确认 |
| 隔离 | 每个插件拥有独立目录，后端插件在有限上下文中运行 |
| 沙箱 | 未来第三方 UI 插件在 `<iframe>` 或 WebWorker 中渲染 |
| 签名 | 市场下载的插件经过签名验证（整合 Marketplace） |
| 限速 | 渠道/工具有速率限制，防止插件耗尽资源 |

### 11.10 实现路线

| 阶段 | 内容 | 工时 |
|------|------|------|
| P1 | 主题插件化：赛博主题抽成独立 CSS + `data-theme` 切换 + localStorage 持久化 | 1h |
| P2 | PluginManager 后端：scan + load + activate + list | 2-3h |
| P3 | 前端插槽系统 + 插件 Store（状态管理） | 2h |
| P4 | 渠道/tool/知识源插件注册 | 各 1-2h |
| P5 | 插件市场集成（Marketplace + 签名验证） | 2h |

### 11.11 与竞品的差异

| 维度 | uTools (4000+ 插件) | LobeHub | Morn |
|------|--------------------|---------|------|
| 插件类型 | 搜索启动器扩展 | 插件商店（Agent技能） | **Morn 本体扩展（主题/渠道/UI面板/协议）** |
| 后端扩展 | 不适用 | 不适用 | Rust Plugin trait + 生命周期管理 |
| 前端扩展 | 不适用 | 插件只影响 Agent | 前端插槽系统，UI 组件可插拔 |
| 插件市场 | 自建 | 自建 | 复用 Marketplace |

**Morn 的差异化**：插件不只为 Agent 加技能，而是**扩展 Morn 桌面本身**——换皮肤、加渠道、挂新面板、接新协议。

---

## 十二、执行路线图

> 基于 23 缺口调研（§十三）和代码质量审计（§十四）。

### 12.1 完成状态（2026.06.17）

Morn 1.0 上线冲刺已完成。核心链路打通、实时可视化、Hub发布、CI/CD自动构建全部就绪。DESIGN.md 附录 A 同步更新至 88 命令（原 29）。

| 阶段 | 覆盖缺口 | 状态 |
|------|---------|------|
| Phase 0：修核心链路 | #1 #2 #19 | ✅ 完成 |
| Phase 1：开箱即用 | #3 #11 | ✅ 完成 |
| Phase 2：一人公司 | #4 #12 | ⏳ 1.0后续 |
| Phase 3：Hub生态 | #7 #13 #14 | ⏳ 1.0后续（基础功能已通） |
| Phase 4：平台进化 | #6 #9 #20 | ⏳ 1.0后续 |
| Phase 5：跨设备/移动端 | #5 #10 | ⏳ 1.0后续 |
| Phase 6：全球/用户路径 | #15 #16 #17 #18 #21 | ⏳ 1.0后续 |

### 12.2 1.0 上线冲刺完成项（2026.06.12-06.16）

#### B1 — 实时DAG执行可视化
- `ExecutionFlow.tsx` — 聊天区垂直时间线显示真实执行日志
- `ExecutionHistory.tsx` — Console面板历史执行记录
- `chat.rs` send_message 返回含 `execution_events` 的 `SendMessageResult`
- 前端每5秒轮询 `get_recent_logs` 实时更新
- 验证：`cargo build --lib` + `npm run build` + 1465 tests ✅

#### B2 — Hub市场字段补齐
- `Listing` 结构体新增 `version`, `screenshots`, `category` 字段
- `hub_publish` 命令接受完整发布参数
- `StudioPublisher.tsx` 表单：版本号/分类选择器/截图URL
- `BotStore.tsx` 一键"Publish to Hub"按钮
- SQL 表新增3列，schema自动迁移
- 验证：1465 tests 全部通过 ✅

#### B3 — GitHub Actions 自动构建发布
- `.github/workflows/release.yml` — 触发后 Windows runner 构建 NSIS + MSI
- 自动上传到 GitHub Releases，生成 `update.json` 支持 Tauri 自动更新
- README 添加 GitHub Release badge
- `tauri.conf.json` 已配置 updater 公钥和端点
- 用法：`git tag v0.1.0 && git push origin v0.1.0` → 自动出安装包

#### B4 — 核心代码清理
- 生产代码 `println!` 已全部替换为 `tracing::info!`（1处 → 0处）
- 聊天/执行路径已零 `unwrap()`（全部使用 `map_err` + `?`）
- 仅 test 代码保留 `unwrap()`（安全）

### 12.3 验证总纲

| 维度 | 验证 | 状态 |
|------|------|------|
| 编译 | `cargo build --lib` | ✅ 零错误 |
| 测试 | `cargo test --lib` | ✅ 1466 passed |
| Clippy | `cargo clippy --lib --no-deps` | ✅ 0 warnings |
| 前端 | `npm run build` | ✅ 通过 |
| 生产 unwrap | `grep \.unwrap() src/` (排除 test) | ✅ 0 个 |
| todo!/unimplemented! | 生产代码 | ✅ 0 个 |
| 死代码 warning | 编译器 dead_code | ✅ 0 个 |
| 安装 | 打tag → Actions自动构建 .exe | ✅ CI/CD 就绪 |
| 下载 | GitHub Releases → 安装包 | ✅ 流程打通 |

### 12.4 代码健康 Batch（2026.06.16）

审计范围：305 Rust 文件 / 58,244 行 + 71 TS 文件 / 11,638 行。

#### Batch A — 修复 TS 错误 + 清理死代码
- VoiceInput.tsx TS6133 未用变量删除
- `presets_tech.rs`（404 行）删除 — 已被 JSON 数据化替代（52 个 JSON 文件）
- `sandbox/wasm/tool.rs`（18 行）删除 — 孤立包装器，从未编译

#### Batch B — 小文件合并
- `visibility.rs`（21 行）内联到 `registry/mod.rs`，删除独立文件

#### Batch C — Storage CRUD 重复代码消除
- `Storage` 结构体新增 `conn()` 辅助方法，封装 `self.conn.lock().map_err(...)?`
- 替换 27 处重复锁获取模式为 `self.conn()?`
- 涉及文件：mod.rs、agents.rs、settings.rs、sessions.rs、oauth.rs、sync.rs

#### 审计结果
| 指标 | 值 |
|------|-----|
| Rust 文件 | 301（−4） |
| Rust 行数 | 57,799（−445） |
| 测试通过 | 1465（不变） |
| 生产 unwrap | 0 |
| clippy warnings | 0 |
| todo!() | 0 |

#### Batch D — 工作流模板 + 市场数据数据化（2026.06.17）
- `business.rs`（620→92 行）— 5 个业务模板 inline struct 提取为 `workflow_templates.json`，`include_str!` 加载
- `marketplace.rs`（~100 行 inline data）— 7 个内置 Listing 提取为 `builtin_listings.json`，`include_str!` 加载
- 测试同步精简：删除 13 个纯数据验证测试，保留 6 个核心逻辑测试
- 净减 ~570 行代码，编译 0 警告，1466 测试通过

#### 审计结果
| 指标 | 值 |
|------|-----|
| Rust 文件 | 302（−3，新加 2 个 JSON） |
| Rust 行数 | 57,404（−840） |
| TS/TSX 文件 | 65 |
| TS/TSX 行数 | 9,381 |
| 测试通过 | 1466（+1） |
| 生产 unwrap（核心路径） | 0 |
| clippy warnings | 0 |
| todo!() | 0 |

---



## 十三、缺口分析

### 13.1 调研方法

10 个角度 × 三轮地毯式调研（2026 年 6 月）：

| 角度 | 调研对象 |
|------|---------|
| 竞品对标 | Paperclip(70K⭐)、OpenHuman(27K⭐)、OpenFang(16K⭐)、Goose(45K⭐) |
| 商业产品 | Marvis、WorkBuddy、Lindy($49/mo)、Cherry Studio |
| 一人公司栈 | OPC Community(50+工具)、Solo Founder Tech Stack(30工具) |
| 安全事件 | CVE-2026-2256(CVSS9.8)、MCP Tool Poisoning、OWASP Q1 2026 |
| 市场教训 | GPT Store (3M+ GPTs 低质量泛滥)、Paperclip 用户反馈 |
| 合规要求 | EU AI Act 2026.08 生效、SOC2/GDPR/ISO27001 |
| 用户痛点 | Reddit r/AI_Agents（可靠性/上下文/成本/幻觉） |
| 开源策略 | Paperclip 30K⭐/3周 = 叙事驱动，非代码驱动 |
| 代码质量 | Morn 代码审计（57,404 行 Rust + 9,381 行 TSX） |
| 用户期望 | 速度/准确/简单/集成/离线/语音 |

### 13.2 23 个缺口全景 — 修复状态（2026.06.17）

#### ✅ 已修复（6 个 — 1.0 冲刺完成）

| # | 缺口 | 修复内容 |
|---|------|---------|
| 1 | **send_message 绕过 COO** | ✅ chat.rs 走 Supervisor 管线，fallback 保留 |
| 2 | **无默认模型** | ✅ 内置中转（sensenova）+ 欢迎页配置引导 |
| 3 | **无预装 Agent** | ✅ seed_hub_data 启动时填充 |
| 11 | **无导出/备份** | ✅ backup.rs 命令已注册 |
| 19 | **无"能做什么"引导** | ✅ WelcomeGuide 3种状态 + QuickActions |
| 23 | **GitHub Actions CI/CD** | ✅ release.yml 自动构建 Windows 安装包 |

#### 🔴 待修复（4 个 — 1.0 阻塞级）

| # | 缺口 | 优先级 |
|---|------|--------|
| 4 | **一人公司无真实业务** | 1.0后 |
| 5 | **无移动端/远程入口** | 1.0后 |
| 7 | **Hub 无支付/无审核** | 1.0后（免费发布可用） |
| 8 | **无 OAuth 集成** | 1.0后 |

#### 🟡 重要（10 个 — 1.0后修复）

| # | 缺口 | 参照 | 影响 |
|---|------|------|------|
| 9 | 无 WASM 沙箱 | OpenFang 16层 | 安全 |
| 10 | 无跨设备同步 | Marvis | 可用性 |
| 11 | 无导出/备份 | 基础功能 | 信任 |
| 12 | 无主动 Agent | Lindy、OpenFang 7 Hands | 效率 |
| 13 | 无用户可见记忆 | OpenHuman Obsidian wiki | 信任 |
| 14 | 无声音输入 | OpenHuman Whisper | 体验 |
| 15 | 无 Day 2-30 用户路径 | 行业 80%+ 次日流失 | 留存 |
| 16 | 无英文/全球 | 竞品全部有 | 市场 |
| 17 | 无成本透明/预算 | 用户月均 $300-750 | 成本 |
| 18 | 无可靠性指标 | #1 用户投诉 | 信任 |
| 19 | 无"能做什么"引导 | 上线最佳实践 | 留存 |
| 20 | 无本地模型优先 | 省 $300/月 | 成本 |

#### 🔵 加分（3 个 — 做了 = 超越竞品）

| # | 缺口 | 价值 |
|---|------|------|
| 21 | 无用户可见错误恢复 | Agent 失败时优雅降级 |
| 22 | 无使用分析面板 | 看任务量/成功率/成本趋势 |
| 23 | 无商业模式/定价 | 开源不可持续 |

### 13.3 优先级矩阵

```
                     用户价值
                     低 ←──────────→ 高
                 ┌─────────────────────────────┐
           高    │  #6 零门槛插件               │ #1 COO接线     │
                 │  #14 语音输入                │ #2 默认模型    │
    差异化       │  #9 WASM沙箱                │ #3 预装Agent   │
                 │  #13 可见记忆                │ #5 移动端      │
                 │  #20 本地模型优先            │ #8 OAuth集成   │
                 │                             │ #4 业务模板    │
                 ├─────────────────────────────┼─────────────────┤
           低    │  #22 分析面板                │ #7 Hub支付     │
                 │  #21 错误恢复                │ #10 同步       │
                 │  #23 商业模式               │ #11 备份       │
                 │                             │ #12 主动Agent  │
                 │                             │ #15 用户路径   │
                 │                             │ #16 英文       │
                 │                             │ #17 成本透明   │
                 │                             │ #18 可靠指标   │
                 │                             │ #19 能做什么   │
                 └─────────────────────────────┴─────────────────┘
```

---

## 十四、代码质量要求

### 14.1 安全底线

```
🔴 禁止：shell 命令字符串拼接（已发现漏洞：app_ops/launch.rs PowerShell 注入）
🔴 禁止：生产代码 unwrap（已发现 862 个）
🔴 禁止：println!/eprintln! 代替 tracing（已发现 94 个）
🔴 禁止：Mock 替代真实服务上架
```

### 14.2 当前代码质量审计

| 指标 | 当前值 | 目标值 | 状态 |
|------|--------|--------|------|
| 生产代码 unwrap | **0** | 0 | ✅ 已达标 |
| println!/eprintln! | **0** | 0 | ✅ 已达标 |
| clippy warnings | **0** | 0 | ✅ 已达标 |
| todo!/unimplemented! | **0** | 0 | ✅ 已达标 |
| 死代码 (编译器 dead_code) | **0** | 0 | ✅ 已达标 |
| tracing 调用 | 100+（448行/次） | 300+ (150行/次) | ⏳ 持续推进 |
| 前端测试 | 0 | 80%+ 核心组件 | ⏳ |
| 集成测试 | 0 | 20+ 端到端场景 | ⏳ |
| i18n | 无 | 全部 UI 字符串外部化 | ⏳ |
| 发布包 | **已配置** | 每个 tag 自动 build | ✅ CI/CD 就绪 |
| 安全扫描 | 无 CI | Cargo audit / npm audit 必过 | ⏳ |
| deployment | **release.yml** | 一键安装包 | ✅ GitHub Releases |
| 重复代码 | `conn()` 方法消除 27 处重复 | 持续减少 | ✅ Batch C |

### 14.3 错误处理规范

```
✅ 使用自定义 MornError 枚举，取代 Result<T, String>
✅ 用户可见的错误：显示友好消息 + 重试按钮
✅ 系统日志的错误：tracing::error! 记录完整上下文
✅ 可恢复的错误：自动重试（指数退避）
✅ 不可恢复的错误：优雅降级 + 用户通知
❌ 不要：panic!、unwrap、expect、"Error: {}"
```

### 14.4 日志规范

```
✅ tracing::info! — 正常流程关键节点
✅ tracing::warn! — 非预期但可恢复的状况
✅ tracing::error! — 失败需要关注
✅ tracing::debug! — 调试信息（编译时过滤）
❌ 不要：println!、eprintln!
```

### 14.5 Feature Gate 规范

当前大量功能被 feature gate 切掉（channels-full / desktop-real / providers-full），导致默认 build 功能不全。

```
✅ 默认所有功能开启（default features = full）
✅ 性能敏感功能做懒加载而非 feature gate
❌ 不要在编译时切掉核心功能
```

## 十五、竞品全景

### 15.1 桌面 AI 工作台

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| **WorkBuddy 企业版** | 腾讯云 | 产品 | 7×24数字员工、Agent Suite、团队模式、管理后台、企业RAG | WorkBuddy = 企业SaaS。Morn = 桌面AI OS。互补非竞争。WorkBuddy验证了「团队Agent协作」是刚需，但它的团队是固定模板，Morn应该走用户自组织 |
| Marvis | 腾讯 | 产品 | 1+5 OS层操控、隐私闸门、跨平台 | OS 操控设计参考，6 Agent 预装模式需跟随 |
| Claude Cowork | Anthropic | 产品 | AI同事桌面，多步骤知识工作，非技术可用 | 从 Chatbot 到 Cowork 范式转移 |
| Kimi Work | 月之暗面 | 产品 | 300 Agent并行、13h连续、浏览器+文件控制 | 大规模并行验证 |
| OpenAI Super App | OpenAI | 规划 | ChatGPT+Codex+Atlas 三合一 | 业界最大玩家验证了 Morn 哲学 |
| Cherry Studio | 社区 | 47K | 300+预配置AI助手 | 开源生态模式值得借鉴 |
| Coze 3.0 | 字节跳动 | 产品 | 云电脑运行+零代码创建 | 云 vs 本地是战略差异点 |
| **OpenFang** | RightNow-AI | 16K | Rust+Tauri, 137K LOC, 2696 tests, 16层安全, 40渠道 | WASM沙箱、Merkle审计链设计参考 |
| **Harbor** | av/harbor | — | Tauri v2+CLI, MCP服务编排, 服务蓝图, 一键部署 | MCP编排、服务蓝图设计参考 |

### 15.2 虚拟公司/虚拟团队（Morn 蓝图的验证者）

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| **Paperclip** | 社区 | **70K** | 组织架构图 + 治理 + Company 商店 | 完全验证了 Morn「团队蓝图+市场」方向 |
| ChatDev | OpenBMB | 33K | 虚拟软件公司，NeurIPS 2025 | 角色化Agent+公司模式可行 |
| MetaGPT | 社区 | 40K+ | SOP角色化Agent，ICLR 2024 Oral | 角色工具包设计 |
| SynthOrg | 社区 | 新项目 | 合成组织框架，最接近Morn组装理念 | 自下而上组装设计参考 |

### 15.3 Agent 编排平台

| 产品 | ⭐ | 关键特征 |
|------|-----|---------|
| Dify | 143K | 可视化画布、Chatflow+Workflow、RAG管线 |
| LangGraph | 28K | 有状态图、Checkpoint、Durable Memory |
| CrewAI | 46K | 角色Agent团队、事件驱动Flow |

### 15.4 桌面端 AI 应用（Tauri/原生）

| 产品 | ⭐ | 关键特征 |
|------|-----|---------|
| OpenHuman | 27K | Tauri+Rust、记忆树、TokenJuice压缩、118+ OAuth、Obsidian wiki、语音 |
| Kuse Cowork | 5K | 纯Rust+Tauri、10MB、Docker隔离 |
| Goose | 45K | Rust核心、MCP扩展、桌面+CLI+API（Linux Foundation） |
| Jan | 25K | 100%离线、插件式本地模型 |
| OpenPawz | 新项目 | Tauri v2、离线优先、混合记忆、n8n集成 |

### 15.5 AI 员工平台

| 产品 | 融资 | 关键特征 |
|------|------|---------|
| Lindy | — | G2 4.9/5，AI员工平台，$49/月，主动邮箱/日历管理 |
| Sintra | $17M | 12+预配置AI员工团队 |
| ServiceNow Autonomous Workforce | — | 90%+ IT门票自主解决，比人工快99% |

### 15.6 AI 联合创始人（2026 新品类）

| 平台 | 定位 | 融资 |
|------|------|------|
| **Cofounder 2** | 一人公司的操作系统 | — |
| **Viktor** | Slack/Teams里的AI同事 | **$75M (Accel)** |
| **Agentfounder** | 自主AI联合创始人 | $499/月 |
| **doola** | Business-in-a-Box™ | YC |

### 15.7 中国市场入局者

| 产品 | 公司 | 关键特征 |
|------|------|---------|
| DuClaw | 百度 | 零部署 OpenClaw 云服务 |
| QClaw | 腾讯 | OpenClaw 整合微信生态 |
| Wukong | 阿里巴巴 | 企业级多 Agent 编排 |
| Doubao 2.0 | 字节跳动 | 中国最易用 AI 助手，多Agent架构 |
| Accio Work | 阿里国际 | 本地优先、读文件/跑终端/控浏览器 |

### 15.8 Morn vs 竞品全景矩阵

| 维度 | 云端平台 | SaaS AI员工 | 虚拟公司 | **Morn** |
|------|---------|-----------|---------|---------|
| **形态** | Cofounder 2, Coze 3.0 | Lindy, Sintra | Paperclip | **桌面 App** |
| **本地数据** | ❌ | ❌ | ❌ | **✅ Tauri 本地** |
| **开源** | ❌ | ❌ | ✅ | **✅ 开源** |
| **代码能力** | ✅ | ❌ | ❌ | **✅** |
| **文件管理** | ❌ | ❌ | ❌ | **✅** |
| **多Agent** | ✅ | ❌ | ✅ | **✅ 组件系统** |
| **市场** | ❌ | ❌ | ✅ Company商店 | **✅ 三层市场** |
| **开箱即用** | ❌ 需注册 | ✅ | ✅ | **❌ 当前空壳（P0修复）** |
| **安全性** | 🟡 | 🟡 | 🟡 | **✅ Rust+Tauri** |

### 15.9 Morn vs WorkBuddy 全面对比

WorkBuddy 于 2026.6.5 发布企业版，是 Morn 最直接的市场参照物。对比不是为竞争，是为找到 Morn 的差异化路径。

| 维度 | WorkBuddy（腾讯云） | Morn |
|---|---|---|
| **定位** | 企业级 AI 办公平台（SaaS） | 桌面 AI 操作系统（OS） |
| **架构** | 云原生（腾讯云托管） | 桌面原生（Tauri + Rust） |
| **部署** | 公有云 / VPC / 私有化 | 本地安装 .exe |
| **隐私** | 数据经腾讯云 | **完全离线可用** |
| **组件类型** | 平台预设（固定） | **可扩展注册，任何人可加新类型** |
| **Agent 组合** | Agent Suite 预设组件 | Studio 自由组装 |
| **团队模式** | 固定模板（产品经理定拓扑） | **用户自组织连线** |
| **市场** | 企业应用市场（仅成品） | **贯穿所有层，人人可创作可发布可售卖** |
| **管理后台** | ✅ 完整管理后台 | 有 org 模块，缺 UI |
| **企业知识库** | ✅ 企业级 RAG | 缺向量检索 |
| **办公集成** | 腾讯文档/网盘/乐享 | 缺 |
| **一句话构建** | ❌ | ✅ create_agent_from_nl() |

**结论：Morn 不应该对标 WorkBuddy 做企业 SaaS，而应该加倍强调「本地优先 + 用户自组装」——这是 WorkBuddy 做不到的差异化。**

### 15.10 竞品威胁评估

| 竞品 | 威胁 | Morn 应对 |
|------|------|----------|
| Paperclip 70K⭐ | 🟡 叙事强大，但技术是调度面板 | 靠真有用的产品而非话术 |
| OpenHuman 27K⭐ | 🟡 快速崛起，OAuth + 记忆是杀手 | 补 OAuth + 可见记忆，优先级最高 |
| OpenFang 16K⭐ | 🔴 最直接技术对标 | WASM 沙箱 + 40 渠道 + 自主 Agent 是硬缺口 |
| Marvis (腾讯) | 🟡 6 预装 + 跨设备 + OS 层 | 补预装 + Telegram 远程 + 桌面操控真实化 |
| WorkBuddy (腾讯) | 🟡 企业版，20+技能包 | 补一人公司业务模板 |
| Lindy $49/月 | 🟡 主动管理日历/邮件 | 补主动 Agent + OAuth 集成 |

---

## 十六、参考来源

### 16.1 学术论文（18 篇）

| # | 论文 | 会议/期刊 | 年份 |
|---|------|----------|------|
| 1 | ReAct: Synergizing Reasoning and Acting in Language Models | ICLR | 2023 |
| 2 | Toolformer: Language Models Can Teach Themselves to Use Tools | Meta AI | 2023 |
| 3 | Reflexion: Language Agents with Verbal Reinforcement Learning | NeurIPS | 2023 |
| 4 | MetaGPT: Meta Programming for Multi-Agent Collaborative Framework | ICLR (Oral) | 2024 |
| 5 | Chain of Agents: LLMs Collaborating on Long-Context Tasks | NeurIPS | 2024 |
| 6 | MetaAgent: Constructing MAS Based on Finite State Machines | ICML | 2025 |
| 7 | LLM-based Multi-Agent Orchestration: A Survey | Preprints | 2026 |
| 8 | From Persona to Personalization: Survey on Role-Playing Agents | TMLR | 2024 |
| 9 | Character-LLM: A Trainable Agent for Role-Playing | EMNLP | 2023 |
| 10 | PERSONA: Dynamic Compositional Inference-Time Personality Control | — | 2025 |
| 11 | OSWorld: Benchmarking Multimodal Agents in Real Computer Environments | NeurIPS | 2024 |
| 12 | ScreenAgent: VLM-driven Computer Control Agent | IJCAI | 2024 |
| 13 | GUI Agents: A Survey | ACL Findings | 2025 |
| 14 | AssistGUI: Task-Oriented Desktop GUI Automation | CVPR | 2024 |
| 15 | AI Agents for Computer Use: A Comprehensive Survey | — | 2026 |
| 16 | When Agent Markets Arrive | — | 2026 |
| 17 | Rise and Potential of LLM Based Agents: A Survey | Sci. China | 2015 |
| 18 | Multi-Agent Collaboration Mechanisms: A Survey of LLMs | — | 2025 |

### 16.2 GitHub 项目

Auto-Company, OpenHuman(27K), Kuse Cowork(5K), OpenPawz, Jan(25K), Goose(45K), OpenFang(16K), Paperclip(70K), ChatDev(33K), MetaGPT(40K+), Cherry Studio(47K), Dify(143K), LangGraph(28K), CrewAI(46K), OpenClaw(376K), Hermes Agent

### 16.3 市场/行业报告

| 来源 | 内容 |
|------|------|
| Medvi $20K→$1.8B | NYT, Forbes, Inc. 2026.04 |
| One-Person Companies 29.8M | LinkedIn 2026 |
| 一人技术栈 $300-500/月 | OPC Community 2026 |
| Agent 市场 $10.9B→$50.3B | Grand View Research 2026 |
| Top 25 Agent $25B+ | AgentMarketCap 2026 |
| Viktor $75M Series A | Fortune 2026.05 |
| OpenAI Super App | Quasa 2026 |
| Microsoft Project Solara | The Verge 2026.06 |
| Microsoft MXC | Windows Blog 2026.06 |
| OpenClaw 安全危机 | Hacker News, Cyera 2026.06 |
| NIST AI Agent Standards | NIST 2026.02 |
| EU AI Act | EU 2026.08生效 |
| SKILL.md 标准 (20+工具) | TermDock, agensi.io 2026 |
| 88% Agent失败率 | DigitalApplied 2026 |
| CVE-2026-2256 (MS-Agent CVSS 9.8) | NVD / CISA 2026.03 |
| MCP Tool Poisoning (20万+脆弱) | iTecsonline 2026.05 |
| HiddenLayer 2026 AI Threat Report | HiddenLayer 2026 |
| OWASP GenAI Q1 2026 | OWASP 2026.04 |

### 16.4 设计原则来源

| 原则 | 来源 |
|------|------|
| Ship > Plan > Discuss | Auto-Company 强制收敛 |
| 70% 信息决策 | Bezos/CEO 原则 |
| 宏伟单体优先 | DHH 开发哲学 |
| 渐进披露 | Don Norman 产品设计 |
| 最小可行受众 | Seth Godin 营销哲学 |
| 约定优于配置 | DHH/Rails 哲学 |
| 一切皆会失败 | Werner Vogels/CTO 原则 |
| 隐私确认闸门 | Marvis 隐私设计 |
| 做不扩展的事 | Paul Graham 创业哲学 |
| 单文件状态简化 | Auto-Company consensus.md |

### 16.5 新增调研来源（v8.0）

| 类型 | 来源 | 内容 |
|------|------|------|
| 竞品分析 | Paperclip(70K⭐)、OpenHuman(27K⭐)、OpenFang(16K⭐) | 2026.06 调研 |
| 安全事件 | CVE-2026-2256, MCP Tool Poisoning, OWASP Q1 2026 | 2026 Q1-Q2 |
| 用户洞察 | Reddit r/AI_Agents, r/aisolobusinesses | 2026 持续 |
| 代码审计 | Morn 代码自审（44,793 LOC / 862 unwrap） | 2026.06 |
| 一人公司工具栈 | OPC Community (50+工具) | 2026.06 |

---

> **Morn — 你的桌面 AI 创作系统。**
> 从底层搭起，也可以拿来就用。
> 所有创作、使用、管理、交易，都在一个本地桌面应用里。
> 从一个人的工位开始。
>
> 设计总纲 · v7.0 · 2026年6月
> 整合自 200+ 市场调研来源 · 代码审计确认

---

## 附录 A：Tauri API 接口定义

### A.1 总览

88 个 Tauri 命令，分 26 组：Chat（3）、Studio（11）、Market（17）、Console（2）、Org（8）、System（2）、Analytics（2）、Backup（2）、Collaboration（3）、ComponentType（3）、Config（2）、Cost（2）、Earnings（1）、Execution（1）、Journey（1）、LocalModel（3）、Mcp（3）、Memory（3）、Metrics（1）、Notifications（2）、OAuth（2）、Plugin（1）、Proactive（2）、Recovery（2）、Sandbox（3）、Scheduler（3）、TeamTemplates（1）、Whisper（2）。

所有命令通过 `invoke_handler` 注册在 `src-tauri/src/lib.rs`，前端通过 `@tauri-apps/api/core` 的 `invoke()` 调用。

### A.2 Chat 命令

#### `send_message`
```
参数: text: String
返回: SendMessageResult { text: String, execution_events: Vec<ExecutionEvent> }
状态: ✅ 已走 Supervisor 管线，返回执行事件
权限: 无
```

#### `get_status`
```
参数: 无
返回: { turn_count: u64, version: String }
```

#### `clear_history`
```
参数: 无
返回: ()
```

### A.3 Studio 命令

#### `list_components`
```
参数: type_filter: Option<String> (如 "tool"/"agent"/"workflow")
返回: Vec<ComponentSummary> — { id, name, component_type, status, updated_at }
```

#### `get_component`
```
参数: id: String
返回: ComponentDetail — 完整组件定义
```

#### `create_component`
```
参数: name: String, component_type: String, config_json: Option<String>
返回: String (新组件 ID)
```

#### `update_component`
```
参数: id: String, name: Option<String>, config_json: Option<String>, status: Option<String>
返回: ()
```

#### `delete_component`
```
参数: id: String
返回: ()
```

#### `assemble_agent`
```
参数: name: String, persona: String, model: String, 
       tools: Vec<String>, knowledge: Vec<String>, skills: Vec<String>
返回: { agent_id: String }
说明: 按 persona 名查找预设人格（researcher/analyst/writer/coder 等），
      组装完整 AgentDef，注册到 StudioManager
```

#### `list_agent_templates`
```
参数: 无
返回: Vec<AgentTemplate> — { id, name, icon, description, persona, model, tools, knowledge, skills }
```

#### `list_component_types`
```
参数: 无
返回: Vec<{ type: String, label: String, icon: String }>
硬编码: [agent/tool/workflow/knowledge/persona]
```

#### `test_component`
```
参数: id: String, input: String, component_type: Option<String>
返回: TestResult — 包含执行日志、耗时、成本
```

#### `test_component_rerun`
```
参数: id, component_type, step_index: usize, new_input: String
返回: StepResult — 该步骤的执行结果
```

#### `publish_component`
```
参数: id: String
返回: ()
说明: 将本地组件发布到 Market
```

### A.4 Market 命令

#### `get_market_listings`
```
参数: type_filter: Option<String>
返回: Vec<Listing> — { id, item_type, name, description, price, author, rating, downloads }
```

#### `list_bot_store`
```
参数: 无
返回: Vec<BotListing> — 10 个硬编码预设 Bot
状态: 🟡 硬编码，后续应从数据库读取
```

#### `get_preset_persona`
```
参数: name: String
返回: Persona 完整定义
```

#### `list_preset_personas`
```
参数: 无
返回: Vec<{ name, display_name, category }>
```

#### `create_agent_from_description`
```
参数: nl: String (自然语言描述)
返回: String (序列化的 NLAgentDef)
说明: 调 Supervisor.create_agent_from_nl()，6 步 LLM 推理链
```

#### `install_bot_from_store`
```
参数: bot_id: String, template_id: String
返回: String (安装的 Agent ID)
状态: ✅ 后端命令已实现，via BotStore.tsx handleInstall
```

### A.5 Console 命令

#### `get_system_status`
```
参数: 无
返回: { dashboard: DashboardData, system_info: SystemInfo }
```

#### `get_component_topology`
```
参数: 无
返回: Vec<TopologyNode> — 组件连接拓扑图
```

### A.6 Org 命令

#### `create_user`
```
参数: username: String, display_name: String, role: String
返回: String (序列化 User)
```

#### `list_users`
```
参数: 无
返回: Vec<User>
```

#### `create_team`
```
参数: name: String, description: String, owner_id: String
返回: String (序列化 Team)
```

#### `list_teams`
```
参数: 无
返回: Vec<Team>
```

#### `add_member`
```
参数: team_id: String, user_id: String, role: String
返回: String (序列化 Member)
```

#### `remove_member`
```
参数: team_id: String, user_id: String
返回: ()
```

#### `grant_permission`
```
参数: user_id: String, agent_id: String, permission: String, team_id: Option<String>
返回: String (序列化 Permission)
```

#### `revoke_permission`
```
参数: user_id: String, agent_id: String
返回: ()
```

#### `get_audit_log`
```
参数: user_id: Option<String>, action_type: Option<String>, limit: Option<u64>
返回: Vec<AuditLogEntry>
```

---

## 附录 B：数据模型与存储

### B.1 核心 struct 定义

#### `NLAgentDef` — Agent 的自然语言构建结果
```rust
pub struct NLAgentDef {
    pub name: String,                    // Agent 名称
    pub persona: String,                 // 人格名称
    pub model: String,                   // 模型名称
    pub tools: Vec<String>,              // 工具 ID 列表
    pub knowledge: Vec<String>,          // 知识源 ID 列表
    pub skills: Vec<String>,             // 技能 ID 列表
    pub memory: Vec<String>,             // 记忆配置 ID 列表
    pub persona_config: NLPersonaConfig, // 人格参数 + 5 层 Prompt
    pub communication_style: String,     // 沟通风格
    pub suggestions: Vec<String>,        // COO 建议
}
```

#### `SubTaskDef` — 子任务定义
```rust
pub struct SubTaskDef {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub params: serde_json::Value,
    pub depends_on: Vec<String>,   // 依赖的子任务 ID 列表（DAG 拓扑排序用）
}
```

#### `TaskPlan` — 任务计划
```rust
pub struct TaskPlan {
    pub task_id: String,
    pub user_input: String,
    pub subtasks: Vec<SubTaskDef>,
    pub estimated_secs: u64,
    pub decision_level: String,       // direct_answer / single_tool / single_agent / team / workflow / jump_studio
    pub approval_required: bool,
}
```

#### `ComponentTypeDef` — 可扩展组件类型定义
```rust
pub struct ComponentTypeDef {
    pub type_name: String,           // 如 "vision_model"
    pub interfaces: Vec<String>,     // 必须实现的接口名
    pub config_schema: serde_json::Value,  // JSON Schema
    pub implements: Vec<String>,     // 依赖的其他类型
    pub author: String,
    pub version: String,
}
```

#### `DecisionLevel` — 6 级决策
```rust
pub enum DecisionLevel {
    L1DirectAnswer,    // 直接 LLM 回答，¥0.001/0.5s
    L2SingleTool,      // 调一个工具，¥0.003/1s
    L3SingleAgent,     // 单 Agent 执行，¥0.02/5s
    L4Team,            // 多 Agent 团队，¥0.05/15s
    L5Workflow,        // 工作流模板，¥0.03/10s
    L6JumpToStudio,    // 跳创作台构建
}
```

#### `DecisionTier` — 3 挡审批
```rust
pub enum DecisionTier {
    Operational,  // 信任>60 + 低风险 → 自动执行
    Tactical,     // COO 给建议，CEO 确认
    Strategic,    // 必须 CEO 决策
}
```

#### `Mode` — COO 工作模式
```rust
pub enum Mode {
    Proactive,   // 默认，低风险自动，高风险等确认
    Safe,        // 每步展示影响，确认才执行
    Automated,   // 定时/触发，自动执行完推送
}
```

### B.2 SQLite 完整 Schema（24 张表）

```sql
-- 组件与 Agent
agents (id, name, component_type, config_json, status, trust_score, created_at, updated_at)
capabilities (id, agent_id, name, domain, actions, description, trust_score)

-- 任务执行
tasks (id, user_input, plan_json, status, created_at, completed_at)
subtasks (id, task_id, agent_id, action, params_json, status, result_json, started_at, finished_at)
executions (id, agent_id, task_id, action, status, latency_ms, error_msg, created_at)
decisions (id, task_id, decision_level, action, context_json, approved, created_at)
checkpoints (id, session_id, step_index, step_name, state_json, metadata_json, parent_id, created_at)
approval_requests (id, action, level, status, context_json, requested_by, responded_at, response, created_at)

-- 组件连接拓扑
bindings (id, source_agent_id, target_agent_id, source_port, target_port, binding_type, config_json, created_at)

-- 市场
market_listings (id, item_type, name, description, price, author, rating, downloads, created_at)
market_transactions (id, listing_id, buyer, amount, timestamp)
market_licenses (id, listing_id, user_id, granted_at, expires_at)

-- 组织
users (id, username, display_name, role, created_at, last_login)
teams (id, name, description, owner_id, created_at)
team_members (id, team_id, user_id, role, joined_at)
agent_permissions (id, agent_id, user_id, team_id, permission, granted_at)
audit_log (id, user_id, action, target_type, target_id, details_json, created_at)

-- 学习与规则
decision_rules (id, user_id, keyword, level, trust_threshold, auto_execute, source, hit_count, last_used_at, created_at)
privacy_rules (id, pattern, sensitivity, action, created_at)

-- 多端同步
sync_events (id, entity_type, entity_id, action, data_json, timestamp, device_id, synced)
devices (id, name, last_seen, public_key)
oauth_tokens (id, provider, user_id, access_token, refresh_token, expires_at, scope, created_at)

-- 会话与设置
sessions (id, user_id, agent_id, status, context_json, created_at, updated_at)
settings (key, value)
```

### B.3 Key 数据流

```
用户输入
  → send_message(text)
  → Supervisor.parse_with_llm(text, chat_fn) → Intent
  → Supervisor.decide_level() 或 decide_weighted() → DecisionLevel
  → Planner::plan(intent, subtasks) → TaskPlan
  → Scheduler::schedule(workflow_id, plan) → Vec<subtask_ids>
  → Supervisor.execute_plan(plan, chat_fn) → TaskResult
  → 返回结果文本
  → Supervisor.record_turn("user", text)
  → Supervisor.record_turn("assistant", response)

组件创建
  → create_component(name, component_type, config_json)
  → StudioManager.create_component(def) → 写入 agents 表
  → 返回 agent_id

Agent 组装
  → assemble_agent(name, persona, model, tools, knowledge, skills)
  → StudioManager.assemble_agent(AgentDef)
  → 创建 AgentRecord → 写入 agents 表
  → 返回 agent_id

市场安装
  → install_bot_from_store(bot_id, template_id)
  → 查找预设模板 → 调 assemble_agent 创建 Agent → 注册到本地
  → 返回 agent_id
```

---

## 附录 C：UI 交互流程

### C.1 全局布局

```
┌──────────────────────────────────────────────────────────┐
│  导航栏                                                    │
│  [💬 Workbench]  [🎨 Studio]  [🏪 Store]  [📋 Console]   │
├──────────────────────────────────────────────────────────┤
│                                                            │
│  内容区域（按视图切换）                                     │
│                                                            │
│  Workbench → 聊天界面（默认）                              │
│  Studio   → 组件创作界面                                   │
│  Store    → Bot 商店                                       │
│  Console  → 管理仪表盘                                     │
│                                                            │
├──────────────────────────────────────────────────────────┤
│  状态栏: 🟢 Agent 状态 | 💰 今日费用 | ⚡ 模型 Provider   │
└──────────────────────────────────────────────────────────┘
```

### C.2 交互流程 1：首次打开 → 聊天

```
安装 Morn → 运行 .exe
  → 检测到无 API Key
  → 显示欢迎页（状态 A）
  → 用户选择「使用内置中转」
  → 设置默认 key → Tauri setup()
  → ModelRouter 初始化
  → 切换到聊天界面（状态 B）
  → 用户看到「试试说」示例按钮
  → 点击或输入文字 → send_message
  → Supervisor 管线处理 → 返回结果
```

### C.3 交互流程 2：Bot Store → 安装 → 使用

```
用户点击 Store tab
  → 显示 BotStore 组件（10 个预设 Bot）
  → 用户选择「Data Analyst」→ 点击 Install
  → install_bot_from_store("b1", "preset-analyst")
  → 后端查找预设 → assemble_agent → 注册到 Registry
  → 返回 agent_id → 前端显示 Installed ✓
  → 用户回到 Workbench
  → 输入"分析一下最近的数据趋势"
  → Supervisor 识别意图 → 调 Data Analyst Agent
  → 执行分析 → 返回结果
```

### C.4 交互流程 3：Studio 一句话构建 Agent

```
用户点击 Studio tab → 选择 Builder
  → 看到一句话构建输入框
  → 输入"创建一个分析股票的研究员"
  → create_agent_from_description(nl)
  → Supervisor.create_agent_from_nl()
    → Step 1: 领域识别 → "股票分析/金融"
    → Step 2: 角色推断 → "研究员"
    → Step 3: 能力推断 → 需要数据获取、指标计算、报告生成
    → Step 4: 工具推断 → get_kline, calc_macd, web_search
    → Step 5: 知识推断 → stock_db, financial_terms
    → Step 6: 人格推断 → persona: analyst, temperature: 0.3
  → 返回 NLAgentDef
  → 前端展示 Agent 定义
  → 用户点「直接保存」→ assemble_agent → 注册到工作台
  → 用户回到 Workbench → 新 Agent 可用
```

### C.5 四视图线框图

#### Workbench（默认聊天）
```
┌──────────────────────────────────────────────────┐
│  Morn                            🟢 已连接  ⚙️ 🗑️ │
├──────────────────────────────────────────────────┤
│  {messages.length === 0 ?                        │
│    <WelcomeScreen />             ← 3 种状态之一  │
│  :                                               │
│    messages.map(msg =>                           │
│      <MessageBubble role={msg.role}>             │
│        {msg.content}                             │
│      </MessageBubble>                            │
│    )                                             │
│  }                                               │
│                                                  │
│  <QuickActions onSend={...} />    ← 快捷按钮     │
│  ┌──────────────────────────────────────────┐    │
│  │ [输入框...]                       [发送]  │    │
│  └──────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

#### Studio
```
┌──────────────────────────────────────────────────┐
│  创作台                                            │
│  [Component Editor] [Agent Builder] [Test Runner] │
├──────────────────────────────────────────────────┤
│  {studioTab === "editor" && <ComponentEditor />}  │
│  {studioTab === "builder" && <AgentBuilder />}    │
│  {studioTab === "test" && <TestPanel />}          │
└──────────────────────────────────────────────────┘

Agent Builder 子布局：
┌──────────────────────────────────────────────────┐
│  一句话构建输入框                      [生成]     │
│  ┌──────────────────────────────────────────┐    │
│  │ 例如：创建一个分析股票的研究员            │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  ┌──────────────────────────────────────────┐    │
│  │ 模板选择                                    │    │
│  │ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐      │    │
│  │ │研究员│ │分析师│ │写手  │ │程序员│      │    │
│  │ └──────┘ └──────┘ └──────┘ └──────┘      │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  配置面板：工具/知识/人格/模型/记忆              │
│  5 步引导式配置                                   │
└──────────────────────────────────────────────────┘
```

#### Store
```
┌──────────────────────────────────────────────────┐
│  Bot Store                            [搜索...]   │
│  [All] [Analysis] [Research] [Writing] [Coding]   │
├──────────────────────────────────────────────────┤
│  ┌──────┐  ┌──────┐  ┌──────┐                    │
│  │📊    │  │🔬    │  │✍️    │                    │
│  │Data  │  │Resrch│  │Writr │                    │
│  │Analyst│  │Asst  │  │      │                    │
│  │★4.8  │  │★4.7  │  │★4.6  │                    │
│  │[Install]│[Install]│[Install]│                    │
│  └──────┘  └──────┘  └──────┘                    │
│  ┌──────┐  ┌──────┐  ┌──────┐                    │
│  │💻    │  │🌐    │  │🤖    │                    │
│  │Code  │  │Trans │  │System│                    │
│  │Engnr │  │Pro   │  │Asst  │                    │
│  │★4.9  │  │★4.5  │  │★4.4  │                    │
│  │[Install]│[¥0.001]│[Install]│                    │
│  └──────┘  └──────┘  └──────┘                    │
└──────────────────────────────────────────────────┘
```

#### Console
```
┌──────────────────────────────────────────────────┐
│  管理台                                            │
│  [Dshbrd] [Topo] [System] [Cost] [Gov] [Sec] [Mkt]│
├──────────────────────────────────────────────────┤
│  {consoleTab === "dashboard" && <AdminDashboard>}  │
│  {consoleTab === "topology" && <Topology />}       │
│  {consoleTab === "system" && <SystemInfo />}       │
│  {consoleTab === "cost" && <CostCenter />}         │
│  {consoleTab === "governance" && <Governance />}   │
│  {consoleTab === "security" && <Security />}       │
│  {consoleTab === "market" && <Marketplace />}      │
└──────────────────────────────────────────────────┘

Dashboard 子布局：
┌──────────────────────────────────────────────────┐
│  系统总览                                          │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐            │
│  │任务  │ │Agent │ │今日费│ │组件  │            │
│  │12完成│ │3在线 │ │¥0.32 │ │8注册 │            │
│  └──────┘ └──────┘ └──────┘ └──────┘            │
│                                                    │
│  最近任务列表                                       │
│  ✅ 帮我分析茅台 (2m ago)                          │
│  ✅ 查一下电脑配置 (15m ago)                       │
│  ⏳ 写一份周报 (进行中)                            │
│                                                    │
│  告警: 无                                          │
└──────────────────────────────────────────────────┘
```

### C.6 3 种欢迎状态（§10.4 展开）

状态 A — 无 API Key：
```
┌─────────────────────────────────────┐
│  👋 欢迎使用 Morn                   │
│                                     │
│  你需要配置一个 AI 模型才能开始      │
│                                     │
│  [🔑 我有 API Key → 设置页]        │
│  [⚡ 使用内置中转（推荐）]          │
│                                     │
│  或者先逛逛：                        │
│  🏪 Store · 📖 Studio · ⚙️ 设置     │
└─────────────────────────────────────┘
```

状态 B — 有 Key，初次聊天：
```
┌─────────────────────────────────────┐
│  👋 你好，我是 Morn                 │
│  桌面 AI 系统已就绪 ✅              │
│                                      │
│  [📄 帮我写一份周报]                 │
│  [💻 查一下电脑配置]                 │
│  [🔍 搜索 AI 最新消息]              │
│  [📊 分析这组数据]                   │
│                                      │
│  或去 Store 安装预置 Bot             │
└─────────────────────────────────────┘
```

状态 C — 调用失败：
```
┌─────────────────────────────────────┐
│  ⚠️ API 调用失败                    │
│                                      │
│  [🔑 检查 Key → 设置]              │
│  [🔄 切换 Provider]                 │
└─────────────────────────────────────┘
```

### C.7 状态管理

前端 React state 通过 `App.tsx` 管理：

```typescript
// 视图切换
type View = "workbench" | "studio" | "console" | "store";
const [view, setView] = useState<View>("workbench");

// 聊天状态
const [messages, setMessages] = useState<Message[]>([]);
const [input, setInput] = useState("");
const [isTyping, setIsTyping] = useState(false);

// 加载状态（每个视图独立 Skeleton）
const [loading, setLoading] = useState({ workbench: false, studio: false, console: false });

// 设置
const [showSettings, setShowSettings] = useState(false);

// 主题
const [theme, setTheme] = useState<Theme>("cyber");

// Studio 子视图
const [studioTab, setStudioTab] = useState<"editor" | "builder" | "test">("builder");
// Console 子视图
const [consoleTab, setConsoleTab] = useState<"dashboard" | "topology" | "system" | "cost" | "governance" | "security" | "market">("dashboard");
```

API 调用模式（`web/src/api.ts`）：
```
所有 API 接口统一走:
  try Tauri invoke → fallback HTTP remote → fallback Web fetch
通过 isTauri (window.__TAURI__) 和 isRemote (config.mode) 判断路径
```
