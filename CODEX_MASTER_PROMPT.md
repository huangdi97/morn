# 给 Codex 的第一条完整指令

你现在负责把当前工作区推进为 **Morn v10.2-R1 可运行工程基线**。

不要先回复一堆计划后停下。请直接读取仓库、执行命令、修改代码、测试、修复，并持续推进，直到 `ACCEPTANCE.md` 的 Tonight Gate 全部满足，或者遇到只有真人才能解决的真实外部 blocker。

## 一、开始前必须读取

按顺序完整读取：

1. `AGENTS.md`
2. `docs/spec/Morn_v10.2-R1_全量设计母版.md`
3. `GOAL.md`
4. `ARCHITECTURE_FREEZE.md`
5. `PLAN.md`
6. `ACCEPTANCE.md`
7. `UI_SPEC.md`
8. `TEST_PLAN.md`
9. `DSH_SPIKE.md`
10. `EVOLUTION_V01.md`
11. `BIOLAB_V01.md`
12. `SCOPE_MATRIX.md`
13. `DECISIONS.md`
14. `STATUS.md`
15. `BLOCKERS.md`
16. `KNOWN_FAILURES.md`

如果仓库中已有旧版 AGENTS/设计/状态文件，也读取并比较；不要静默丢失已有约束。

## 二、先检查，不要盲目重构

立即执行并记录：
- `git status`
- 当前 branch
- 最近提交
- repository tree
- Cargo workspace / Tauri / frontend package manager
- DB/migrations
- 当前 tests
- 当前 build/lint/test

把结果写入 `STATUS.md` 和必要的 `KNOWN_FAILURES.md`。

如果当前仓库不是 greenfield，优先增量迁移；不要为了匹配文档目录名而大规模重写。

## 三、今晚的实现优先级

必须优先完成：

1. Stable Semantic Kernel
2. Operational World L0
3. Artifact / Decision / Outcome
4. WorkPackage / Acceptance / OutcomeContract
5. Actor / Role / Workcell minimum
6. HarnessSpec/Binding + Capability Seam + Effect Class
7. Action Gateway
8. Durable checkpoint/resume minimum
9. DeepSeek Harness architecture spike/provider boundary
10. Evolution Candidate/Branch/Evaluation/Promotion guard
11. BioLab minimal vertical slice
12. Workbench / Studio / Console / Hub
13. Full test/build/demo verification

不要先做 Predictive World、企业分布式 runtime、机器人、多行业完整 Dream Factory 等 P2。

## 四、实现策略

每个 milestone 都按：

```text
contract/test
→ minimal implementation
→ persistence/application wiring
→ integration test
→ UI wiring（适用时）
→ regression
→ update STATUS
```

不要批量创建空 module。

## 五、强制不变量

### Canonical state
Harness/Runtime/Actor 不能直接写正式 World 状态。

### Artifact
不能原地覆盖，必须新版本。

### Work acceptance
Agent 自己说“完成”不等于完成；必须 AcceptanceSpec。

### Effects
E0/E1/E2/E3 必须建模。
E3 未批准默认拒绝。

### Harness-neutral
切换 DSH/Native/CLI provider 后，Morn Identity/World/Work/Artifact 语义不变。

### Evolution
Candidate 无权直接修改 Production。
Promotion 必须经过 evaluation + policy/approval，并生成新版本。

### UI
四个产品表面必须使用同一真实后端，不准只用前端 mock 冒充完成。

## 六、DeepSeek Harness

先完成 `DSH_SPIKE.md` 中 8 项验证。

如果联网/依赖允许：
- 使用官方 DSH 安装与启动方式；
- 做最小真实 smoke；
- 所有 Tool/Action 必须有 Morn gateway seam；
- event 归一化。

如果外部 DSH 真实集成失败：
- 不要停止整个工程；
- 完成 Morn Provider contract、fixture/contract tests 和第二 provider；
- 把完整错误与复现写 `BLOCKERS.md`；
- final report 明确写“真实 DSH smoke blocked”，不得写“已完整集成”。

Cordis 同理：只做 spike，不因其 API 不稳阻塞核心工程。

## 七、UI

按 `UI_SPEC.md`。
先让 Workbench 可见完整闭环，再扩 Studio/Console/Hub。

首要 demo：
```text
BioLab Dataset
→ WorkPackage
→ Analysis Artifact
→ Review
→ PI Approval
→ governed Action
→ StateDiff
→ Reviewed Scientific Claim Outcome
```

这个 demo 要能在 UI 中跨 Workbench/Console/Hub 看见同一批真实后端记录。

## 八、测试

每一阶段运行相关测试。
最终必须运行 `scripts/run_all.ps1`（如脚本需要根据仓库事实修订，你负责修订）。

任何失败：
- 先定位；
- 修；
- 重跑；
- 不要通过删测试/ignore/弱化断言让它绿。

## 九、文档更新

持续更新：
- `STATUS.md`
- `DECISIONS.md`
- `BLOCKERS.md`
- `KNOWN_FAILURES.md`

最终生成：
`reports/tonight_final_report.md`

报告必须包含：
- 实际完成能力；
- 关键文件；
- migrations；
- 测试命令/结果；
- UI demo 路径；
- DSH spike 结果；
- Evolution guard 结果；
- 未完成项及原因；
- 下一步 P1/P2。

## 十、工作终止规则

普通错误不要问我，继续修。

只有以下情况才停下来请求真人：
- 需要真实 secret/账号；
- 需要不可逆外部操作；
- 需要付费/许可证；
- 存在不可自动决策的数据丢失风险；
- 两个强约束冲突且任何选择都会造成不可逆大迁移。

除此之外，持续执行到验收完成。

现在开始：先读取文件和仓库，执行 M0，然后继续 M1、M2……不要停在“我建议下一步”。
