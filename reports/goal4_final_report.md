# Morn v10.2 Goal 4 Final Report

Date: 2026-08-15
Goal ID: `MORN-V10.2-G4-PERSISTENCE-REAL-OUTCOME-OPINT`

## 1 Summary

Goal 4 完成 **CORE COMPLETE**：Goal3 正式对象全部持久化（schema v2 canonical SoR + restart hydration +
immutable history + idempotency）、Rollback Execution、Real Provider Seams（EvolutionPlannerProvider +
HarnessSmokeContract）、OperationalEpisode / OutcomeDataset / typed StateEncoder / 六类 Predictor（Duration、
FailureRisk、Cost、HumanIntervention、OutcomeAcceptance、TransitionRisk）带 uncertainty/calibration/drift、
Prediction→Actual→PredictionError→Calibration 闭环、Compiler/Evolution 集成与 Operational Intelligence UI。

**FULL COMPLETE 不成立**：真实 BioLab 数据 pilot 因无合法真实 dataset 而 EXTERNAL BLOCKED（G4-B-002），
禁止捏造真实数据；adapter/manifest/provenance/validation 已完成。真实 DeepSeek Harness smoke 仍为外部/凭据
blocker（B-001），HarnessSmokeContract + fixture 全绿，不 fake。

`scripts/run_all.ps1` 全绿（exit 0）；Rust ~142 tests + 前端 2 tests，0 failed / 0 ignored。

## 2 Baseline
- branch: master；starting commit: d8cbe12（Goal 3 closeout）
- Goal1/2/3 regression: green（M0 + 最终 run_all）

## 3 Persistence（M1-M2）
- schema v2：`morn_records` 增加 immutable 列 + v1→v2 migration（保留旧行）。
- 持久化对象：certification(spec/run/decision/capability/release)、managed(run/receipt/acceptance)、
  replacement(comparison/record/r4)、flywheel(pattern/candidate/trace)、distillation、rollback、opint
  (episode/prediction/predictor params)。
- write-through `persist_all` + restart `hydrate_all`（稳定 workspace id）；immutable receipts 拒绝 overwrite。
- Proof: `morn-store` 10 tests；`persistence_goal4` 2 tests（restart hydration、immutable）。

## 4 Rollback Execution（M3）
- RollbackRequest→Approval（requester != approver）→Compatibility（previous ∈ known releases）→Execute→Verify→
  RollbackReceipt；history 保留；receipt 标记 e3_external_effects_preserved=true（不宣称 E3 外部效果已回滚）。
- Proof: `morn-assurance::rollback` 4 tests + API `/api/rollback/*`。

## 5 Real Provider Seams（M4）
- `EvolutionPlannerProvider`（DeterministicPlannerProvider 测试 provider）→ 结构化 proposal →
  ProposalValidator（schema parse → rule → policy → capability/authority）→ FlywheelCandidate；proposal 永不改 production。
- `HarnessSmokeContract`：connect/health/scoped execution/action-gateway mediation/event normalization/
  provenance/teardown；MornNative + DSH(Fixture) 全过；DSH(Real) 返回 blocker 报告。
- Proof: `morn-evolution::planner` 4 + `morn-harness::smoke` 3 tests。

## 6 OperationalEpisode + OutcomeDataset（M5-M6）
- `OperationalEpisode`：six-record mapping（world/work/organization/execution/evidence/outcome）+ economics +
  labels（success/accepted/failure_type/escalation/rework）；labels 来自独立 AcceptanceDecision/terminal status。
- `OutcomeDataset`：DatasetSnapshot（immutable）、DatasetManifest（source/license/checksum）、LabelDefinition、
  FeatureSchemaVersion、SplitManifest（temporal）、DataQualityReport（duplicate/future/workspace leakage）。
- Proof: `morn-opint::episode` 2 + `dataset` 3 tests。

## 7 State Representation + Predictors（M7-M8）
- `StateEncoder`：22 维 typed/interpretable 特征（world/work/org/execution/resource/evidence），无 embedding。
- `PredictorRegistry` 六类：Duration（mean±std）、FailureRisk（rate+SE）、Cost、HumanIntervention、
  OutcomeAcceptance（Brier/calibration）、TransitionRisk；每 prediction 带 value/interval/uncertainty/confidence/
  context_match/feature_schema/evidence_refs；insufficient-data（<3 样本）拒绝预测；snapshot/restore 持久化。
- Proof: `morn-opint::predictor` 7 tests。

## 8 Prediction→Actual→Calibration→Drift（M9/M11）
- predict() 先存（actual=None）→ record_actual() 独立记录 + PredictionError（重复被拒）→ calibrate()（Brier）→
  drift_check()（calibration_or_context）→ monitor()（calibration trend、schema mismatch、out-of-context、stale、
  recalibration candidate）。
- Proof: E2E + predictor tests + API `/api/opint/*`。

## 9 Compiler/Evolution Integration（M9）
- Compiler 生成 A/B 方案；predictions 作为 evidence（非 authority）；低置信度/out-of-context 不可自动选择；
  compile 仍需 human approval。
- Evolution：flywheel candidate expected benefit vs actual prediction_error（expected-vs-actual）。
- UI：Workbench Operational Intelligence（train & predict + interval/confidence/context）、Console Predictor
  Registry & Persistence（episodes/snapshots/rollback receipts/insufficient-data）。

## 10 Real BioLab Pilot（M10）
- `PilotManifest`（source/license/checksum 必填；synthetic 拒绝）+ `RealPilotService` pipeline contract。
- 无合法真实 dataset → run_pipeline 返回 External blocker（G4-B-002）；CORE（adapter/manifest/provenance/
  validation）全部通过；禁止捏造真实数据。
- Proof: `morn-opint::pilot` 2 tests。

## 11 Tests
```text
scripts/run_all.ps1 -> exit 0
cargo fmt --check                    -> pass
cargo check                          -> pass
cargo clippy -D warnings             -> pass
cargo test --workspace               -> pass（~142 tests，0 failed，0 ignored）
frontend typecheck/lint/test/build   -> pass（2 tests）
tauri desktop build                  -> pass
frontend ui smoke (playwright)       -> pass（4 surfaces + v0.2/v0.3 interactions + studio compiler）
server build + demo smoke            -> pass
cargo test -p morn-app --test e2e_goal4   -> pass（goal4_full_pipeline_e2e）
cargo test -p morn-app --test persistence_goal4 -> pass（restart hydration + immutable）
```

## 12 Migrations
- schema v1 → v2：`ALTER TABLE morn_records ADD COLUMN immutable INTEGER NOT NULL DEFAULT 0`（幂等）；旧行保留。
- 无破坏性迁移；opint/rollback 记录复用 canonical `morn_records`。

## 13 Known Failures
- KF-001 / KF-005 / KF-006 / KF-007（既有环境边界与 smoke 修复）保持。
- 无新增产品缺陷。

## 14 External Blockers
- B-001（Active）：真实 DeepSeek Harness smoke 缺官方安装物/API 凭据；contract + fixture 全绿，不 fake。
- G4-B-002（Active）：真实 BioLab 数据 pilot FULL blocked —— 无合法真实 dataset；CORE adapter/manifest/
  provenance/validation 已完成，禁止捏造真实数据。

## 15 Completion Status
- **CORE COMPLETE**：是（persistence/rollback/provider seams/episode/dataset/state/predictors/calibration/
  drift/UI 全部实现、测试、持久化、restart-proof）。
- **FULL COMPLETE**：否（真实 BioLab 数据 pilot 与真实 DSH smoke 为外部 blocker）。
- **EXTERNAL BLOCKED**：G4-B-002（真实数据）+ B-001（DSH 凭据）。

## 16 Deferred to Goal 5
- L3 因果/反事实预测；更强模型（仅数据充分后比较 sequence/GNN/world-model）；企业分布式 durable runtime；
  Managed Work 真实计费；多行业 Dream Factory；ERP/MES/APS Replacement Pilot。

## 17 Next Recommended Goal
1. Goal 5a：接入真实 BioLab dataset（用户提供或经授权公开数据）跑 FULL pilot：snapshot → pre-run predictions →
   managed work → actual → error → calibration。
2. Goal 5b：真实 LLM planner（凭据就绪后）经 ProposalValidator 生成 EvolutionCandidate；真实 DSH reconnect/
   health/scoped work smoke。
3. Goal 5c：Predictor 升级（数据充分后引入 calibrated classifier / GBDT / survival），保留 uncertainty 与
   context-of-use 门。