# Morn Architecture（v11.5-R1 六 Plane 现状映射）

> 本文件映射 v11.5 canonical 到仓库真实代码（Phase 20，Goal §24）。canonical 指针见 `docs/canonical/CURRENT.md`。

## 六 Plane

```text
1. Semantic / Specification Plane   → src/work/ + schemas/
2. Durable Control Plane            → src/control/（SQLite cp_* 表 + 12 controllers）
3. Composition Plane (Cordis)       → src/runtime/cordis/（唯一直接依赖 Cordis）
4. Execution Plane                  → src/providers/（harness/solver/human/connector/policy/gateway/execution_env）
5. Trust & Governance Plane         → src/providers/policy.rs + gateway.rs + src/work/authority.rs
6. Capability Supply Chain          → src/work/capability.rs + src/studio/{artifact2capability,package}.rs
```

Domain Products：

```text
Morn Factory                        → src/domains/factory/scenario.rs（CNC-17 reference scenario）
```

## 层完成度（Phase 22 口径）

| Plane | 状态 | 证据 |
|---|---|---|
| 1. Semantic | IMPLEMENTED | schemas/ 8 份 JSON Schema + src/work/ 12 文件；tests/work/tests.rs（G1） |
| 2. Durable Control | IMPLEMENTED | src/control/（14 表 + 12 controllers + outbox/lease）；tests/control_plane_g2.rs（G2 真 kill） |
| 3. Composition (Cordis) | IMPLEMENTED | cordis-rs =0.8.39 exact pin；src/runtime/cordis/；tests/runtime_cordis_g3.rs（G3 binding pin） |
| 4. Execution | IMPLEMENTED（real harness integration BLOCKED_EXTERNAL） | src/providers/ 9 文件；tests/providers_g4g5.rs（G4/G5）+ binding_pin_g6.rs（G6） |
| 5. Trust & Governance | IMPLEMENTED | policy.rs（deny-by-default）+ gateway.rs（PEP）；tests/trust_governance.rs |
| 6. Capability Supply Chain | IMPLEMENTED | capability.rs + artifact2capability.rs + package.rs；tests/capability_supply_chain.rs（G8）/ artifact2capability.rs / capability_package.rs |

## 语义边界（可观察）

- `Work is the unit of coordination` — Work.spec/status + WorkController 阶段推进（src/control/controllers/state_loop.rs）
- `Capability is the unit of composition` — WorkcellResolver 硬约束/软评分分离（src/control/workcell.rs）
- `Accepted Outcome is the unit of value` — ValueAssessment（src/work/outcome.rs）；reject 不 satisfy gate
- `Agent is one kind of executor` — ExecutorKind::Agent（Workcell 0-Agent 路径；solver/human provider）
- `Nothing may silently rewrite a committed fact` — append/supersede/retract（src/work/common.rs）+ Binding pin（tests/binding_pin_g6.rs）

## 依赖方向

```text
work（语义）→ control（控制平面）→ providers/runtime（实现）
work/control 不 import cordis/dsh/pi/temporal（tests/work_module_isolation.rs = 0 violations）
```

## 关键 ADR

见 `docs/adr/ADR-MORN-001..010`（Cordis composition only；DSH/Pi 是 Harness Provider；durable truth 独立；hot-swap 不改 active Attempt；capability 供应链；标准优先；Factory read-first；control loop + reconciliation；non-destructive evolution；domain 语义不绑定 Cordis types）。
