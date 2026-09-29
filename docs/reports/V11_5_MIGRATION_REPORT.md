# Morn v11.5-R1 Migration Report（Phase 22）

**矩阵：** `docs/reports/V11_5_CODE_MAPPING_AND_MIGRATION_MATRIX.md`（Phase 0 对象级矩阵）

## 本 Goal 迁移执行摘要（Phase 0→22）

| Phase | 迁移/实现 | commit | 结果 |
|---|---|---|---|
| 0 Reality Audit | 真实仓库事实 + baseline 先跑后记录 + 对象级矩阵 + canonical 落位 | 450e0ed | G0 PASS |
| 1 Semantic | 8 份 JSON Schema + src/work/ 12 文件（Work.spec/status、Conditions、Authority、四态、Binding/Attempt 全状态机、Outcome/Acceptance/Value、Profile、HistoricalRecord） | d1f09bf | G1 PASS |
| 2 Control Plane | SQLite 14 张 cp_* 表 + event history/projection + 12 controllers + outbox/lease + 真 kill/restart 测试 | d65cea3 | G2 PASS |
| 3 Cordis | cordis-rs =0.8.39 exact pin + src/runtime/cordis adapter + 8 service slots + binding-pin lifecycle 测试 | 6b3abf2 | G3 PASS |
| 4 Execution Plane | HarnessProvider 统一合同 + DSH/Pi honest-blocked skeletons + fake harness/solver/human/connector + credential boundary | e21756f | G4/G5 contract PASS |
| 5 Binding Pin | unload/restart/second-provider/reconcile-first 故障注入 + receipt readback 防降级 | 965c4e0 | G6 PASS |
| 6 Receipt/Reconciliation | 完整 gateway 链 + retry storm + duplicate receipt + crash-after-commit（持久 fake CMMS） | e5b7704 | G7 PASS |
| 7 Outcome/Acceptance/Value | identifiable-source outcomes + multi-role acceptance + reject 不 satisfy gate + ValueAssessment | e24c72e | PASS |
| 8 Trust/Governance | PolicyProvider（deny-by-default）+ ActionGateway PEP + enforcement 测试 + env-var test race 修复 | bb05aeb | PASS |
| 9 Execution Environment | ExecutionEnvironmentSpec（DSec）+ local-process/container + future providers 诚实 FutureProvider + admission 检查 | 84b2919 + 27bead7 | PASS |
| 10 Capability Supply Chain | 四态分离 + lifecycle + strict-profile qualification gate + get_latest_qualification | e22521d | G8 PASS |
| 11 Resolver/Workcell | WorkcellResolver（硬约束 vs 软评分）+ typed resolution failures + 0-Agent/Agent+Rule/Agent+Solver+Human | 7a87719 | PASS |
| 12 Artifact2Capability | OpenAPI2Capability + Repo2Capability 确定性 compiler + 结构化失败状态 + qualification-gated release | dbc8165 | PASS |
| 13 Package/Provenance | content-addressed SHA-256 layout（ring，OCI-mappable）+ LOCAL_UNSIGNED provenance + tampering detection + PackageInputs options object | a26b092 + 75398d9 | PASS |
| 14 Profiles | 四 Profile conformance suite + 机器可读报告 + G9 required-condition gate + frozen version | 9c7b55e | G9 PASS |
| 15 Factory Thin Slice | CNC-17 deterministic scenario + 全 canonical 链 + evidence bundle + swap 入口 | 13e1080 | G10 PASS |
| 16 UI | playwright config msedge channel（browser provider 修复）+ e2e 真跑（2 passed/8 failed 已知 gap） | （本 commit 系） | PARTIAL |
| 17 Telemetry | morn.* 11 semantic attrs + CloudEvents envelope + secret redaction | b845dd6 | PASS |
| 18 Threat Model | 18 威胁矩阵 + reordered/duplicate/slow/upgrade fault injection + projection retract 修复 | 2591513 | PASS |
| 19 Conformance/Swap | swap benchmark（Run A/B 业务不变）+ provider conformance 同一合同 | d869886 | G11 PASS_WITH_LIMITATIONS |
| 20 Docs/ADR | ADR-MORN-001..010 + 5 guides + ARCHITECTURE.md + CURRENT 指针 | b367db2 | PASS |
| 21 Verify 收口 | scripts/verify-all.ps1 统一 gate（fmt/clippy/Rust/web/e2e SKIP_EXTERNAL） | 3922f69 | ALL_LOCAL_GATES_PASS |

## 旧资产映射（不推倒重写，Phase 0 矩阵执行结果）

- TaskRecord → Work 载体（HistoricalRecord 引用，不复制）——G1 证据
- approval → Authority 前身；capability registry → Capability 声明层；skill → Manifest 条目
- checkpoint → controller checkpoint；cost → ValueAssessment；hash-chain → receipt 完整性
- TF/RV/LC/BIOFERM 保持 LEGACY 不迁移（仅追溯）
- AgentLoop 硬编码输出记录为 Reality Gap（agent loop 属 Harness 职责——Phase 4 HarnessProvider 已说明边界）

## 推倒检查

- 无：旧 Core/Factory 能力经过显式映射与迁移，没有为了新架构无证据推倒。
- Cordis 是依赖 + adapter（非 fork）；DSH/Pi 是 out-of-process skeleton（非合并内部 tree）。
- 无第二套与 Six Records 重复的事实账本（G1 证据）。
