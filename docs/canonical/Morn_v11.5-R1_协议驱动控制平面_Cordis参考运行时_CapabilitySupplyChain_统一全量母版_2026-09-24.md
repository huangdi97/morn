# Morn v11.5-R1 统一全量母版

## 协议驱动工作控制平面 + Cordis 参考组合运行时 + Capability Supply Chain + Domain Profiles

**日期：2026-09-24**  
**状态：Design Convergence / Proposed Canonical Master for Review（架构收敛母版，尚非生产批准）**  
**直接继承：**《Morn v11.4-R1｜Microkernel + CapabilitySpec + HarnessProvider 全量架构收敛母版（2026-09-17）》  
**上游规范基线：**《Morn 全量设计母版 v11.3-R1（2026-09-13）》  
**历史工程事实与证据账本：**《Morn 全量整合母版｜截至 2026-08-31》  

**本版目的：**在 v11.4 的 Work-first、Capability-first、Outcome-centric、Harness-neutral 基础上，正式吸收 Cordis / DeepSeek Harness、Paper2Agent、DeepSeek Elastic Compute（DSec）、Learnware、Kubernetes Controller Pattern、Temporal / Dapr Durable Execution、CloudEvents、OpenTelemetry、OPA / Cedar、OCI / ORAS、Sigstore / SLSA、SPIFFE、MCP / A2A 与 WebAssembly Component Model 等公开体系带来的新证据，把 Morn 从“极轻 Microkernel + Provider”进一步收敛为：

> **Morn = Semantic Specification + Durable Work Control Plane + Reference Composition Runtime + Provider Fabric + Capability Supply Chain + Domain Profiles。**

其中，**Reference Composition Runtime 第一阶段直接采用 Cordis；DeepSeek Harness 与 Pi 作为独立 Harness Provider；Morn 不重造通用插件框架、Agent Harness、Sandbox、Policy Engine、Workflow Engine 或 Package Registry。**

---

# 0. 阅读规则、继承规则与证据语言

本版继续严格区分四类表述：

- **[继承]**：来自 v11.3、v11.4 或 2026-08-31 历史工程事实账本，继续有效但按本版重新解释和组织。
- **[研究]**：来自截至 2026-09-24 的外部公开资料，仅支持对应事实，不等于 Morn 已实现。
- **[新裁决]**：本版依据前述讨论与研究形成的目标架构/产品决策。
- **[待验证]**：必须通过真实代码审计、独立测试、客户授权数据、影子运行、现场结果或商业验证才能确认。

## 0.1 Canonical 继承规则

1. v11.4-R1 是本版直接设计基线；v11.3-R1 继续作为上游规范依据。
2. 2026-08-31 母版只承担历史工程事实、RV/TF01/LC01/BIOFERM 证据、Brownfield 路线和旧实现追溯，不因“旧内容未冲突”而自动成为现行规范。
3. v11.5 不允许通过文档修改历史工程成熟度。新增 `ControlPlane`、`CordisRuntimeAdapter`、`CapabilityPackage`、`Controller` 等概念，不代表代码已经存在。
4. 旧对象若进入 v11.5 运行面，必须显式映射为：`Spec/Status Resource`、`Controller`、`Provider`、`CapabilityPackage`、`Domain Profile` 或兼容层。
5. 任何客户生产连接、工业写入、质量放行、安全控制、付款、采购、外部承诺仍需独立授权与现场验证。

## 0.2 当前工程状态不得被架构文档升级

继续保持：

```text
LIVE_CUSTOMER_DATA_RECEIVED  != proven by this document
LIVE_FACTORY_READ_ONLY       = EXTERNAL_BLOCKED unless later revalidated
LIVE_SHADOW_OUTCOME          = EXTERNAL_BLOCKED unless later revalidated
LIVE_ATTRIBUTION             = EXTERNAL_BLOCKED unless later revalidated
REAL_PRODUCTION_WRITE        = NO unless separately authorized and verified
CONTROLLED_REAL_WRITE        = NO unless separately authorized and verified
COMMERCIAL_MATURITY          = NOT_CLAIMED
```

历史上的 Core / Factory / RV / TF01 / LC01 / BIOFERM 工程证据继续有效，但不得用本版目标设计倒推“当时已经实现”。

---

# 1. 本版最高层裁决

## 1.1 五句总原则

> **Work is the unit of coordination.**  
> **Capability is the unit of composition.**  
> **Accepted Outcome is the unit of value.**  
> **Agent is one kind of executor.**  
> **Everything may evolve; nothing may silently rewrite a committed fact or contract.**

中文固定为：

> **以工作作为协调单元；以能力作为组合单元；以被接受的真实结果作为价值单元；智能体只是执行器的一种；一切都允许演进，但已生效的事实与契约不得被静默改写。**

## 1.2 一句话产品定义

**Morn = protocol-driven, outcome-oriented Work & Operations Control Plane。**

中文：

> **Morn 是一个协议驱动、面向真实结果的工作与运营控制平面，用统一契约把人、智能体、模型、规则、求解器、软件系统与机器组合成可治理、可持续、可验证、可验收的真实工作。**

## 1.3 Morn Factory 定义

> **Morn Factory = 运行在 Brownfield 制造企业现有 ERP / MES / QMS / CMMS / APS / WMS / Historian / SCADA / DCS 等系统之上的制造业 Work & Operations Domain Profile + Solution Packages。**

它不是 MES、APS、SCADA、PLC、数字孪生或“智能工厂总平台”的替代物。

## 1.4 本版明确不再把 Morn 定义成什么

- 不是 Agent Harness；
- 不是单一 Agent Runtime；
- 不是“智能体超级工厂”本身；
- 不是数字员工平台本身；
- 不是 Ontology 平台；
- 不是 Workflow Engine；
- 不是 Sandbox 平台；
- 不是 IAM / Policy Engine；
- 不是 Package Registry；
- 不是 MES / APS / RPA / BPM 的替代物；
- 不是“所有东西都由一个超级 Agent 决定”的系统。

---

# 2. 从 v11.4 到 v11.5：真正发生了什么变化

v11.4 的核心方向仍正确：Work-first、Capability-first、Harness-neutral、Outcome-centric、Brownfield-first。但有三个需要修正的地方。

## 2.1 `Microkernel` 不再是最高层定义

v11.4 容易产生一个误解：

```text
Morn Microkernel
  ├ Work
  ├ Authority
  ├ Receipt
  ├ Outcome
  └ Acceptance
```

看起来这些是必须由 Morn 自己写死的“不可替代模块”。

v11.5 改为：

```text
Morn Semantic Specification
      ↓
Morn Durable Control Plane
      ↓
Reference Composition Runtime = Cordis
      ↓
Providers / Capabilities / External Runtimes
```

`Work / Authority / Receipt / Outcome / Acceptance` 被重新定义为**协议资源与语义契约**，其参考 Controller/Provider 可以替换。

## 2.2 Cordis 直接使用，但只解决 Composition

[新裁决] 第一阶段不再自研 `MornMetaKernel` 的 DI、Service Registry、Plugin Lifecycle、Effect Cleanup、Dependency Graph、Scoped Context、Hot Replacement 等通用机制。

这些直接交给 Cordis。

但：

```text
Cordis Effect Lifecycle
!=
Business Effect Lifecycle
```

Cordis 可以撤销 event listener、service registration、timer、watcher 等运行时 effect；现实世界已经创建的采购单、维修单、质量状态、客户通知、物理动作不能靠 `fiber.dispose()` 撤销。

所以 Cordis 只位于 **Composition Plane**。

## 2.3 Morn 新增真正的一等公民：Durable Control Plane

过去我们容易把“Work Runtime”理解成某个 Harness/Workflow 的运行状态。

v11.5 明确：

> **真实 Work 的 durable state 必须独立于 Cordis Fiber、Agent Session、Sandbox Instance 和 Workflow Execution。**

因此新增：

- API / Resource Store；
- Append / Supersede Historical Facts；
- Current Projection；
- Controllers；
- Reconciliation；
- Binding；
- Desired / Observed State；
- Conditions；
- Profile Conformance。

---

# 3. 外部研究带来的架构启发：吸收什么，不吸收什么

## 3.1 Cordis：吸收“时空可组合性”，不把业务世界变成 Fiber

[研究] Cordis 背后的论文《A Programming Paradigm for Spatiotemporal Composability》把动态组合拆为：

- **Temporal composability**：组件卸载时，其局部 effects 可完整撤销；
- **Spatial composability**：组件声明依赖，依赖出现/消失时响应式激活/停用。

Cordis 用 Context、Service、Fiber、reversible effects、reactive dependency/coeffect 来实现这一模型。

Morn 吸收：

```text
stable service slot
replaceable provider
explicit dependency
scope
lifecycle
reversible runtime effect
configuration reconciliation
```

Morn 不吸收为业务语义：

```text
Fiber lifecycle = Work lifecycle        NO
Effect disposer = business rollback     NO
Service ready = business qualified      NO
Plugin active = customer accepted       NO
```

## 3.2 DeepSeek Harness：吸收“插件化 Harness”，并学习其 Session / Cordis 分层

[研究] DSH 把 model adapter、tool registry、session、agent loop 等都做成 Cordis plugin，并用稳定 `ctx.<service>` key + `inject` 管理依赖。

更重要的是，DSH 自己把：

```text
Cordis runtime events
```

和：

```text
append-only SessionEvent log
```

分开。后者是 Agent interaction history 的 source of truth，持久化接口也独立为 provider。

Morn 借鉴这个分层：

```text
Cordis = composition truth
Morn durable event/history = work truth
```

## 3.3 Paper2Agent：吸收“资产 → 可执行能力”的编译范式

[研究] Paper2Agent 不是论文 RAG。它把论文、补充材料、代码、数据、教程、workflow 解析为可执行 MCP Resource/Tool，并生成测试进行验证。Nature 论文中，100 篇计算生物学论文中 74 篇成功 agentify，599 个提出工具中 593 个通过自动验证；失败主要来自缺代码/数据/模型、环境依赖失败和脚本不可泛化。

Morn 不复制“Paper Agent”产品，而泛化为：

```text
Artifact2Capability
```

输入可以是：

- Paper；
- Code Repository；
- SOP；
- API / OpenAPI；
- Notebook；
- Model；
- Existing Agent；
- Workflow；
- Equipment Manual；
- Excel Rules；
- Human Role Definition。

输出不是直接上线的 Agent，而是：

```text
Candidate Capability Package
→ Verification
→ Qualification
→ Release
→ Site Admission
```

## 3.4 DSec：吸收“Execution Environment 不是 Docker 的同义词”

[研究] DeepSeek Elastic Compute（DSec）通过统一 SDK 暴露 FnCall、Container、microVM、Full VM 等后端，并负责 placement、lifecycle、stateful execution、资源管理与大规模 sandbox 调度。

Morn 因此将 Sandbox 改为：

```text
ExecutionEnvironmentSpec
+
ExecutionEnvironmentProvider
```

Capability 只声明需求；Provider 决定落到 local process、container、microVM、VM、remote sandbox、edge host 或 physical executor。

## 3.5 Learnware：吸收“Implementation + Specification”，扩展到所有 Executor

[研究] Learnware specification 描述模型的 specialty / utility，并贯穿上传、搜索、复用和部署。

Morn 将其推广：

> **Capability = Implementation + Manifest + Qualification + Policy/Authority Envelope + Provenance。**

并适用于 Agent、Rule、Solver、Human、Robot、Model、API、Workflow 等。

## 3.6 Kubernetes Controller：吸收 Desired / Observed + Reconciliation

[研究] Kubernetes controller 持续观察 current state，并让其趋近 spec 中的 desired state；通常不是一个超级 Controller，而是多个 controller 分别管理局部状态。

Morn 借鉴：

```text
Work.spec      = desired business state / contract
Work.status    = observed work state
Controller     = reconciler
Condition      = semantic gate
```

但 Morn 不复制 Kubernetes 的基础设施对象模型；Morn 管的是现实工作、业务动作、结果和验收。

## 3.7 Temporal / Dapr：吸收 Durable Execution，但不把 Morn 变成 Workflow Engine

Temporal / Dapr 可用于长时程、故障恢复、timer、signal、HITL、activities 等。

Morn 将它们视为 `DurableExecutionProvider`，而不是系统语义本体。

一部分工作可由 Controller + DB 完成；稳定子流程可交给 Temporal/Dapr；不确定推理交给 Agent；优化交给 Solver；责任判断交给 Human。

## 3.8 标准优先，而不是 Morn 自造协议宇宙

优先复用：

| 问题 | 优先标准/生态 |
|---|---|
| Tool / Resource | MCP |
| Agent ↔ Agent | A2A |
| REST API | OpenAPI |
| Event Envelope | CloudEvents |
| Telemetry | OpenTelemetry |
| Policy Decision | OPA / Cedar |
| Workload Identity | SPIFFE / 企业 IAM |
| Package / Artifact | OCI / ORAS |
| Signing | Sigstore |
| Build Provenance | SLSA |
| Local Typed Component | Wasm Component Model / WIT（可选） |
| Composition | Cordis |
| Harness | DSH / Pi / openJiuwen / AgentRun 等 |
| Durable Workflow | Temporal / Dapr（可选） |
| Elastic Sandbox | DSec-like Provider / Cloud Sandbox |

---

# 4. Morn 六 Plane 总体架构

```text
┌──────────────────────────────────────────────────────────┐
│ 1. Semantic / Specification Plane                        │
│ Work / Capability / Authority / Binding / Attempt        │
│ Receipt / Outcome / Acceptance / Profile / Qualification │
└──────────────────────────┬───────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│ 2. Durable Control Plane                                 │
│ API / Store / Event History / Projection / Controllers   │
│ Reconciliation / Conditions / Conformance                │
└──────────────────────────┬───────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│ 3. Composition Plane                                     │
│ Cordis Reference Runtime                                 │
│ Context / Service / Dependency / Fiber / Effect / Config │
└──────────────────────────┬───────────────────────────────┘
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
┌────────────────┐ ┌────────────────┐ ┌───────────────────┐
│4. Execution    │ │5. Trust &      │ │6. Capability      │
│Plane           │ │Governance      │ │Supply Chain       │
│DSH/Pi/Solver   │ │OPA/Cedar/IAM   │ │Compiler/Registry  │
│Temporal/DSec   │ │Approval/SPIFFE │ │OCI/Sigstore/SLSA  │
│Human/Robot/API │ │Policy Gateway  │ │Qualification      │
└────────────────┘ └────────────────┘ └───────────────────┘
```

Domain Products 位于以上六 Plane 之上：

```text
Morn Factory
Morn Work
Morn Research
Morn BioLab
Morn Personal
...
```

它们不是独立平台，而是：

> **Domain Profile + Domain Resources + Capability Packages + UX + Connectors + Qualification/Acceptance。**

---

# 5. Plane 1：Morn Semantic / Specification Plane

## 5.1 为什么“协议”比“内核代码”更重要

Morn 不再寻找一块“永远不可变的代码”。

真正要稳定的是：

- 对象怎样标识；
- 工作怎样定义；
- 能力怎样描述；
- 权限怎样表达；
- 执行怎样绑定；
- 外部副作用怎样记录；
- 结果怎样观察；
- 验收怎样表达；
- 版本如何演进；
- Provider 如何被替换而不篡改历史。

这些属于 `Morn Semantic Specification`。

实现可以自由替换；Contract 版本化演进；运行 Binding 在其 execution scope 内 pin；历史事实通过 supersession/retraction 演进，不能无痕覆盖。

## 5.2 核心资源模型

v11.5 建议最小资源集合：

```text
ObjectRef
Observation
Situation
Work
Commitment
Principal / Actor / Role
AuthorityRequest / AuthorityDecision
Capability
CapabilityRelease
QualificationRecord
SiteAdmission
ExecutionBinding
ActionAttempt
Receipt
ObservedOutcome
AcceptanceDecision
ValueAssessment
Profile
SolutionPackage
```

这些不是要求全部成为独立数据库表，而是协议资源/对象语义。

## 5.3 与 v11.3 六类 Record 的映射

| v11.3 Record | v11.5 主要语义 | 处理 |
|---|---|---|
| World Record | ObjectRef / Observation / Situation / SourceBinding | 保留，外部系统仍为事实权威 |
| Workforce Record | Principal / Actor / Role / Authority / Delegation | 保留，Role 与 Agent 解耦 |
| Work Record | Work.spec / Work.status / Commitment / AcceptanceSpec | 核心强化 |
| Decision Record | ContextSnapshot / Candidate / Assumption / Constraint | 保留，Agent/Solver 不成为事实权威 |
| Execution Record | Binding / Attempt / Receipt / Reconciliation | 核心强化 |
| Outcome Record | ObservedOutcome / Acceptance / ValueAssessment | 核心强化 |

不新增一套与 Six Records 并行、重复的事实账本。

---

# 6. Work Resource：从“任务状态”升级为 Desired / Observed Contract

## 6.1 Work.spec

`spec` 描述期望状态与运行约束，示例：

```yaml
apiVersion: work.morn.io/v1alpha1
kind: Work
metadata:
  uid: work-1042
  generation: 7
spec:
  workType: factory.equipment-exception
  goal:
    type: restore_operational_readiness
  targetRefs:
    - asset:CNC-17
  constraints:
    - no_plc_write
    - no_quality_release
  acceptance:
    required:
      - machine_operational_verified
      - affected_orders_accounted
      - production_owner_accepts
  authorityProfileRef: plant-a/night-shift/v3
  budget:
    maxCostCNY: 50
    maxHumanMinutes: 120
  profileRef: morn.factory.readonly/v1alpha1
```

## 6.2 Work.status

`status` 是当前观察，不是用户期望：

```yaml
status:
  observedGeneration: 7
  phase: RECONCILING
  conditions:
    - type: CapabilityResolved
      status: "True"
    - type: AuthoritySatisfied
      status: "True"
    - type: ExternalActionVerified
      status: "False"
      reason: OutcomeUnknown
  activeBindingRef: binding-221
  activeAttemptRef: attempt-83
  outcomeRef: null
```

## 6.3 Conditions 是语义 Gate，不是固定模块

Factory Profile 可以要求：

```text
CapabilityResolved = True
AuthoritySatisfied = True
SourceBindingsReady = True
```

才能进入某个执行阶段。

产生这些 Conditions 的 Provider 可以替换。

因此：

> **Implementation replaceable; semantic gate not silently bypassable.**

---

# 7. Plane 2：Durable Control Plane

## 7.1 Control Plane 是 Morn 自研重点之一

Reference Control Plane 第一阶段建议保持轻：

```text
HTTP/gRPC API
PostgreSQL
Append history tables
Current projections
Outbox
Controller workers
Lease / idempotency
Reconciliation jobs
Profile conformance
```

第一版不需要一开始部署 Kafka、Kubernetes、Temporal、SPIFFE、microVM fleet。

## 7.2 Controller 模型

建议最少拆为：

```text
WorkController
SituationController
CapabilityResolverController
BindingController
AuthorityController
ExecutionController
ReceiptController
ReconciliationController
OutcomeController
AcceptanceController
QualificationController
AdmissionController
```

Controller 只负责把局部 observed state 向 desired conditions 推进，不允许一个“超级 Agent”直接拥有所有控制权。

## 7.3 为什么采用控制回路而不是纯 DAG

真实企业工作常见：

- 外部状态随时变化；
- 审批可能等待数天；
- API timeout 后结果未知；
- Provider 可用性变化；
- 同一 Work 中既有确定性步骤，也有不确定推理；
- 人可以中途修改目标或约束；
- 现实动作可能不可逆。

因此主架构是：

```text
Durable Resource State
+
Controllers / Reconciliation
+
Optional Durable Workflow
+
Agent / Solver / Human / Rules
```

而不是把整个企业运营画成一张固定 LangGraph。

## 7.4 DurableExecutionProvider

当稳定子流程确实适合 Workflow 时，可接：

```text
TemporalProvider
DaprWorkflowProvider
CustomerBPMProvider
```

Morn 仍保留业务 Work spec/status 与 outcome semantics。

---

# 8. Plane 3：Cordis Reference Composition Runtime

## 8.1 正式裁决：直接用 Cordis，不复制 Cordis

Morn Reference Runtime 第一阶段直接采用 upstream/pinned Cordis。

Cordis 负责：

```text
Context
Service slots
Dependency injection
Fiber lifecycle
Effects
Typed events
Config composition
Provider replacement
Node-local plugin loading
```

Morn 不重复开发：

```text
MornContext
MornDI
MornPluginRegistry
MornFiber
MornEffectSystem
```

## 8.2 Cordis-first, not Cordis-locked

代码边界必须严格：

```text
packages/spec/
packages/control-plane/
packages/contracts/
packages/profiles/
packages/capability/

runtime/cordis/       <-- 唯一直接依赖 Cordis 的区域
```

业务域包禁止到处直接：

```ts
import { Context, Service, Fiber } from '@deepseek-ai/cordis'
```

而通过内部 adapter / port 使用。

## 8.3 Morn Service Slots

Reference Cordis Host 可提供：

```text
ctx.capabilities
ctx.harnesses
ctx.solvers
ctx.sandboxes
ctx.connectors
ctx.identity
ctx.policy
ctx.telemetry
ctx.compilers
```

而 `work / authority / receipt / outcome` 的 durable truth 仍由 Control Plane 持有；Cordis service 可作为 controller/provider 的运行时接口，但不是唯一事实存储。

## 8.4 为什么不把 DSH 内部 Cordis Tree 与 Morn 强行合并

建议 DSH 作为独立进程 Provider：

```text
Morn Control Plane
       │
Morn Cordis Host
       │
DSH Gateway Provider
       │ JSON-RPC / ACP / SDK
       ▼
DSH Process
       │
DSH-owned Cordis Tree
```

理由：

1. DSH/Cordis 仍快速迭代，减少 breaking change 污染；
2. 安全边界更清楚；
3. Morn 不依赖 DSH internal service graph；
4. Pi、openJiuwen、AgentRun 等可采用相同 out-of-process provider 模式；
5. DSH 自己的 SessionEvent history 与 Morn Work history 继续分离。

---

# 9. Plane 4：Execution Plane

Execution Plane 不是一种 Runtime，而是异构执行器集合。

## 9.1 HarnessProvider

统一最小合同示例：

```text
createSession(binding)
submit(input)
resume(checkpoint)
cancel(reason)
getEvents(cursor)
getUsage()
getArtifacts()
close()
```

Provider：

```text
DeepSeekHarnessProvider
PiHarnessProvider
OpenJiuwenProvider
AgentRunProvider
FutureProvider
```

## 9.2 SolverProvider

```text
OR-Tools
Gurobi
Existing APS
Simulation/Optimization Service
```

Solver 的结果是 Candidate，不自动成为决定。

## 9.3 HumanProvider

Human 也是一等 Capability：

```text
site engineer
planner
quality reviewer
operator
customer approver
```

需要：资格、站点、班次、授权、SLA、成本等 Capability metadata。

## 9.4 ActionProvider / Connector

```text
ERP
MES
QMS
CMMS
WMS
SCADA
Email
GitHub
CRM
Robot Gateway
```

所有有真实副作用的 Action 必须通过 Action Gateway / enforcement point，不允许 Agent 直接持有生产凭据。

---

# 10. ExecutionBinding：从“热插拔 Provider”到“安全绑定”

## 10.1 为什么 Cordis 热替换不能直接作用于正在执行的业务 Attempt

Cordis 可以在 Provider 消失时卸载 consumer，再在新 Provider 出现时重新 load。

但业务动作可能已经发生：

```text
POST /cmms/create-order
→ network timeout
```

此时把 Provider A 自动切到 B 并继续同一个 Attempt，可能造成重复工单。

因此新增并强化：

> **ExecutionBinding = 某一 Work/Capability/Provider/Policy/Profile/Runtime 组合在特定 execution scope 内的固定绑定。**

## 10.2 Binding 示例

```yaml
apiVersion: execution.morn.io/v1alpha1
kind: ExecutionBinding
metadata:
  uid: binding-221
spec:
  workRef: work-1042
  capabilityRelease:
    name: cmms-create-order
    digest: sha256:abc...
  provider:
    id: sap-pm-adapter
    version: 4.2.1
  authorityDecisionRef: authz-288
  runtime:
    type: connector
    environmentRef: env-119
  profileRef: morn.factory/v1alpha1
status:
  phase: BOUND
```

## 10.3 Provider 切换规则

- 新 Work 可使用新 Provider；
- 尚未开始的 Binding 可重解；
- 已开始的 Attempt 默认 pin 原 Binding；
- Provider 故障后先进入 `SUSPENDED` / `OUTCOME_UNKNOWN`；
- 新 Provider 继续执行必须创建新 Binding / 新 Attempt 或显式 MigrationDecision；
- 资格、权限、幂等、补偿能力必须重新检查。

---

# 11. Action / Receipt / Reconciliation

## 11.1 外部动作状态机继续保留并强化

```text
PROPOSED
→ AUTHORIZED
→ DISPATCHED
→ ACKNOWLEDGED
→ COMMITTED
→ OBSERVED
→ VERIFIED
```

不确定分支：

```text
DISPATCHED
→ OUTCOME_UNKNOWN
→ RECONCILING
→ COMMITTED / NOT_COMMITTED / ESCALATED
```

## 11.2 为什么 HTTP 200 不等于 Outcome

区分：

```text
Agent says success
!=
Tool returns success
!=
Remote system committed
!=
World state observed
!=
Business outcome accepted
```

## 11.3 幂等与业务键

每个可重试外部 Action 必须尽可能定义：

```text
work_id
binding_id
attempt_id
idempotency_key
business_key
source_system
```

ReconciliationController 以 source-of-truth 回读为优先，而不是 Agent 自述。

---

# 12. Outcome 与 Acceptance

## 12.1 Outcome 是观察到的现实结果

例如：

```text
downtime = 52 min
order_A_delay = 11 min
quality_status = PASS
machine_vibration = 3.1 mm/s
```

## 12.2 Acceptance 是对 Outcome 的业务判断

例如：

```text
ProductionOwner = ACCEPT
QualityOwner = ACCEPT
MaintenanceOwner = COMPLETE
```

因此：

```text
ExecutionFinished != Delivered
Delivered != OutcomeObserved
OutcomeObserved != Accepted
```

## 12.3 Accepted Outcome 才是价值单元

Morn 不以 Agent Run 数、Token 数、Skill 数作为主要价值指标。

关键指标：

```text
accepted_work_rate
reopen_rate
mean_time_to_accepted_outcome
human_minutes_per_accepted_work
cost_per_accepted_work
unknown_outcome_rate
duplicate_side_effect_rate
```

---

# 13. 历史、版本与时间：从“immutable everything”改为 non-destructive evolution

## 13.1 四类可变性

| 类型 | 规则 |
|---|---|
| Draft / Current Projection | 可修改 |
| Released Contract / Artifact | 版本内冻结，修改产生新版本 |
| Running Binding | execution scope 内 pin，迁移必须显式 |
| Historical Fact | 可 supersede / retract / redact，不可无痕覆盖 |

## 13.2 双时间语义

关键 Observation 建议支持：

```text
valid_time      = 事实在现实世界何时有效/发生
recorded_time   = 系统何时知道/记录
```

例：02:13 的温度后来被校准修正。

旧 Observation 可以标记 invalidated，新 Observation `supersedes` 旧事实；当前 projection 使用新事实，但历史仍能解释当时为什么做出报警/决策。

## 13.3 删除与合规

历史非破坏性不等于“永远不准删数据”。

在隐私、密钥泄露、合同或合规要求下可：

```text
redact payload
crypto-shred
physical delete controlled content
```

但应尽可能保留最小审计元信息：谁、何时、为何、依据什么删除。

---

# 14. Plane 5：Trust & Governance Plane

## 14.1 Authority 不等于 Tool Permission

Morn 需要表达：

```text
WHO
on behalf of WHOM
for WHICH WORK
on WHICH OBJECT
at WHICH SITE
in WHICH STATE
within WHICH TIME WINDOW
with WHICH PARAMETERS
under WHICH BUDGET
with WHAT DELEGATION DEPTH
```

## 14.2 Policy Decision 与 Enforcement 分离

Morn 定义：

```text
AuthorityRequest
AuthorityDecision
AuthorityEvidence
```

Provider 可以是：

```text
OPA
Cedar
Customer IAM
SAP authorization
Morn native minimal policy
Human approval
```

但所有真实副作用必须穿过 Morn Action Gateway / PEP。

## 14.3 Agent 默认没有真实世界权限

正确调用链：

```text
Agent proposal
→ Action Gateway
→ Policy Enforcement Point
→ Authority Provider
→ Execution Environment Policy
→ Connector
→ External System
```

禁止：

```text
Agent → Production API with long-lived credential
```

## 14.4 Workload Identity

长期可兼容 SPIFFE / 企业 workload identity。

目标是让：

```text
controller
connector
harness provider
capability runtime
```

拥有短期、可验证、最小权限身份，而非共享 API key。

---

# 15. Execution Environment：DSec 带来的正式抽象

## 15.1 ExecutionEnvironmentSpec

```yaml
execution:
  isolation: microvm
  stateful: true
  persistence: work-scoped
  cpu: 4
  memory: 8GiB
  gpu: none
  network:
    mode: allowlist
    destinations:
      - historian.internal
  filesystem:
    mode: workspace-write
  secrets:
    - historian-read-token
```

## 15.2 Provider 类型

```text
FnCallProvider
LocalProcessProvider
ContainerProvider
MicroVMProvider
FullVMProvider
CloudSandboxProvider
DSecLikeProvider
EdgeHostProvider
PhysicalExecutorProvider
```

## 15.3 Qualification 与环境绑定

Capability 在 container 验证通过，不代表在 local process / Windows / 某 OT gateway 上自动合格。

QualificationRecord 必须包含：

```text
capability_digest
runtime_provider
execution_environment
model/harness version
site/profile
input distribution
verifier
limitations
```

---

# 16. Plane 6：Capability Supply Chain

## 16.1 Capability 正式定义

> **Capability = Implementation + CapabilityManifest + InterfaceContract + ExecutionRequirements + Qualification + AuthorityEnvelope + Provenance。**

Implementation 可以是：

```text
Agent / LLM / Rule / Skill / Workflow / Solver / Human / Robot
Predictive Model / Digital Twin / API / Connector / RPA / Service
```

## 16.2 CapabilityManifest 建议结构

```yaml
apiVersion: capability.morn.io/v1alpha1
kind: Capability
metadata:
  name: equipment-investigator
  version: 1.4.0
spec:
  kind: agent
  provides:
    - equipment.anomaly.investigate
  semantic:
    suitableFor:
      - rotating-equipment.anomaly
    notSuitableFor:
      - safety-critical-control
  interfaces:
    inputSchema: schema://equipment-investigation-input/v1
    outputSchema: schema://equipment-investigation-output/v1
  sideEffects:
    level: none
  execution:
    harnessCompatible: [dsh, pi]
    minimumIsolation: container
    network: [historian]
  authority:
    maximum:
      - historian.read
      - cmms.read
  economics:
    expectedLatencyP95: 45s
    expectedCostCNY: 2.5
  provenance:
    sourceRef: repo://...
```

## 16.3 Declared / Observed / Qualified / Admitted 必须分离

```text
Declared Capability
→ Observed Capability
→ Qualified Capability
→ Released Capability
→ Site-Admitted Capability
```

一个 Agent 自己写“我会排产”不等于排产能力已合格。

---

# 17. Artifact2Capability：Paper2Agent 的 Morn 泛化

## 17.1 Compiler Family

第一批设计：

```text
Paper2Capability
Repo2Capability
OpenAPI2Capability
SOP2Procedure
Model2Capability
```

未来：

```text
Notebook2Capability
Workflow2Capability
ExistingAgent2Capability
HumanRole2Capability
Machine2Capability
```

## 17.2 Pipeline

```text
Artifact Discovery
→ Extract executable contribution
→ Generate Candidate Manifest
→ Wrap Interface
→ Build Environment
→ Generate Tests
→ Fresh Verifier
→ QualificationRecord
→ Package
→ Sign
→ Release
→ Site Admission
```

## 17.3 失败是合法状态

允许：

```text
RESOURCE_ONLY
EXTRACTED
WRAPPED_UNVERIFIED
BLOCKED_MISSING_ARTIFACT
BLOCKED_ENVIRONMENT
FAILED_VERIFICATION
```

不允许：读完 PDF 就直接 `RELEASED`。

---

# 18. Capability Package 与供应链

## 18.1 OCI / ORAS

Capability Package 建议可发布为 OCI Artifact：

```text
oci://registry.example/morn/capabilities/equipment-investigator@sha256:...
```

包内容示例：

```text
capability.yaml
schemas/
runtime/
tests/
qualification/
policy/
sbom/
provenance/
docs/
```

## 18.2 Sigstore / SLSA

目标：

```text
可寻址
可验证
可签名
可追溯
可撤销
```

通过：

```text
OCI digest
Sigstore signature / identity
SLSA provenance
SBOM
QualificationRecord
```

建立“能力供应链”，而不是只做 Agent Marketplace。

---

# 19. Capability Resolver

## 19.1 Resolver 不是 LLM 猜测器

输入：

```text
Work goal
required interfaces
site/profile
authority ceiling
risk
cost/latency budget
qualification requirement
execution environment
availability
```

输出：候选能力及解释。

## 19.2 硬约束与软评分分离

硬过滤：

```text
interface compatible
profile allowed
site admitted
authority <= ceiling
qualification sufficient
runtime/environment available
side-effect class allowed
```

软评分：

```text
quality
latency
cost
historical success
human preference
resource utilization
```

## 19.3 Workcell = Minimum Sufficient Executor Mix

最终可能是：

```text
Rule + SQL + Human
```

也可能：

```text
DSH Agent + Solver + Human + CMMS Connector
```

也可能 0 Agent。

---

# 20. Blueprint / Role / Digital Employee

## 20.1 WorkBlueprint

描述一种可复用 Work：

```text
trigger
goal
constraints
acceptance
required semantic slots
candidate capability requirements
risk/authority envelope
```

## 20.2 RoleBlueprint

> **Digital Employee = RoleBlueprint + ActorInstance。**

Role 是组织和 UX 接口，不是一个永久 Agent Session。

同一个“生产异常协调员”可在不同 Work 中调用不同 Harness/Agent/Solver/Human。

## 20.3 Workcell

Workcell 是某次 Work 的动态执行组合，生命周期通常短于 Role，独立于 Harness Session。

---

# 21. Morn Studio / Capability Factory / Hub

## 21.1 Studio 不再以“创建 Agent”为第一入口

首问：

> **你希望完成什么工作？什么结果才算完成？**

Pipeline：

```text
Natural language / form / imported process
→ Work discovery
→ WorkBlueprint
→ Acceptance / Authority / Constraints
→ Capability Requirements
→ Resolver
→ WorkcellTemplate
→ Test Harness
→ Qualification
→ SolutionPackage
```

## 21.2 Capability Factory 生产什么

```text
CapabilityPackage
WorkBlueprint
RoleBlueprint
WorkcellTemplate
SolutionPackage
QualificationRecord
```

不是 Agent 数量。

## 21.3 Hub

Hub 是受控目录，不是无门槛 App Store。

显示：

```text
release digest
qualification scope
site admission
permissions
cost/latency
known limitations
revocation/deprecation
```

---

# 22. Workbench / Console / UX

## 22.1 Workbench

用户看到：

```text
Work
Goal
Current facts
Unknowns
Candidates
Evidence
Required approvals
Active bindings
External action status
Outcome
Acceptance
```

而不是 Agent 聊天窗口堆叠。

## 22.2 Console

运维/工程视图：

```text
Controllers
Cordis services
Providers
Bindings
Harness sessions
Execution environments
Policy decisions
Receipts/reconciliation
Capability releases
Profile conformance
Telemetry
```

## 22.3 Digital Employee UX

前端仍可提供：

```text
+ 创建数字员工
```

但后台必须转换为 WorkTypes + RoleBlueprint + Capability Requirements，而不是只生成 Prompt。

---

# 23. Event、Telemetry 与统一可观测

## 23.1 CloudEvents 作为外层 Event Envelope

Morn 不自造 event transport 格式。

建议：

```json
{
  "specversion": "1.0",
  "id": "evt-23898",
  "source": "morn://plant-a/cmms",
  "type": "io.morn.action.receipt.v1",
  "subject": "work/work-1042",
  "time": "2026-09-24T13:02:11Z",
  "data": {
    "attempt_id": "attempt-83",
    "external_ref": "MO-88273"
  }
}
```

扩展字段：

```text
mornworkid
mornbindingid
mornattemptid
mornprofile
traceparent
```

## 23.2 OpenTelemetry

建议统一 attributes：

```text
morn.work.id
morn.work.type
morn.attempt.id
morn.binding.id
morn.capability.id
morn.capability.digest
morn.provider.id
morn.profile.id
morn.site.id
morn.authority.decision_id
```

不得自造一套孤立 tracing 系统。

---

# 24. 协议边界：MCP / A2A / OpenAPI / WIT

## 24.1 MCP

适用于 Agent ↔ Tool / Resource。

Morn Capability 可贡献 MCP server/tool，但 MCP tool success 仍需映射到 Attempt/Receipt/Outcome。

## 24.2 A2A

适用于外部独立 Agent peer，Morn 不要求所有 Agent 跑在自己的 Harness 内。

A2A Task/Message/Artifact 与 Morn Work/Outcome 不能一一等同，需要 adapter。

## 24.3 OpenAPI / gRPC / IPC

普通系统连接优先使用已有企业接口，不为“Agent 化”强行包装 MCP。

## 24.4 Wasm Component Model / WIT

可作为未来“小型、高信任、本地能力插件”的方向：接口通过 import/export contract 表达；没有被赋予 import 的能力不能访问相应资源。

第一阶段不强制所有 Capability WASM 化。

---

# 25. Domain Profile：不是插件列表，而是 Guarantee Profile

## 25.1 Profile 定义最低保证

示例：

```yaml
apiVersion: profile.morn.io/v1alpha1
kind: Profile
metadata:
  name: morn.factory.readonly
spec:
  requirements:
    durableWorkState: required
    provenance: required
    sourceOfTruthBinding: required
    authority:
      requiredFor: external-side-effect
    receipt:
      requiredFor: external-action
    reconciliation:
      requiredIf: outcome-unknown
    acceptance:
      humanRequiredFor:
        - schedule_commitment
        - quality_decision
    execution:
      minimumIsolation: container
    observability:
      traceRequired: true
```

## 25.2 Profile 可版本化演进

```text
factory/v1
→ factory/v1.1
→ factory/v2
```

已发布版本内容冻结；可以 deprecate/revoke；不能静默把 v1 内容改掉后仍称 v1。

## 25.3 多种 Profile

```text
Morn Lite Profile
Morn Enterprise Profile
Morn Factory Profile
Morn Research Profile
Morn BioLab Profile
```

它们要求不同保证，不共享一个“大而全固定 Core”。

---

# 26. Morn Factory Profile

## 26.1 Factory 强制关注

```text
Source-of-Truth binding
Site / Line / Asset scope
valid_time / recorded_time
external action receipt
OUTCOME_UNKNOWN reconciliation
authority / approval
physical action envelope
quality / safety boundary
human accountability
```

## 26.2 Factory 不是“智能工厂平台”

Morn Factory 位于：

```text
Smart Factory
   ↑
Morn Factory Work & Operations Control Plane
   ↑
ERP / MES / QMS / CMMS / APS / SCADA / DCS / PLC / people
```

它主要协调 Brownfield 跨系统工作闭环。

## 26.3 首个产品楔子保持不变

> **停机 / 插单后的异常—产能—交期影响评审。**

第一阶段优先：

```text
READ
MAP
ANALYZE
CANDIDATE
EVIDENCE
EXPERT REVIEW
```

默认：

```text
NO PRODUCTION WRITE
```

直到现场资格与授权成立。

---

# 27. Factory 完整运行模拟：CNC-17

## 27.1 现场事件

```text
02:13
CNC-17 spindle vibration = 8.7 mm/s
```

SCADA/Monitoring 产生 Observation，进入 Morn Event Gateway。

## 27.2 SituationController

建立：

```text
Situation: possible_equipment_degradation
asset: CNC-17
current_order: WO-84729
batch: LOT-043
```

不先启动 Agent。

## 27.3 WorkController

匹配 `factory.equipment-exception` WorkBlueprint，创建 `work-1042`：

```text
Goal: restore production readiness
Constraints: no PLC write, no quality release
Acceptance: machine verified + order impact accounted + owner accepts
```

## 27.4 CapabilityResolverController

硬过滤后得到：

```text
VibrationRule@digestA
EquipmentInvestigator@digestB / DSH
ScheduleSolver@digestC / CP-SAT
CMMSRead@digestD
MaintenanceEngineer@Plant-A
```

形成 Workcell。

## 27.5 BindingController

为 EquipmentInvestigator 创建 Binding：

```text
capability digest B
HarnessProvider = DSH
model policy = balanced
site = Plant-A
read-only authority
container sandbox
```

## 27.6 DSH Investigator

DSH 负责 Agent loop、context、tool calls；Morn 只接收结构化 Candidate / EvidenceRefs / Uncertainties。

Agent 不得直接宣布“轴承损坏已确认”。

## 27.7 Evidence acquisition

新增读操作获取：

```text
secondary vibration
temperature
spindle load
maintenance history
```

Hypothesis 更新。

## 27.8 Solver

CP-SAT / Existing APS 产生：

```text
A continue 2h then maintenance
B immediate stop + reroute CNC-09
C split batch
D no action
```

ConstraintValidator 独立检查硬约束。

## 27.9 Human decision

生产负责人选择 B。

AuthorityController 进一步拆分：

```text
maintenance order     → maintenance owner
schedule change       → planner
quality inspection    → quality owner
```

## 27.10 CMMS ActionAttempt

```text
Attempt-83
PROPOSED
AUTHORIZED
DISPATCHED
```

网络 timeout。

Morn 不直接 retry，而进入：

```text
OUTCOME_UNKNOWN
```

## 27.11 ReconciliationController

按 idempotency/business key 回查 CMMS：

```text
MO-88273 exists
```

记录 Receipt：

```text
COMMITTED
OBSERVED
VERIFIED
```

## 27.12 运行中 DSH 崩溃

DSH Provider 崩溃不改变：

```text
work-1042
binding history
receipts
authority decisions
```

若要切 Pi：

```text
new Binding
new Attempt/session
qualification check
context transfer policy
```

不得把 Pi 当作同一个旧 Attempt 自动续跑。

## 27.13 Outcome

维修后：

```text
downtime: 52 min
order A delay: 11 min
order B: on time
quality: pass
vibration: 3.1 mm/s
```

## 27.14 Acceptance

Production / Quality / Maintenance 各自产生 AcceptanceDecision，Work 最终进入 ACCEPTED。

## 27.15 ValueAssessment

对比 baseline：

```text
coordination time
human minutes
accepted outcome cost
late-order minutes
unknown outcome rate
duplicate action count
```

---

# 28. 数字员工、智能体工厂、智能工厂三个概念的最终边界

## 28.1 数字员工

> **产品/组织界面。**

`RoleBlueprint + ActorInstance`，可以调用多个 Agent/Skill/Solver/Human，也可能 0 Agent。

## 28.2 智能体/能力工厂

> **Morn 的 Capability / Solution Factory 子系统。**

生产经验证的 Capability / Blueprint / SolutionPackage，不以 Agent 数量为产量。

## 28.3 智能工厂

> **Morn Factory 服务的现实环境。**

Morn 不拥有 PLC、机器人、厂房、MES、SCADA、DCS 等完整智能工厂栈。

---

# 29. 市场七层栈与 Morn 位置

| 层 | 主要问题 | 代表形态 | Morn 策略 |
|---|---|---|---|
| L0 Model | 推理 | DeepSeek / GPT / Claude / Gemini | 采购、替换 |
| L1 Harness | Agent loop / tool / context | DSH / Pi / openJiuwen | Provider 化 |
| L2 Runtime | Sandbox / Identity / Gateway | AgentRun / AgentCore / DSec-like | Provider 化 |
| L3 Builder / Factory | Agent / Skill / Template | Agentar / Dify / Oracle | Studio 吸收，不拼数量 |
| L4 Digital Workforce | Agent lifecycle / Role | ServiceNow / Workday / Agent 365 | 兼容，不做总定义 |
| L5 Work / Process | Case / Action / Outcome | Palantir / UiPath / Celonis | 直接竞争/合作层 |
| L6 Industrial Ops | 生产/质量/维护/设备 | Siemens / Digiwin / Rockwell / Honeywell 等 | Factory 竞争/集成层 |
| L7 Morn Target | 轻量 Brownfield real-work control plane | 待实证 | 做薄、做窄、做可证伪 |

Morn 不宣称 L7 是空白市场。

---

# 30. 与主要外部体系的精确关系

## 30.1 Cordis

直接使用于 Reference Composition Runtime；不等同 Morn Protocol，不承担 durable business truth。

## 30.2 DeepSeek Harness

独立 Harness Provider / Bundle；负责 Agent loop / session / tools / LLM；不成为 Morn Control Plane。

## 30.3 Pi

第二 Harness Provider；用于证明 Harness-neutral，并提供轻量本地路径。

## 30.4 Paper2Agent

作为 `Artifact2Capability` 设计参照与可选 Compiler Provider；不把 Morn 变成 Paper Agent 产品。

## 30.5 DSec

作为 Execution Environment architecture 参照；Morn 不复制其大规模集群实现。

## 30.6 Learnware

贡献 Specification / discovery / reuse 思想；Morn 扩展到所有 Capability，并加入 Qualification、Authority、Execution、Provenance。

## 30.7 Palantir / UiPath / Celonis / ServiceNow

不是“低层框架竞品”，而是 Work / Operations 层强参照和潜在集成对象。

## 30.8 Siemens / Digiwin / Honeywell / Rockwell 等

Factory 领域的强平台；Morn 不正面替换 OT/工业软件栈，而验证轻量 Brownfield 跨系统 Work 闭环。

---

# 31. 安全与威胁模型

必须考虑：

```text
prompt injection
malicious tool output
confused deputy
credential theft
provider substitution
capability supply-chain attack
untrusted generated code
sandbox escape
replay / duplicate action
stale authority
stale binding
model hallucinated success
source-of-truth conflict
insider approval abuse
```

## 31.1 最小安全原则

1. Agent 不直接持有高权限长期凭据；
2. Side-effect Action 必须穿过 PEP；
3. Capability package 必须可验证来源和 digest；
4. Attempt 使用幂等 key；
5. Unknown outcome 必须 reconcile；
6. Provider replacement 不得静默改变 active Binding；
7. Qualification / Acceptance / Authority 不能由被评对象自己修改；
8. 高风险 Profile 必须要求 human gate；
9. Harness sandbox 不自动视为 industrial safety boundary；
10. 每个高权限 Action 必须可追到 Work / Binding / AuthorityDecision / Receipt。

---

# 32. Data Model 草案

Reference Store 第一阶段可采用 PostgreSQL，核心表/投影建议：

```text
resources
work_specs
work_status
work_conditions
observations
situations
capability_releases
qualification_records
site_admissions
execution_bindings
action_attempts
receipts
outcomes
acceptance_decisions
authority_decisions
profiles
historical_events
outbox
controller_leases
```

不要求所有资源一表一类；可通过 JSONB + typed projection 逐步演进。

---

# 33. API / Contract 草案

```text
POST /works
GET  /works/{id}
PATCH /works/{id}/spec
GET  /works/{id}/status
POST /works/{id}/acceptance

POST /capabilities/search
GET  /capabilities/{name}/{version}
POST /capabilities/{id}/qualifications
POST /capabilities/{id}/admissions

POST /bindings
GET  /bindings/{id}
POST /attempts
POST /attempts/{id}/receipts
POST /attempts/{id}/reconcile

POST /authority/decide
GET  /profiles/{id}
POST /profiles/{id}/conformance
```

所有外部 API 版本化；内部 Cordis API 不泄漏为公共业务契约。

---

# 34. 代码结构建议

```text
morn/

  packages/
    spec/
    contracts/
    control-plane/
    controllers/
    profiles/
    capability/
    qualification/
    events/
    telemetry/

  runtime/
    cordis/

  providers/
    harness-dsh/
    harness-pi/
    policy-opa/
    policy-cedar/
    durable-temporal/
    durable-dapr/
    sandbox-container/
    solver-ortools/
    connector-*/

  factory/
    compilers/
    registry/
    signing/

  domains/
    factory/
    research/
    biolab/

  apps/
    workbench/
    studio/
    console/

  conformance/
    spec-tests/
    profile-tests/
    provider-contract-tests/
```

---

# 35. v11.4 → v11.5 迁移矩阵

| v11.4 主题 | v11.5 处理 | 裁决 |
|---|---|---|
| Microkernel | 降级为 Semantic Specification + Reference Controllers | REDEFINE |
| Work / Authority / Receipt / Outcome / Acceptance | 从 kernel module 改为 protocol resources / semantic gates | KEEP / REFRAME |
| CapabilitySpec | 升级为 CapabilityManifest + Qualification + Supply Chain | EXPAND |
| HarnessProvider | 保留，DSH/Pi out-of-process 优先 | KEEP |
| Provider SPI | 保留，增加 Binding pin / migration semantics | EXPAND |
| WorkBlueprint | 保留 | KEEP |
| RoleBlueprint / Digital Employee | 保留 | KEEP |
| Workcell | 保留，Resolver 驱动 | KEEP |
| SolutionPackage | 保留，OCI package / provenance 可选 | EXPAND |
| Studio | 增 Artifact2Capability | EXPAND |
| Capability Factory | 升级为 Capability Supply Chain | EXPAND |
| Generic Sandbox | 改 ExecutionEnvironmentProvider | REDEFINE |
| Runtime State Separation | 升级为 Work / Runtime / Environment 三层状态 | EXPAND |
| Factory Thin Slice | 保留 | KEEP |
| Pi Default Harness | 改为第二 Provider；Reference 可先 DSH + Pi 双实现 | ADJUST |
| 自研 Meta-Kernel | 不实施 | DROP |
| Cordis | 直接采用于 Composition Plane | ADOPT |
| Durable Control Plane | 新增一等架构层 | ADD |
| Controller/Reconciliation | 新增核心执行范式 | ADD |
| OCI/Sigstore/SLSA | 能力供应链参考标准 | ADD |
| CloudEvents/OTel | 标准化事件与可观测 | ADD |

---

# 36. 现有代码迁移策略：不推倒重写

## Phase 0 - Reality Audit

读取当前真实代码、测试、部署、六类 Record、Factory/BIOFERM 路径，输出：

```text
Current Architecture Map
Current State Ownership Map
Current Side-effect Map
Current Agent/Workflow Runtime Map
Current Persistence Map
```

## Phase 1 - Semantic Adapter

在现有 Core 上先建立：

```text
Work spec/status facade
ExecutionBinding
Attempt/Receipt mapping
Outcome/Acceptance mapping
```

不立即删旧实现。

## Phase 2 - Durable Control Plane Minimum

```text
Postgres durable state
outbox
controller loop
reconciliation
conditions
```

## Phase 3 - Cordis Spike

只验证：

```text
service slot
provider swap
scope
lifecycle
config reload
adapter boundary
```

不要先把全部业务迁入 Cordis。

## Phase 4 - DSH Provider + Pi Provider

DSH 采用官方公开 out-of-process 接口；Pi 实现同一 HarnessProvider Contract。

## Phase 5 - CapabilityManifest / Resolver

完成硬约束过滤 + 可解释评分；先不做复杂自学习调度。

## Phase 6 - Factory Thin Slice

把 CNC-17 类 read-only 异常评审跑通。

## Phase 7 - Artifact2Capability MVP

先做：

```text
Repo2Capability
OpenAPI2Capability
```

Paper2Capability 可作为第二阶段。

## Phase 8 - Supply Chain / Profile Conformance

引入 digest、package、qualification、site admission、profile conformance。

## Phase 9 - Customer Evidence

有真实授权才进入历史数据 / read-only / shadow；否则保持 Engineering Ready。

---

# 37. 工程 Gates

## G0 Reality

- 当前架构与状态归属完成；
- 无“设计即实现”误报。

## G1 Semantic Mapping

- v11.3 Six Records → v11.5 resources 映射清楚；
- 不出现第二套事实账本。

## G2 Durable State

- 进程崩溃后 Work 可恢复；
- current projection 可由历史/资源恢复；
- outbox/idempotency 测试通过。

## G3 Cordis Isolation

- Cordis 只存在于 runtime adapter；
- domain/control-plane 无直接 Cordis 类型泄漏。

## G4 DSH Provider

- DSH session 可启动/恢复/取消；
- DSH success 不直接修改 Work accepted state。

## G5 Second Harness

- Pi 实现同一 HarnessProvider；
- 同一测试 Work 可换 provider。

## G6 Binding Pin

- active Attempt 不被热替换 provider 静默迁移；
- migration 有显式记录。

## G7 Receipt/Reconciliation

- timeout + remote commit 能被正确 reconcile；
- duplicate side effect = 0（测试域）。

## G8 Capability Qualification

- declared != qualified != admitted 在代码和 UI 上分离。

## G9 Profile Conformance

- Factory Profile 缺 Authority/Receipt 等 required condition 时 Work 不可进入对应阶段。

## G10 Factory Thin Slice

- read-only 异常—产能—交期评审端到端完成；
- baseline 对比可生成。

## G11 Provider Swap Benchmark

- DSH → Pi 替换后业务语义不变；
- runtime state 可丢弃/迁移，Work history 不丢失。

## G12 Customer Gate

- 只有真实客户授权数据和接受结果才能升级真实世界成熟度声明。

---

# 38. 90 天建议路线

## Day 1-15：Reality Audit + ADR

- 当前代码/状态/side-effect 全量审计；
- 冻结 ADR：Cordis composition only；
- 冻结 protocol resource names；
- 建立 Architecture Decision Log。

## Day 16-30：Durable Control Plane Minimum

- Work spec/status；
- controller framework；
- binding/attempt/receipt；
- Postgres/outbox；
- crash/recovery tests。

## Day 31-45：Cordis + DSH

- runtime/cordis；
- DSH gateway；
- service slots；
- telemetry；
- isolation tests。

## Day 46-55：Pi + Resolver

- Pi provider；
- CapabilityManifest；
- Resolver；
- provider swap tests。

## Day 56-70：Factory Thin Slice

- fake/fixture MES/CMMS/APS；
- CNC exception flow；
- Workbench；
- authority/reconciliation；
- accepted outcome report。

## Day 71-80：Capability Factory MVP

- Repo2Capability；
- OpenAPI2Capability；
- qualification record；
- package digest。

## Day 81-90：Conformance / Release Readiness

- Factory Profile；
- profile tests；
- failure injection；
- cost/human-minutes；
- site deployment package；
- 若无真实授权，停止于 Engineering Ready。

---

# 39. 产品验证与商业假设

架构不是护城河。必须继续验证：

1. **首个真实工作闭环是否更快？**
2. **异常处置是否更可靠，尤其 unknown outcome / reconciliation？**
3. **第二站点是否显著减少映射、规则、测试与交付工时？**
4. **客户是否能保留自己的系统、数据、Harness、Cloud 与 IAM 主权？**
5. **相比 Palantir/UiPath/Celonis/ServiceNow/Siemens/现有集成，Morn 的轻量接入是否真的更便宜？**
6. **accepted outcome 是否带来可量化业务收益？**

失败时应收窄为集成/工程服务、特定 Work Package，或停止不成立的产品线。

---

# 40. Metrics

## 40.1 Control Plane

```text
work_recovery_success_rate
controller_reconcile_latency
stale_binding_rate
condition_convergence_time
```

## 40.2 Execution

```text
attempt_success_rate
outcome_unknown_rate
reconciliation_success_rate
duplicate_side_effect_rate
provider_failure_recovery_time
```

## 40.3 Capability

```text
qualification_pass_rate
site_admission_lead_time
resolver_top1_success
resolver_fallback_rate
capability_reuse_across_sites
```

## 40.4 Business

```text
accepted_work_rate
mean_time_to_accepted_outcome
human_minutes_per_work
cost_per_accepted_outcome
reopen_rate
baseline_delta
```

---

# 41. 明确停止条件

出现以下任一情况，不得通过“加 Agent / 加模型 / 加架构层”掩盖：

- Work acceptance 无法定义；
- Source-of-truth 无法回读；
- 外部动作无法建立 receipt/reconciliation；
- Agent 能绕过 Action Gateway；
- Capability 无法说明资格范围；
- Provider 热替换破坏 active binding；
- Cordis API 泄漏进全域代码，形成锁定；
- Control Plane 比客户已有 Case/BPM 平台更重但没有增量价值；
- Factory 第一包不优于“现有流程 + 最低成本自动化”；
- 第二站点没有复用收益；
- 没有 accepted outcome，却只汇报 Agent 调用数；
- 安全/质量场景需要的 guarantee 无法被 Profile 验证。

---

# 42. 对外表述边界

## 42.1 当前允许

- Morn 目标是 harness-neutral；
- Morn 以 Work / Capability / Outcome 为核心抽象；
- Morn Reference Composition Runtime 计划优先采用 Cordis；
- Morn Factory 采用 Brownfield-first；
- Capability Factory 目标是生产可验证能力资产和工作方案；
- 数字员工是 Role Interface 之一。

## 42.2 实现/测试后才能说

- “DSH/Pi 可无损替换”；
- “跨 Harness continuity 已证明”；
- “Capability 自动编译可靠”；
- “Factory Profile 已生产级合规”；
- “可以安全自动写生产系统”；
- “第二站点交付成本显著下降”。

## 42.3 禁止

- “Morn 是唯一 Work-first 平台”；
- “本体 + Harness + Agent + 治理是独占技术壁垒”；
- “已经实现自治工厂”；
- “文档设计完成 = 商业成熟”；
- “Agent 说成功 = 业务成功”。

---

# 43. ADR 冻结建议

## ADR-MORN-001：不自研通用组合框架

Reference Composition Runtime 首先采用 Cordis，Morn 通过 adapter 隔离其 API。

## ADR-MORN-002：DSH/Pi 是 Harness Provider，不是 Control Plane

DSH 优先 out-of-process；Pi 作为第二 Provider。

## ADR-MORN-003：Durable Work Truth 独立于 Cordis/Agent Session

Work/Binding/Attempt/Receipt/Outcome/Acceptance 持久状态独立保存。

## ADR-MORN-004：Provider hot-swap 不改变 active Attempt

Provider 变化只影响未来 binding；运行中迁移必须显式。

## ADR-MORN-005：Capability 进入供应链治理

Capability Release 采用 digest、qualification、provenance、site admission；逐步兼容 OCI/Sigstore/SLSA。

## ADR-MORN-006：标准优先

MCP/A2A/CloudEvents/OTel/OPA/Cedar/OCI 等能满足需求时不自造等价协议。

## ADR-MORN-007：Morn Factory 首包保持 read-first

未完成真实客户 Gate 前，不扩到自动生产写入。

---

# 44. 外部研究来源登记（截至 2026-09-24）

> 本节只记录架构研究依据；产品版本变化很快，实施前需按 pinned version 重新核验。

**[R01] DeepSeek Harness - Cordis Primer / Services / Architecture**  
Cordis 是 DSH 底层 vendored plugin framework；Service 使用稳定 `ctx.<key>`；插件通过 `inject` 声明依赖；model adapter、tool registry、session log、agent loop 均可插件化。  
https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/cordis-primer.md  
https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/architecture.md

**[R02] Cordis / Spatiotemporal Composability**  
Cordis 论文将 temporal composability 与 spatial composability 统一进 Context paradigm。  
https://arxiv.org/abs/2608.25512

**[R03] DSH Session / Persistence**  
Session 是 append-only `SessionEvent` log；消息历史由 log 推导；持久化通过可替换 persistence service。  
https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/subsystems/session.md  
https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/subsystems/persistence.md

**[R04] Paper2Agent - Nature / GitHub**  
把 paper + code + data + workflow 转换为可验证 MCP resources/tools；强调现有代码绑定、测试、复现与限制。  
https://www.nature.com/articles/s41586-026-11044-y  
https://github.com/jmiao24/Paper2Agent

**[R05] DeepSeek Elastic Compute (DSec)**  
统一 FnCall / container / microVM / full VM sandbox backend，用于大规模 stateful agent execution。  
https://arxiv.org/abs/2609.22978

**[R06] Learnware Specification**  
Specification 描述模型 specialty/utility，贯穿搜索、复用与部署。  
https://learnware.readthedocs.io/en/latest/components/spec.html

**[R07] Kubernetes Controllers**  
Controller 持续让 current state 向 spec desired state 收敛。  
https://kubernetes.io/docs/concepts/architecture/controller/

**[R08] Temporal Durable Execution**  
长时间 workflow、状态恢复、activities、signals/timers、HITL 等。  
https://docs.temporal.io/

**[R09] Dapr Workflow / Durable Execution**  
支持 stateful、long-running、fault-tolerant workflow 与 HTTP/gRPC 管理。  
https://docs.dapr.io/developing-applications/building-blocks/workflow/workflow-overview/

**[R10] OPA**  
Policy decision 与 enforcement 解耦，接受结构化输入返回 decision。  
https://www.openpolicyagent.org/docs

**[R11] Cedar**  
Authorization request 采用 principal / action / resource / context。  
https://docs.cedarpolicy.com/auth/authorization.html

**[R12] CloudEvents**  
统一事件 envelope，required fields 包括 id/source/specversion/type。  
https://github.com/cloudevents/spec/blob/main/cloudevents/spec.md

**[R13] OpenTelemetry Semantic Conventions**  
统一 trace/metric/log/resource 等语义命名。  
https://opentelemetry.io/docs/concepts/semantic-conventions/

**[R14] OCI Image/Artifact + ORAS**  
OCI manifest 支持 content-addressable artifact；ORAS 用 OCI registry 分发任意 artifacts。  
https://specs.opencontainers.org/image-spec/manifest/  
https://oras.land/docs/1.1/

**[R15] Sigstore / Cosign**  
身份绑定签名与透明日志。  
https://docs.sigstore.dev/cosign/signing/overview/

**[R16] SLSA Provenance**  
记录 artifact 在哪里、何时、如何生产，用于验证与重建。  
https://slsa.dev/spec/v1.2/provenance

**[R17] SPIFFE**  
SPIFFE ID + SVID 提供可验证 workload identity。  
https://spiffe.io/docs/latest/spiffe-specs/spiffe-id/

**[R18] WebAssembly Component Model / WIT**  
WIT 用 imports/exports 定义组件 contract；组件通过接口组合。  
https://component-model.bytecodealliance.org/design/wit.html

**[R19] A2A Protocol**  
跨框架、跨语言、跨供应商 Agent 互操作标准，定义 capability discovery、Task/Message/Artifact 等。  
https://github.com/a2aproject/A2A/blob/main/docs/specification.md

**[R20] v11.4-R1 既有市场与产品研究来源**  
PI-Desktop、openJiuwen、AgentRun、Agentar、Microsoft Agent 365、Workday ASOR、ServiceNow Autonomous Workforce、Palantir Ontology MCP、UiPath Maestro、Celonis、Siemens Intelligence Center X、鼎捷 EIOSpace、创新奇智 AInnoGC、OPC-Nexus 等沿用 v11.4 来源登记；本版不重复扩写所有产品能力，实际实施前须重新核验目标版本。

---

# 45. 最终架构结论

## 45.1 最终关系

```text
                         MORN

              Semantic Specification
                       │
                 Domain Profiles
                       │
                       ▼
              Durable Control Plane
       Work / Binding / Attempt / Receipt /
       Outcome / Acceptance / Controllers
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
   Composition      Trust        Capability
      Plane          Plane       Supply Chain
     Cordis        OPA/Cedar    OCI/Qualification
          │
          ▼
      Provider Fabric
          │
 ┌────────┼─────────┬─────────┬──────────┐
 ▼        ▼         ▼         ▼          ▼
DSH      Pi      Temporal   Solver     Human
Agent  Harness    /Dapr    /Models    /Robot
          │
          ▼
     External World
 ERP / MES / QMS / CMMS / APS / SCADA / APIs
          │
          ▼
       Receipt
          │
      Observed Outcome
          │
       Acceptance
```

## 45.2 最短解释

> **Cordis 管“软件能力怎么动态组合”；DSH/Pi 管“Agent 怎么跑”；Temporal/Dapr 可管“某些流程怎么持久执行”；Morn 管“真实工作是什么、谁能做什么、执行绑定到谁、外部动作是否真实发生、现实结果是什么、是否被接受”。**

## 45.3 什么最接近 Morn 的“核心”

不是一份不可变代码。

而是：

```text
versioned semantic contracts
+
durable work state
+
explicit execution bindings
+
non-destructive historical evolution
+
profile conformance
```

## 45.4 现在真正需要做的下一步

不是继续画更大的架构图，也不是立刻重写 Core。

而是：

```text
读取真实代码和测试
→ Current Architecture / State Ownership / Side-effect Map
→ 冻结 ADR-MORN-001~007
→ 实现最小 Durable Work Control Plane
→ Cordis Spike
→ DSH Provider
→ Pi Provider
→ Binding / Receipt / Reconciliation
→ Factory read-only thin slice
→ Capability Manifest / Resolver
→ 再判断 Artifact2Capability 与 Supply Chain 是否进入下一阶段
```

**Morn 的最终产品完成标准仍然只有一个：在明确授权、安全和成本边界内，持续把客户真实 Work 推进到可核对的现实 Outcome，并由相应责任主体接受。**

