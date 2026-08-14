# UI_SPEC.md — Morn v0.1 产品表面

## 1. 总原则

Workbench / Studio / Console / Hub 是同一后端的四个视角。

视觉目标：
- 专业桌面工作台；
- 信息密度高但层级清楚；
- 不做“聊天机器人首页”；
- Chat 可保留为辅助入口；
- 关键状态优先：Running / Waiting / Blocked / Approval / Failed / Complete。

不得：
- 四个 surface 各自维护一份模拟数据；
- 用一堆不可点击卡片冒充功能；
- 每个子功能都开一个顶级页面。

## 2. App Shell

建议：
- 左侧主导航：Workbench / Studio / Console / Hub
- 顶部：Workspace、全局搜索、当前环境、Harness health、Attention badge
- 主内容：surface route
- 可选右侧 drawer：Detail / Approval / Artifact / Trace

## 3. Workbench

首页信息：
- 当前 Mission
- 关键 Operational Objects
- 我的 WorkPackages / WorkContracts
- Workcell 状态
- 待审批 Action
- Artifacts / Decisions
- Blocked / Failure
- Metrics / Outcomes
- 最近 StateDiff/Event
- Attention Queue

### Workcell Live View

每个 member：
- member name
- member type
- role
- harness/runtime
- status
- current step
- last event
- blocked reason

Workcell 下显示：
- WorkContract
- acceptance progress
- attention
- recent artifact
- outcome target

### Attention Queue
分类：
- Approval Required
- Policy Conflict
- Evidence Conflict
- Tool Failure
- Budget Risk
- Deadline Drift
- Low Confidence
- Irreversible Action
- Representation Boundary

## 4. Studio

今晚必须可用而不是“大而全”：

### Actor Builder
字段：
- template/name
- origin
- workspace binding
- role binding
- harness binding

### Role & Harness Builder
- RoleSlot responsibilities
- accepted member types
- required capabilities
- HarnessSpec/Binding

### WorkPackage Builder
- objective
- inputs
- required outputs
- acceptance
- execution mode
- allowed/prohibited actions
- budget/deadline
- approval gates

### Workcell Builder
把 RoleSlot 绑定到 Human/Actor/Program/Service/Device。

### Domain/World Builder
最小创建 ObjectType / ActionType / relation。

### Evaluation
展示 evaluation suite / result。

### Solution Manifest Preview
把当前配置序列化为 machine-readable manifest 预览。

## 5. Console

重点是治理：

- Identity / Registry
- Workforce
- Operational World state
- WorkPackage / Commitment
- Delegation / Authority（若已实现）
- Memory Ownership
- Harness / Runtime health
- Approval / Attention
- Policies
- Trace / Errors
- Outcomes / Metrics
- Version / Rollback
- Evolution Promotion Decisions

今晚不需要企业级组织大屏，但所有状态必须来自真实 record。

## 6. Hub

今晚做 registry，不做商业市场。

资产类型至少：
- ActorTemplate
- RoleBlueprint
- HarnessTemplate
- WorkPackageTemplate
- WorkcellBlueprint
- DomainPack
- EvaluationPack
- CertifiedWorkCapability（可先 metadata only）

资产字段：
- id/version
- author/owner
- dependencies
- permissions
- risk
- test/evaluation status
- trust level
- lifecycle

Trust：
- Unverified
- CommunityTested
- Verified
- Certified
- Restricted
- Deprecated
- Retired

## 7. Evolution Center

可以作为 Workbench/Console 子页：

每个 Candidate：
- current state
- candidate type
- evidence window
- expected benefit
- risk
- required evaluation
- shadow status
- promotion gate
- promotion decision

按钮必须遵守 backend policy，不能前端直接“Promote”。

## 8. BioLab Demo

演示数据流必须后端创建：

```text
Dataset
→ WorkPackage
→ Analysis Artifact
→ Review
→ PI Approval
→ StateDiff
→ Reviewed Claim Outcome
```

Workbench 可看到 run；
Console 可看到 approval/trace；
Hub 可看到 BioLab domain/template；
Studio 可查看/创建 WorkPackage。
