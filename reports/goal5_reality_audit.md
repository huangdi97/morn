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
