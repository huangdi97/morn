# ACCEPTANCE.md — 今晚验收门

## A. 必过 Gate

### A1 Build
- [ ] Rust workspace build 通过
- [ ] 前端 build/typecheck 通过
- [ ] Tauri app 可启动或至少 desktop build 通过（若环境缺系统依赖，必须给出明确 blocker）
- [ ] 锁文件存在且一致

### A2 Kernel / World
- [ ] Workspace 隔离有测试
- [ ] Ledger 关键事件可追溯
- [ ] Object/Event/StateSnapshot 可持久化
- [ ] Canonical World 写入必须走受治理 service/action gateway

### A3 Artifact / Decision / Outcome
- [ ] Artifact 不可原地覆盖
- [ ] Review/Approval 可持久化
- [ ] DecisionPackage 关联 evidence / approval
- [ ] Outcome 可关联 StateDiff/Verification

### A4 Work
- [ ] WorkPackage 有 AcceptanceSpec
- [ ] 没有 AcceptanceSpec 的 work 不得被标记为“通过验收”
- [ ] ExecutionMode 能区分 deterministic/probabilistic/physical/regulated/social/mixed
- [ ] RoleSlot 和 MemberBinding 分离

### A5 Harness
- [ ] Actor identity 不属于 Runtime
- [ ] HarnessSpec/Binding 可版本化
- [ ] 至少 2 条 provider path 通过同一契约测试
- [ ] 切换 provider 不改 Morn canonical record
- [ ] provider E0 unmount cleanup 有测试
- [ ] Action proposal 不可绕过 ActionGateway

### A6 Effects
- [ ] E0 lifecycle_reversible
- [ ] E1 transactional
- [ ] E2 compensatable
- [ ] E3 irreversible
- [ ] E3 未批准执行失败
- [ ] E2 缺 compensation 时验证失败

### A7 Durable
- [ ] checkpoint 持久化
- [ ] resume 测试
- [ ] failure/recovery/attention 可追踪

### A8 Evolution
- [ ] Candidate 与 Production 分离
- [ ] Branch/Evaluation/PromotionDecision 可持久化
- [ ] failed evaluation 不能 promote
- [ ] promotion 产生新版本
- [ ] rollback metadata 存在

### A9 BioLab
- [ ] 最小 schema
- [ ] Dataset→Reviewed Claim E2E test/demo
- [ ] reviewer + human approval gate
- [ ] claim 可追到 dataset / analysis / artifact / decision

### A10 UI
- [ ] Workbench 不是静态 mock
- [ ] Studio 真实创建或预览 domain objects
- [ ] Console 使用同一 backend records
- [ ] Hub 展示真实 registry assets
- [ ] loading / empty / error / blocked 状态
- [ ] 关键页面无明显 overflow/console error

### A11 Docs
- [ ] STATUS 同步
- [ ] DECISIONS 同步
- [ ] BLOCKERS 同步
- [ ] KNOWN_FAILURES 同步
- [ ] final report 已生成

## B. DSH Gate

### B1 Morn 侧必须完成
- [ ] `DeepSeekHarnessProvider` contract
- [ ] Context provenance mapping
- [ ] Event normalization mapping
- [ ] Tool→ActionGateway enforcement seam
- [ ] scope isolation test/fixture
- [ ] provider lifecycle test

### B2 外部真实集成
- [ ] 若当前环境可安装/启动 DSH：真实 smoke 通过
- [ ] 若不可：`BLOCKERS.md` 有复现命令、原始错误、环境信息，且 Morn 侧 contract test 仍通过

B2 不允许伪造。

## C. 不算完成

出现以下任一情况不能标记“Tonight Done”：
- 大量 `todo!()` / `unimplemented!()` 位于 must-pass 路径；
- UI 只硬编码演示 JSON；
- 测试被删除/ignored 才变绿；
- Runtime 直接写数据库绕过 domain；
- DSH provider 只是空 struct 却写“已集成”；
- Evolution candidate 能直接改 production；
- Artifact 更新覆盖旧版本；
- E3 可无批准执行。
