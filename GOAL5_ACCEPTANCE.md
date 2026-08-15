# GOAL5_ACCEPTANCE.md

> 状态：2026-08-16 逐条核对。`[x]` = 通过（代码 + 测试 + 命令）。全量验证
> `scripts/run_all.ps1` exit 0：Rust 64 suites / 218 passed / 0 failed / 0 ignored；
> frontend typecheck/lint/test(4)/build；CLI smoke 12 commands；Tauri build；UI smoke 全 OK。

## A Baseline
- [x] Goal1–4 regressions green —— `scripts/run_all.ps1` exit 0（cargo test/clippy/fmt、frontend、Tauri、UI smoke、demo smoke、CLI smoke）
- [x] reality audit complete —— `reports/goal5_reality_audit.md`（2026-08-15 初版 + 2026-08-16 完成态审计）

## B Domain Neutrality
- [x] domain audit —— `reports/domain_neutralization_audit.md`（CORE_BUG 全部消除，2026-08-16 Resolution）
- [x] no Core→domain dependency —— `scripts/check_domain_boundary.ps1`（含领域词启发式守卫）+ `conformance::architecture_conformance_core_has_no_domain_pack`；`morn-app` 仅 optional feature-gated 依赖
- [x] Core zero-domain build/start/work —— `cargo check -p morn-app`（零领域）；zero-domain server：domain_packs=[] 且 /api/biolab/run 404；`pure_core_e2e`
- [x] generic UI zero-domain —— Workbench `biolabEnabled()` 门控 + Studio 通用默认（DECISIONS D-028）；frontend 单元测试

## C Reference Extraction
- [x] BioLab outside Core —— `domain-packs/biolab-reference`（crate `morn-biolab-reference`），Core crates 无 non-optional 依赖
- [x] public Domain SDK only —— biolab-reference Cargo.toml 仅依赖 kernel/world/artifact/work/org/actor/harness/runtime/capability
- [x] conformance passes —— `reference_e2e` + ui_smoke BioLab E2E OK
- [x] Core passes with pack absent —— zero-domain build/test/server 全绿
- [x] history preserved after disable/uninstall —— `reference_e2e`（history 含 uninstalled；manifest record 保留）；`package::pack_lifecycle_full_cycle_preserves_history`

## D Stable Kernel
- [x] semantic contract v1 —— `morn-kernel/src/contracts.rs`（22 canonical contracts, `SEMANTIC_CONTRACT_V1`, `ContractSnapshot::v1`）
- [x] API/schema versions —— `morn-kernel::version::Version`；store schema v2
- [x] compatibility/deprecation policy —— `ContractCompatibility`（Compatible/RequiresReevaluation/Incompatible）+ `DeprecationPolicy`
- [x] contract tests —— kernel contracts 3 tests（snapshot complete/compat matrix/provider cannot redefine）

## E Capability/Provider
- [x] generic capability types —— `morn-capability`（effect/execution modes）
- [x] no AI-only assumptions —— `RuleIntelligence`/`SolverIntelligence`/Program/Human 均受支持（provider_conformance_two_fixtures）
- [x] HarnessProvider/RuntimeProvider —— `morn-harness::HarnessProvider`（morn-native + DeepSeek fixture）；`morn-runtime::RuntimeProvider`（新协议 + FixtureRuntime）
- [x] >=2 implementations/fixtures —— IntelligenceProvider×2 + HarnessProvider×2 + RuntimeProvider fixture
- [x] conformance/health/fallback/permission —— harness contract_tests + conformance.rs（provider/runtime）+ chaos（timeout/malformed 显式错误）

## F Connector
- [x] generic contract/external refs/mapping/sync cursor —— `morn-integration`（ConnectorSpec/Instance/ExternalObjectRef/MappingSpec/SyncCursor/ConnectorReceipt）
- [x] governed writes —— `ConnectorProvider::execute_governed` 需 `ApprovedActionToken`（connector_write_requires_gateway_token）
- [x] retry/idempotency —— GenericFixtureConnector duplicate → idempotent（chaos/conformance）
- [x] conformance fixture —— `connector_conformance_governed_write_idempotent` + chaos connector tests

## G Process Intelligence
- [x] generic trace/observed graph —— `morn-process`（ObservedEvent/Trace/ObservedWorkGraph/ProcessMiner）
- [x] bottleneck/wait/rework/loop/handoff primitives —— ProcessSignals（repeated_handoff/wait_bottleneck/rework/loop_detected/duplicate_approval/manual_copy）
- [x] no domain dependency —— `no_domain_ontology_required` 测试；activities 为任意字符串

## H Node/Distributed
- [x] node identity/registration/capabilities/resources/health/lease —— `morn-node`（MornNode/NodeType/NodeId/health/lease）
- [x] claim/heartbeat/failover/checkpoint transfer/remote execution/signal/dedupe —— `DistributedRuntime`（claim/heartbeat/checkpoint/failover/lost_leases）+ `DurableRuntime`（signal/checkpoint/restore）
- [x] real local 2-node E2E —— `two_node_failover_with_dedupe_and_idempotency` + `pure_core_e2e` Phase 3（A claim→checkpoint→stale lease→B failover→dedupe→无重复外部效果）

## I Deployment
- [x] topology/placement/runtime-storage-secret-policy bindings —— `DeploymentSpec`（runtime/storage/secret/policy binding）+ `Topology`/`NodeGroup`/`PlacementRule`
- [x] validation/upgrade/rollback strategy schema —— `Topology::validate` + upgrade/rollback_strategy 字段；`deployment_topology_validation`（valid + incompatible placement rejected）

## J Domain SDK / Pack
- [x] ontology/type/action/event/artifact/outcome/role/work/policy/capability/connector/evaluation/UI extension contracts —— `DomainDefinition::declare` 13 kinds（含 connector_requirement/evaluation/ui_extension）；validate 拒绝未知 kind
- [x] init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff —— `PackLifecycle` 全生命周期 + 测试

## K Plugin
- [x] manifest/deps/permissions/compat/migrations/health/lifecycle —— `PluginManifest` 全字段 + `validate`（plugin_type 白名单、core_compat、safe_name）
- [x] Core policy cannot be bypassed —— ActionGateway 唯一状态提交路径；connector 需 token；architecture guard（插件/连接器不能 raw write canonical DB）

## L Product UI
- [x] zero-domain Workbench/Studio/Console/Hub —— 4 surfaces 零 Domain 可用（unit test + server 实测 domain_packs=[]）
- [x] dynamic domain UI extension —— `domain_packs` 广告 + `biolabEnabled()` 数据驱动门控
- [x] no static fake must-pass data —— 全部真实 backend API；Playwright 断言真实渲染

## M CLI
- [x] doctor/status/node/provider/connector/plugin/domain/package/compat/conformance —— `morn-cli` 全部子命令 + `migrate`；run_all 的 CLI smoke 覆盖（12 commands exit 0）

## N Compatibility/Migration
- [x] matrix/preflight/fresh+upgrade/failure/restore plan/no silent destructive migration —— migration_security.rs（migration_plan_preflight_dryrun_apply_verify、downgrade_requires_restore_plan、fresh_and_upgrade_migration_are_idempotent、migration_failure_does_not_corrupt）+ kernel contracts compat

## O Security
- [x] workspace isolation —— `cross_workspace_artifact_leakage_denied`
- [x] secret redaction —— `secret_never_serialized_to_audit`
- [x] provider-connector permissions —— `connector_write_requires_gateway_token`；E3 需 approval（`e3_approval_enforced`）
- [x] node auth —— `node_identity_and_lease_enforced`
- [x] package permissions —— `safe_name`（path traversal/control/injection 名拒绝）`unsafe_names_rejected_path_traversal_and_injection`
- [x] audit —— kernel ledger；ExecutionReceipt/记录不可变（persistence_goal4）
- [x] injection —— `command_injection_boundary_in_pack_names`（argv 不 shell 插值 + 名校验）
- [x] path traversal —— `path_traversal_rejected_by_pack_manifest`

## P Reliability
- [x] process/node/provider/connector/DB/migration/plugin/duplicate event/action failure tests —— chaos.rs 7 tests（provider timeout/malformed、connector timeout/duplicate、node loss/stale lease failover、duplicate signal、checkpoint mismatch、migration failure、plugin init failure）+ pure_core restart
- [x] explicit recover/block/escalate/compensate —— 每个 chaos 断言显式状态（Blocked/Escalated/Err/idempotent），无 silent corruption

## Q Conformance
- [x] Provider/Runtime/Connector/Plugin/DomainPack/Architecture conformance —— conformance.rs 6 tests

## R Final
- [x] Pure Core v1.0 RC E2E —— `pure_core_e2e::pure_core_e2e_zero_domain`
- [x] Reference Pack E2E —— `reference_e2e::reference_pack_e2e`
- [x] backend regression —— run_all exit 0（218 Rust tests）
- [x] frontend typecheck/lint/test/build —— 全过（4 tests）
- [x] desktop build where supported —— `cargo build -p morn-desktop` pass
- [x] UI smoke —— Playwright 4 surfaces + BioLab E2E + Goal2/3 交互 + studio compiler 全 OK
- [x] security + chaos —— core-tests（chaos 7 / migration_security 10）
- [x] reports/goal5_final_report.md —— 已生成
- [x] CORE COMPLETE explicitly stated —— `reports/goal5_final_report.md`：CORE COMPLETE = YES

## 禁止项核对（均不成立）
- 无 fake distributed runtime（2-node 真 failover/dedupe 测试）
- 无 fake connector/provider（GenericFixtureConnector + conformance；DSH Real 不伪造）
- 无 static fake UI（全部真实 API）
- 无 ignored tests / 弱化断言（0 ignored；断言均为真实行为）
- 无 todo!() / unimplemented!()
- BioLab 不是改名冒充 generic（独立 reference pack，Core 零领域可运行）
- 插件/Connector 不能绕过 Core 权限（token/gateway/architecture guard）
- uninstall 不删除历史 provenance（reference_e2e + package 测试）
