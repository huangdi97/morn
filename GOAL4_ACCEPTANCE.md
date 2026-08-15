# GOAL4_ACCEPTANCE.md

> 状态：2026-08-15 逐条核对。`[x]` = 通过；`[~]` = CORE 通过但 FULL blocked（真实外部数据）。
> 全量验证 `scripts/run_all.ps1` exit 0；Rust ~142 tests + 前端 2 tests，0 failed / 0 ignored。

## Baseline
- [x] Goal1/2/3 regression green（run_all exit 0）
- [x] Goal3 E2E green（goal3_full_pipeline_e2e）
- [x] frontend/Tauri/Playwright green（typecheck/lint/test/build + tauri desktop build + UI smoke）

## Persistence
- [x] Goal3 formal objects persisted（certification/managed/replacement/flywheel/distillation/rollback + opint records）
- [x] migrations fresh+upgrade（schema v2；v1→v2 ALTER 保留旧行；fresh_migration_is_v2 / upgrade_from_v1）
- [x] restart hydration（persistence_goal4::goal3_services_survive_restart；predictor params restart-proof）
- [x] workspace isolation（morn_records workspace 过滤；stable workspace on restart）
- [x] immutable release/evidence history（save_record_immutable 拒绝 overwrite；receipts/decisions/releases immutable）
- [x] idempotency/version conflict（immutable conflict rejected；persist_immutable idempotent）
- [x] API parity（全部持久化经同一 MornStore；API 行为不变）

## Rollback
- [x] request/approval/compatibility/execute/verify/receipt（RollbackService + API）
- [x] no history erasure（request+receipt 均保留）
- [x] no fake E3 rollback（receipt.e3_external_effects_preserved=true）

## Provider
- [x] EvolutionPlannerProvider + validators（parse/rule/policy/capability gate）
- [x] HarnessSmokeContract（connect/health/scoped/action-gateway/events/provenance/teardown）
- [x] real DSH smoke OR explicit active blocker（B-001 Active；Real 模式返回 blocker，不 fake）
- [x] no fake integration

## Episode/Data
- [x] six-record mapping + store + query/export（OperationalEpisode + assembler + persist/load）
- [x] DatasetSnapshot/LabelDefinition/FeatureSchema/Split/DataQuality（OutcomeDataset）
- [x] leakage checks（duplicate/future/workspace；clean dataset passes）

## State/Predictors
- [x] typed state encoder（StateEncoder 22 维可解释特征）
- [x] Duration/Failure/Cost/Human/Outcome/Transition 六类（PredictorRegistry）
- [x] model/version/context/uncertainty/evaluation/calibration（per-prediction uncertainty/interval/context；Brier calibration）

## Feedback/Integration
- [x] prediction stored before run（predict 先存，actual=None）
- [x] actual independently stored（record_actual；重复记录被拒；error 派生）
- [x] PredictionError + calibration/drift（Brier + drift_check + monitor）
- [x] Studio candidate compare（Compiler A/B 方案 + predictions as evidence；低置信度不自动选择）
- [x] Compiler prediction evidence（E2E 中 predictions 作为候选证据）
- [x] Evolution expected-vs-actual（flywheel candidate expected benefit vs actual prediction_error）
- [x] Workbench/Console real backend（opint API + UI）

## Pilot
CORE:
- [x] adapter/manifest/provenance/validation（PilotManifest + RealPilotService + tests）
- [x] no fabricated real data（synthetic source 拒绝注册）
FULL:
- [~] lawful real dataset -> EXTERNAL BLOCKED（G4-B-002：无合法真实 dataset；不捏造）
- [ ] real pilot E2E / actual outcome / prediction vs actual / calibration（待真实数据）

## Quality
- [x] fmt/clippy/lint/unit/integration/contract/E2E/frontend/docs（run_all exit 0）
- [x] reports/goal4_final_report.md 已生成

## 禁止项（逐条核对均不成立）
- 无 fake persistence（SQLite canonical SoR；restart 验证）
- 无 synthetic label 冒充真实 Outcome（labels 来自独立 AcceptanceDecision/terminal status）
- 无 fake DSH（B-001 Active；Real 返回 blocker）
- 无 hard-coded predictor pass（insufficient-data 拒绝预测）
- 无 training score 冒充 test（temporal split）
- 无 data leakage（duplicate/future/workspace 检测）
- 无 uncertainty 缺失（每 prediction 带 uncertainty/interval/confidence）
- predictor 不自动批准生产（低置信度/out-of-context 不可自动决策）
- 无破坏性 rollback（history 保留；E3 preserved）