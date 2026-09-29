# Morn v11.5-R1 Known Gaps and External Blockers（Phase 22）

## External Blockers（逐项含解除条件）

| # | 项 | 缺什么 | 谁才能解除 | 解除后的验收步骤 |
|---|---|---|---|---|
| 1 | DSH Harness 真实接入（G4） | DeepSeek Harness 进程/接口访问 + API key | 用户提供 DSH 运行环境与凭据 | `DshHarnessProvider` skeleton 已有 contract tests（e21756f）；接入后跑真实 contract smoke（start/resume/cancel/events/usage）+ CNC-17 Run A 真跑 |
| 2 | Pi Harness 真实接入（G5/G11） | Pi 访问 + 凭据 | 用户提供 | `PiHarnessProvider` skeleton 已有；接入后跑 Pi contract + swap benchmark（Run A=DSH Run B=Pi 业务不变条件）→ G11 升级真实 PASS |
| 3 | 真实客户数据 / 现场（G12） | 客户授权数据、现场安全/回滚/审批条件 | 客户 + 人类授权 | 母版 0.2 边界重新验证；read-only/shadow 计划；真实 ObservedOutcome/Acceptance 取证 |
| 4 | Sigstore/SLSA 真实签名 | 真实 signer（Sigstore/cosign 或企业 signer） | 用户提供 signer 基础设施 | `LocalUnsignedProvenanceProvider` 已有 contract（LOCAL_UNSIGNED 诚实标记）；接入后 package 升级 SIGNED status |
| 5 | 容器化 DB restart（G2 附注） | Docker/DB 集群部署 | 用户部署 | SQLite 文件级重启已证明 durable state；容器化后跑 DB restart fault injection |
| 6 | OPA/Cedar 真实 policy provider | OPA/Cedar 部署 | 用户部署 | `ReferencePolicyProvider` 已证明 PolicyProvider contract 可用；接入后跑同一 conformance suite |

## Known Gaps（真实债务，非阻塞发布）

| 项 | 优先级 | 原因 | 是否阻塞发布 |
|---|---|---|---|
| e2e ③–⑩ 失败（welcome-flow 时序/遮挡 + UI 结构漂移） | P3 | probe 取证：欢迎页 overlay（z-9999 fixed）在 skipWelcome 点击后未关闭导致 send-btn 被遮挡；Studio 8 tab/Store/Console 结构与 v0.2.0 UI 漂移 | 否（Phase 0 baseline 已记录；核心链路在 Rust 层全绿） |
| v11.5 Workbench/Console/Studio 完整 UI（Work list、conditions、candidates、active workcell 等） | P3 | 当前 web UI 是现栈 chat-first UI；v11.5 Workbench 概念未做完整 UI 重写（Goal §20 要求延续现栈，不为重写换框架） | 否（控制平面语义/API 已完整；UI 是 UX 层） |
| npm audit 11 vulnerabilities | P4 | web 依赖审计 | 否（Phase 21 评估记录） |
| node 版本 ci.yml=20 vs release.yml=22 | P4 | CI 配置漂移 | 否 |
| web lint 脚本无 eslint 配置 | P4 | 无配置 | 否 |
| update.json 空 signature | P4 | 不伪造签名（Phase 22） | 否（诚实标记） |
| 未来大型能力（大规模 DSec、K8s、SPIFFE 全栈、WASM 全组件化、真实 ERP/MES 全矩阵） | P5 | 按母版 §29 DONE 定义不要求本次全部实现 | 否 |
