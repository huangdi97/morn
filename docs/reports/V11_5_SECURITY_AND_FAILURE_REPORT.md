# Morn v11.5-R1 Security and Failure Report（Phase 22，Goal §22）

**威胁模型：** `docs/security/MORN_V11_5_THREAT_MODEL.md`（18 项威胁 + 防御落点 + fault injection 矩阵）

## 威胁覆盖（18 项全部有防御代码 + 测试证据）

prompt/tool injection、malicious capability package、provider impersonation、credential leakage、action replay、duplicate write、stale authority、binding substitution、qualification spoofing、provenance tampering、external source conflict、unknown outcome、compromised harness、sandbox escape assumptions、UI approval confusion、cross-site capability misuse、event reordering/duplication、clock/time semantics。

## Fault Injection 结果（真实自动化测试）

| 故障注入 | 测试 | 结果 |
|---|---|---|
| kill process（Attempt 后真 kill） | `tests/control_plane_g2.rs` g2_process_kill… | Work/Binding/Attempt 存活；reconcile 继续；duplicate 被拒 |
| provider crash（unload mid-execution） | `tests/binding_pin_g6.rs` g6_unload_during_execution_pin_unchanged | Binding 不被静默迁移；显式 suspend；re-mount 不恢复 |
| provider restart | `tests/binding_pin_g6.rs` g6_provider_restart_no_duplicate_side_effect | 无重复副作用；幂等生效 |
| second provider load | `tests/binding_pin_g6.rs` g6_load_second_provider_creates_new_binding | 新 Binding/新 Attempt；旧 Attempt 不 auto-resume |
| crash-after-commit + timeout | `tests/receipt_reconciliation_g7.rs` g7_crash_after_commit_then_reconcile | 持久 fake CMMS 订单存活；reconcile→VERIFIED；no duplicate |
| retry storm | `tests/receipt_reconciliation_g7.rs` g7_retry_storm_is_idempotent | 10 duplicate 调用 + 5 duplicate dispatch → 1 order 1 attempt |
| duplicate receipt | `tests/receipt_reconciliation_g7.rs` g7_duplicate_receipt… | 幂等；VERIFIED 不被降级 |
| reordered event | `tests/failure_engineering.rs` g_reordered_event_projection_is_deterministic | 乱序 replay deterministic；retract 取消效果 |
| duplicate event | `tests/failure_engineering.rs` g_duplicate_event_is_a_no_op | duplicate event id 被忽略 |
| slow provider | `tests/failure_engineering.rs` g_slow_provider_surfaces_typed_and_never_fakes | typed 错误；不伪造成功；订单真实存在 |
| malformed receipt | ReceiptController transport-ack ≠ business-commit | transport ack 只记录不验证 |
| expired authority | `tests/trust_governance.rs`（windowed/expired） | 窗口外拒绝 |
| revoked capability | `tests/capability_supply_chain.rs`（Revoked 终态） | 终态分离 |
| profile upgrade during work | `tests/failure_engineering.rs` g_profile_upgrade_during_work_pins_work_state | Work 生成/状态不被静默改写；released 版本冻结 |
| DB restart | SQLite 文件级（G2 fresh store 重开同 DB）；容器化 SKIP_EXTERNAL | durable state 已证明；容器化待外部条件 |

## 安全不变量（全部贯穿）

```text
REAL_PRODUCTION_WRITE = NO
CONTROLLED_REAL_WRITE = NO
```

- Agent 不直接持有生产凭据（CredentialProvider 注入；PEP 拒绝无授权写）
- Side-effect action 必须穿过 Action Gateway/PEP（`tests/trust_governance.rs`）
- Package digest 真实 SHA-256（ring known-answer 测试）；tampering 被检测；provenance 诚实 LOCAL UNSIGNED
- Secrets 不入 git；logs redact（`tests/telemetry.rs` redact_for_log）
