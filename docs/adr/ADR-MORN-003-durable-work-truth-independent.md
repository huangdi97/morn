# ADR-MORN-003：Durable Work Truth 独立于 Cordis Fiber / Agent Session

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §2.3）
**上下文：** 过去把 "Work Runtime" 理解成某个 Harness/Workflow 的运行状态。真实 Work 的 durable state 必须独立于 Cordis Fiber、Agent Session、Sandbox Instance 和 Workflow Execution。
**决定：** 新增 Durable Control Plane：API/Resource Store（SQLite 14 张 cp_* 表）、Append/Supersede Historical Facts、Current Projection、Controllers、Reconciliation、Binding、Conditions、Profile Conformance。canonical progress 不存进程内存。
**证据：** `src/control/`（store/ddl/projection/controllers）；`tests/control_plane_g2.rs`（真进程 kill 后 Work/Attempt/Binding 存活 + reconcile 继续）。
