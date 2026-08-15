# GOAL2_PRECONDITIONS.md — Goal 1 基线验证

Codex 开始 Goal 2 前必须执行。

## 1. Git / Workspace
记录：`git status`、branch、HEAD、未提交修改、workspace tree。禁止破坏性 reset。

## 2. Goal 1 抽样回归
至少验证：
- backend build/test；
- frontend build/typecheck；
- Workspace isolation；
- Artifact immutable version；
- E3 approval gate；
- Work AcceptanceSpec gate；
- Harness provider switch invariant；
- checkpoint/resume；
- Evolution promotion guard；
- BioLab Dataset→ReviewedClaim E2E。

## 3. 查重
搜索是否已存在：
`Delegation`、`RepresentationContract`、`DecisionPolicyAsset`、`SolutionCompiler`、`ProblemSpec`、`WorkGraph`、`ReplayRun`、`EvaluationRun`、`ShadowRun`。

已有能力不重复造，优先补齐和迁移。

## 4. Exit
- PASS → Goal 2 M1；
- REGRESSION → 先修 Goal 1，再进入 Goal 2。
