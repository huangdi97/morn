# GOAL5.md — Morn Core Completion / v1.0 RC

## Goal ID
`MORN-G5-CORE-COMPLETION-V1RC`

## Mission
把当前 Morn 收敛为一个领域无关、插件化、可部署、可扩展、可迁移、可测试、可升级、可分布式运行、可被任意 Domain Pack 使用的稳定 Work OS / Digital Workforce OS / Evolvable Organization Runtime。

## Core Boundary

Morn Core 可以拥有：

```text
Identity / Workspace / Policy / Permission / Approval / Secrets / Ledger / Lifecycle / Version / Package
Operational Object / Relation / State / Event / Action / StateDiff
Artifact / Decision / Outcome / Evidence / Provenance
Work / WorkPackage / WorkContract / AcceptanceSpec / OutcomeContract / WorkGraph / Workcell
Actor / Role / Member / Delegation / Commitment / Accountability / Representation
Capability / Harness / Runtime / Provider / Connector / ExecutionMode / EffectClass
Durable Workflow / Checkpoint / Signal / Retry / Compensation / Escalation / Attention
Solution Compiler / Organization Compiler / Replay / Simulation / Evaluation / Certification
Evolution / Distillation / Managed Work / Replacement
Node / Deployment / Topology / Registry / Observability / Audit
Domain SDK / Extension SDK / Plugin SDK / Package/Hub contracts
```

Morn Core 禁止拥有具体行业语义：

```text
Dataset / ScientificClaim / Hypothesis / ExperimentDesign / Patient / Drug / Assay
ProductionOrder / Machine / MaintenanceOrder / QualityIncident / MuseumExhibit / ...
```

这些只能由 Domain Pack 定义。

## Dependency Law

```text
Infrastructure
↑
Kernel / Semantic Contracts
↑
World / Work / Artifact / Organization
↑
Capability / Harness / Runtime / Connector
↑
Durable / Compiler / Assurance / Evolution
↑
Product Surfaces / SDK / CLI
↑
Domain Packs / Solutions / Instances
```

严格禁止 `Morn Core → Concrete Domain Pack`。

## Milestones

### M0 Reality Audit
读取 Goal1–4 final reports（若存在）、真实代码、DB/migration、UI、测试；运行全量 regression；生成 `reports/goal5_reality_audit.md`。

### M1 Domain Neutralization Audit
扫描 crates/modules/API/UI/schema/tests/fixtures/CLI/package，识别领域泄漏；生成 `reports/domain_neutralization_audit.md`。

### M2 Extract Existing Reference Domain
把 BioLab 等现有实例逻辑移出 Core，改成 `domain-packs/biolab-reference` 或等价 reference pack；Core 零 Domain 仍可 build/start/test；reference 只依赖 public Domain SDK。

### M3 Stable Semantic Kernel Freeze v1
冻结 canonical IDs、workspace、world、work、artifact/decision/outcome、actor/role、capability、policy、effect、provenance、version/lifecycle；定义 schema/API/compat/deprecation/migration contracts。

### M4 Capability Fabric Completion
统一 CapabilitySpec/Version/Binding/Provider/Invocation/Result/Health/Policy/Cost/Context。类型必须覆盖 Rule/Program/Solver/Model/LLM/Tool/API/Service/Human/Device/Hybrid，不能 AI 特权化。

### M5 Harness / Runtime / Provider SDK
完成 HarnessProvider、RuntimeProvider、可选 IntelligenceProvider；支持 health、scope/session、invoke、events、pause/resume/checkpoint/restore/signal、teardown；至少两个不同本地实现/fixture 通过 conformance。

### M6 Connector / Integration SDK
完成通用 discover/auth/read/query/subscribe/poll/propose_write/execute_governed_action/normalize/map/health/retry/rate_limit/teardown；建立 ExternalSystem/Object/Action/Event、MappingSpec、SyncPolicy/Cursor、Receipt；所有 write 经过 Action Gateway。

### M7 Generic Process Intelligence
实现领域无关 ObservedEvent/Trace/Handoff/WorkCandidate/ObservedWorkGraph/Variant/Bottleneck/Rework/Waiting/Loop/ManualCopy/DuplicateApproval；generic fixture 能从事件恢复真实 observed process。

### M8 Morn Node
NodeIdentity/Type/Capabilities/Resources/Health/Policy/Endpoint/Registration/Lease/Status；至少 Desktop/Team/PrivateServer/Edge/Worker。

### M9 Distributed Durable Runtime
work claim、lease、heartbeat、worker loss、failover、checkpoint transfer、remote execution、signal routing、idempotency、at-least-once event handling、dedupe、stale lease recovery、capability routing。用本地 2-node/worker 做真正 failover 证明。

### M10 Deployment / Topology
DeploymentSpec/Topology/NodeGroup/PlacementRule/RuntimeBinding/SecretBinding/StorageBinding/PolicyBinding/UpgradeStrategy/RollbackStrategy；表达 desktop/server/team+workers/private+edge；不要求真实 K8s。

### M11 Domain SDK v1
允许领域在不改 Core 下定义 ontology、objects、relations、actions、events、policies、roles、work templates、artifacts、outcomes、capabilities、connector requirements、evaluations、UI extensions。

### M12 Domain Pack Lifecycle
Draft→Validated→Built→Installed→Enabled→Disabled→UpgradePending→Deprecated→Uninstalled；支持 init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff；uninstall 不能破坏历史 provenance。

### M13 Extension / Plugin System
支持 capability/harness/runtime/connector/domain/evaluation/simulation/UI/CLI extensions；manifest/version/dependencies/permissions/compatibility/lifecycle/health/sandbox hooks/install/uninstall/enable/disable；插件不能绕过 Core 权限。

### M14 Generic Product Surfaces
Workbench/Studio/Console/Hub 在零 Domain 下完整可用；任何 BioLab 入口只能由 reference pack 注入；禁止 Core UI 大量 `if domain == biolab`。

### M15 Developer CLI / SDK
统一或补齐 `morn doctor/status/migrate/node/provider/connector/plugin/domain/package/compatibility/conformance/test core`，复用 canonical services，不绕 DB。

### M16 Compatibility / Versioning / Migration
Core API、Semantic Contract、Package Manifest、Domain SDK、Provider Protocol、Plugin Protocol、DB Schema 版本；compatibility matrix；preflight/snapshot/migration/verify/restore plan。

### M17 Security / Isolation / Audit Hardening
workspace isolation、secret redaction、permission enforcement、node auth、plugin/connector permissions、audit integrity、path traversal、command injection boundary、unsafe external write、secret leakage tests。

### M18 Reliability / Failure / Chaos
process crash、node loss、duplicate/delayed/reordered event、stale lease、network unavailable、provider crash/malformed result、connector timeout、DB busy、partial transaction、corrupt manifest、version mismatch、migration failure、restart during action、duplicate action；系统必须 explicit recover/block/escalate/compensate，不 silent corrupt。

### M19 Conformance Framework
ProviderConformance / RuntimeConformance / ConnectorConformance / PluginConformance / DomainPackConformance / ArchitectureConformance；以后新增扩展无需修改 Core test suite 即可验证。

### M20 Core v1.0 RC
Pure Core E2E：零 Domain → start → workspace → generic work → capability/workcell → execute → approval → artifact/decision/outcome → durable restart → node failover → connector fixture action → audit → restart/history readable。

Reference E2E：install biolab-reference → domain objects/UI injected → conformance/E2E → disable/uninstall → Core healthy → historical provenance readable。

最终生成 `reports/goal5_final_report.md`，明确 `CORE COMPLETE = YES/NO`。
