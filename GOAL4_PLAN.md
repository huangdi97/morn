# GOAL4_PLAN.md — M0→M12

M0 Baseline & Reality Check：Goal1-3 full regression、Goal3 E2E、UI/Tauri；定位哪些 Goal3 对象仍 in-memory。

M1 Persistence Schema：migration + repository + fresh/upgrade DB tests。

M2 Persistence Service Parity：现有 Goal3 service 改 repository-backed；restart、isolation、idempotency、immutable history、version conflict、API parity。

M3 Rollback Execution：request→approval→compatibility→activate previous controlled version→verify→receipt。

M4 Real Provider Seams：EvolutionPlannerProvider + HarnessSmokeContract；DSH 外部阻塞不停止本地核心。

M5 OperationalEpisode Store：assembler/store/version/query/export/provenance。

M6 OutcomeDataset：snapshot/labels/features/splits/quality/leakage checks。

M7 State/Feature Representation：deterministic typed encoder，覆盖 work/org/execution/resource/evidence/time/missingness。

M8 Predictor Registry + Baselines：Duration/Failure/Cost/Human/Outcome/Transition 六类；评测、适用域、不确定性、校准。

M9 Prediction Integration：Studio compare、Compiler evidence、Evolution expected delta、Workbench、Console。

M10 Real BioLab Pilot：真实数据可得则 dataset→pre-run prediction→managed work→reviewed claim→actual→error→calibration；否则只完成 adapter/manifest/validation 并标 FULL BLOCKED。

M11 Drift/Monitoring：calibration trend、schema mismatch、out-of-context、outcome drift、stale flag、recalibration candidate；不自动在线更新生产模型。

M12 Full E2E + Closeout：Persisted Capability→Managed Delivery→Episode→Dataset→Predict→Compare→Execute→Actual→Error→Calibration→EvolutionCandidate；生成 reports/goal4_final_report.md。
