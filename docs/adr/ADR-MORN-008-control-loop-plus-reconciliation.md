# ADR-MORN-008：Control loop + reconciliation 优先于 all-in-one agent/DAG

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §7.3）
**上下文：** 真实企业工作常见：外部状态随时变化、审批等待数天、API timeout 后结果未知、Provider 可用性变化、同一 Work 既有确定性步骤也有不确定推理、人可中途修改目标、现实动作可能不可逆。
**决定：** 主架构 = Durable Resource State + Controllers/Reconciliation + Optional Durable Workflow + Agent/Solver/Human/Rules。不是把整个企业运营画成一张固定 DAG。Controller 只负责把局部 observed state 向 desired conditions 推进；不允许一个"超级 Agent"直接拥有所有控制权。
**证据：** `src/control/controllers/state_loop.rs`（WorkController deterministic 阶段推进）；`src/control/controllers/`（12 个 reference controllers）；`tests/control_plane_g2.rs`（幂等 + crash-safe）。
