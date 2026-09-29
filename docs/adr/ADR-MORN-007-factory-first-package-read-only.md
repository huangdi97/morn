# ADR-MORN-007：Factory 首包 read-first / no production write

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §26.3）
**上下文：** Factory 首个产品楔子是"停机/插单后的异常—产能—交期影响评审"。未完成真实客户 Gate 前，自动生产写不可承诺。
**决定：** 初始模式 READ/MAP/ANALYZE/CANDIDATE/EVIDENCE/EXPERT REVIEW；NO PRODUCTION WRITE（fake CMMS/CMMS 在 sandbox 模式）；`morn.factory.readonly` profile 的 production_write = FORBIDDEN。
**证据：** `src/work/profile.rs` factory_readonly()；`src/domains/factory/scenario.rs`（sandbox only）；`tests/profile_conformance.rs`（production_write_forbidden）；`tests/trust_governance.rs`（PEP 拒绝无授权写）。
