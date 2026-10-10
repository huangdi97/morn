# Morn v11.5 — DeepSeek Harness 官方发行与 ACP Provider 收敛核验

日期：2026-10-08  
状态：**SDK Provider 代码已实现；真实官方运行时/凭据 smoke 尚未验证；ACP 仍是增强生命周期路径**  
适用：Morn Work Control Plane / Provider Fabric / HarnessProvider / Factory read-only thin slice

## 1. 已核验事实与历史纠偏

旧版 `BLOCKERS.md` 的 August B-001 在当时报告「无官方安装包」。这个结论**不能继续作为 2026-10-08 的现状**，但旧检测日志必须保留以便审计。

DeepSeek 官方现在已提供：

- 官方产品及安装入口：https://www.deepseek.com/harness/en/ ，`npx @deepseek-ai/dsh web`
- 官方源码：https://github.com/deepseek-ai/deepseek-harness ，开发者预览、可能发生不兼容变更
- 官方自动化 ACP 包：https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/acp/acp/README.md
- 自动化进程启动方式：源码构建后 `pnpm dsh --profile acp`（**stdio JSON-RPC，不能将 Web UI HTTP 端口误当 ACP API**）
- Pi SDK 的进程内接口参考：https://pi.dev/docs/latest/sdk （Pi 是另一 HarnessProvider，不是 DSH fallback 的事实代理）

2026-10-08 当前实现事实：Morn 已有 `DshSdkStdioClient` 和 `DeepSeekHarnessProvider::with_real_sdk`，可按官方 SDK 的 `initialize → session/prompt → durable inbox receipt → session.status=idle → shutdown` 语义驱动一个真实 SDK runtime；仓库内协议 fixture 可验证 wire contract，但没有官方凭据与实际 DSH runtime smoke。因此 **Morn-side adapter 已实现 ≠ 官方 runtime 已实测 ≠ 真实模型成功 ≠ 客户 Outcome 已验收**。

## 2. 边界冻结

```text
Morn Work / Spec / Status / Conditions / Acceptance (durable canonical truth)
                  |
     HarnessProvider -> ExecutionBinding (generation/version/digest pinned)
                  |
       +----------+------------------------------+
       |                                         |
 DshSdkStdioClient                        DshAcpBridge (enhanced/future)
 implemented Morn path                    persistent lifecycle path
       |                                         |
 dsh --profile sdk                         dsh --profile acp
       |                                         |
   DSH session / agent / Cordis plugin tree / model / sandbox
                  |
    event + execution receipt + proposal (NOT accepted outcome)
```

- **Cordis** 负责 DSH 进程内部插件组合与生命周期；Morn 的 Work 状态不下放。
- **SDK/ACP session** 都只是执行上下文，不是 Work ID，不因 `session/prompt` 成功而推进 `Accepted`。当前 Morn 真实代码路径优先接官方 SDK wire；ACP 用于需要 `session/list/resume/close/cancel/request_permission` 的增强生命周期。
- **Permission / tool effects**：将来自 ACP 的授权请求绑定 `Work`、`ExecutionBinding`、Profile、Site、AuthorityDecision 和 `ExternalActionPermit`；没有明确许可一律拒绝。ACP 是可信程序接口，绝非隐含的生产写入授权。
- **Cancel** 只终止/中断会话活动；若外部写入已派发，不应转成「回滚成功」，必须保留 `OUTCOME_UNKNOWN` 并做 source-of-truth reconciliation。
- **Resume** 按历史 Work/Binding/Provider 版本恢复，失败或 provider 变化只能执行明确重新绑定/迁移，不能静默替换。
- **Credentials** 属于隔离的 Provider 进程或 secret store。不得进入 LLM prompt、日志、Work history、仓库或截图。
- **Real runtime** 必须在受控工作目录/隔离环境执行。公开的 `sdk-minimal` 示例可能使用 `danger-full-access` shell，绝不可在 Factory 或主机生产目录无条件复用。

## 3. Morn HarnessProvider ⇄ DSH SDK/ACP 契约映射

| Morn 边界 | 当前 SDK wire | ACP 增强面 | 必须补充的 Morn 保证 |
|---|---|---|---|
| `start` | `initialize` + 本地 session id（首 prompt 懒创建） | `session/new` | 独立 sandbox/workspace，绑定 Work generation 与 exact provider/runtime identity |
| `send` | `session/prompt` + `session.event/status` | `session/prompt/update` | 等待 durable inbox receipt 到 root idle；文本永远不是业务事实 |
| `stream_events` | SDK `session.event` 投影为 Morn execution events | ACP `session/update` | 不混同 durable domain events / Outcome |
| `inspect` | Morn-owned local projection | `session/list` | 区分 executor status 与 Work desired/observed |
| `interrupt` | **不支持，fail-closed** | `session/cancel` / `$/cancel_request` | E2/E3 side effects 独立对账 |
| `resume` | **不支持，fail-closed** | `session/resume` | 相同 workspace identity、binding version/digest |
| `terminate` | 只有 process-level `shutdown`，无 per-session close | `session/close` | 关闭 session/process 不等于真实世界副作用回滚 |
| external permission | SDK 当前没有 server→client approval surface | `session/request_permission` | 不可 auto-allow，仍需 Profile/Authority/Permit/Effect PEP |

注意：官方 SDK 明确没有 mid-turn cancel / session-close；Morn SDK Provider 因此对 `interrupt/resume/terminate(session)` fail-closed。官方 ACP 已提供 `list/resume/close/cancel/request_permission`，但 Morn 尚未把 ACP 作为第二个真实 Provider transport 接入，**不能把 ACP 能力误报成当前 SDK Provider 能力**。

## 4. 建议的可执行工程交付顺序

1. **协议兼容测试**：使用官方版本固定的 ACP server，以独立工作目录启动；记录精确包版本、commit/digest、配置与 `initialize` 协商结果。
2. **受限 read-only session**：在 no production credentials 的隔离 workspace `session/new` → `session/prompt` → 收取 `session/update` → `session/close`；测试异常退出、取消、重启后 resume。
3. **Rust SDK bridge（本轮代码已落地）**：专用子进程 + JSON-RPC 请求 ID 关联 + durable inbox receipt→idle activity interval + exact SDK server identity 校验；仍需继续补齐 bounded timeout、stderr 隔离/诊断上限和正式 runtime smoke。
4. **Morn Contract**：实现 `HarnessProvider`，将 fixture 与真实 ACP 适配走同一接口测试；provider 自身不能调用 Work canonical writes。
5. **外部动作治理**：action/permission 必须通过 Work/Profile/Authority/Effect/Binding/Permit 检查。遇到 timeout-after-commit 不得第二次 dispatch。
6. **可复现与升级**：带真实 SDK/runtime 版本的 manifest、签名/安装来源记录；Provider upgrade 先 shadow/conformance，然后创建新 Binding。
7. **真实验收**：仅在有授权的 site/customer 和真正外部结果时才有资格进入 G12；不能以 SDK prompt 成功替代 outcome acceptance。

## 5. 验收门禁（逐项独立判定）

| Gate | 通过条件 | 本次状态 |
|---|---|---|
| DSH-DIST | 官方可安装包与源码可核验 | `EVIDENCED_BY_PUBLIC_SOURCE`（**非本地安装验收**） |
| DSH-SDK-ADAPTER | Morn Rust Real provider 能驱动官方 SDK wire，并保持 Work truth 隔离 | `IMPLEMENTED_CODE_PENDING_EXACT_HEAD_CI` |
| DSH-SDK-LIVE | 固定版本官方 SDK runtime 的 initialize/prompt/receipt/idle/shutdown | `NOT_RUN` |
| DSH-ACP-WIRE | 固定版本 ACP 进程的 initialize/new/prompt/update/close | `NOT_RUN` |
| DSH-SESSION-RECOVERY | 取消、重启、resume、并发隔离 | `NOT_RUN` |
| DSH-AUTH | 默认拒绝权限与 E2/E3 side effects，绑定真实许可 | `NOT_RUN` |
| DSH-PROVIDER | Morn Rust Real provider adapter + protocol fixture | `IMPLEMENTED_CODE_PENDING_EXACT_HEAD_CI` |
| DSH-SWAP | DSH ↔ Pi 替换后 Work history/Outcome/Acceptance 不变 | `FIXTURE_ONLY` |
| CUSTOMER-G12 | 授权数据、权威系统观察与独立验收 | `EXTERNAL_BLOCKED` |

## 6. 运行说明（不能假装已执行）

官方 Web UI 快速启动：`npx @deepseek-ai/dsh web`。这一命令**不是 Morn DSH Provider 的集成测试**。

要做当前 Morn Provider 的真实自动化，请在隔离环境准备官方 `dsh --profile sdk`（或 Python SDK bundled runtime）、显式 `DSH_HOME`、workspace、凭据与模型，然后运行真实 smoke。需要持久 session lifecycle / permission request 时，再实现 ACP transport。仓库内 fake-wire PASS 只能证明 Morn-side protocol handling，不得列为 live DSH PASS。

## 6.1 2026-10-10：DSH 直接工具的执行前封锁

官方 `sdk` Profile 基于完整的 dsh-base，默认包含 Bash/PowerShell、文件系统、Skill、Subagent、Web 等模型可见工具。官方 CLI 的 `--patch` overlay 在 Profile/Home 层之后应用，因此是 Morn 在启动前收紧组合树的正确边界。

Morn 的真实 DSH SDK 路径现在采用三层防线：

1. **Global ToolRuntime guard**：Morn 的最后层 `--patch` 插入一个无额外依赖的 Cordis 插件，在全局 `ctx.tools.guard()` 上注册单调拒绝；此前 Profile/Home 层增加的未知、自定义工具也不能因为“不在禁用名单”而执行。
2. **最小可见面与只读 sandbox**：同一 overlay 隐藏官方 SDK Profile 当前已知的 shell/fs/jobs/skill/subagent/workflow/todo/goal/web/MCP 等工具生产者，并把 sandbox 固定为 `read-only`；子进程环境再设置 `DSH_PERMISSION_MODE=read-only`。overlay 与 guard 的字节内容用 SHA-256 固定进 route identity。
3. **Post-observation containment**：现有 event normalization 仍检查任何 `tool/call` / `tool/result`。如果被固定的 runtime 仍报告工具活动，Morn 拒绝输出、降级 Provider health 并立即回收 owned process。

第三层不能撤销已经发生的工具副作用，只是异常检测；**前两层 + exact runtime digest + 外部 tool-mediation attestation 才构成执行前控制**。生产配置不能关闭这份策略：仅 Rust `cfg(test)` 的 fake-wire fixture 可旁路，相关字段不暴露给 serde。Morn 不把 DSH 自身工具权限当成 Authority，E1/E2/E3 仍必须走 Morn Capability / Authority / ExternalAction。

该 overlay 针对被 runtime digest/version 固定的官方 `sdk` Profile。若部署改用其他 Profile，`validate_for_real` fail-closed；不能静默把 `web`、`sdk-minimal` 或任意自定义 Profile 当成等价运行路径。

## 7. 参考来源

- DeepSeek 官方：https://www.deepseek.com/harness/en/
- DeepSeek Harness GitHub：https://github.com/deepseek-ai/deepseek-harness
- DSH SDK protocol：https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/sdk/protocol/README.md
- DSH SDK client：https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/sdk/client/README.md
- DSH official ACP package：https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/acp/acp/README.md
- DeepSeek Python SDK warning/example：https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/user/guide/python-sdk.md
- Pi SDK：https://pi.dev/docs/latest/sdk
