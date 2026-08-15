# Goal4 恢复与专项指令

## A 上下文重置
读取 GOAL4_CODEX_MASTER_PROMPT.md、STATUS_GOAL4.md、GOAL4_ACCEPTANCE.md、DECISIONS/BLOCKERS/KNOWN_FAILURES、reports/goal3_final_report.md；查看 git status/diff/recent commits/tests；从第一个未完成 milestone 继续。

## B Persistence 卡住
先 Certification+CapabilityRelease vertical slice：migration→repository→service→API→restart test；通过后扩 managed/replacement/evolution/distillation。

## C Predictor 卡住
先 DurationPredictor：authoritative label→dataset split→historical/simple baseline→evaluation→uncertainty→registry→inference；再复制 contract。

## D 数据不足
禁止 synthetic outcome。实现 insufficient-data、baseline-only、context restriction；继续 Episode/Dataset/Registry/UI；FULL pilot 标 blocker。

## E DSH 不可用
保留 B-001 Active；HarnessSmokeContract/adapter/error handling/Action Gateway contract tests 全绿；不要 fake real smoke。

## F 最终验收
逐条 GOAL4_ACCEPTANCE，Goal1-3 full regression；本地可修全部 green；生成 goal4_final_report，明确 CORE/FULL/blockers。
