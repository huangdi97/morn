# GOAL2_ACCEPTANCE.md

## A Goal 1 Regression
- [ ] backend/frontend regression green
- [ ] Artifact immutable
- [ ] E3 approval gate
- [ ] Work AcceptanceSpec gate
- [ ] Harness switch invariant
- [ ] Durable minimum
- [ ] Evolution production guard

## B Mixed Organization
- [ ] Delegation scope/expiry
- [ ] retained Accountability
- [ ] Representation allow/deny/revoke
- [ ] DecisionPolicyAsset versioned
- [ ] Workcell lifecycle tested

## C Durable v0.2
- [ ] durable WorkflowRun
- [ ] Wait/Signal
- [ ] Pause/Resume
- [ ] Retry
- [ ] Compensation
- [ ] Escalation
- [ ] Budget guard
- [ ] Attention Queue
- [ ] restart recovery
- [ ] Drift detection

## D Compiler
- [ ] ProblemSpec
- [ ] WorkGraph
- [ ] WorkPackage generation
- [ ] Acceptance/Outcome generation
- [ ] ExecutionMode
- [ ] RoleSlot/MemberType
- [ ] CapabilityResolver + CapabilityGap
- [ ] HarnessPlan
- [ ] EvaluationPlan
- [ ] ProposedSolution
- [ ] ValidationReport
- [ ] ApprovedSolution → SolutionPackage
- [ ] Explainability metadata

## E Work System as Code
- [ ] manifest schema/version/validate/export/import/diff/compatibility/dry-run
- [ ] no automatic production activate

## F Replay/Evaluation
- [ ] replay isolated
- [ ] tool/harness/permission/approval/untrusted context/representation/evidence conflict/malformed data/budget/regression scenarios
- [ ] structured EvaluationReport

## G Shadow
- [ ] baseline/candidate compare
- [ ] no production side effect
- [ ] acceptance/policy/outcome compare
- [ ] readiness decision

## H BioLab
- [ ] Loop A/B/C complete
- [ ] human wet-lab gate
- [ ] provenance/review/reproducibility
- [ ] Dream Factory assets exportable

## I UI
- [ ] Workbench timeline/WorkGraph/replay-shadow
- [ ] Studio compiler/Simulation Lab
- [ ] Console delegation/representation/durable/eval/shadow
- [ ] Hub new assets
- [ ] no static fake data on must-pass path

## J E2E
- [ ] Goal → Compiler → ProposedSolution → Human Review → SolutionPackage → Replay → Evaluation PASS → Shadow → BioLab execution → Artifact/Decision/Outcome → Final EvaluationReport

## K Quality
- [ ] fmt/lint/unit/integration/contract/E2E
- [ ] frontend build/typecheck
- [ ] desktop build/smoke where environment permits
- [ ] docs updated
- [ ] reports/goal2_final_report.md

## 不算完成
empty structs/pages、hard-coded success、ignored tests、weakened assertions、fake capability resolution、fake external integration、Replay 写 production、Shadow 真实执行 E3、Compiler auto-deploy、伪造湿实验自治。
