# GOAL5_PROVIDER_CONNECTOR_RUNTIME_SDK.md

## Capability vs Provider
Capability = 能做什么；Provider = 谁/什么提供；Harness = 如何可靠组织智能执行；Runtime = 在哪里/如何跑；Connector = 如何访问外部系统。

## Provider Conformance
每个 provider：registration/version negotiation/health/scoped invocation/cancel/timeout/normalized error/audit/teardown/secret redaction/permission denial。

Provider 不得直接修改 canonical production state。

## Connector Contract
`ConnectorSpec/Instance/CredentialRef/ExternalSystemRef/ExternalObjectRef/ExternalActionRef/ExternalEvent/MappingSpec/SyncPolicy/SyncCursor/Health/Receipt`。

Read：External→Connector→normalize→provenance→proposal/validation→commit。
Write：Work/Decision→Action Proposal→Policy→Approval→Action Gateway→Connector→External Action→Verify→Receipt→StateDiff/Outcome。

提供 generic fixture connector：read/events/governed write/timeout/duplicate/rate limit/idempotency。

## Distributed Runtime Local Proof
Node A claim → checkpoint → A dies → lease expires → B restores → duplicate event → dedupe → no duplicate external effect → complete → failover audit。
