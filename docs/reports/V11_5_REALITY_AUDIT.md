# Morn v11.5-R1 Reality Audit

**日期：2026-09-25**
**Phase：0 / G0 Reality Audit**
**审计对象：** 仓库 `E:/AI/morn` @ HEAD `2cd9fbd6fa57930eaa71616f170f958037190654`（main）
**方法：** 只读审计（两个并行 explorer 全量盘点 + 人工核查），先证明现在是什么，未先改。
**Canonical：** `docs/canonical/Morn_v11.5-R1_协议驱动控制平面_Cordis参考运行时_CapabilitySupplyChain_统一全量母版_2026-09-24.md`

---

## 1. Git / Workspace Reality

| 项 | 值 |
|---|---|
| repo root | `E:/AI/morn` |
| branch | `main` |
| HEAD | `2cd9fbd` fix: 3 core fixes — real LLM dispatch, r2d2 pool, collaboration delegates |
| origin | `https://github.com/huangdi97/morn.git`（fetch/push） |
| 总 commit 数 | 184 |
| 审计起始 worktree | clean，仅 2 个 untracked Morn MD（GOAL + HANDOFF，用户粘贴件） |
| Cargo workspace | root `morn`（src/）+ member `src-tauri`（morn-desktop） |
| 语言/运行时 | Rust 2021（stable-msvc），web: TypeScript 5.3 / React 18.2 / Vite 5.4 / Node 20（CI） |
| lockfile | `Cargo.lock`（7827 行）、`web/package-lock.json` |
| migrations | `src/core/storage/mod.rs:101-381` 内嵌建表 + `:104-111` market_*→hub_* 迁移（代码内迁移，无独立 migration 目录） |
| CI | `.github/workflows/ci.yml`（build/test/fmt/clippy + windows tsc/tauri build）、`release.yml`；**web vitest/playwright 不在 CI** |
| verify 脚本 | **无**（无 Makefile / scripts/ / justfile） |
| docs canonical 指针 | 审计时**无**（现已在 `docs/canonical/CURRENT.md` 建立） |
| 实际位置 | Core: `src/core/`（59 子模块）· UI: `web/src/` · Desktop 壳: `src-tauri/`（~110 Tauri commands）· plugin/workflow: `src/core/plugin_manager/`, `src/core/workflow/` · policy: `src/core/security/`, `src/core/approval.rs` · DB: `src/core/storage/`（SQLite/r2d2）· org: `src/org/` |

---

## 2. 模块盘点（Physical）

~290 个 .rs 文件，14 个目录：

| 模块 | 行数 | 说明 |
|---|---|---|
| `src/core/` | 24,732 | 59 子模块：storage、task_engine、long_task_engine、agent_loop、orchestrator、supervisor、workflow、registry、capability、security、approval、delegation、scan_bind、skill_manifest、sop_template、repo_map、dual_llm、model_router、engine(dag)、pipeline、plugin_manager(30 plugins)、memory、console backend、cost 等 |
| `src/component/` | 6,206 | model/knowledge(vector=sled)/memory/persona/skill/builtins/tool(builtins) |
| `src/channel/` | 3,434 | 16 渠道 adapter（dingtalk/feishu/wecom/telegram/rest_api/smtp/…，多数 gated `channels-full`） |
| `src/computer/` | 3,294 | browser/fs/sys/app/desktop ops + perception（gated `computer`） |
| `src/bridge/` | 1,232 | a2a、a2a_discovery、chat_agent（真实 LLM dispatch）、local_model、sync |
| `src/console/` | 1,138 | backend/governance/security/cost（Rust 侧） |
| `src/studio/` | ~2,800 | StudioManager、editors、executor |
| `src/config/` | 794 | settings/channels/model/server |
| `src/org/`, `src/hub/`, `src/mcp/`, `src/sandbox/`, `src/protocol/`, `src/themes/` | — | org(team/user/permission/audit)、hub(gateway: mock+Stripe)、mcp、sandbox(wasm/wasmtime)、protocol(**仅 a2a**)、themes(纯静态资源，非 Rust 模块) |
| `src-tauri/` | ~5,000 | Tauri 2 壳，37 command 模块，~110 commands，**0 测试** |
| `web/src/` | ~19,000 | App(5 视图) + 30 console panels + studio 14 + api.ts(731) + i18n(702 keys) |

---

## 3. Canonical 概念成熟度（EXISTS_AND_TESTED / EXISTS_PARTIAL / DOC_ONLY / MISSING / LEGACY / EXTERNAL_BLOCKED / UNKNOWN_NEEDS_PROOF）

| Canonical 概念 | 现状 | 证据 | 成熟度 |
|---|---|---|---|
| Work（任务模型） | TaskRecord + TaskStatus(status 字符串/enum) | `core/storage/tasks/`, `core/long_task_engine.rs:9-15` | **EXISTS_PARTIAL**（无 spec/status 分离） |
| Work.spec / Work.status 分离 | 不存在 | grep `WorkSpec\|observedGeneration` = 0 | **MISSING** |
| Condition（语义 Gate） | 不存在 | grep = 0 | **MISSING** |
| Commitment | 不存在 | grep = 0 | **MISSING** |
| Principal / Actor / Role | TeamManager/UserManager/PermissionChecker/persona registry | `src/org/`, `src/component/persona/` | **EXISTS_PARTIAL**（Role 与 Agent 未解耦为 Workcell 语义） |
| Authority request/decision | approval gate（300s wait）、delegation | `core/approval.rs`(318), `core/delegation.rs`(240), `core/security/governance/approvals.rs` | **EXISTS_PARTIAL**（无 AuthorityRequest/AuthorityDecision 契约、无 time window/budget envelope） |
| Capability / Skill | capability registry + skill_manifest + skill builtins + registry/version | `core/capability/`(def 13, registry 178), `core/skill_manifest.rs`(288), `core/registry/`(manager 502, version 336) | **EXISTS_PARTIAL**（无 Manifest/Qualification/Release/SiteAdmission、无四态分离） |
| Harness / HarnessProvider | 不存在（dual_llm 是 model-level，不是 harness） | grep `HarnessProvider` = 0 | **MISSING** |
| Workcell | 不存在（orchestrator group 是多 Agent 团队，非 Work 动态组合） | grep `Workcell` = 0 | **MISSING** |
| ActionAttempt | 不存在 | grep = 0 | **MISSING** |
| Receipt | hash-chain audit log、verify_payment trait | `core/security/constitution.rs:104,154`(9 tests), `hub/gateway.rs:106`(mock+Stripe) | **EXISTS_PARTIAL**（非外部动作 receipt 语义） |
| Reconciliation | 不存在 | grep = 0 | **MISSING** |
| ObservedOutcome | 不存在 | grep = 0 | **MISSING** |
| Acceptance | 不存在（只有 approval） | grep `AcceptanceDecision` = 0 | **MISSING** |
| ValueAssessment | cost budget/report | `console/cost/`(budget 262, report 240) | **EXISTS_PARTIAL** |
| Situation / Observation / ObjectRef | 不存在 | grep = 0 | **MISSING** |
| Profile | security/profile.rs（33 行，薄） | `core/security/profile.rs` | **EXISTS_PARTIAL**（非 Guarantee Profile） |
| SolutionPackage | 不存在 | grep = 0 | **MISSING** |
| Blueprint | agent_templates、sop_template、workflow templates | `core/agent_templates.rs`(135), `core/sop_template.rs`(275) | **EXISTS_PARTIAL**（Agent-blueprint，非 WorkBlueprint 语义） |
| Six Records 账本 | 无显式六类账本；Task/Decision/sessions/costs 表存在 | `core/storage/mod.rs:101-381` | **EXISTS_PARTIAL**（分散表，无统一 ledger，也无第二套重复账本） |
| World Record（外部事实权威） | 不存在 | — | **MISSING** |
| TF / RV / LC01 / BIOFERM 历史接口 | 仓库代码/docs 零命中 | grep = 0 | **LEGACY**（仅存在于 2026-08-31 历史账本，从未落地本仓库） |
| CNC-17 / MES / CMMS / SCADA | 零命中 | grep = 0 | **MISSING**（Factory thin slice 全新） |
| persistence（SQLite + r2d2） | 全部表 + 迁移 + 默认路径；sled 仅 vector.rs | `core/storage/mod.rs:436-442` | **EXISTS_AND_TESTED**（storage tests 174+293+506 等） |
| outbox / lease / checkpoint | checkpoint 存在（agent_loop per-phase、long_task_engine、governance/checkpoints）；outbox/lease 无 | `core/checkpoint.rs`(212) | **EXISTS_PARTIAL** |
| crash/restart recovery | panic hook + crash.log（进程级），无 Work 级恢复测试 | `src-tauri/src/lib.rs:38` | **EXISTS_PARTIAL** |
| 事件 state | event_bus（进程内）、sync events | `core/event_bus.rs`(214) | **EXISTS_PARTIAL**（无 durable event history/projection） |
| protocol 模块 | 仅 a2a；service keys（morn:task-engine 等）；无版本化 | `src/protocol/mod.rs:1-2` | **EXISTS_PARTIAL** |
| LLM dispatch（真实） | ChatAgent::chat_async（reqwest，3 retries）+ LocalLlmClient | `bridge/chat_agent.rs:121-193` | **EXISTS_AND_TESTED** |
| Agent loop | AgentLoop Plan→Implement→Review，**输出为硬编码模板字符串，非真实 LLM 调用** | `core/agent_loop.rs:86-153` | **EXISTS_PARTIAL**（Reality Gap：声称 agent loop，实际未接 LLM） |
| task-engine（长任务） | LongTaskEngine + ChildProcess（re-spawn --execute-task） | `core/long_task_engine.rs`, `core/task_engine/child_process.rs:41` | **EXISTS_AND_TESTED**（23 tests） |
| Tauri commands | ~110 commands / 37 模块 | `src-tauri/src/lib.rs:360-467` | **EXISTS_AND_TESTED**（编译级；无 Tauri 集成测试） |
| UI v11.5 概念（Work/Conditions/Authority/Receipt/Acceptance/Capability/Harness/Workcell/Factory/profile） | **全部缺失**；仅 ExecutionEvent log ticker | explorer 2 全量正则核查 | **MISSING**（UI 为 v0.2.0 三工作台应用） |
| docs v11.5 概念 | 12 个 docs/*.md 全为 v0.2.0 时代 | explorer 2 | **LEGACY** + **DOC_ONLY** |
| Cordis | 零命中（未引入） | grep = 0 | **MISSING**（Phase 3 Spike 全新） |
| DSH / Pi Harness | 零命中 | grep = 0 | **MISSING** + **EXTERNAL_BLOCKED**（无真实凭据/接口访问） |
| 真实客户数据 / 现场 | 无 | 母版 0.2 边界 | **EXTERNAL_BLOCKED** |

---

## 4. 结论：架构现状一句话

当前仓库是一个 **v0.2.0 时代的桌面 AI 工作台**（Rust core + Tauri + React）：
有真实的 storage/channel/bridge/LLM dispatch/task-engine/plugin 系统与 1,511 个测试，
但 v11.5 的核心语义（Work.spec/status、Condition、ExecutionBinding、Attempt/Receipt/Reconciliation、
ObservedOutcome/Acceptance、Capability 四态、Harness/Workcell、Profile conformance、Factory thin slice）
**在代码、测试、UI、docs 中全部缺失或仅以不兼容形态存在（EXISTS_PARTIAL）**。

v11.5 工程的主体是**新建语义层 + 控制平面 + 供应链 + Factory 楔子**，
并将 EXISTS_PARTIAL 的旧资产（TaskRecord、approval、capability registry、checkpoint、cost、persona、org）
**显式映射迁移**进新架构，而不是推倒重写。

## 5. G0 Reality Gate 结论

- Reality Audit：完成（本文件）
- 基线测试：完成（`V11_5_BASELINE_TEST_REPORT.md`，真实先跑后记录）
- 代码映射：完成（`V11_5_CODE_MAPPING_AND_MIGRATION_MATRIX.md`）

**G0 = PASS**（证据齐全，允许进入 Phase 1）。
