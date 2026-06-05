# Morn

> 你的桌面 AI 创作系统
> 设计总纲 · 2026年6月 · v5.0 最终版

---

## 一、What is Morn

Morn 是一个跑在 Windows 桌面的 AI 创作操作系统。

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

## 二、核心架构

### 2.1 四层组合模型

```
Layer 0: 基础设施
  LLM 模型 / 数据源 / 操作系统 / 网络 / MCP

      ↓ 组合

Layer 1: 原子组件
  工具（Tool）— 单一操作，get_kline、web_search、send_msg
  知识（Knowledge）— 静态信息，股票代码库、公司档案
  技能（Skill）— 流程模板，技术分析流程、报告生成
  人格（Persona）— 思维模型 + 行为定义，analyst、writer
  记忆（Memory）— 存储配置：短期/长期/经验/工作记忆
  模型（Model）— LLM 选择 + 参数 + 成本档位

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

### 2.2 标准组件接口

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

### 2.3 系统架构

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

### 2.4 进程模型

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

---

## 三、创作台（Studio）— 组件创作

### 3.1 一句话构建 Agent

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

### 3.2 创作哪些组件

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

### 3.3 Agent 人格的深度设计

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

### 3.3.1 实操参考：Auto-Company 的 14 专家人格

Auto-Company (MaxMiksa, nicepkg fork) 用纯 Markdown 文件实现了 14 个 Agent 人格，每个文件就是一个完整的专家定义。这是目前可查的最完整的开源人格实现，值得 Morn 直接参考。

**文件结构（`.claude/agents/` 目录下 14 个 `.md` 文件）：**

```
ceo-bezos.md         — CEO，Jeff Bezos 思维
cto-vogels.md        — CTO，Werner Vogels
critic-munger.md     — 逆向思考，Charlie Munger
product-norman.md    — 产品设计，Don Norman
ui-duarte.md         — UI 设计，Matías Duarte
interaction-cooper.md — 交互设计，Alan Cooper
fullstack-dhh.md     — 全栈开发，DHH
qa-bach.md           — 质量保证，James Bach
devops-hightower.md  — DevOps，Kelsey Hightower
marketing-godin.md   — 营销，Seth Godin
operations-pg.md     — 运营，Paul Graham
sales-ross.md        — 销售，Aaron Ross
cfo-campbell.md      — 财务，Patrick Campbell
research-thompson.md — 研究分析，Ben Thompson
```

**每个文件的模板：**

```yaml
---
name: ceo-bezos
description: "公司CEO（Jeff Bezos 思维模型）。当评估新产品/功能想法、商业模式和定价方向时使用。"
model: inherit
---

## Role                # 一句话角色定位
公司CEO，负责战略决策、商业模式设计、优先级判断。

## Persona             # 你是谁 + 受谁影响
你是一位深受 Jeff Bezos 经营哲学影响的 AI CEO。

## Core Principles     # 核心（5-7条原则，每条带具体操作）
### Day 1 心态
- 快速决策：多数决策是双向门（可逆的），70%信息就能行动
- 用 70% 的信息做决策，等到 90% 已经太慢

### 客户至上
- 一切从客户需求出发，逆向工作（Working Backwards）
- 先写 PR/FAQ（新闻稿+常见问题），再写代码

### 飞轮效应
- 每个决策问：这会加速飞轮还是减慢飞轮？

## Decision Framework  # 同样核心（具体场景下的决策流程）
### 有新想法时：
1. 这解决了什么客户问题？
2. 市场有多大？能成为有意义的业务吗？
3. 写出 PR/FAQ

### 优先级排序：
1. 不可逆决策（单向门）慎重，可逆决策（双向门）快推
2. 优先做能产生复利效应的事情

## Communication Style # 沟通风格
- 用数据和叙事结合
- 用 6 页备忘录而非 PPT

## Output Format       # 产出格式
1. 先明确客户是谁
2. 给出战略判断
3. 识别关键风险
4. 提出可执行的下一步
```

**关键发现：名字 vs 规则的真实作用**

| 要素 | 作用 | 没有它会怎样 |
|------|------|-------------|
| Core Principles（写死） | **保证行为可控**，决策不会跑偏 | LLM 随意发挥，行为不可预测 |
| Decision Framework（写死） | **保证输出结构一致** | 每次回答格式不同，无法程序化处理 |
| "你是 Bezos"（名字） | 激活 LLM 训练数据中的深层上下文 | 输出变教科书式，缺"经验感" |

核心结论：**名字激活知识，规则控制行为。两者缺一不可。**

只给名字不给规则 → LLM 会调用相关知识但行为不可控，可能产生幻觉。
只给规则不给名字 → 行为按规则走但输出比较干瘪，缺少深度经验感。

**对 Morn 的映射：**

Morn 的 `Persona` struct 字段与 Auto-Company 模板几乎一一对应：

| Morn Persona 字段 | Auto-Company 对应 | 说明 |
|---|---|---|
| `prompt_layers.l1_core_identity` | Persona 段首句 | 核心身份声明 |
| `prompt_layers.l2_skill_instructions` | Decision Framework | 技能指令层 |
| `prompt_layers.l3_format_template` | Output Format | 格式模板 |
| `prompt_layers.l4_constraints` | Core Principles 中的限制条款 | 约束规则 |
| `prompt_layers.l5_conversation_style` | Communication Style | 对话风格 |
| `core_principles` | Core Principles | 核心思维原则 |
| `decision_framework` | Decision Framework | 决策流程 |
| `anti_patterns` | （隐含于原则中） | 禁止的行为 |

**对预置人格模板的指导原则：**

1. 每个模板必须先写 `core_principles`（5-7条具体可操作的原则），再命名字
2. 每条原则必须带具体场景的操作方法——不是"要有数据驱动思维"，而是"先看数据再说话，不以单一指标下结论"
3. `decision_framework` 必须覆盖该人格最常见的 2-3 个典型场景
4. 名字可以不是真实人物，但必须是能激活 LLM 训练知识的鲜明角色原型
5. 人格模板是一个**可编辑的起点**，不是硬编码——用户可以改、删、加原则

### 3.4 组合画布

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

### 3.5 测试面板

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

### 3.6 协作组合模式（7 种）

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

### 3.7 Agent 构建指导

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

### 3.8 多 Agent 团队构建

单 Agent 做不了的事，交给团队。

**预置团队模板：**

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

### 3.9 工作流构建（预定义 + 自定义）

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

---

## 四、工作台（Workbench）— 日常使用

工作台是用户日常待的地方。用户说"帮我分析茅台"，COO 自己决定怎么干——直接回答、调工具、派 Agent、组团队、套工作流模板，还是甚至跳创作台现做一个。用户看到的就是结果。

### 4.1 交互流程

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

### 4.2 COO 的三种工作模式

```
🏃 主动模式（默认）
  低风险直接执行，高风险展示计划等确认

🛡 安全模式（高风险自动切换）
  每步展示要做什么、影响什么，确认才执行

🤖 自动化模式（定时/触发）
  按计划自动执行，完成后推送报告
```

### 4.3 COO 决策协议

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

### 4.4 接管整个电脑

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

## 五、管理台（Console）— 运营管理

### 5.1 导航视图

| 视图 | 功能 |
|------|------|
| 📊 仪表盘 | 系统总览：任务/成本/组件/告警 |
| 🤖 组件拓扑 | 图形化所有组件连接，拖拽断连/重连/替换 |
| 💰 成本中心 | 按组件/Agent/模型拆分的费用，预算管理，每日趋势 |
| ⚙ 治理 | 安全策略例外、API Key、渠道绑定、信任阈值、审批队列 |
| 🔐 安全 | Dual-LLM 拦截记录、宪法状态、审计日志 |
| 🏪 市场 | 上架商品、下载统计、创作者收益 |
| 💻 系统 | CPU/内存/磁盘、Morn 进程管理、日志搜索 |

### 5.2 信任评分

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

### 5.3 成本中心

```
按 Agent/工具/模型拆分成本
预算设置 + 超限行为（切低成本模型/暂停非必要/通知）
每日/每月趋势图
每笔调用记录可追溯
```

---

## 六、渠道与连接

### 6.1 全渠道列表

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

### 6.2 微信生态分层

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

## 七、市场（Marketplace）

### 7.1 商品类型

参考 When Agent Markets Arrive (Liu et al., 2026) + uTools 4000+ 插件生态：

| 类型 | 价格参考 |
|------|---------|
| 工具（Tool） | ¥0.001/次 |
| 知识（Knowledge） | 免费 ~ ¥0.01 |
| 技能（Skill） | ¥0.01/次 |
| 人格（Persona） | 免费 |
| Agent | ¥0.05/次 |
| 团队模板 | ¥0.20/次 |

### 7.2 交易模型

```
按次计费: 用户支付 → 扣除模型成本 → 创作者+平台分成
月订阅: 不限次数（合理使用）
收益: 创作者在管理台查看
```

### 7.3 生态关键路径

```
用户从市场中购买组件 → 在创作台组合成 Agent
→ 在工作台使用 → 效果好 → 发布到市场卖
→ 别人买了再组合 → 生态滚起来
```

---

## 八、安全模型

### 8.1 四层宪法

```
L1（硬编码）: 格式化磁盘/删系统文件/访问其他进程内存 → 不可绕过
L2（需审批）: 写工作区外目录/未注册域名/shell 命令 → CEO 确认
L3（需通知）: 读工作区文件/调用注册 API/沙箱代码 → 推送通知
L4（自由）: 对话/查数据/搜索 → 不限制
```

参考：Greywall 三层安全边界 + archestra 3.8k⭐ Dual-LLM

### 8.2 Dual-LLM 安全

```
用户输入 → 主 LLM（正常推理）
         → 副 LLM（不同模型，保守配置，检查注入风险）
               → 无风险 → 正常执行
               → 发现风险 → 阻断，标记 HIGH，等用户审批

6 个安全检查点（参考 archestra）：
  认证 → 参数验证 → 内容净化 → 权限检查 → 审计日志 → 路由放行
```

---

## 九、Phase 0 实现路径

### 9.1 目标

```
~1000 行代码。一个 .exe。一个全能 Agent。
端到端链路：安装 → 输入"帮我看看今天A股"→ 拆任务 → 执行 → 输出结果
```

### 9.2 模块分配

| 模块 | 行数 | 内容 |
|------|------|------|
| core/ | 400 | 最简 Supervisor + Registry + Task Engine |
| desktop/ | 150 | Tauri 入口 + 系统托盘 |
| web/ | 250 | 单页 React 聊天界面 |
| bridge/ | 100 | chat-agent 调用 DeepSeek |
| channel/ | 100 | 桌面端 + CLI |

### 9.3 不做的事

| 功能 | 原因 |
|------|------|
| 多 Agent | 只做 chat-agent 一个，验证链路 |
| 四层组合模型 | 硬编码 chat-agent |
| IM 渠道 | 仅桌面端 + CLI |
| 市场 | 无 |
| Dual-LLM | 单信任评分 |
| DAG 可视化 | 文字进度 |
| 电脑操控 | 仅 LLM 回答 |

---

## 十、路线图

```
Phase 0 ━━━━━━━ 立即
  单人 · 单 Agent · 桌面端 · CLI
  验证端到端链路

Phase 1 ━━━━━━━ 3个月
  组件体系（工具/知识/技能/人格/记忆/模型）
  创作台（组合 + 测试 + 发布）
  IM 渠道（企微/钉钉/飞书 + 小程序 + PWA）
  API + Webhook + 邮件
  多 Agent + 团队组合
  电脑操控（文件/应用/桌面）

Phase 2 ━━━━━━━ 6个月
  市场上线
  审批点 + Dual-LLM + 管理台
  QQ/Telegram/浏览器扩展
  成本中心 + 信任评分
  7 种协作模式

Phase 3 ━━━━━━━ 未来
  手机 App + 跨设备同步
  远程 Agent (A2A)
  企业多人版（可选）
  Agent 微调（LoRA）
```

---

## 十一、竞品全景

### 11.1 桌面 AI 工作台

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| WorkBuddy | 腾讯云 | 产品 | 桌面+IM远程、20+技能、MCP、定时 | 验证"桌面+IM"模式 |
| Kimi Work | 月之暗面 | 产品 | 300 Agent并行、13h连续、浏览器+文件控制 | 大规模并行验证 |
| Marvis | 腾讯 | 产品 | 1+5 OS层操控、隐私闸门、跨平台 | OS 操控设计参考 |
| UI-TARS Desktop | 字节跳动 | 27k | 视觉优先桌面操控、3模型尺寸、OSWorld顶级 | 桌面操控竞争者 |
| Kimi Work | 月之暗面 | Beta | 300预置Agent、13h连续编码 | 预置Agent数量启示 |

### 11.2 多渠道 AI 助手

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| QwenPaw | 阿里云 | 15k | 15+ IM渠道、自进化记忆、技能自动加载 | 渠道架构参考 |
| CowAgent | 社区 | 41k | 微信/企微/钉钉/飞书/QQ/公众号/Telegram | 中国多渠道需求验证 |
| OpenClaw | Peter Steinberger | 376k | 24+ Channel Adapter、Gateway+Sandbox | 渠道适配器架构 |

### 11.3 Agent 框架与编排

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| Dify | 社区 | 143k | 可视化画布、Chatflow+Workflow、RAG管线 | 画布节点类型参考 |
| LangGraph | LangChain | 28k | 有状态图、Checkpoint、Durable Memory | 图编排参考 |
| CrewAI | CrewAI | 46k | 角色Agent团队、事件驱动Flow | 角色团队模式 |
| MetaGPT | 社区 | 40k | SOP角色化Agent、ICLR2024 Oral | 角色工具包设计 |
| mission-control | 社区 | 5.2k | 信任评分、7种编排、SQLite零依赖 | 信任评分+编排模式 |
| archestra | 社区 | 3.8k | Dual-LLM安全、MCP注册、6检查点 | 安全架构参考 |
| Auto-Company | MaxMiksa | — | 14专家人格、强制收敛、6标准工作流 | 人格深度+流程模板 |

### 11.4 桌面端 AI 应用（Tauri/原生）

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| OpenHuman | tinyhumansai | 27k | Tauri+Rust、记忆树、TokenJuice压缩 | 记忆系统参考 |
| Kuse Cowork | kuse-ai | 5k | 纯Rust+Tauri、10MB、Docker隔离 | 极简架构验证 |
| Goose | Block→AAIF | 45k | Rust核心、MCP扩展、桌面+CLI+API | 三接口模式 |
| golutra | 社区 | 3.6k | Tauri+Vue3+Rust、多Agent编排 | Tauri技术验证 |
| Jan | janhq | 25k | 100%离线、插件式本地模型 | 离线架构参考 |

### 11.5 桌面操控 Agent

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| Agent S3 | Simular | 8k | 首个超人类 OSWorld 72.60%、体验学习 | 桌面操控SOTA |
| Open Interpreter | 社区 | 58k | 代码生成即工具、大规模验证 | 代码范式参考 |
| ScreenAgent | 学术 | — | VLM+VNC、Plan-Execute-Act | 桌面循环架构 |
| AssistGUI | 学术 | — | 层级规划+Critic验证 | 桌面任务分解 |
| Bytebot | 社区 | 6k | 容器化桌面、Agent自有OS | 沙箱方案参考 |

### 11.6 可视化构建与市场

| 产品 | 公司/作者 | ⭐ | 关键特征 | 对 Morn 的启发 |
|------|----------|-----|---------|---------------|
| Langflow | Langflow | 147k | 可视化画布、工作流自动成API | 画布+API模式 |
| Sim Studio | simstudioai | 10k | Copilot生成节点、自然语言→工作流 | AI辅助构建 |
| Superpowers | obra | 57k | SKILL.md技能格式、自治发现 | 技能标准化 |
| Composio | ComposioHQ | 13k | 1000+工具、托管认证、MCP-native | 工具注册表 |
| uTools | 独立团队 | 5M用户 | 4000+插件、plugin.json、搜索启动器 | 插件生态参考 |

### 11.7 中国市场新入局

| 产品 | 公司 | 关键特征 |
|------|------|---------|
| DuClaw | 百度 | 零部署 OpenClaw 云服务 |
| QClaw | 腾讯 | OpenClaw 整合微信生态 |
| Wukong | 阿里巴巴 | 企业级多 Agent 编排 |
| Doubao 2.0 | 字节跳动 | 中国最易用 AI 助手，多Agent架构 |
| Accio Work | 阿里国际 | 本地优先、读文件/跑终端/控浏览器 |
| Manus + My Computer | Monica.im | 自主完成复杂任务，桌面端"My Computer" |

---

## 十二、参考来源

### 12.1 学术论文（18 篇）

| # | 论文 | 会议/期刊 | 年份 | 引用位置 |
|---|------|----------|------|---------|
| 1 | ReAct: Synergizing Reasoning and Acting in Language Models | ICLR | 2023 | 核心循环 |
| 2 | Toolformer: Language Models Can Teach Themselves to Use Tools | Meta AI | 2023 | 工具内化 |
| 3 | Reflexion: Language Agents with Verbal Reinforcement Learning | NeurIPS | 2023 | 后执行反思 |
| 4 | MetaGPT: Meta Programming for Multi-Agent Collaborative Framework | ICLR (Oral) | 2024 | 角色化Agent |
| 5 | Chain of Agents: LLMs Collaborating on Long-Context Tasks | NeurIPS | 2024 | 链式拓扑 |
| 6 | MetaAgent: Constructing MAS Based on Finite State Machines | ICML | 2025 | FSM引擎 |
| 7 | LLM-based Multi-Agent Orchestration: A Survey | Preprints | 2026 | 7种组合模式 |
| 8 | From Persona to Personalization: Survey on Role-Playing Agents | TMLR | 2024 | 人格三层架构 |
| 9 | Character-LLM: A Trainable Agent for Role-Playing | EMNLP | 2023 | 人格微调 |
| 10 | PERSONA: Dynamic Compositional Inference-Time Personality Control | — | 2025 | 人格向量组合 |
| 11 | OSWorld: Benchmarking Multimodal Agents in Real Computer Environments | NeurIPS | 2024 | 桌面操控评测 |
| 12 | ScreenAgent: VLM-driven Computer Control Agent | IJCAI | 2024 | Plan-Execute-Act |
| 13 | GUI Agents: A Survey | ACL Findings | 2025 | 三模态感知 |
| 14 | AssistGUI: Task-Oriented Desktop GUI Automation | CVPR | 2024 | 层级规划+Critic |
| 15 | AI Agents for Computer Use: A Comprehensive Survey | — | 2026 | 安全三层授权 |
| 16 | When Agent Markets Arrive | — | 2026 | Agent市场 |
| 17 | Rise and Potential of LLM Based Agents: A Survey | Sci. China | 2025 | 三组件架构 |
| 18 | Multi-Agent Collaboration Mechanisms: A Survey of LLMs | — | 2025 | 协作分类 |

### 12.2 GitHub 项目（34 个）

Auto-Company, OpenHuman, Kuse Cowork, AionUi, NeuralAgent, Jan, UI-TARS Desktop, Agent S3, Open Interpreter, Bytebot, Goose, Superpowers, Composio, AgentScope (阿里), Langflow, Sim Studio, Dify, LangGraph, CrewAI, MetaGPT, mission-control, archestra, golutra, QwenPaw, CowAgent, OpenClaw, Hermes Workspace, Hermes Desktop, WorkBuddy, Kimi Work, Marvis, uTools, SkillsMP, ClawHub

### 12.3 产品参考（15 个）

WorkBuddy (腾讯), Marvis (腾讯), Kimi Work (月之暗面), QwenPaw (阿里), CowAgent, Dify, uTools, DuClaw (百度), QClaw (腾讯), Wukong (阿里), Doubao 2.0 (字节), Accio Work (阿里国际), Manus (Monica.im), Anthropic Claude Cowork, OpenAI Operator

### 12.4 设计原则来源

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

---

> **Morn — 你的桌面 AI 创作系统。**
> 从底层搭起，也可以拿来就用。
> 所有创作、使用、管理、交易，都在一个本地桌面应用里。
> 从一个人的工位开始。
