# ADR-MORN-001：不自研通用组合框架，reference runtime 采用 Cordis

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §8.1 / ADR-MORN-001 建议）
**上下文：** Morn 需要 DI、Service Registry、Plugin Lifecycle、Effect Cleanup、Dependency Graph、Scoped Context、Hot Replacement 等通用机制。自研（MornMetaKernel）成本高、验证负担重。
**决定：** Reference Composition Runtime 第一阶段直接采用 upstream/pinned Cordis（`cordis-rs = "=0.8.39"`，MIT，github.com/dshbox/cordis-rs —— DeepSeek Harness Cordis 4.x 的 Rust port）。Morn 不重复开发 MornContext / MornDI / MornPluginRegistry / MornFiber / MornEffectSystem。
**边界：** Cordis 只位于 Composition Plane（`src/runtime/cordis/` 是唯一直接依赖 Cordis 的区域）；业务域包通过内部 adapter 使用，禁止直接 `use cordis::`。Cordis Effect Lifecycle ≠ Business Effect Lifecycle（`fiber.dispose()` 不能撤销现实世界的采购单/维修单）。
**证据：** `src/runtime/cordis/mod.rs`（facade + upstream pin 常量）；`tests/runtime_cordis_g3.rs`（isolation + binding pin）；`tests/work_module_isolation.rs`（依赖扫描 = 0 violations）。
