# GOAL3_EVOLUTION_FLYWHEEL.md

## 输入
ExecutionRecord、OutcomeRecord、AttentionItem、HumanCorrection、FailureRecord、Retry/Compensation、EvaluationResult、DeliveryAcceptance。

## Pattern
至少支持 repeated_success、repeated_failure、repeated_human_correction、repeated_approval、high_latency_step、high_cost_step、deterministic_candidate、redundant_handoff、capability_gap、low_value_software_dependency。

## Candidate
必须关联 source evidence window、affected work/workflow、baseline metrics、proposed change、risk、required evaluation、expected benefit、rollback plan。

## Promotion

```text
Candidate
→ Branch
→ Replay
→ Evaluation
→ Shadow
→ Certification
→ Promotion
→ New Version
```

不能直接修改 production。

## Distillation
如果稳定路径可转为 Rule / State Machine / Program / Solver，优先确定性能力。长尾 fallback to Actor / attention / escalation。
