# DECISIONS.md

> Codex 只能追加/修订有明确依据的 Decision。不要把猜测写成既定事实。

## D-001 — 近期实现范围
Status: Accepted

今晚以 P0 全量 + P1 本地闭环核心 + UI + BioLab + Governed Evolution v0.1 为目标。
P2 不以 mock 冒充完成。

Reason:
母版明确区分近期工程与真实 Outcome 数据出现后的后期能力。

## D-002 — Runtime/Adapter 不拥有 Canonical State
Status: Accepted

所有 Runtime/Harness/CLI Adapter 只能产生 proposal/event/receipt，不直接成为 World/Work/Artifact Source of Truth。

## D-003 — DeepSeek Harness 是 Provider
Status: Accepted

不 fork DSH 成 Morn，不把 DSH Session 当正式事实。

## D-004 — Cordis 只做 Spike
Status: Accepted

除非现有仓库已有稳定依赖并有测试，否则不把 Cordis 设为 Semantic Kernel 硬依赖。

## D-005 — UI 共用一个后端
Status: Accepted

Workbench/Studio/Console/Hub 是同一 Domain/Application 层的四个产品表面。

## D-006 — Evolution 不直接改 Production
Status: Accepted

Promotion 生成新 production version，保留 rollback point。

## D-007 — Greenfield 技术栈选择：Rust workspace + Web 前端 + SQLite adapter
Status: Accepted

Context:
`E:\AI\morn新` 无 `.git`、无源码；环境缺 Rust。M0 安装 rustc/cargo 1.97.1 (MSVC) 成功；Node 22 / npm / pnpm 可用。

Decision:
- 按母版以 Rust workspace（13 crates）实现 Stable Semantic Kernel 与领域层。
- 持久化用本地 SQLite adapter（`morn-store`，rusqlite bundled），Kernel 接口不写死 SQLite。
- 四个产品表面用同一 axum HTTP 后端 + React/Vite 前端；前端只调 application API，不绕过 domain rules。
- Tauri desktop shell 本轮不构建（见 KF-002），不假装“已构建”。

Reason:
环境可稳定编译 Rust/前端；本轮目标是可运行、可测试、可演示基线。

Alternatives:
Python/FastAPI 后端（可用但偏离母版 Rust/Tauri 主线）；纯内存服务（违反可持久化验收）。

Consequences:
- 未来接 PostgreSQL 只需新增 store adapter。
- Tauri 壳已实现为边界层（无业务逻辑），复用同一 `morn-app` 后端。

Tests/Proof:
`scripts/run_all.ps1` exit 0；BioLab E2E smoke 通过。

## D-008 — DeepSeek Harness 真实集成：当前为外部/凭据 blocker
Status: Accepted

Context:
官方 DSH（agent harness, "Everything is a Plugin", Cordis 驱动, Developer Preview）在当前环境没有可安装的官方发行物；PyPI 上唯一同名包是第三方 OpenAI 兼容客户端，启动需要真实 DeepSeek API key（无凭据）。

Decision:
- 完成 `DeepSeekHarnessProvider` 边界（Fixture 模式通过同一 provider contract suite 作为第二条 provider path）。
- 真实 `start/send` 在 `Real` 模式返回 External 错误，禁止伪造“已完整集成”。
- 完整复现与错误写入 `BLOCKERS.md` B-001。

Reason:
DSH_SPIKE.md 明确“真实集成失败时记录 blocker，Morn contract tests 仍必须通过”。

Tests/Proof:
`crates/morn-harness/tests/contract_tests.rs`：2 provider paths 通过同一契约；`deepseek_harness_real_mode_reports_external_blocker` 断言 Real 模式报 external。

## D-009 — 新增 Codex 决策：UI 数据流必须来自同一真实后端
Status: Accepted

Context:
ACCEPTANCE A10 与 UI_SPEC 禁止四 surface 各自维护模拟数据。

Decision:
前端所有页面通过 `/api/*` 读取真实 record（world objects、work packages、artifacts、attention、outcomes、traces、evolution promotions、hub assets、BioLab E2E）；E2E demo 由后端 POST `/api/biolab/run` 触发，不在前端硬编码。

Tests/Proof:
`demo smoke` 在 `run_all.ps1` 中启动 server 并验证 health + E2E + workbench 对象数。
