# GOAL2_ACCEPTANCE.md

> 状态：2026-08-15 逐条核对。`[x]` = 通过（代码 + 测试 + 命令）。全量验证 `scripts/run_all.ps1` exit 0；
> Rust 87 tests + 前端 2 tests，0 failed / 0 ignored。

## A Goal 1 Regression
- [x] backend/frontend regression green（run_all exit 0；cargo test / typecheck / lint / build）
- [x] Artifact immutable（morn-artifact tests + store restart）
- [x] E3 approval gate（morn-runtime gateway tests）
- [x] Work AcceptanceSpec gate（morn-work tests）
- [x] Harness switch invariant（morn-harness contract tests）
- [x] Durable minimum（morn-work checkpoint/resume）
- [x] Evolution production guard（morn-evolution tests）

## B Mixed Organization
- [x] Delegation scope/expiry（`delegation_scope_and_expiry`）
- [x] retained Accountability（`retained_accountability_is_not_transferred` + AccountabilityChain）
- [x] Representation allow/deny/revoke（`representation_allow_deny_revoke`）
- [x] DecisionPolicyAsset versioned（`decision_policy_asset_is_versioned`）
- [x] Workcell lifecycle tested（`workcell_lifecycle` / `workcell_cannot_jump_to_completed`）

## C Durable v0.2
- [x] durable WorkflowRun（`morn-work::DurableRuntime`）
- [x] Wait/Signal（`wait_human_signal_and_resume` / `duplicate_signal_is_rejected`）
- [x] Pause/Resume（`pause`/`resume`）
- [x] Retry（`retry_then_exhausted_blocks`）
- [x] Compensation（`e2_compensation_completes`）
- [x] Escalation（`escalate` + attention）
- [x] Budget guard（`budget_stop_blocks_run`）
- [x] Attention Queue（open_attention + ResumeDrift/CapabilityGap kinds）
- [x] restart recovery（`morn-store::durable_run_restart_resumes_via_sqlite`）
- [x] Drift detection（`restart_resume_via_checkpoint_and_drift`）

## D Compiler
- [x] ProblemSpec（`ProblemSpecBuilder`）
- [x] WorkGraph（nodes/edges/cycle detection/validator）
- [x] WorkPackage generation（propose）
- [x] Acceptance/Outcome generation（node acceptance -> AcceptanceSpec）
- [x] ExecutionMode（nature->executor planner）
- [x] RoleSlot/MemberType（MemberTypePlan）
- [x] CapabilityResolver + CapabilityGap（Resolved/Missing + gap blocks compile）
- [x] HarnessPlan（harness + fallbacks + rationale）
- [x] EvaluationPlan（acceptance/policy/failure modes/reviewers）
- [x] ProposedSolution（full object）
- [x] ValidationReport（harness compat + approval gate checks）
- [x] ApprovedSolution → SolutionPackage（compile requires approval）
- [x] Explainability metadata（CompilerDecisionSource）

## E Work System as Code
- [x] manifest schema/version/validate/export/import/diff/compatibility/dry-run（`ManifestService` tests）
- [x] no automatic production activate（compile 只生成本地 package）

## F Replay/Evaluation
- [x] replay isolated（`replay_never_touches_production`）
- [x] tool/harness/permission/approval/untrusted context/representation/evidence conflict/malformed data/budget/regression scenarios（12 FaultKind + tests）
- [x] structured EvaluationReport（correctness/acceptance/policy/recovery/outcome/interventions/cost/regressions/evidence/decision）

## G Shadow
- [x] baseline/candidate compare（`ShadowRunner::compare`）
- [x] no production side effect（`isolated_side_effects=true`）
- [x] acceptance/policy/outcome compare（readiness logic）
- [x] readiness decision（Ready/NotReady/Conditional tests）

## H BioLab
- [x] Loop A/B/C complete（`run_loop_a` / `run_dataset_to_claim_e2e` / `run_loop_c` + E2E）
- [x] human wet-lab gate（ExperimentDesign/ReleaseCandidate require PI approval；wet_lab placeholder only）
- [x] provenance/review/reproducibility（artifact versions + review + approval + claim evidence links）
- [x] Dream Factory assets exportable（`export_dream_factory_assets` + test）

## I UI
- [x] Workbench timeline/WorkGraph/replay-shadow（v0.2 cards + buttons）
- [x] Studio compiler/Simulation Lab（Goal→Compiler→Review→Manifest + eval）
- [x] Console delegation/representation/durable/eval/shadow（v0.2 cards）
- [x] Hub new assets（hub2: solution/evaluation/simulation/work capability/role/workflow）
- [x] no static fake data on must-pass path（全部真实 API；Playwright smoke 验证）

## J E2E
- [x] Goal → Compiler → ProposedSolution → Human Review → SolutionPackage → Replay → Evaluation PASS → Shadow → BioLab execution → Artifact/Decision/Outcome → Final EvaluationReport
  （`crates/morn-app/tests/e2e_goal2.rs` `goal2_full_pipeline_e2e`）

## K Quality
- [x] fmt/lint/unit/integration/contract/E2E（run_all exit 0；87 Rust tests）
- [x] frontend build/typecheck（npm run build = tsc + vite）
- [x] desktop build/smoke where environment permits（cargo build -p morn-desktop；沙箱外启动验证）
- [x] docs updated（STATUS_GOAL2 / DECISIONS / BLOCKERS / KNOWN_FAILURES / GOAL2_ACCEPTANCE）
- [x] reports/goal2_final_report.md 已生成

## 不算完成（逐条核对均不成立）
- 无 empty structs/pages（均有真实行为与测试）
- 无 hard-coded success（Loop C 一致性是真实数据检查）
- 无 ignored tests / weakened assertions（0 ignored）
- 无 fake capability resolution（缺 capability 输出 Gap 并阻止 compile）
- 无 fake external integration（真实 DSH 保持 B-001 blocker，不伪造）
- Replay 不写 production（隔离 runner 测试）
- Shadow 不真实执行 E3（只比较 evaluation 结果）
- Compiler 不 auto-deploy（compile 需 human approval，仅生成本地 package）
- 不伪造湿实验自治（wet lab 为 human/device placeholder + approval gate）