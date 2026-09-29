# Morn v11.5-R1 Factory Reference Scenario Report（Phase 22，G10）

## 场景

CNC-17（canonical §27.1 fixture）：

```text
asset: CNC-17
current_order: WO-84729
batch: LOT-043
vibration: 8.7 mm/s
site: plant-a
```

模式：READ / MAP / ANALYZE / CANDIDATE / EVIDENCE / EXPERT REVIEW；**NO PRODUCTION WRITE**（fake CMMS sandbox only）。

## 运行证据（真实自动化测试：`tests/factory_thin_slice.rs`，4 测试全绿）

完整链路（canonical §27）：

```text
SCADA/Alarm event → Situation → Work creation → Binding
→ Equipment Investigator（harness）→ vibration evidence
→ scheduling solver → 4 candidate plans → human decision（candidate B）
→ Fake CMMS action（sandbox）→ intentional timeout after commit
→ OUTCOME_UNKNOWN → reconciliation → Receipt verified
→ observed states → ObservedOutcome → 3-role Acceptance → ValueAssessment
```

## Evidence Bundle 关键项

| 项 | 证据 |
|---|---|
| DSH/Pi | fake-dsh harness（真实 DSH/Pi BLOCKED_EXTERNAL——skeleton + honest blocker 合同已证） |
| binding | binding-cnc17，binding_pinned = true（仍 BOUND/同 provider/同 generation） |
| unknown | OUTCOME_UNKNOWN → RECONCILING → VERIFIED 真实发生（timeout-after-commit 模式） |
| reconciliation | 按 business key 回查（source-of-truth readback，不用 Agent 自述） |
| receipt | receipt-cnc17（cmms-readback source；transport ack ≠ business commit） |
| outcome | outcome-cnc17（ExternalSystem source；downtime 52 min / order_A_delay 11 min / vibration 3.1 mm/s） |
| acceptance | 3-role（production/quality/maintenance）ACCEPT |
| no duplicate action | true（business key 幂等；re-submit 返回同一 MO-ref） |
| audit timeline | ≥12 条有序 controller/condition 变化（SCADA→…→ValueAssessment） |

## 确定性与可重放

- 一键可跑：`FactoryScenarioRunner::run(&ScenarioFixture::cnc17(), orders_path)`。
- 两次运行产生相同 evidence bundle（同 work id、同 business key、同 external ref、同 acceptance set、同 timeline）——`g10_scenario_is_deterministic` 通过。
- Evidence bundle round-trip：serde 序列化/反序列化一致（机器可读证据）。

## Provider Swap

Run A = fake-dsh / Run B = fake-pi：业务不变条件一致（见 `V11_5_PROVIDER_SWAP_REPORT.md`）。
