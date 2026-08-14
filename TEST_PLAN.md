# TEST_PLAN.md — 测试策略

## 1. 测试金字塔

### Domain unit
覆盖：
- IDs/version/status
- Artifact versioning
- acceptance semantics
- effect classification
- production/evolution guard
- policy decisions

### Repository integration
SQLite：
- migrations
- CRUD/append
- transaction rollback
- workspace isolation
- restart persistence

### Service integration
- ActionGateway
- Artifact review/approval
- Work completion gate
- Harness event normalization
- durable checkpoint/resume

### Contract
同一 harness contract suite 跑：
- MornNativeHarness
- DeepSeekHarnessProvider bridge/fixture
- 若已有 CLI adapter，再跑 CLI adapter

### E2E
BioLab path。

### Frontend
至少：
- typecheck
- lint
- component/smoke（按仓库现有框架）
- build

## 2. 必写不变量测试

1. Runtime cannot mutate canonical world directly
2. Artifact old version remains readable after supersede
3. E3 action without approval is denied
4. E2 action without compensation contract fails validation
5. Work without AcceptanceSpec cannot be accepted
6. Harness switch preserves actor/work/artifact identity
7. Provider unmount cleans E0 only
8. Failed evolution evaluation cannot promote
9. Promotion produces a new production version
10. Restart can load checkpoint and resume
11. Downstream approved-only consumer rejects draft artifact
12. BioLab Claim lineage reaches Dataset/AnalysisRun/Decision

## 3. 失败策略

- 每发现 regression 先加/保留复现测试，再修。
- 不删除原有测试。
- 外部依赖 flaky 时：
  - 把外部 smoke 分层；
  - Morn contract test 仍必须稳定；
  - 记录 blocker。

## 4. 一键验证

优先使用仓库已有脚本。
如果没有，完善 `scripts/run_all.ps1`。

最终报告记录：
- command
- exit code
- pass/fail count（若工具提供）
- skipped 原因
- external blockers
