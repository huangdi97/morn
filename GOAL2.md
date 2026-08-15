# GOAL2.md — Morn v10.2 Goal 2

## Goal ID
`MORN-V10.2-G2-SOLUTION-FACTORY`

## 北极星
把 Morn 从“可执行 Work OS 基线”推进为：

> **可以把目标/问题编译成可审查 Solution / Work System，支持长程可恢复执行、历史回放、仿真评测，并在 BioLab 中运行三个受治理科研闭环的 Morn v0.2。**

## G2.1 Mixed Organization 完整化
补齐并真实测试：
- Human / Actor / DeterministicWorker / ExternalService / Device；
- RoleSlot / MemberBinding / ResponsibilityBinding；
- Delegation / Commitment / Accountability Chain；
- ActorOrigin / RepresentationContract / DecisionPolicyAsset；
- Workcell lifecycle。

不变量：
```text
Role != Actor
Assignment != Delegation
Delegation != Accountability transfer
Digital Twin != Human
Representation != unrestricted authority
```

## G2.2 Durable Work Runtime v0.2
在 Goal 1 checkpoint/resume 上增加：
- WorkflowDefinition/Run/Step；
- Signal / Wait / Timer；
- Pause/Resume；
- RetryPolicy；
- Compensation；
- Escalation；
- BudgetGuard；
- AttentionQueue；
- Drift detection / Replan request；
- RecoveryReceipt。

目标：一个持续数小时/数天的 WorkContract 能等待真人、重启恢复、失败补偿、升级和终止，而不是依赖一次 Agent Loop。

## G2.3 Organization/Solution Compiler v0.2
实现：
```text
Goal / Request
→ ProblemSpec
→ Constraints / Assumptions
→ Operational Object Requirements
→ WorkGraph
→ WorkPackage / WorkContract
→ AcceptanceSpec / OutcomeContract
→ ExecutionMode
→ RoleSlots
→ MemberType Plan
→ Capability Requirements
→ Capability Resolution / Gap
→ HarnessBinding / RuntimeProfile
→ Workflow
→ EvaluationPlan
→ ProposedSolution
→ ValidationReport
→ ApprovedSolution
→ SolutionPackage
```

Compiler 本阶段只生成**可审查**方案，不自动部署 Production。

## G2.4 Work System as Code v0.2
SolutionPackage 至少支持：
- manifest schema；
- validate；
- version；
- diff；
- export；
- import/load；
- compatibility check；
- dry-run compile。

## G2.5 Historical Replay
输入：World snapshot、WorkContract、Workflow version、Harness profile、fixtures/events。

输出：ReplayRun、step results、StateDiff、Outcome、deviation、ReproducibilityReport。

Replay 不得写 Production。

## G2.6 Simulation & Evaluation v0.1
至少支持：
1. Tool failure；
2. Harness failure；
3. permission denial；
4. irreversible action approval missing；
5. wrong/untrusted knowledge；
6. Representation boundary violation；
7. evidence conflict；
8. malformed/anomalous data；
9. timeout/budget exhaustion；
10. regression against previous Solution version。

输出：correctness、acceptance、policy、recovery、outcome、human intervention、cost/latency、regression、evidence refs。

## G2.7 Shadow
实现本地/fixture Shadow：
```text
same input/events
→ baseline solution
→ candidate solution
→ isolated side effects
→ compare
→ readiness decision
```

## G2.8 BioLab Dream Factory v0.1
必须跑通三闭环：

### A Literature → Hypothesis → Experiment Design
Evidence → Hypothesis → Review → ExperimentDesign → Human Approval

### B Dataset → Reviewed Scientific Claim
Dataset → QC → Analysis → Artifact → Independent Review → Human Approval → Claim

### C Result → Figure/Table → Manuscript Consistency
ApprovedResult → Figure/Table → ManuscriptDraft → Claim-Evidence Check → Review → ReleaseCandidate

湿实验保持 Human / Device gate。

## G2.9 UI v0.2
### Workbench
long-running Work、timeline、signal/wait、blocked/escalated、WorkGraph、Replay/Shadow compare、BioLab 三闭环。

### Studio
ProblemSpec Builder、Solution Compiler、WorkGraph、MemberType、CapabilityGap、HarnessPlan、EvaluationPlan、Manifest preview/diff、Simulation Lab。

### Console
Delegation/Accountability、Representation、Durable state、Attention、Evaluation、Shadow、Solution versions/readiness。

### Hub
SolutionTemplate、DomainPack、EvaluationPack、SimulationScenario、WorkCapabilityCandidate、version/dependency/compatibility。

## G2.10 最终 E2E
```text
User Goal
→ Compiler
→ ProposedSolution
→ Human Review/Approval
→ SolutionPackage
→ Historical Replay
→ Evaluation PASS
→ Local Shadow
→ BioLab Workcell execution
→ Artifact/Decision/Outcome
→ Final EvaluationReport
```

## Definition of Done
不是“类型都存在”，而是：Compiler 真能编译；Durable 真能恢复；Replay 真隔离；Evaluation 真注入失败；Shadow 真比较；BioLab 三闭环真走后端；UI 真接统一后端；全量回归通过。
