# GOAL3_REPLACEMENT_PILOT.md

## 默认试点
`BioLab: Dataset → Reviewed Scientific Claim`

如果仓库已有更成熟、可量化的具体 Work，可替换默认试点，但必须记录 Decision。

## Ladder
R0 Observe → R1 Assist → R2 Orchestrate → R3 Shadow Replace → R4 Partial Replace Candidate

## Baseline
需要可复现：inputs、steps、roles、tools/software、cycle time、human effort、output quality、failure/rework、evidence coverage、acceptance/outcome。

## Candidate
Morn-native candidate 必须相同输入、相同或更严格 AcceptanceSpec、同类 Outcome metric、所有 side effect 隔离。

## Comparison
Quality、Acceptance、Cycle Time、Human Minutes、Retries、Error/Rework、Cost Estimate、Evidence Coverage、Policy Violations、Outcome。

## Decision
只有 candidate >= baseline on critical quality/safety + evaluation passed + certification passed + human approval，才可生成 `PartialReplaceCandidate`。仍不自动 retirement。

## Rollback
必须能回 existing/manual path 或 previous Morn Solution version。
