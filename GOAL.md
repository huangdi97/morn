# GOAL.md — 今晚目标

## Goal ID

`MORN-V10.2-TONIGHT-BASELINE`

## 北极星

把 Morn 从“完整设计母版”推进到一个 **可运行、可测试、可演示、可继续迭代的 Morn v0.1 工程基线**。

## Tonight Definition of Done

### G0 — Repository Baseline
- 仓库能在本机完成依赖安装；
- 后端/前端基线测试可运行；
- 有一键验证脚本；
- 旧功能不得因本轮重构无说明地消失。

### G1 — Stable Semantic Kernel
实现并测试：
- Principal / Identity
- Workspace
- Policy / Approval
- Ledger
- lifecycle metadata

### G2 — Operational World L0
实现并测试：
- ObjectType / Object
- Relation
- Event
- StateSnapshot
- ActionType / ActionProposal
- Goal / Metric
- StateDiff / Outcome

### G3 — Artifact / Decision / Outcome
实现并测试：
- Artifact + immutable versions
- review / approval
- provenance / lineage
- DecisionPackage
- VerificationReport
- OutcomeRecord

### G4 — Work Contract
实现并测试：
- WorkPackage
- WorkContract
- AcceptanceSpec
- OutcomeContract
- ExecutionMode
- RoleSlot / MemberBinding
- minimal Workcell

### G5 — Actor / Harness / Runtime Separation
实现并测试：
- ActorTemplate / ActorInstance
- ActorOrigin
- HarnessSpec / Version / Binding
- Runtime binding
- CapabilityDefinition / Provider / Consumer
- Scope
- ExecutionEvent / ExecutionReceipt

### G6 — Controlled Action
实现并测试：
- Preview
- Authorize
- Execute
- Verify
- Recover
- E0/E1/E2/E3 effect policy
- E3 human gate 默认拒绝未批准执行

### G7 — Durable Minimum
实现并测试：
- checkpoint
- pause/resume
- restart/load
- failure record
- attention item

### G8 — DSH/Cordis Spike
产出：
- `DeepSeekHarnessProvider` 边界；
- provider lifecycle / scope / event normalization contract test；
- DSH 可真实运行则至少一个 smoke；
- 第二 Provider：MornNativeHarness 或 CLI adapter；
- Harness 切换不改变 Morn canonical records；
- Cordis 只形成 spike 结论，不侵入 Kernel。

### G9 — Evolution Engine v0.1
实现并测试：
- EvolutionCandidate
- EvolutionBranch
- Evaluation result
- PromotionDecision
- rollback metadata
- production guard：candidate 无权直接改 production

### G10 — BioLab v0.1
实现：
- 最小 Domain Schema；
- 一个完整样例：
  `Dataset → QC/Analysis Artifact → Review → Approval → Action/StateDiff → Reviewed Claim Outcome`
- 湿实验保持 human/device placeholder，不假装自动化。

### G11 — Product Surfaces
四个产品表面共用同一后端：
- Workbench
- Studio
- Console
- Hub

至少可真实浏览/创建/审批/查看：
- Mission / World
- WorkPackage
- Workcell
- Artifact / Decision
- Approval / Attention
- Outcome
- Harness status
- Evolution Candidate
- Domain asset

### G12 — Final Verification
- formatting / lint
- unit tests
- integration tests
- contract tests
- persistence/recovery test
- UI build / smoke
- demo path
- docs/status 同步

## 不在 Tonight Done 内

以下不是今晚“必须真实实现”的功能：
- Operational World L2/L3 预测、因果、反事实；
- 企业级 PostgreSQL/event bus/IAM/distributed durable runtime；
- 真实机器人/仪器控制；
- 多行业 Dream Factory 完整产品；
- 自动 Role/Software/Organization 重构；
- 全自动商业 Work-as-a-Service；
- 完整 process mining / BI / digital twin engine；
- 用 Morn 重写 ERP/MES/APS。

这些只能有稳定接口、Schema、adapter seam 或明确 future issue，禁止 mock 后宣称完成。
