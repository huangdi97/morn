# Morn v10.2 Goal 3 — Codex 总指令

Goal 1 和 Goal 2 已完成。

现在执行 Goal 3：**Certified Work Capability + Managed Work / Outcome Delivery + Evolution Flywheel + Replacement Pilot**。

Goal ID：`MORN-V10.2-G3-DELIVERY-EVOLUTION-REPLACEMENT`

## 必读
1. AGENTS.md
2. docs/spec/Morn_v10.2-R1_全量设计母版.md
3. Goal 1 final report
4. Goal 2 final report
5. GOAL3.md
6. GOAL3_PLAN.md
7. GOAL3_CERTIFICATION_SPEC.md
8. GOAL3_EVOLUTION_FLYWHEEL.md
9. GOAL3_MANAGED_WORK_SPEC.md
10. GOAL3_REPLACEMENT_PILOT.md
11. GOAL3_UI_SPEC.md
12. GOAL3_TEST_PLAN.md
13. GOAL3_ACCEPTANCE.md
14. STATUS_GOAL3.md
15. DECISIONS.md
16. BLOCKERS.md
17. KNOWN_FAILURES.md

## 先验证已有工程
先运行 Goal 1 + Goal 2 regression。不要重做已完成模块。

## 连续执行
按 GOAL3_PLAN.md：M0 Verify baseline → M1 Certification → M2 Evolution Flywheel → M3 Deterministic Distillation → M4 Managed Work → M5 Replacement Baseline → M6 Shadow Replace → M7 Partial Replace Decision → M8 UI → M9 E2E/Closeout。

除真实外部 blocker，不要停下来问我下一步。

## 核心约束
1. Certified Work Capability 不是 Agent：认证对象必须包含 Work/Workflow/Acceptance/Outcome/Evaluation/Deployment context。
2. Evolution 不能自动改生产：Candidate → Branch → Replay/Evaluation → Shadow → Certification → Promotion。
3. 最小充分智能：必须完成至少一个 deterministic distillation 示例，不要把“进化”理解为创建更多 Agent。
4. Managed Work 必须来自 Certified Capability。
5. Delivery Acceptance 独立于执行者，执行 Actor 不能自己宣布客户/科研 Outcome 通过。
6. Replacement 基本单位是 Work，不要试图重写 ERP/MES/全部实验室软件。
7. Shadow 无真实副作用。
8. R4 只生成 PartialReplaceCandidate + evidence + human decision，不自动 retirement。

## 默认 Replacement Pilot
优先 `BioLab Dataset → Reviewed Scientific Claim`，除非仓库已有更成熟且更可量化的具体 Work。

## 最终 E2E
Goal2 Solution → repeated runs → trace/outcome → Evolution Candidate → deterministic/workflow improvement → replay/evaluation → shadow → certification → Certified Work Capability → Managed Work → DeliveryReceipt → Human Acceptance → baseline comparison → Partial Replace Candidate → outcome feedback。

## 测试
不得 ignored test、hard-coded pass、fake certification、fake customer acceptance、fake replacement、shadow 写 production、未认证 capability 进入 Managed Work。

持续修到 GOAL3_ACCEPTANCE.md 所有本地可完成项通过。

## 文档
持续更新 STATUS_GOAL3.md、DECISIONS.md、BLOCKERS.md、KNOWN_FAILURES.md。最终生成 `reports/goal3_final_report.md`。

现在直接开始，不要只给计划。
