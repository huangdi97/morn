# Morn Goal 5 — Codex MASTER 超长总指令
## Core Completion / v1.0 Release Candidate

Goal ID：`MORN-G5-CORE-COMPLETION-V1RC`

Goal 1、Goal 2、Goal 3、Goal 4 已经完成。现在你负责完成 **Morn 本体**，不是继续做行业 Demo。

## 你的最终交付

今晚连续完成：

```text
Domain Neutralization
Stable Semantic Kernel v1 Freeze
Capability Fabric
Harness / Runtime / Provider SDK
Connector / Integration SDK
Generic Process Intelligence
Morn Node
Distributed Durable Runtime
Deployment / Topology
Domain SDK v1
Domain Pack Lifecycle
Plugin / Extension System
Generic Workbench / Studio / Console / Hub
Developer CLI / SDK
Compatibility / Version / Migration
Security / Isolation / Audit Hardening
Reliability / Chaos
Conformance Framework
Core v1.0 Release Candidate E2E
```

同时把现有 BioLab 从 Core 中彻底抽离为 reference/conformance Domain Pack。BioLab 只能依赖 Morn，Morn 不能依赖 BioLab。

## 必读顺序
1. AGENTS.md
2. 最新 Morn 设计母版（若存在）
3. Goal1/2/3/4 final reports（存在的都读）
4. GOAL5.md
5. GOAL5_PLAN.md
6. GOAL5_ARCHITECTURE_FREEZE.md
7. GOAL5_DOMAIN_NEUTRALIZATION.md
8. GOAL5_PROVIDER_CONNECTOR_RUNTIME_SDK.md
9. GOAL5_DOMAIN_PLUGIN_SDK.md
10. GOAL5_UI_CLI_SPEC.md
11. GOAL5_SECURITY_CHAOS.md
12. GOAL5_CONFORMANCE.md
13. GOAL5_TEST_PLAN.md
14. GOAL5_ACCEPTANCE.md
15. STATUS_GOAL5.md
16. DECISIONS.md
17. BLOCKERS.md
18. KNOWN_FAILURES.md

然后读取真实代码、Cargo/frontend workspaces、Tauri、DB、migrations、API、tests、scripts、packages。不要只按文档猜实现。

## M0：真实审计
运行 git status/branch/log、workspace inventory、全量 Goal1–4 regression。识别 Goal5 各能力哪些已完成、哪些 partial、哪些 missing；不得创建平行 V2/V3。生成 `reports/goal5_reality_audit.md`。

## 核心红线

### 1. Domain-neutral
Core 不能认识 Dataset/ScientificClaim/Hypothesis/Experiment/ProductionOrder/Machine 等行业对象。搜索领域词但判断真实语义，不盲删 generic data infrastructure。

### 2. 依赖单向
Domain Pack → public Morn SDK；严禁 Core → Domain Pack。建立自动 architecture guard。

### 3. Work-first
Agent 不是 primary business object。

### 4. Model-neutral
LLM/ML/solver/model 只是 provider/capability；Morn 不以训练基础模型为目标。

### 5. Governed action
所有外部 write/不可逆 effect 都经过 Action Gateway、Policy、Approval、Verify、Receipt。

### 6. Production/Evolution separation
Evolution candidate 不能直接改 production。

### 7. Historical truth
pack uninstall/version rollback 不删除历史 provenance。

### 8. One canonical implementation
重构时使用 compatibility adapter → migrate callers → delete obsolete path，禁止长期保留两套事实源。

## M1–M2：Domain Neutralization / Reference Extraction
先做 Core→Domain dependency graph 和 repo-wide audit。任何 concrete domain 进入 core generic layer，抽象成 generic contract。现有 BioLab 有价值，不删除，迁移为 reference pack；它只能通过 Domain SDK/public services 接入。

必须证明：zero-domain Core build/test/start；reference pack install/enable 后领域对象/UI 才出现；disable/uninstall 后 Core 健康；历史 provenance 可读。

## M3：Kernel Freeze v1
不要重写稳定 kernel。识别并 version canonical contracts：Identity/Workspace、World Object/Event/Action、Artifact/Decision/Outcome、Work/WorkPackage/Contract、Actor/Role、Capability、Policy/Permission/Approval、EffectClass、Provenance、Lifecycle/Version。加 contract snapshots、compatibility、deprecation、migration rules。

## M4–M5：Capability / Provider SDK
消除 Agent/LLM 特权。Work requires Capability；Capability 可由 Rule/Program/Solver/LLM/Model/API/Human/Device/Hybrid 提供。HarnessProvider/RuntimeProvider/IntelligenceProvider 均为公开 protocol。至少两个不同 fixture/provider 跑同一 conformance。Provider 不拥有 Workspace truth、不 raw write canonical DB、不绕权限/Action Gateway、不泄密。

## M6：Connector SDK
只做通用 integration fabric，不今晚实现完整 SAP/MES/LIMS。完成 ConnectorSpec、External refs、Mapping、Sync Cursor、Health、Receipt。read path 和 governed write path 明确。做 generic fixture connector，覆盖 timeout/duplicate/rate-limit/idempotency。

## M7：Generic Process Intelligence
建立 ObservedEvent→Trace→Handoff/Wait/Loop/Rework→ObservedWork/ObservedWorkGraph。用 generic fixture 识别 repeated handoff、wait bottleneck、rework、duplicate approval、manual copy、loop。Core 不写 lab/factory heuristics。

## M8–M9：Morn Node / Distributed Runtime
建立 Node identity/registration/capability/resource/health/lease。至少用两个本地/simulated nodes 做真正集成测试：A claim→checkpoint→A dies→lease expires→B restore→duplicate event→dedupe→external effect 不重复→complete→audit failover。不能一个 worker 冒充分布式。可用 Temporal/Restate/DBOS adapter seam，但 Morn 自己 work/idempotency/audit semantics 必须完整。

## M10：Deployment/Topology
完成 Desktop、Single Server、Team+Workers、Private+Edge 的 schema/placement/resource/secret/storage/runtime/policy binding/upgrade-rollback strategy/dry-run validation。不要求真实 K8s。

## M11–M12：Domain SDK / Pack Lifecycle
Domain SDK 必须让任何新行业无需改 Core 即可定义 ontology/objects/relations/events/actions/artifacts/outcomes/roles/work templates/policies/capabilities/connector requirements/evaluations/UI extensions。

提供 hello-domain minimal fixture，命令/服务支持 init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff。Pack uninstall 不删历史 canonical records/provenance。

## M13：Plugin/Extension
统一 capability/harness/runtime/connector/domain/eval/sim/UI/CLI extensions。Manifest 含 id/version/type/core+sdk compatibility/deps/permissions/entrypoints/config/secrets/migrations/health。声明权限不等于自动授权，runtime policy authoritative。

## M14：Generic UI
零 Domain 下 Workbench/Studio/Console/Hub 必须完整可用。Domain UI 通过 extension point 注入；禁止 Core 大量 `if domain == biolab`。所有 must-pass UI 接真实 backend，禁止 static fake cards。

## M15：Developer CLI
整合现有 CLI，不重复造。至少覆盖 doctor/status/migrate/node/provider/connector/plugin/domain/package/compatibility/conformance/test core。命令调用 canonical services，不自行改 DB。

## M16：Compatibility / Migration
定义 Core API、semantic contracts、DB schema、Domain SDK、Provider/Plugin protocols、Package manifest versions 和 compatibility matrix。Migration 必须 preflight/snapshot/dry-run/apply/verify/failure handling/restore plan。禁止 silent destructive migration。

## M17：Security
执行 GOAL5_SECURITY_CHAOS.md：cross-workspace leakage、secret redaction、plugin/connector permission、node auth、E3 approval、duplicate irreversible action、path traversal、command injection boundary、safe error/audit。所有本地可修失败必须修完。

## M18：Chaos
必须做真实 failure injection 自动测试：process crash、DB busy、duplicate/reordered events、provider timeout/malformed result、connector timeout、node loss/stale lease、checkpoint mismatch、migration/plugin failure、duplicate action。结果只能 recover/block/escalate/compensate，不能 silent corrupt。

## M19：Conformance
完成 Provider/Runtime/Connector/Plugin/DomainPack/Architecture conformance kits。以后新增扩展不应修改 Core tests 才能证明兼容。

## M20：Final E2E

### Pure Core E2E
零 Domain：start→workspace→generic work→capability/workcell→execute→approval→artifact→decision→outcome→checkpoint→node failover→connector fixture governed action→receipt→audit→restart→history readable。

### Reference E2E
Core running→install biolab-reference→validate/enable→domain types/UI appear→reference conformance/E2E→disable→uninstall→Core healthy→historical provenance readable。

## 工作纪律
每个 milestone：inspect → contract tests → implementation/refactor → persistence/migration → API/UI → targeted tests → regression → STATUS/DECISIONS/BLOCKERS/KNOWN_FAILURES → next。不要停下来问是否继续。

普通编译错误、borrow/type/TS/lint/test/migration/UI/dependency/fixture/flaky test 全部自行解决。

只有以下才允许停止并请求真人：真实 Secret/账号、付费或许可证、不可逆真实外部操作、真实硬件、不可恢复数据风险、两个硬约束冲突且会导致不可逆 migration。即使某个 external blocker 存在，也记录后继续所有其他任务。

## 禁止伪完成
禁止 todo!/unimplemented!/ignored tests/删除失败测试/降低断言/hard-coded pass/static fake UI/fake two-node failover/fake connector/provider/secret smoke/把 BioLab 改名冒充 generic/隐藏 Core→BioLab import/plugin raw DB write/connector bypass Action Gateway/uninstall 删除历史。

## 代码质量
遵循现有风格，small modules/high cohesion/low coupling/typed errors/clear domain-application-infra boundaries/no god files/no copy-paste variants/reusable public SDK/deterministic tests。重构结束必须删除 obsolete path。

## 状态与验收
持续更新 STATUS_GOAL5.md、DECISIONS.md、BLOCKERS.md、KNOWN_FAILURES.md。最终逐条执行 GOAL5_ACCEPTANCE.md，为每项给出 code location/test/command/result。

最后运行全量 backend/frontend/Tauri(UI 环境允许)/UI smoke、Goal1–4 regression、architecture/domain/provider/runtime/connector/plugin conformance、security、chaos、Pure Core E2E、Reference Pack E2E。

生成 `reports/goal5_final_report.md`，包含 starting/final commit、架构 before/after、domain leakage 清理、SDK/Provider/Connector/Node/Distributed proof、Pack lifecycle、zero-domain UI、reference extraction、安全/chaos/conformance、所有命令结果、external blockers、deferred instance-layer items。

所有本地可完成项通过后必须明确写：

`CORE COMPLETE = YES`

最终 Definition of Done：删除/不安装 BioLab、Factory、Pharma 等全部实例，Morn 仍然是一个完整可运行系统；任何新行业都通过 Domain SDK/Pack/Connector/Provider 扩展，而不是修改 Stable Semantic Kernel。

现在立即开始。不要只回复计划。不要等待我逐步确认。持续执行 M0→M20，直到所有本地可完成验收项全部通过。
