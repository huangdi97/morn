# Morn 代码完善：测试补全 + 代码质量修复

## 执行规则

1. 修改前确认当前源码状态
2. 改完立即 `cargo test` 验证不破坏现有测试
3. 改完立即 `cargo build` 确认 0 warning
4. 改完立即 `cargo fmt --check` 确认格式合规
5. 最终运行 `cargo test` + `cargo build` + `cargo fmt --check` + `web/npm run build` 全量验证

## 14 条核心准则编程原则

**准则 1: Think Before Coding** — 理解全貌再动手，不要边写边想
**准则 2: Readability Over Cleverness** — 代码是写给人看的
**准则 3: Single Responsibility** — 每个函数/模块只做一件事
**准则 4: Fail Fast** — 错误尽早暴露，不要吞错误
**准则 5: Test-Driven Habit** — 改完代码后第一时间跑测试验证
**准则 6: Don't Repeat Yourself** — 重复代码抽成函数
**准则 7: Composition Over Inheritance** — 组合优于继承
**准则 8: YAGNI** — 不要超前实现未要求的功能
**准则 9: Least Astonishment** — 最小惊喜原则，接口行为要直观
**准则 10: No Silent Failures** — 所有错误都要被处理或传播
**准则 11: Prefer Standard Library** — 标准库够用时别引入第三方依赖
**准则 12: Small Commits** — 每完成一个独立改动就提交
**准则 13: Document Why, Not What** — 注释解释"为什么"，不解释"是什么"
**准则 14: Keep Dependencies Minimal** — 只引入真正需要的 crate

## 低耦合原则

1. **模块间只通过公开接口通信** — 不要跨模块直接访问内部结构体字段
2. **测试独立于实现细节** — 测试公共 API 而非私有函数（除非是纯工具函数）
3. **新测试模块互相独立** — 每个 `#[cfg(test)] mod tests` 只测试本模块功能

---

## Phase 1: Clippy Warning 修复（3 个）

### 任务 1.1: 修复 println 空字符串

**文件:** `src/main.rs:53`

```rust
println!("");  // ❌ clippy 警告：空字符串
```

改为:

```rust
println!();    // ✅ 直接打印空行
```

### 任务 1.2: 修复复杂类型

**文件:** `src/main.rs:66`

冗余闭包 `|s| Marketplace::new(s)` → 直接 `Marketplace::new`

### 任务 1.3: 修复冗余闭包

同上，`chat_fn` 字段的类型过于复杂 → 用 `type` 别名简化

## Phase 2: a2a_discovery.rs 补测试

### 任务 2.1: 添加 peer 注册与查询测试

测试 `register_agent()` 后能通过设备名查到 agent。

### 任务 2.2: 添加 duplicate 去重测试

同一个 agent 注册两次，`discover_peers()` 应返回 1 条。

### 任务 2.3: 添加 serialize/deserialize 测试

A2AMessage 的 JSON 序列化/反序列化往返测试。

### 任务 2.4: 添加 task assignment 测试

构造 A2AMessage::TaskAssignment，验证其 JSON 结构和字段完整性。

## Phase 3: 大模块补测试

### 任务 3.1: component/tool.rs 补测试

- test_tool_registry: 注册/查询 tool
- test_tool_execution_basic: 简单 tool 执行
- test_tool_error_handling: tool 失败时返回错误

### 任务 3.2: component/skill.rs 补测试

- test_skill_load: 加载 skill 定义
- test_skill_execute: 执行 skill
- test_skill_invalid: 无效 skill 返回错误

### 任务 3.3: component/persona.rs 补测试

- test_persona_create: 构造 persona
- test_persona_default: 默认 persona 属性
- test_persona_to_system_prompt: 系统提示生成

## 验证

最终全量检查：

```bash
cd ~/morn-desktop
cargo build 2>&1 | grep -E "^(warning|error)" | wc -l   # 应为 0
cargo clippy 2>&1 | grep "warning:" | wc -l              # 应为 0
cargo test 2>&1 | grep "test result"                     # 全部通过
cargo fmt --check                                         # 无 diff
cd web && npm run build                                    # 前端构建通过
```
