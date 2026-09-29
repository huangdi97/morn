# CURRENT — Canonical 指针

**本文件是仓库 canonical 指针。** 按 Goal（Phase 0）要求落位，旧母版不删除。

## 现行设计规范（最高优先级）

- `docs/canonical/Morn_v11.5-R1_协议驱动控制平面_Cordis参考运行时_CapabilitySupplyChain_统一全量母版_2026-09-24.md`
  - 状态：**Design Convergence / Proposed Canonical Master for Review（架构收敛母版，尚非生产批准）**（原文档头部状态，未被本仓库升级）
  - 直接继承：v11.4-R1（2026-09-17）；上游规范基线：v11.3-R1（2026-09-13）

## 执行 Goal

- `docs/canonical/Morn_v11.5-R1_全量连续执行_GOAL_2026-09-25.md`
  - 状态：已批准的连续执行 Goal（Phase 0–22），执行状态见 `docs/reports/V11_5_EXECUTION_STATE.md`

## ADR（架构决策记录，Phase 20 冻结）

- `docs/adr/ADR-MORN-001..010` — Cordis composition only；DSH/Pi 是 Harness Provider；durable truth 独立；hot-swap 不改 active Attempt；capability 供应链治理；标准优先；Factory read-first；control loop + reconciliation；non-destructive evolution；domain 语义不绑定 Cordis types

## Guides（Phase 20）

- `docs/guides/provider-authoring.md` — HarnessProvider/Connector/Policy/ExecutionEnvironment 合同与规则
- `docs/guides/capability-authoring.md` — 四态分离、Manifest、Artifact2Capability、Qualification、Release
- `docs/guides/profile-authoring.md` — Guarantee Profile 与 conformance
- `docs/guides/local-development.md` — 本地构建环境与工程规范
- `docs/guides/factory-reference-scenario.md` — CNC-17 reference scenario 运行指南
- `ARCHITECTURE.md` — 六 Plane 现状映射
- `docs/security/MORN_V11_5_THREAT_MODEL.md` — 威胁模型（18 项）+ fault injection 矩阵

## 辅助文件

- `docs/canonical/Morn_v11.5-R1_AGENT_HANDOFF_TEMPLATE.md` — Agent 交接模板
- `docs/canonical/Morn_v11.5-R1_AGENT_PACK_README.md` — Agent Pack 说明

## 历史文档（仅追溯，不是现行规范）

- `docs/*.md`（architecture/channels/components/computer-control/console/coo/development/market/plugin-development/plugin-examples/setup/studio，共 12 个）— v0.2.0 时代工程文档，仅用于现状追溯与旧接口定位，**不得静默继承为现行设计**。
- 仓库中不存在 v11.4 / v11.3 / 2026-08-31 母版文件（未在本仓库落盘过）。

## 当前成熟度声明边界（继承自母版 0.2，不得被架构文档升级）

```text
LIVE_CUSTOMER_DATA_RECEIVED  != proven by any document in this repo
LIVE_FACTORY_READ_ONLY       = EXTERNAL_BLOCKED unless later revalidated
LIVE_SHADOW_OUTCOME          = EXTERNAL_BLOCKED unless later revalidated
REAL_PRODUCTION_WRITE        = NO unless separately authorized and verified
CONTROLLED_REAL_WRITE        = NO unless separately authorized and verified
COMMERCIAL_MATURITY          = NOT_CLAIMED
```
