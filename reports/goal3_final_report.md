# Morn v10.2 Goal 3 Final Report

Date: 2026-08-15
Goal ID: `MORN-V10.2-G3-DELIVERY-EVOLUTION-REPLACEMENT`

## 1 Executive Summary

Goal 3 在 Goal 1/2 基线上增量完成 **Certified Work Capability + Evolution Flywheel v0.2 +
Deterministic Distillation + Managed Work / Outcome Delivery + BioLab Replacement Pilot（R3 Shadow Replace →
R4 Partial Replace Candidate）**，并跑通完整 E2E：
`Goal2 Solution → repeated runs → trace → Evolution Candidate → Distillation → Replay/Evaluation → Shadow →
Certification → Certified Work Capability → Managed Delivery → DeliveryReceipt → Human Acceptance →
Baseline Comparison → Partial Replace Candidate → outcome feedback`。

无 fake certification / fake acceptance / fake replacement；Shadow 不写 production；未认证 capability 不进入
Managed Work。`scripts/run_all.ps1` 全绿（exit 0）；Rust ~109 tests + 前端 2 tests，0 failed / 0 ignored。

## 2 Baseline
- branch: master；starting commit: 392ce38（Goal 2 closeout）
- Goal 1 regression: green；Goal 2 regression: green（M0 复验 + 最终 run_all）

## 3 Certification
- `morn-assurance::certification`：CertificationSpec / Run / Decision / CertifiedWorkCapability /
  CapabilityRelease；evidence（evaluation + replay + shadow）非空才能 start；evaluate 检查 acceptance rate、
  policy、recovery、reproducibility、provenance、known-failure coverage；human gate → Conditional → approve；
  version rules（patch_compatible / requires_reevaluation / full_recertification）；context-of-use enforcement；
  suspend/deprecate/retire 状态机。
- 示例：`dataset-to-reviewed-claim@1.0.0` Certified（证据 = eval PASS + replay reproduced + shadow Ready）。
- Tests: 5。

## 4 Evolution Flywheel v0.2
- `morn-evolution::flywheel`：trace ingestion、repetition detector、failure/human-correction pattern、
  high-latency/high-cost pattern、candidate generator（deterministic_distillation / harness_patch / skill /
  workflow）、evidence window、baseline metrics；candidate 为 data-only，无生产写权限。
- Tests: 4。

## 5 Deterministic Distillation
- 选择 BioLab `qc` 稳定 Actor 步骤 → 蒸馏为 `qc-rule`（rows>0 && even）；回归 3/3 匹配 + 1 个 long-tail
  fallback 回 Actor；记录 quality/latency/cost/human 对比。
- Tests: 3。

## 6 Managed Work / Outcome Delivery
- `morn-assurance::managed_work`：ManagedWorkService 只接受 Certified/Restricted capability；DeliveryLifecycle
  状态机；SloConfig（quality/deadline/acceptance method/metric/target/retry liability/human fallback/evidence/
  billing basis schema）；HumanFallback；RetryLiability（exhaustion → Escalated）；DeliveryReceipt 完整校验；
  AcceptanceDecision 独立于 executor（executor 不可自判）。
- E2E：start → deliver（receipt）→ independent reviewer accept → Closed。
- Tests: 4。

## 7 Replacement Pilot（BioLab Dataset → Reviewed Scientific Claim）
- baseline（manual）：quality .9、human 120min、cost 200、evidence .6
- native candidate（morn-native）：quality .95、human 15min、cost 40、evidence 1.0
- R3 Shadow Replace：同输入隔离比较（isolated_side_effects=true）
- R4 Partial Replace Candidate：critical 不降 + eval passed + certification passed + human approved →
  `status=approved` + rollback path（`rollback ... to existing/manual path`）；不自动 retirement。
- Tests: 5。

## 8 UI v0.3
- Workbench：Evolution Center（patterns/candidates/distillation）、Managed Work（certify+start+deliver+accept）、
  Replacement Compare（manual vs native 指标）。
- Console：Certification、Managed Deliveries、Replacement Records、Durable runs、Delegation/Representation。
- Hub：Certified Work Capabilities、Capability Releases、Replacement Records、Solution/Evaluation/Simulation assets。
- 全部真实 backend API；Playwright UI smoke 覆盖 G3 交互。

## 9 Tests
```text
scripts/run_all.ps1 -> exit 0
cargo fmt --check                    -> pass
cargo check                          -> pass
cargo clippy -D warnings             -> pass
cargo test --workspace               -> pass（~109 tests，0 failed，0 ignored）
frontend typecheck/lint/test/build   -> pass（2 tests）
tauri desktop build                  -> pass
frontend ui smoke (playwright)       -> pass（4 surfaces + v0.2 + G3 interactions + studio compiler）
build server binary                  -> pass
demo smoke (server + BioLab E2E)     -> pass
cargo test -p morn-app --test e2e_goal3 -> pass（goal3_full_pipeline_e2e）
```

## 10 Migrations
- Goal 1/2 schema 不变；Goal 3 新增能力为本地服务对象（certification/managed/replacement/flywheel/distillation），
  通过 API 暴露；未新增破坏性 migration。

## 11 Known Failures
- KF-007（fixed）：Playwright strict-mode 按钮冲突（exact 匹配）。
- KF-006（fixed）：UI smoke innerText 大小写。
- KF-001 / KF-005（环境边界）：esbuild spawn EPERM in sandbox；Tauri GUI 启动在 sandbox token 下被拒。

## 12 External Blockers
- B-001（Active，Goal 1 遗留）：真实 DeepSeek Harness smoke 需官方安装物或真实 API 凭据；Goal 3 全部使用本地
  deterministic/evaluation 证据，不依赖真实 DSH。

## 13 Deferred to Goal 4
- 企业级分布式 Durable Runtime；自动真实岗位重构/软件退役；真实机器人/仪器自治；多行业 Dream Factory；
  Managed Work 真实计费结算（billing schema 已保留）；ERP/MES/APS Replacement Pilot。

## 14 Next Recommended Goal
1. Goal 4a：把 CertifiedWorkCapability + Managed Delivery 落库（capability release history、
   delivery receipts 持久化、rollback 执行）。
2. Goal 4b：接入真实 LLM planner 生成 Evolution Candidate（保留 rule-based 校验与 human gate）。
3. Goal 4c：真实 DeepSeek Harness 集成（凭据就绪后），把 BioLab 三闭环作为首个跨 harness Managed Workcell。
---

## Re-verification Addendum (2026-08-16)

Full `scripts/run_all.ps1` re-run against current HEAD (Goals 3-5 committed + working tree): **exit 0**,
`=== Verification complete ===`.

- Rust: 64 suites, **208 passed, 0 failed, 0 ignored** (includes `goal3_full_pipeline_e2e ... ok`)
- Frontend: 2 passed (vitest); typecheck / lint / build pass
- Playwright UI smoke: 4 surfaces + BioLab E2E + Goal 2 + Goal 3 interactions + studio compiler — all OK
- demo smoke: `demo smoke OK: health=ok bootstrap_objects=1 e2e_steps=7 objects=1`
- GOAL3_ACCEPTANCE.md A-I re-checked against current code/tests; hub_v3 still exposes certified
  capabilities / capability releases / replacement records.

Regression fixes made during re-verification (see DECISIONS D-026 / KNOWN_FAILURES KF-009):
1. demo smoke now builds the server with `--all-features` (fixes `/api/biolab/run` 404 after G5 domain
   neutralization) and runs `/api/demo/bootstrap` before the BioLab E2E, asserting bootstrap-seeded
   world_objects >= 1 and work_packages >= 1.
2. `morn-app` zero-domain build warning fixed (`WorkspaceId` gated under `domain-biolab`).
