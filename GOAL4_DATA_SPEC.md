# Goal4 Operational Episode & Outcome Data Spec

## Episode contract
```yaml
episode:
  id:
  workspace_id:
  domain:
  context_of_use:
  started_at:
  ended_at:
  world: {before_ref, after_ref, state_diff_ref}
  work: {package_ref, contract_ref, graph_ref, acceptance_ref, outcome_contract_ref}
  organization: {workcell_ref, member_bindings, delegation_refs, representation_refs}
  execution: {execution_mode, harness_refs, runtime_refs, actions, retries, compensations, approvals, attention_items}
  evidence: {artifacts, decisions, provenance_refs}
  outcome: {delivery_receipt_ref, acceptance_decision_ref, outcome_record_ref, metrics}
  economics: {latency, compute, model_usage, cost, human_minutes}
  labels: {success, accepted, failure_type, escalation, rework}
```

Label truth 必须来自 authoritative record：AcceptanceDecision、run timestamps、recorded human actions、terminal failure state。禁止执行 Actor/LLM 自评作为 Outcome truth。

Dataset 默认 temporal holdout；必要时按 case/workspace group split。自动检测 post-outcome feature leakage、future artifact version、same-episode duplicate leakage。
