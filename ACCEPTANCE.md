# ACCEPTANCE.md — 今晚验收门

> 状态：2026-08-15 逐条核对。`[x]` = 通过（代码+测试）；`[~]` = 部分/有说明；`[ ]` = 未通过。
> 全量验证：`scripts/run_all.ps1` exit 0；41 Rust 测试 + 2 前端测试全绿。

## A. 必过 Gate

### A1 Build
- [x] Rust workspace build 通过（`cargo build --workspace`）
- [x] 前端 build/typecheck 通过（`npm run build` = tsc + vite build）
- [x] Tauri app：desktop build 通过（`cargo build -p morn-desktop`，纳入 run_all）；二进制在沙箱外可启动（WebView2 151.0.4129.78）；沙箱内 GUI 启动被沙箱 token 拒绝（os error 5，KF-005，环境边界）
- [x] 锁文件存在且一致（Cargo.lock、package-lock.json）

### A2 Kernel / World
- [x] Workspace 隔离有测试（kernel `workspace_lifecycle` + store `workspace_isolation_filters_by_workspace`）
- [x] Ledger 关键事件可追溯（append-only ledger + store append/restart 测试）
- [x] Object/Event/StateSnapshot 可持久化（store object/snapshot 记录 + 重启测试）
- [x] Canonical World 写入必须走受治理 service/action gateway（`WorldService::commit_state` 私有 state；`runtime_cannot_commit_world_directly`）

### A3 Artifact / Decision / Outcome
- [x] Artifact 不可原地覆盖（supersede 保留旧版本；`old_version_remains_readable_after_supersede` + store 重启测试）
- [x] Review/Approval 可持久化（store save_review/save_artifact_approval）
- [x] DecisionPackage 关联 evidence / approval（DecisionPackage 结构含 source_artifacts/approvals/action_proposal）
- [x] Outcome 可关联 StateDiff/Verification（OutcomeRecord.state_snapshot_ids + verification_report_id；BioLab outcome 断言）

### A4 Work
- [x] WorkPackage 有 AcceptanceSpec（`work_with_satisfied_acceptance_can_be_accepted`）
- [x] 没有 AcceptanceSpec 的 work 不得被标记为“通过验收”（`work_without_acceptance_spec_cannot_be_accepted`）
- [x] ExecutionMode 能区分 deterministic/probabilistic/physical/regulated/social/mixed（enum + 测试）
- [x] RoleSlot 和 MemberBinding 分离（`role_slot_and_member_binding_are_separate`）

### A5 Harness
- [x] Actor identity 不属于 Runtime（ActorInstance 持有 identity_id；provider switch 测试）
- [x] HarnessSpec/Binding 可版本化（Version + HarnessVersion）
- [x] 至少 2 条 provider path 通过同一契约测试（MornNative + DeepSeekHarness(Fixture)，`contract_tests.rs`）
- [x] 切换 provider 不改 Morn canonical record（`provider_switch_preserves_actor_identity_and_canonical_records`）
- [x] provider E0 unmount cleanup 有测试（contract suite “E0 unmount cleanup” + idempotent guard）
- [x] Action proposal 不可绕过 ActionGateway（gateway 是唯一 write path；`runtime_cannot_commit_world_directly`）

### A6 Effects
- [x] E0 lifecycle_reversible（EffectContract::e0 + unmount 测试）
- [x] E1 transactional（EffectContract::e1）
- [x] E2 compensatable（`e2_without_compensation_fails_validation` + e2 默认 compensation_action）
- [x] E3 irreversible（`e3_without_approval_is_denied` / `e3_with_approval_is_authorized_and_executes_via_world`）
- [x] E3 未批准执行失败（同 e3_without_approval 测试）
- [x] E2 缺 compensation 时验证失败（同 e2_without_compensation 测试）

### A7 Durable
- [x] checkpoint 持久化（store save_checkpoint/load_checkpoints）
- [x] resume 测试（`checkpoint_resume_roundtrip`）
- [x] failure/recovery/attention 可追踪（RecoveryRecord + AttentionItem；`tool_failure_creates_attention_and_recovery`）

### A8 Evolution
- [x] Candidate 与 Production 分离（candidate 无写权限；仅 promote() 创建新版本）
- [x] Branch/Evaluation/PromotionDecision 可持久化（store 集成测试 `evolution_and_review_records_persist`）
- [x] failed evaluation 不能 promote（`failed_evaluation_cannot_promote`）
- [x] promotion 产生新版本（`successful_promote_creates_new_version`：1.0.0 → 1.1.0）
- [x] rollback metadata 存在（`rollback_restores_previous_version`）

### A9 BioLab
- [x] 最小 schema（biolab.Dataset/AnalysisRun/ScientificClaim object types + roles）
- [x] Dataset→Reviewed Claim E2E test/demo（`biolab_e2e_reaches_reviewed_claim_with_lineage` + HTTP smoke）
- [x] reviewer + human approval gate（statistical reviewer review + PI approval）
- [x] claim 可追到 dataset / analysis / artifact / decision（E2E 断言 + outcome 记录）

### A10 UI
- [x] Workbench 不是静态 mock（真实 API `/api/workbench`）
- [x] Studio 真实创建或预览 domain objects（真实 work_packages/object_types/roles）
- [x] Console 使用同一 backend records（`/api/console`：ledger traces/outcomes/promotions）
- [x] Hub 展示真实 registry assets（`/api/hub`）
- [x] loading / empty / error / blocked 状态（UI 组件齐全）
- [~] 关键页面无明显 overflow/console error：前端 build 通过；浏览器级 runtime QA 未自动化（KF-003 deferred）

### A11 Docs
- [x] STATUS 同步
- [x] DECISIONS 同步
- [x] BLOCKERS 同步
- [x] KNOWN_FAILURES 同步
- [x] final report 已生成（reports/tonight_final_report.md）

## B. DSH Gate

### B1 Morn 侧必须完成
- [x] `DeepSeekHarnessProvider` contract（`run_provider_contract` 通过）
- [x] Context provenance mapping（RuntimeContext.work_package_id/provenance_refs/policy_snapshot_version）
- [x] Event normalization mapping（ExecutionEventKind 归一化 + contract 断言）
- [x] Tool→ActionGateway enforcement seam（ActionGateway 唯一 write path）
- [x] scope isolation test/fixture（CapabilityScope + contract suite）
- [x] provider lifecycle test（mount/start/send/inspect/interrupt/resume/terminate）

### B2 外部真实集成
- [ ] 若当前环境可安装/启动 DSH：真实 smoke 通过 → 不可（B-001）
- [x] 若不可：`BLOCKERS.md` 有复现命令、原始错误、环境信息，且 Morn 侧 contract test 仍通过

B2 不允许伪造 → 真实 smoke 保持为 blocker，未标记完成。

## C. 不算完成

以下任一情况不能标记“Tonight Done” —— 逐条核对均不成立：
- [x] 无大量 `todo!()` / `unimplemented!()` 在 must-pass 路径
- [x] UI 不硬编码演示 JSON（全部来自真实 API）
- [x] 测试未被删除/ignore 才变绿（0 ignored；全绿）
- [x] Runtime 未直接写数据库绕过 domain（gateway/world service 是唯一 write path）
- [x] DSH provider 不是空 struct（完整边界 + fixture contract；真实集成如实标记 blocked）
- [x] Evolution candidate 不能直接改 production（仅 promote() 创建新版本）
- [x] Artifact 更新不覆盖旧版本（immutable versions）
- [x] E3 不能无批准执行（gate 测试）
