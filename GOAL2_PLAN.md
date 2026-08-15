# GOAL2_PLAN.md — M0 → M10

## M0 Verify Goal 1
执行 `GOAL2_PRECONDITIONS.md`。更新 `STATUS_GOAL2.md`。

## M1 Mixed Organization Completion
实现/补齐 Delegation、Commitment、Accountability、RepresentationContract、DecisionPolicyAsset、Workcell lifecycle。

测试：authority scope、expiry、retained accountability、representation allow/deny/revoke、wrong member type。

## M2 Durable Work Runtime v0.2
实现 WorkflowRun、Checkpoint、Signal、Wait/Timer、Retry、Compensation、Escalation、Budget、Attention、Drift/Replan。

关键测试：process restart→resume、wait human signal、timeout、retry exhausted、E2 compensation、budget stop、drift attention。

## M3 ProblemSpec + WorkGraph
Goal → ProblemSpec → Constraints/Assumptions → Operational Object requirements → WorkGraph。

LLM 只能作为候选 planner；结构化输出必须 validator 通过。

## M4 Solution Compiler v0.2
WorkGraph → WorkPackages → Acceptance/Outcome → ExecutionMode → Role/Member → Capability Resolution/Gap → HarnessPlan → Workflow → EvaluationPlan → ProposedSolution → ValidationReport。

## M5 Work System as Code
实现 manifest schema/version/validate/export/import/diff/compatibility/dry-run。不得自动 activate production。

## M6 Replay / Simulation / Evaluation
实现隔离 runner，并加入 Tool/Harness/Permission/Approval/UntrustedContext/Representation/EvidenceConflict/MalformedData/Budget/Regression 场景。

## M7 Shadow
同输入跑 baseline/candidate；隔离执行；比较 acceptance/policy/outcome/interventions/cost/latency。

## M8 BioLab Dream Factory v0.1
跑通 Literature→ExperimentDesign、Dataset→ReviewedClaim、Result→ManuscriptConsistency 三闭环。

## M9 Product Surfaces v0.2
Studio Compiler → Workbench long-running → Console governance/eval → Hub assets。

## M10 E2E + Closeout
Goal → Compile → Review → SolutionPackage → Replay → Evaluation → Shadow → BioLab execution → Outcome。

完成全量回归和 `reports/goal2_final_report.md`。
