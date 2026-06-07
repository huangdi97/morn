# 回滚 delegate_task 产物 + 用 Codex 重做结构改进

## Status: PLAN ONLY — 等老板说「跑」再执行

## 背景
- commit `0e5455a` 混入了 delegate_task 做的文档注释和 unwrap 修复
- 已 revert：`d3fac2b` 将整个 commit 回滚到 `2c5abe9` 状态
- 工作树还有 Round 2 未提交改动（storage/ 拆目录等），已 stash 为 `Round2-uncommitted`

## 目标
还原结构改进（文件拆分 + 类型别名 + 版本方法 + 分段），但：
- ❌ 不加 `//!` 模块文档
- ❌ 不加 `///` 函数注释
- ❌ 不改 `.unwrap()` 处理
- ✅ pub 接口签名不变
- ✅ cargo build 通过

## 执行计划

### Step 1: persona.rs 拆成 5 文件
原 `src/component/persona.rs`（2116行）拆为：
- `src/component/persona/mod.rs` — 核心 struct + impl + 测试 + pub use 重导出
- `presets_general.rs` — general/human/assistant 预设
- `presets_tech.rs` — developer/coder/architect/analyst 预设
- `presets_creative.rs` — writer/designer/artist 预设
- `presets_industry.rs` — finance/medical/lawyer/teacher/researcher 预设
- 更新 `src/component/mod.rs`
- 删原 `persona.rs`

### Step 2: storage.rs 类型别名 + 设置表
- `Storage struct` 后加 `type CheckpointRow / ApprovalRequestRow / OAuthTokenRow / SessionRow`
- `init_tables()` 加 settings 表
- 替换 inline tuple 返回类型为类型别名

### Step 3: registry.rs version 字段 + 版本方法
- `AgentTemplate` / `Capability` 加 `pub version: String`
- `Registry.storage` → `_storage`
- 移除 `#[allow(dead_code)]`
- 所有模板实例加 `version: "0.1.0"`
- 新增：`get_version` / `list_by_version` / `check_conflict`

### Step 4: 其他编译警告修复
- 检查 `assembler.rs`、`personality_engine.rs`、`cli.rs` 等文件的纯结构优化
- 仅类型/结构级改动，跳过注释和 unwrap

### Step 5: 验证
- `cargo build` ✅
- `cargo test --lib` ✅
- 提交 commit

### Step 6: 恢复 Round 2 工作
- `git stash pop` 恢复未提交的 storage 拆目录等改动
- 解决可能的冲突

## 执行工具
- Codex CLI（`codex exec`），一次性喂入完整 task 描述

## 文件变更清单
（见之前 commit `0e5455a` 的 stat：63 files changed, 5198 insertions(+), 2319 deletions(-)，但只取其中结构改动部分）
