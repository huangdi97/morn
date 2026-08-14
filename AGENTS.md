# AGENTS.md — Morn v10.2-R1 工程执行总规则

> Codex 在任何修改之前必须先读：
>
> 1. `docs/spec/Morn_v10.2-R1_全量设计母版.md`
> 2. `GOAL.md`
> 3. `PLAN.md`
> 4. `ACCEPTANCE.md`
> 5. `ARCHITECTURE_FREEZE.md`
> 6. `UI_SPEC.md`
> 7. `TEST_PLAN.md`
> 8. `DECISIONS.md`
> 9. `STATUS.md`
> 10. `BLOCKERS.md`
> 11. `KNOWN_FAILURES.md`

## 1. 工作模式

你不是在“生成一个 Demo”，而是在建立 Morn v10.2-R1 的可继续演进工程基线。

执行流程固定为：

```text
Inspect Repository
→ Establish Baseline
→ Freeze Contracts
→ Implement Small Vertical Slice
→ Test
→ Integrate
→ Update Status/Decisions
→ Next Slice
→ Full Verification
```

除真正不可由本地事实解决的外部阻塞外，不要停止在“建议下一步”。直接继续实现、测试、修复。

## 2. Source of Truth 优先级

1. 现有仓库中已经被测试证明的事实；
2. 本工程包 `ARCHITECTURE_FREEZE.md` / `ACCEPTANCE.md`；
3. `docs/spec/Morn_v10.2-R1_全量设计母版.md`；
4. 已记录且仍有效的 `DECISIONS.md`；
5. Codex 自己的推断。

如果仓库现状与母版矛盾：
- 不静默覆盖；
- 记录冲突；
- 优先迁移而不是大爆炸重写；
- 必须保持可编译、可测试、可回滚。

## 3. 不可破坏架构约束

### 3.1 Work-first
先有 Goal / WorkPackage / Acceptance / Outcome，再选择 Actor、Program、Human、Service 或 Device。

### 3.2 Canonical State
Actor/LLM/Harness/Runtime 只能产生 Proposal。
正式状态变化必须经过：

```text
Proposal
→ Schema Validation
→ Domain Validation
→ Policy
→ Approval / Simulation（按风险）
→ Action Gateway
→ Commit
→ Verify
→ Ledger / Outcome
```

### 3.3 Stable Semantic Kernel
以下语义由 Morn 拥有，Provider/插件不得重新定义：
- Identity / Principal / Workspace
- Operational World Object / State / Event / Action
- WorkPackage / WorkContract / AcceptanceSpec / OutcomeContract
- Artifact / Decision / Outcome
- Role / Delegation / Authority / Accountability
- Ledger / Provenance

### 3.4 Dynamic Capability Fabric
Model、Agent Loop、Tool、Memory Backend、Sandbox、Analytics、Simulation、Runtime 可替换，但必须通过稳定 Provider/Adapter 接口。

### 3.5 Harness / Runtime 中立
Actor 的身份、Role、Workspace、Artifact、Memory Ownership、Work 历史不属于 DSH/Codex/Claude/Hermes 等 Runtime。

### 3.6 Artifact 不可原地覆盖
修改生成新 Version，并保留 lineage / derived_from / supersedes。

### 3.7 Governed Effects
Effect Class：
- E0 lifecycle_reversible
- E1 transactional
- E2 compensatable
- E3 irreversible

Provider unmount 只负责 E0 lifecycle cleanup。
E2 必须有 compensation。
E3 默认 Preview + Approval + Verify。

### 3.8 Production / Evolution 分离
Evolution 必须：

```text
Branch
→ Candidate
→ Replay/Simulation
→ Evaluation
→ Shadow（高风险/必要时）
→ Promotion Decision
→ New Version
→ Rollback Point
```

不得原地修改 Production。

### 3.9 最小充分智能
如果 Rule / Program / Solver 能以更低成本、更高可靠性完成 Work，不要强行使用 LLM/Agent。

## 4. 代码边界

母版推荐领域结构：

```text
morn-kernel
morn-world
morn-artifact
morn-capability
morn-harness
morn-runtime
morn-actor
morn-organization
morn-work
morn-foundry
morn-evolution
morn-assurance
morn-domain
morn-package
```

现有仓库若已有更合理拆分，不为“目录完全一致”而重写；但语义边界必须等价。

依赖方向：

```text
Kernel / World / Work contracts
        ↑
Actor / Organization
        ↑
Harness / Runtime adapters
        ↑
Product surfaces
```

禁止 Adapter/Runtime 反向成为业务 Source of Truth。

## 5. 实现原则

- 优先小而完整的 vertical slice，不优先“大量 scaffold”。
- 一个模块尽量只承担一个清晰职责。
- 避免巨型文件、巨型 trait、万能 manager。
- Public API 小而稳定；内部实现可替换。
- Schema 与 migration 必须版本化。
- 核心 ID 使用强类型或至少有明确 newtype/domain type。
- 时间、状态、版本、审批、Outcome 不靠自由文本表示。
- 所有外部执行必须产生 ExecutionReceipt / Trace。
- 所有关键写操作必须可审计。
- 错误必须结构化，禁止大面积 `unwrap()` / 吞错误。
- 不为测试而暴露不合理 public API。

## 6. Rust / Tauri / 前端

- 继承仓库已有 Rust edition、Tauri 版本、JS/TS 包管理器、UI 技术栈。
- 如果是 greenfield：选择当前环境能稳定安装与编译的 Rust/Tauri 稳定版本；固定 lockfile。
- 前端只调用 application/service API，不直接绕过 domain rules 写 SQLite。
- Tauri command 是边界层，不堆业务逻辑。
- UI 必须有 loading / empty / error / blocked / approval 状态。
- Demo 数据必须通过同一 application/domain API 创建，不能只在前端硬编码伪造。

## 7. 数据与持久化

今晚优先 Local/Desktop：
- SQLite：Kernel/World/Work 元数据；
- 本地 Artifact content store；
- Secret 不明文落 SQLite；
- durable checkpoint/log；
- 本地 Harness host/sidecar 可作为可选进程。

Kernel 接口不得写死 SQLite，使未来 PostgreSQL 可替换。

## 8. 外部依赖与 DeepSeek Harness

先做 Architecture Spike，再冻结 Provider：
- 不 fork DSH 成 Morn；
- 不把 DSH session 当正式事实；
- WorkPackage/Artifact Context 注入必须保留 Morn provenance；
- Tool Call 必须能经过 Morn Action Gateway；
- Session Event 归一化为 Morn ExecutionEvent；
- Scope 隔离可测试；
- E0 mount/unmount cleanup 可测试；
- 切换 Provider 后 World/Work/Artifact 不变。

若真实 DSH 在当前环境无法安装/启动：
- 保留真实 Provider 边界代码；
- 用 fixture/contract test 验证 Morn 侧不变量；
- 将外部错误原文和复现步骤写 `BLOCKERS.md`；
- 禁止伪造“已完成真实 DSH 集成”。

Cordis 只做 spike，不成为 Semantic Kernel 硬依赖。

## 9. 测试纪律

每完成一个 vertical slice：
1. 运行该 slice 单测；
2. 运行相关集成测试；
3. 修复；
4. 再继续。

结束前运行 `scripts/run_all.ps1` 或仓库等价全量验证。

禁止：
- 删除失败测试；
- 将 assert 改弱以过测试；
- 标记大量 ignored；
- 因第三方依赖失败而跳过 Morn 自己的契约测试。

## 10. 文档纪律

每个阶段结束必须更新：
- `STATUS.md`
- `DECISIONS.md`（如有新架构决定）
- `BLOCKERS.md`（真实阻塞）
- `KNOWN_FAILURES.md`（已知但未阻塞的问题）
- `CHANGELOG.md`（如仓库存在）

所有“完成”必须能指向代码 + 测试 + 可复现命令。

## 11. 停止条件

只有以下情况允许停止并请求人类：
- 需要真实 Secret / Credential；
- 需要不可逆外部操作；
- 需要购买/授权/法律许可；
- 现有仓库存在无法自动决定的数据丢失风险；
- 互相矛盾的产品要求会导致不可逆大迁移。

普通编译错误、测试失败、依赖冲突、类型错误、UI bug 不属于“请求人类”的理由，继续修。
