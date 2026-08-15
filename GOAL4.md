# GOAL4.md

## 北极星
把 Morn 从“能认证/托管/进化/Shadow Replace 的本地系统”推进为：

> 状态可持久化、可跨进程恢复、可执行 rollback、可以从真实 Work/Outcome 构造 Operational Episodes，并用带不确定性和适用域的 L2 预测能力辅助 Compiler / Evolution / Replacement。

## G4.1 Goal3 Production Persistence
正式持久化：CertificationSpec/Run/Decision、CertifiedWorkCapability、CapabilityRelease、ManagedWorkRun、DeliveryReceipt、AcceptanceDecision、ReplacementBaseline/Decision、ShadowComparison、EvolutionPattern/EvidenceWindow/Candidate、DistillationRecord、CapabilityLifecycleEvent。

必须：workspace isolation、immutable history、FK integrity、idempotency、migrations、restart hydration、version conflict、transaction boundary。

## G4.2 Rollback Execution v0.1
```text
RollbackRequest
→ Policy/Approval
→ CompatibilityCheck
→ activate previous controlled version/binding
→ Verify
→ RollbackReceipt
```
不删除历史；不把 E3 外部效果描述成已回滚。

## G4.3 Real Planner / Harness Seam
实现 `EvolutionPlannerProvider`：evidence/patterns/current solution → structured proposal → schema/rule/policy validator → EvolutionCandidate。

实现 `HarnessSmokeContract`：connect/health/scoped execution/action-gateway/events/provenance/teardown。
真实 DeepSeek Harness 缺凭据时保留 blocker，但不得 fake smoke。

## G4.4 OperationalEpisode
将 World/Work/Decision/Workforce/Execution/Outcome 六类 Record 汇总为版本化 Episode：context、state before/after、work graph/contract、workcell、bindings、harness/runtime、actions、artifacts、decisions、failures/retries/compensation、human corrections、acceptance/outcome、latency/cost/human effort、policy/provenance/model versions。

## G4.5 Outcome Dataset
实现 EpisodeDataset、DatasetSnapshot、DatasetManifest、LabelDefinition、FeatureSchemaVersion、SplitManifest、DataQualityReport；防 train/test leakage、future leakage、duplicate leakage、workspace boundary violation。

## G4.6 Operational State Representation
先 typed/interpretable：WorldState + WorkState + OrganizationState + ExecutionState + ResourceState + EvidenceState。先 categorical/numeric/boolean/temporal/graph-derived/aggregates/missingness，不先造大模型 embedding。

## G4.7 Predictor Registry
对象：PredictorSpec/Version、TrainingRun、EvaluationRun、CalibrationReport、ContextOfUse、Prediction、PredictionEvidence、ModelDriftReport。

首批六类：Duration、FailureRisk、Cost、HumanIntervention、OutcomeAcceptance、TransitionRisk/NextState。

所有 Prediction 必须带：value/distribution、uncertainty、context match、model version、feature schema、evidence refs、calibration status。

## G4.8 Minimum Model Strategy
优先 deterministic/historical → linear/logistic → tree/GBDT/survival/calibrated classifier；只有数据充分才比较 sequence/GNN/world-model candidates。禁止 synthetic label 冒充真实性能。

## G4.9 Prediction→Actual→Calibration
保存 decision-time prediction；真实/受控 run 后记录 actual；计算 PredictionError；更新 Calibration/Drift。禁止事后改写原 prediction。

## G4.10 Compiler + Evolution Integration
Compiler 比较 A/B/C ProposedSolution 的 outcome/risk/time/cost/human load/uncertainty，再结合 replay/simulation/human review。Prediction 只是 evidence，不是 authority。
Evolution Candidate 保存 expected delta，真实执行后保存 actual delta + error。

## G4.11 Real BioLab Pilot v1
默认 `Dataset → Reviewed Scientific Claim`。真实数据优先级：用户提供 > 仓库已有合法真实数据 > 环境允许下载的公开数据。禁止 Codex 捏造“真实数据”。

## G4.12 Operational Intelligence UI
Workbench：ETA/risk/cost/human/outcome/uncertainty + prediction vs actual。
Studio：Candidate A/B/C compare。
Console：Persistence health、Predictor Registry、Calibration/Drift、Rollback receipts。
Evolution Center：expected delta vs actual delta。
