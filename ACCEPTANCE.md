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
- [x] 关键页面无明显 overflow/console error：Playwright UI smoke（`frontend/scripts/ui_smoke.mjs`）加载四个表面 + BioLab E2E，0 console error / 0 pageerror，exit 0

### A11 Docs
- [x] STATUS 同步
- [x] DECISIONS 同步
- [x] BLOCKERS 同步
- [x] KNOWN_FAILURES 同步
- [x] final report 已生成（reports/tonight_final_report.md）

## B. DSH Gate

### B1 Morn 侧本地可完成项
- [x] `DeepSeekHarnessProvider` fixture contract（`run_provider_contract`）
- [x] 官方 DSH SDK stdio/JSON-RPC real adapter：`initialize` / `session/prompt` / `shutdown`
- [x] Context provenance + exact Work generation / ExecutionBinding / execution-environment identity
- [x] deployment-pinned runtime version + SHA-256 digest；`serverInfo.version` 仅作 wire peer evidence
- [x] real provider 仅允许显式挂载 `morn.effects<=E0` scope；E1/E2/E3 必须走 Morn ExternalAction
- [x] receipt→idle→durable `turn/end` settlement；只有 `completed` 可成为 executor success evidence
- [x] timeout / lost settlement 强制回收 owned runtime，标记 outcome-unknown，禁止 blind retry
- [x] 当前 SDK 不支持的 interrupt/resume/per-session close 明确 fail-closed，不伪造生命周期
- [x] DSH public event normalization 不复制 private reasoning、assistant body、tool args/results
- [x] child environment scrub + isolated DSH_HOME + attested execution environment requirement
- [x] bounded wire frame / bounded reader queue / strict JSON-RPC version、response-id、result/error shape 校验
- [x] real-wire fake runtime contract tests，无需把 fixture 当作 authenticated live evidence

### B2 外部真实集成
- [ ] 使用已授权真实 DSH distribution + model route + credential + runtime-attested environment 完成 live settled-turn smoke（B-001）
- [ ] live provider health lease 的证据来自真实运行，而不是 wire fixture
- [x] 缺少上述外部条件时保持 EXTERNAL_BLOCKED；本地 CI 不伪造 PASS

B2 仍不允许伪造。官方发行与 Morn real adapter 已存在；未完成的是**经过授权的真实运行证据**，不是代码占位。

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
