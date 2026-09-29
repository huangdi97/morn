# ADR-MORN-002：DSH/Pi 是 Harness Provider，不是 Work Control Plane

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §8.4）
**上下文：** DSH/Pi 负责 Agent loop、context、tool calls、LLM。若把它们当作 Control Plane，Work truth 会与 Harness Session 混淆。
**决定：** DSH 优先作为独立进程 Harness Provider（out-of-process；JSON-RPC/ACP/SDK 公开接口，不依赖 DSH 内部私有 Cordis tree）；Pi 作为第二 Provider 实现同一 HarnessProvider contract。
**边界：** DSH/Pi SessionEvent 可以保留为 Harness evidence，不得直接成为 Work truth。Morn 不合并 DSH-owned Cordis tree。
**证据：** `src/providers/harness.rs`（统一 contract）；`src/providers/harness_dsh.rs` / `src/providers/harness_pi.rs`（out-of-process skeleton + honest blocker）；`tests/providers_g4g5.rs`（同一 contract test）。
