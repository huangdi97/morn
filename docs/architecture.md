# 系统架构

## 四层架构

```
┌──────────────────────────────────────────────────────────┐
│                      接入层                               │
│  Desktop UI | CLI | Telegram | 企微 | 钉钉 | 飞书 | REST │
│  QQ Bot | 微信小程序 | 微信公众号 | Webhook | SMTP        │
├──────────────────────────────────────────────────────────┤
│                    COO 主管                               │
│  1. Intent Parser (NL → 结构化意图)                       │
│  2. Planner (意图分解为 DAG 计划)                         │
│  3. Router (匹配组件 + 信任评分加权)                      │
│  4. Scheduler (DAG 调度)                                  │
│  6 级决策树: 直接答→单工具→Agent→团队→模板→创作台        │
├──────────────────────────────────────────────────────────┤
│                   组件体系                                │
│  6 类原子组件: Tool / Knowledge / Skill / Persona        │
│  / Memory / Model                                         │
│  可自由组合成 Agent → 组合成团队 → 发布到市场             │
├──────────────────────────────────────────────────────────┤
│                    存储层                                  │
│  SQLite (agents / capabilities / tasks / subtasks         │
│  / executions / decisions / bindings)                     │
└──────────────────────────────────────────────────────────┘
```

## COO 主管 6 级决策树

`src/core/supervisor.rs` 中的 `decide_level()` 方法按优先级依次匹配：

| 级别 | 名称 | 触发条件 | 成本 |
|------|------|----------|------|
| L1 | DirectAnswer | 问候、感谢、简单知识问答 | ¥0.001 / 0.5s |
| L2 | SingleTool | 搜索、计算、翻译等单工具操作 | ¥0.003 / 1s |
| L3 | SingleAgent | 默认级别，需要单 Agent 分析 | ¥0.02 / 5s |
| L4 | Team | 复杂多维任务 | ¥0.05 / 15s |
| L5 | Workflow | 标准工作流模板 | ¥0.03 / 10s |
| L6 | JumpToStudio | 创建/修改组件或工作流 | variable |

## 组件 Trait 体系

`src/core/component.rs` 定义了三个核心 Trait：

- **`Component`** — 基础生命周期：`init` → `run` → `pause` → `stop`
- **`IOComponent`** — 端口通信：`ports()` / `send()` / `recv()`
- **`SecureComponent`** — 权限声明：`required_permissions()`

另有 `ComponentType` 枚举标识 8 种类型：Tool、Knowledge、Skill、Persona、Memory、Model、Agent、Pipeline。

## 安全宪法四层模型

`src/core/security.rs` 中的 4 级安全策略：

| 级别 | 含义 | 示例 |
|------|------|------|
| L1 HardBlocked | 硬拦截 | 格式化磁盘、删除系统文件、修改注册表 |
| L2 NeedApproval | 需用户审批 | 执行 Shell 命令、写工作区外文件 |
| L3 NeedNotify | 需通知用户 | 读工作区外文件、访问未注册域名 |
| L4 Free | 自由执行 | 聊天、搜索、读工作区文件、调用注册 API |

Dual-LLM（`src/core/dual_llm.rs`）提供 6 个检查点：Auth → ParamValidate → ContentSanitize → Permission → Audit → Route。

## 数据流

```
用户输入 → ChannelAdapter → Supervisor.decide()
  → DecisionLevel → TaskPlan (DAG)
  → TaskEngine.run_plan() / run_dag_plan()
  → 执行 subtasks → 汇总 TaskResult → 输出响应
```

## 事件总线

`src/core/event_bus.rs` 的 `SimpleEventBus` 支持发布 / 订阅模式。预定义事件：

- `supervisor.plan.created` — 计划创建
- `supervisor.plan.executing` — 计划执行中
- `supervisor.task.completed` — 任务完成
- `supervisor.task.failed` — 任务失败
- `chat_agent.response` — LLM 响应
- `system.ready` — 系统就绪