# DSH_SPIKE.md — DeepSeek Harness / Cordis Architecture Spike

## 目标

验证 DeepSeek Harness 是 Morn 的 Provider，而不是 Morn 的 Source of Truth。

## 必验 8 项

1. DSH 能作为独立 Provider 启动并被 Morn 管理；
2. 每个 Actor/Workcell 能有隔离 Scope；
3. WorkPackage/Artifact Context 可注入，同时保留 Morn provenance；
4. Tool Call 可强制经过 Morn Action Gateway；
5. DSH Session Event 可归一化为 Morn ExecutionEvent；
6. Provider/Plugin 卸载能清理 E0 lifecycle effect；
7. 替换 Agent Loop/Model Provider 后，World/Work/Artifact 不变；
8. Cordis 是否适合 Provider Lifecycle/Scope/Effect Tracking，且不侵入 Semantic Kernel。

## Provider Contract 建议

Morn 侧至少需要概念能力：

```text
start(actor, work, ctx) -> session
send(session, event) -> proposal
stream_events(session) -> HarnessEvent
inspect(session) -> snapshot
interrupt(session)
resume(session)
terminate(session)
```

不要把上面的示意接口机械照抄成一个巨型 trait；适配仓库语言风格。

## Context Mapping

注入 DSH 的 model-visible context 必须能映射回：
- source WorkPackage id/version
- Artifact id/version
- Workspace
- policy snapshot/version
- actor/harness version
- provenance refs

## Tool Mapping

DSH Tool Request：
```text
DSH Tool Request
→ Morn normalized Tool/Action Proposal
→ ActionGateway.preview
→ authorize
→ execute
→ verify
→ normalized result
→ DSH
```

禁止 provider 直接拿 Morn DB connection 做写入。

## Event Mapping

至少归一化：
- session_started
- model_request/model_response summary metadata
- tool_proposed
- tool_started
- tool_completed
- tool_failed
- checkpoint
- interrupted
- resumed
- completed
- failed

不保存模型私有 chain-of-thought；保存可审计摘要、输入输出 hash/refs、版本、receipt。

## Cordis

今晚只回答：
- lifecycle/scope 是否有可复用模式；
- effect/coeffect 是否能映射到 Morn capability seam；
- API 稳定性是否足以硬依赖。

默认结论应倾向：**不作为 Semantic Kernel 硬依赖**，除非仓库已有成熟使用且测试证明稳定。

## 真实集成失败时

必须在 `BLOCKERS.md` 记录：
- OS
- runtime versions
- install command
- start command
- error
- expected
- workaround tried
- Morn contract tests 是否仍过

不得把 fixture 测试写成“真实 DSH 已集成”。

## 2026-10-09 SDK safety convergence

- The real DSH SDK adapter is intentionally **E0-only** at the HarnessProvider seam because the current SDK wire has no Morn-controlled permission callback for internal tool calls.
- A real DSH execution scope must carry `morn.effects<=E0`. E1/E2/E3 actions remain governed Morn ExternalAction operations; provider output cannot bypass Authority/permit/reconciliation.
- Public durable DSH `session.event` records are normalized into Morn ExecutionEvents for model request/response summaries, tool proposal/result and turn checkpoints. Assistant text, tool arguments/results and private reasoning are not copied into audit summaries.
- This restriction is a control-plane admission rule, **not a sandbox attestation**. Real deployments still require an execution environment that independently confines the DSH process/workspace.
