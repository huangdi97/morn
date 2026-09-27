# Morn v11.5-R1 Execution State

> 本文件由 Agent 自主运行机制维护（Goal §30）。下一 Agent 必须从本文件 + `docs/canonical/CURRENT.md` + canonical 母版开始，不从聊天记忆猜测。

**最后更新：** Phase 2 完成后（2026-09-25）


## Current HEAD

`main` @（Phase 2 commit，见 Git log 最新一条）；Phase 0 基线 HEAD 为 `2cd9fbd6fa57930eaa71616f170f958037190654`。

## Completed phases

- **Phase 0 — Reality Audit**（G0 = PASS）
  - `docs/reports/V11_5_REALITY_AUDIT.md`（真实仓库事实 + 成熟度分级）
  - `docs/reports/V11_5_BASELINE_TEST_REPORT.md`（先跑后记录，含初始失败与根因修复）
  - `docs/reports/V11_5_CODE_MAPPING_AND_MIGRATION_MATRIX.md`（对象级矩阵）
  - canonical 落位：`docs/canonical/`（母版 + GOAL + HANDOFF + README + `CURRENT.md`），旧文档未删除
- **Phase 1 — Semantic Specification**（G1 = PASS）
  - `schemas/` 8 份 JSON Schema（common/world/work/authority/capability/execution/outcome/profile，self-contained + $ref）
  - `src/work/` 语义模块（12 文件）：Work.spec/status、9 个 Condition 语义 Gate、AuthorityRequest/Decision（deny-by-default）、Capability 四态、ExecutionBinding/ActionAttempt（全状态机）/Receipt/Reconciliation、ObservedOutcome/Acceptance/Value、Profile（production_write=FORBIDDEN）、HistoricalRecord（supersede/retract/双时间）
  - `src/work/validation.rs`：真实 schema 验证器（$ref + definition 入口）；17 个 schema 测试 + 1 个模块隔离测试
  - G1 证据：schema round-trip ✓、TaskRecord→Work carrier 映射（不建第二账本）✓、protocol types 无 Cordis/DSH/Pi 依赖 ✓
- **Phase 2 — Durable Control Plane Minimum**（G2 = PASS）
  - `src/control/`：ControlPlaneStore（SQLite，14 张 cp_* 表：work/condition/semantic_event/binding/attempt/receipt/reconciliation/outcome/acceptance/capability_release/qualification/profile_release/outbox/controller_lease，DDL 独立 ddl.rs）
  - event history（append idempotent + supersede/retract 显式字段 + contract/schema version 元数据）+ deterministic projection（replay 规则：retract 取消效果、last-write-wins phase）
  - 12 个 reference controllers（WorkController 阶段推进幂等、BindingController、AuthorityController time-window 检查、ExecutionController 幂等 dispatch + Binding 前置检查、ReceiptController transport-ack≠business-commit 区分、ReconciliationController business-key 回读、OutcomeController、AcceptanceController、QualificationController、ProfileConformanceController、SituationController、CapabilityResolverController）
  - outbox（claim/complete/requeue，exactly-once）+ controller lease/checkpoint（exclusive lease）
  - G2 证据（tests/control_plane_g2.rs，7 测试全绿）：**真进程 kill**（子进程写 Work/Binding/Attempt 后被真 kill）→ 重启（fresh store）→ Work 存活 ✓、Attempt/Binding 不丢 ✓、reconcile 继续 → VERIFIED ✓、duplicate idempotency key 被拒 ✓；deterministic replay ✓；outbox exactly-once ✓；lease 互斥 ✓；from_value 回归测试 ✓

## Current gate status

```text
G0  Reality Audit          PASS
G1  Semantic Mapping       PASS
G2  Durable State          PASS
G3  Cordis Isolation       前置证据已就绪（isolation test）；G3 在 Phase 3 收口
G4-G11                     未开始
G12 Customer Gate          BLOCKED_EXTERNAL（无真实客户授权数据）
```

## Tests last run

- Rust: `cargo test --workspace` = **1342 passed, 0 failed**（1300 lib + 7 G2 + 34 integration + 1 isolation）；`cargo fmt --check` PASS；`cargo clippy --lib -- -D warnings` PASS（CI 标准）
- Web: vitest 6 passed；tsc PASS；build PASS（Phase 0 时点）
- e2e（chromium 1228）: 2 passed / 8 failed（③–⑩ UI 结构漂移 → Phase 16 重写）
- `npm audit`: 11 vulnerabilities（记录，Phase 21 评估）

## Architecture decisions

1. v11.5 canonical 落位 `docs/canonical/`，`CURRENT.md` 为指针；12 个 `docs/*.md` 标记 LEGACY（仅追溯）。
2. 目录等价映射（不强制大迁移）：语义层 = `src/work/`（schemas = `schemas/`）；控制平面扩展 `src/core/storage/` + 新 controllers；Cordis adapter 计划 `src/runtime/cordis/`；providers 计划 `src/providers/`；Factory 计划 `src/domains/factory/`。
3. SQLite 保持为第一版 durable store（母版 §32 允许；不引入 Kafka/K8s/Temporal）。
4. Windows 本地构建需会话级 `INCLUDE`/`LIB`（MSVC 在 `D:\Code\Visual Studio\Visual Studio\VC\Tools\MSVC\14.50.35717` + SDK 10.0.26100.0）；E: 盘空间紧张（target/ 15GB），构建前清理 `target/debug/incremental`；verify 脚本需处理。
5. 旧资产映射：TaskRecord→Work 载体（HistoricalRecord 引用，不复制）、approval→Authority 前身、capability registry→Capability 声明层、skill→Manifest 条目、checkpoint→controller checkpoint、cost→ValueAssessment、hash-chain→receipt 完整性。TF/RV/LC/BIOFERM 保持 LEGACY 不迁移。
6. AgentLoop 硬编码输出记录为 Reality Gap（agent loop 属 Harness 职责，Phase 4 处理 Harness 时说明）。
7. 语义模块内部统一 `crate::work::<Type>` re-export 引用风格；`resource.rs` 承载 Work 资源类型（避免 `work::work` 命名冲突）。

## Files changed（Phase 2）

- 新增：`src/control/`（mod/store/ddl/store_execution/store_records/store_runtime/projection/controllers/{mod,state_loop,execution_flow,resolve_loop,outcome_flow}）、`tests/control_plane_g2.rs`
- 修改：`src/lib.rs`（+pub mod control）

## Open root causes

- e2e ③–⑩ 失败：e2e 预期与 v0.2.0 UI 结构漂移（Studio 8 tab、Store/Console 结构、消息流 DOM）→ Phase 16
- npm audit 11 vulnerabilities → Phase 21
- node 版本 ci.yml=20 vs release.yml=22 → Phase 21
- web lint 脚本无 eslint 配置 → Phase 21
- update.json 空 signature → Phase 22（不伪造）
- E: 盘空间紧张 → 构建前清理 incremental（已临时处理）

## External blockers

| 项 | 缺什么 | 谁才能解除 | 解除后的验收步骤 |
|---|---|---|---|
| DSH Harness Provider 真实接入 | DeepSeek Harness 进程/接口访问 + API key | 用户提供 DSH 运行环境与凭据 | Phase 4：DSH contract smoke（start/resume/cancel/events）真跑 |
| Pi Harness Provider 真实接入 | Pi 访问 + 凭据 | 用户提供 | Phase 5：Pi contract + swap benchmark 真跑 |
| 真实客户数据 / 现场（G12） | 客户授权数据、现场安全/回滚/审批条件 | 客户 + 人类授权 | 母版 0.2 边界重新验证；read-only/shadow 计划 |
| Cordis upstream 可用性 | 需确认 upstream Cordis exact version/commit（公开仓库核验） | Phase 3 Spike 时核验（网络可达则自动） | exact pin + 许可证记录 |

## Next exact actions

1. Phase 2：durable control plane minimum——SQLite 扩展表（work/work_condition/semantic_event/execution_binding/action_attempt/receipt/reconciliation/observed_outcome/acceptance/outbox/controller_lease）+ append/supersede event history + deterministic projection + replay/crash 恢复测试 + event idempotency + outbox。
1. Phase 3：Cordis Spike——确认 upstream Cordis exact version/commit（公开仓库核验）、exact pin、许可证记录、`src/runtime/cordis/` adapter（service slot/provider swap/scope/lifecycle/config reload/adapter boundary）。
2. Phase 3 G3：代码扫描/依赖图证明 domain/spec/control-plane 无大面积 Cordis coupling；active Attempt 不因 Cordis provider reload 被偷偷换 Binding。
3. 测试 → 取证 → 更新本文件 → commit → Phase 4。
