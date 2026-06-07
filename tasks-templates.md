# 拆分 workflow/templates.rs（659行）

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
- 模块间只传ID，不传对象
- 不 import 其他模块的内部函数
- 依赖倒置：模块通过接口/ID通信，不直接耦合

### 执行规则
- 不允许手动改代码文件
- 改完后必须跑 cargo build + cargo test 验证
- 不改任何 pub 接口签名，不加文档注释，不改.unwrap()

## 任务

拆分 src/core/workflow/templates.rs（659行）为子目录。

1. 创建 src/core/workflow/templates/ 目录
2. 创建 templates/mod.rs — 公共类型（WorkflowTemplate 等）+ pub use 重导出
3. 创建 templates/code_review.rs — code_review_workflow(), code_audit_workflow()
4. 创建 templates/deploy.rs — deploy_workflow(), ci_cd_pipeline_workflow()
5. 创建 templates/data_analysis.rs — data_pipeline_workflow(), report_gen_workflow(), research_workflow()
6. 创建 templates/support.rs — customer_support_workflow(), qa_workflow()
7. 更新 src/core/workflow/mod.rs — 加 pub mod templates; pub use templates::*;
8. 删除 src/core/workflow/templates.rs

## 验证
```bash
cargo build && cargo test --lib
```
