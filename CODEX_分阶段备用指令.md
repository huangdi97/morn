# Codex 分阶段备用指令

> 只有当 Codex 会话中断、上下文重置或某阶段卡住时使用。正常情况下只用 `CODEX_MASTER_PROMPT.md`。

## 指令 A — 恢复上下文

读取 `AGENTS.md`、`GOAL.md`、`PLAN.md`、`ACCEPTANCE.md`、`STATUS.md`、`DECISIONS.md`、`BLOCKERS.md`、`KNOWN_FAILURES.md` 和 git diff。
不要重复已完成工作。
找出 `STATUS.md` 中第一个未完成 Milestone，从那里继续。
先运行相关测试确认当前基线，然后继续实现直到下一验收 Gate。

## 指令 B — 让 Codex 自己修到绿

不要给我解释“可能是什么原因”后停下。
请直接：
1. 复现当前失败；
2. 定位最小根因；
3. 补/保留回归测试；
4. 修改实现；
5. 重跑目标测试；
6. 重跑相关 suite；
7. 更新 STATUS/KNOWN_FAILURES；
8. 若没有真实外部 blocker，继续下一个未完成任务。

## 指令 C — UI 继续

根据 `UI_SPEC.md` 检查当前 Workbench/Studio/Console/Hub。
删除或替换“只为展示而存在”的硬编码 mock。
把 UI 连接到真实 application/domain API。
优先完成 BioLab E2E 在四个 surface 上的一致投影。
补齐 loading/empty/error/blocked/approval 状态，并运行 frontend build/test。

## 指令 D — DSH 集成继续

读取 `DSH_SPIKE.md` 和当前 provider 代码。
先运行 provider contract suite。
再尝试真实 DeepSeek Harness smoke。
外部依赖失败时记录完整 blocker，但不要让核心工程停下。
重点证明：
- scope 隔离；
- context provenance；
- tool→ActionGateway；
- event normalization；
- E0 cleanup；
- provider switch invariant。

## 指令 E — 最终验收

逐条对照 `ACCEPTANCE.md`。
每一条必须给出：
- 代码位置；
- 测试；
- 结果。
运行全量验证。
修复所有可本地修复失败。
生成 `reports/tonight_final_report.md`。
只有真实外部 blocker 可以保留未通过，并必须在报告中明确。
