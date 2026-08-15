# GOAL2_COMPILER_SPEC.md

## 输入
```yaml
solution_request:
  goal:
  workspace:
  domain:
  constraints:
  available_members:
  available_capabilities:
  available_harnesses:
  budget:
  deadline:
  risk_profile:
```

## ProblemSpec
至少包含：objective、success_definition、domain、world_scope、constraints、assumptions、risks、unknowns、required_evidence、prohibited_conditions。

## WorkGraph
WorkNode 是 Work，不是 Agent：objective、nature、inputs、outputs、acceptance、dependencies、risks。

WorkEdge 支持：data、approval、state、temporal、evidence dependency。必须检测 cycle/impossible dependency。

## ExecutionMode Planner
先判：deterministic / probabilistic / physical / regulated / social / mixed；再选 program / actor / human / external_service / device / hybrid，并输出 rationale。

## MemberType Planner
输出 RoleSlot、accepted member types、recommendation、alternatives、missing capability。禁止默认生成 Agent。

## Capability Resolver
状态：Resolved / PartiallyResolved / Missing / Incompatible / Restricted。Missing 必须形成 CapabilityGap。

## Harness Planner
依据 tools、sandbox、environment、data boundary、latency、cost、model policy、recovery、evaluation history 选择 HarnessSpec/RuntimeProfile/fallback。

## Evaluation Plan
每个 WorkPackage 至少有 acceptance、policy、failure modes、reviewer、regression baseline、evidence requirement。

## ProposedSolution
必须包含 ProblemSpec、World requirements、WorkGraph、WorkPackages、WorkContracts、RoleSlots、MemberTypePlan、CapabilityResolution、HarnessPlan、Workflow、EvaluationPlan、Assumptions、DecisionSources、UnresolvedGaps、RiskSummary。

## Validation
检查 WorkGraph、Acceptance、executor、capability、harness、authority、policy、data boundary、budget/deadline feasibility、critical gaps、high-risk approval、evaluation coverage。

## Compile
只有 ApprovedSolution 可 compile 为 SolutionPackage。本阶段只生成本地 package，不自动生产部署。
