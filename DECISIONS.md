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

## Goal 2 Decisions

### D-010 — Solution Compiler 使用规则型参考 planner，LLM 仅作候选
Status: Accepted

Context:
GOAL2_COMPILER_SPEC 要求 compiler 真能编译、可解释、不虚构 capability。

Decision:
`morn-foundry::SolutionCompiler` 用规则/模板做 analyze→propose→validate→compile；每个 planner decision 记录
`CompilerDecisionSource`（source_facts / rules_or_templates / model_provider="rule-based (no LLM)" / assumptions /
confidence / alternatives / human_review_status）。缺失 capability 输出 CapabilityGap 并阻止 compile。
Compile 只在 ApprovedSolution 后生成 SolutionPackage，绝不自动 deploy production。

Tests/Proof:
`crates/morn-foundry` 13 tests（含 golden cases：deterministic→program、capability gap 阻止 compile、
incompatible harness validation fail、regulated approval gate、explainability）。

### D-011 — Durable v0.2 以本地 SQLite 为真实持久化，不用内存冒充
Status: Accepted

Context:
GOAL2 恢复指令 C：“内存状态不算 durable”。

Decision:
`morn-work::DurableRuntime` 支持 checkpoint→process restart→load→drift check→resume；`morn-store` 持久化
workflow definition/run/checkpoint/signal，并有 SQLite 重启恢复集成测试。

Tests/Proof:
`morn-store::durable_run_restart_resumes_via_sqlite`；`morn-work` durable 12 tests（signal idempotency、
retry exhausted、E2 compensation、budget stop、drift attention）。

### D-012 — Replay/Simulation/Shadow 使用隔离 state，绝不写 production
Status: Accepted

Context:
GOAL2_ARCHITECTURE_FREEZE：Replay/Simulation/Shadow 默认 read production snapshot、write isolated run state。

Decision:
`morn-assurance`：ReplayRunner/EvaluationRunner/ShadowRunner 只读 scenario/recorded events 并写入自身 report
集合；EvaluationRunner 注入 12 类故障；Shadow 只比较 evaluation 结果并输出 readiness，不执行外部动作。

Tests/Proof:
`crates/morn-assurance` 11 tests（replay reproduced/deviation、evaluation pass/fail/conditional/regression、
shadow ready/not-ready/conditional、replay 不触碰 production）。

### D-013 — BioLab 三闭环全部走后端真实 artifact/approval 路径
Status: Accepted

Context:
GOAL2_BIOLAB_DREAM_FACTORY：Loop A/B/C 必须真实可复现，湿实验保持 Human/Device gate。

Decision:
Loop A/B/C 通过 BioLabService + ArtifactService（immutable version + review + PI approval）实现；
Loop C 的一致性检查比较 figure/analysis 行数与 claim 证据链接（真实数据检查，非硬编码成功）。

Tests/Proof:
`crates/morn-biolab` 5 tests + `morn-app` E2E 全链路（Goal→Compile→Replay→Eval PASS→Shadow Ready→BioLab
三闭环→Outcome→Final EvaluationReport）。

### D-014 — UI v0.2 继续共用同一后端，不引入模拟数据
Status: Accepted

Context:
GOAL2_UI_SPEC：Workbench/Studio/Console/Hub v0.2 必须接真实 backend API。

Decision:
新增 `/api/compiler/*`、`/api/durable/*`、`/api/evaluation/run`、`/api/shadow/compare`、`/api/replay/run`、
`/api/biolab/loop-a|c|assets`、`/api/hub2`；四个页面全部调用这些真实端点。

Tests/Proof:
`frontend/scripts/ui_smoke.mjs`（Playwright）覆盖 4 surfaces + durable/replay/shadow/eval/loop/studio compiler；
`npm run typecheck/lint/test/build` 全过。

## Goal 3 Decisions

### D-015 — Certified Work Capability 不是 Agent，认证必须有完整证据
Status: Accepted

Context:
GOAL3_CERTIFICATION_SPEC：禁止一个 demo pass 就 Certified；禁止忽略失败 case；critical gap 下禁止认证。

Decision:
`morn-assurance::CertificationService` 要求证据（evaluation + replay + shadow）非空才能 start_run；
evaluate 检查 acceptance rate / policy / recovery / reproducibility / provenance / known-failure coverage；
human_gate_required 时 evaluate 只产生 Conditional，必须 approve 才 Certified；version 变化按
patch_compatible / requires_reevaluation / full_recertification 判断；Restricted capability 受 context-of-use 约束；
Suspended 不可启动 Managed Work。

Tests/Proof:
`crates/morn-assurance::certification` 5 tests（insufficient evidence、failed policy、human gate、
version rules、context-of-use）。

### D-016 — Evolution Flywheel 候选只读，不触碰 production
Status: Accepted

Context:
GOAL3_EVOLUTION_FLYWHEEL：Candidate → Branch → Replay → Evaluation → Shadow → Certification → Promotion；
不能直接修改 production。

Decision:
`morn-evolution::EvolutionFlywheel` 只 ingest trace/correction、检测 pattern、生成 data-only candidate
（FlywheelCandidate 无 mutation 方法、无 production 写路径）；candidate 携带 evidence window 与 baseline metrics。

Tests/Proof:
`crates/morn-evolution::flywheel` 4 tests（pattern detection、candidate links evidence、human correction、
no production mutation）。

### D-017 — Deterministic Distillation：稳定 Actor 步骤优先确定性能力
Status: Accepted

Context:
GOAL3 最小充分智能：至少完成一个 deterministic distillation 示例，保留长尾 fallback。

Decision:
`morn-evolution::DistillationService` 把 BioLab `qc` 稳定步骤蒸馏为 `qc-rule`（rows>0 && even），
回归比较 program vs actor（3/3 match），special/edge 输入 fallback 回 actor，记录 cost/latency/human 对比。

Tests/Proof:
`crates/morn-evolution::distillation` 3 tests（regression passed、actor fallback、quality 不降）。

### D-018 — Managed Work 只接受 Certified Capability；Acceptance 独立于执行者
Status: Accepted

Context:
GOAL3_MANAGED_WORK_SPEC：未认证 capability 不得进入 Managed Work；执行 Actor 不能自己判定 Outcome 通过。

Decision:
`morn-assurance::ManagedWorkService::start` 只接受 Certified/Restricted capability；`decide` 拒绝
decided_by == executor；DeliveryReceipt 必须完整；SLO/HumanFallback/RetryLiability 均有 schema 与测试。

Tests/Proof:
`crates/morn-assurance::managed_work` 4 tests（only certified can start、full lifecycle、
executor cannot self-accept、retry exhaustion escalates）。

### D-019 — Replacement 只做 R3/R4，不自动退役
Status: Accepted

Context:
GOAL3_REPLACEMENT_PILOT：Shadow 无真实副作用；R4 只生成 PartialReplaceCandidate + evidence + human decision。

Decision:
`morn-assurance::ReplacementPilot`：shadow_compare 隔离执行（isolated_side_effects=true）比较 quality/
acceptance/human/cost/evidence/policy/outcome；decide_r4 需要 candidate 关键指标不降 + eval passed +
certification passed + human approved，生成带 rollback path 的 R4 candidate；不自动 retirement。

Tests/Proof:
`crates/morn-assurance::replacement` 5 tests（same-input comparison、worse candidate 拒绝、
policy regression 拒绝、human approval 必需、R4+rollback）。

### D-020 — UI v0.3 继续共用同一后端
Status: Accepted

Context:
GOAL3_UI_SPEC：Evolution Center / Managed Work / Certification / Replacement Compare / Hub certified assets。

Decision:
新增 `/api/evolution/*`、`/api/distill/run`、`/api/certify/*`、`/api/managed/*`、`/api/replacement/*`、`/api/hub3`；
Workbench/Console/Hub 全部调用真实端点。

Tests/Proof:
Playwright UI smoke 覆盖 G3 交互；`npm run typecheck/lint/test/build` 全过。