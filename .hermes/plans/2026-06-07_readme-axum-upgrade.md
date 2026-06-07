# README 更新 + axum/tower 升级计划

## 背景
Phase 1-2 完成后项目状态已变，README 过期。axum/tower 依赖版本偏低需升级。

## 任务 A：README 更新（低风险，可手写）

### 改动项
| # | 原内容 | 改后 |
|---|--------|------|
| 1 | tests-417 badge（L4） | tests-469 |
| 2 | "0 errors, 3 minor warnings"（L20） | "0 errors, 0 warnings" |
| 3 | "417 passed"（L21） | "469 passed" |
| 4 | 项目结构树（L98-136） | 更新 core/supervisor/ → 子目录、persona/ → 子目录、workflow/ → templates/ 子目录 |
| 5 | 无 computer 模块描述 | 新增 computer/ 模块说明（7文件，45测试） |

### 执行方式
手写（纯文档改动），改完后 `cargo build` 确认不影响代码

## 任务 B：axum 0.7→0.8 + tower 0.4→0.5 升级（高风险）

### 前置条件
- [ ] 确认当前 axum 0.7 的使用范围（哪些文件 import axum）
- [ ] 确认 tower 0.4 的使用范围

### 风险点
- axum 0.8 路由 API 有破坏变更（Router::new() 接口变化）
- tower 0.5 Layer trait 签名变化
- 可能存在隐式依赖需要同步升级

### 执行方式
1. 先 grep 搜索所有 import 和使用
2. 写 tasks.md 给老板确认
3. 按指定工具执行

## 执行顺序
先 A（README，低风险，直接做）→ 然后 B（axum，需先出 tasks.md 确认）

## 约束
- A 段手写纯文档文件（README.md），不触代码
- B 段涉及代码改动，必须出 tasks.md → 等确认 → 执行
