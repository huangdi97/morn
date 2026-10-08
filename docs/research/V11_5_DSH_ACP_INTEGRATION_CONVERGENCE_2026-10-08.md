# Morn v11.5 — DeepSeek Harness 官方发行与 ACP Provider 收敛核验

日期：2026-10-08  
状态：**设计与事实核验；不等于已实现的真实 DSH 会话**  
适用：Morn Work Control Plane / Provider Fabric / HarnessProvider / Factory read-only thin slice

## 1. 已核验事实与历史纠偏

旧版 `BLOCKERS.md` 的 August B-001 在当时报告「无官方安装包」。这个结论**不能继续作为 2026-10-08 的现状**，但旧检测日志必须保留以便审计。

DeepSeek 官方现在已提供：

- 官方产品及安装入口：https://www.deepseek.com/harness/en/ ，`npx @deepseek-ai/dsh web`
- 官方源码：https://github.com/deepseek-ai/deepseek-harness ，开发者预览、可能发生不兼容变更
- 官方自动化 ACP 包：https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/acp/acp/README.md
- 自动化进程启动方式：源码构建后 `pnpm dsh --profile acp`（**stdio JSON-RPC，不能将 Web UI HTTP 端口误当 ACP API**）
- Pi SDK 的进程内接口参考：https://pi.dev/docs/latest/sdk （Pi 是另一 HarnessProvider，不是 DSH fallback 的事实代理）

2026-10-08 仓库事实：`crates/morn-harness/src/provider.rs` 的 `DshMode::Fixture` 有测试契约，但 `DshMode::Real` 仍然明确返回 `Error::External`。因此 **官方可安装 ≠ Morn 已接通 ≠ 已做真实模型 Smoke ≠ 客户 Outcome 已被验收**。

## 2. 边界冻结

```text
Morn Work / Spec / Status / Conditions / Acceptance (durable canonical truth)
                  |
     HarnessProvider -> ExecutionBinding (generation/version/digest pinned)
                  |
       DshAcpBridge (future external process, JSON-RPC stdio)
                  |
             dsh --profile acp
                  |
   DSH session / agent / Cordis plugin tree / model / sandbox
                  |
    event + execution receipt + proposal (NOT accepted outcome)
```

- **Cordis** 负责 DSH 进程内部插件组合与生命周期；Morn 的 Work 状态不下放。
- **ACP session** 是执行上下文，不是 Work ID，不因 `session/prompt` 成功而推进 `Accepted`。
- **Permission / tool effects**：将来自 ACP 的授权请求绑定 `Work`、`ExecutionBinding`、Profile、Site、AuthorityDecision 和 `ExternalActionPermit`；没有明确许可一律拒绝。ACP 是可信程序接口，绝非隐含的生产写入授权。
- **Cancel** 只终止/中断会话活动；若外部写入已派发，不应转成「回滚成功」，必须保留 `OUTCOME_UNKNOWN` 并做 source-of-truth reconciliation。
- **Resume** 按历史 Work/Binding/Provider 版本恢复，失败或 provider 变化只能执行明确重新绑定/迁移，不能静默替换。
- **Credentials** 属于隔离的 Provider 进程或 secret store。不得进入 LLM prompt、日志、Work history、仓库或截图。
- **Real runtime** 必须在受控工作目录/隔离环境执行。公开的 `sdk-minimal` 示例可能使用 `danger-full-access` shell，绝不可在 Factory 或主机生产目录无条件复用。

## 3. Morn HarnessProvider ⇄ DSH ACP 契约映射

| Morn 边界 | DSH 官方 ACP 能力 | 必须补充的 Morn 保证 |
|---|---|---|
| `start` | `initialize`、`session/new` | 独立 sandbox/workspace，绑定 Morn Work generation，记录 exact provider/runtime identity |
| `send` | `session/prompt`、`session/update` | 文本/事件/提案可以引用，但不直接成为业务事实 |
| `stream_events` | `session/update` | 有序、去重、会话与 Work 双向追溯；不混同 durable domain events |
| `inspect` | `session/list`、状态通知 | 区分 session live status 与 Work desired/observed |
| `interrupt` | `session/cancel` / `$/cancel_request` | E2/E3 side effects 继续独立对账 |
| `resume` | `session/resume` | 相同 workspace identity、binding version/digest、历史固定不变 |
| `terminate` | `session/close` | 将执行 receipt 与 accepted outcome 分开；关闭进程不保证业务效果消失 |
| `mount/unmount` | ACP MCP server mounts / scope lifecycle | 新工具仍需 Capability qualification 和外部 PEP |
| external permission | `session/request_permission` | 不能 auto-allow；明确动作、资源、Site、有效期和 EffectClass |

注意：官方 ACP 列出了已实现的 `session/list`、`session/resume`、`session/close`，但 Morn Rust `DshMode::Real` 还没有连接这些方法。**不能提前将未连通功能标记为 Supported。**

## 4. 建议的可执行工程交付顺序

1. **协议兼容测试**：使用官方版本固定的 ACP server，以独立工作目录启动；记录精确包版本、commit/digest、配置与 `initialize` 协商结果。
2. **受限 read-only session**：在 no production credentials 的隔离 workspace `session/new` → `session/prompt` → 收取 `session/update` → `session/close`；测试异常退出、取消、重启后 resume。
3. **Rust sidecar bridge**：专用子进程生命周期 + JSON-RPC 请求 ID 关联、超时、stderr 隔离、最大消息长度、取消和异常退出分类。stdout 仅协议，敏感配置不入日志。
4. **Morn Contract**：实现 `HarnessProvider`，将 fixture 与真实 ACP 适配走同一接口测试；provider 自身不能调用 Work canonical writes。
5. **外部动作治理**：action/permission 必须通过 Work/Profile/Authority/Effect/Binding/Permit 检查。遇到 timeout-after-commit 不得第二次 dispatch。
6. **可复现与升级**：带真实 SDK/runtime 版本的 manifest、签名/安装来源记录；Provider upgrade 先 shadow/conformance，然后创建新 Binding。
7. **真实验收**：仅在有授权的 site/customer 和真正外部结果时才有资格进入 G12；不能以 SDK prompt 成功替代 outcome acceptance。

## 5. 验收门禁（逐项独立判定）

| Gate | 通过条件 | 本次状态 |
|---|---|---|
| DSH-DIST | 官方可安装包与源码可核验 | `EVIDENCED_BY_PUBLIC_SOURCE`（**非本地安装验收**） |
| DSH-ACP-WIRE | 固定版本 ACP 进程的真实 initialize/new/prompt/update/close 可追溯 | `NOT_RUN` |
| DSH-SESSION-RECOVERY | 取消、重启、resume、并发隔离 | `NOT_RUN` |
| DSH-AUTH | 默认拒绝权限与 E2/E3 side effects，绑定真实许可 | `NOT_RUN` |
| DSH-PROVIDER | Morn Rust Real provider 与 fixture 同一 contract suite | `NOT_IMPLEMENTED` |
| DSH-SWAP | DSH ↔ Pi 替换后 Work history/Outcome/Acceptance 不变 | `FIXTURE_ONLY` |
| CUSTOMER-G12 | 授权数据、权威系统观察与独立验收 | `EXTERNAL_BLOCKED` |

## 6. 运行说明（不能假装已执行）

官方 Web UI 快速启动：`npx @deepseek-ai/dsh web`。这一命令**不是 Morn DSH Provider 的集成测试**。

要做真正的 Provider 自动化，请在独立环境按官方文档准备 `dsh --profile acp`、明确凭据与 sandbox/permission 控制，再运行 Morn 自己的 ACP sidecar contract tests。当前仓库尚未提供已验证的 sidecar，因此不得将其列为 PASS。

## 7. 参考来源

- DeepSeek 官方：https://www.deepseek.com/harness/en/
- DeepSeek Harness GitHub：https://github.com/deepseek-ai/deepseek-harness
- DSH official ACP package：https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/acp/acp/README.md
- DeepSeek Python SDK warning/example：https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/user/guide/python-sdk.md
- Pi SDK：https://pi.dev/docs/latest/sdk
