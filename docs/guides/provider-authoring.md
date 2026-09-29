# Provider Authoring Guide（Morn v11.5-R1）

Morn 的 Execution Plane 是异构执行器集合。Provider 是能力的运行时载体；业务层只依赖 Morn contract。

## HarnessProvider（Agent Harness）

实现 `src/providers/harness.rs` 的统一最小合同：

```text
capabilities()                     # 静态能力 + 诚实 blocker
start_session(binding)             # Binding 必须先存在
resume_session(session_ref)
submit(session, input)
cancel(session)
checkpoint(session)
get_events(session, cursor)
get_usage(session)
health()                           # Healthy | Unavailable | ExternalBlocked
close()
```

规则：
1. Session events 只是 Harness evidence——绝不直接成为 Work truth（映射进 Attempt/Receipt/Outcome 由 Control Plane 完成）。
2. 无真实凭据/endpoint 时 `health()` 诚实返回 ExternalBlocked；`start_session` 返回带 `REAL_PROVIDER_INTEGRATION_BLOCKED` 的 typed 错误——不伪造 session。
3. 凭据经 `CredentialProvider` 注入（scope→value 不越界，不入 git，不硬编码）。

参考实现：`src/providers/harness_dsh.rs`（out-of-process skeleton）、`src/providers/harness_fake.rs`（deterministic fake，控制平面验证）。

## ConnectorProvider（外部动作）

实现 `src/providers/connector.rs`：

```rust
fn create_order(business_key, payload_json) -> Result<ConnectorReceipt, String>
fn query_order(business_key) -> Result<Option<String>, String>
```

规则：
1. SAFETY：fake connector 仅用于 sandbox——绝不再是真实生产 endpoint。
2. 幂等：同 business key 返回既有订单（绝不重复）。
3. `timeout_after_commit` 模式模拟 "committed but unacknowledged"（G7 reconciliation 测试）。

## PolicyProvider（权限决策）

实现 `src/providers/policy.rs`：deny-by-default；OPA/Cedar/企业 IAM 可替换同一 contract。决策与执行分离（PEP 是 gateway）。

## ExecutionEnvironmentProvider（执行环境）

实现 `src/providers/execution_env.rs`：capability 只声明需求；Provider 决定落到 local process / container / microVM。future providers 诚实标 `FutureProvider`。

## 测试要求

所有 HarnessProvider 必须通过同一 contract test（`tests/providers_g4g5.rs` 模式）；provider conformance 是 Goal G11 的输入。
