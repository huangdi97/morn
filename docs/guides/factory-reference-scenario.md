# Factory Reference Scenario Guide（CNC-17）

## 运行

```rust
use morn::domains::factory::scenario::{FactoryScenarioRunner, ScenarioFixture};

let fixture = ScenarioFixture::cnc17();  // CNC-17 / WO-84729 / LOT-043 / vibration 8.7
let evidence = FactoryScenarioRunner::run(&fixture, orders_path)
    .expect("the reference scenario runs end-to-end");
```

一键可跑、deterministic：两次运行产生相同 evidence bundle（同 work id、同 business key、同 external ref、同 acceptance set、同 timeline）。

## 链路（canonical §27）

```text
SCADA/Alarm event → Situation → Work creation → Binding
→ Equipment Investigator（harness）→ vibration evidence
→ scheduling solver → candidate plans → human decision
→ Fake CMMS action（sandbox only）→ intentional timeout after commit
→ OUTCOME_UNKNOWN → reconciliation → Receipt verified
→ fake MES/QMS/APS observed states → ObservedOutcome
→ multi-role Acceptance → ValueAssessment
```

## Evidence Bundle

`ScenarioEvidence`（`src/domains/factory/scenario.rs`）携带：

- binding_pinned（Binding 仍 BOUND/同 provider/同 generation）
- outcome_unknown_then_verified（OUTCOME_UNKNOWN → RECONCILING → VERIFIED 真实发生）
- external_ref + no_duplicate_action（幂等 business key；无重复工单）
- outcome_id / acceptance_ids（3-role）/ value_assessment_id
- audit_timeline（有序 controller/condition 变化，≥12 条）

## 模式

READ / MAP / ANALYZE / CANDIDATE / EVIDENCE / EXPERT REVIEW；NO PRODUCTION WRITE（sandbox fake only）。

## Provider Swap

`FactoryScenarioRunner::run_with_harness(fixture, path, "fake-pi")`——同一 scenario 换 harness；业务不变条件保持（G11 benchmark）。真实 DSH/Pi BLOCKED_EXTERNAL 时标 PASS_WITH_LIMITATIONS。

测试证据：`tests/factory_thin_slice.rs`（G10）/ `tests/provider_swap_g11.rs`（G11）。
