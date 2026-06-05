# 贡献指南

感谢你考虑为 Morn 贡献代码。请遵循以下流程和规范。

## Issue 提交流程

### Bug 报告

- 使用 GitHub Issues 标记 `bug`
- 包含：环境信息（OS / Rust 版本）、复现步骤、期望行为、实际行为
- 附上相关日志或错误输出

### Feature 请求

- 使用 GitHub Issues 标记 `enhancement`
- 说明功能背景、使用场景、期望 API

### RFC（重大变更）

- 先开 Discussion 讨论，达成共识后提交 RFC Issue
- RFC 需包含：动机、设计思路、接口变更、迁移方案

## PR 流程

1. Fork 仓库，基于 `main` 创建分支：`git checkout -b feature/your-feature`
2. 提交前确保 `cargo build` 通过、`cargo test` 全绿
3. 提交 PR 到 `main` 分支，描述变更内容和动机
4. 等待 Code Review，根据反馈修改
5. 合并后分支将被删除

## 编码规范

- Rust 代码使用 `cargo fmt` 格式化，`cargo clippy` 无 warning
- 命名约定：类型和 Trait 使用 `PascalCase`，函数和变量使用 `snake_case`，常量使用 `SCREAMING_SNAKE_CASE`
- 避免 `unwrap()`，优先使用 `?` 运算符和模式匹配
- 所有 `pub` 项必须有 Rustdoc 注释

## 测试要求

- 新增功能必须包含单元测试
- `cargo test` 必须全绿
- 修改逻辑时先运行现有测试确保无回归

## 文档要求

- 新模块必须包含模块级 Rustdoc 注释
- `pub` 函数和类型须有文档注释（`///`）
- 文档应当说明用途、参数、返回值、panic 条件

## 行为准则

请遵守 [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) 行为准则。