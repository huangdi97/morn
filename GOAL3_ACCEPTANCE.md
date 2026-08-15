# GOAL3_ACCEPTANCE.md

> 状态：2026-08-15 逐条核对，2026-08-16 全量 re-verify。`[x]` = 通过（代码 + 测试 + 命令）。全量验证 `scripts/run_all.ps1` exit 0；
> Rust 208 tests（64 suites，含 goal3_full_pipeline_e2e）+ 前端 2 tests，0 failed / 0 ignored。

## A Baseline
- [x] Goal 1 regression green（run_all exit 0：cargo test / typecheck / lint / build / Tauri / UI smoke / demo smoke）
- [x] Goal 2 regression green（compiler/durable/replay/eval/shadow/BioLab loops 全部保持）

## B Certification
- [x] CertificationSpec（capability/version/context-of-use/suites/acceptance rate/policy/recovery/reproducibility/human gate/provenance/failure coverage）
- [x] CertificationRun（evidence 非空才能开始）
- [x] CertificationDecision（Certified/Conditional/Failed/Restricted + reasons）
- [x] CertifiedWorkCapability（WP template + workcell + role/harness + workflow + acceptance + outcome + eval pack + deployment + evidence）
- [x] CapabilityRelease（仅 Certified/Restricted 可 release）
- [x] version recertification rules（patch_compatible / requires_reevaluation / full_recertification）
- [x] context-of-use enforcement（Restricted 只在允许 context 运行）
- 禁止 fake certification：insufficient evidence 无法 certify、failed critical policy 无法 certify、human gate 必须 approval（测试覆盖）

## C Evolution
- [x] trace/evidence ingestion（ingest_trace / ingest_human_correction）
- [x] repetition detector（repeated success/failure/approval）
- [x] failure/human-correction pattern（repeated failure / repeated human correction）
- [x] candidate generator（deterministic_distillation / harness_patch / skill / workflow）
- [x] evidence window（candidate.source_evidence_window 引用 trace ids）
- [x] replay/eval/shadow/certification path（E2E 串通）
- [x] no direct production mutation（FlywheelCandidate 无写方法；E2E 验证）

## D Distillation
- [x] one Actor step distilled（BioLab qc）
- [x] deterministic implementation（qc-rule: rows>0 && even）
- [x] regression（program vs actor 3/3 match，quality 不降）
- [x] Actor fallback（special/edge input -> actor）
- [x] cost/latency/quality comparison（RegressionReport：quality_delta / latency_reduction / cost_reduction / human_load_reduction）

## E Managed Work
- [x] ManagedWorkService
- [x] DeliveryLifecycle（Requested→…→Delivered→Accepted/Rejected→Closed，状态机验证）
- [x] SLO（SloConfig：quality/deadline/acceptance method/metric/target/retry liability/human fallback/evidence/billing basis）
- [x] HumanFallback（trigger/required role/handoff/authority/resume）
- [x] RetryLiability（max retries；exhaustion -> Escalated）
- [x] DeliveryReceipt（完整字段校验）
- [x] AcceptanceDecision（Automatic/IndependentReviewer/HumanCustomer/Hybrid；executor 不可自判）

## F Replacement Pilot
- [x] baseline profile（WorkVariantMetrics manual）
- [x] observed work graph（ObservedWorkGraph）
- [x] Morn orchestrated variant（指标记录）
- [x] native candidate（morn-native metrics）
- [x] R3 Shadow Replace（shadow_compare isolated）
- [x] comparison metrics（quality/acceptance/cycle/human/retries/rework/cost/evidence/policy/outcome）
- [x] R4 PartialReplaceCandidate gate（critical 不降 + eval + cert + human approval）
- [x] rollback（r4.rollback_path -> existing/manual；不自动 retirement）

## G UI
- [x] Evolution Center（Workbench：patterns/candidates/distillation）
- [x] certification view（Console：certified capabilities）
- [x] managed work dashboard（Workbench + Console：delivery queue、SLO、lifecycle、acceptance）
- [x] replacement compare（Workbench：manual vs native 指标）
- [x] Hub certified asset view（hub3：capabilities/releases/records）
- [x] no fake must-pass data（全部真实 API；Playwright 验证）

## H E2E
- [x] trace → candidate（goal3_full_pipeline_e2e）
- [x] evaluation（PASS）
- [x] certification（Certified）
- [x] managed work（start → deliver → receipt）
- [x] delivery receipt（complete）
- [x] acceptance（independent reviewer）
- [x] replacement comparison（candidate meets critical）
- [x] feedback → new evolution candidate（delivery outcome 回流 flywheel）

## I Quality
- [x] fmt/lint（cargo fmt --check / clippy -D warnings / eslint）
- [x] unit/integration（~109 Rust tests）
- [x] Goal1+2 regression（run_all exit 0）
- [x] frontend build（npm run build）
- [x] final report（reports/goal3_final_report.md）

## 不算完成（逐条核对均不成立）
- 无 fake certification（证据不足/失败 policy 无法 certify）
- 无 fake acceptance（executor 不可自判；独立 reviewer）
- 无 fake replacement（比较真实指标；R4 需 eval+cert+human）
- Shadow 不写 production（isolated_side_effects=true）
- 未认证 capability 不进 Managed Work（start 校验）
- 无 ignored tests / hard-coded pass / 弱化断言（0 ignored）