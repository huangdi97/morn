# Morn v11.5-R1 Code Mapping & Migration Matrix

**日期：2026-09-25**
**Phase：0 / G0**
**列定义：** canonical concept · existing code · existing tests · current maturity · v11.5 target · migration action · risk
**原则：** 不推倒重写；EXISTS_PARTIAL 旧资产显式映射迁移；无第二套重复事实账本。

成熟度图例：ATEST=EXISTS_AND_TESTED · PARTIAL=EXISTS_PARTIAL · MISSING=MISSING · LEGACY=LEGACY

## 1. Six Records → v11.5 resources

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| Work Record | `core/storage/tasks/`(TaskRecord), `core/long_task_engine.rs`(TaskStatus), `core/task_engine/` | storage/tasks 23+tests, long_task 内嵌, integration_test | PARTIAL | `Work.spec/Work.status` 协议资源（desired/observed contract） | **新增** `work/` 语义模块（spec/status/Condition）；TaskRecord 保持，作为持久化载体映射 | 低（新增层，不改旧表） |
| World Record | 无 | 无 | MISSING | `ObjectRef/Observation/Situation`（外部系统事实权威） | **新建** | 低 |
| Decision Record | `core/storage/decision_rules/`, `core/decision_rules.rs`, supervisor DecisionRecord | storage/decision_rules 190, tests | PARTIAL | 保留语义：ContextSnapshot/Candidate/Assumption（Agent/Solver 不成为事实权威） | **映射**：新控制平面 Decision 资源复用现有表语义 | 低 |
| Execution Record | `core/scan_bind.rs`, `core/checkpoint.rs`, workflow engine | scan_bind 内嵌, checkpoint 内嵌, workflow engine_tests 5 | PARTIAL | `ExecutionBinding/ActionAttempt/Receipt/Reconciliation` 一等资源 | **新增**（scan_bind 旧名映射为 Binding 语义前身） | 中 |
| Outcome Record | 无（仅 cost report） | — | MISSING | `ObservedOutcome/AcceptanceDecision/ValueAssessment` | **新建**；ValueAssessment 映射 `console/cost/` 语义 | 低 |
| Workforce Record | `src/org/`(Team/User/Permission/Audit), `core/agent_pool/`, persona registry | org 内嵌, agent_pool tests, persona tests | PARTIAL | `Principal/Actor/Role`（Role 与 Agent 解耦） | **映射**：org/persona 保持，Role 语义在 Work/Workcell 中解耦 | 低 |

## 2. Authority / Delegation

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| Authority | `core/approval.rs`(approval gate 300s), `core/security/governance/approvals.rs`, `console/backend` | approval 内嵌 tests, governance/approvals 137 | PARTIAL | `AuthorityRequest/AuthorityDecision/AuthorityEvidence` 契约 + time window/budget/preconditions/delegation depth/revocation | **扩展**：在 approval 之上建 Authority 契约层（Provider-neutral）；reference policy provider | 中 |
| Delegation | `core/delegation.rs`(240) | 内嵌 tests | PARTIAL | delegation depth 语义并入 AuthorityRequest | **映射** | 低 |
| Action Gateway / PEP | 无独立 PEP（tool registry 直接执行） | — | MISSING | side-effect action 必须穿 Gateway/PEP | **新建**（Factory 楔子路径） | 中（安全关键） |

## 3. Attempt / Receipt / Reconcile

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| ActionAttempt | 无 | 无 | MISSING | Attempt 状态机（PROPOSED→…→VERIFIED + 不确定支路） | **新建** | 低 |
| Receipt | `core/security/constitution.rs`(verify_chain hash audit), `hub/gateway.rs`(verify_payment) | constitution 9 tests | PARTIAL | 外部动作 Receipt（source/provenance，transport ack ≠ business commit） | **映射**：hash-chain 语义复用为 receipt 完整性；新建外部 receipt 语义 | 中 |
| Reconciliation | 无 | 无 | MISSING | OUTCOME_UNKNOWN → RECONCILING → VERIFIED/FAILED；business key 回读 | **新建**（G7 核心） | 中 |
| idempotency | 无 business key 体系（task_engine 有 checkpoint） | — | MISSING | work_id/binding_id/attempt_id/idempotency_key/business_key | **新建** | 中 |

## 4. Capability / Skill / Harness

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| Capability | `core/capability/`(def 13, registry 178), `core/registry/`(manager 502, version 336, capability 82) | registry tests, integration_test Registry 部分 | PARTIAL | `CapabilityManifest/CapabilityRelease/QualificationRecord/SiteAdmission` + 四态分离（Declared/Observed/Qualified/Admitted） | **扩展**：registry 语义映射为 Capability 声明层；新增 Manifest/Qualification/Admission | 中 |
| Skill | `component/skill/`(registry, builtins 5, self_evolution), `core/skill_manifest.rs`(288) | skill 内嵌 tests, self_evolution 19 | PARTIAL | Capability 的 skill 实现形态之一 | **映射**：skill 保持为实现载体，语义升级为 Manifest 条目 | 低 |
| Harness | 无（dual_llm 是 model-level；model_router 是路由） | dual_llm tests 15, model_router tests 15 | MISSING | `HarnessProvider` 统一合同（capabilities/startSession/submit/cancel/checkpoint/events/usage/health/close） | **新建**；DSH/Pi 为独立进程 Provider（无真实凭据 → skeleton + contract tests + BLOCKED_EXTERNAL） | 中 |
| Workcell | 无（orchestrator group/modes 是多 Agent 团队） | orchestrator tests | MISSING | Workcell = 某 Work 的最小充分执行组合（动态） | **新建**；orchestrator 团队语义不冒充 Workcell | 中 |

## 5. Studio / SolutionPackage / Blueprint

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| Studio | `src/studio/`(StudioManager, editors, executor), web Studio 视图(AgentBuilder 等 14) | studio/editors 28 | PARTIAL | Studio 以"你希望完成什么工作"为入口；WorkBlueprint/RoleBlueprint editor；Artifact2Capability 流 | **扩展**（Phase 16） | 低 |
| SolutionPackage | 无 | 无 | MISSING | SolutionPackage preview/release（OCI 兼容可选） | **新建**（Phase 13） | 低 |
| Blueprint | `core/agent_templates.rs`, `core/sop_template.rs`, `core/workflow_builder.rs`, `core/supervisor/team_builder.rs` | agent templates(≥8 tests), workflow templates(≥13) | PARTIAL | WorkBlueprint（trigger/goal/constraints/acceptance/required slots） | **映射**：agent/workflow templates 保持；新增 WorkBlueprint 语义 | 低 |

## 6. Factory

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| Factory Domain | 无 | 无 | MISSING | `morn.factory.readonly` Profile + CNC-17 thin slice（READ/MAP/ANALYZE/CANDIDATE/EVIDENCE/EXPERT REVIEW，NO PRODUCTION WRITE） | **新建**（Phase 15） | 中 |
| MES/CMMS/APS/SCADA 连接器 | 无 | 无 | MISSING | Fake MES/CMMS/APS sandbox connectors | **新建**（Phase 8.5） | 低 |
| TF / RV / LC01 / BIOFERM 历史接口 | 零命中 | 零 | LEGACY | 不进入 v11.5 运行面（仅历史追溯） | 无（保持 LEGACY 状态，不迁移） | 无 |

## 7. UI / Workbench / Console

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| Workbench | web App(5 视图) + 30 console panels + studio 14；仅 ExecutionEvent log ticker | vitest 6, e2e 10(2 pass) | PARTIAL（v0.2.0 形态） | Work list/detail + conditions + candidates + authority + attempts/receipts/reconciliation + outcome/acceptance | **扩展**（Phase 16）：延续现栈（React18/Tauri），新增 Work 控制面视图；Agent Session 只作次级信息 | 中 |
| Console | web Console 30 panels（多数 mock 数据）+ Rust console backend | vitest 部分 | PARTIAL | providers health / service slots / capabilities / qualification / site admission / profiles / bindings / external blockers | **扩展**（Phase 16） | 低 |
| Studio 入口 | API-key-centric welcome 流 | e2e ① | PARTIAL | "你希望完成什么工作？"入口 | **扩展**（Phase 16） | 低 |
| i18n | zh/en 702 keys | — | ATEST（修复 Provider 缺失后） | 中文默认 + Unknown/Blocked 显式 | 已修复 Provider/默认语言；Unknown/Blocked 显式随 Phase 16 | 低 |

## 8. Persistence / Event State

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| durable store | SQLite(rusqlite+r2d2) `core/storage/mod.rs:101-381`, sled(vector.rs) | storage tests(174+293+506+…) | ATEST | Control Plane store（work/condition/semantic_event/binding/attempt/receipt/reconciliation/outcome/acceptance/capability_release/qualification/profile_release/outbox/lease） | **扩展**：SQLite 保持（第一版优先利用现有 DB，母版 §32 允许 JSONB→typed projection 演进；SQLite 等价于 local durable store）；不引入 Kafka/K8s/Temporal | 低 |
| event history + projection | event_bus(进程内), sync events | event_bus tests | PARTIAL | append/supersede + deterministic projection + replay + idempotency + outbox | **新建**（Phase 2 核心，G2） | 中 |
| CloudEvents | 无 | 无 | MISSING | CloudEvents 外层 envelope | **新建**（Phase 17） | 低 |
| OTel | tracing(0.1) 宏 | — | PARTIAL | morn.* semantic attrs | **扩展**（Phase 17） | 低 |
| migration | 内嵌 market_*→hub_* 迁移 | storage tests | PARTIAL | forward/rollback strategy 显式 | **映射**（Phase 2/21） | 低 |

## 9. Runtime / Protocol / Policy

| canonical concept | existing code | existing tests | maturity | v11.5 target | migration action | risk |
|---|---|---|---|---|---|---|
| protocol 模块 | `src/protocol/`(仅 a2a) | bridge a2a tests | PARTIAL | language-neutral contracts；service keys 版本化 | **扩展**（Phase 1） | 低 |
| Cordis | 零命中 | 无 | MISSING | `runtime/cordis` adapter（唯一直接依赖区） | **新建**（Phase 3 Spike；upstream pin；不 fork） | 中 |
| Profile | `core/security/profile.rs`(33 行) | 内嵌 | PARTIAL | `Profile` schema + conformance engine（morn.lite/enterprise/factory.readonly/research） | **扩展**（Phase 14，G9） | 中 |
| Sandbox/ExecutionEnvironment | `src/sandbox/`(wasm/wasmtime, feature-gated) | sandbox tests | PARTIAL | `ExecutionEnvironmentSpec/Provider`（local-process/container 优先；microVM 等为 FUTURE_PROVIDER） | **映射/扩展**（Phase 13） | 中 |

## 10. 迁移策略总述

1. **Phase 1** 新建 `work/`（或 `packages/spec` 等价映射为 Rust 模块树 `src/work/`、`src/protocol/` 扩展）语义模块：Work.spec/status、Condition、Situation、Observation、AuthorityRequest/Decision、Binding、Attempt、Receipt、ObservedOutcome、AcceptanceDecision、ValueAssessment、Profile、SolutionPackage——language-neutral schema + serde 类型 + validation tests。旧表不删。
2. **Phase 2** durable control plane：SQLite 扩展表 + event history/projection + outbox + controllers（幂等/可重入/crash-safe/lease/checkpoint）。
3. **Phase 3** Cordis spike：`runtime/cordis` adapter（Rust 侧等价位置 `src/runtime/cordis/`），upstream pinned。
4. **Phase 4-5** HarnessProvider 合同 + DSH/Pi skeleton（真实凭据缺失 → BLOCKED_EXTERNAL）+ Binding pin + 故障注入测试。
5. **Phase 6-7** Action/Receipt/Reconciliation（Fake CMMS 全链路测试）+ Outcome/Acceptance/Value。
6. **Phase 8-13** Trust/Governance + ExecutionEnvironment + Capability Supply Chain + Resolver/Workcell + Artifact2Capability + Package。
7. **Phase 14-16** Profiles + Factory thin slice + UI。
8. **Phase 17-22** Telemetry + Threat model + Conformance + Docs/ADR + verify + Release/Evidence。

**映射原则执行确认：** 本矩阵未发现"设计即实现"误报未记录的情况（AgentLoop 硬编码输出已在 Reality Audit 记录为 Reality Gap）；
无第二套事实账本（Six Records 映射为分散表语义，新控制平面复用而非并行重复）；
TF/RV/LC/BIOFERM 保持 LEGACY 不迁移。

**G1 语义映射 Gate 的前置证据（本矩阵部分）已就绪；G1 在 Phase 1 完成后收口。**
