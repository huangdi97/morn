# STATUS_GOAL2.md

Goal: MORN-V10.2-G2-SOLUTION-FACTORY
Status: IN PROGRESS — M0 done

## Goal 1 Baseline
- branch: master
- commit: 2c3d6d5 (A10 Playwright UI smoke; Goal 1 full green)
- backend regression: PASS — `scripts/run_all.ps1` exit 0 (fmt/check/clippy -D warnings/test; 41 Rust tests, 0 failed/ignored)
- frontend regression: PASS — typecheck/lint/test(2)/build; Playwright UI smoke 4 surfaces + BioLab E2E, 0 console errors
- desktop: PASS — `cargo build -p morn-desktop` (Tauri v2), binary launches outside sandbox
- dup check: Delegation/RepresentationContract/ShadowRun ids exist; missing DecisionPolicyAsset/SolutionCompiler/ProblemSpec/WorkGraph/ReplayRun/EvaluationRun/WorkflowRun/Signal/RetryPolicy/Escalation/BudgetGuard -> Goal 2 adds them (no rework)

## Milestones
- [x] M0 Verify Goal 1
- [ ] M1 Mixed Organization Completion
- [ ] M2 Durable Work Runtime v0.2
- [ ] M3 ProblemSpec + WorkGraph
- [ ] M4 Solution Compiler v0.2
- [ ] M5 Work System as Code
- [ ] M6 Replay / Simulation / Evaluation
- [ ] M7 Shadow
- [ ] M8 BioLab Dream Factory v0.1
- [ ] M9 Product Surfaces v0.2
- [ ] M10 E2E + Closeout

## Current Focus
M1 Mixed Organization Completion.

## Compiler Progress
ProblemSpec / WorkGraph / ExecutionMode / MemberType / CapabilityResolver / HarnessPlan / EvaluationPlan / SolutionPackage。

## Durable Progress
checkpoint / signal-wait / retry / compensation / escalation / budget / attention / drift。

## Replay/Eval
replay / simulation / shadow / regression。

## BioLab
Loop A / Loop B / Loop C。

## UI
Workbench / Studio / Console / Hub。

只有 GOAL2_ACCEPTANCE 全部本地可完成项通过后，才可标记 DONE。