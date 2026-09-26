# Morn v11.5-R1 Execution State

> 本文件由 Agent 自主运行机制维护（Goal §30）。下一 Agent 必须从本文件 + `docs/canonical/CURRENT.md` + canonical 母版开始，不从聊天记忆猜测。

**最后更新：** Phase 1 完成后（2026-09-25）

## Current HEAD

`main` @（Phase 1 commit，见 Git log 最新一条）；Phase 0 基线 HEAD 为 `2cd9fbd6fa57930eaa71616f170f958037190654`。

## Completed phases

- **Phase 0 — Reality Audit**（G0 = PASS）
  - `docs/reports/V11_5_REALITY_AUDIT.md`（真实仓库事实 + 成熟度分级）
  - `docs/reports/V11_5_BASELINE_TEST_REPORT.md`（先跑后记录，含初始失败与根因修复）
  - `docs/reports/V11_5_CODE_MAPPING_AND_MIGRATION_MATRIX.md`（对象级矩阵）
  - canonical 落位：`docs/canonical/`（母版 + GOAL + HANDOFF + README + `CURRENT.md`），旧文档未删除
- **Phase 1 — Semantic Specification**（G1 = PASS）
  - `schemas/` 8 份 JSON Schema（common/world/work/authority/capability/execution/outcome/profile，self-contained + $ref）
  - `src/work/` 语义模块（12 文件）：ObjectRef/SourceRef/Provenance/HistoricalRecord（supersede/retract/双时间）、Observation/Situation、Work.spec/status（observedGeneration/phase/conditions/unknowns/acceptance/blockers）、9 个 Condition 语义 Gate、Principal/Actor/Role/Commitment、AuthorityRequest/Decision/Evidence（time window/budget/delegation depth/revocation，deny-by-default）、Capability 四态+Lifecycle/Manifest/Release/QualificationRecord/SiteAdmission、ExecutionBinding/ActionAttempt（AttemptStatus 全状态机含 OUTCOME_UNKNOWN/RECONCILING）/Receipt/ReconciliationRecord、ObservedOutcome/AcceptanceDecision/ValueAssessment、Profile（morn.lite/enterprise/factory.readonly/research，production_write=FORBIDDEN）/SolutionPackage
  - `src/work/validation.rs`：真实 schema 验证器（$ref + definition 入口 + type/required/enum/const/minimum/items）
  - 测试：17 个 schema validation/round-trip/mapping 测试 + 1 个模块隔离测试（tests/work_module_isolation.rs：无 cordis/deepseek/dsh::/temporal/kafka/rusqlite/sled 耦合）
  - G1 证据：schema round-trip ✓、旧记录兼容映射（TaskRecord→Work carrier，不建第二账本）✓、protocol types 无 Cordis/DSH/Pi 依赖 ✓

## Current gate status

```text
G0  Reality Audit          PASS
G1  Semantic Mapping       PASS
G2  Durable State          （Phase 2 进行中）
G3  Cordis Isolation       前置证据已就绪（isolation test）；G3 在 Phase 3 收口
G4-G11                     未开始
G12 Customer Gate          BLOCKED_EXTERNAL（无真实客户授权数据）
```

## Tests last run

- Rust: `cargo test --workspace` = **1335 passed, 0 failed**（1300 lib + 34 integration + 1 isolation）；`cargo fmt --check` PASS；`cargo clippy --lib` PASS
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

## Files changed（Phase 1）

- 新增：`schemas/`（8 schema）、`src/work/`（mod/common/world/resource/condition/principal/authority/capability/execution/outcome/profile/validation/tests）、`tests/work_module_isolation.rs`
- 修改：`src/lib.rs`（+pub mod work）

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
2. Phase 2 controllers：WorkController 等 reference controllers（幂等/可重入/crash-safe/lease/checkpoint，输出 condition changes）。
3. Phase 2 G2：进程在 Attempt 后 kill → 重启 → Work 仍在 → Attempt/Binding 不丢 → Reconciliation 继续 → 不重复执行已提交副作用。
4. 测试 → 取证 → 更新本文件 → commit → Phase 3。
