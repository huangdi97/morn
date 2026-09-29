# ADR-MORN-005：Capability 进入 qualification/supply-chain governance

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §16）
**上下文：** CapabilitySpec（v11.4）只是声明。Capability = Implementation + Manifest + InterfaceContract + ExecutionRequirements + Qualification + AuthorityEnvelope + Provenance。
**决定：** Capability 四态（Declared/Observed/Qualified/Admitted）分离；asset lifecycle 与 site admission 分开；strict-profile gate：无 PASSED qualification 不进关键 binding；content-addressed package（SHA-256，OCI-mappable layout）+ 诚实 LOCAL UNSIGNED provenance。
**证据：** `src/work/capability.rs`（四态 + lifecycle + manifest）；`src/control/controllers/resolve_loop.rs`（strict gate）；`src/studio/package.rs`（digest + verify + provenance）；`tests/capability_supply_chain.rs` / `tests/capability_package.rs` / `tests/artifact2capability.rs`。
