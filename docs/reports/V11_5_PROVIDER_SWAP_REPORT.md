# Morn v11.5-R1 Provider Swap Report（Phase 22，Goal §23.3）

## Benchmark 设计

同一 CNC-17 scenario（`src/domains/factory/scenario.rs`），两个 harness provider：

```text
Run A = fake-dsh
Run B = fake-pi
```

业务不变条件（Goal §23.3 checklist）：

- 同一 Work contract（work-cnc17 / situation-cnc17）
- 相同 authority boundary（attempt 经 Action Gateway）
- 不绕过 receipt（OUTCOME_UNKNOWN → VERIFIED）
- 不重复 side effect（business key 幂等）
- Outcome/Acceptance 语义一致（3-role acceptance）
- Provider 失败不会让 Work 丢失（binding pin）

## 结果（真实自动化测试）

`tests/provider_swap_g11.rs`（3 测试全绿）：

| 检查 | Run A (fake-dsh) | Run B (fake-pi) |
|---|---|---|
| Work contract | work-cnc17 | work-cnc17（相同） |
| Binding pinned | true | true（相同） |
| External ref | MO-…（business key 决定） | 相同 |
| no_duplicate_action | true | true（相同） |
| Outcome/Acceptance | outcome-cnc17 + 3 acceptance | 相同 |
| ValueAssessment | value-cnc17 | 相同 |
| Audit timeline | 结构相同 | 相同 |

- Run A 确定性：重复运行产生相同 evidence bundle。
- Provider identity 差异是 swap 的目的，但从不泄漏进业务 truth 字段。

## Provider Conformance

所有 HarnessProvider 跑同一 contract test（`tests/providers_g4g5.rs`）：

- fake harness：完整合同通过（start/resume/submit/cancel/checkpoint/events/usage/health/close）。
- DSH/Pi skeletons：honest blocker 合同通过（`REAL_PROVIDER_INTEGRATION_BLOCKED`；无凭据/endpoint → ExternalBlocked；不伪造 session）。
- 四个 provider 是同一 trait object（harness-neutral 在代码层证明）。

## 局限（诚实标记）

**G11 = PASS_WITH_LIMITATIONS**：真实 DSH/Pi 双 provider 真跑 BLOCKED_EXTERNAL（无凭据/endpoint 访问）。解除条件：用户提供 DSH/Pi 运行环境与凭据；解除后跑真实 contract smoke + swap benchmark → 升级真实 PASS。

**不得声称：**"DSH/Pi 可无损替换"已真实 PASS——只有 contract 层与 fake swap 已证明（母版 §42.2 边界）。
