# DECISIONS.md

> Codex 只能追加/修订有明确依据的 Decision。不要把猜测写成既定事实。
>
> **Current v11.5 note (2026-10-17):** D-001–D-030 retain the original
> greenfield/v1 decision history. Where a historical decision conflicts with
> the current branch, the later D-031+ decision explicitly supersedes it.
> Current implementation truth is also summarized in `STATUS.md`; external
> evidence blockers remain authoritative in `BLOCKERS.md`.

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
Status: Superseded by D-031 (historical greenfield decision)

除非现有仓库已有稳定依赖并有测试，否则不把 Cordis 设为 Semantic Kernel 硬依赖。

## D-005 — UI 共用一个后端
Status: Accepted

Workbench/Studio/Console/Hub 是同一 Domain/Application 层的四个产品表面。

## D-006 — Evolution 不直接改 Production
Status: Accepted

Promotion 生成新 production version，保留 rollback point。

## D-007 — Greenfield 技术栈选择：Rust workspace + Web 前端 + SQLite adapter
Status: Accepted as historical greenfield baseline; desktop-build clause superseded by D-033

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
Status: Superseded by D-032 (historical August environment observation)

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

## Goal 4 Decisions

### D-021 — Goal3 正式对象全部进入 canonical SQLite SoR（schema v2）
Status: Accepted

Context:
GOAL4_PERSISTENCE_SPEC：禁止 DB 影子副本 + in-memory 真相并存；one canonical persistence path。

Decision:
`morn_records` 增加 immutable 列（v1→v2 migration）；certification/managed/replacement/flywheel/distillation/
rollback/episode/prediction 全部经 MornStore 持久化；app 层 write-through persist_all + restart hydrate（稳定 workspace）。

Tests/Proof:
`morn-store` 10 tests（fresh v2、v1→v2 upgrade、immutable reject overwrite、capability restart hydration）；
`morn-app::persistence_goal4` 2 tests（restart hydration、receipt immutable）。

### D-022 — Rollback 只回滚 Morn 受控版本/绑定，E3 外部效果不宣称已回滚
Status: Accepted

Context:
GOAL4：Rollback Request→Approval→Compatibility→Activate→Verify→Receipt；不删除历史。

Decision:
`morn-assurance::RollbackService`：requester != approver；previous version 必须在已知 release 内；receipt 标记
e3_external_effects_preserved=true；request/receipt 永不删除。

Tests/Proof:
`morn-assurance::rollback` 4 tests（valid、unauthorized、incompatible、history preserved）。

### D-023 — Provider 提案只形成 candidate，必须过 schema/rule/policy/capability gate
Status: Accepted

Context:
GOAL4_REAL_PROVIDER_SPEC：provider proposal 永远不能直接 mutate production。

Decision:
`morn-evolution::EvolutionPlannerProvider` 只输出结构化 proposal；`ProposalValidator` 依次做 parse/rule/policy/
capability 校验后才生成 FlywheelCandidate。`morn-harness::run_harness_smoke` 作为 HarnessSmokeContract；
DSH Real 模式返回 blocker 报告，不 fake success。

Tests/Proof:
`morn-evolution::planner` 4 tests + `morn-harness::smoke` 3 tests。

### D-024 — 预测只做 evidence，不做 authority；prediction 先存、actual 后记、不可改写
Status: Accepted

Context:
GOAL4：Prediction→Actual→PredictionError→Calibration；禁止事后改写原 prediction。

Decision:
`morn-opint::PredictorRegistry`：predict() 先存 prediction（actual=None）；record_actual() 独立记录 actual 并计算
error（重复记录被拒）；calibrate() 用 (predicted, actual) 对计算 Brier；context mismatch → 低置信度不可自动决策；
数据不足 → insufficient-data 拒绝预测。

Tests/Proof:
`morn-opint::predictor` 7 tests；E2E goal4 验证 prediction-before-actual。

### D-025 — 真实 BioLab 数据 pilot：无合法真实数据则 FULL blocked，不捏造
Status: Accepted

Context:
GOAL4_REAL_PILOT_SPEC：真实数据优先用户提供 > 仓库已有 > 公开可下载；禁止 Codex 捏造“真实数据”。

Decision:
`morn-opint::RealPilotService` 提供 PilotManifest（source/license/checksum 必填，synthetic 拒绝注册）与 pipeline
contract；仓库无合法真实 dataset → run_pipeline 返回 External blocker（G4-B-002），CORE 继续完成。

Tests/Proof:
`morn-opint::pilot` 2 tests（synthetic 拒绝、无 manifest pipeline blocked）。
### D-026 — 领域中立后 demo smoke 走 bootstrap + 领域包 E2E，断言对齐当前架构
Status: Accepted (2026-08-16)

Context:
G5 领域中立后，BioLab 移入 domain-packs/biolab-reference（morn-app 的 `domain-biolab` feature 默认关闭）；
共享 app world 不再预置 3+ 实验室对象，`/api/biolab/run` 只在 all-features server 上注册。旧的
`scripts/run_all.ps1` demo smoke 用默认 feature 构建 server 导致 `/api/biolab/run` 404，且
`world_objects >= 3` 断言基于中立化前的预置模型。

Decision:
- demo smoke 的 server 构建改为 `--all-features`（与 UI smoke 一致），使领域包路由存在；
- demo flow：POST `/api/demo/bootstrap`（零领域通用种子：1 object + 1 work package + 1 artifact）
  再 POST `/api/biolab/run`（领域包 E2E，all_ok），然后验证 workbench world_objects >= 1、
  work_packages >= 1；
- 断言数量增加（health + bootstrap + biolab + workbench objects + work packages），不是削弱断言：
  阈值从 3 改为 1 是因为中立化后共享世界不再默认含 3+ 对象，bootstrap 契约就是 1；领域包对象在其自身
  WorldService 内（隔离，符合 G5 设计）。

Tests/Proof:
- 手工 smoke：health=ok, bootstrap objects=1/work_packages=1, biolab all_ok=True steps=7,
  workbench objects=1/work_packages=1；
- `scripts/run_all.ps1` 全绿 exit 0（208 Rust tests / 0 ignored；frontend 2；UI smoke 全部 OK；
  demo smoke OK）。

### D-027 — Core 编译器去领域启发式：governed-deliverable 模板由通用信号触发
Status: Accepted (2026-08-16)

Context:
M1 领域中立审计发现 `morn-foundry::compiler` 的 decompose_nodes 用
`goal.contains("claim") || goal.contains("hypothesis") || request.domain.contains("biolab")`
触发 evidence/analysis/review/approval/release 模板（CORE_BUG，Core 认识领域词）。

Decision:
改为领域无关触发：goal 含 "review"/"approv"，或约束声明 governed release
（"release" + "unapproved"/"irreversible"）时走 governed 模板；release 节点措辞改为
"release reviewed result"，验收改为 "outcome linked to evidence/artifact/decision"。
领域名单独不再改变分解（测试 `governed_template_is_domain_neutral` 证明）。

Tests/Proof:
`morn-foundry` 14 tests；`governed_template_is_domain_neutral` 断言 generic 域 reviewed goal=5 nodes、
domain="biolab" 的 transform goal=2 nodes。check_domain_boundary.ps1 新增领域词启发式守卫。

### D-028 — UI 领域注入用 backend domain_packs 广告，Core UI 不再硬编码 BioLab
Status: Accepted (2026-08-16)

Context:
M14 要求零 Domain 下 Workbench/Studio/Console/Hub 完整可用；此前 Workbench 无条件渲染
BioLab 按钮/卡片并直接调用 /biolab/*，Studio 默认 domain="biolab"。

Decision:
- `/api/workbench` 增加 `domain_packs` 字段：启用领域包时广告（如 ["biolab-reference"]），
  零 Domain 时为空数组；
- Workbench 通过 `biolabEnabled(domain_packs)` 数据驱动门控 BioLab 区块（extension point），
  Core UI 无 `if domain == biolab` 硬编码；
- Studio 默认 goal/domain 改为通用（"Deliver a reviewed report" / "generic"）；
- `/api/compiler/run` 默认 domain 改 "generic"。

Tests/Proof:
frontend `Workbench.gating.test.ts`（zero-domain=false / enabled=true）；ui_smoke 在 all-features
下仍通过（domain_packs 广告后按钮出现）；zero-domain server 实测 domain_packs=[] 且 /api/biolab/run 404。

### D-029 — Developer CLI 补齐 provider/connector/plugin/compat/migrate，全部走 canonical services
Status: Accepted (2026-08-16)

Context:
M15 验收要求 CLI 覆盖 doctor/status/node/provider/connector/plugin/domain/package/compat/conformance。

Decision:
`morn-cli` 新增 provider（IntelligenceProvider 双 fixture + harness smoke）、connector
（fixture 健康/幂等）、plugin（manifest validate）、compat（semantic contract matrix）、
migrate（schema version/preflight）。命令只调用 canonical services，不直改 DB。
run_all.ps1 新增 developer CLI smoke 步骤覆盖全部子命令。

Tests/Proof:
`morn doctor/status/provider/connector list|health/plugin validate/compat/migrate status/node list/domain list/package inspect/conformance` 全部 exit 0。

### D-030 — Pack/Plugin 名字在 Core 边界做 identifier-safe 校验（path traversal / command injection）
Status: Accepted (2026-08-16)

Context:
M17 安全矩阵要求 path traversal / command injection 边界；此前 `safe_name` 校验缺失，
migration_security 的对应测试只验证测试数据本身，未验证 Morn 实现。

Decision:
`morn-package::safe_name` 拒绝空名、`/`、`\`、`..`、控制字符；PackManifest::validate 与
PluginManifest::validate（新增，校验 plugin_type 白名单 + core_compat）统一调用；
CLI 传参用 argv（不 shell 插值）。安全测试改为断言 Morn 自身校验。

Tests/Proof:
`morn-package` 5 tests（含 unsafe_names_rejected_path_traversal_and_injection、
plugin_manifest_validate_type_and_name）；migration_security 10 tests。

## v11.5 Current Decisions — 2026-10-17

### D-031 — Cordis 是参考组合运行时，不是 Morn 语义内核
Status: Accepted

Context:
D-004 的“只做 Spike”属于早期 greenfield 决策。当前仓库已经将
`@deepseek-ai/cordis@4.0.4` 固定为 reference composition host，同时 v11.5 明确
Work/Authority/Receipt/Outcome/Acceptance 不属于 Cordis 生命周期状态。

Decision:
- Cordis 负责 node-local Context / Service slot / dependency / Fiber / plugin lifecycle。
- Cordis 可以影响未来的 provider/capability 可用性，但不得修改既有 Work generation、
  ExecutionBinding、外部效果真值或业务验收。
- Morn 协议/语义不得依赖某个 Cordis 插件实现细节；其他实现满足协议与 conformance 即可。

Proof:
`runtime/cordis-host`、CI Composition runtime job、`docs/architecture-v11.5.md`。

### D-032 — DSH Real adapter 本地实现完成；Live DSH 只剩独立外部证据
Status: Accepted

Context:
D-008 记录的是 2026-08 环境。官方 DeepSeek Harness 现有公开发行物；Morn 已有 real
SDK stdio/JSON-RPC adapter、ACP Work-bound lifecycle controller、live evidence workflows
和 runtime/environment/profile/route identity gates。

Decision:
- 当前经 Morn 明确 re-qualification 的官方 DSH distribution 是
  `0.2.1-rc.2`；SDK/ACP wire identity 与 distribution provenance 分开判断。
- 生产配置必须匹配 exact qualified version，并绑定 deployment-attested SHA-256 digest、
  profile configuration、route 和 execution environment。
- future DSH package 即使保持相同 wire version，也不得自动继承资格；必须重新审核并更新
  qualification constant/tests/workflow gate。
- `#[cfg(test)]` protocol fixture 可以使用 synthetic runtime identity，但该 bypass 不可序列化、
  不可由生产环境启用，也不能产生 live-runtime claim。
- B-001 的含义因此是“缺 authenticated official-runtime live evidence”，不再是“Real adapter 未实现”。

Proof:
`crates/morn-harness/src/dsh_sdk.rs`,
`crates/morn-harness/src/dsh_acp.rs`,
`.github/workflows/live-dsh-evidence.yml`,
exact-head Actions `39026643224` for implementation checkpoint `49023ac`.

### D-033 — Provider lifecycle 是 capability-negotiated；ExecutionReceipt 属于 Morn turn boundary
Status: Accepted

Context:
不同 harness transport 的生命周期并不一致。当前 DSH SDK 与 Pi RPC 都能完成真实 turn，
但不能诚实地映射为 provider-neutral per-session close。把 `terminate()` 当成所有 receipt 的
前置条件会迫使 adapter 伪造上游不存在的语义。

Decision:
- `start/send/inspect` 是基础执行边界；interrupt/resume/session-close 由
  `HarnessProviderFeatures` 显式协商，不支持就 fail-closed。
- 对 Work E0 execution，每次 turn 后由 Morn application boundary 使用 pinned
  RuntimeContext + normalized events + runtime identity + settlement classification 生成并持久化
  `ExecutionReceipt`。
- provider 真正支持 session close 时，`terminate()` 也可以返回 receipt；但 receipt 的业务语义
  不依赖 fake close。
- timeout、未知 settlement、tool/subagent policy escape 或无法证明外部结果时，receipt 保持
  `outcome-unknown` / 未终结并要求 reconciliation。
- Harness receipt 永远不是 `ObservedOutcome` 或 `AcceptanceDecision`。

Proof:
`crates/morn-app/src/api.rs::run_e0_harness_turn`,
`execution_receipt_v115` persistence tests,
`docs/architecture-v11.5.md` receipt taxonomy。

### D-034 — Terminal Work truth 只能由持久化、可复核的业务证据投影
Status: Accepted

Context:
v11.5 将 Accepted/Rejected 从“内存 controller 收到一个 AcceptanceDecision 就改 phase”
收紧为 durable trust chain。今天的 terminal-review hardening 还禁止 generic Work CAS 直接创建
Accepted/Rejected projection。

Decision:
- final Accept/Reject 必须属于 exact Work generation 和 pinned AcceptanceSpec。
- referenced outcomes 必须 source-grounded，并通过 deployment-attested provenance。
- reviewer identity、role、validity、exact one-shot review authorization 与 consumption 都必须可持久恢复并重验。
- generic Work CAS 不得凭 caller-supplied phase 创造 terminal business truth。
- Conditional/RequestMoreEvidence 保持非 terminal。
- CustomerValidated value 只能引用同 generation、真正 final-accepted 的 exact outcome。

Proof:
`morn-control-plane` durable acceptance/trust tests and 2026-10-17 terminal projection commits
`d4b7ee5`–`c73da10`。

### D-035 — Repository-local complete 与 Production/Customer ready 是两个独立结论
Status: Accepted

Context:
用户要求把本地可设计、可编码、可测试部分全部做完；同时 B-001/B-002/B-003/B-004、
G4-B-002、G12 和 ProductionWrite 依赖真实凭据、部署身份、真实数据或客户系统。

Decision:
- 当 exact-head CI、current-state docs、local trust/control/provider/UI/conformance gates 全绿且
  repository-local required-design audit 无缺口，可以声明 `ALL_LOCAL_GATES_PASS`。
- 不得由上述结论推导 `LIVE_DSH`、`LIVE_PI`、`PRODUCTION_READY`、`CUSTOMER_G12`
  或 ProductionWrite。
- PR #1 保持 draft，直到相应外部 evidence gate 真正运行并产生独立证据；fixture、截图、protocol fake
  和文档不能清除 external blocker。

Proof:
`STATUS.md`, `BLOCKERS.md`, protected live-evidence workflows and exact-head GitHub Actions.

