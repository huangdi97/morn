# Morn v10.2 Goal 2 Final Report

Date: 2026-08-15
Goal ID: `MORN-V10.2-G2-SOLUTION-FACTORY`

## 1 Executive Summary

Goal 2 在 Goal 1 基线上增量完成 **Organization/Solution Compiler + Durable Work Runtime v0.2 +
Replay/Simulation/Evaluation + Shadow + BioLab Dream Factory v0.1 + 四个产品表面 v0.2**，并跑通完整 E2E：
`Goal → Compiler → ProposedSolution → Human Review → SolutionPackage → Replay → Evaluation PASS → Shadow → BioLab
执行 → Artifact/Decision/Outcome → Final EvaluationReport`。

无 mock / 空壳 / ignored tests / 弱化断言。真实 DeepSeek Harness smoke 仍为外部/凭据 blocker（B-001，Goal 1 遗留），
Goal 2 不依赖真实 DSH。`scripts/run_all.ps1` 全绿（exit 0）；Rust 87 tests + 前端 2 tests，0 failed / 0 ignored。

## 2 Goal 1 Baseline
- branch: master
- starting commit: 2c3d6d5（Goal 1 全绿：run_all exit 0）
- baseline tests: cargo test（41 Rust）+ frontend typecheck/lint/test/build + Tauri build + Playwright UI smoke 全过

## 3 Mixed Organization
- `morn-organization`: DecisionPolicyAsset（versioned decision framework）、AccountabilityChain（judged/delegated/
  verified/approved/executed/retained/revoked）、Delegation（authority scope、expiry、retained accountability、
  redelegation guard）、Workcell lifecycle state machine、member-type validation。
- `morn-actor`: RepresentationContract allow/deny/revoke。
- Tests: `crates/morn-organization` 9 + `crates/morn-actor` 3。

## 4 Durable Runtime
- `morn-work::DurableRuntime`：状态机 Draft→Ready→Running→WaitingSignal/WaitingApproval/Paused/RetryScheduled/
  Compensating/Blocked/Escalated→Completed/Failed/Cancelled（转移验证）。
- Checkpoint（JSON payload）→ process restart → load → drift check（world/artifact/contract/policy version 或
  harness availability）→ resume；drift 触发 attention（ResumeDrift/CapabilityGap）。
- Signal（human_approval/external_event/manual_resume/cancel/data_arrived/reviewer_response，idempotent by id）；
  TimerWait；RetryPolicy（max attempts/backoff/retryable/budget impact）；E2 CompensationPlan（不伪装 E3）；
  Escalation；BudgetGuard（token/money/time）；Attention Queue。
- Restart proof: `morn-store::durable_run_restart_resumes_via_sqlite`（SQLite 持久化 definition/run/checkpoint，
  重启后 drift check + signal 恢复）。
- Tests: `morn-work` durable 5 + workflow 2 + store 1（重启）。

## 5 Solution Compiler
- `morn-foundry`：
  - analyze：SolutionRequest → ProblemSpec（objective/success/constraints/assumptions/risks/evidence/
    prohibited）→ WorkGraph（WorkNode=Work 非 Agent；typed edges data/approval/state/temporal/evidence；
    cycle/impossible-dependency 检测）。
  - propose：每个 node → WorkPackage + AcceptanceSpec + ExecutionMode（nature→executor，deterministic→program）、
    MemberTypePlan、CapabilityResolution(+Gap，缺能力不虚构 tool)、HarnessPlan(+fallbacks+rationale)、
    EvaluationPlan。
  - validate：harness 兼容性、regulated/physical 必须有 approval gate、无空 work packages。
  - approve → compile：仅 ApprovedSolution 可生成 SolutionPackage（manifest JSON）；存在 unresolved gap 阻止 compile。
- Explainability：每个 planner decision 保存 `CompilerDecisionSource`（source_facts / rules / model_provider /
  assumptions / confidence / alternatives / human_review_status）。
- Tests: `crates/morn-foundry` 13（含 golden cases：CSV→报告优先 program、capability gap、incompatible harness、
  regulated approval gate、compile 需 approval）。

## 6 Work System as Code
- `ManifestService`：validate（morn/domain、problem.objective、work_packages 非空）、version（minor bump）、
  diff（domain 变更、wp added/removed）、export（JSON string）、import（roundtrip）、compatibility（domain 一致 +
  major 一致）、dry-run compile（无副作用）。
- 不自动 activate production。
- Tests: `crates/morn-foundry::manifest` 4。

## 7 Replay / Simulation / Evaluation
| Scenario | Result | Evidence |
|---|---|---|
| clean run | Pass | `EvaluationRunner::run` test |
| approval missing (E3) | Fail (policy) | `approval_missing_fails_policy` |
| tool failure | Conditional + recovery | `tool_failure_is_conditional_with_recovery` |
| regression vs previous | Fail + regression listed | `regression_detected_against_previous` |
| budget exhausted | Conditional (stop) | `budget_exhausted_stops_run` |
| replay unchanged | reproduced | `replay_reproduces_when_unchanged` |
| replay drift | deviation_detected | `replay_detects_deviation_under_drift` |
| replay isolation | no production write | `replay_never_touches_production` |

- Fault kinds: tool_timeout / tool_error / harness_crash / model_unavailable / permission_denied /
  approval_missing / untrusted_context / representation_violation / evidence_conflict / malformed_data /
  budget_exhausted / deadline_expired。
- Structured EvaluationReport：correctness / acceptance / policy / provenance / recovery / outcome /
  human_interventions / latency / cost / regressions / failures / evidence_refs / decision(pass/fail/conditional)。
- Tests: `crates/morn-assurance` 11。

## 8 Shadow
- `ShadowRunner::compare(baseline_eval, candidate_eval)`：同输入、隔离执行、比较 acceptance/policy/outcome/cost，
  输出 readiness（Ready / NotReady / Conditional）+ notes；`isolated_side_effects=true`，不执行外部动作。
- Tests: candidate worse → NotReady；equal pass → Ready；candidate conditional → Conditional。

## 9 BioLab Dream Factory
- Loop A：Literature → EvidenceMap/Summary → HypothesisSpec → ExperimentDesign → statistical/method review → PI
  approval（wet lab 前 human gate）。
- Loop B：Dataset → QC → Analysis → Artifact → Independent Review → PI Approval → Scientific Claim（Goal 1 E2E 保留）。
- Loop C：Approved Result → Figure/Table → ManuscriptDraft → ClaimEvidenceMatrix → Consistency（figure/analysis
  行数一致 + claim 证据链接，真实数据检查）→ Review → PI Approval → ReleaseCandidate。
- Wet lab：human/device placeholder + approval gate（不伪造自治）。
- 可导出资产：DomainPack / 3 WorkPackageTemplates / RoleBlueprints / EvaluationPack / SimulationScenarios /
  SolutionTemplate（`export_dream_factory_assets`）。
- Tests: `crates/morn-biolab` 5（Loop A/C + 一致性 mismatch 检测 + assets）。

## 10 UI
- Workbench v0.2：Durable Work Runtime（start/signal/attention）、Replay/Shadow/Evaluation、BioLab Loops A&C、
  E2E 运行器。
- Studio v0.2：Goal → Compiler（ProblemSpec/WorkGraph/WorkPackages/Capability/Validation）→ Approve & Compile →
  SolutionPackage Manifest。
- Console v0.2：Durable runs、Attention、Delegation & Representation、Trace/Outcomes/Promotions。
- Hub v0.2：Solution Templates / Evaluation Packs / Simulation Scenarios / Work Capability Candidates /
  Workflow Templates / Domain Packs。
- 全部真实 backend API；Playwright UI smoke 覆盖 4 surfaces + v0.2 交互 + Studio compiler 流。

## 11 Tests
```text
scripts/run_all.ps1 -> exit 0
cargo fmt --check                    -> pass
cargo check                          -> pass
cargo clippy -D warnings             -> pass
cargo test --workspace               -> pass（87 tests，0 failed，0 ignored）
frontend typecheck/lint/test/build   -> pass（2 tests）
tauri desktop build                  -> pass
frontend ui smoke (playwright)       -> pass（4 surfaces + v0.2 交互 + studio compiler）
build server binary                  -> pass
demo smoke (server + BioLab E2E)     -> pass
cargo test -p morn-app --test e2e_goal2 -> pass（goal2_full_pipeline_e2e）
```

## 12 Migrations
- Goal 1 schema v1 不变（morn_records / ledger_entries）。
- Goal 2 新增持久化记录：workflow_definition / workflow_run / checkpoint / signal（morn-store 方法 +
  SQLite 重启恢复测试）。solution/compiler/replay/evaluation/shadow 为本地服务对象（API 态），未落库（符合
  “本阶段只生成本地 package，不自动生产部署”）。

## 13 Known Failures
- KF-006（fixed）：UI smoke innerText 大小写（CSS text-transform）→ 断言改为大小写不敏感。
- KF-001（环境边界）：vitest/esbuild spawn EPERM 在 sandbox 内（审批环境通过）。
- KF-005（环境边界）：Tauri GUI 启动在 sandbox token 下被拒（沙箱外正常）。
- KF-003（deferred）：浏览器级 UI runtime QA 已由 Playwright 覆盖（不再 deferred）。

## 14 External Blockers
- B-001（Active，Goal 1 遗留）：真实 DeepSeek Harness smoke 需官方安装物或真实 API 凭据；Morn 侧 provider
  contract 通过，Goal 2 不依赖真实 DSH。

## 15 Deferred to Goal 3
- Predictive World L2/L3；企业级分布式 Durable Runtime；自动真实岗位重构/软件退役；真实机器人/仪器自治；
  多行业 Dream Factory；Managed Work / Work-as-a-Service 商业交付；ERP/MES/APS Replacement Pilot。

## 16 Next Recommended Goal
1. Goal 3a：Organization Compiler 接入真实 LLM planner（保留 rule-based 校验与 CapabilityGap 硬约束）。
2. Goal 3b：Work System as Code 升级为可安装/可 diff/可回滚的 Solution 版本管线（SolutionPackage 落库）。
3. Goal 3c：真实 DeepSeek Harness 集成（凭据就绪后），把 BioLab Loop A/B/C 作为首个跨 harness Workcell。