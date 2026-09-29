# ADR-MORN-006：标准优先：MCP/A2A/CloudEvents/OTel/OPA/OCI 等

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §3.8）
**上下文：** Morn 不自造协议宇宙。Tool/Resource 用 MCP；Agent↔Agent 用 A2A；REST API 用 OpenAPI；Event Envelope 用 CloudEvents；Telemetry 用 OpenTelemetry；Policy Decision 用 OPA/Cedar；Package 用 OCI/ORAS；Signing 用 Sigstore；Provenance 用 SLSA；Identity 用 SPIFFE/企业 IAM。
**决定：** 优先复用标准；CloudEvents 是外部/跨组件 Event Envelope（内部事件行不强行存成 CloudEvents JSON）；OTel semantic attrs 用 morn.* 前缀（11 个 canonical keys）；OPA/Cedar 作为可替换 PolicyProvider（reference 实现已证明 contract 可用）。
**证据：** `src/telemetry/mod.rs`（CloudEvent + morn.* attrs）；`src/providers/policy.rs`（PolicyProvider trait + reference 实现）；`tests/telemetry.rs` / `tests/trust_governance.rs`。
