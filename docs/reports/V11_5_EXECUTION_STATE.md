# Morn v11.5-R1 Execution State

> 本文件由 Agent 自主运行机制维护（Goal §30）。下一 Agent 必须从本文件 + `docs/canonical/CURRENT.md` + canonical 母版开始，不从聊天记忆猜测。
**最后更新：** Phase 0 完成后

## Current HEAD

`main` @（Phase 0 commit，见 Git log 最新一条 `docs:` 提交）；Phase 0 基线 HEAD 为 `2cd9fbd6fa57930eaa71616f170f958037190654`。

## Completed phases

- **Phase 0 — Reality Audit**（G0 = PASS）
  - `docs/reports/V11_5_REALITY_AUDIT.md`（真实仓库事实 + 成熟度分级）
  - `docs/reports/V11_5_BASELINE_TEST_REPORT.md`（先跑后记录，含初始失败与根因修复）
  - `docs/reports/V11_5_CODE_MAPPING_AND_MIGRATION_MATRIX.md`（对象级矩阵）
  - canonical 落位：`docs/canonical/`（母版 + GOAL + HANDOFF + README + `CURRENT.md`），旧文档未删除

## Current gate status

```text
G0  Reality Audit          PASS
G1  Semantic Mapping       （Phase 1 进行中）
G2-G12                     未开始
G12 Customer Gate          BLOCKED_EXTERNAL（无真实客户授权数据）
```

## Tests last run

- Rust: `cargo test --workspace` = **1317 passed, 0 failed**（1283 lib + 34 integration）；`cargo fmt --check` PASS；`cargo clippy --lib` PASS
- Web: vitest **6 passed (3 files)**；`tsc --noEmit` PASS；`npm run build` PASS
- e2e（chromium 1228）: **2 passed / 8 failed**（③–⑩ UI 结构漂移 → Phase 16 重写）
- `npm audit`: 11 vulnerabilities（记录，Phase 21 评估）

## Architecture decisions

1. v11.5 canonical 落位 `docs/canonical/`，`CURRENT.md` 为指针；12 个 `docs/*.md` 标记 LEGACY（仅追溯）。
2. 目录等价映射（不强制大迁移）：新语义层落在 `src/work/`；控制平面扩展 `src/core/storage/` + 新 controllers；Cordis adapter 落在 `src/runtime/cordis/`；providers 落在 `src/providers/`；Factory 落在 `src/domains/factory/`。
3. SQLite 保持为第一版 durable store（母版 §32 允许；不引入 Kafka/K8s/Temporal）。
4. Windows 本地构建需会话级 `INCLUDE`/`LIB`（MSVC 在 `D:\Code\Visual Studio\Visual Studio\VC\Tools\MSVC\14.50.35717` + SDK 10.0.26100.0）；verify 脚本需自动探测。
5. 旧资产映射：TaskRecord→Work 载体、approval→Authority 前身、capability registry→Capability 声明层、skill→Manifest 条目、checkpoint→controller checkpoint、cost→ValueAssessment、hash-chain→receipt 完整性。TF/RV/LC/BIOFERM 保持 LEGACY 不迁移。
6. AgentLoop 硬编码输出记录为 Reality Gap（agent loop 属 Harness 职责，Phase 4 处理 Harness 时说明）。

## Files changed（Phase 0）

- 修复：`src-tauri/Cargo.toml`（features org,sandbox）、`src/core/code_tool.rs`（跨平台拆分重构）、`src/core/code_tool_runner.rs`（新）、`src/core/code_tool_tests.rs`（新，外置）、`src/component/skill/self_evolution/scan.rs`（dirs::home_dir）
- Web 修复：`web/src/main.tsx`（LocaleProvider 根级别）、`web/src/i18n/index.tsx`（中文默认）、`web/src/App.tsx`（.send-btn）、`web/e2e/human-simulation.spec.ts`（选择器/断言对齐）、`web/src/test/setup.ts`（fetch 边界 stub）、`web/src/test/WelcomeGuide.test.tsx`（LocaleProvider）、`web/vitest.config.ts`（e2e exclude）
- 文档：`docs/canonical/`（4 文件 + CURRENT.md）、`docs/reports/`（3 份 Phase 0 报告 + 本文件）

## Open root causes

- e2e ③–⑩ 失败：e2e 预期与 v0.2.0 UI 结构漂移（Studio 8 tab、Store/Console 结构、消息流 DOM）→ Phase 16
- npm audit 11 vulnerabilities → Phase 21
- node 版本 ci.yml=20 vs release.yml=22 → Phase 21
- web lint 脚本无 eslint 配置 → Phase 21
- update.json 空 signature → Phase 22（不伪造）

## External blockers

| 项 | 缺什么 | 谁才能解除 | 解除后的验收步骤 |
|---|---|---|---|
| DSH Harness Provider 真实接入 | DeepSeek Harness 进程/接口访问 + API key | 用户提供 DSH 运行环境与凭据 | Phase 4：DSH contract smoke（start/resume/cancel/events）真跑 |
| Pi Harness Provider 真实接入 | Pi 访问 + 凭据 | 用户提供 | Phase 5：Pi contract + swap benchmark 真跑 |
| 真实客户数据 / 现场（G12） | 客户授权数据、现场安全/回滚/审批条件 | 客户 + 人类授权 | 母版 0.2 边界重新验证；read-only/shadow 计划 |
| Cordis upstream 可用性 | 需确认 upstream Cordis exact version/commit（公开仓库核验） | Phase 3 Spike 时核验（网络可达则自动） | exact pin + 许可证记录 |

## Next exact actions

1. Phase 1：新建 `src/work/` 语义模块（Work.spec/status、Condition、Situation、Observation、ObjectRef、AuthorityRequest/Decision、ExecutionBinding、ActionAttempt、Receipt、ObservedOutcome、AcceptanceDecision、ValueAssessment、Profile、SolutionPackage、Commitment、Principal/Actor/Role、Capability*）+ serde + validation tests（schema round-trip）。
2. Phase 1 G1：old record compatibility/mapping tests（TaskRecord→Work 载体映射）；确认无重复 canonical ledger；protocol types 不依赖 Cordis/DSH/Pi 实现类型。
3. 测试（cargo test/fmt/clippy）→ 取证 → 更新本文件 → commit → Phase 2。
