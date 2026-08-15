# GOAL2_UI_SPEC.md

## Workbench v0.2
新增：long-running Work timeline、WorkGraph、wait/signal、retry/compensation/escalation、approval、checkpoint、Replay/Shadow compare、BioLab 三闭环。

## Studio v0.2
主流程：
```text
Describe Goal
→ ProblemSpec Preview
→ WorkGraph
→ WorkPackage Plan
→ Role/Member Plan
→ Capability Resolution
→ Harness Plan
→ Evaluation Plan
→ Validate
→ Human Review
→ Compile SolutionPackage
```
必须显示 assumptions、gaps、confidence、decision sources、risks。

Simulation Lab：选择 scenario → fault injection → run → EvaluationReport。

## Console v0.2
Delegation graph、Accountability chain、Representation contracts、Durable runs、waits/signals、retry/compensation、Attention、EvaluationRuns、ShadowRuns、SolutionVersions/readiness。

## Hub v0.2
SolutionTemplate、DomainPack、EvaluationPack、SimulationScenario、WorkCapabilityCandidate、RoleBlueprint、WorkflowTemplate，显示 version/dependencies/compatibility/trust/evaluation/changelog。

## UX 约束
- 不用 Chat 代替流程 UI；
- Compiler 决策依据可展开；
- failure/blocked 可见；
- 不提供无治理的 Auto Deploy Production；
- high-risk actions 有明确 Human Gate；
- must-pass path 不用静态 fake JSON。
