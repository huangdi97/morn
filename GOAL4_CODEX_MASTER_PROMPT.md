# Morn v10.2 Goal 4 — Codex MASTER 中文总指令

Goal 1、Goal 2、Goal 3 已完成。现在执行 `MORN-V10.2-G4-PERSISTENCE-REAL-OUTCOME-OPINT`。

## 必读顺序
AGENTS.md；现有设计母版；Goal1/2 reports；`reports/goal3_final_report.md`；GOAL4.md；GOAL4_PLAN.md；Persistence/Data/Predictor/RealProvider/RealPilot/UI/Test/Acceptance；STATUS_GOAL4.md；DECISIONS/BLOCKERS/KNOWN_FAILURES；然后实际代码/schema/migrations/repositories/API/frontend。

## 先验证真实基线
运行 Goal1-3 regression、Goal3 E2E、frontend/Tauri/Playwright；记录 branch/HEAD/git status。若 regression 先修。

## 不要重做 Goal3
Goal3 已有 certification/flywheel/distillation/managed work/replacement/UI。Goal4 首先把它们生产化、持久化，不复制一套 v2 domain type。

## 连续执行 M0→M12
Baseline→Persistence Schema→Service Parity→Rollback→Provider Seams→Episode→Dataset→State Encoder→Predictor Registry→Integration→Real Pilot→Drift→Closeout。除真实外部 blocker 外不要问我下一步。

## Persistence 是 P0
必须 one canonical persistent SoR；migration、repository、restart proof、idempotency、versioning、workspace isolation、immutable history。禁止 DB 影子副本 + in-memory 真相并存。

## Rollback
Request→Approval→Compatibility→Activate previous Morn-controlled version/binding→Verify→Receipt。不得删除历史，不得把 E3 外部效果说成已回滚。

## Provider
真实 LLM planner 输出只形成 candidate proposal，必须 schema/rule/policy/human/evaluation gate。DeepSeek Harness 若仍缺官方安装物/API secret，保留 blocker，不 fake smoke，不停止其余 Goal4。

## Episode/Data
训练/预测数据只来自 authoritative Morn records。禁止 LLM 自评 label、synthetic outcome 冒充真实、post-outcome/future leakage。

## Predictors
先可信 baseline，不为“世界模型”名义增加复杂度。必须实现 Duration、FailureRisk、Cost、HumanIntervention、OutcomeAcceptance、TransitionRisk；每类有 version/context/evaluation/uncertainty/calibration（适用时）。数据不足就 `insufficient-data`，不要伪造性能。

## Compiler/Evolution
Prediction 是 evidence，不是 authority。低置信度/域外必须 restricted/human attention，不自动选择/部署。

## Real Pilot
优先真实合法 BioLab dataset。没有真实数据就完成 adapter/manifest/provenance/validation，把 FULL COMPLETE 标 blocked，严禁捏造真实数据。CORE 可继续。

## 状态与测试
每个 milestone：contract/test→implementation→persistence→integration→UI→regression→STATUS。持续更新 STATUS_GOAL4.md、DECISIONS.md、BLOCKERS.md、KNOWN_FAILURES.md。

最终逐条通过 GOAL4_ACCEPTANCE.md，运行 Goal1-3 full regression，生成 `reports/goal4_final_report.md`，明确 CORE COMPLETE / FULL COMPLETE / EXTERNAL BLOCKED。

最终必须证明：
```text
Persisted Certified Capability
→ Managed Delivery
→ restart/recovery
→ OperationalEpisode
→ OutcomeDataset
→ Predictor
→ Prediction
→ Candidate Comparison
→ Execute
→ Actual Outcome
→ PredictionError
→ Calibration
→ EvolutionCandidate
```

现在直接开始，不要只给计划。
