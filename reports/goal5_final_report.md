# Morn Goal 5 Final Report — Core v1.0 Release Candidate

Date: 2026-08-16
Goal ID: `MORN-G5-CORE-COMPLETION-V1RC`

## CORE COMPLETE = YES

Morn Core v1.0 RC 完成：删除/不安装任何领域实例（BioLab、Factory、Pharma 等）后，Morn 仍然是一个
完整可运行系统；新行业通过 Domain SDK / Pack / Connector / Provider 扩展，不修改 Stable Semantic Kernel。
唯一非本地项为两个既有 external blocker（B-001 真实 DSH smoke、G4-B-002 真实 BioLab 数据 pilot），与 Core 本体无关。

## 1 Starting / Final Commit
- starting: `088cf91`（Goal 4 closeout；M0 reality audit）
- final: `07dde38`（G5 全量 closeout；含 93a7077 代码补齐 + 07dde38 文档/验收）

## 2 Architecture before/after
- Before：morn-app → morn-biolab（Core 依赖领域）；kernel 含领域 ID；compiler 含 biolab 启发式；
  UI 硬编码 BioLab 卡片；无 Connector/Node/Domain SDK/Pack/Plugin/CLI。
- After：Core crates 零领域（仅 optional feature 可挂 reference pack）；kernel 冻结 22 个 canonical
  contracts；Capability/Provider/Connector/Process/Node/Distributed/Deployment/Domain SDK/Pack/Plugin/
  CLI/Compatibility/Security/Chaos/Conformance 全部就位；UI 经 domain_packs 数据驱动注入。

## 3 Domain Neutralization
- `reports/domain_neutralization_audit.md`：CORE_BUG 全部消除（kernel 领域 ID → reference pack；
  compiler 领域启发式 → 通用 governed 模板，DECISIONS D-027）。
- 自动守卫：`scripts/check_domain_boundary.ps1`（依赖方向 + 领域词启发式）+ ArchitectureConformance 测试。
- 证明：zero-domain server `domain_packs=[]`、`/api/biolab/run` 404；all-features server
  `domain_packs=[biolab-reference]`、BioLab E2E all_ok=True。

## 4 Reference Extraction
- BioLab 迁至 `domain-packs/biolab-reference`（crate `morn-biolab-reference`），仅依赖 public Morn SDK。
- `reference_e2e`：Core 零领域启动 → install/validate/build/enable → 领域声明注入 →
  reference conformance E2E → disable（声明消失）→ uninstall（历史 provenance 保留）→ Core 健康。

## 5 Stable Semantic Kernel v1 Freeze
- `morn-kernel/src/contracts.rs`：22 个 canonical contracts、`SEMANTIC_CONTRACT_V1`、`ContractSnapshot::v1`、
  `ContractCompatibility`（Compatible/RequiresReevaluation/Incompatible）、`DeprecationPolicy`。
- 测试：snapshot complete、compat matrix、provider cannot redefine canonical contracts。

## 6 Capability / Provider SDK
- HarnessProvider（MornNativeHarness + DeepSeekHarnessProvider fixture，contract smoke 双实现）。
- IntelligenceProvider（RuleIntelligence + SolverIntelligence，conformance 双 fixture；非 AI-only）。
- RuntimeProvider（新增：morn-runtime 协议 + FixtureRuntime + conformance kit）。
- Provider 不写 canonical DB、不绕 Action Gateway；chaos 证明 timeout/malformed → 显式错误。

## 7 Connector SDK
- `morn-integration`：ConnectorSpec/Instance/MappingSpec/SyncCursor/ExternalActionRequest/ConnectorReceipt/
  ApprovedActionToken/ConnectorRegistry + GenericFixtureConnector。
- Read path（External→normalize→proposal）与 governed write path（Policy→Approval→Gateway→Connector→Verify→Receipt）。
- Conformance/chaos：timeout、duplicate、idempotency、token 校验。

## 8 Generic Process Intelligence
- `morn-process`：ObservedEvent→Trace→ObservedWorkGraph→ProcessSignals
  （repeated_handoff / wait_bottleneck / rework / loop_detected / duplicate_approval / manual_copy）。
- 无领域 ontology；activities 任意字符串。

## 9 Morn Node / Distributed Durable Runtime
- `morn-node`：MornNode/NodeType/NodeId/health/lease；DistributedRuntime claim/heartbeat/checkpoint/
  failover/lost_leases/dedupe。
- 真 2-node 本地证明：A claim→checkpoint→A 死→lease 过期→B failover→duplicate event dedupe→
  外部效果不重复→owner 转移（`two_node_failover_with_dedupe_and_idempotency` + `pure_core_e2e` Phase 3）。

## 10 Deployment / Topology
- `DeploymentSpec`（runtime/storage/secret/policy binding + upgrade/rollback strategy）+ `Topology`/
  `NodeGroup`/`PlacementRule`；`Topology::validate` 测试 valid + incompatible placement rejected。

## 11 Domain SDK v1 / Pack Lifecycle / Plugin
- Domain SDK：DomainDefinition/DomainDeclaration/DomainRegistry，13 种声明（含 connector_requirement/evaluation/ui_extension）。
- PackLifecycle：init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff；历史不删。
- PluginManifest：deps/permissions/core+sdk compat/migrations/health；validate 校验 type/name 安全。
- Core policy 不可绕过：ActionGateway + connector token + architecture guard。

## 12 Generic Product Surfaces（zero-domain UI）
- Workbench/Studio/Console/Hub 零 Domain 完整可用；BioLab UI 由 backend `domain_packs` 广告数据驱动
  （`biolabEnabled()`，DECISIONS D-028）；Studio 默认通用；无 static fake cards。

## 13 Developer CLI
- `morn doctor|status|node|provider|connector|plugin|domain|package|compat|migrate|conformance`
  —— 全部走 canonical services；run_all 的 CLI smoke 覆盖（12 commands exit 0）。

## 14 Compatibility / Migration / Security / Chaos / Conformance
- Migration：preflight/snapshot/dry-run/apply/verify/restore；downgrade 拒绝；fresh+upgrade idempotent。
- Security：workspace isolation、secret redaction、E3 approval、connector token、node auth、
  pack-name 安全（path traversal/injection）、audit/ledger。
- Chaos：7 个真实 failure injection（provider/connector/node/duplicate signal/checkpoint mismatch/
  migration/plugin），全部显式 recover/block/escalate/compensate。
- Conformance：Provider/Runtime/Connector/Plugin/DomainPack/Architecture 6 套 kit。

## 15 E2E
- Pure Core E2E：零 Domain start→workspace→generic work→capability/workcell→execute→approval→
  artifact→outcome→restart→node failover→connector governed action→receipt→history readable。
- Reference E2E：install biolab-reference→validate/enable→领域类型/UI 注入→conformance/E2E→
  disable→uninstall→Core 健康→历史 provenance 可读。

## 16 Commands / Results（2026-08-16）
```text
scripts/run_all.ps1                       -> exit 0（=== Verification complete ===）
cargo fmt --check / check / clippy -D warnings / test --workspace --all-features
                                          -> pass（64 suites, 218 passed, 0 failed, 0 ignored）
cargo test -p morn-core-tests             -> pass（chaos 7 / conformance 6 / migration_security 10 /
                                               pure_core_e2e 1 / reference_e2e 1）
zero-domain Core build（cargo check -p morn-app）-> pass
domain boundary guard                     -> pass
developer CLI smoke                       -> pass（12 commands）
frontend typecheck/lint/test/build        -> pass（4 tests）
tauri desktop build                       -> pass
frontend ui smoke (playwright)            -> pass（4 surfaces + BioLab E2E + Goal2/3 + studio compiler）
demo smoke                                -> pass（health=ok bootstrap_objects=1 e2e_steps=7 objects=1）
```

## 17 External Blockers（非本地、不阻塞 Core）
- B-001：真实 DeepSeek Harness smoke 需官方安装物或真实 API 凭据（Morn provider contract 已完成并通过）。
- G4-B-002：真实 BioLab 数据 pilot 需合法真实 dataset（pipeline contract 已完成，FULL pilot 保持 blocked）。

## 18 Deferred instance-layer items（不属于 Core v1.0 RC）
- 行业实例（BioLab/Factory/Pharma 等）仅作为 reference/conformance packs 存在，不进入 Stable Semantic Kernel。
- 真实分布式部署（K8s/云）、真实硬件自治、真实多系统 ERP/MES/LIMS 集成 —— 通过 Connector SDK / Node 扩展。
