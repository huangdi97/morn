# STATUS_GOAL3.md

Goal: MORN-V10.2-G3-DELIVERY-EVOLUTION-REPLACEMENT
Status: COMPLETE — M0..M9 done; Goal1+2 regression green; final report generated

## Baseline
- Goal 1: green（scripts/run_all.ps1 exit 0）
- Goal 2: green（87 Rust tests at G2 close）
- branch: master
- commit at start: 392ce38
- M0 re-check: cargo test --workspace 全绿，无回归

## Milestones
- [x] M0 Verify baseline
- [x] M1 Certification Model（CertificationSpec/Run/Decision/CertifiedWorkCapability/CapabilityRelease，context-of-use，version recertification rules）
- [x] M2 Evolution Flywheel v0.2（trace/pattern/candidate with evidence windows；candidate 无生产写权限）
- [x] M3 Deterministic Distillation（qc actor step -> deterministic program + actor fallback + regression 比较）
- [x] M4 Managed Work / Outcome Delivery（certified-only start、SLO、human fallback、retry liability、DeliveryReceipt、acceptance 独立于 executor）
- [x] M5 Replacement Baseline（ExistingSystemMapping + ObservedWorkGraph + variant metrics）
- [x] M6 Shadow Replace（isolated baseline vs candidate comparison）
- [x] M7 Partial Replace Decision（R4 gate：quality/safety 不降 + eval/cert + human approval + rollback；不自动 retirement）
- [x] M8 Product Surfaces（Evolution Center、Managed Work、Certification、Replacement Compare、Hub certified assets）
- [x] M9 E2E + Closeout（goal3_full_pipeline_e2e；Goal1+2 regression green；reports/goal3_final_report.md）

## Certification
- candidate: dataset-to-reviewed-claim@1.0.0（Certified）
- run: CertificationRun（evidence = evaluation PASS + replay reproduced + shadow Ready）
- decision: Certified（无 critical gap；证据充分）
- release: CapabilityRelease（schema 就绪）

## Evolution
- trace miner: EvolutionFlywheel::ingest_trace
- pattern detector: detect_patterns（repeated success/failure/human correction/approval/high latency/high cost）
- human correction: ingest_human_correction
- candidate generator: generate_candidates（deterministic_distillation / harness_patch / skill / workflow）

## Distillation
- selected step: qc（BioLab）
- deterministic candidate: qc-rule（rows>0 && even）
- regression: passed（3/3 match + 1 long-tail fallback）
- fallback: special/edge input -> actor

## Managed Work
- service: ManagedWorkService
- delivery: Requested→…→Delivered→Accepted→Closed（状态机验证）
- SLO: reproducibility=1.0, acceptance=independent review
- receipt: DeliveryReceipt（完整字段校验）
- acceptance: IndependentReviewer（executor 不可自判）

## Replacement
- baseline: manual（quality .9, human 120min, cost 200）
- shadow: candidate 优于 baseline（isolated）
- metrics: quality/acceptance/cycle/human/cost/evidence/policy/outcome
- decision: R4 approved + rollback path（不 retirement）

## UI
- Workbench: Evolution Center、Managed Work、Replacement Compare（真实 API）
- Evolution Center: patterns/candidates/distillation
- Console: Certification、Managed Deliveries、Replacement Records
- Hub: Certified Work Capabilities、Capability Releases、Replacement Records

## Latest Tests
```text
scripts/run_all.ps1 -> exit 0
cargo fmt/check/clippy(-D warnings)/test -> pass（~109 Rust tests，0 failed/ignored）
frontend typecheck/lint/test/build -> pass
tauri desktop build -> pass
frontend ui smoke (playwright) -> pass（4 surfaces + v0.2 + G3 interactions + studio compiler）
server build + demo smoke -> pass
cargo test -p morn-app --test e2e_goal3 -> pass（goal3_full_pipeline_e2e）
```