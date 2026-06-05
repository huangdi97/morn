# COO Supervisor 文档

## COO 是什么

COO（Chief Operating Officer）是 Morn 的主管大脑，负责理解用户意图、制定执行计划、调度资源、记录决策日志。核心逻辑在 `src/core/supervisor.rs`。

## 6 级决策树

`Supervisor::decide_level()` 按优先级依次匹配关键词：

| 级别 | 名称 | 触发关键词 | 成本 |
|------|------|------------|------|
| L1 | DirectAnswer | hello, hi, thanks, bye, who are you | ¥0.001 / 0.5s |
| L2 | SingleTool | search, look up, find, calculate, convert, translate | ¥0.003 / 1s |
| L3 | SingleAgent | 默认级别（以上均不匹配时） | ¥0.02 / 5s |
| L4 | Team | complex, multi-step, multiple, comprehensive, full | ¥0.05 / 15s |
| L5 | Workflow | report, analysis, research, investigate, plan, strategy | ¥0.03 / 10s |
| L6 | JumpToStudio | create an agent, build an agent, configure, customize | variable |

每个级别有对应的 `cost_tier()` 返回估算开销描述。

## CooMode

`src/core/supervisor.rs::CooMode` 枚举三种运行模式：

| 模式 | 含义 |
|------|------|
| `Active` | 默认模式，计划自动执行，不要求审批 |
| `Safe` | 安全模式，执行前输出计划预览并要求确认 |
| `Auto` | 自动模式 |

通过 CLI 命令 `/mode active|safe|auto` 切换。

## 工作流引擎

`src/core/workflow.rs` 定义了 `WorkflowTemplate` 和 `WorkflowStep`，包含 16 种 `WorkflowAction`：

LLMCall、ToolCall、AgentCall、TeamCall、SubWorkflow、CodeExec、KnowledgeQuery、HumanApproval、HumanInput、Notification、Condition、Loop、Wait、Fork、Join

8 个内建工作流模板：

| ID | 名称 | 类别 | 步骤数 |
|----|------|------|--------|
| workflow-task-execution | Task Execution | general | 6 |
| workflow-deep-analysis | Deep Analysis | research | 4 |
| workflow-news-monitor | News Monitor | monitoring | 5 |
| workflow-report-gen | Report Generation | reporting | 6 |
| workflow-code-delivery | Code Delivery | development | 7 |
| workflow-product-launch | Product Launch | product | 6 |
| workflow-decision-eval | Decision Evaluation | strategy | 6 |
| workflow-scheduled-inspection | Scheduled Inspection | operations | 4 |

## 信任评分机制

`src/core/trust_evaluator.rs` 的四层评估模型：

```
overall = output_quality * 0.3 + trace_score * 0.3
        + component_quality * 0.2 + user_feedback * 0.2
```

- **OutputQuality** — 内容相关性、格式合规性、完整性
- **TraceQuality** — 调用链完整性、错误率、重试次数
- **ComponentQuality** — 初始化成功率、正常运行时间、资源效率
- **DriftQuality** — 近期性能趋势漂移检测

`Registry::update_trust_score()` 同步更新注册中心的能力信任分。