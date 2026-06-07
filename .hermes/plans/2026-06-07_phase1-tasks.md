# Phase 1 执行任务（供执行 agent 读取）

## 编码准则（执行 agent 必须遵守）

### 15条核心准则
| # | 准则 | 说明 |
|---|------|------|
| 1 | **Think Before Coding** | 先想后写，改之前先理解整体结构 |
| 2 | **Simplicity First** | 简单优先，不加不必要的抽象层 |
| 3 | **Surgical Changes** | 一次一事，一个PR/commit只做一件事 |
| 4 | **Goal-Driven Execution** | 目标驱动，先明确"要什么"再"怎么写" |
| 5 | **架构优先，拒绝补丁** | 不堆补丁，架构不合理就重构 |
| 6 | **面向组件的构建** | 模块化，每个组件职责清晰 |
| 7 | **显式优于隐式** | 明确的数据流和依赖，不搞魔术 |
| 8 | **代码整洁与自文档化** | 代码即文档，命名和结构说明一切 |
| 9 | **单一职责** | 一个函数/类只做一件事 |
| 10 | **组合优于委托** | 组合模式 > 继承/委托 |
| 11 | **单一状态源** | 状态只在一个地方管理 |
| 12 | **避免语法糖** | 可读性 > 炫技 |
| 13 | **命名一致性** | 同一概念用同一命名 |
| 14 | **文件不超过300行** | 超了拆 |
| 15 | **执行工具由老板指定** | 他用什么就用什么，不自动 fallback |

### 低耦合3条
- **模块间只传ID，不传对象**
- 采集器产出信号 → 只传 signal_id → 分析器自己从 DB 读
- 不 import 其他模块的内部函数
- 依赖倒置：模块通过接口/ID通信，不直接耦合

### 执行规则
- 不允许手动改代码文件（patch/write_file/terminal 改代码）— 所有代码修改必须通过 opencode run
- 每轮只修改 1 个文件或 1 个逻辑变更（Rust项目）
- 改完后必须跑 `cargo build` + `cargo test` 验证
- 编译错误卡在 3 次以上同一错误 → 停止，读全量错误定位根因

---

## 任务列表

### 1.1 clippy fix（12个warnings）

| 文件 | warning | 修复方式 |
|------|---------|----------|
| src/channel/desktop.rs:3 | empty_line_after_doc_comments | `///` → `//!` inner doc comment |
| src/core/agent_loop.rs:45 | dead_code: field `event_bus` | 加 `#[allow(dead_code)]` + 注释 |
| src/core/workflow_builder.rs:28 | dead_code: field `registry` | 同上 |
| src/channel/cli.rs:10 | type_complexity | 加 `type ChatFn = ...` 别名 |
| src/core/approval.rs:24 | from_str 混淆 | 改名 `from_str_value` |
| src/core/code_tool.rs:109 | ptr_arg: &PathBuf | `&PathBuf` → `&Path` |
| src/core/kanban/columns.rs:21,58 | from_str 混淆×2 | 改名 + 更新 caller |
| src/core/privacy_gate.rs:19 | from_str 混淆 | 改名 + 更新 caller |
| src/core/storage/governance.rs:123 | too_many_arguments(8) | 用 args struct 包装 |
| src/core/storage/oauth.rs:6 | too_many_arguments(8) | 同上 |
| src/org/permissions.rs:12 | from_str 混淆 | 改名 + 更新 caller |

验证：`cargo clippy --lib` → 0 warnings

### 1.2 computer/ 加测试（7文件零覆盖）

每个文件追加 `#[cfg(test)] mod tests { ... }`，测试所有 pub fn：
- mod.rs（3测试）：SecurityLevel、ComputerOpResult
- desktop_ops.rs（9测试）：鼠标/键盘/剪贴板/截图
- sys_ops.rs（8测试）：壁纸/音量/网络/电源
- browser_ops.rs（8测试）：填表/导航/多标签
- app_ops.rs（4测试）：启动/关闭/列表/安装
- fs_ops.rs（7测试）：读写/移动/删除/搜索/压缩
- perception.rs（6测试）：可达性/截图/OCR/编码

**不改原逻辑，不加 mock，纯模拟路径断言。**

验证：`cargo test --lib` → 全部通过

### 1.3 dead_code 审查注释（28处）

28处以 `#[allow(dead_code)]` 标注的每处加注释 `/* 预留：xx功能 */` 说明保留理由。**不删任何代码。**

验证：`cargo build` → 通过

---

## 验证命令

```bash
cargo clippy --lib  # 0 warnings
cargo build         # 0 errors
cargo test --lib    # 全部通过
```
