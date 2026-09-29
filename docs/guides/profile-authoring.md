# Profile Authoring Guide（Morn v11.5-R1）

Profile 是 guarantee requirements，不是 provider 名单。

## 现有 Profiles

```text
morn.lite                 # 本地/个人：durable_work_state + accepted_outcome_semantics
morn.enterprise           # 全保证链 + observability
morn.factory.readonly     # Factory 首包：canonical 链 + production_write=FORBIDDEN
morn.research             # 科研/实验室
```

## Factory Readonly 要求（canonical）

```text
durable_work_state = required
source_of_truth_binding = required
provenance = required
authority_before_external_side_effect = required
receipt_after_external_action = required
reconciliation_on_unknown = required (if outcome-unknown)
accepted_outcome_semantics = required
profile/version pinning = required
production_write = forbidden
human gates: schedule_commitment, quality_decision
```

## 规则

1. 同一 Profile 允许：DSH or Pi / OPA or Cedar/reference / local store or future Temporal / SAP or fake connector——要求保证，不点名实现。
2. 已发布版本内容冻结；可以 deprecate/revoke；不能静默把 v1 内容改掉后仍称 v1。
3. requirement 映射 Condition gate（`src/control/controllers/state_loop.rs` gate_for_requirement）；Profile 要求 Condition；Provider/Controller 负责产生 Condition。

## Conformance

`ProfileConformanceController`（`src/control/controllers/state_loop.rs`）检查 required/forbidden guarantees against Work conditions；机器可读报告见 `tests/profile_conformance.rs`（G9）。
