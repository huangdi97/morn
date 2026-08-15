# Goal 5 Reality Audit

Date: 2026-08-15
Baseline commit: 088cf91 (Goal 4 closeout)

## Environment
- branch: master；worktree clean（仅 GOAL5 文档未跟踪/修改）
- Rust: 1.97.1 (MSVC)；Node 22；npm 11；Playwright chromium 已装；Tauri v2

## Regression（M0）
```text
cargo test --workspace  ->  green（Goal1-4 全量，~142 tests，0 failed/0 ignored）
```
（Goal 4 结束时 run_all.ps1 exit 0 已含 backend/frontend/Tauri/UI smoke；M0 复验 cargo test 无失败）

## Workspace Inventory
- crates (16)：kernel, world, artifact, capability, harness, runtime, actor, organization, work,
  evolution, foundry, assurance, biolab, store, app, opint
- src-tauri (Tauri v2 shell)、frontend (React/Vite/TS)、scripts/run_all.ps1
- DB: SQLite schema v2（morn_records immutable + ledger_entries）

## Goal5 Capability Readiness
| Capability | Status | Evidence |
|---|---|---|
| Goal1-4 regression | DONE | run_all exit 0 |
| Domain Neutralization | MISSING | Core→biolab dependency（morn-app → morn-biolab）；kernel 含 biolab 领域 ID |
| Reference Extraction | MISSING | biolab 在 crates/ 内，非独立 pack |
| Stable Kernel Freeze | PARTIAL | kernel 已稳定但无 v1 contract snapshot/compat/deprecation |
| Capability Fabric | PARTIAL | capability/effect 已存在；无统一 invoke/health/fallback registry |
| Provider SDK | PARTIAL | HarnessProvider 存在；无 RuntimeProvider/IntelligenceProvider public SDK |
| Connector SDK | MISSING | 无 generic connector contract |
| Generic Process Intelligence | MISSING | opint 有 episode；无 observed-process 检测 |
| Morn Node | MISSING | 无 node identity/lease |
| Distributed Durable Runtime | PARTIAL | DurableRuntime checkpoint/resume；无 2-node failover/dedupe |
| Deployment/Topology | MISSING | 无 |
| Domain SDK | MISSING | 无 DomainDefinition |
| Domain Pack Lifecycle | MISSING | 无 install/enable/disable/uninstall |
| Plugin System | MISSING | 无 |
| Generic zero-domain UI | PARTIAL | 前端基本 generic 但含 biolab 卡片 |
| Developer CLI | MISSING | 无 morn CLI |
| Compatibility/Migration | PARTIAL | schema v2 migration；无 compat matrix/preflight |
| Security/Chaos/Conformance | MISSING | 无专项测试 |

## External blockers（沿用）
- B-001：真实 DeepSeek Harness smoke（凭据/官方安装物）
- G4-B-002：真实 BioLab 数据 pilot FULL（无合法真实 dataset）
（Goal5 本地可完成项不依赖二者）


---

## Post-completion Audit (2026-08-16)

Baseline: HEAD 5f53ba2 (G5 M3-M15) + working tree. Full regression:
`scripts/run_all.ps1` exit 0 — Rust 64 suites / 218 passed / 0 failed / 0 ignored;
frontend typecheck/lint/test(4)/build; CLI smoke 12 commands; Tauri build; Playwright
UI smoke all OK; demo smoke OK.

| Capability | Status (2026-08-16) | Evidence |
|---|---|---|
| Goal1-4 regression | DONE | run_all exit 0 |
| Domain Neutralization | DONE | compiler heuristic removed; domain boundary guard + ArchitectureConformance test; zero-domain server domain_packs=[] |
| Reference Extraction | DONE | domain-packs/biolab-reference; reference_e2e install/enable/disable/uninstall + history |
| Stable Kernel Freeze | DONE | kernel contracts.rs (22 contracts, v1 snapshot, compat, deprecation) |
| Capability Fabric | DONE | morn-capability + RuntimeProvider/IntelligenceProvider/HarnessProvider |
| Provider SDK | DONE | harness contract tests (2 impls); intelligence conformance (2 fixtures); runtime conformance |
| Connector SDK | DONE | morn-integration ConnectorSpec/Instance/Mapping/Cursor/Receipt + GenericFixtureConnector |
| Generic Process Intelligence | DONE | morn-process ProcessMiner (handoff/wait/rework/loop/dup-approval/manual-copy) |
| Morn Node | DONE | morn-node MornNode/NodeType/lease/health + 2-node failover tests |
| Distributed Durable Runtime | DONE | DistributedRuntime claim/checkpoint/failover/dedupe/stale-lease; pure_core_e2e Phase 3 |
| Deployment/Topology | DONE | DeploymentSpec/Topology/NodeGroup/PlacementRule + validation test |
| Domain SDK | DONE | DomainDefinition/DomainRegistry, 13 declaration kinds |
| Domain Pack Lifecycle | DONE | PackLifecycle init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff |
| Plugin System | DONE | PluginManifest (deps/permissions/compat/migrations/health) + validate |
| Generic zero-domain UI | DONE | Workbench gating (biolabEnabled), Studio generic, domain_packs API |
| Developer CLI | DONE | doctor/status/node/provider/connector/plugin/domain/package/compat/migrate/conformance |
| Compatibility/Migration | DONE | compat matrix + migration preflight/snapshot/dry-run/apply/verify/restore |
| Security | DONE | workspace isolation/secret redaction/E3 approval/connector token/node auth/pack-name validation/audit |
| Chaos | DONE | chaos.rs 7 failure-injection tests, explicit recover/block/escalate/compensate |
| Conformance | DONE | Provider/Runtime/Connector/Plugin/DomainPack/Architecture kits |
| Pure Core E2E | DONE | pure_core_e2e (zero-domain full chain + restart + failover + governed action) |
| Reference Pack E2E | DONE | reference_e2e (install/enable/conformance/disable/uninstall/history) |

External blockers unchanged: B-001（真实 DSH smoke）、G4-B-002（真实 BioLab 数据 pilot）。
