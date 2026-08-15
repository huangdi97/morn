# STATUS_GOAL5.md

Goal: MORN-G5-CORE-COMPLETION-V1RC
Status: COMPLETE — M0..M20 done; Goal1-4 regression green; CORE COMPLETE = YES
CORE COMPLETE: YES (final report: reports/goal5_final_report.md)

## Baseline
- branch: master
- starting commit: 088cf91 (Goal 4 closeout; reality audit 2026-08-15)
- Goal1-4: green (run_all exit 0 at G4 closeout; re-verified 2026-08-16)
- working tree: clean (committed)

## Milestones (evidence = code + test + command)

- [x] M0 Reality Audit — reports/goal5_reality_audit.md (2026-08-15) + this status; full regression green
- [x] M1 Domain Neutralization — compiler heuristic removed (crates/morn-foundry/src/compiler.rs: governed-deliverable template, no domain words); kernel domain IDs removed; reports/domain_neutralization_audit.md
- [x] M2 Reference Extraction — BioLab moved to domain-packs/biolab-reference (crate morn-biolab-reference); Core zero-domain build/start/work proven
- [x] M3 Kernel Freeze v1 — crates/morn-kernel/src/contracts.rs (22 canonical contracts, SEMANTIC_CONTRACT_V1, ContractSnapshot, ContractCompatibility, DeprecationPolicy) + tests
- [x] M4 Capability Fabric — morn-capability (effect/execution modes); no AI-only assumptions (Rule/Solver/Program/Human all supported)
- [x] M5 Provider SDK — HarnessProvider (morn-native + DeepSeek fixture), IntelligenceProvider (RuleIntelligence + SolverIntelligence), RuntimeProvider (morn-runtime) — all with conformance kits
- [x] M6 Connector SDK — morn-integration: ConnectorSpec/Instance/MappingSpec/SyncCursor/ExternalActionRequest/ConnectorReceipt/ApprovedActionToken/GenericFixtureConnector/ConnectorRegistry; governed writes via token
- [x] M7 Generic Process Intelligence — morn-process: ObservedEvent/Trace/ObservedWorkGraph/ProcessMiner detects repeated handoff/wait bottleneck/rework/loop/duplicate approval/manual copy; no domain ontology
- [x] M8 Morn Node — morn-node: MornNode/NodeType/NodeId/registration/health/lease; 2-node failover tests
- [x] M9 Distributed Durable Runtime — DistributedRuntime claim/heartbeat/checkpoint/failover/dedupe/stale-lease recovery; DurableRuntime checkpoint/restore/signal; real local 2-node E2E (morn-node tests + pure_core_e2e Phase 3)
- [x] M10 Deployment/Topology — DeploymentSpec/Topology/NodeGroup/PlacementRule + validation (valid + incompatible-placement-rejected tests)
- [x] M11 Domain SDK — morn-domain-sdk: DomainDefinition/DomainDeclaration/DomainRegistry; 13 declaration kinds incl. connector_requirement/evaluation/ui_extension
- [x] M12 Domain Pack Lifecycle — morn-package: PackLifecycle init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff; history preserved after uninstall (reference_e2e + package tests)
- [x] M13 Plugin System — PluginManifest (deps/permissions/compat/migrations/health) + validate (type/name/path-safety); Core policy cannot be bypassed (architecture guard + connector token + gateway)
- [x] M14 Generic Product Surfaces — zero-domain Workbench/Studio/Console/Hub; BioLab UI gated on backend domain_packs advertisement (Workbench.tsx biolabEnabled); Studio generic defaults; unit test + UI smoke
- [x] M15 Developer CLI — morn doctor/status/node/provider/connector/plugin/domain/package/compat/migrate/conformance (crates/morn-cli); calls canonical services; CLI smoke in run_all
- [x] M16 Compatibility/Migration — kernel contracts compat matrix; migration plan preflight/snapshot/dry-run/apply/verify/restore; downgrade rejected; fresh+upgrade idempotent (migration_security.rs)
- [x] M17 Security — cross-workspace isolation, secret redaction, E3 approval, connector write requires gateway token, node identity/lease, pack-name path-traversal/command-injection validation (safe_name), audit (ledger)
- [x] M18 Reliability/Chaos — chaos.rs: provider timeout/malformed, connector timeout/duplicate, node loss/stale lease failover, duplicate signal, checkpoint mismatch, migration failure, plugin init failure; all explicit recover/block/escalate/compensate
- [x] M19 Conformance — conformance.rs: Provider (2 fixtures) / Runtime (DurableRuntime + RuntimeProvider fixture) / Connector / Plugin / DomainPack / Architecture kits
- [x] M20 Core v1.0 RC E2E — pure_core_e2e (zero-domain full chain + restart + node failover + governed connector action + pack lifecycle) + reference_e2e (install/enable/disable/uninstall + history readable)

## Final Verification (2026-08-16)
```text
scripts/run_all.ps1 -> exit 0（=== Verification complete ===）
Rust: 64 suites, 218 passed, 0 failed, 0 ignored（含 pure_core_e2e / reference_e2e / e2e_goal2/3/4）
core-tests: chaos 7 / conformance 6 / migration_security 10 / pure_core_e2e 1 / reference_e2e 1
frontend: typecheck/lint/test(4)/build pass
CLI smoke: doctor/status/provider/connector/plugin/compat/migrate/node/domain/package/conformance OK
tauri desktop build: pass
UI smoke (playwright): 4 surfaces + BioLab E2E + Goal2/3 interactions + studio compiler OK
demo smoke: health=ok bootstrap_objects=1 e2e_steps=7 objects=1
zero-domain proof: server built without domain-biolab -> /api/workbench domain_packs=[] , /api/biolab/run 404 (absent)
all-features proof: domain_packs=[biolab-reference], BioLab E2E all_ok=True
```

## External Blockers
- B-001（真实 DeepSeek Harness smoke，需官方安装物或真实凭据）—— Active，Goal5 本地项不依赖
- G4-B-002（真实 BioLab 数据 pilot，需合法真实 dataset）—— Active，Goal5 本地项不依赖
