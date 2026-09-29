# Morn v11.5-R1 Execution State

> 本文件由 Agent 自主运行机制维护（Goal §30）。下一 Agent 必须从本文件 + `docs/canonical/CURRENT.md` + canonical 母版开始，不从聊天记忆猜测。

 **最后更新：** Phase 22 完成——Goal 收口（ENGINEERING_COMPLETE_WITH_EXTERNAL_BLOCKERS，2026-09-28）

 `main` @ `b26162a`（Phase 22 commit）；Phase 0 基线 HEAD 为 `2cd9fbd6fa57930eaa71616f170f958037190654`。

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
- **Phase 3 — Cordis Reference Composition Runtime**（G3 = PASS）
  - **Cordis Spike 完成**（真实 dependency，非 fork）：upstream `cordis-rs` =0.8.39（facade over cordis-core 0.3.24），github.com/dshbox/cordis-rs，MIT，DeepSeek Harness Cordis 4.x 的 Rust port，MSRV 1.85（本机 rustc 1.97.1 ✓）；crates.io API 核验（max_version 0.8.39，72 versions，pre-1.0 rapid）；exact pin 在根 Cargo.toml（`cordis-rs = "=0.8.39"`，无宽泛 semver）
  - `src/runtime/cordis/`（唯一直接依赖 Cordis 的区域）：mod.rs（facade + 禁止等价映射文档 + upstream pin 常量）、slots.rs（8 个 Morn service slot：capabilities/harnesses/solvers/executionEnvironments/connectors/policy/identity/telemetry，inherent CONTRACT 常量 + Service trait NAME）、host.rs（CompositionHost：mount/unmount/upgrade/scoped，Cordis 错误 → ServiceSlotError）
  - G3 证据（tests/runtime_cordis_g3.rs，8 测试全绿）：provider register/unregister ✓、provider reload 不 rebind active Attempt（**binding pin**：reload 后 Binding 仍 BOUND/同 provider/同 generation，Attempt 仍 pin 原 Binding）✓、provider upgrade 身份保持 ✓、scoped service 隔离 ✓、unavailable typed 错误 ✓、slot contract names 稳定 ✓、upstream pin 记录 ✓、Work lifecycle Morn-owned（Fiber ≠ Work）✓
  - 隔离测试扩展（tests/work_module_isolation.rs）：扫描 src/work + src/control（含 controllers 子目录）共 24+ 文件，import-path 级检测（cordis::/use cordis/deepseek/dsh::/temporal::/kafka::/sled::）= 0 violations ✓
  - 附带根因修复：self_evolution 测试并发 bug（mod.rs 与 scan.rs 两个独立 guard 不互斥 → 统一迁入 scan_tests.rs 单 guard；scan.rs 313 行超限 → 拆分 scan.rs 210 + scan_tests.rs 178）
 - **Phase 4 — Execution Plane**（G4 = PASS_WITH_LIMITATIONS，G5 = PASS_WITH_LIMITATIONS）
   - 收编上一轮未提交半成品（Goal §27）：`src/providers/` 9 文件 + `tests/providers_g4g5.rs` + `src/lib.rs`（+pub mod providers），commit `e21756f`
   - HarnessProvider 统一最小合同（capabilities/startSession/resumeSession/submit/cancel/checkpoint/getEvents/getUsage/health/close；session events 仅 Harness evidence）
   - DshHarnessProvider / PiHarnessProvider：out-of-process skeleton，CredentialProvider 注入（凭据不硬编码不入 git），`REAL_PROVIDER_INTEGRATION_BLOCKED` 诚实标记（无凭据/无 endpoint → ExternalBlocked，不伪造 session）
   - FakeHarnessProvider（deterministic，control-plane 验证）+ ReferenceCpsatSolver（4 候选，无 Agent）+ ReferenceHumanProvider（assign/decision-once/escalate）+ FakeCmms/FakeMes Connector（business-key 幂等 + timeout-after-commit 模式）
   - EnvCredentialProvider：scope→value 映射不越界 + redacted fingerprint（安全日志）
   - G4/G5 证据（tests/providers_g4g5.rs，9 测试全绿）：fake 全合同 ✓、DSH/Pi honest blocker 合同 ✓、同 trait 对象（harness-neutral）✓、credential 边界 ✓、solver deterministic 无 Agent ✓、human 全路径 ✓、Fake CMMS 幂等/无重复 ✓、timeout-after-commit→unknown 真实存在 ✓
   - 附带根因修复：上一轮工具 UTF-8→GBK 双重编码乱码（11 处注释）→ 字节级替换修复（tests/providers_g4g5.rs 10 处 + src/work/execution.rs 1 处），mojibake=0 验证
- **Phase 5 — Binding Pin**（G6 = PASS）：`tests/binding_pin_g6.rs` 4 故障注入（unload 中执行 pin 不变/restart 无重复副作用/second provider 新 Binding/side-effect attempt 先 reconcile）+ 根因修复：receipt readback 不降级 verified attempt（965c4e0）
- **Phase 6 — Receipt/Reconciliation**（G7 = PASS）：`tests/receipt_reconciliation_g7.rs` 5 测试（完整 gateway 链/retry storm/duplicate receipt/crash-after-commit 持久 fake CMMS）（e5b7704）
- **Phase 7 — Outcome/Acceptance/Value**：`tests/outcome_acceptance_value.rs` 5 测试 + 根因修复：REJECT/CONDITIONAL 不 satisfy acceptance gate（e24c72e）
- **Phase 8 — Trust/Governance**：PolicyProvider（deny-by-default）+ ActionGateway PEP + enforcement 测试 + env-var test race 修复（bb05aeb）
- **Phase 9 — Execution Environment**：ExecutionEnvironmentSpec（DSec）+ local-process/container + future providers 诚实 FutureProvider + admission 检查（84b2919 + 27bead7）
- **Phase 10 — Capability Supply Chain**（G8 = PASS）：四态分离 + lifecycle + strict-profile qualification gate + get_latest_qualification（e22521d）
- **Phase 11 — Resolver/Workcell**：WorkcellResolver（硬约束 vs 软评分）+ typed resolution failures + 0-Agent/Agent+Rule/Agent+Solver+Human（7a87719）
- **Phase 12 — Artifact2Capability**：OpenAPI2Capability + Repo2Capability 确定性 compiler + 结构化失败状态 + qualification-gated release（dbc8165）
- **Phase 13 — Package/Provenance**：content-addressed SHA-256（ring，OCI-mappable）+ LOCAL_UNSIGNED provenance + tampering detection + PackageInputs options object（clippy）（a26b092 + 75398d9）
- **Phase 14 — Profiles**（G9 = PASS）：四 Profile conformance suite + 机器可读报告 + required-condition gate + frozen version（9c7b55e）
- **Phase 15 — Factory Thin Slice**（G10 = PASS）：CNC-17 deterministic scenario + 全 canonical 链 + evidence bundle（binding pin/no-duplicate/audit timeline）+ swap 入口（13e1080）
- **Phase 16 — UI**：playwright config msedge channel（browser provider 修复）+ e2e 真跑（2 passed/8 failed——welcome-flow 时序/遮挡 + UI 结构漂移，已知 gap P3）
- **Phase 17 — Telemetry**：morn.* 11 semantic attrs + CloudEvents envelope + secret redaction（b845dd6）
- **Phase 18 — Threat Model**：18 威胁矩阵（docs/security/MORN_V11_5_THREAT_MODEL.md）+ reordered/duplicate/slow/upgrade fault injection + 根因修复：projection retract 取消被 retract 事件效果（2591513）
- **Phase 19 — Conformance/Swap**（G11 = PASS_WITH_LIMITATIONS）：swap benchmark（Run A=fake-dsh Run B=fake-pi 业务不变条件一致）+ provider conformance 同一合同（d869886）
- **Phase 20 — Docs/ADR**：ADR-MORN-001..010 + 5 guides + ARCHITECTURE.md 六 Plane 映射 + CURRENT 指针（b367db2）
- **Phase 21 — Verify 收口**：scripts/verify-all.ps1 统一 gate（fmt/clippy/Rust tests/web vitest+build/e2e SKIP_EXTERNAL 显式，fail-fast）——RESULT: ALL_LOCAL_GATES_PASS（3922f69）
- **Phase 22 — Release/Evidence Bundle**：7 份收口报告（Gate Matrix/Provider Swap/Factory Scenario/Security/Failure/Migration/Final）+ evidence bundle（HEAD/依赖/测试计数/场景 ID/timeline/screenshots/blockers）（b26162a）
## Current gate status

```text
 G0  Reality Audit          PASS
G1  Semantic Mapping       PASS
G2  Durable State          PASS
G3  Cordis Isolation       PASS
 G4  DSH Provider           PASS_WITH_LIMITATIONS（skeleton contract + honest blocker；真实接入 BLOCKED_EXTERNAL）
 G5  Pi / Second Harness    PASS_WITH_LIMITATIONS（同一合同 + harness-neutral；真实 swap BLOCKED_EXTERNAL）
 G6-G11                     未开始
G12 Customer Gate          BLOCKED_EXTERNAL（无真实客户授权数据）
```

## Tests last run

- Rust: `cargo test --workspace` = **1434 passed, 0 failed**（1301 lib + 34 integration + 99 conformance/G-gates，连续两次运行一致）；cargo fmt --check PASS；cargo clippy --lib -- -D warnings PASS（CI 标准）；`cargo fmt --check` PASS；`cargo clippy --lib -- -D warnings` PASS（CI 标准）
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

## Files changed（Phase 4）

- 新增：`src/providers/`（mod/harness/harness_dsh/harness_pi/harness_fake/solver/human/connector/credential）、`tests/providers_g4g5.rs`
- 修改：`src/lib.rs`（+pub mod providers）、`src/work/execution.rs`（注释乱码修复）

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
| DSH Harness Provider 真实接入 | DeepSeek Harness 进程/接口访问 + API key | 用户提供 DSH 运行环境与凭据 | skeleton + contract tests 已完成（e21756f）；解除后：DSH contract smoke（start/resume/cancel/events）真跑 |
| Pi Harness Provider 真实接入 | Pi 访问 + 凭据 | 用户提供 | skeleton + contract tests 已完成（e21756f）；解除后：Pi contract + swap benchmark 真跑 |
| 真实客户数据 / 现场（G12） | 客户授权数据、现场安全/回滚/审批条件 | 客户 + 人类授权 | 母版 0.2 边界重新验证；read-only/shadow 计划 |
| Cordis upstream 可用性 | 需确认 upstream Cordis exact version/commit（公开仓库核验） | Phase 3 Spike 时核验（网络可达则自动） | exact pin + 许可证记录 |

## Next exact actions
Goal 已收口：ENGINEERING_COMPLETE_WITH_EXTERNAL_BLOCKERS。
1. 解除外部阻塞（见 External blockers 表）：DSH/Pi 凭据、真实客户授权、Sigstore signer、OPA 部署。
2. push plan：git push origin main（无 force push；推前确认 branch policy）。
3. 下一产品步骤（需人类授权）：read-only shadow → DSH/Pi 真实接入 → 第二站点复用验证。
