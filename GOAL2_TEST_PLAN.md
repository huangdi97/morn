# GOAL2_TEST_PLAN.md

## Mixed Organization
测试 authority scope/expiry、retained accountability、representation allow/deny/revoke、wrong member type。

## Durable
restart resume、wait signal、duplicate signal idempotency、timeout、retry/retry exhausted、E2 compensation、E3 no compensation bypass、budget stop、drift/replan/attention。

## Compiler Golden Cases
1. 固定 CSV→标准报告：优先 program，不自动创建 Actor。
2. 综合证据→研究假设：Actor + reviewer。
3. 湿实验执行：Human/Device + approval。
4. regulated/high-risk：必须 approval/policy gate。
5. capability gap：必须 Gap，不得虚构 tool。
6. incompatible harness：validation fail。
7. explainability：每个 planner decision 有 source/rationale。

## Manifest
serialize/deserialize、schema validation、version、diff、dependency incompatibility、dry-run、missing asset。

## Replay/Simulation
read-only production、fixture reproducibility、tool/harness fault、permission、approval、untrusted context、representation、evidence conflict、malformed data、budget、regression。

## Shadow
no production write、same input、comparison report、candidate worse → fail/conditional。

## BioLab
Loop A/B/C E2E，验证 provenance/version/independent review/human approval/claim-evidence consistency。

## UI
compiler smoke、durable timeline、attention action、simulation run、BioLab loops、manifest preview、build/typecheck/lint。

## Full Regression
Goal 1 全量回归必须继续通过。
