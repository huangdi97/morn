# ADR-MORN-009：Current projection 可变，released contracts 版本化，history non-destructive

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §13）
**上下文：** "immutable everything" 不现实：current projection 需要更新；released contract 需要冻结；historical fact 需要 supersede/retract/redact 而非无痕覆盖。
**决定：** 四类可变性：Draft/Current Projection 可修改；Released Contract/Artifact 版本内冻结（修改产生新版本）；Running Binding execution scope 内 pin（迁移必须显式）；Historical Fact append/supersede/retract/redact（不可无痕覆盖）。双时间（valid_time/recorded_time）；retract 取消被 retract 事件的效果（replay deterministic）。
**证据：** `src/work/common.rs`（HistoricalRecord + 双时间）；`src/control/projection.rs`（retract 取消效果）；`tests/failure_engineering.rs`（reordered/duplicate event）；`tests/profile_conformance.rs`（version frozen）。
