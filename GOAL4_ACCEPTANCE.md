# Goal4 Acceptance

## Baseline
- [ ] Goal1/2/3 regression green
- [ ] Goal3 E2E green
- [ ] frontend/Tauri/Playwright green where environment permits

## Persistence
- [ ] Goal3 formal objects persisted
- [ ] migrations fresh+upgrade
- [ ] restart hydration
- [ ] workspace isolation
- [ ] immutable release/evidence history
- [ ] idempotency/version conflict
- [ ] API parity

## Rollback
- [ ] request/approval/compatibility/execute/verify/receipt
- [ ] no history erasure
- [ ] no fake E3 rollback

## Provider
- [ ] EvolutionPlannerProvider + validators
- [ ] HarnessSmokeContract
- [ ] real DSH smoke OR explicit active blocker
- [ ] no fake integration

## Episode/Data
- [ ] six-record mapping + store + query/export
- [ ] DatasetSnapshot/LabelDefinition/FeatureSchema/Split/DataQuality
- [ ] leakage checks

## State/Predictors
- [ ] typed state encoder
- [ ] Duration/Failure/Cost/Human/Outcome/Transition
- [ ] model/version/context/uncertainty/evaluation/calibration

## Feedback/Integration
- [ ] prediction stored before run
- [ ] actual independently stored
- [ ] PredictionError + calibration/drift
- [ ] Studio candidate compare
- [ ] Compiler prediction evidence
- [ ] Evolution expected-vs-actual
- [ ] Workbench/Console real backend

## Pilot
CORE:
- [ ] adapter/manifest/provenance/validation
- [ ] no fabricated real data
FULL:
- [ ] lawful real dataset
- [ ] real pilot E2E
- [ ] actual outcome
- [ ] prediction vs actual
- [ ] calibration

## Quality
- [ ] fmt/clippy/lint/unit/integration/contract/E2E/frontend/docs
- [ ] reports/goal4_final_report.md

禁止：fake persistence、synthetic label 冒充真实 Outcome、fake DSH、hard-coded predictor pass、training score 冒充 test、data leakage、无 uncertainty、predictor 自动批准生产、破坏性 rollback。
