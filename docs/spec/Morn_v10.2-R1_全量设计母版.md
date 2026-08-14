# Morn v10.2-R1 全量设计母版

## 智能工作、混合组织、可组合 Harness、可进化数字工厂与行业 AI 梦工厂操作系统

**副标题：从 Stable Semantic Kernel、DeepSeek Harness/Cordis、WorkPackage/Workcell 到 Evolution Engine、Evolving Digital Factory 与 Dream Factory**  
**文档状态：Reference Architecture / Implementation Baseline**  
**日期：2026-08-14**  
**继承基线：Morn v10.1-R1 全量设计母版；向前继承 v10.0-R1 与 Morn v8.2（`DESIGN (32).md`）**

> **重要说明**：本版不是另起炉灶，而是在 v10.1-R1 全量母版基础上的结构性升级。v10.1 已把 Harness、数字分身、Work-as-a-Service、Work System as Code 纳入正式架构；v10.2 进一步吸收 DeepSeek 官方 DeepSeek Harness、Cordis 的时空可组合性、Siemens Industrial AI / Digital Twin / hybrid workforce 路线，以及本轮关于“数字工厂可持续衍生与进化”的设计讨论。新增重点不是继续堆模块，而是冻结四个更深的不变量：**Stable Semantic Kernel + Dynamic Capability Fabric；Capability Seam + Governed Composability；生产世界与 Evolution World 分离；Morn 的最终优化目标不是 Agent 数量，而是以最低智能复杂度持续提升真实 Outcome。** Morn 因而从“可信智能工作的生产与交付系统”进一步升级为**可创建、模拟、运行并持续进化数字组织/数字工厂的 Evolvable Intelligent Work OS**。

---

# 0. 文档目标与阅读方式

这份文档不是“再加一批模块”的扩展清单，而是对 Morn 的**对象模型、运行闭环、产品边界、构建/复用策略和行业落地顺序**进行持续收敛。

Morn 的演进目标从：

```text
组件 → Agent → 多 Agent 团队
```

升级为：

```text
目标/问题
→ Operational World
→ WorkPackage / Work Contract
→ Workcell / Mixed Organization
→ Harness + Runtime
→ Artifact / Decision / Action / Outcome
→ Verified Delivery
→ Solution
→ Dream Factory
→ 真实运行反馈与行业资产回流
```

本文同时区分三种层次：

1. **完整参考架构**：Morn 长期应具备的对象、协议和边界；
2. **近期工程基线**：当前代码仓库真正需要重构和实现的能力；
3. **后期能力**：只有在真实客户、真实 Outcome 和足够数据出现后才值得建设的预测、因果和自治能力。

全文采用一个原则：**不以“模块多”证明先进，而以核心原语是否清晰、系统是否可验证、是否能落到真实工作来判断。**

## 0.1 v10.2-R1 本轮整合重点

本轮仍不通过增加新的“产品层”来膨胀架构，而是在现有 Kernel、Operational World、Work、Harness、Assurance、Foundry 与 Dream Factory 上增加**可组合执行与受治理进化**两条纵深能力：

1. **Stable Semantic Kernel + Dynamic Capability Fabric**：Identity、World、WorkPackage、Artifact、Decision、Outcome、Authority 等正式语义由 Morn 固定；Model、Agent Loop、Tool、Memory Backend、Sandbox、Runtime 等执行能力可以动态替换；
2. **DeepSeek Harness 作为首个重点 Harness Provider**：不把 Morn fork 成 DeepSeek Harness，也不把 DSH Session 当成 Morn Source of Truth；通过 `Morn Harness Provider API / morn-dsh-bridge` 接入其 Agent Loop、Tool、Session、Sandbox、Skill、Subagent 等成熟执行能力。[R51][R52]
3. **Capability Seam**：吸收 DeepSeek Harness 的 Service Definition / Provider / Consumer seam 思想，统一 Morn 内不同 Analytics、Sandbox、Agent Execution、Simulation、Storage、Tool 等 Provider 的可替换边界。[R52]
4. **Governed Composability**：吸收 Cordis 的 spatial/temporal composability、effect/coeffect 思想，但将其扩展到企业真实工作中的权限、状态、证据和责任；技术可组合不等于组织上可挂载。[R53][R54]
5. **Effect Classification**：正式区分 Lifecycle Reversible、Transactional、Compensatable、Irreversible 四类 Effect，避免把“插件可卸载”误认为“现实世界副作用可逆”；
6. **Evolving Digital Factory / Digital Organization**：数字工厂不只是数字孪生，而是 `Operational World + Twin + Work + Workforce + Software/Device + Verification + Outcome + Evolution`；Virtual Factory 负责分支、试错和验证；
7. **Replacement Ladder / Domain Penetration Loop**：Morn 进入行业时从 Observe/Connect 开始，逐步 Assist、Orchestrate、Shadow、Partial Replace、Native Replace，而不是第一天重写 ERP/MES/APS；
8. **Evolution Engine 成为第五闭环**：从 Trace、Process、Outcome 和 Human Correction 中发现可复用能力、冗余流程、岗位重构和软件吸收机会，通过 Branch → Replay/Simulation → Evaluation → Shadow → Promotion 受治理升级；
9. **最小充分智能原则**：如果稳定的 Work 可以由 Rule、Program、Solver 完成，就不继续使用高成本 LLM/Agent；系统进化可能表现为“把 Agent 蒸馏成确定性能力”。

这些变化进一步收敛 Morn 的战略中心：**Morn 不追求拥有最多 Agent，而追求让真实工作的结构、执行方式、责任边界与组织形态本身成为可计算、可验证、可版本化、可替换、可进化的对象。**

# 1. 执行摘要

## 1.1 最终定位

Morn v10.2 的推荐定义是：

> **Morn 是一个本地优先、用户拥有、Harness/Runtime 中立、可持续进化的智能工作与混合组织操作系统。它把自然语言目标、Operational World、行业资产和既有软件/数据/设备编译成可委托的 WorkPackage/Work Contract，由 Human、Actor、确定性程序、外部服务与 Device 组成的 Workcell 执行；底层通过可组合 Harness/Capability Provider 使用 DeepSeek Harness、Codex、Claude Code、Hermes、OpenClaw、openJiuwen 或 Morn Native 等执行环境，上层通过 Artifact、Decision、Action、Verification 与 Outcome 保持正式事实和责任链。Morn 还能把生产运行轨迹、失败、人类纠错和真实 Outcome 送入受治理 Evolution Engine，在虚拟/影子世界中生成、评估和认证新的 Skill、Workflow、Workcell、Role、Native Module 与组织结构，使数字部门、数字实验室和数字工厂能够持续演化。**

Morn 的上层不是单一“数字员工工厂”，而是可以建设多个行业 AI 梦工厂的**元工厂**：

```text
Morn Core / Intelligent Work OS
      ↓
Solution & Organization Foundry
      ↓
BioLab / Pharma / Manufacturing / Personal ... Dream Factory
      ↓
Certified Work Capability / Work-as-a-Service
      ↓
Customer Solution / Personal Work System
```

## 1.2 Morn 不是什么

Morn 不应被定位成：

- 一个聊天机器人桌面壳；
- 一个“比 OpenClaw/Hermes 多几个 Tool”的个人 Agent；
- 一个更小的 AgentArts/openJiuwen；
- 一个 Golutra/AgentSpace/Commonly/Tutti 的简单复制；
- 一个纯多 Agent 编排器；
- 一个纯 Ontology/Knowledge Graph 产品；
- 一个 RPA 替代品；
- 一个数据湖、BI、MLOps、数字孪生或流程挖掘平台；
- 一个只服务企业数字员工的 HR 系统；
- 一个一开始就声称拥有“通用企业世界模型”的系统；
- 一个靠自动创建十几个 Agent 就称为“数字团队”的产品；
- 一个把“数字分身”简化成人格 Prompt 或聊天记录模仿器的系统。

## 1.3 最重要的十二个架构转变

### 转变 A：从 Agent-first 到 Work-first

Agent 只是执行者，不是中心。中心是：**真实世界对象、目标、工作、正式交付与 Outcome**。

### 转变 B：从 Team of Agents 到 Mixed Work System

团队成员不全是 Agent。Morn 一等支持：Human、Actor、Deterministic Worker、External Service/Agent、Device/Robot。

### 转变 C：从 Chat-first 到 Artifact/Decision/Outcome-first

消息只负责交流；正式工作以有 Schema、版本、状态、来源、审批和血缘的对象推进。

### 转变 D：从 Agent Builder 到 Organization/Solution Compiler

用户描述目标后，Morn 不直接生成 Agent，而是编译问题、工作包、岗位、成员类型、执行模式、能力缺口、Harness、Workflow、Policy、Evaluation 与 Deployment。

### 转变 E：从模板商店到 Dream Factory

行业资产的最终商品不是“某个 Agent”，而是**经过验证的 Certified Work Capability**。

### 转变 F：从 Runtime-first 到 Harness + Runtime 双层执行模型

模型或 CLI 不是完整工作系统。Morn 必须明确区分：

```text
Actor = 谁
Role = 负责什么
Harness = 怎样可靠地工作
Runtime = 具体在哪里/靠什么执行
```

Codex CLI、Claude Code、Hermes、OpenClaw 等都可能同时承担 Context、Tool、Session、Planning、Memory 与 Event Normalization，因此需要统一 `ExecutionHarness` 接口，而不是仅写 `runtime = codex`。[R41][R42]

### 转变 G：从“Agent Persona”到组织身份、代表权与责任

必须区分：

```text
Independent Actor
Role-derived Actor
Human-delegated Actor / Digital Delegate
```

尤其是真人数字分身，需要 `RepresentationContract` 明确它可以代表谁、在什么范围、是否只能建议、哪些决定永远不能代签或代投票。

### 转变 H：从 Tool/SaaS 交付到 Work Contract / Outcome Delivery

客户最终购买的不一定是 Agent、席位或软件许可，而可能是：

> “把这项工作按这个质量、时限和证据标准交付给我。”

因此 WorkPackage 必须具备 OutcomeContract、DeliveryReceipt、质量 SLO、人工兜底和验收/计费基础，使 Dream Factory 可以支持 Work-as-a-Service / Outcome BaaS。[R47]


### 转变 I：从固定 Harness 到可组合 Capability Fabric

DeepSeek Harness 进一步证明，Harness 自身不应被视为一个固定大块：模型适配器、Tool Registry、Session Log、Agent Loop 等都可以作为可替换 Provider。Morn 因此采用 `Capability Definition → Provider → Consumer` seam，并保持自己的正式语义 Kernel 不被插件重新定义。[R51][R52]

### 转变 J：从“Everything is a Plugin”到 Stable Kernel + Governed Composability

Morn 不采用“所有东西都是插件”的业务语义。`Identity / World / Work / Decision / Outcome / Authority / Ledger` 属于不可随意重定义的 Semantic Kernel；只有执行与能力层允许动态挂载。Cordis 的 effect/coeffect 用于描述技术依赖和生命周期，Morn 再增加 Authority、Operational、Evidence、Accountability 维度。[R53][R54]

### 转变 K：从自动化现有流程到持续重构 Work System

Morn 不把既有软件、岗位和部门结构当作天然不变量。系统应从真实 Work/Event/Outcome 中识别重复协调、低价值审批、可确定化 Agent 工作、可吸收软件模块和可重构 Role，形成 Evolution Candidate。

### 转变 L：从 Digital Twin 到 Evolving Digital Organization / Factory

Digital Twin 只回答“现在是什么样”；Morn 的终局是让组织和工厂的 Work Graph、Workforce、Software Mapping、Capability、Policy 与 Automation Level 本身可被 Branch、Simulation、Shadow、Version 和 Promotion，最终形成可持续演化的数字组织。

# 2. 为什么 2026 年必须重新定义 Morn

## 2.1 Agent OS 已经不再稀缺

AIOS、OpenFang、openJiuwen 等已经把 Agent Runtime、模型调用、Memory、Tool、MCP/A2A、多 Agent、Sandbox、调度、安全和桌面/Studio 等能力做成系统化底座。[R01][R02][R03]

因此，下列能力只能算 Morn 的基础设施：

```text
Agent Loop
Model Router
Memory
Skill
Tool
MCP / A2A
Sandbox
Multi-Agent
Scheduler
Tracing
```

路线 A（Morn 自研 Native Agent Platform）仍然可行，但其意义是**掌握架构和参考实现**，而不是靠“Runtime 功能比别人更多”取胜。

## 2.2 自进化 Skill、Profile、多 Agent 协作也在快速商品化

Hermes 已经覆盖长期记忆、程序性 Skill、多个隔离 Profile、Profile 克隆和持久协作；研究界也在推进 Skill 生命周期、自演化与可复用子 Agent 资产。[R04][R05]

所以“个人助手会成长为专业 Agent”本身不是足够差异化。Morn 必须更进一步，把成长转化成：

```text
经验
→ 候选 Skill
→ 测试
→ 版本
→ 认证
→ Role Contract
→ 组织身份与权限
→ 影子运行
→ 正式上岗
→ Outcome 反馈
```

## 2.3 企业正在从“Agent”走向“数字劳动力/Agentic Application”

ServiceNow Autonomous Workforce 强调 AI specialist 的 scope、authority、governance 与端到端流程；Oracle Fusion Agentic Applications 将专业 Agent 团队、业务对象、Workflow、Tool、Policy、Approval 和 Audit 组合成结果导向应用；SAP Joule Studio 开始从意图生成 PRD、技术规格、代码骨架和测试产物。[R06][R07][R08]

这意味着 Morn 的竞争对象未来不是某个 Agent，而是：

> **AI-native Work System / Agentic Application / Autonomous Work**。

## 2.4 企业上下文正在从 RAG 走向 Ontology、Process Twin 和 Operational Context

Palantir Ontology 把企业对象、关系、Action、Function 与 Security 作为运营层；Microsoft Fabric IQ 建立共享企业语义/本体；Celonis Context Model 则强调从真实流程和业务知识形成动态运营上下文；TrustGraph 等开源项目把 Ontology、Graph、Embedding、Provenance 和 Retrieval Policy 变成可版本化 Context Core。[R09][R10][R11][R12]

Morn 必须区分：

```text
Knowledge / Context：这个领域知道什么
Operational World：这个项目当前是什么状态、发生了什么、允许改变什么
Process Intelligence：现实工作实际上是怎样运行的
```

## 2.5 数字工厂与虚拟工厂揭示“执行前验证”的重要性

Siemens Eigen Engineering Agent 直接进入 TIA Portal 真实工程项目，理解工程对象、生成/修改工程内容并校验；Siemens Digital Twin Composer 与 NVIDIA Omniverse 展示了在真实产线执行前进行虚拟设计、仿真和验证的路线。[R13][R14][R15]

这对 Morn 的直接启发是：

> **专业工作不能只“生成”，必须有领域验证栈、模拟/历史回放和最终授权。**

## 2.6 Quick BI AIPro 揭示“数据/决策底座”不能被当成一个 Tool

用户提供的 Quick BI AIPro 截图显示其从数据接入到分析交互按 AI-native 重构，强调智能匹配数据源、企业语义理解、MCP、组织化复用、数据溯源，并从个人敏捷跃迁到组织协同。[U01]

因此，Morn 需要显式容纳 Analytics/Decision Intelligence Plane，而不是把 Quick BI、Power BI 或 Jupyter 都压缩成“一个工具调用”。


## 2.7 DeepSeek Harness / Cordis 使 Harness 从“适配器”升级为可组合 Runtime Substrate

DeepSeek 官方 DeepSeek Harness 将“Everything is a Plugin”作为核心设计，并明确由 Cordis 驱动；其模型适配器、Tool Registry、Session Log、Agent Loop 等都可通过配置替换。官方架构还定义了 scope、持久 Session Event、Capability Seam，以及“model-visible implies recorded”的运行时不变量。[R51][R52]

这对 Morn 的意义不是“把 Morn 改写成 DSH”，而是：

```text
Morn Semantic Kernel
        ↓ controls
Morn Harness & Capability Fabric
        ↓
DeepSeek Harness / Codex / Hermes / ...
```

其中 DSH 是重点 Provider，而不是 World/Work/Organization 的事实所有者。

Cordis 论文将 temporal composability 定义为组件移除时可完整撤销其运行时副作用，将 spatial composability 定义为组件依赖可声明并随 Context 变化进行反应式管理，并以 revertible effects / reactive coeffects 实现。[R53][R54]

Morn 吸收其思想，但不会把现实业务动作误归为可逆插件副作用。邮件发送、真实付款、实验、机器人动作、法律签署等需要补偿、审批或事前阻止，而非简单 disposer。

## 2.8 产业 AI 正从“数字化”走向“可适应执行”

Siemens 2026 年路线把 Industrial AI Operating System、Digital Twin Composer、Eigen Engineering Agent 与 Intelligence Center X 并行推进：一边建立数字孪生/虚拟验证世界，一边把工业数据、流程、人和 AI Agent 组合成受治理的 hybrid workforce，并让行业 Agent 从建议转向端到端执行与验证。[R13][R14][R55][R56]

这进一步确认 Morn 的行业路线应是：

```text
Connect existing systems
→ build Operational World
→ model real Work
→ organize mixed Workcells
→ simulate / verify
→ replace specific Work
→ absorb selected software functions
→ evolve organization structure
```

而不是“先造一个万能工业 Agent”。

---

# 3. Morn 的最终战略定位

## 3.1 产品定位

### 对个人

长期个人助手、个人项目 Workspace、可迁移 Actor、个人数字团队、个人工作系统。

### 对专业人士

将专业方法、Skill、Artifact、Decision 和 Workflow 固化为可重复工作能力。

### 对开发者

开发 Actor、Skill、Tool、Connector、Runtime Adapter、Evaluation Pack、Domain Pack 和 Solution。

### 对实施者/咨询机构

把客户模糊需求编译成 WorkPackage、Workcell、系统集成、验证和部署包。

### 对企业

人类与数字劳动力统一组织、工作、权限、预算、责任、交付和 Outcome 管理。

### 对行业生态

建设 BioLab、Pharma、Manufacturing 等 Dream Factory。

## 3.2 一句话产品公式

```text
Morn = Stable Semantic Kernel
     + Operational World / Work Graph
     + Mixed Workforce / Workcell
     + Governed Harness & Capability Fabric
     + Artifact / Decision / Action / Outcome
     + Simulation / Verification
     + Evolution Engine
     + Solution Foundry
     + Industry Dream Factory
```

## 3.3 核心商品的变化

过去：

```text
Bot / Agent
```

现在：

```text
Certified Work Capability
```

例如 BioLab 不应主卖“单细胞分析 Agent”，而应主卖：

> **Dataset → Reviewed Scientific Claim** 的认证工作能力。

它内部包含：

```text
WorkPackage Schema
+ Workcell Blueprint
+ Role/Harness
+ Skills/Pipelines/Connectors
+ Artifact Schemas
+ Acceptance Criteria
+ Verification Suite
+ Historical Cases
+ Deployment Profile
```

---

# 4. 设计原则与不可破坏约束

## 4.1 Work-first

先定义目标、工作边界、输入输出和验收，再决定用不用 Agent。

## 4.2 Canonical State 不由 LLM 直接改写

LLM/Actor 只能生成 Proposal：

```text
Proposal
→ Schema Validation
→ Semantic/Domain Validation
→ Policy
→ Approval/Simulation
→ Commit
→ Verify
```

## 4.3 Actor 与 Runtime 分离

Actor 的身份、Workspace、Artifact、Role、评测和生命周期属于 Morn；Runtime 可替换。

## 4.4 Template 与 Instance 严格分离

```text
ActorTemplate / ActorInstance
RoleBlueprint / RoleSlot
TeamBlueprint / TeamInstance
WorkPackageTemplate / WorkPackage
WorkcellBlueprint / Workcell
SolutionTemplate / SolutionInstance
DomainPack / DomainInstallation
```

## 4.5 Chat 不是正式事实系统

只要输出将：

- 被另一岗位正式使用；
- 影响状态或决策；
- 需要审查、版本、复现或追责；

就必须形成 Artifact/Decision/Action/Outcome 对象。

## 4.6 Mixed Organization 是一等模型

不能把真人、程序和设备伪装成 Agent。

## 4.7 所有副作用走 Action Gateway

Runtime、Skill 或 Tool 不能绕过身份、Policy、审批、凭证和 Ledger。

## 4.8 LLM 是候选规划器，不是神秘总裁

组织设计来源必须可解释：用户事实 + Domain Pack + 已验证模板 + 显式规则 + Registry + LLM 建议 + Evaluation + Human Approval。

## 4.9 评测先于自治升级

系统从 L0 到 L5 自治必须有可量化升级门槛；不能因 Demo 成功直接放权。

## 4.10 客户私有数据不回流公共资产

只允许在授权/脱敏后回流通用失败模式、评测结构、Skill/Workflow 改进，不默认回流客户原始数据与私有知识。


## 4.11 Stable Semantic Kernel，Dynamic Capability Fabric

正式业务原语必须稳定：

```text
Identity / Workspace
Operational Object / State / Event / Action
WorkPackage / Acceptance / OutcomeContract
Artifact / Decision / Outcome
Role / Delegation / Authority / Accountability
Ledger / Provenance
```

Model、Agent Loop、Tool、Memory Backend、Sandbox、Analytics、Simulation、Runtime 可以替换。插件可扩展能力，但不能重新定义上述语义。

## 4.12 Governed Composability

“技术上可以组合”不是上线条件。一个 Extension/Provider 只有同时满足以下约束才可挂载：

```text
Dependency compatible
+ Context/coeffect satisfied
+ Effect class acceptable
+ Authority sufficient
+ Policy allowed
+ Data-boundary allowed
+ Evaluation passed
+ Provenance known
```

任何一项失败都必须拒绝或降级。

## 4.13 Production 与 Evolution 分离

生产中的 Certified Capability、Policy、Harness、Workflow 不允许被 Actor 任意自修改。所有演化采用：

```text
Branch
→ Candidate
→ Replay / Simulation
→ Regression / Safety Evaluation
→ Shadow
→ Approval / Certification
→ Promotion
→ Rollback Point
```

## 4.14 最小充分智能原则

Morn 优化的是完成 Outcome 所需的**最低智能复杂度**，不是 Agent 数量：

```text
Rule / Program / Solver > LLM/Agent
```

如果确定性能力能够以更低成本、更高可靠性完成同一 Work，应优先确定性路径；成熟 Agent Skill 也可以被蒸馏为 Rule/Program，只把异常尾部留给 Actor。

---

# 5. 全局架构

```text
┌──────────────────────────────────────────────────────────────┐
│ 10. Dream Factory & Ecosystem                                │
│ Domain Packs / Certified Work / Work-as-a-Service / Market   │
├──────────────────────────────────────────────────────────────┤
│ 9. Solution & Organization Foundry                           │
│ ProblemSpec / Org Compiler / WorkContract / Deployment       │
├──────────────────────────────────────────────────────────────┤
│ 8. Simulation & Assurance                                    │
│ Replay / Simulation / Eval / Shadow / Certification          │
├──────────────────────────────────────────────────────────────┤
│ 7. Organization & Workforce Runtime                          │
│ Human / Actor / Worker / Service / Device / Workcell         │
├──────────────────────────────────────────────────────────────┤
│ 6. Work & Evidence Plane                                     │
│ Mission / WorkPackage / Artifact / Decision / Outcome        │
├──────────────────────────────────────────────────────────────┤
│ 5. Operational World Plane                                   │
│ Object / Relation / Event / State / Action / Goal / Metric   │
├──────────────────────────────────────────────────────────────┤
│ 4. Agent / Capability / Harness Platform                     │
│ Model / Memory / Skill / Tool / Harness / Runtime / Workflow │
├──────────────────────────────────────────────────────────────┤
│ 3. Morn Kernel                                               │
│ Identity / Workspace / Policy / Ledger / Lifecycle           │
├──────────────────────────────────────────────────────────────┤
│ 2. Adapter & Integration Fabric                              │
│ MCP/A2A/API/CLI/Daemon + BI/ERP/ELN/LIMS/Simulators          │
├──────────────────────────────────────────────────────────────┤
│ 1. Infrastructure & External Foundations                     │
│ Compute / Data / Model / Durable Runtime / Cloud / Device    │
└──────────────────────────────────────────────────────────────┘
```

Workbench、Studio、Console、Hub 是横跨这些平面的**产品表面**，而不是四套独立后端。

Harness 不是第 11 层，而是 Agent/Capability 平面的正式执行资产；数字分身不是新产品层，而是 Actor 的一种来源和代表关系；AI BaaS 是 Dream Factory 的交付模式。v10.2 的 Evolution Engine 同样**不是第 11 层**，而是一条横跨 World、Work、Assurance、Foundry 和 Dream Factory 的受治理闭环：

```text
World/Work/Execution/Outcome Trace
        ↓
Evolution Candidate
        ↓
Simulation / Replay / Evaluation
        ↓
Shadow / Certification
        ↓
Promoted Capability / Workflow / Role / Native Module
        ↓
New Production Version
```

因此 Morn 保持稳定的分层架构，同时获得跨层进化能力。

# 6. “底座地图”：Morn 下面到底站着什么

Morn 不应试图自建所有底座。完整 AI-native 组织通常会依赖至少以下能力域：

| 能力域 | 解决的问题 | 代表参考 | Morn 策略 |
|---|---|---|---|
| Compute / AI Factory | GPU、Token、推理吞吐、缓存、调度 | NVIDIA、华为 Agentic Infra | 外接；Morn 只管预算/路由 |
| Model / MLOps | 模型训练、实验、Registry、部署 | MLflow/云平台 | 外接 |
| Data Platform | 湖仓、数据库、对象存储、实时数据 | 云数据平台 | 外接 |
| Metadata / Data Governance | 目录、所有权、质量、血缘 | DataHub/OpenLineage | 适配并吸收语义 |
| Analytics / BI | 指标、分析、洞察 | Quick BI AIPro、Power BI | Provider 模式 |
| Decision Intelligence | 归因、预测、建议 | BI/DI 平台 | 作为 Decision 输入 |
| Knowledge / RAG | 外部知识与检索 | RAG/GraphRAG | Capability Provider |
| Context Engineering | 可版本化上下文与检索策略 | TrustGraph | 适配 Context Core |
| Semantic / Ontology | 企业统一语义与实体关系 | Fabric IQ、Palantir | 参考/适配 |
| Process Intelligence | 企业真实流程与瓶颈 | Celonis | 外接/事件回流 |
| Digital / Virtual Twin | 对象/工厂/物理世界模拟 | Siemens/NVIDIA/Dassault | Simulator Provider |
| Operational World | 当前业务对象、状态、动作 | Palantir/UOSE 类 | Morn 必须有统一标准 |
| Agent Runtime | Agent Loop、Tool、Memory | openJiuwen/OpenFang/Hermes | Native + Adapter |
| Durable Execution | 长程、恢复、Signal、Timer | Temporal/Restate/DBOS | 外接或适配 |
| Automation / RPA | 确定性执行 | RPA/脚本/工作流 | Worker 类型 |
| MCP/A2A/Integration | 能力和 Agent 互联 | MCP/A2A | 标准适配 |
| Identity / Agent SoR | 数字主体身份与生命周期 | Microsoft/Workday | Morn 自有上层语义 |
| Governance / Control Tower | 发现、权限、Policy、Kill Switch | Microsoft/ServiceNow | Console 融合 |
| Observability / Evaluation | Trace、SLO、质量、成本 | OpenTelemetry/Agent Eval | 自有 Work Outcome 语义 |
| Workforce / Organization | 谁负责什么、负载、绩效、成本 | Workday/Paperclip | Morn 核心 |
| Professional Workflow | 专业计算/研发/工程执行 | Nextflow/Jupyter/TIA | 外接专业底座 |

Morn 的战略位置是：

```text
专业底座们
   ↓
Morn Adapter Fabric
   ↓
Morn Work & Organization Control Plane
   ↓
WorkPackage / Workcell / Verified Delivery
   ↓
Dream Factory / Solution
```

---

# 7. 六个核心 Record：Morn 的数据与责任骨架

v10.0 用三个 System of Record 描述 WHO / WHAT WORK / WHAT WORLD。v10.2 保留这三个 SoR 的兼容语义，但在工程上拆成六个可独立版本、查询和审计的 Record，使责任链更清楚。

```text
World Record
   ↓
Work Record
   ↓
Decision Record
   ↓
Workforce Record
   ↓
Execution Record
   ↓
Outcome Record
   ↓
World Record（新状态）
```

## 7.1 World Record

回答 **WHERE / WHAT WORLD**：真实世界或项目中有什么、当前处于什么状态。

```text
Object
Relation
State
Event
Goal
Metric
Allowed Action
```

它对应原 `Operational World System of Record` 的主体。

## 7.2 Workforce Record

回答 **WHO**：谁存在、谁属于谁、谁会什么、谁被允许做什么。

```text
Principal
Human
Actor
DeterministicWorker
ExternalService
Device
Identity
Role
Capability
Authority
Delegation
Manager
Sponsor
ActorOrigin
RepresentationContract
Performance
Cost
Lifecycle
```

它不仅管理 Agent，也管理真人、服务和设备。

## 7.3 Work Record

回答 **WHAT WORK**：正在做什么、交付什么、谁承诺、是否完成。

```text
Mission
WorkPackage
WorkContract
AcceptanceSpec
OutcomeContract
Workcell
Assignment
Commitment
Deadline
DeliveryState
```

## 7.4 Decision Record

回答 **WHY**：为什么选择这个方案、基于什么证据、谁批准。

```text
ContextSnapshot
CandidateOptions
Evidence
Assumptions
SelectedOption
Uncertainty
PolicyDecision
Approval
AuthorityChain
```

## 7.5 Execution Record

回答 **HOW / WHAT ACTUALLY RAN**：到底由什么 Harness、Runtime、Tool、Workflow 执行。

```text
HarnessSpec/Version
Runtime/Version
ModelPolicy
SkillVersion
Tool/ConnectorVersion
WorkflowRun
Checkpoint
ExecutionReceipt
RecoveryRecord
Token/Cost/Latency
```

Harness 版本必须独立于 Actor 版本，否则失败时无法区分“模型错、Harness 错、Context 错、Skill 错还是 Tool 错”。

## 7.6 Outcome Record

回答 **DID IT WORK**：是否达到验收、业务/科研指标是否改善、世界状态到底改变了什么。

```text
Artifact
VerificationReport
StateDiff
MetricDelta
ActualOutcome
AcceptanceDecision
Customer/Human Feedback
```

## 7.7 六个 Record 与三个 SoR 的兼容映射

```text
Operational World SoR = World Record + World-facing Outcome
Workforce SoR         = Workforce Record
Work SoR              = Work + Decision + Execution + Delivery Outcome
```

Artifact / Provenance / Ledger 贯穿六个 Record，并保证同一次工作可以回答：

```text
谁 → 为什么 → 做了什么 → 通过什么执行 → 交付了什么 → 是否有效
```

# 8. Morn Kernel

Kernel 必须小、稳定、低耦合，不知道 BioLab、金融或制造的细节。

核心职责：

```text
Identity
Workspace
Event Envelope
Policy
Permission
Approval
Secrets
Ledger
Lifecycle
Package/Version
Canonical IDs
Transaction Boundary
```

## 8.1 Identity

统一管理：Human、Actor、Service、External Agent、Device、Organization。

身份模型至少包含：

```yaml
identity:
  id:
  principal_type:
  owner:
  sponsor:
  manager:
  organization:
  status:
  assurance_level:
  credentials_ref:
  created_at:
  expires_at:
```

Owner、Sponsor、Manager 必须分开：技术配置责任、业务生命周期责任、组织管理责任不是同一回事。Microsoft 的 Agent Identity/Blueprint/Sponsor 设计可作为参考。[R17]

## 8.2 Workspace

Workspace 是数据、成员、Policy、Artifact、Memory 和 Secret 的隔离边界。

类型：

```text
PersonalWorkspace
ProjectWorkspace
OrganizationWorkspace
CustomerWorkspace
LabWorkspace
DepartmentWorkspace
SandboxWorkspace
```

跨 Workspace 数据传播默认禁止；必须通过明确的 Export/Import/Share Contract。

## 8.3 Ledger

Ledger 记录不可随意改写的：

- 关键 Proposal；
- Policy Decision；
- Approval；
- Tool/Action 调用；
- State Commit；
- Artifact Release；
- Delegation；
- Evaluation；
- Deployment；
- Rollback。

Ledger 不等于把模型私有推理全文永久保存。应保存**可审计的决策摘要、输入输出 Hash、版本和证据链**。

---

# 9. Operational World Plane

## 9.1 为什么必须有这一层

Artifact 能回答“正式产出了什么”，但无法独立回答：

- 这个实验现在是否已经运行？
- 这个样本在哪个批次？
- 这个客户现在处于哪个阶段？
- 这个设备是不是停机？
- 这个 WorkPackage 执行后真实状态是否改变？

Operational World 是 Morn 中可执行的**组织/项目状态模型**。

## 9.2 两级模型

### Domain Model

定义某行业有什么类型、关系、状态、动作。

### Organization/Project World Instance

是某个真实客户/项目的实例。

```text
BioLab Domain Model
        ↓ instantiate
Aging Lab World / Project 001
```

## 9.3 核心对象

```yaml
object_type:
  id: biolab.Sample
  properties:
  allowed_states:
  relations:
  actions:
  policies:
```

```yaml
object_instance:
  id: sample-S023
  type: biolab.Sample
  workspace_id:
  state:
    qc_status: warning
    processing_status: sequenced
  version:
```

## 9.4 Event

Event 是已发生事实，不是意图：

```yaml
event:
  type: sequencing_completed
  timestamp:
  related_objects:
    - sample-S023
    - run-R014
  actor:
  evidence:
```

## 9.5 Action

Action 是受治理的状态改变：

```yaml
action_type:
  id: biolab.exclude_sample
  preconditions:
    - sample.qc_status == failed
  effects:
    - sample.analysis_status = excluded
  approval_required:
    - statistical_reviewer
    - human_pi
  reversible: true
  verification:
```

## 9.6 Goal、Metric、Outcome

Operational World 不止有状态，还要表达“为什么行动”。

```yaml
goal:
  statement: 在8周内形成至少一个可重复验证的机制假设
  metrics:
    - reproducibility_rate
    - cycle_time
    - evidence_coverage
```

Outcome 必须与 Task Completed 区分：

```text
Task completed：邮件已发送
Outcome：客户是否回复/是否进入下一销售阶段

Task completed：分析已运行
Outcome：是否形成通过独立复核的科学主张
```

## 9.7 世界模型成熟度

Morn 不应第一天声称具有“真正企业世界模型”。按四级推进：

- **L0 Operational Semantic Model**：对象、关系、状态、事件、Action；
- **L1 Operational Dynamics**：状态机、规则、资源约束、Expected Effects；
- **L2 Predictive Operational Model**：时间/成本/风险/Outcome 预测；
- **L3 Causal/Counterfactual World Model**：因果、干预、反事实与策略优化。

v10.2 近期工程目标以 L0-L1 为主；L2-L3 需要真实 Outcome 数据后再做。Business World Model 研究可作为长期参考，但目前更多是架构研究而非成熟通用产品。[R18]

---

# 10. Context、Knowledge、Memory 与 Operational World 的边界

| 对象 | 核心问题 | 是否可主观 | 是否改变业务状态 |
|---|---|---:|---:|
| Knowledge | 外部世界已知什么 | 否/低 | 否 |
| Context Core | 当前任务该给模型什么上下文 | 否/策略化 | 否 |
| Memory | 某 Actor 从经历中记住什么 | 可以 | 否 |
| Artifact | 团队正式产出了什么 | 不应 | 间接 |
| Operational State | 真实业务/科研对象当前是什么 | 否 | 是 |
| Ledger | 谁何时做了什么 | 否 | 记录 |
| Outcome | Action 产生了什么效果 | 否 | 是 |

TrustGraph 的 Context Core 思路适合 Morn：Ontology、Graph、Embedding、Provenance 和 Retrieval Policy 应是可版本、可部署的上下文资产。[R12]

但 Context Core 不应替代 Operational World。

---

# 11. Artifact-first 升级为 Evidence-first Work System

## 11.1 Message 与 Artifact

Message：讨论、提问、解释、临时协调。  
Artifact：正式交接、审批、版本化、复现和长期保存。

## 11.2 Artifact 基础结构

```yaml
artifact:
  id:
  type:
  schema_version:
  workspace_id:
  content_ref:
  structured_content:
  created_by:
  generated_by_activity:
  source_artifacts:
  version:
  status:
  reviewers:
  approvals:
  confidentiality:
  checksum:
  created_at:
```

## 11.3 生命周期

```text
Draft
→ Submitted
→ In Review
→ Changes Requested
→ Approved
→ Locked
→ Superseded
→ Archived
```

Artifact 不能原地覆盖；修改必须创建新版本并保留 `derived_from`。

## 11.4 Provenance

采用 W3C PROV 的思想：Entity、Activity、Agent 及 wasGeneratedBy、used、wasDerivedFrom、wasAttributedTo 等关系。[R19]

Morn 进一步加入：

```text
ArtifactVersion
DecisionPackage
ExecutionReceipt
VerificationReport
StateDiff
OutcomeRecord
```

## 11.5 Decision Package

```yaml
decision_package:
  id:
  intent:
  decision_subject:
  context_snapshot:
  world_state_version:
  source_artifacts:
  alternatives:
  selected_option:
  reasoning_summary:
  assumptions:
  uncertainty:
  supporting_evidence:
  conflicting_evidence:
  policy_decision:
  authority_chain:
  delegation_chain:
  approvals:
  action_proposal:
  expected_effects:
  expected_outcomes:
  execution_receipt:
  actual_state_diff:
  actual_outcome:
  model_version:
  skill_version:
  tool_version:
  workflow_version:
  signature:
```

XScientist 等科研系统开始把探索 DAG、代码、结果、Claim-Evidence 锚点、Hash、失败分支和重执行 Hook 打包成研究 Artifact，这验证了科研系统需要比“聊天历史”更严格的执行记录。[R20]

---

# 12. WorkPackage：Morn 的核心可委托工作原语

## 12.1 定义

WorkPackage 是一份可以正式委托、执行、验收和追责的工作合同，而不是一句任务描述。

```yaml
work_package:
  id:
  objective:
  world_context:
  inputs:
  required_outputs:
  acceptance_criteria:
  execution_mode:
  allowed_actions:
  prohibited_actions:
  budget:
  deadline:
  approval_gates:
  failure_policy:
  verification_suite:
  accountable_owner:
```

## 12.2 WorkPackage 与 Task 的区别

Task 可以是：

```text
“分析一下这个单细胞数据。”
```

WorkPackage 必须明确：输入、锁定版本、交付物、验收条件、权限、预算、失败策略、责任人和证据。

## 12.3 AcceptanceSpec

AcceptanceSpec 是 WorkPackage 的完成定义：

```yaml
acceptance_spec:
  required_artifacts:
  schema_checks:
  domain_rules:
  reproducibility:
  reviewer_requirements:
  metric_thresholds:
  forbidden_conditions:
  human_approval:
```

## 12.4 Autonomy Maturity

```text
L0 Inform
L1 Assist
L2 Draft
L3 Supervised Execute
L4 Bounded Autonomy
L5 Certified Autonomous Delivery
```

自治等级绑定 WorkPackage，而不是永久绑定某个 Actor。

## 12.5 ExecutionMode：先判断工作性质，再选择执行者

用友 BIP 6 的“智能双模”强化了一个重要原则：规则确定、精确计算的工作应优先确定性执行，需要判断、概率推理和复杂上下文的工作才进入 Agentic 路径。[R45][R46]

Morn 固化为：

```yaml
execution_mode:
  nature:
    - deterministic
    - probabilistic
    - physical
    - regulated
    - social
    - mixed
  executor:
    - program
    - actor
    - human
    - external_service
    - device
    - hybrid
  rationale:
  fallback:
```

Member Type Planner 不再只问“要不要 Agent”，而是先判断工作性质，再决定最合适执行方式。

## 12.6 OutcomeContract：从 WorkPackage 升级为 Work Contract

当 WorkPackage 面向客户、外部团队或托管服务交付时，增加：

```yaml
outcome_contract:
  deliverable:
  quality_slo:
  deadline:
  acceptance_method:
  business_or_scientific_metric:
  target:
  billing_basis:
    - fixed
    - usage
    - milestone
    - outcome
  retry_liability:
  human_fallback:
  escalation:
  evidence_required:
```

这使 Morn 能够支持：

```text
“给我一个 Agent”
→
“把 Dataset → Reviewed Scientific Claim 这项工作按约定标准交付”
```

WorkPackage 是内部执行原语；加上 OutcomeContract 后，它可以成为真正的 `Work Contract` 和 Work-as-a-Service 基础。

# 13. Workcell：围绕工作动态组装执行单元

Team 往往是相对稳定组织；Workcell 是围绕某个 WorkPackage 动态编译出来的执行单元。

```text
Workcell
= RoleSlots
+ ResponsibilityBindings
+ MemberBindings
+ Capabilities
+ Workspace
+ WorkPackage
+ Harness
+ Workflow
+ Evaluation
```

例如单细胞分析 Workcell：

```text
Bioinformatics Actor          → 方法选择/解释
Scanpy/Seurat Pipeline        → 确定性计算
Data Steward                  → 数据/元数据
Statistical Reviewer          → 独立审查
Human PI                      → 高风险批准
```

任务完成后 Workcell 可以销毁，但 Artifact、Decision、Outcome、Provenance 永久保留。

---

# 14. 混合组织模型

## 14.1 五种成员类型

### Human

最终责任、专业判断、伦理/安全审批、不可逆决策与现实执行。

### Actor

开放式语义理解、推理、解释、协调与草稿生成。

### Deterministic Worker

规则检查、脚本、Pipeline、统计计算、格式转换等可重复执行。

### External Agent / Service

Hermes、OpenClaw、SaaS、企业内部 Agent、第三方分析服务等。

### Device / Robot

仪器、机器人、传感器、边缘设备和物理执行系统。

## 14.2 RoleSlot

RoleSlot 定义“组织需要谁承担什么职责”，而不是先创建 Agent。

```yaml
role_slot:
  id:
  responsibilities:
  required_capabilities:
  accepted_member_types:
  inputs:
  outputs:
  authority:
  accountable_to:
  evaluation_suite:
```

## 14.3 ResponsibilityBinding

一个岗位可以由多个成员共同承担不同责任：

```yaml
responsibility_binding:
  work_package: experiment-design-001
  propose:
    actor: experiment-designer
  calculate:
    worker: power-analysis-service
  review:
    actor: statistical-reviewer
  approve:
    human: human-pi
  execute:
    human: lab-technician
```

## 14.4 最小充分团队原则

不默认生成十几个永久 Agent。优先：

```text
稳定核心成员
+
按 WorkPackage 动态实例化 Specialist/Workcell
```

## 14.5 双阶段 Member Type Planner

```text
阶段一：Work Nature Classifier
Deterministic / Probabilistic / Physical / Regulated / Social / Mixed

阶段二：Executor Planner
Program / Actor / Human / Service / Device / Hybrid
```

例如 BioLab：

```text
文献证据综合       → agentic
统计功效计算       → deterministic
实验方案批准       → human
单细胞分析         → hybrid
湿实验             → human/device
```

该模型直接防止“所有任务都包装成 Agent”。

# 15. Delegation、Commitment 与 Accountability

## 15.1 Delegation 不等于 assign(task)

正式委托必须表达：

```yaml
delegation:
  delegator:
  delegatee:
  objective:
  task_scope:
  authority_scope:
  resource_scope:
  time_budget:
  cost_budget:
  token_budget:
  valid_from:
  expires_at:
  can_redelegate:
  required_evidence:
  acceptance_criteria:
  failure_policy:
  escalation:
  retained_accountability:
```

## 15.2 Commitment

Commitment 表示一个成员对结果的承诺，而不只是“当前有一个 Task ID”。

需要：deadline、expected outputs、dependencies、status、blocked_reason、owner、escalation。

## 15.3 Accountability Chain

Morn 必须记录：

```text
谁作出 Judgment
谁 Delegated
谁 Verified
谁 Approved
谁 Executed
谁仍保留最终 Responsibility
谁在何时 Terminated/Revoked 授权
```

---

# 16. Actor 模型

## 16.1 Actor 不是 Prompt

Actor 是具有持续身份、状态、能力、关系和生命周期的软件主体。

```text
Actor
├── Constitution
├── Mind
├── Experience
├── Capabilities
├── Environment
└── Assurance
```

### Constitution

Identity、Owner、Sponsor、Values、Hard Boundaries、Lifecycle。

### Mind

Model Policy、Goal/Commitment、Planner、Belief/World View、Self Model、Uncertainty。

### Experience

Event、Working/Episodic/Semantic/Procedural/User/Self Memory。

### Capabilities

Skill、Tool、Knowledge、Pipeline、Subactor、Harness、Runtime。

### Environment

Workspace、Credentials、Channel、Sensor、Body。

### Assurance

Policy、Evaluation、Provenance、Audit、Version、Rollback。

## 16.2 ActorTemplate / ActorInstance

```yaml
actor_instance:
  id:
  template_id:
  identity_id:
  actor_origin:
  owner:
  sponsor:
  harness_binding:
  runtime_binding:
  model_policy:
  capabilities:
  memory_policy:
  workspace_bindings:
  role_bindings:
  status:
```

## 16.3 Seed

“空白人”继续保留为 Studio 的产品隐喻，但底层不要实现成超级 Seed 对象：

```text
Seed UI Experience
→ ActorTemplate + ActorInstance + Bindings
```

## 16.4 ActorOrigin：数字主体从哪里来

v10.1 正式区分三类来源：

```text
Independent Actor
从零创建，属于某个人或组织。

Role-derived Actor
由岗位模板 + 组织规则 + Workspace 实例化。

Human-delegated Actor
来自真实人的受控数字代理/数字分身。
```

不同来源决定 Memory、权限、退出、交接和责任模型，不能只靠 `persona = CEO` 区分。

## 16.5 RepresentationContract：数字分身代表谁、能代表到什么程度

对 Human-delegated Actor 必须显式建模：

```yaml
representation_contract:
  principal: human-ceo-001
  delegate_actor: ceo-twin-001
  representation_scope:
    - strategy_review
    - proposal_feedback
  cannot_represent:
    - board_vote
    - legal_signature
    - hiring_final_decision
  authority_level: advisory
  disclosure:
    must_identify_as_ai: true
  valid_until:
  revoke_anytime: true
  accountable_principal:
```

数字分身的任何输出都必须能区分：

```text
AI自己的建议
历史偏好的模拟
经授权的代表性意见
具有正式组织效力的决定
```

## 16.6 DecisionPolicyAsset：复制判断框架，不是复制人格

企业希望沉淀关键人的能力时，优先拆成：

```text
Decision Criteria
Priorities
Trade-off Rules
Risk Preference
Review Rubric
Escalation Threshold
Known Exceptions
Historical Decisions
```

另行保存 Communication Profile 与 Expertise Profile。这样“专家数字分身”更像可治理的决策资产，而不是模仿语气的人格 Prompt。

# 17. Memory

Memory 不是向量数据库的别名。

建议类型：

```text
Working
Episodic
Semantic
Procedural
Preference/Relationship
Self
```

Memory Write Gate：

```text
Candidate Memory
→ Source Check
→ Fact/Hypothesis Classification
→ Sensitivity
→ Ownership Scope
→ Workspace Scope
→ Conflict Detection
→ Retention/Expiry
→ Approval Requirement
→ Persist
```

Runtime 短期上下文属于 Runtime/Harness；Actor 长期记忆属于 Morn；团队正式事实属于 Artifact/Operational World；不可变动作史属于 Ledger。

## 17.1 Memory Ownership

v10.1 增加明确所有权：

```text
Personal Memory
个人拥有，组织不得默认继承。

Role Memory
岗位拥有，可按规则交接给下一任。

Workspace Memory
项目/团队拥有，成员离开后仍属于 Workspace。

Institutional Memory
组织审核后的长期经验和规则。

Twin Private Memory
真人数字分身主人专属，不得默认转移。

Shared Experience
经脱敏、验证和批准后可转成组织 Skill/Evaluation。
```

## 17.2 调岗/离职/数字分身撤销

正确交接不是“复制全部记忆”，而是：

```text
Personal Memory              → 保留/删除依个人与合规规则
Role Knowledge               → 可交接
Validated Skill              → 可交接
Approved Artifact            → 属于 Workspace/组织
Institutional DecisionRecord → 属于组织
Private Relationship Memory  → 不默认交接
Twin Private Memory          → 跟随主体撤销或归档策略
```

任何 Memory 提升为组织资产都必须经过来源审查、隐私检查和验证门。

# 18. Skill：可验证程序性能力

Skill 需要从 Prompt 升级为合同：

```yaml
skill:
  id:
  version:
  purpose:
  scope:
  applicable_when:
  prohibited_when:
  inputs:
  outputs:
  preconditions:
  procedure:
  dependencies:
  environment_assumptions:
  failure_modes:
  recovery:
  risk_level:
  approvals:
  provenance:
  tests:
  metrics:
  lifecycle:
```

生命周期：

```text
Draft → Experimental → Tested → Verified → Certified → Deprecated → Retired
```

经验不能直接覆盖生产 Skill；必须通过 Experience Compiler 和回归测试。

---

# 19. Tool、Connector 与 Action Contract

## 19.1 Tool ≠ Connector

Tool 表示语义动作，Connector 绑定具体外部系统。

```text
get_batch_record
├── SAP Connector
├── Customer MES Connector
└── Veeva Connector
```

## 19.2 Action-first Gateway

高风险动作统一：

```text
Discover/Search
→ Resolve
→ Preview
→ Authorize
→ Execute
→ Verify
→ Recover
```

## 19.3 Action Contract

```yaml
action_contract:
  id:
  target_type:
  discover:
  resolve_target:
  preview_effect:
  execute:
  verify_effect:
  recover:
  reads:
  writes:
  side_effects:
  preconditions:
  expected_effects:
  possible_failures:
  idempotency:
  reversibility:
  approval:
  evidence_output:
```

MCP/A2A 负责连接和互操作，不负责企业级责任、权限和状态提交；Morn 负责上层语义。


## 19.4 Effect Classification：软件可逆不等于现实可逆

Cordis 的可逆 Effect 很适合描述注册 Provider、Listener、Tool、Prompt Fragment、临时 Runtime Resource 等生命周期副作用；Morn 必须把真实业务副作用进一步分级：[R52][R54]

```text
E0 Lifecycle Reversible
  register/unregister、mount/dispose、临时 Context

E1 Transaction Reversible
  尚未提交的数据库/事务状态，可 rollback

E2 Compensatable
  创建订单→取消订单、创建会议→取消会议；不是数学逆操作，而是补偿动作

E3 Irreversible
  外部邮件、付款、物理实验、机器人动作、公开发布、法律签署
```

ActionContract 新增：

```yaml
effect_policy:
  class: lifecycle_reversible | transactional | compensatable | irreversible
  compensation_action:
  irreversible_reason:
  preview_required: true
  approval_required:
  verification_required: true
```

E3 默认不得依赖“事后恢复”，必须依靠最小权限、Preview、Simulation、Approval 与 Verify。

---

# 20. Agent / Capability / Harness Platform

## 20.1 路线 A：可以自研，但要正确理解

Morn 可以拥有自己的 Native Platform，但不应把“自研所有 Agent 能力”作为差异化目标。底层目标是：

```text
统一的主体接口
+ 统一 Harness 接口
+ 统一 Action/Policy Gateway
+ 统一 Trace/State 接口
```

而不是复刻每个 CLI/框架的内部实现。

## 20.2 Morn Native Runtime 最小能力

```text
Model Invocation
Controlled Loop
Context Assembly
Structured Proposal
Tool/Action Proposal
Checkpoint
Basic Memory Adapter
Workflow Node
Trace
Cancel / Pause / Resume
```

## 20.3 Harness 与 Runtime 的边界

```text
Actor   = 谁
Role    = 负责什么
Harness = 怎样工作（Context、Tool、Orchestration、State、Validation、Recovery）
Runtime = 在哪里/用什么执行（CLI、API、Daemon、Container、Remote）
```

同一 Actor 可以保持身份、Role、Memory、Skill 和权限不变，只切换 Harness 或 Runtime。AgentSpace 已经公开展示了“同一数字员工跨 Claude Code、Codex、OpenClaw、Hermes 等 Harness 切换”的工程模式，说明该能力会快速成为基础能力。[R42]

## 20.4 一等 Harness Provider 类型

```text
CLI Harness
Daemon Harness
API Harness
Container Harness
Remote Harness
Native Harness
```

首批建议：

```text
CodexCLIHarness
ClaudeCodeHarness
OpenClawHarness
HermesHarness
OpenJiuwenHarness
LangGraphHarness
MornNativeHarness
```

Golutra、Tutti 等项目证明“保留用户已有 CLI，在其上建立控制层”是现实且高价值的产品路径。[R41][R44]

## 20.5 Runtime/Harness 不得拥有 Morn 的正式资产

以下必须留在 Morn：

- Identity；
- Workspace；
- WorkPackage/WorkContract；
- Canonical World State；
- Artifact/Decision/Outcome；
- Role/Delegation/Authority；
- Evaluation/Certification；
- Solution/Dream Factory 生命周期。

外部 Harness 只得到最小、临时、可审计的工作上下文和能力授权。


## 20.6 DeepSeek Harness：首个重点默认 Harness Provider

DeepSeek 官方仓库将 DSH 定义为 open-source agent harness，核心架构为“Everything is a Plugin”，由 Cordis 驱动；当前仍处于 Developer Preview，并明确提示会发生 breaking changes。[R51]

Morn 因此采用 **B → C** 路线：

```text
现在（B）：
Morn Kernel 独立
→ Morn Harness Provider API
→ DeepSeek Harness Provider

以后验证（C）：
Morn Kernel 独立
→ Morn Capability Fabric
→ Cordis（若 API/生态足够稳定）
→ DSH / other Providers
```

不采用：

```text
fork DeepSeek Harness
→ 把 Morn World/Work/Organization 全塞进 DSH
```

首个桥接包建议：

```text
morn-dsh-bridge
├── morn-context-provider
├── morn-artifact-provider
├── morn-evidence-provider
├── morn-action-gateway-provider
├── morn-attention-provider
└── event-normalizer
```

DSH 可以创建/运行 Agent Session，但不能直接修改 Morn Canonical World。

## 20.7 Capability Seam

吸收 DSH 的 seam 结构：[R52]

```text
Capability Seam
├── Service Definition
├── Service Provider
└── Consumer
```

例如：

```text
AnalyticsCapability
├── QuickBIProvider
├── PythonAnalyticsProvider
└── BusinessAnalysisWorkcell (consumer)
```

```text
AgentExecution
├── DeepSeekHarnessProvider
├── CodexCLIProvider
├── HermesProvider
└── Actor HarnessBinding (consumer)
```

Provider 变化不应迫使上层 WorkContract、Actor、Artifact、Decision 或 Outcome 迁移。

## 20.8 Scope Tree 与运行时不变量

DSH 已提供 global/agent 等作用域化能力，Morn 将其扩展为业务作用域树：[R52]

```text
Organization
└── Workspace
    └── Solution / Workcell
        └── Actor
            └── Execution Run
```

下级 Scope 可以 add / override / restrict / isolate，但不得绕过上级安全 Policy。

Morn 冻结以下执行不变量：

```text
Model-visible information      → Recorded
Decision-affecting context     → Provenanced
Action-authorizing context     → Recorded
Canonical-state mutation       → Receipted
Artifact dependency            → Lineaged
```

Prompt/Session 是执行视图，不是唯一正式事实系统。

## 20.9 事件域必须分离

参考 DSH 对 Session/Agent/Capability Event 的区分，Morn 明确：

```text
WorldEvent      = 世界正式事实
WorkEvent       = 工作正式事实
ExecutionEvent  = Harness/Tool/Workflow 运行事实
LiveEvent       = UI/临时协调，可不长期保留
CapabilityEvent = Provider/Extension 生命周期与策略事件
```

底层可以共享 Event Store，但语义和保留策略不能混为一个 `events` 表。

# 21. Context Builder

一次模型调用的上下文不应等于“所有文件+所有聊天”。

```text
Goal / WorkPackage
+ Current Step
+ Operational World Slice
+ Relevant Artifacts
+ Approved Knowledge
+ Scoped Memory
+ Role/Harness
+ Allowed Actions
+ Policy
+ Budget
→ Runtime Context
```

Context Builder 必须记录：为什么选中某个上下文、版本是什么、是否来自不可信外部内容。

---

# 22. Analytics & Decision Intelligence Plane

Quick BI AIPro 的意义不是“可以接一个 BI Tool”，而是企业数据分析本身是一层长期基础能力。[U01][R23]

Morn 需要定义通用 Analytics Provider 接口：

```text
DataAsset
Metric
SemanticConcept
AnalysisSkill
AnalysisPlan
AnalysisRun
Insight
Recommendation
AnalyticsArtifact
Lineage
```

正确链路：

```text
Data
→ AnalysisRun
→ Insight
→ Recommendation
→ DecisionPackage
→ ActionProposal
→ Approval
→ Action
→ Outcome
```

Quick BI、Power BI、Fabric、Jupyter/R 可作为 Provider；Morn 不重做 BI 引擎。

---

# 23. Semantic / Ontology / Context Layer

## 23.1 三类语义体系

### BI Semantic Layer

指标、维度、业务术语、分析逻辑。

### Enterprise Ontology

对象类型、关系、业务语义，代表参考 Fabric IQ、TrustGraph。[R10][R12]

### Operational Ontology

在语义之上增加 Action、Function、Security、State Change，Palantir 是重要参考。[R09]

Morn 的 Domain Pack 需要兼容三者，但重点是将语义映射成 Work、Action 与 Outcome。

---

# 24. Process Intelligence 与 Object-Centric Event

Domain Pack 只能描述“应该怎样工作”；真实系统必须知道“实际怎样工作”。

```text
Designed Process
        ↕
Object-Centric Event Log
        ↓
Process Mining
        ↓
Deviation / Bottleneck / Rework / Shadow Path
        ↓
Workflow/Evaluation Improvement
```

Celonis Context Model 的价值在于把真实流程数据、业务知识和结果形成持续运营上下文，并用于人、Agent 和系统的协作。[R11]

Morn Event 建议支持同时关联多个对象：

```yaml
event:
  event_type: analysis_completed
  timestamp:
  related_objects:
    - Experiment: EXP-018
    - Dataset: DS-029
    - AnalysisRun: AR-103
  principal:
  state_before:
  state_after:
  evidence:
```

---

# 25. Digital Twin、Virtual Factory 与 Simulation

## 25.1 四类 Twin

### Asset Twin

设备、产品、细胞、机器等实体的数字孪生。

### Process Twin

工作/业务/科研过程实际上如何运行。

### Organization Twin

组织结构、Role、Authority、责任与当前项目关系。

### Workforce Twin

人、Actor、Worker、Service、Device 的能力、负载、成本、绩效和状态。

四者共同为 Operational World 提供高质量状态。

## 25.2 Digital Thread

Twin 回答“现在是什么样”；Digital Thread 回答“怎样一路变成现在”。

Morn 的 Artifact + Decision + Action + Outcome + Provenance 就是一条通用智能工作 Digital Thread。

## 25.3 Simulation Provider

Morn 不自建所有模拟器，而定义：

```text
SimulationAdapter
Scenario
WorldSnapshot
ActionSequence
ExpectedState
ObservedState
VerificationResult
```

制造可接 Siemens/NVIDIA；BioLab 可用历史实验回放、统计模拟和 Protocol Validator；企业运营可用沙箱业务数据。[R14][R15]


## 25.4 Production World、Shadow World 与 Evolution World

Morn 不在真实生产世界里直接试错。每次重大流程、能力、岗位或软件替换都应从 Production Snapshot 分支：

```text
Production World
      ↓ snapshot
Evolution World A / B / C
      ↓
Replay / Simulation / Counterfactual
      ↓
Shadow World
      ↓
Promotion Gate
      ↓
New Production Version
```

这使“组织结构和工作结构”也具备类似代码分支、测试和发布的治理方式。

## 25.5 Digital Factory、Virtual Factory 与 Evolving Digital Factory

Morn 中三者严格区分：

```text
Digital Factory
= 真实工厂的 Operational World + Work + Workforce + Software/Device + Outcome 同步

Virtual Factory
= Digital Factory Snapshot + Simulation + Counterfactual Scenario

Evolving Digital Factory
= Digital Factory + Virtual Factory + Evolution Engine + Governed Promotion
```

因此 Virtual Factory 是安全试错环境，Digital Factory 是真实运营映射，Evolving Digital Factory 才是长期终态。

---

# 26. Durable Execution：长程工作不能靠聊天会话

原 Task Engine 的 DAG、状态、Retry、Timeout 和 Approval 基础继续保留，但要升级为：

```text
Checkpoint
Pause/Resume
Human Interrupt
Persistent Signal
Expected State
Observed State
State Diff
Drift Detection
Replanning
Compensation
Rollback
Watchdog
Deadline
Budget
Escalation
```

架构策略：

- Desktop/local：在现有 Rust Task Engine 上实现轻量事件溯源、Checkpoint、幂等和恢复；
- Enterprise：通过 Adapter 接 Temporal 或 Restate；
- Python/TS 工作负载可选择 DBOS sidecar；
- Agent 内部图可接 LangGraph。

Temporal/Restate/DBOS 的成熟 Durable Execution 设计应复用，不需要 Morn 重新发明完整分布式调度平台。[R24][R25][R26]

---

# 27. Simulation & Assurance Plane

所有高价值 WorkPackage 应定义领域验证栈：

```text
Schema Validation
→ Static Validation
→ Domain Rule Validation
→ Project Standard Validation
→ Simulation/Historical Replay
→ Regression Evaluation
→ Independent Review
→ Human Approval
→ Shadow/Canary
→ Production Observation
```

评测层级：

```text
Component
Skill
Tool/Action Contract
Actor
Role
WorkPackage
Workcell
Workflow
Solution
Deployment
Online Outcome
```

认证状态：

```text
Draft
Experimental
Verified
Certified
Restricted
Deprecated
Retired
```


## 27.1 Evolution Promotion Gate

任何自动生成的 Skill、Workflow、Harness Patch、Role 重构、Native Module 或组织结构都只能先成为 `EvolutionCandidate`：

```text
Candidate
→ Dependency/Schema Check
→ Historical Replay
→ Regression Evaluation
→ Safety/Policy Evaluation
→ Simulation
→ Shadow
→ Human/Policy Gate（按风险）
→ Certified Version
→ Staged Promotion
→ Outcome Monitoring
→ Commit / Rollback
```

Production World 不允许“边跑边任意改自己”。

---

# 28. Observability、Agent SRE 与 Outcome SRE

Morn 需要三层观测：

### Technical Observability

模型、Token、延迟、Tool、错误、Trace、资源。

### Work Observability

WorkPackage 状态、Blocked、Acceptance、人工返工、恢复、交付。

### Outcome Observability

真实业务/科研结果、价值指标、失败模式、策略效果。

Console 不应只问“Agent 有没有报错”，还要问：

> **这个工作是否真的被正确交付？结果是否改变了目标指标？**

---

# 29. Solution & Organization Foundry

## 29.1 不是一个万能 Planner Agent

Foundry 是可审计编译管线：

```text
User Goal
→ Solution Intent Parser
→ ProblemSpec Builder
→ Domain Pack Resolver
→ Operational World Bootstrap
→ Work Decomposer
→ WorkPackage Compiler
→ Acceptance Compiler
→ RoleSlot Generator
→ Member Type Planner
→ Capability Resolver
→ Harness / Delegation Compiler
→ Artifact Contract Generator
→ Workflow Compiler
→ Policy Compiler
→ Evaluation Generator
→ Simulation Plan
→ Deployment Compiler
→ Proposed Solution
→ Human Approval
```

## 29.2 谁作出“建立生物学实验室团队”的判断

| 判断 | 主要来源 |
|---|---|
| 用户到底要什么 | 用户事实 + Intent Parser |
| 生物实验室有哪些对象/约束 | BioLab Domain Pack |
| 可复用哪类团队/工作产品 | Scenario/Certified Work Templates |
| 工作怎样拆解 | Work Decomposer + LLM 候选 |
| 需要什么岗位 | WorkPackage/Team Compiler |
| 用 Agent、人、程序还是设备 | Member Type Planner + Policy |
| 有哪些可用能力 | Capability Resolver / Registry |
| 是否可部署 | Simulation/Evaluation + Human Approval |

任何模型建议都应产生 Decision Record：

```yaml
decision:
  subject:
  proposed_value:
  reasons:
  based_on:
  generated_by:
  confidence:
  validation:
  status:
  approved_by:
```

## 29.3 Capability Resolver

输出 `Reuse / Install / Integrate / Configure / Build / Defer / Reject Matrix`。

这一点非常重要：Dream Factory 不是要求 Morn 自己实现每一个行业能力。

---

# 30. HarnessSpec：数字成员的可版本化执行工程合同

Harness 比 Persona/Prompt 更接近生产级数字员工的执行定义。Golutra、AgentSpace、Tutti 等项目正在把不同 CLI/Runtime 统一到编排层；用友 YonCode 则把 Harness 工程明确拆成信息边界、工具系统、执行编排、记忆与状态、评估与观测、约束校验恢复六层。[R41][R42][R44][R46]

## 30.1 Harness 六层模型

```text
HarnessSpec
├── Context Boundary
├── Capability / Tool Boundary
├── Orchestration
├── Memory & State
├── Evaluation & Observability
└── Constraint / Recovery
```

## 30.2 HarnessSpec

```yaml
harness_spec:
  id: biolab-research-harness
  version: 1.3.0

  runtime_profile:
    preferred: codex-cli
    fallbacks:
      - hermes
      - morn-native

  context_policy:
    workspace_scope:
    world_scope:
    artifact_scope:
    memory_scope:
    max_context:
    retrieval_strategy:
    progressive_disclosure: true

  capability_policy:
    allowed_tools:
    forbidden_tools:
    approval_tools:
    allowed_skills:
    external_connectors:

  orchestration:
    planner:
    max_steps:
    delegation:
    retry:
    timeout:
    parallelism:

  state_policy:
    checkpoints:
    resumable: true
    memory_write_policy:
    canonical_state_write: forbidden

  validation:
    output_schema:
    validators:
    acceptance_hooks:
    reviewer_hooks:

  recovery:
    on_tool_failure:
    on_context_overflow:
    on_policy_violation:
    on_invalid_output:
    fallback_harness:

  observability:
    traces: true
    metrics:
    token_budget:
    cost_budget:
```

## 30.3 HarnessBinding 与 RuntimeBinding 分离

```text
ActorInstance
├── HarnessBinding
│   ├── HarnessSpec ID / Version
│   └── Runtime Profile Policy
└── RuntimeBinding
    ├── Provider
    ├── Endpoint / CLI / Daemon
    └── Session / Environment
```

这样可以做到：

```text
Actor身份不变
Role不变
Memory不变
Workspace不变
权限不变

仅 Harness / Runtime 切换
```

## 30.4 Harness 必须独立版本化

一次正式执行至少要追踪：

```text
ActorTemplate v2.1
RoleContract v3
Harness v1.7
ModelPolicy v5
SkillPack v4
ContextSnapshot v12
Tool/Connector v9
Workflow v6
```

这样失败时才能定位到模型、Harness、Skill、Context、Tool、Policy 或环境的具体版本。

## 30.5 Harness 的自动优化边界

2026 年研究已经开始探索 Harness 评测与自优化，但 Morn 不允许生产 Harness 直接自修改后上线。[R49][R50]

正确路径：

```text
Failure Trace
→ Harness Candidate Patch
→ Sandbox Replay
→ Regression Evaluation
→ Human/Policy Review
→ New Harness Version
→ Shadow Run
→ Release
```

## 30.6 Harness 与 Work Contract 的关系

Role定义职责；WorkContract定义这次要交付什么；Harness定义这次怎样可靠执行。

```text
Role = 长期职责边界
WorkContract = 本次工作边界与验收
Harness = 执行工程边界
Runtime = 实际执行环境
```

四者必须分离。


## 30.7 HarnessSpec 与 Capability Seam 的关系

HarnessSpec 不再列死每一个实现，而优先引用抽象 Capability：

```yaml
harness_spec:
  requires:
    llm: capability.llm
    filesystem: capability.fs
    sandbox: capability.sandbox
    agent_loop: capability.agent-loop
    artifact_io: morn.artifact
    action_gateway: morn.action-gateway
```

实际 Provider 由 Scope、Policy、Cost、Evaluation、Availability 决定。

## 30.8 Coeffect / Effect 声明

每个 Extension/Capability 应显式声明：

```yaml
requires:        # coeffects
  services:
  world_objects:
  data_access:
  authority:
  environment:

provides:
  capabilities:

effects:
  class:
  reads:
  writes:
  external_effects:
  creates_artifacts:
  state_transitions:
```

Morn 的自动组合器必须先检查 `requires` 是否满足，再检查 `effects` 是否被 Authority/Policy 允许，最后才允许 mount/execute。

## 30.9 Cordis 采用边界

Cordis 是值得深入验证的 Meta-Framework，但官方明确其 API 仍处于 active development，论文也是 2026-08-13 的 active-revision preprint。[R53][R54]

因此：

- 当前只做 Architecture Spike；
- 不把 Morn Kernel Schema 绑定 Cordis API；
- 不依赖 Cordis 承担 Canonical State/Identity/Work Ledger；
- 若后续成熟，可让 `morn-harness` 的 Provider Lifecycle / Scope / Effect Tracking 基于 Cordis；
- 保留移除 Cordis 后仍可运行的 Morn Semantic Contract。

# 31. Domain Pack

新版 Domain Pack 不只是“知识库+几个 Agent”。

```text
DomainPack
├── ContextCore
├── Ontology / Semantic Model
├── Operational Object Model
├── Event Model
├── State Machines
├── Action Contracts
├── Artifact Schemas
├── Decision Schemas
├── Outcome Metrics
├── Role Blueprints
├── Harness Templates
├── Skills
├── Tools / Connectors
├── WorkPackage Templates
├── Workcell Blueprints
├── Workflows
├── Simulations
├── Failure Cases
├── Evaluation Packs
├── UI Extensions
└── Solution Templates
```

Domain Pack 的职责是提供行业事实、结构和经过验证的资产，不允许把模型常识自动当作行业规范。

---

# 32. Dream Factory：真正的行业 AI 梦工厂

Dream Factory 不是 Agent 模板商店，而是一套把行业项目持续加工成可复用工作能力的生产体系。

## 32.1 Industry Asset Factory

沉淀：

```text
Domain Ontology
Operational Object/Action Model
Artifact/Decision Schemas
Role Blueprint
HarnessSpec
Skill/Tool/Connector
Workflow
Policy
Evaluation
Failure Case
Simulation Scenario
```

## 32.2 Solution Factory

```text
Customer Goal
→ ProblemSpec
→ WorkPackages / OutcomeContracts
→ Workcells
→ Harness/Runtime Selection
→ Integration
→ Evaluation
→ Deployment
```

## 32.3 Verification & Certification Factory

```text
Component Test
→ Skill Test
→ Harness Test
→ Role Test
→ WorkPackage Test
→ Workcell Test
→ Solution Replay
→ Shadow Run
→ Certification
```

## 32.4 Delivery & Evolution Factory

```text
Production Trace
→ Failure Pattern
→ Human Correction
→ New Evaluation
→ Candidate Skill/Harness/Workflow
→ Regression
→ Certified Asset
→ New Solution Version
```

## 32.5 Work-as-a-Service / Outcome BaaS

用友“银账通对账全托管”的公开案例体现了一种重要商业模式：客户不再深度操作整个过程，而是把端到端工作交出去，主要验收最终闭环结果。[R47]

Morn Dream Factory 因此应支持：

```text
Software / Agent License
        ↓
Certified Work Capability
        ↓
Managed Workcell
        ↓
Work-as-a-Service / Outcome Delivery
```

例如：

```text
BioLab：Dataset → Reviewed Scientific Claim
Pharma：Deviation → Reviewed Investigation Draft
Finance：Raw Transactions → Verified Reconciliation Package
Manufacturing：Alarm → Verified Maintenance Plan
```

客户购买的是可验收工作能力，而不是“几个 Agent”。

## 32.6 Solution Delivery Lifecycle

吸收用友 FDE/持续运营、华为梦工厂和工业AI项目的共性，形成：

```text
Discovery
→ Baseline
→ ProblemSpec
→ Prototype
→ Historical Replay
→ Shadow Run
→ Value Validation
→ Controlled Deployment
→ Production
→ Outcome Review
→ Asset Extraction
→ Next Version
```

Dream Factory 的竞争力最终来自这个生产工艺，而不是 Hub 中模板数量。


# 33. Evolution Engine：Morn 的第五闭环

Morn 的最终目标不是把既有组织自动化一次，而是让**工作结构和执行形态本身持续可计算、可评估、可版本化、可进化**。

## 33.1 五个核心闭环

```text
1. World Loop
Observe → State → Action → New State

2. Work Loop
Goal → WorkPackage → Workcell → Delivery

3. Evidence Loop
Artifact → Review → Decision → Provenance

4. Outcome Loop
Expected → Actual → Delta → Attribution

5. Evolution Loop
Trace → Pattern → Candidate → Simulation → Evaluation → Shadow → Certification → Promotion
```

前四环让系统可靠工作，第五环让系统在不破坏治理的情况下变得更好。

## 33.2 Evolution Engine 组成

```text
Evolution Engine
├── Trace Miner
├── Object-Centric Process Miner
├── Work Graph Analyzer
├── Capability Gap Detector
├── Redundancy Detector
├── Bottleneck Detector
├── Human Effort Analyzer
├── Software Dependency Analyzer
├── Outcome Attribution
├── Candidate Generator
├── Branch / Scenario Manager
├── Simulator / Replay Runner
├── Evaluation Engine
├── Certification Engine
└── Promotion / Rollback Manager
```

## 33.3 六种进化

### Capability Evolution

反复出现的成功轨迹 → Candidate Skill/Tool/Program → Evaluation → Certified Capability。

### Workflow Evolution

识别重复转交、无价值同步、重复审批和可自动提交步骤，生成 Workflow vNext；只有通过 Replay/Shadow 后才 Promotion。

### Role Evolution

当某岗位的大量工作已经迁移到数字 Workcell，Morn 可以提出 Role 合并、Supervisor 化或职责重分配建议；组织变更本身也是需要审批的 Artifact/Decision。

### Actor Evolution

Actor 的自治等级不是永久身份属性，而是由特定 WorkPackage/Scope 下的长期评测逐级提升：Assist → Draft → Supervised Execute → Bounded Autonomous → Certified Autonomous。

### Software / System Evolution

现有软件最初是 Provider；当 Morn 已具备稳定的 Native Capability 时，可以进入：

```text
Connector
→ Wrapper/Orchestrated Use
→ Shadow Native Capability
→ Partial Replacement
→ Native Replacement
→ Legacy Module Retirement
```

### Organization Evolution

Morn 可以从长期 Work Graph 中识别“仅因部门边界产生的同步/转交工作”，提出从 Department-centric 向 Outcome-centric Workcell 的重构候选。

## 33.4 Agent 进化不一定意味着更多 Agent

一个成熟 Agent 连续执行大量同构任务后，如果决策边界稳定、异常率低，可将常见路径蒸馏成：

```text
Rule / State Machine / Program / Solver
```

只把长尾异常留给 Actor。Morn 因而优化：

```text
Outcome Quality ↑
Reliability ↑
Speed ↑

Money / Token / Human Burden / Risk / Coordination Complexity ↓
```

核心原则：**最小化完成目标所需要的智能复杂度。**

## 33.5 Organization Branching

组织和数字工厂可以像代码一样创建受治理分支：

```text
Factory v2.4
├── Branch A：增加两个 Digital Employees
├── Branch B：取消一个审批层
├── Branch C：替换 APS Provider
└── Branch D：重构采购 Workcell
```

每个分支在历史数据/数字孪生中比较 Cost、Quality、Risk、Cycle Time、Human Load、Energy、Revenue 等指标，再决定是否进入 Shadow。

## 33.6 Capability Inheritance / Industry Evolution Flywheel

一个客户项目中的有效能力只有经过：

```text
去客户敏感信息
→ 抽象 Context of Use
→ 依赖显式化
→ 跨场景 Evaluation
→ Certification
```

才能成为可复用 Domain Asset，再部署到其他客户并产生新的站点特化版本。

```text
Site Experience
→ Generalized Candidate
→ Certified Domain Capability
→ Site-specific Variant
→ New Outcome
→ Improved Generic Capability
```

客户私有数据不默认回流；回流的是经过授权/脱敏的失败模式、评测结构和可复用能力。

## 33.7 Evolution Center 产品表面

Workbench/Console 可提供 Evolution Center：

```text
发现：
- 重复人工流程
- 可自动化 WorkPackage
- Agent Skill 可确定化机会
- 低利用率软件模块
- 重复审批
- Capability Gap

每个候选显示：
Current State
Expected Benefit
Risk
Evidence Window
Required Simulation
Shadow Status
Promotion Gate
```

Evolution Center 不是“自动修改生产”的按钮，而是受治理组织改进台。

# 34. Workbench：从聊天台变成工作现场

首页建议展示：

```text
当前 Mission
关键 Operational Objects
我的 WorkPackages / WorkContracts
Workcell 状态
待审批 Action
Artifacts / Decisions
异常与 Blocked
关键 Metrics / Outcomes
近期世界状态变化
Attention Queue
```

Chat 仍然重要，但只是交互入口之一。

关键视图：

- Mission View；
- World View；
- WorkPackage Board；
- Artifact/Decision Viewer；
- Approval Center；
- Team/Workcell View；
- Digital Thread；
- Outcome Dashboard；
- Harness/Runtime Live View；
- Attention Queue。

## 34.1 Workcell Live View

Golutra、Tutti等项目强化了“factory floor”式实时执行视图的价值：人类不应一直盯聊天，而应只在 Blocked、Approval、Risk、Conflict、Failure 时被拉回。[R41][R44]

建议：

```text
┌────────────────────────────────────┐
│ BioLab Analysis Workcell           │
├────────────────────────────────────┤
│ Analyst / Codex        RUNNING     │
│ Python Pipeline        COMPLETE    │
│ Statistical Reviewer  WAITING     │
│ PI Approval            PENDING     │
├────────────────────────────────────┤
│ Current Work Contract              │
│ Dataset → Reviewed Claim           │
├────────────────────────────────────┤
│ Attention                          │
│ ⚠ Sample S-034 exclusion requested│
└────────────────────────────────────┘
```

## 34.2 Attention Queue

Attention Queue 是人类稀缺注意力的统一入口：

```text
Approval Required
Policy Conflict
Evidence Conflict
Tool Failure
Budget Risk
Deadline Drift
Low Confidence
Irreversible Action
Representation Boundary
```

正常执行静默推进，只有真正需要人类判断时才升级。

# 35. Studio：真正的 Foundry UI

继续保留一个 Studio，不新造十个顶层产品。

内部能力：

```text
Component / Capability Builder
Actor Factory
Role & Harness Builder
Skill Factory
Tool/Connector Builder
WorkPackage Factory
Workcell / Team Builder
Workflow Designer
Artifact/Decision Schema Builder
Ontology / Operational World Builder
Simulation Lab
Evaluation Lab
Solution Builder / Compiler
Domain Pack Builder
Dream Factory Builder
```

其中“Factory”是生产线概念，不是独立应用。

---

# 36. Console：Intelligence & Workforce Control Tower

Console 从“看 Agent 运行日志”升级为组织控制塔：

```text
Identity / Registry
Organization Graph
Workforce Graph
Operational World State
WorkPackage / Commitment
Delegation / Authority
Representation Contracts
Memory Ownership
Harness / Runtime Health
Approvals / Attention Queue
Policies
Autonomy Levels
Trace / Errors
Outcomes / Metrics
Cost / Budget
Incidents
Deployments / Nodes
Version / Rollback
Lifecycle / Retirement
```

Microsoft Agent 365、ServiceNow AI Control Tower、Workday ASOR，以及 AgentSpace 这类新项目都说明“跨 Agent 发现、身份、治理、生命周期、Harness 和统一控制平面”正在快速基础设施化。[R16][R28][R29][R42]

Morn 的区别是：Console 不只治理 Agent，还治理 Human/Worker/Service/Device、WorkContract、Representation、Memory Ownership 和 Outcome。

# 37. Hub：从 Bot Store 到智能工作资产市场

可发布资产：

```text
Component
Model Adapter
Runtime Adapter
Skill
Tool
Connector
KnowledgePack
ContextCore
Ontology Module
ArtifactSchema
DecisionSchema
RoleBlueprint
ActorTemplate
HarnessTemplate
TeamBlueprint
WorkPackageTemplate
WorkcellBlueprint
Workflow
PolicyPack
EvaluationPack
SimulationScenario
DomainPack
SolutionTemplate
Certified Work Capability
DreamFactory
```

每项资产必须包含：版本、作者、License、依赖、权限、数据范围、风险、测试、认证、兼容性、Changelog、弃用策略。

信任等级：

```text
Unverified
Community Tested
Verified
Certified
Restricted
Deprecated/Retired
```

---

# 38. BioLab Dream Factory：第一块真实样板

## 38.1 为什么优先 BioLab

BioLab 能同时验证：

- 专业领域 Ontology；
- Artifact-first；
- Human + Actor + Pipeline；
- 长程工作流；
- Evidence/Claim；
- 数据血缘；
- 真实分析代码；
- 审批；
- 后期仪器/机器人；
- Dream Factory 资产回流。

同时，当前科研 Agent 研究依然显示真正闭环自主科研离生产成熟很远。Nature 的 Robin 将文献与数据分析 Agent 结合，但湿实验仍由人类完成；AutoScientists、InternAgent、XScientist 等更多聚焦计算科研、探索与复现。[R30][R31][R32][R20]

因此 BioLab v0.x 应定位为：

> **科研运营与证据闭环系统，而不是“全自主实验室”。**

## 38.2 Lab Operational World

```text
ResearchProgram
ResearchQuestion
Evidence
Hypothesis
Experiment
ExperimentSpec
AnalysisPlan
ProtocolSpec
Sample
Material
Instrument
Run
Observation
Dataset
AnalysisRun
Result
ScientificClaim
Figure
Manuscript
DecisionRecord
```

必须严格区分：

```text
Hypothesis
≠ Observation
≠ Measurement
≠ Derived Result
≠ Interpretation
≠ Scientific Claim
≠ Validated Claim
```

## 38.3 成员

```text
Human PI
Lab Director Actor
Literature/Evidence Scientist
Hypothesis Scientist
Experiment Designer
Protocol Engineer
Human Lab Coordinator
Human Technician / Device
Data Steward
Bioinformatics Scientist
Statistical Reviewer
Scientific Critic
Manuscript Writer
Lab Operations
```

不创建 13 个永久 Agent；稳定核心 + 动态 Workcell。

## 38.4 首批 Certified Work Products

### Literature-to-Evidence

ResearchQuestion → EvidenceGraph。

### Question-to-Hypothesis

Evidence → Reviewable/Falsifiable Hypothesis。

### Hypothesis-to-Experiment

Approved Hypothesis → ExperimentSpec + AnalysisPlan。

### Protocol-to-Human-Run

Approved Protocol → Human Work Order → Run/Deviation/Data Capture。

### Dataset-to-Reviewed-Claim

RawDataset → QC → AnalysisRun → Independent Review → ScientificClaim。

### Claim-to-Manuscript

Approved Claims + Figures + Methods → Manuscript Draft。

### Manuscript-Consistency-Audit

Manuscript → 引用/方法/图表/主张/分析一致性验证。

## 38.5 湿实验模式

A. Human execution（MVP）  
B. Local automation（Opentrons/SiLA/厂商 SDK）  
C. Cloud lab

任何物理执行：

```text
Experiment Intent
→ ExperimentSpec
→ ProtocolSpec
→ Device-independent Operation Graph
→ Device Binding
→ Code
→ Static/Simulation Check
→ Human Approval
→ Execution
→ Run Artifact
```

禁止自由文本 LLM 直接控制实验设备。

## 38.6 Runtime 分工

```text
科学开放推理       → Morn Native/Hermes/其他强推理 Runtime
数据分析           → Python/R/Jupyter/Nextflow
实验 Protocol      → Deterministic governed workflow
物理执行           → Human / Instrument / Cloud Lab Adapter
```

## 38.7 Scientific Digital Thread

```text
ResearchQuestion
→ Evidence
→ Hypothesis
→ ExperimentSpec
→ Protocol
→ Run
→ Sample
→ RawDataset
→ AnalysisRun
→ ScientificClaim
→ Figure
→ Manuscript
```

每一步追踪 Who/When/Version/Input/Tool/Approval/Outcome。

---

# 39. Pharma Dream Factory 样板

建议首个药企场景不是“全流程新药研发”，而是**质量偏差与 CAPA 调查**。

Operational Objects / Artifacts：

```text
Batch
DeviationEvent
Equipment
Material
Personnel
EvidenceBundle
Timeline
RootCauseCandidate
InvestigationDraft
CAPADraft
ApprovalRecord
EffectivenessCheck
```

Agent 可做：检索、整理、时间线、相似案例、缺失证据、草案、趋势分析。

首期禁止：

- 批次最终放行；
- 修改原始记录；
- 正式关闭偏差；
- 最终根因批准；
- 修改生产参数；
- 正式监管提交。

---

# 40. Personal Morn

个人模式不能被企业架构吞掉。

```text
Personal Assistant
+ Personal Workspace
+ Personal Memory
+ Files/Browser/Email/Calendar
+ Personal WorkPackages
+ Approval Center
+ Specialist Workcells
+ Personal Harness Profiles
```

可形成：Research、Writing、Coding、Job Search、Learning、Project Management 等专业 Actor/Workcell。

个人与客户 Workspace 默认隔离；个人经验产品化必须经过脱敏、测试和认证，不能把客户内容写回个人 Memory。

## 40.1 Personal Agent 与 Organization-native Actor

个人 Agent 是用户私有工具；组织原生 Actor 则需要：

```text
独立身份
组织位置
Owner/Sponsor
Role
权限
审计
组织Workspace
生命周期
```

两者可共享 Runtime/Harness，但治理语义不同。

## 40.2 Digital Delegate / Human Twin

用户可以创建代表自己的 Human-delegated Actor，但必须有 RepresentationContract；Morn 不把“像本人说话”自动等同于“有权代表本人决定”。

典型产品：

```text
Personal Research Delegate
PI Digital Delegate
Founder/CEO Advisory Twin
Domain Expert Twin
```

其主要价值是复制可验证的判断框架、知识和沟通习惯，而不是复制全部人格和隐私记忆。

# 41. 数字员工、数字分身、虚拟员工、AI Coworker、Agentic Application 的统一归位

## 41.1 Digital Worker

以执行工作为中心，通常 AI + Automation + Tool。

## 41.2 Digital Employee

更强调岗位、职责和组织语义；市场上同名产品能力差异很大，不能仅凭名称判断成熟度。

## 41.3 AI Specialist

专业受限岗位，强调 scope/authority/governance，ServiceNow 是代表。[R06]

## 41.4 AI Coworker

更像面向人的统一协作入口/工作伙伴；适合映射为 Personal/Professional Actor 产品形态。

## 41.5 Virtual Employee / Digital Human

必须区分智能主体与表现层：

```text
Actor Core
+ Interface (chat/voice)
+ Embodiment (avatar/digital human/robot)
```

## 41.6 Human-delegated Actor / Digital Delegate

这一类不是普通数字员工，而是受控代表某个真实人。

```text
Human Principal
→ RepresentationContract
→ Delegated Actor
→ Limited Scope / Disclosure / Expiry / Revocation
```

必须显式区分“建议”“模仿”“代表性意见”“正式授权决定”。

## 41.7 Agentic Application

Oracle 的方向说明未来企业购买的可能是完整结果型应用，而不是单个 Agent。[R07]

Morn 中统一映射为：

```text
Solution
= Operational World
+ WorkContracts
+ Workcells
+ Harnesses
+ Policies
+ Integrations
+ Verification
+ Deployment
```

# 42. 虚拟工厂、数字工厂、AI 工厂、数字员工与 Morn 的关系

## 42.1 AI Factory / Token Factory

生产智能计算、Token、模型推理能力。Morn 不做 GPU/Token Factory，只管理 Model Policy、Budget、Cost、Latency、Cache 与 Provider Failover。[R33]

## 42.2 Digital Factory

在 Morn 语义中，Digital Factory 不是“一个 3D 模型”，而是现实工厂的数字运营系统：

```text
Digital Factory
= Operational World
+ Asset/Process/Organization/Workforce Twin
+ Work Graph
+ Mixed Workforce
+ Software/System Providers
+ Automation/Devices
+ Verification
+ Outcome Loop
```

ERP、MES、APS、PLM、WMS、TMS、SCADA、PLC、BI 等在初期都是 Provider，而不是必须被 Morn 重写的永久组件。

## 42.3 Virtual Factory

Virtual Factory 是可被复制、暂停、快进、修改和试错的工厂分支：

```text
Digital Factory Snapshot
+ Simulation
+ Counterfactual Scenario
+ Synthetic Actors/Devices/Events
= Virtual Factory
```

用途：布局/排产/设备/人员/机器人/流程/政策修改的事前验证。Siemens Digital Twin Composer 与 NVIDIA 相关工业数字孪生路线可作为 Simulator Provider 参考。[R14][R15][R55]

## 42.4 Digital Employee / Virtual Employee

`Digital Employee` 的核心是组织工作身份；`Virtual Employee` 是 `Digital Employee + Embodiment`（Voice/Avatar/AR/VR/Robot Body）。Actor 的身份与心智/工作资产不能绑定某个 3D 形象。

同一个 Actor 可以在：

```text
Chat UI
Voice
Avatar
AR/VR
Robot Binding
```

之间切换，而 Role、Memory、Authority、Work History 保持连续。

## 42.5 Evolving Digital Factory

长期终态不是 Digital Twin，而是：

```text
Digital Twin        → 知道现在怎样
Digital Workforce   → 能在其中工作
Virtual Factory     → 可以安全试错
Morn Work OS        → 组织真实工作
Evolution Engine    → 持续改造工作系统
```

共同形成 **Evolving Digital Factory**：能够感知自身状态、评价工作结构、生成改进候选、在虚拟世界验证，并经过治理后升级真实生产版本。

## 42.6 Replacement Ladder：替代的基本单位是 Work，不是整家公司

Morn 进入传统行业采用分级替代：

```text
R0 Observe        只读接入、建立事实
R1 Assist         辅助真人，不改变正式流程
R2 Orchestrate    Morn 控制 Work，原系统/人执行
R3 Shadow Replace Morn Native 能力与旧系统并行比较
R4 Partial Replace 替代某个 Workflow / 岗位职责 / 软件模块
R5 Native Replace 旧模块可正式退役
R6 Autonomous Domain Operation 多个原模块被 Morn-native Work System 接管
```

R6 也不意味着必须替代 ERP/PLC/账务核心等交易或物理基础设施，而是某一业务域的智能工作已由 Morn 原生体系承担。

## 42.7 Domain Penetration Loop

所有行业使用统一深入路径：

```text
CONNECT
→ UNDERSTAND
→ ASSIST
→ ORCHESTRATE
→ AUTOMATE
→ SHADOW
→ VALIDATE
→ REPLACE
→ OPTIMIZE
→ EVOLVE
```

开始时 Morn 是 Control Plane；随着 Outcome 数据、Domain Capability 与 Verification 积累，可以逐步向 Execution Plane 下沉。

## 42.8 Software Absorption

Morn 不预设“某类软件永远存在”。如果一个传统应用长期只剩少量稳定功能，而 Morn 已有可验证 Native Capability，则可以进入：

```text
Existing Software
→ Connector
→ Observed Work Graph
→ Morn-native Candidate
→ Shadow Comparison
→ Partial Migration
→ Retirement
```

这里取代的是具体 Work/Module，不是为了追求“软件数量归零”。

## 42.9 Industry AI Dream Factory

Industry AI Dream Factory 生产的不是一座固定工厂，而是可复用的行业能力与数字组织蓝图：

```text
Morn Core
→ Manufacturing Dream Factory
→ Digital Factory Template
→ Customer Digital Factory
→ Evolution / Outcome Feedback
→ Certified Domain Assets
```

Morn 因而是“制造和运营数字工厂的元工厂”，而不是单一数字工厂产品。

---

# 43. Build vs Integrate：必须自己做什么

## 43.1 Morn 必须自己掌握

```text
Morn Kernel contracts
Workspace / Identity semantic model
Portable Actor standard
Workforce SoR
Work SoR
Operational World standard
RoleSlot / ResponsibilityBinding
Delegation / Accountability
WorkPackage / AcceptanceSpec
Workcell
Artifact / Decision / Outcome model
Action Gateway semantic contract
Organization/Solution Compiler
Domain Pack standard
Evaluation/Certification semantics
Solution Package
Dream Factory asset feedback
Workbench/Studio/Console/Hub experience
```

## 43.2 优先复用/适配

```text
基础大模型/Embedding
GPU/Token基础设施
MCP / A2A
OAuth / OIDC / 企业IAM
W3C PROV
OpenTelemetry
PostgreSQL / SQLite / Object Storage
Temporal / Restate / DBOS
DataHub / OpenLineage
Quick BI / Power BI / Fabric
Celonis / Palantir 类企业底座
专业模拟器
Nextflow / Jupyter / R / Python
Robot / Instrument SDK
Hermes / OpenClaw / openJiuwen 等 Runtime
```

## 43.3 Native Runtime 与 DeepSeek Harness 的正确地位

v10.2 调整优先级：Morn **必须自研执行抽象与控制面，但不再要求第一阶段自研完整 Agent 执行栈**。

`MornNativeHarness` 的定位：

- 最小本地 fallback；
- Reference Harness；
- 测试/基准 Runtime；
- 确保关键契约不依赖单一第三方。

`DeepSeekHarnessProvider` 的定位：

- 首个重点默认 Harness Provider 候选；
- 优先复用 Agent Loop、Tool Pipeline、Session、Skill、Subagent、Sandbox、Web/Headless 等执行能力；
- 通过 Morn Action Gateway 和 Event Normalizer 与正式 Work/World 系统隔离。[R51][R52]

`Cordis` 的定位：

- Capability Fabric 的实验性 Meta-Framework 候选；
- 当前只做 Architecture Spike，不成为 Morn Semantic Kernel 的硬依赖，因为官方 API 仍未稳定。[R53][R54]

因此 Morn 的长期中立性来自**自己的语义合同和 Provider API**，而不是自己重写所有 Agent Runtime。

---

# 44. 数据模型

## 44.1 Kernel

```text
principals
identities
organizations
workspaces
workspace_memberships
policies
permissions
approval_requests
approval_decisions
secrets
ledger_entries
lifecycle_records
package_installations
```

## 44.2 Operational World

```text
object_types
objects
relation_types
relations
state_snapshots
event_types
events
action_types
action_instances
goals
metrics
outcome_records
```

## 44.3 Actor / Workforce

```text
actor_templates
actor_instances
actor_origins
representation_contracts
decision_policy_assets
communication_profiles
expertise_profiles
harness_bindings
runtime_bindings
model_policies
capability_bindings
workspace_bindings
role_blueprints
role_slots
member_bindings
responsibility_bindings
organization_edges
delegations
commitments
performance_records
```

## 44.4 Memory Ownership

```text
memory_records
memory_ownership_scopes
memory_retention_policies
memory_transfer_records
memory_promotion_requests
institutional_memory_records
```

## 44.5 Work / Evidence

```text
missions
work_package_templates
work_packages
work_contracts
acceptance_specs
outcome_contracts
execution_modes
assignments
artifact_types
artifacts
artifact_versions
artifact_relations
artifact_reviews
artifact_approvals
decision_packages
execution_receipts
verification_reports
delivery_packages
attention_items
```

## 44.6 Harness / Workflow

```text
harness_specs
harness_versions
harness_runtime_profiles
harness_evaluation_runs
workflow_definitions
workflow_versions
workflow_runs
workflow_steps
workflow_checkpoints
signals
compensation_actions
recovery_records
```


## 44.7 Capability Seam / Evolution

```text
capability_definitions
capability_providers
capability_consumers
capability_bindings
capability_requirements
capability_effects
effect_classifications
provider_scopes
provider_lifecycle_events

evolution_candidates
evolution_branches
evolution_experiments
evolution_evaluations
shadow_runs
promotion_decisions
replacement_records
retirement_records
```

Evolution 对象不直接覆盖 Production 对象；Promotion 后创建新的正式版本，并保留 `derived_from / supersedes`。

## 44.8 Foundry / Dream Factory

```text
problem_specs
scenario_blueprints
capability_requirements
capability_resolution_records
solution_templates
solution_instances
domain_packs
domain_installations
evaluation_packs
evaluation_runs
simulation_scenarios
certifications
work_capability_products
delivery_lifecycles
deployment_profiles
deployments
node_registrations
customer_feedback_records
```

# 45. API 与核心接口

## 45.1 Execution Harness

```rust
trait ExecutionHarness {
    fn start(&self, actor: &ActorInstance, work: &WorkContract, ctx: RuntimeContext)
        -> Result<HarnessSession>;
    fn send(&self, session: &HarnessSession, event: ActorEvent)
        -> Result<ActorProposal>;
    fn stream_events(&self, session_id: &str)
        -> Result<Box<dyn Iterator<Item = HarnessEvent>>>;
    fn inspect(&self, session_id: &str) -> Result<HarnessSnapshot>;
    fn interrupt(&self, session_id: &str) -> Result<()>;
    fn resume(&self, session_id: &str) -> Result<()>;
    fn terminate(&self, session_id: &str) -> Result<()>;
}
```

## 45.2 Action Gateway

```rust
trait ActionGateway {
    fn preview(&self, proposal: ActionProposal) -> Result<ActionPreview>;
    fn authorize(&self, proposal: ActionProposal) -> Result<PolicyDecision>;
    fn execute(&self, authorized: AuthorizedAction) -> Result<ExecutionReceipt>;
    fn verify(&self, receipt: ExecutionReceipt) -> Result<VerificationResult>;
    fn recover(&self, failure: ActionFailure) -> Result<RecoveryResult>;
}
```

## 45.3 Artifact Store

```rust
trait ArtifactStore {
    fn create(&self, draft: ArtifactDraft) -> Result<ArtifactVersion>;
    fn review(&self, id: ArtifactId, review: Review) -> Result<()>;
    fn approve(&self, id: ArtifactId, approval: Approval) -> Result<()>;
    fn supersede(&self, old: ArtifactId, new: ArtifactDraft)
        -> Result<ArtifactVersion>;
    fn lineage(&self, id: ArtifactId) -> Result<ProvenanceGraph>;
}
```

## 45.4 Organization Compiler

```rust
trait SolutionCompiler {
    fn analyze(&self, request: SolutionRequest) -> Result<ProblemSpec>;
    fn propose(&self, problem: ProblemSpec) -> Result<ProposedSolution>;
    fn validate(&self, solution: ProposedSolution) -> Result<ValidationReport>;
    fn compile(&self, approved: ApprovedSolution) -> Result<SolutionPackage>;
}
```

Compiler 输出必须显式包含：

```text
ExecutionMode
RoleSlot
MemberType
HarnessBinding
RuntimeProfile
AcceptanceSpec
OutcomeContract
Assumptions
Decision Sources
```

## 45.5 Representation Service

```rust
trait RepresentationService {
    fn authorize_representation(
        &self,
        principal: PrincipalId,
        actor: ActorId,
        scope: RepresentationScope,
    ) -> Result<RepresentationContract>;

    fn check(&self, actor: ActorId, action: RepresentationalAction)
        -> Result<RepresentationDecision>;

    fn revoke(&self, contract_id: RepresentationContractId) -> Result<()>;
}
```


## 45.6 Capability Seam

```rust
trait CapabilityProvider {
    fn definition(&self) -> CapabilityDefinition;
    fn requirements(&self) -> CapabilityRequirements;
    fn effect_contract(&self) -> EffectContract;
    fn mount(&self, scope: CapabilityScope) -> Result<ProviderHandle>;
    fn unmount(&self, handle: ProviderHandle) -> Result<()>;
}
```

Provider 的 `unmount` 只负责 E0 生命周期清理，不得假定 E2/E3 现实 Effect 被自动逆转。

## 45.7 Evolution Engine

```rust
trait EvolutionEngine {
    fn analyze(&self, window: ObservationWindow) -> Result<Vec<EvolutionCandidate>>;
    fn branch(&self, candidate: EvolutionCandidate) -> Result<EvolutionBranch>;
    fn evaluate(&self, branch: EvolutionBranch) -> Result<EvolutionReport>;
    fn shadow(&self, approved: ApprovedEvolution) -> Result<ShadowRun>;
    fn promote(&self, certified: CertifiedEvolution) -> Result<ReleaseRecord>;
    fn rollback(&self, release: ReleaseRecord) -> Result<RollbackRecord>;
}
```

`promote()` 必须经过 Policy/Certification，不允许由单个 Actor 直接调用绕过治理。

# 46. 工程代码架构建议

继承现有 Rust/Tauri 桌面主线，但按领域拆 crate：

```text
crates/
├── morn-kernel/
│   ├── identity
│   ├── workspace
│   ├── policy
│   ├── approval
│   ├── secrets
│   ├── ledger
│   └── lifecycle
├── morn-world/
│   ├── object
│   ├── relation
│   ├── state
│   ├── event
│   ├── action
│   ├── goal
│   └── outcome
├── morn-artifact/
│   ├── schema
│   ├── version
│   ├── review
│   ├── provenance
│   ├── decision
│   └── delivery
├── morn-capability/
│   ├── registry
│   ├── skill
│   ├── tool
│   ├── connector
│   ├── knowledge
│   ├── memory
│   ├── analytics
│   └── model
├── morn-harness/
│   ├── spec
│   ├── binding
│   ├── seam
│   ├── scope
│   ├── effects
│   ├── dsh_provider
│   ├── cli
│   ├── daemon
│   ├── api
│   ├── container
│   ├── normalization
│   ├── evaluation
│   └── recovery
├── morn-runtime/
│   ├── native
│   ├── context
│   ├── model_gateway
│   ├── action_gateway
│   ├── sandbox
│   └── adapters
├── morn-actor/
│   ├── template
│   ├── instance
│   ├── origin
│   ├── representation
│   ├── decision_policy
│   ├── bindings
│   ├── self_model
│   └── lifecycle
├── morn-organization/
│   ├── role
│   ├── member_binding
│   ├── responsibility
│   ├── delegation
│   ├── commitment
│   ├── team
│   └── workcell
├── morn-work/
│   ├── mission
│   ├── work_package
│   ├── work_contract
│   ├── acceptance
│   ├── outcome_contract
│   ├── execution_mode
│   ├── assignment
│   ├── workflow
│   ├── attention
│   ├── checkpoint
│   └── recovery
├── morn-foundry/
│   ├── problem_spec
│   ├── decomposer
│   ├── package_compiler
│   ├── member_type_planner
│   ├── execution_mode_planner
│   ├── capability_resolver
│   ├── harness_compiler
│   ├── evaluation_generator
│   └── deployment_compiler
├── morn-evolution/
│   ├── trace_miner
│   ├── work_graph_analyzer
│   ├── candidate
│   ├── branch
│   ├── simulation
│   ├── shadow
│   ├── certification
│   └── promotion
├── morn-assurance/
│   ├── evaluator
│   ├── simulation
│   ├── shadow
│   ├── certification
│   └── incident
├── morn-domain/
│   ├── registry
│   ├── ontology
│   ├── pack
│   └── scenario
└── morn-package/
    ├── manifest
    ├── work_system_as_code
    ├── dependency
    ├── installer
    ├── migration
    └── signature
```

外部运行 Sidecar / Harness Host：

```text
runtime-host/
├── python
├── jupyter
├── codex-cli-adapter
├── claude-code-adapter
├── langgraph-adapter
├── hermes-adapter
├── openclaw-adapter
├── openjiuwen-adapter
└── scientific-pipelines
```

关键依赖方向：

```text
Kernel/World/Work contracts
        ↑
Actor/Organization
        ↑
Harness/Runtime adapters
        ↑
Product surfaces
```

禁止 Runtime/CLI Adapter 反向拥有正式业务对象。

# 47. 存储与部署

## 47.1 Local/Desktop

- SQLite：Kernel、World、Work 元数据；
- 本地对象存储：Artifact 内容；
- OS Keychain/加密 Secret Store；
- 可选本地向量/全文索引；
- Native durable task log；
- Python Sidecar；
- 本地 CLI Harness Host。

## 47.2 Team/Enterprise

```text
PostgreSQL
+ Object Storage
+ Message/Event Bus
+ Enterprise IAM
+ Central Audit
+ Durable Runtime Adapter
+ Harness Daemon/Remote Runner
+ Optional Graph/Vector Stores
```

Kernel 接口不能写死某个数据库。

## 47.3 Morn Node：本地优先的多节点部署模型

用友“AI盒子”、AgentSpace Remote Daemon 等形态说明，企业不会只有“桌面版/云版”二元选择。[R42][R45]

Morn 定义逻辑部署 Profile，而不自造硬件：

```text
Morn Desktop
个人电脑/单用户

Morn Team Node
团队服务器/共享 Harness

Morn Private Node
企业内网/私有云

Morn Edge Node
实验室、工厂、设备现场

Morn Cloud Control
跨节点治理、Registry、策略和观测
```

WorkContract 可以选择执行节点，Secrets/数据驻留/设备权限仍由 Workspace 与 Policy 决定。

# 48. Solution Package 与 Work System as Code

```text
solution.morn/
├── manifest.yaml
├── domain/
├── world/
├── actors/
├── representations/
├── roles/
├── harnesses/
├── runtimes/
├── work-packages/
├── work-contracts/
├── workcells/
├── workflows/
├── artifacts/
├── capabilities/
├── connectors/
├── policies/
├── evaluations/
├── simulations/
├── deployment/
├── secrets.schema.yaml
├── migrations/
└── changelog.md
```

安装：

```text
Verify Signature
→ Check Compatibility
→ Resolve Dependencies
→ Request Configuration
→ Bind Connectors/Harnesses
→ Initialize World
→ Run Evaluation
→ Create Shadow Instance
→ Human Approval
→ Activate
```

升级：

```text
New Version
→ Migration Preview
→ Regression Evaluation
→ Snapshot
→ Staged Upgrade
→ Monitor
→ Commit / Rollback
```

## 48.1 Work System as Code

Tutti 的 `tutti.toml` 说明 Role、Runtime、Workflow、Gate、Policy 可以像基础设施一样版本化。[R44] Morn 应扩大这一思想：

```yaml
morn:
  domain: biolab
  version: 1.0

world:
  schema: biolab-world@1.2

roles:
  experiment_designer:
    member_type: actor
  pi:
    member_type: human

harnesses:
  research:
    spec: biolab-research-harness@1.3

workcontracts:
  dataset_to_claim:
    acceptance: biolab-claim-v3
    outcome: reviewed-scientific-claim-v2

workcells:
  analysis:
    roles: [analyst, reviewer, pi]

policies:
  - sample-exclusion-policy

evaluations:
  - biolab-analysis-suite
```

这样 Team/Solution/WorkContract 可以：

```text
Version
Diff
Review
Fork
Export
Install
Rollback
```

Studio 是可视化编辑器，Manifest 是机器可执行的事实来源；二者必须双向一致。

# 49. 竞争与差异化

## 49.1 已经商品化/趋于商品化的能力

```text
Agent Loop
MCP/A2A
多模型
Memory
Skill
多Profile
Subagent Delegation
多Agent聊天
DAG Workflow
Local Assistant
Agent Marketplace
Agent Org Chart
Basic Ontology
Artifact Sidebar
Agent Identity
Control Tower
本地优先
Human + Agent Workspace
多 Runtime / CLI 切换
Agent 身份跨 Runtime 保留
Role/Owner/Approval/Audit
Agent Ops as Code
基础 Harness
```

Golutra、AgentSpace、Commonly、Tutti、YonCode 等正在同时覆盖这些方向。[R41][R42][R43][R44][R46]

所以不能拿这些单点当护城河。

## 49.2 Morn 的差异化组合

```text
Executable Operational World
× Work Contract / Acceptance / Outcome Contract
× Mixed Workforce
× Governed Composability / Harness-neutral Execution
× Artifact / Decision / Execution / Outcome Records
× Delegation / Representation / Accountability
× Organization Compiler
× Simulation / Certification
× Evolution Engine / Replacement Ladder
× Work System as Code
× Industry Asset & Delivery Flywheel
```

## 49.3 真正会越来越强的壁垒

- 行业 Operational Object/Action 模型；
- 真实 WorkContract/Acceptance/Outcome 标准；
- 专家纠错数据；
- 失败模式库；
- 专项 Evaluation；
- 高质量 Connector/Harness Adapter；
- 经过真实项目认证的 Workcell；
- 客户系统集成经验；
- 真实 Outcome 反馈；
- 决策与执行 Provenance；
- 伙伴/行业专家网络；
- Work-as-a-Service 的交付运营能力。

## 49.4 反向检验

如果明天 MornNativeRuntime 被替换成 openJiuwen/Hermes/Codex CLI，Morn 仍应保留：

```text
Identity/Workspace
Operational World
Workforce/Organization
WorkPackage/WorkContract/Acceptance/OutcomeContract
Artifact/Decision/Outcome
Representation/Delegation/Authority
HarnessSpec/Policy
Evaluation/Certification
Solution/Dream Factory
```

如果这些也消失，Morn 就只是外部 Runtime 的 UI 壳。

## 49.5 更严格的竞争边界

最新开源项目进一步缩小了“可宣称差异化”的空间：

```text
Golutra       → 多 CLI Agent 编排与实时执行现场
AgentSpace    → Human + Agent Workspace、数字员工、Harness Router、权限治理
Commonly      → 跨 Runtime 团队空间、Memory、任务板
Tutti         → Agent Ops as Code、Gate、Artifact、审计、Factory Floor
YonCode       → 企业 Harness 六层工程
YonOnto/LOM   → 企业本体/业务语义
AI BaaS       → 结果托管交付
```

因此 Morn 必须坚持：**把真实工作从定义、授权、组织、执行、验证到结果回流完整打通**，而不是继续拼组件数量。

# 50. 参考竞品/研究反向工程矩阵

| 参考 | 可观察设计模式 | Morn 吸收 | 明确不照搬 |
|---|---|---|---|
| Huawei Industry AI Foundry | Agent平台之上沉淀行业 Workflow/Skill/Model/Data/Ontology/Tool | Dream Factory 四工厂、行业资产复用 | 不自建华为级云/算力生态 |
| Huawei Agentic Infra | Token、持续学习、调度、安全自治 | Resource Policy/Provider Adapter | 不自建 GPU/Token Factory |
| Quick BI AIPro | 企业语义、数据广场、Skill、MCP、分析产物、组织复用 | Analytics Plane、Insight→Decision 链 | 不重做 BI 引擎 |
| Siemens Eigen | 深度嵌入真实工程项目、校验、端到端任务 | Native Workspace Adapter、Verification | 不重做 TIA |
| Siemens/NVIDIA Digital Twin | 虚拟调试、工厂模拟、机器人训练 | Simulation Provider | 不自建通用物理仿真 |
| Palantir Ontology | Object/Link/Action/Function/Security 运营层 | Operational World | 不复制封闭数据平台 |
| Celonis Context Model | 实际流程 + 业务知识 + 动态运营上下文 | Process Twin、event feedback | 不自建全套 process mining |
| Fabric IQ | 共享企业 Ontology/语义 | Semantic Provider | 不锁 Microsoft 生态 |
| Workday ASOR | Agent system of record / blended workforce | Workforce SoR | 不只管理 Agent |
| Microsoft Agent 365 | Agent registry/control plane/identity/governance | Console/identity | 不锁 Microsoft Runtime |
| ServiceNow Autonomous Workforce | AI specialist scope/authority/governance | Harness/Role/Autonomy | 不只在 ITSM/Now 生态 |
| Oracle Agentic Applications | 结果导向 Agent 团队+业务对象+流程+政策 | Solution 模型 | 不绑定 Fusion 数据模型 |
| SAP Joule Studio | 意图→PRD/spec/code/test/preview | Solution Compiler 参考 | 不把 code-gen 当全部 solution |
| Golutra | 多 CLI Agent、并行执行、长程 Workflow、实时执行现场；BSL 1.1 | CLI Harness Provider、Workcell Live View、Attention | 不复制受限制源码；不把软件工程场景当全部 Morn |
| AgentSpace | Human + Agents 同 Workspace；AgentRouter 跨 Claude/Codex/OpenClaw/Hermes；Owner/Approval/Governance | HarnessBinding、数字员工Board、权限与跨Runtime身份 | 不把“Human+Agent空间”当独有差异化 |
| Commonly | Pod、Human+跨Runtime Agent、Memory/Task Board/Marketplace | 跨Runtime协作、共享Workspace参考 | 不复制其 chat/task-centric 产品边界 |
| Tutti | `tutti.toml` org code、Roles/Runtime/Workflow/Gate/Policy、Artifact、Factory Floor | Work System as Code、Gate/Record、执行现场 | 不只服务 Coding SDLC |
| Yonyou BIP / YonCode | Harness六层工程；确定性治理；长程开发交付 | HarnessSpec 六层、版本化、Recovery | 不绑定 BIP 元数据/ERP 生态 |
| YonOnto / YonLOM | 企业实体/关系/规则/本体驱动业务理解 | Domain/Enterprise Ontology Provider | 不把 Ontology 等同 Operational World |
| YonClaw | 企业经营智能执行、身份/角色/组织/权限/流程状态 | Organization-aware Harness/Action | 不复制超级Agent入口 |
| Yonyou AI BaaS | 客户从全流程操作转为最终结果验收 | OutcomeContract、Work-as-a-Service | 不把Morn变成单一财务外包平台 |
| Alibaba digital-agent/digital-twin materials | 企业Agent强调独立身份、组织位置、权限、审计与生命周期 | ActorOrigin、RepresentationContract、Memory ownership | 媒体材料不等同完整官方产品规范 |
| DeepSeek Harness | Everything is a Plugin；Agent Loop/Tool/Session/Model 等均可替换；Capability Seam；model-visible implies recorded | 首个重点 Harness Provider、Capability Seam、Event/Scope 不变量 | 不把 DSH Session 当 Morn Source of Truth；不 fork 成整个 Morn |
| Cordis | Spatial/Temporal Composability；revertible effects；reactive coeffects；动态配置/HMR | Capability Fabric 的 effect/coeffect 与 lifecycle 参考 | 不把现实动作误作可逆；API 未稳定时不硬绑定 |
| Siemens Intelligence Center X | 数据+流程+AI Agent+Human hybrid workforce；在既有系统之上治理执行 | Digital Factory Control Plane、混合 workforce、Domain Penetration | 不锁 Siemens 生态；不复制全部工业软件 |
| Siemens Industrial AI / Eigen | 工程 Agent 端到端计划/执行/验证；Industrial AI OS 与 Digital Twin | 深场景 WorkPackage、Verification、Replacement Ladder | 不以通用 Agent 替代工业验证/真实项目上下文 |
| OpenFang | Rust Agent OS、本地、Memory、Skill、MCP/A2A | Native Runtime 工程参考 | 不与其拼 Agent OS 功能量 |
| AIOS | Agent Kernel 资源抽象 | Kernel/Runtime 边界 | 不把上层组织塞入 Kernel |
| openJiuwen | Agent Core/Studio/MCP/A2A/Swarm | Runtime Adapter | 不硬绑定其对象模型 |
| Hermes | 长期个人 Agent、Skill、Profile、协作 | Personal Runtime/experience | 不宣称“会成长”是独有 |
| Paperclip | Agent 公司、组织图、目标、预算、治理 | Org/goal/budget 参考 | 不要求所有员工都是 Agent |
| Xpert UOSE | Enterprise object/action/policy/approval/evidence | Operational action 参考 | 不只面向企业数据语义 |
| TrustGraph | Portable Context Core | Context Core | 不把 context 当运行世界 |
| Temporal/Restate/DBOS | Durable execution | 后端执行适配 | 不重造完整分布式引擎 |
| W3C PROV | Entity/Activity/Agent provenance | Provenance 标准 | 增加 Work/Decision/Outcome 语义 |
| AgentFactory | 成功经验沉淀为可执行子Agent资产 | Capability evolution | 不让自演化绕过认证 |
| Harness research 2026 | Harness本身影响成本/性能；稳定约束应进入代码/Schema/Validator；开始出现Harness自优化 | HarnessSpec/Eval/Release lifecycle | 不允许生产Harness无验证自改 |
| PlanBench-XL | 长程工具规划脆弱性 | Long-horizon recovery/eval | 不假设模型越来越强就自动解决 |
| Anthropic AI Organizations | 组织级行为≠单Agent行为 | Org-level eval | 不把多Agent数量当能力 |
| Robin | 半自主科学发现，人类湿实验 | BioLab human-in-loop | 不宣称全自主实验室 |
| AutoScientists/InternAgent | 计算科研多Agent、自主探索/复现 | Research Workcell | 不取代实验治理 |
| XScientist | Git-like research artifact/provenance | Scientific Digital Thread | 不只保存论文结果 |

# 51. 开发路线：完整参考架构 vs 近期工程

## Phase 0：Architecture Freeze

冻结核心词：

```text
Principal
Workspace
Object
Event
State
Action
Artifact
Decision
Outcome
Actor / ActorOrigin
RepresentationContract
RoleSlot
Commitment
Delegation
WorkPackage / WorkContract
AcceptanceSpec / OutcomeContract
ExecutionMode
HarnessSpec / HarnessBinding
Workcell
Solution
```

同时冻结 Memory Ownership 与 Work System Manifest。禁止继续增加没有对象归属的新宏大模块。


## Phase 0.5：DeepSeek Harness / Cordis Architecture Spike

在正式扩写 Agent Runtime 前只做小型验证：

1. DSH 能否作为独立 Provider 启动并被 Morn 管理；
2. 每个 Workcell/Actor 能否拥有隔离 Scope；
3. WorkPackage/Artifact Context 能否注入但仍由 Morn 记录来源；
4. Tool Call 能否强制经过 Morn Action Gateway；
5. DSH Session Event 能否归一化为 Morn ExecutionEvent；
6. Provider/Plugin 卸载是否正确清理 E0 lifecycle effects；
7. 替换 Agent Loop / Model Provider 后，Morn World/Work/Artifact 是否完全不变；
8. Cordis 是否适合承担 `morn-harness` 的 Provider Lifecycle/Scope/Effect Tracking，而不侵入 Semantic Kernel。

通过后才冻结 `DeepSeekHarnessProvider` 与 `CapabilitySeam v0.1`。

## Phase 1：Morn Work Kernel

实现：

```text
Principal / Identity
Workspace
ObjectType/Object
Relation
Event
StateSnapshot
ActionType/Action
Artifact/Version
DecisionPackage
Outcome
Policy
Ledger
Memory Ownership metadata
```

验收：任何正式状态变化都能追溯；Artifact 不可原地覆盖；Memory 归属明确。

## Phase 2：Harness Fabric + DeepSeek Harness Provider + Controlled Action

实现：

```text
HarnessSpec/Version
HarnessBinding
CapabilityDefinition / Provider / Consumer
Scope Tree
Effect Classification E0-E3
DeepSeekHarnessProvider
CLI Harness Adapter contract
MornNativeHarness(minimal fallback)
Context Builder
Action Gateway
Policy / Approval
Execution Event Normalizer
Execution Receipt
```

验收：Runtime/Harness 无法直接写 Canonical State；E0 可 lifecycle cleanup、E1 可 rollback、E2 有 compensation、E3 强制风险门；同一 Actor 能在 DSH 与第二个 Harness 之间切换且正式身份/Work/Artifact 不变。

## Phase 3：Mixed Organization & Representation

实现：Actor/Human/Worker/Service/Device、RoleSlot、ResponsibilityBinding、ActorOrigin、RepresentationContract、Delegation、Commitment、Accountability。

## Phase 4：Durable Execution

Checkpoint、Pause/Resume、Signal、Drift、Replan、Compensation、Escalation、Budget、Attention Queue。

## Phase 5：Organization/Solution Compiler v0.2

```text
Goal
→ ProblemSpec
→ Operational Objects
→ WorkGraph
→ WorkPackage / WorkContract
→ Acceptance / OutcomeContract
→ ExecutionMode
→ RoleSlots
→ Member Types
→ Capability Gaps
→ HarnessBinding
→ Workflow
→ Evaluation Plan
```

先生成可审查方案，不立即自动部署。

## Phase 6：Simulation & Evaluation

历史回放、Tool/Harness故障、权限攻击、错误知识、Representation越界、专家意见稀释、数据异常、回归和影子运行。

## Phase 7：BioLab Dream Factory v0.1

只做三个闭环：

```text
文献→证据→假设→实验设计
数据→质控→分析→独立复核→科学主张
批准结果→图表→论文→一致性检查
```

湿实验真人执行。

## Phase 8：Evolution Engine v0.1 + Asset/Harness Flywheel

先实现受限 Evolution：

```text
Trace / Process / Outcome
→ Pattern / Bottleneck / Repetition
→ Candidate Skill / Workflow / Harness Patch
→ Historical Replay
→ Evaluation
→ Shadow
→ Certification
→ New Solution Version
```

暂不自动重构真实岗位或退役软件，只产生可审查 Candidate。

## Phase 9：Managed Work / Outcome Delivery

先在 BioLab 内部或小范围客户场景验证：

```text
Certified Work Capability
→ Managed Workcell
→ OutcomeContract
→ DeliveryReceipt
→ Customer Acceptance
```

不急于商业化，但底层对象必须支持未来 Work-as-a-Service。

## Phase 10：Domain Penetration / Replacement Pilot

选择一个已经稳定运行的具体工作场景，执行：

```text
Observe/Connect
→ Orchestrate
→ Morn-native Candidate
→ Shadow Replace
→ Value Validation
→ Partial Replace
```

目标不是替整个 ERP/MES，而是验证至少一个传统 Workflow/岗位职责/软件功能能够被 Certified Work Capability 正式替代并可回滚。

## Phase 11：Operational World L2/L3

在有真实 Outcome 数据之后，再做预测、因果、反事实和策略优化，并让 Evolution Engine 用于多候选策略比较。

# 52. 近期 P0/P1/P2 范围

## P0 必须完成

- Kernel/Workspace/Identity；
- Operational World L0；
- Artifact/Decision/Outcome；
- Action Gateway；
- ActorTemplate/Instance；
- `ActorOrigin` Schema；
- `HarnessSpec + HarnessBinding` Schema；
- `CapabilitySeam + EffectClass` Schema；
- DSH/Cordis Architecture Spike；
- `ExecutionMode`；
- RoleSlot/MemberBinding；
- WorkPackage/Acceptance/OutcomeContract Schema；
- Memory Ownership Scope；
- Work System Manifest v0；
- 基础 Durable Checkpoint；
- Evaluation Hook；
- BioLab 最小 Domain Schema。

## P1

- Delegation/Accountability；
- RepresentationContract；
- DecisionPolicyAsset；
- Workcell；
- `DeepSeekHarnessProvider` + 至少另一个 CLI Harness Adapter；
- EvolutionCandidate / Branch / Promotion 基础 Schema；
- Organization Compiler；
- Historical Replay；
- Attention Queue / Workcell Live View；
- Hub 新资产模型。

## P2

- Process Intelligence feedback；
- Analytics Provider；
- 企业 Durable Runtime；
- Morn Team/Private/Edge Node；
- Managed Work / Outcome Delivery；
- 仪器/机器人；
- 更强 Simulation；
- Predictive World；
- 多行业 Dream Factory；
- Domain Penetration / Replacement Pilot；
- Evolution Engine（Role/Software/Organization Candidate 级）。

# 53. 验收标准

## 53.1 跨场景不改 Kernel

同一个 Kernel 可创建 Personal、BioLab、Pharma Work System。

## 53.2 Runtime 可替换

Actor 切换 Runtime 后身份/Workspace/Artifact/Role/历史不丢。

## 53.3 Artifact/Decision/Outcome 闭环

至少跑通：

```text
Actor A 产生 Artifact
→ Reviewer 审查
→ Human 批准
→ Action 执行
→ State Diff
→ Outcome
→ 下游 Actor 只读取 Approved 版本
```

## 53.4 WorkPackage 验收

每个 WorkPackage 有 AcceptanceSpec，不能只靠“Agent说完成”。

## 53.5 Durable Recovery

关闭 Morn 后重启可继续；Tool失败可恢复；人工审批等待数小时/数天后可继续。

## 53.6 Organization Compiler 可解释

任何自动生成岗位或成员选择都能显示：依据、规则、Domain资产、模型、置信度、人类批准状态。

## 53.7 BioLab 真实可复现

分析结果可从锁定数据、代码、环境和参数重新运行；论文主张可追到 AnalysisRun 和 Dataset。


## 53.8 Harness Provider 中立

同一 WorkPackage 在 `DeepSeekHarnessProvider` 与第二个 Harness Provider 上运行时，Morn 的 Identity、WorkContract、Canonical World、Artifact/Decision Schema 与 Outcome 语义保持不变。

## 53.9 Governed Evolution

任何 EvolutionCandidate 都不能直接修改 Production；至少完成 Branch + Evaluation + Promotion Decision。高风险变化必须经过 Shadow/Human Gate；新版本必须可回滚。

## 53.10 Replacement Pilot

至少一个实际工作从 `Existing System/Manual Workflow` 进入 Shadow Replace，并能以统一 Acceptance/Outcome 指标证明是否适合 Partial Replace，而不是只比较 Agent 输出文本。

---

# 54. 安全与治理

## 54.1 Zero-Trust Runtime

任何 Runtime 默认不拥有 Secret 和广泛写权限。

## 54.2 Least Privilege

权限粒度至少到：Workspace + Resource/Object + Action + Context。

## 54.3 Prompt Injection

外部网页/文档属于 Untrusted Content，不得直接改变系统 Policy 或授权。

## 54.4 Irreversible Action

不可逆、高风险、现实世界动作默认 Human Approval；只有经过 Certified Work Capability 才能逐步提高自治。

## 54.5 Kill Switch

Actor、Workcell、Solution、Connector 都应支持独立冻结/终止。

## 54.6 证据保留

保存结构化决策与执行证据，不把未经授权的敏感数据或模型私有推理无限保留。

---

# 55. 成本与资源治理

每个 WorkPackage 记录：

```text
Token budget
Model cost
Tool/API cost
Compute cost
Human review cost
External service cost
Device time
Elapsed time
```

每个 Workcell 可以计算：单位交付成本、人工返工率、成功率、失败恢复成本。

Dream Factory 的资产选择不能只看“模型能力”，还要看：

```text
Quality × Reliability × Latency × Cost × Risk × Maintainability
```

---

# 56. Anti-goals：明确不做

近期明确不做：

- 从零训练通用大模型；
- GPU/Token Factory；
- 全功能湖仓/BI；
- 全功能 LIMS/MES/ERP；
- 全功能数字孪生物理引擎；
- 全功能流程挖掘平台；
- 所有行业同时建设；
- “完全自治公司”；
- 自由文本 LLM 直接控制实验/工业设备；
- 让 Agent 自行修改生产 Skill/Policy/Harness/Organization；
- 把 Cordis/DeepSeek Harness 的插件生命周期可逆性误写成真实业务动作可逆；
- 每个客户 Fork 一套核心代码；
- 用大量 Agent 数量代替正确的工作结构；
- 为“取代软件”而从零重写所有 ERP/MES/APS/LIMS；
- 把 Evolution 等同于更多 Agent、更多 Token 或无约束自我复制；
- 在没有预测/反事实能力前夸大“世界模型”。

---

# 57. 主要风险与应对

## 范围爆炸

应对：任何新能力必须归入现有原语/Plane；优先 Work Kernel + BioLab。

## 与大厂重叠

应对：不和大厂拼基础设施，专注跨底座 Work & Organization Control Plane。

## Compiler 看起来聪明但实际胡编

应对：DomainPack、模板、显式规则、Decision Record、Capability Inventory、Simulation、Human Approval。

## 多 Agent 错误放大

应对：最小充分 Workcell、Expertise routing、独立 Review、Artifact Contract、Schema/Policy。

## 行业资产不专业

应对：专家评审、真实案例、认证、明确适用范围与失败模式。

## Runtime 锁定

应对：Portable Actor + Adapter + Morn owns canonical state。

## Memory 污染

应对：Memory Gate、来源、Workspace、置信度、过期、用户可见、Artifact 优先。

## 自治过快

应对：Autonomy ladder + Shadow/Canary + 证据化升级。


## DSH/Cordis 上游变化

应对：DSH 当前 Developer Preview、Cordis API 未稳定；用 Provider API、contract tests、vendor-neutral event schema 隔离，上游只作为执行实现，不拥有 Morn Canonical State。[R51][R53]

## Evolution 漂移或“优化错目标”

应对：Outcome 指标不能单一化；保留安全/合规/人类负担/质量/成本多目标约束；所有 Candidate 先在 Evolution World 验证；高风险结构变化需要独立 Review。

## 软件吸收导致隐藏依赖丢失

应对：Replacement 之前先做 Process/Dependency Mining、Shadow、长周期 Outcome 对比与 rollback plan；不能因为表面功能相同就直接退役旧系统。

---

# 58. 用户提供材料与重点启发

## U01：Quick BI AIPro（2026-08-04 截图）

截图明确强调：从数据接入到分析交互以 AI 为底座重新设计；新增智能匹配数据源、企业语义理解、MCP 连接、组织化复用、数据溯源；目标从个人敏捷走向组织协同。

**Morn 吸收**：Analytics Plane、企业语义 Provider、Insight→Decision→Action→Outcome、个人资产到组织资产的认证门。  
**Morn 不照搬**：不自建 BI、指标引擎和报表平台。

## U02：智东西《工业AI落地最后一公里，不是部署AI，是敢把任务交给AI》

关键启发：工业 AI 的分水岭是是否可以把边界清晰、有验收标准的完整任务交给 AI，而不是是否部署了聊天式 Agent；Eigen 与真实工程项目、TIA Portal、项目标准、验证和人类部署授权结合。

**Morn 吸收**：WorkPackage、AcceptanceSpec、Workcell、Native Workspace Adapter、Verification Stack。

## U03：华为行业 AI 梦工厂相关材料

关键启发：Agent 平台之外，还需要标准化行业资产、伙伴生态、验证交付和项目经验沉淀。

**Morn 吸收**：Dream Factory 四工厂、DomainPack、Certified Work Capability。

## U04：百型智能 OntoZ / 企业本体相关材料

百型官方当前公开强调企业本体、共享共识、数据/逻辑/行动三层、智能体矩阵以及基于业务结果优化。[R34]

**Morn 吸收**：Operational World、共享企业/项目状态、Outcome feedback。  
**Morn 保留边界**：不把商业宣传中的“真实世界模型/因果”等词自动等同于已被独立验证的通用世界模型。

## U05：Golutra

用户提供 GitHub：`https://github.com/golutra/golutra`。

可观察方向包括多 CLI Agent 编排、并行执行、长程 Workflow、实时执行现场以及将 Codex/Claude Code/OpenClaw 等统一纳入工作空间。[R41]

**Morn 吸收**：

- CLI Harness 成为一等 Provider；
- `HarnessBinding + RuntimeBinding`；
- Workcell Live View；
- Attention Queue；
- 尽量保留用户已有 CLI/Runtime，不强迫迁移。

**许可注意**：Golutra 当前公开仓库使用 Business Source License 1.1，并声明未来变更许可证；可研究设计模式，但不得在未完成许可证评估的情况下直接复制其源码进入 Morn。[R41]

## U06：阿里云“企业数字分身”相关媒体材料

用户提供 DoNews 链接。该材料强调企业 Agent 与个人 Agent 的差别：组织内数字主体需要身份、组织位置、负责人、权限、审计、Memory 和生命周期。

**Morn 吸收**：

```text
ActorOrigin
RepresentationContract
DecisionPolicyAsset
Memory Ownership
Organization-native Actor lifecycle
```

**边界**：该链接是媒体材料，不自动视为阿里云完整产品规范；与阿里云现有 Agent ID Guard 等官方能力可相互印证“Agent 独立身份、最小权限、审计”正在成为基础设施。[R48]

## U07：用友 BIP 6 / YonCode / YonOnto / YonClaw / AI BaaS

用户提供 TechSir 文章，并进一步用用友官方资料核对。

**关键启发一：智能双模**  
确定性工作优先传统程序/流程，概率判断和复杂语义才进入 Agentic 执行。

**关键启发二：Harness 六层工程**  
信息边界、工具系统、执行编排、记忆与状态、评估与观测、约束校验恢复。[R46]

**关键启发三：本体不是全部 Operational World**  
YonLOM/YonOnto 强化企业实体、关系、规则和语义；Morn 继续增加 Event、State、Action、Outcome 才构成运行世界。[R45]

**关键启发四：结果交付/BaaS**  
企业可以把全链路工作托管出去，只验收最终闭环结果。[R47]

**Morn 吸收**：ExecutionMode、HarnessSpec、Work-as-a-Service、OutcomeContract、Morn Node、Solution Delivery Lifecycle。

## U08：AgentSpace / Commonly / Tutti 扩展检索

本轮扩展发现：

- AgentSpace 已支持 Human + Agents 同 Workspace、数字员工角色/Owner/Skill/Knowledge、跨 Claude Code/Codex/OpenClaw/Hermes 的 AgentRouter Harness、权限和审批；[R42]
- Commonly 支持 Human + 跨 Runtime Agent 的 Pod、Memory、Task Board 和自托管；[R43]
- Tutti 将 Role/Runtime/Workflow/Gate/Policy 写成 `tutti.toml`，强调可版本化的“org code”和 Factory Floor。[R44]

因此以下能力不能再单独宣称为 Morn 独有：

```text
本地优先
Human + Agent Workspace
多 Runtime
跨 Runtime 身份连续
Owner/Approval/Audit
Agent Ops as Code
```

Morn 的竞争焦点必须继续上移到：

> **真实工作合同 + 组织责任 + Operational World + 证据/决策/执行/结果闭环 + 行业认证交付。**


## U09：DeepSeek Harness / Cordis（2026-08-13~14）

用户提供 DeepSeek Harness 截图并提出“时空可组合、effect/coeffect、Runtime 自修改、是否可作为 Morn 基础”的问题。本版以 DeepSeek 官方 GitHub/架构文档和 Cordis 官方仓库/论文为准，正式吸收：

```text
Everything is a Plugin
Capability Seam
Scope
Persistent Event
Model-visible → Recorded
Effect / Coeffect
Spatiotemporal Composability
```

同时明确边界：Morn 采用 Stable Semantic Kernel + Dynamic Capability Fabric，不把 Work/World/Authority 插件化；DSH 是重点 Harness Provider，Cordis 是实验性 Fabric 候选，而非业务语义地基。[R51][R52][R53][R54]

## U10：Siemens Industrial AI / Digital Factory 路线

用户补充《西门子向工业AI，再进一步》及相关赛意/工业AI材料，并明确希望 Morn 不只是“连接”，而是能够创建数字工厂、数字员工、虚拟员工，并在深入场景后逐步替代既有工作、岗位和部分软件。

本版据 Siemens 官方资料把路线收敛为：

```text
Digital Factory
+ Virtual Factory
+ Hybrid Workforce
+ Verified Domain Agent
+ Replacement Ladder
+ Evolution Engine
= Evolving Digital Factory
```

替代单位从“整家公司/整套工业软件”下沉为 WorkPackage、岗位职责、Workflow、软件模块，并通过 Observe → Shadow → Validate → Replace 推进。[R13][R14][R55][R56]

## U11：Morn Evolution / 自衍生与自进化

用户进一步明确：Morn 应能整合传统生产中的多个软件与多岗位员工，并在需要时自行衍生、持续进化。

本版将其从“未来展望”提升为正式设计原则：

- 五环系统中的 Evolution Loop；
- Capability / Workflow / Role / Actor / Software / Organization 六类进化；
- Production World 与 Evolution World 分离；
- Organization Branching；
- Agent → Deterministic Capability 的反向蒸馏；
- Capability Inheritance / Industry Evolution Flywheel；
- Evolving Digital Factory。

# 59. 参考资料

以下优先列官方文档、GitHub 或论文；中文媒体和用户截图主要作为线索发现，不作为唯一架构依据。DeepSeek Harness/Cordis 与 Siemens 新增结论均以官方仓库/文档为主要依据。

**[R01] AIOS — LLM Agent Operating System / Agent Kernel**  
GitHub: https://github.com/agiresearch/AIOS

**[R02] OpenFang — Open-source Agent Operating System**  
GitHub: https://github.com/RightNow-AI/openfang

**[R03] openJiuwen organization / agent-core / protocol / relay**  
GitHub: https://github.com/openJiuwen-ai

**[R04] NousResearch Hermes Agent**  
GitHub: https://github.com/NousResearch/hermes-agent

**[R05] AgentFactory: Self-Evolving Agents through Executable Subagent Assets (2026)**  
arXiv: https://arxiv.org/abs/2603.18000

**[R06] ServiceNow Autonomous Workforce**  
https://newsroom.servicenow.com/press-releases/details/2026/ServiceNow-launches-Autonomous-Workforce-that-thinks-and-acts-adds-Moveworks-to-the-ServiceNow-AI-Platform/default.aspx

**[R07] Oracle Fusion Agentic Applications**  
https://www.oracle.com/europe/news/announcement/oracle-introduces-fusion-agentic-applications-2026-03-24/

**[R08] SAP Joule Studio — intent-based enterprise agentic development**  
https://news.sap.com/2026/05/new-joule-studio-enterprise-scale-agentic-development/

**[R09] Palantir Foundry Ontology / Operational Layer**  
https://www.palantir.com/explore/platforms/foundry/ontology/

**[R10] Microsoft Fabric IQ Ontology**  
https://learn.microsoft.com/en-us/fabric/iq/ontology/overview

**[R11] Celonis Context Model / Process Intelligence**  
https://www.celonis.com/news/press/celonis-launches-the-context-model-to-eliminate-enterprise-ais-operational-blind-spots-agrees-to-acquire-ai-decision-intelligence-leader-ikigai-labs

**[R12] TrustGraph — Context Cores / ontology / provenance / retrieval**  
https://github.com/trustgraph-ai/trustgraph

**[R13] Siemens Eigen Engineering Agent**  
https://press.siemens.com/global/en/pressrelease/siemens-launches-eigen-engineering-agent-bringing-purpose-built-ai-industrial

**[R14] Siemens Digital Twin Composer**  
https://www.siemens.com/en-us/company/digital-transformation/industrial-metaverse/introducing-digital-twin-composer/

**[R15] NVIDIA Omniverse / Digital Twin**  
https://docs.nvidia.com/omniverse/index.html

**[R16] Workday Agent System of Record**  
https://blog.workday.com/en-us/managing-ai-powered-future-of-work.html

**[R17] Microsoft Entra Agent Identity / Blueprint / Sponsor**  
https://learn.microsoft.com/en-us/entra/agent-id/identity-platform/create-blueprint

**[R18] Business World Model: A Platform for Agents to Simulate Strategic Plans (2026)**  
arXiv: https://arxiv.org/abs/2606.10044

**[R19] W3C PROV-O: The PROV Ontology**  
https://www.w3.org/TR/prov-o/

**[R20] XScientist: Toward Reproducible Agent-Native Science (2026)**  
arXiv: https://arxiv.org/abs/2607.12301

**[R21] Apple ML Research — Multi-Agent Teams and Expert Utilization**  
https://machinelearning.apple.com/research/multi-agent-teams-experts

**[R22] Anthropic Alignment Science — AI Organizations (2026)**  
https://alignment.anthropic.com/2026/ai-organizations/

**[R23] Alibaba Cloud Quick BI AIPro Overview**  
https://help.aliyun.com/zh/quick-bi/user-guide/quick-bi-aipro-overview

**[R24] Temporal Durable Execution**  
https://github.com/temporalio/temporal

**[R25] Restate Durable Execution / AI examples**  
https://github.com/restatedev/ai-examples

**[R26] DBOS Durable Workflows**  
https://www.dbos.dev/

**[R27] Huawei Cloud Industry AI Foundry / 行业AI梦工厂**  
https://www.huaweicloud.com/ai/index.html  
金融专区参考：https://www.huaweicloud.com/news/2026/20260718174216756.html

**[R28] Microsoft Agent 365 — Agent Control Plane**  
https://learn.microsoft.com/en-us/microsoft-agent-365/overview

**[R29] Microsoft Agent Governance Toolkit**  
https://github.com/microsoft/agent-governance-toolkit

**[R30] Robin — semi-autonomous scientific discovery (Nature, 2026)**  
https://www.nature.com/articles/s41586-026-10652-y

**[R31] AutoScientists**  
GitHub: https://github.com/mims-harvard/AutoScientists

**[R32] InternAgent**  
GitHub: https://github.com/InternScience/InternAgent

**[R33] Huawei Agentic Infra / Token Factory**  
https://www.huawei.com/cn/news/2026/6/inspire-agenticera-agenticinfra

**[R34] 百型智能 OntoZ 官方**  
https://www.100x-agent.com/zh

**[R35] Paperclip AI — Agent company orchestration**  
https://github.com/agencyenterprise/paperclip-ai

**[R36] Xpert AI / UOSE**  
https://github.com/xpert-ai/xpert

**[R37] PlanBench-XL — Long-Horizon Tool Planning (2026)**  
arXiv: https://arxiv.org/abs/2606.22388

**[R38] DataHub Metadata Platform**  
https://docs.datahub.com/

**[R39] OpenLineage**  
https://github.com/OpenLineage/OpenLineage

**[R40] Nextflow — Reproducible Scientific Workflows**  
https://nextflow.io/

---


**[R41] Golutra — Multi-agent orchestration platform**  
GitHub: https://github.com/golutra/golutra  
官网: https://www.golutra.com/  
许可：仓库当前声明 Business Source License 1.1。

**[R42] AgentSpace — Human + Agents. One Team. One Workspace**  
GitHub: https://github.com/HKUDS/AgentSpace

**[R43] Commonly — humans + cross-vendor AI agents workspace**  
GitHub: https://github.com/Team-Commonly/commonly

**[R44] Tutti — Agent Ops / org code / tutti.toml**  
GitHub: https://github.com/nutthouse/tutti

**[R45] 用友 YonLOM / 企业本体资料**  
https://www.yonyou.com/news/4732  
https://www.yonyou.com/news/4866

**[R46] 用友 YonCode / Harness 六层工程**  
https://yonyou.com/subject/yonbip/news/5149  
https://yonyou.com/subject/yonbip/news/5977

**[R47] 用友银账通 AI BaaS / 结果托管交付**  
https://www.yonyou.com/news/5169  
https://www.yonyou.com/news/5270

**[R48] Alibaba Cloud Agent ID Guard — Agent identity / least privilege / audit**  
https://www.alibabacloud.com/help/zh/doc-detail/3036579.html

**[R49] Harness efficiency / cost research (2026)**  
arXiv: https://arxiv.org/abs/2607.06906

**[R50] Harness engineering / constraint-as-code / self-improvement research (2026)**  
arXiv: https://arxiv.org/abs/2607.08028  
arXiv: https://arxiv.org/abs/2608.02276


**[R51] DeepSeek Harness — official repository**  
GitHub: https://github.com/deepseek-ai/deepseek-harness

**[R52] DeepSeek Harness Architecture / Cordis / Capability Seam / Runtime Invariants**  
https://deepseek-harness.github.io/deepseek-harness/reference/

**[R53] Cordis — Meta-Framework of Spatiotemporal Composability**  
GitHub: https://github.com/cordiverse/cordis

**[R54] A Programming Paradigm for Spatiotemporal Composability — Draft 2026-08-13**  
GitHub/PDF: https://github.com/cordiverse/paper

**[R55] Siemens — Industrial AI Operating System / Digital Twin Composer, CES 2026**  
https://press.siemens.com/global/en/pressrelease/siemens-unveils-technologies-accelerate-industrial-ai-revolution-ces-2026

**[R56] Siemens Intelligence Center X — hybrid workforce / governed industrial AI orchestration**  
https://news.siemens.com/en-us/siemens-intelligence-center-x/

**[R57] Siemens — self-verifying agentic AI workflows for EDA / PCB**  
https://news.siemens.com/en-us/siemens-nvidia-dac-2026/

# 60. 未来展望：从智能工作 OS 到可进化数字组织

本节不是脱离工程现实的“无限自治”愿景，而是建立在前文 Work/World/Outcome/Evolution 契约之上的长期方向。任何能力只有经过真实数据、评测、Shadow 与治理，才从 Future Candidate 升为正式产品能力。

## 60.1 第一阶段：Morn 作为 Work & Organization Control Plane

Morn 连接既有：

```text
ERP / MES / APS / PLM / WMS / TMS
BI / Data / Knowledge
Codex / DeepSeek Harness / Hermes / other Agents
Human / Service / Device
```

主要价值是建立统一 Identity、World、Work、Decision、Action、Outcome 和 Attention。

## 60.2 第二阶段：Digital Workforce / Digital Department

通过经过认证的 WorkPackage/Workcell 把重复专业工作交给 Digital Employee，同时保留 Human Sponsor、Approval、Accountability 与 Exception Handling。

组织从“每个软件一个 Copilot”转向“一个 Outcome 对应一个跨系统 Workcell”。

## 60.3 第三阶段：Digital Lab / Digital Factory

Morn 建立完整 DomainPack、Operational World、Process/Asset/Organization/Workforce Twin、专业 Solver/Model/Device Adapter 与 Verification Stack，使整个实验室/工厂能够在统一 Work OS 下运行。

## 60.4 第四阶段：Native Capability 与软件消融

Morn 不再只调用外部应用，而是根据长期真实 Work Graph 和 Outcome 逐步形成 Native Capability：

```text
Observed Manual/Software Work
→ Candidate Capability
→ Shadow
→ Certified Native Module
→ Partial Replacement
→ Legacy Retirement
```

长期保留的通常是交易、物理和监管基础设施；大量人机界面型、协调型、分析型、流程型中间软件可能被统一到 Morn Work System。

## 60.5 第五阶段：Evolving Digital Organization

数字组织能够：

- 发现新 Capability Gap；
- 生成候选 Skill/Workflow/Actor/Program；
- 发现无价值协调和重复审批；
- 在 Virtual Organization/Factory 中比较组织分支；
- 动态调整 Workcell 与自治等级；
- 把成熟 Agent 工作确定化；
- 把有效站点能力抽象为 Domain Asset；
- 根据真实 Outcome 选择下一版本。

此时“组织版本”像软件版本一样可审计：

```text
Factory v1.0
→ v1.3 Quality Digital Worker
→ v2.0 Native Planning Capability
→ v2.4 Robot Binding
→ v3.0 Organization Redesign
```

## 60.6 Morn Manufacturing / BioLab / Pharma 等不是独立 Kernel

未来可形成：

```text
Morn Manufacturing
Morn BioLab
Morn Pharma
Morn Finance
Morn Personal
```

但它们共享同一个 Stable Kernel；差异来自 Domain Model、WorkPackage、Capability、Harness、Verification、Data 与 Outcome History。这样才能避免每进入一个行业就重新造平台。

## 60.7 真正的长期壁垒

Kernel 代码本身并不是最强壁垒。真正越来越难复制的是：

```text
Stable Kernel
× Domain Operational Model
× Real Work Contracts
× Connector / Harness Integration
× Failure & Correction Data
× Evaluation / Certification
× Production Outcome History
× Replacement Experience
× Evolving Domain Assets
```

## 60.8 终局不是“完全自治”，而是“可治理地改变自己”

Morn 的长期上限不由“Agent 是否像人”决定，而由系统能否：

1. 正确表示真实世界和工作；
2. 对行动、责任与证据保持可追溯；
3. 在虚拟/影子环境中产生和比较替代方案；
4. 只把经过验证的改进推进生产；
5. 在环境变化后继续重构自身工作结构。

因此最重要的未来能力不是 unrestricted self-modification，而是：

> **Governed Organizational Evolution：组织能够改变自己，同时保持可验证、可回滚、可追责。**

# 61. 最终架构结论

Morn 不应再被解释成：

```text
“给模型装上记忆、工具和技能，再把多个Agent放在一起。”
```

也不应被解释成：

```text
“把Codex、Claude Code、Hermes、OpenClaw统一到一个桌面里。”
```

后者很有价值，但 Golutra、AgentSpace、Commonly、Tutti 等已经证明它会快速成为基础设施能力。[R41][R42][R43][R44]

Morn 应被理解为：

```text
真实世界/项目状态
        ↓
目标与问题
        ↓
WorkPackage / Work Contract
+ AcceptanceSpec / OutcomeContract
        ↓
ExecutionMode
        ↓
动态 Workcell
Human + Actor + Program + Service + Device
        ↓
HarnessSpec + RuntimeBinding
        ↓
受治理 Action
        ↓
Artifact + Decision + Execution Record
        ↓
Verification + Delivery Receipt
        ↓
Actual Outcome
        ↓
Operational World 新状态
        ↓
Dream Factory 资产与交付能力回流
```

Morn 的长期核心不是“拥有多少 Agent”或“支持多少 Runtime”，而是：

> **能否把一个人的目标或一个组织的问题，编译成一套可解释、可授权、可执行、可验证、可交付的智能工作系统，并让这套系统在不破坏责任、证据和安全边界的前提下，根据真实 Outcome 持续生成更好的能力、流程、岗位、软件映射和组织版本。**

因此，本版最终定义为：

> **Morn 是可进化的智能工作与混合组织操作系统，也是行业 AI 梦工厂的生产、交付与演化底座。它不要求第一天取代所有专业软件，而是先把数据、模型、Agent/Harness、Workflow、Ontology、Process Intelligence、数字孪生、行业软件、人类和设备组织成可认证工作能力；随着真实运行、验证和 Outcome 累积，再通过 Replacement Ladder 把部分工作、岗位职责、Workflow 和软件模块逐步吸收到 Morn-native Capability 中。**

最终差异化公式收敛为：

```text
Morn
=
Stable Semantic Kernel
× Operational World / Work Graph
× Work Contract
× Mixed Workforce
× Governed Composability / Harness-neutral Execution
× Decision & Evidence
× Verified Delivery
× Outcome
× Evolution Engine
× Work System as Code
× Certified Work Capability
× Dream Factory
```

第一阶段的成败标准不是“做出最多功能”，而是完整打通：

```text
Morn Work Kernel
+ Operational World L0/L1
+ WorkPackage / WorkContract
+ Artifact / Decision / Outcome
+ HarnessSpec / CapabilitySeam / HarnessBinding
+ DeepSeekHarnessProvider + MornNative fallback
+ Controlled Action / EffectClass
+ Mixed Organization
+ Durable Workflow
+ Organization Compiler
+ Governed Evolution v0.1
+ BioLab Certified Work Capability
```

当这条链在真实项目中稳定运行，Morn 才真正从 Agent Builder 跨越到**可信智能工作的定义、组织、执行与交付标准**。

