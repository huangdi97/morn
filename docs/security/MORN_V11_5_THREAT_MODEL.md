# Morn v11.5-R1 Security / Threat Model / Failure Engineering

**状态：** 与仓库真实代码、测试一致的威胁模型（Phase 18，Goal §22）
**代码事实源：** `src/work` / `src/control` / `src/providers` / `src/runtime/cordis` / `src/telemetry`
**测试证据：** `tests/`（G2/G6/G7/G8 fault-injection 路径）+ 本轮补充（reordered event / slow provider / profile upgrade during work）

---

## 1. 威胁清单与防御（18 项）

| # | 威胁 | 防御（代码位置） | 测试证据 |
|---|---|---|---|
| 1 | Prompt/tool injection | Harness 事件仅为 Harness evidence，不直接成为 Work truth；Agent 输出经结构化 Candidate/EvidenceRefs 接收（`src/providers/harness.rs` BOUNDARY，`src/domains/factory/scenario.rs` 步骤 5） | `tests/providers_g4g5.rs`（session events 不入 Work）；`tests/outcome_acceptance_value.rs`（agent self-report 不是 OutcomeSource） |
| 2 | Malicious capability package | Capability 四态分离（Declared≠Qualified≠Admitted）；strict-profile gate：无 PASSED qualification 不进关键 binding（`src/work/capability.rs`，`src/control/controllers/resolve_loop.rs`） | `tests/capability_supply_chain.rs` G8 |
| 3 | Provider impersonation | Provider slot 有 contract names（morn.*）；provider 身份记录在 Binding（provider_id/version）；hot-swap 不静默改 Binding（`src/runtime/cordis/slots.rs`，`src/control/controllers/execution_flow.rs`） | `tests/runtime_cordis_g3.rs`；`tests/binding_pin_g6.rs` G6 |
| 4 | Credential leakage | CredentialProvider 接口（scope→value 不越界）；Harness 不天然持有生产凭据；fingerprint redaction；EnvCredentialProvider 由 app shell 注入（`src/providers/credential.rs`） | `tests/providers_g4g5.rs` g5_credential_provider_boundary；`tests/telemetry.rs`（redact_for_log） |
| 5 | Action replay | 幂等 key 唯一约束（SQLite INSERT OR IGNORE）；duplicate attempt 被拒（`src/control/store_execution.rs`，ExecutionController.dispatch） | `tests/control_plane_g2.rs`（duplicate idempotency key 被拒）；`tests/receipt_reconciliation_g7.rs` retry storm |
| 6 | Duplicate write | Connector 按 business key 幂等（同 key 返回既有订单）；G7：crash+timeout+retry storm 不产生重复工单（`src/providers/connector.rs`，`src/providers/connector_persistent.rs`） | `tests/receipt_reconciliation_g7.rs`（crash-after-commit→reconcile→no duplicate）；`tests/factory_thin_slice.rs`（no_duplicate_action） |
| 7 | Stale authority | AuthorityDecision.authorizes(now) 在 use 时重查 time window；gateway deny-by-default（`src/work/authority.rs`，`src/providers/gateway.rs`） | `tests/trust_governance.rs`（windowed 决策在窗口外拒绝；PEP denies expired/revoked） |
| 8 | Binding substitution | ExecutionBinding 一等资源；Attempt pin Binding；provider reload 只影响 future resolution（`src/work/execution.rs`，`tests/binding_pin_g6.rs`） | `tests/binding_pin_g6.rs`（unload/restart/second-provider 三路注入） |
| 9 | Qualification spoofing | Qualification 由 evaluator identity 记录；被评对象不能自己改；EXPIRED 不 qualify 新 binding（`src/control/controllers/resolve_loop.rs`） | `tests/capability_supply_chain.rs`（expired 阻断）；`tests/artifact2capability.rs`（compiler 不是 evaluator） |
| 10 | Provenance tampering | Package digest 是真实 SHA-256（ring）；verify() 重算 digest；tampering 被检测；provenance 诚实 LOCAL UNSIGNED（`src/studio/package.rs`） | `tests/capability_package.rs`（known-answer digest + tampering detected + signature:null） |
| 11 | External source conflict | Observation 支持 supersedes/invalidated + 双时间（valid_time/recorded_at）；current projection 用新事实但历史可解释（`src/work/world.rs`，`src/work/common.rs`） | `src/work/tests.rs`（supersede/retract）；`tests/control_plane_g2.rs`（replay deterministic） |
| 12 | Unknown outcome | OUTCOME_UNKNOWN 一等状态 → RECONCILING → VERIFIED/FAILED/STILL_UNKNOWN；timeout 不自动当 failure（`src/work/execution.rs` AttemptStatus，ReconciliationController） | `tests/control_plane_g2.rs`（kill 后 reconcile 继续）；`tests/receipt_reconciliation_g7.rs`（timeout→VERIFIED） |
| 13 | Compromised Harness | Harness session 事件不成为 Work truth；Provider 故障 → Binding SUSPENDED → 新 Binding/新 Attempt 才能继续（`src/providers/harness.rs` BOUNDARY，BindingController.suspend） | `tests/binding_pin_g6.rs`；`tests/providers_g4g5.rs`（honest blocker） |
| 14 | Sandbox escape assumptions | Sandbox fake 永远非真实 endpoint；ExecutionEnvironmentSpec admission 检查 isolation 要求 vs provider guarantees；future providers 诚实 FutureProvider（`src/providers/connector.rs` SAFETY，`src/providers/execution_env.rs`） | `tests/execution_environment.rs`（future providers admit nothing；local 不 admit container） |
| 15 | UI approval confusion | 审批决策由 identifiable principal 记录；REJECT/CONDITIONAL 记录为业务事实但不满足 acceptance gate（`src/control/controllers/outcome_flow.rs` SAFETY） | `tests/outcome_acceptance_value.rs`（reject 不 satisfy gate） |
| 16 | Cross-site capability misuse | SiteAdmission 独立记录（site/profile/scope/admitted_by/revoked）；resolver 硬过滤 site_admitted（`src/work/capability.rs`，`src/control/workcell.rs`） | `tests/workcell_resolver.rs`（site not admitted 阻断）；`tests/capability_supply_chain.rs` |
| 17 | Event reordering/duplication | Event append idempotent by id；replay deterministic（retract 取消效果）；outbox exactly-once（`src/control/store.rs`，`src/control/projection.rs`，`src/control/store_runtime.rs`） | `tests/control_plane_g2.rs`（replay + outbox）；本轮补充 `tests/failure_engineering.rs`（reordered event） |
| 18 | Clock/time semantics | 双时间（valid_time/recorded_time）+ AuthorityRequest.is_within_time_window + AuthorityDecision.expires_at（`src/work/authority.rs`） | `tests/trust_governance.rs`（windowed 决策窗口内外行为） |

---

## 2. 最小安全原则（master §31.1）在代码中的落点

1. **Agent 不直接持有高权限长期凭据** — `src/providers/credential.rs`（凭据经 CredentialProvider 注入）；`src/providers/gateway.rs`（Agent 无凭据路径被 PEP 拒绝）。
2. **Side-effect Action 必须穿过 PEP** — `src/providers/gateway.rs` ActionGateway.evaluate/authorize_dispatch；`tests/trust_governance.rs`（无 decision/revoked/expired/unbound 全部 deny）。
3. **Capability package 可验证来源和 digest** — `src/studio/package.rs`（SHA-256 + verify + provenance contract）。
4. **Attempt 使用幂等 key** — `src/work/execution.rs` ActionAttempt.idempotency_key（SQLite 唯一约束）。
5. **Unknown outcome 必须 reconcile** — `src/control/controllers/outcome_flow.rs` ReconciliationController（source-of-truth readback，不用 Agent 自述）。
6. **Provider replacement 不得静默改变 active Binding** — `tests/binding_pin_g6.rs`（三路故障注入）。
7. **Qualification/Acceptance/Authority 不能由被评对象自己修改** — `src/control/controllers/resolve_loop.rs`（evaluator identity）；`tests/outcome_acceptance_value.rs`（reject 不 satisfy）。
8. **高风险 Profile 必须要求 human gate** — `src/work/profile.rs` factory_readonly.human_gates（schedule_commitment/quality_decision）。
9. **Harness sandbox 不自动视为 industrial safety boundary** — `src/providers/execution_env.rs`（ExecutionEnvironmentSpec admission 独立检查）。
10. **每个高权限 Action 可追到 Work/Binding/AuthorityDecision/Receipt** — `src/telemetry/mod.rs`（11 个 morn.* semantic attrs 覆盖全链）+ `src/domains/factory/scenario.rs`（audit timeline）。

---

## 3. Fault Injection 矩阵（Goal §22）

| 故障注入 | 已有/本轮 | 测试 | 结果 |
|---|---|---|---|
| kill process（Attempt 后真 kill） | 已有（G2） | `tests/control_plane_g2.rs` g2_process_kill_after_attempt_preserves_work_and_prevents_duplicate | Work/Binding/Attempt 存活；reconcile 继续；duplicate 被拒 |
| provider crash（Attempt 期间 unload） | 已有（G6） | `tests/binding_pin_g6.rs` g6_unload_during_execution_pin_unchanged | Binding 不被静默迁移；显式 suspend 记录；re-mount 不恢复旧 Binding |
| provider restart（重跑防护） | 已有（G6） | `tests/binding_pin_g6.rs` g6_provider_restart_no_duplicate_side_effect | 重启不重复副作用；idempotency 生效 |
| second provider load | 已有（G6） | `tests/binding_pin_g6.rs` g6_load_second_provider_creates_new_binding | 新 Binding/新 Attempt；旧 Attempt 不被 auto-resume |
| crash-after-commit + timeout | 已有（G7） | `tests/receipt_reconciliation_g7.rs` g7_crash_after_commit_then_reconcile | 持久 fake CMMS 订单存活；reconcile→VERIFIED；no duplicate |
| retry storm | 已有（G7） | `tests/receipt_reconciliation_g7.rs` g7_retry_storm_is_idempotent | 10 次 duplicate 调用 + 5 次 duplicate dispatch → 1 order 1 attempt |
| duplicate receipt | 已有（G7） | `tests/receipt_reconciliation_g7.rs` g7_duplicate_receipt_is_idempotent_and_never_downgrades | duplicate 幂等；VERIFIED 不被降级 |
| malformed receipt（transport ack） | 已有（G7） | ReceiptController.record transport-ack≠business-commit 分支 | transport ack 只记录不验证 |
| expired authority | 已有（G8/Phase 8） | `tests/trust_governance.rs`（windowed + expired）；`tests/capability_supply_chain.rs`（expired qualification） | 窗口外拒绝；expired 不 qualify |
| revoked capability | 已有（Phase 10） | `tests/capability_supply_chain.rs`（CapabilityState::Revoked 终态） | revoked 终态分离 |
| reordered event | **本轮补充** | `tests/failure_engineering.rs` g_reordered_event_projection_is_deterministic | 乱序事件 replay 仍 deterministic；retract 取消效果 |
| duplicate event | 已有（G2） | `tests/control_plane_g2.rs`（event id 幂等）+ `tests/failure_engineering.rs`（duplicate event no-op） | duplicate event id 被忽略 |
| slow provider | **本轮补充** | `tests/failure_engineering.rs` g_slow_provider_surfaces_typed_and_never_fakes | 慢 provider（超时模式）→ typed 错误；不伪造成功 |
| profile upgrade during work | **本轮补充** | `tests/failure_engineering.rs` g_profile_upgrade_during_work_pins_work_state | Work 生成/状态不被 profile 升级静默改写；released 版本冻结 |

### DB restart

G2 的 "restart" 用 fresh store 重开同一 SQLite DB（真 kill 后）。容器化 DB restart 超出本地环境（无 Docker 集群），SQLite 文件级重启已证明 durable state 语义；标记 `SKIP_EXTERNAL`（本地无 DB 集群部署）。

---

## 4. 结论

- 18 项威胁全部有防御代码落点与测试证据；无"设计即防御"误报。
- 故障注入路径（kill/crash/timeout/storm/duplicate/reorder/slow/upgrade）有自动测试。
- REAL_PRODUCTION_WRITE = NO / CONTROLLED_REAL_WRITE = NO 贯穿始终。
- 剩余项：真实客户授权数据（G12）为 BLOCKED_EXTERNAL；OPA/Cedar/Sigstore/SPIFFE 为 contract 已保留的 provider 替换点（reference 实现已证明 contract 可用）。
