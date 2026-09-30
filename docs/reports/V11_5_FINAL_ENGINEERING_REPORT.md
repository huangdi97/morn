# Morn v11.5-R1 Final Engineering Report（Phase 22，Goal §32）

**HEAD：** `3922f6957356758447a9eda3cd60f201ef4db0f9`（main，领先 origin/main 19 个本地 commit，未推送）
**收口状态：** `ENGINEERING_COMPLETE_WITH_EXTERNAL_BLOCKERS`

## A. Repository State

```text
branch:            main
HEAD:              3922f6957356758447a9eda3cd60f201ef4db0f9
origin:            https://github.com/huangdi97/morn.git（领先 19 个本地 commit，未推送——无明确 push 权限确认，见 push plan）
worktree status:   非追踪文件：.pi/（goal 工件）、Morn_*.md 根目录副本、src-tauri/gen/（Tauri 生成 schema）——保留，不删除
baseline HEAD:     2cd9fbd6fa57930eaa71616f170f958037190654（Phase 0 起点）
```

**Push plan：** 本地 commit 已按 Phase 分组（feature/fix/docs 分类清晰）；推远端前确认 branch policy（是否允许直接 push main 或需要 PR）；建议 `git push origin main`（无 force push）。

## B. Architecture Completion（六 Plane）

| Plane | 状态 | 证据文件/测试路径 |
|---|---|---|
| 1. Semantic / Specification | **IMPLEMENTED** | `schemas/`（8 份 JSON Schema）+ `src/work/`（12 文件）；`tests/work/tests.rs`、`src/work/validation.rs`（G1） |
| 2. Durable Control Plane | **IMPLEMENTED** | `src/control/`（SQLite 14 表 + 12 controllers + outbox/lease/projection）；`tests/control_plane_g2.rs`（G2 真 kill/restart） |
| 3. Composition Plane (Cordis) | **IMPLEMENTED** | `cordis-rs =0.8.39` exact pin + `src/runtime/cordis/`（facade/slots/host）；`tests/runtime_cordis_g3.rs`（G3 binding pin） |
| 4. Execution Plane | **IMPLEMENTED**（real harness integration BLOCKED_EXTERNAL） | `src/providers/`（harness/dsh/pi/fake/solver/human/connector/connector_persistent/credential/execution_env/policy/gateway）；`tests/providers_g4g5.rs`（G4/G5）+ `tests/binding_pin_g6.rs`（G6） |
| 5. Trust & Governance | **IMPLEMENTED** | `src/providers/policy.rs`（deny-by-default）+ `src/providers/gateway.rs`（PEP）+ `src/work/authority.rs`；`tests/trust_governance.rs` |
| 6. Capability Supply Chain | **IMPLEMENTED** | `src/work/capability.rs` + `src/studio/artifact2capability.rs` + `src/studio/package.rs`；`tests/capability_supply_chain.rs`（G8）+ `tests/artifact2capability.rs` + `tests/capability_package.rs` |

Domain Products：`src/domains/factory/scenario.rs`（CNC-17 reference scenario，G10）。
Telemetry：`src/telemetry/mod.rs`（morn.* 11 semantic attrs + CloudEvents + redaction）。

## C. Gate Matrix（G0–G12）

见 `V11_5_GATE_MATRIX.md`：

```text
PASS                    = G0 G1 G2 G3 G6 G7 G8 G9 G10
PASS_WITH_LIMITATIONS   = G4 G5 G11（skeleton/contract 真实通过；真实 provider 接入被外部条件阻塞）
BLOCKED_EXTERNAL        = G12（无真实客户授权数据；非工程失败）
FAIL                    = 无
```

## D. Test Matrix（真实数量）

```text
Rust unit（lib）          = 1301 passed, 0 failed
Rust integration          = 34 passed, 0 failed
Rust conformance/G-gates  = 99 passed, 0 failed
  （G2=7, G3=8, G4/G5=9, G6=4, G7=5, G8=6, G9=6, G10=4, G11=3,
    trust_governance=5, execution_environment=7, capability_package=5,
    artifact2capability=7, workcell=9, telemetry=4, failure_engineering=4,
    work_module_isolation=1, outcome_acceptance=5 — 合计 94+integration 内）
Rust 总计                 = 1434 passed, 0 failed（连续两次运行一致）
cargo fmt --check         = PASS
cargo clippy --lib -D warnings = PASS
web vitest                = PASS（Phase 0 时点 6 tests；当前 tsc PASS）
web build（tsc + vite）   = PASS
playwright e2e            = 10 passed / 0 failed（真实 msedge 浏览器跑通；三层 overlay 根因已修复，commit 0872019）
verify-all                = RESULT: ALL_LOCAL_GATES_PASS（-WithE2E 终跑 exit 0：fmt + clippy + Rust 1434/0 + web + e2e 10/0 真跑）
failure-injection         = 15 路径有自动测试（见 V11_5_SECURITY_AND_FAILURE_REPORT.md）
```

## E. Factory Scenario（CNC-17）

见 `V11_5_FACTORY_REFERENCE_SCENARIO_REPORT.md`。关键项：

```text
DSH/Pi           = fake-dsh harness（真实 BLOCKED_EXTERNAL，skeleton 合同已证）
binding          = binding-cnc17, pinned（BOUND/同 provider/同 generation）
unknown          = OUTCOME_UNKNOWN → RECONCILING → VERIFIED 真实发生
reconciliation   = business key 回查（source-of-truth readback）
receipt          = receipt-cnc17（cmms-readback；transport ack ≠ business commit）
outcome          = outcome-cnc17（ExternalSystem；downtime 52min/order_A_delay 11min/vibration 3.1mm/s）
acceptance       = 3-role ACCEPT（production/quality/maintenance）
no duplicate     = true（business key 幂等）
deterministic    = 两次运行 evidence bundle 完全一致
```

## F. External Blockers（逐项解除条件）

见 `V11_5_KNOWN_GAPS_AND_EXTERNAL_BLOCKERS.md`：

1. DSH 真实接入（G4）— 需 DeepSeek Harness 进程/接口 + API key（用户提供）
2. Pi 真实接入（G5/G11）— 需 Pi 访问 + 凭据（用户提供）
3. 真实客户数据/现场（G12）— 需客户授权 + 现场安全/回滚/审批条件（客户 + 人类授权）
4. Sigstore/SLSA 签名 — 需真实 signer（LOCAL_UNSIGNED 诚实标记已就位）
5. 容器化 DB restart — 需 Docker/DB 集群部署
6. OPA/Cedar 真实 policy provider — 需部署（ReferencePolicyProvider 已证 contract）

## G. Remaining Debt（真实债务）

| 项 | 优先级 | 原因 | 阻塞发布 |
|---|---|---|---|

| v11.5 Workbench/Console/Studio 完整 UI | P3 | 现栈 chat-first UI 未做 v11.5 Workbench 完整重写 | 否 |
| npm audit 5（从 11 修复）/ node 漂移已修复（c664155）/ eslint 无配置 / update.json 空 signature | P4 | 剩余需 breaking changes 或人工决策；不伪造 | 否 |

## H. Next Product Step

本 Goal 工程收口后，下一阶段真实客户计划（需人类授权）：

1. **read-only shadow**：客户授权历史数据接入 → 同一 CNC-17 scenario 用客户真实 MES/CMMS 数据回放 → 真实 ObservedOutcome 取证（G12 解除路径）。
2. **DSH/Pi 真实接入**：用户提供凭据 → G4/G5 升级真实 PASS → G11 swap benchmark 真跑。
3. **第二站点**：验证 capability reuse across sites（母版 §39.3 商业假设）。

## DONE 定义核对（Goal §29）

1. ✅ v11.5 Reference Implementation 核心语义/Control Plane/Cordis composition/Provider contracts/DSH/Pi skeletons/Capability Supply Chain 基础/Profiles/Factory thin slice 已落地；real provider 与 future provider 明确、真实地标记 external/future。
2. ✅ 旧 Core/Factory 能力显式映射与迁移（`V11_5_MIGRATION_REPORT.md`），没有无证据推倒。
3. ✅ 所有本地可执行 Gate 通过（verify-all ALL_LOCAL_GATES_PASS）。
4. ✅ reference CNC scenario 一键重复运行（deterministic 两次一致）。
5. ✅ crash/restart/unknown/reconcile/provider-failure 路径有自动测试（15 路径）。
6. ✅ 至少一个非 Agent executor 路径证明 Agent is one executor（0-Agent Workcell + solver/human provider）。
7. ⚠️ 两 Harness 真实可用时完成 swap——DSH/Pi 真实凭据外部不可得，诚实阻塞（PASS_WITH_LIMITATIONS）。
8. ⚠️ UI 可完成至少一个 Work 全链路——控制平面语义/API 已完整；e2e 全链路 10/0 通过（welcome→发消息→Studio→Console→设置→持久化）；v11.5 Workbench 专用视图（Work list/conditions/candidates）仍为现栈 chat-first UI 的扩展项（P3，非阻塞）。
9. ✅ 文档、ADR、报告和代码一致。
10. ✅ 没有虚假"客户价值/生产写/商业成熟"声明（REAL_PRODUCTION_WRITE=NO 贯穿；G12 BLOCKED_EXTERNAL）。
