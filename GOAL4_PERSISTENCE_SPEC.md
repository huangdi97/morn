# Goal4 Persistence Spec

## 原则
Goal3 正式对象必须成为 durable SoR，不是 process memory cache。one canonical persistence path；workspace scoped；immutable evidence/release history；transactional；explicit version；idempotency；audit events；no silent overwrite。

建议逻辑实体：certification_specs/runs/decisions、certified_work_capabilities、capability_releases/lifecycle_events、managed_work_runs、delivery_receipts、acceptance_decisions、human_fallback_records、replacement_baselines/shadow_comparisons/replacement_decisions、evolution_patterns/evidence_windows/candidates、distillation_records、rollback_requests/receipts。

先映射现有 domain objects，不要复制 v2 类型。

必须测试：fresh migration、Goal1/2 DB upgrade、restart hydration、workspace isolation、idempotent receipt、concurrent/version conflict、history immutable。
