# GOAL2_SIMULATION_EVAL_SPEC.md

## Replay vs Simulation vs Shadow
- Replay：过去输入/事件隔离重放。
- Simulation：主动注入假设、故障、权限、数据异常。
- Shadow：同输入下候选不产生生产副作用，与 baseline/production 比较。

## Scenario
至少：id、type、base_world_snapshot、solution_version、inputs、fault_injections、mocked_or_sandboxed_actions、expected_invariants、metrics、stop_conditions。

## Fault Injection
`tool_timeout / tool_error / harness_crash / model_unavailable / permission_denied / approval_missing / untrusted_context / representation_violation / evidence_conflict / malformed_data / budget_exhausted / deadline_expired`

## EvaluationResult
至少：scenario_id、solution_version、correctness、acceptance、policy、provenance、recovery、outcome、human_interventions、latency、cost、regressions、failures、evidence_refs、decision(pass/fail/conditional)。

## Promotion Readiness
Goal 2 只生成 readiness，不自动 promote 高风险 Production。

## Regression
新 SolutionVersion 必须至少对比 previous version、critical acceptance cases、safety/policy、known failures、BioLab golden fixtures。
