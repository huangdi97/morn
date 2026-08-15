# Goal4 Predictor Spec

状态：Draft→Trained→Evaluated→Calibrated→Shadow→ApprovedForDecisionSupport→Restricted/Stale/Suspended/Deprecated。

统一 Prediction：predictor/version/target/value/interval_or_distribution/confidence/uncertainty/context_match/feature_schema/evidence_refs/generated_at。

六类：
1. Duration：MAE/median AE/interval coverage。
2. FailureRisk：PR-AUC/Brier/calibration；类平衡支持时再看 ROC-AUC。
3. Cost：MAE/MAPE(适用时)/interval coverage。
4. HumanIntervention：count/minutes/fallback required。
5. OutcomeAcceptance：以独立 AcceptanceDecision/verified Outcome 为标签；Brier/calibration/precision-recall。
6. TransitionRisk：先 rule/historical baseline，预测 blocked/retry/escalated next state。

数据量小：baseline-only/insufficient-data。Out-of-context 默认不能用于自动推荐。
