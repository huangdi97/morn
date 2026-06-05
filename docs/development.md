# 开发指南

## 项目结构详解

| 目录 | 职责 |
|------|------|
| `src/main.rs` | 入口，解析 `MORN_API_KEY`，初始化组件，启动 CLI REPL |
| `src/lib.rs` | 模块声明（8 个模块） |
| `src/core/` | 内核：COO 主管、注册中心、存储、执行引擎、安全、事件总线等 |
| `src/component/` | 六类原子组件（Tool / Knowledge / Skill / Persona / Memory / Model） |
| `src/bridge/` | LLM API 适配器（ChatAgent） |
| `src/channel/` | 渠道适配（CLI、Telegram、企微、钉钉、飞书等 12 个渠道） |
| `src/studio/` | 创作台后端（Manager、Publisher、Tester） |
| `src/console/` | 管理台后端（Dashboard、Cost、Governance） |
| `src/computer/` | 电脑操控（桌面 / 文件 / 浏览器 / 应用 / 系统 / 感知） |
| `src/market/` | 市场（Marketplace、Listing、Transaction、License） |
| `src-tauri/` | Tauri 桌面入口 |
| `web/` | React 前端（工作台、创作台、管理台） |

## 开发流程

```bash
# 1. 修改代码
# 2. 编译检查
cargo build

# 3. 运行测试
cargo test

# 4. 代码格式化
cargo fmt

# 5. Clippy 检查
cargo clippy --all-targets -- -D warnings
```

## 测试体系

`cargo test` 执行所有测试，包含：

- **单元测试** — 每个核心模块底部有 `#[cfg(test)] mod tests` 块
- **集成测试**（计划中）

主要测试覆盖：
- `supervisor` — 决策树匹配、上下文构建、turn 记录
- `engine` — DAG 调度、拓扑排序、循环依赖检测
- `storage` — 7 张表的 CRUD 操作
- `trust_evaluator` — 信任评分公式、四层评估
- `security` — 安全策略检查、拦截/审批/通知
- `dual_llm` — 6 个检查点、日志记录
- `orchestrator` — 7 种协作模式、4 种共识机制
- `workflow` — 8 个内建模板
- `marketplace` — 上架、购买、评分、搜索、安装

## Rustdoc

所有 `pub` 项应包含文档注释：

```rust
/// 做什么
///
/// 参数说明
/// 返回值说明
/// 何时 panic
pub fn my_function() -> Result<(), String> { }
```

生成文档：`cargo doc --no-deps --open`

## 代码风格

- 使用 `cargo fmt` 自动格式化
- Clippy 配置无 warning（`cargo clippy --all-targets -- -D warnings`）
- 命名：`PascalCase`（类型 / Trait）、`snake_case`（函数 / 变量）、`SCREAMING_SNAKE_CASE`（常量）
- 避免 `unwrap()`，优先 `?` 或模式匹配