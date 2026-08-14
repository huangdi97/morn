# PLAN.md — Codex 连续执行顺序

> 原则：先让最小 vertical slice 贯通，再横向扩模块。每个 Milestone 都必须有测试和状态更新。

## M0 — Inspect & Baseline

1. 读取全部工程文件。
2. `git status`、当前分支、最近提交、现有目录、构建系统、测试框架。
3. 找出现有：
   - Rust workspace / Tauri app
   - domain model
   - persistence
   - frontend
   - runtime/agent adapters
   - tests
4. 运行当前 baseline。
5. 记录失败到 `KNOWN_FAILURES.md`。
6. 不要先重写。

Exit:
- 已知道现有工程事实；
- baseline 命令被写入 `STATUS.md`。

## M1 — Freeze Contracts

建立/整理：
- strong IDs / version types
- shared errors
- timestamps/status enums
- core domain contracts

先写 tests，再实现最小 API。

Exit:
- domain contracts compile；
- 不依赖 DSH/Tauri/UI。

## M2 — Kernel + SQLite

实现：
- Identity / Workspace
- Policy / Approval
- Ledger
- repository ports
- SQLite adapters + migrations

测试：
- workspace isolation
- approval persistence
- append-only ledger semantics

## M3 — World + Controlled Action

实现：
- Object/Relation/Event/StateSnapshot
- ActionProposal
- ActionGateway
- E0-E3
- StateDiff
- Outcome

关键测试：
- Runtime 不能直接 commit world state
- E3 未批准必须拒绝
- E2 要求 compensation contract
- state update 生成 ledger + receipt

## M4 — Artifact / Decision

实现：
- Artifact immutable version
- Review / Approval
- DecisionPackage
- Provenance
- VerificationReport

关键测试：
- old version 不被覆盖
- 下游只能读取 Approved（在需要 approved 的 work path）
- lineage 可查询

## M5 — Work / Organization Minimum

实现：
- WorkPackage
- AcceptanceSpec
- OutcomeContract
- ExecutionMode
- RoleSlot
- MemberBinding
- Workcell
- ActorTemplate/Instance/Origin

关键测试：
- WorkPackage 没有 acceptance 不得进入 certified/complete
- deterministic work 可绑定 program
- actor 不等于 role

## M6 — Harness Fabric

实现：
- HarnessSpec/Version/Binding
- RuntimeBinding
- CapabilitySeam
- Scope tree
- ExecutionEventNormalizer
- ExecutionReceipt
- MornNativeHarness minimal fallback

然后做 DSH spike。

关键测试：
- provider switch invariant
- provider mount/unmount E0 cleanup
- session event 不成为 canonical source
- tool/action proposal 走 ActionGateway

## M7 — Durable Minimum

实现：
- checkpoint
- pause/resume
- recovery record
- attention queue
- persisted work run state

关键测试：
- 运行中断后重新实例化 service 可 resume
- tool failure 可形成 attention/recovery

## M8 — Evolution v0.1

实现：
- candidate
- branch
- evaluation
- promotion decision
- rollback point

暂不实现自动组织重构。

关键测试：
- candidate 不得直接 mutate production
- promotion 创建新版本而非 overwrite
- failed evaluation 无法 promote

## M9 — BioLab Vertical Slice

Domain minimum:
- Dataset
- AnalysisRun
- QCResult
- ScientificClaim
- Review
- Approval

跑通：
```text
Dataset
→ WorkPackage
→ Analysis Artifact
→ Reviewer
→ Human Approval
→ governed Action
→ StateDiff
→ Reviewed Scientific Claim Outcome
```

允许用小型 fixture 数据，不允许把分析结果硬编码在 UI。

## M10 — Product Surfaces

实现共享导航和四个表面：

### Workbench
- mission summary
- world objects
- work packages
- workcell live state
- approvals
- artifact/decision
- outcomes
- attention
- harness runtime health
- evolution candidates

### Studio
今晚只把已有后端能力做成可用 Builder：
- Actor
- Role/Harness
- WorkPackage
- Workcell
- Domain/World
- Evaluation
- Solution manifest preview

### Console
- Identity/Registry
- World state
- Work/Commitment
- Harness health
- Approval/Attention
- Policy
- Trace/errors
- Outcome/cost placeholder based on real records
- Version/rollback

### Hub
- DomainPack
- ActorTemplate
- HarnessTemplate
- WorkPackageTemplate
- WorkcellBlueprint
- EvaluationPack
- Certified capability metadata

不要先做 marketplace 商业功能。

## M11 — End-to-End Verification

顺序：
1. fmt
2. lint
3. unit
4. integration
5. persistence/recovery
6. harness contract
7. evolution guard
8. frontend typecheck/lint/test
9. app build
10. demo smoke

失败就修；不要绕过。

## M12 — Closeout

更新：
- `STATUS.md`
- `DECISIONS.md`
- `BLOCKERS.md`
- `KNOWN_FAILURES.md`
- changelog
- 生成 `reports/tonight_final_report.md`

报告必须包含：
- 完成了什么；
- 代码位置；
- 测试命令和结果；
- 未完成什么；
- 哪些是外部 blocker；
- 下一步最短路径。
