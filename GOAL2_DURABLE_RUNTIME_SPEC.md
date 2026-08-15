# GOAL2_DURABLE_RUNTIME_SPEC.md

## 状态机
```text
Draft → Ready → Running
Running → WaitingSignal | WaitingApproval | Paused | RetryScheduled | Compensating | Blocked | Escalated | Completed | Failed | Cancelled
```
状态转移必须验证。

## Checkpoint
保存 workflow run/version、current/completed steps、pending work、canonical record refs/versions、harness session refs、budget consumed、attention、last event、timestamp。

不保存未经授权的私有 chain-of-thought。

## Signal
支持 human_approval、external_event、manual_resume、cancel、data_arrived、reviewer_response。必须有 identity/authority/timestamp/schema/idempotency。

## Drift Detection
Resume 前比较 World/Artifact/WorkContract/Policy version 与 Capability/Harness availability。超出允许范围：DriftDetected → Replan/HumanAttention。

## Retry
max attempts、backoff、retryable/nonretryable、budget impact、escalation。

## Compensation
仅 E2 明确路径：记录 original action、compensation action、result、residual risk。E3 不得伪装成 compensation。

## Budget
token/model、money estimate/actual、time、retry budget。超限 pause/block/escalate。

## Attention Types
ApprovalRequired / PolicyConflict / EvidenceConflict / ToolFailure / BudgetRisk / DeadlineDrift / LowConfidence / IrreversibleAction / RepresentationBoundary / ResumeDrift / CapabilityGap。
