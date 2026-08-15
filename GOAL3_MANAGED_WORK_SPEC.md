# GOAL3_MANAGED_WORK_SPEC.md

## ManagedWorkService
输入 CertifiedWorkCapability、WorkContract、Requester context、SLO、workspace/data boundary、human fallback。
输出 ManagedWorkRun、DeliveryReceipt、AcceptanceDecision。

## Delivery Lifecycle
Requested → Accepted → Scheduled → Running → Waiting → HumanFallback → Verification → Delivered → Accepted/Rejected → Retrying/Escalated/Cancelled → Closed

## DeliveryReceipt
必须有 work contract/version、capability/version、execution receipt refs、artifacts、decisions、state diff、verification、outcome、SLO result、evidence、failures/retries、human interventions、timestamps。

## Acceptance
不能由执行 Actor 自己判定。来源：automatic acceptance rules、independent reviewer、human customer/PI、hybrid。

## Human Fallback
定义 trigger、required role、handoff context、authority、resume path。

## Billing
仅保留 fixed / usage / milestone / outcome schema，不实现真实支付结算。
