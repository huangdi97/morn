# GOAL2_ARCHITECTURE_FREEZE.md

## Goal 1 内核不得重新定义
```text
Principal / Identity / Workspace
Object / State / Event / Action
Artifact / Decision / Outcome
WorkPackage / WorkContract / Acceptance / OutcomeContract
Actor / Role / Harness / Runtime
Ledger / Provenance
EffectClass E0-E3
Production vs Evolution separation
```

## Goal 2 新增正式对象
```text
ProblemSpec / Constraint / Assumption
WorkGraph / WorkNode / WorkEdge
CapabilityRequirement / Resolution / Gap
MemberTypePlan / HarnessPlan / RuntimeProfile / EvaluationPlan
ProposedSolution / ValidationReport / ApprovedSolution / SolutionPackage / SolutionVersion
WorkflowDefinition / WorkflowRun / WorkflowStep
Checkpoint / Signal / TimerWait / RetryPolicy / CompensationPlan / Escalation / BudgetGuard / AttentionItem
ReplayScenario / ReplayRun / ReplayReport
SimulationScenario / SimulationRun / FaultInjection
EvaluationSuite / EvaluationRun / EvaluationResult
ShadowProfile / ShadowRun / ShadowComparison
```

## Compiler 边界
Compiler 可：`analyze → propose → validate → compile`。

Compiler 不可：
- 绕过审批 deploy；
- 直接执行外部动作；
- 改 production world；
- 缺 capability 时伪造解决方案。

缺失必须输出 `CapabilityGap`。

## Explainability Contract
每个自动生成 WorkPackage、RoleSlot、MemberType、HarnessBinding、Workflow、EvaluationPlan 必须保存：
`source_facts / rules_or_templates / model_provider_if_any / assumptions / confidence / alternatives / human_review_status`。

## Durable 边界
Checkpoint 引用 canonical record/version，不复制并篡改正式事实；resume 前做 drift check。

## Replay/Simulation/Shadow
默认 read production snapshot、write isolated run state、no irreversible production side effect。

## BioLab
BioLab 只属于 Domain 层，不得污染 Kernel。Scientific Claim 必须有 source、analysis/artifact、review、approval、provenance、version。
