# Morn v10.2 Goal 2 — Codex 总指令

Goal 1 已完成。现在负责 Goal 2：**Organization/Solution Compiler + Durable Work Runtime + Simulation/Evaluation + BioLab Dream Factory v0.1**。

不要重建 Goal 1，不要创建空壳模块；在当前真实仓库上增量开发。

## 必读
1. 当前 `AGENTS.md`
2. `docs/spec/Morn_v10.2-R1_全量设计母版.md`
3. `GOAL2_PRECONDITIONS.md`
4. `GOAL2.md`
5. `GOAL2_ARCHITECTURE_FREEZE.md`
6. `GOAL2_PLAN.md`
7. `GOAL2_COMPILER_SPEC.md`
8. `GOAL2_DURABLE_RUNTIME_SPEC.md`
9. `GOAL2_SIMULATION_EVAL_SPEC.md`
10. `GOAL2_BIOLAB_DREAM_FACTORY.md`
11. `GOAL2_UI_SPEC.md`
12. `GOAL2_TEST_PLAN.md`
13. `GOAL2_ACCEPTANCE.md`
14. `STATUS_GOAL2.md`
15. `DECISIONS.md` / `BLOCKERS.md` / `KNOWN_FAILURES.md`
16. Goal 1 final report（若存在）和现有代码/测试/migrations。

## M0 先验证 Goal 1
严格执行 `GOAL2_PRECONDITIONS.md`。Goal 1 regression 先修，再进入 Goal 2。

## 连续执行 M1→M10
```text
M1 Mixed Organization Completion
M2 Durable Work Runtime v0.2
M3 ProblemSpec + WorkGraph
M4 Solution Compiler v0.2
M5 Work System as Code
M6 Replay / Simulation / Evaluation
M7 Shadow
M8 BioLab Dream Factory v0.1
M9 Product Surfaces v0.2
M10 E2E + Closeout
```
除真实外部 blocker 外，不要每阶段停下来问是否继续。

## 强制继承 Goal 1 不变量
Stable Semantic Kernel、Canonical State 受治理写路径、Artifact immutable version、AcceptanceSpec、Harness/Runtime 中立、Action Gateway、E0-E3、Production/Evolution separation、Runtime Adapter 不拥有正式业务对象。

## Compiler 必须是真 pipeline
```text
Request → ProblemSpec → WorkGraph → WorkPackage → Acceptance/Outcome → ExecutionMode → Role/Member → Capability Resolution/Gap → HarnessPlan → Workflow → EvaluationPlan → ProposedSolution → Validation → Human Review → SolutionPackage
```
禁止：一句话只输出几个 Agent；缺能力时发明 Tool；deterministic work 强制 Agent；high-risk 无 approval；compile 后直接 deploy production。

每个 planner decision 保存依据。

## Durable
必须有 long-running WorkflowRun、checkpoint、signal/wait、retry、compensation、escalation、budget、attention、restart/resume、drift detection。

## Replay/Simulation/Shadow
必须使用隔离 state。真实高风险外部 Action 在这些模式下必须 dry-run/sandbox/fixture/mock external boundary/no-op 之一。

## BioLab
完成三闭环：
1. Literature → Evidence → Hypothesis → Experiment Design
2. Dataset → QC → Analysis → Independent Review → Scientific Claim
3. Approved Result → Figure/Table → Manuscript → Claim-Evidence Consistency

湿实验保持 Human/Device gate；BioLab 不能污染 Kernel。

## UI
Studio：Goal→Compiler→Review→Manifest。
Workbench：long-running run / WorkGraph / wait / attention / replay / shadow。
Console：delegation / representation / durable / evaluation / shadow。
Hub：solution/domain/evaluation/simulation assets。
必须接真实 backend API。

## 测试纪律
每 Milestone：contract test → implementation → target tests → regression → update status。
不得删除/ignore/弱化测试，不得把 fake resolved 当 capability success。

## 文档
持续更新 `STATUS_GOAL2.md`、`DECISIONS.md`、`BLOCKERS.md`、`KNOWN_FAILURES.md`。
最终生成 `reports/goal2_final_report.md`，包含 baseline commit、changes、migrations、compiler examples、durable recovery proof、replay/simulation/shadow、BioLab 3 loops、UI routes、test results、blockers、deferred Goal 3。

## 只允许请求真人的情况
真实 Secret/Credential、外部账号、许可证/付费、不可逆外部操作、数据丢失风险、两个强约束冲突且自动选择会造成不可逆迁移。

普通编译错误、测试失败、依赖冲突、UI bug、类型错误，自己继续修。

## 现在开始
读取文件 → M0 baseline → 更新 STATUS_GOAL2 → M1→M10 → 逐条通过 GOAL2_ACCEPTANCE → final report。不要只输出计划，直接工作。
