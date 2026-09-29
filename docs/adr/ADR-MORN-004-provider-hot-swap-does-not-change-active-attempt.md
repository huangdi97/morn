# ADR-MORN-004：Provider hot-swap 不改变 active Attempt

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §10.3）
**上下文：** Cordis 可以在 Provider 消失时卸载 consumer，再在新 Provider 出现时重新 load。但业务动作可能已经发生（POST /cmms/create-order → timeout）；此时把 Provider A 自动切到 B 并继续同一 Attempt 可能造成重复工单。
**决定：** ExecutionBinding = 一等资源；Attempt 开始前必须有 Binding；active Attempt pin 到 Binding；Cordis hot reload 只影响 future resolution，不能静默改变 active Binding；Provider 故障 → SUSPENDED/OUTCOME_UNKNOWN；换 Provider 创建新 Binding/新 Attempt；已发生 side effect 的 Attempt 先 reconcile，不得从头重跑。
**证据：** `src/work/execution.rs`（Binding/Attempt 状态机）；`src/control/controllers/execution_flow.rs`（dispatch 前置检查 + suspend）；`tests/binding_pin_g6.rs`（unload/restart/second-provider/reconcile-first 四路注入）。
