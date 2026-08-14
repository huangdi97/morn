# ARCHITECTURE_FREEZE.md — v10.2 近期工程冻结

## 1. 核心术语

今晚不再引入新的顶层抽象。冻结：

```text
Principal
Identity
Workspace

ObjectType
Object
Relation
Event
StateSnapshot
ActionType
ActionProposal
Action
Goal
Metric

Artifact
ArtifactVersion
DecisionPackage
VerificationReport
OutcomeRecord

ActorTemplate
ActorInstance
ActorOrigin
RepresentationContract

RoleSlot
MemberBinding
ResponsibilityBinding
Delegation
Commitment

WorkPackage
WorkContract
AcceptanceSpec
OutcomeContract
ExecutionMode
Workcell

HarnessSpec
HarnessVersion
HarnessBinding
RuntimeBinding
CapabilityDefinition
CapabilityProvider
CapabilityConsumer
CapabilityScope
ExecutionEvent
ExecutionReceipt

EvolutionCandidate
EvolutionBranch
EvolutionEvaluation
ShadowRun
PromotionDecision

Solution / DomainPack / WorkSystemManifest
```

## 2. 六个 Record

```text
World Record
Work Record
Decision Record
Workforce Record
Execution Record
Outcome Record
```

一次正式工作必须可以回答：

```text
谁
→ 为什么
→ 做了什么工作
→ 通过什么 Harness/Runtime 执行
→ 产生什么 Artifact/Action
→ 是否通过验收
→ 世界状态改变了什么
→ Outcome 是否达到
```

## 3. 正式写路径

### World 写路径
```text
Actor/Program/Human Proposal
→ Validate
→ Policy
→ Approval/Simulation as required
→ ActionGateway
→ Commit StateSnapshot/StateDiff
→ Verify
→ Ledger
→ Outcome
```

### Artifact 写路径
```text
Draft
→ Submitted
→ InReview
→ ChangesRequested | Approved
→ Locked
→ Superseded
→ Archived
```

Artifact content 变化 = 新版本。

## 4. Harness 关系

```text
Actor = 谁
Role = 负责什么
Harness = 怎样可靠地工作
Runtime = 在哪里/靠什么执行
```

同一 Actor 切换 Harness/Runtime 后：
- identity 不变；
- workspace 不变；
- work contract 不变；
- artifact/history 不变；
- authority 不变；
- Morn canonical record 不变。

## 5. Capability Seam

Provider：
- 声明 Definition；
- 声明 Requirements；
- 声明 EffectContract；
- mount(scope)；
- unmount(handle)。

`unmount` 只承诺 E0 清理。

## 6. Evolution 边界

Production object 与 Evolution object 分表/分 aggregate/分 namespace。
Evolution Candidate 只能产生：
- proposal；
- branch；
- replay/evaluation；
- promotion decision。

没有 promotion + policy/certification，不可变成 production version。

## 7. UI 边界

Workbench / Studio / Console / Hub 是横跨同一 Domain/Application Service 的产品表面，不得各自建立独立业务逻辑副本。

## 8. Repository Migration Rule

如果现有仓库已经有类似概念：
- 优先 alias/migration/adapter；
- 不重复创建语义等价对象；
- 在 `DECISIONS.md` 写旧→新映射；
- 保证数据库 migration 有 forward path；
- 高风险破坏性 migration 先 snapshot/backup。
