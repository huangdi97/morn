# GOAL3_TEST_PLAN.md

## Certification
- insufficient evidence cannot certify
- failed critical policy cannot certify
- version change invalidates certification when required
- restricted capability obeys context-of-use
- suspended capability cannot start managed work

## Evolution
- pattern detection from fixture runs
- candidate links source evidence
- candidate cannot mutate production
- failed evaluation cannot promote
- human correction contributes candidate evidence
- deterministic candidate preserves fallback

## Distillation
- repeated path → deterministic candidate
- regression compares Actor baseline
- long-tail input falls back to Actor
- quality not worse beyond threshold
- cost/latency comparison recorded

## Managed Work
- only certified capability can start
- SLO tracked
- human fallback
- retry liability
- delivery receipt completeness
- acceptance independent of executor

## Replacement
- baseline reproducible
- same input comparison
- no production side effect in shadow
- worse candidate cannot produce R4
- policy regression blocks R4
- human approval required
- rollback path exists

## Regression
Goal 1 + Goal 2 full regression must remain green.
