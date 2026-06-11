# Morn 管理台 (Console)

> 系统监控 · 拓扑可视化 · 成本中心 · 治理策略

## 仪表盘 (Dashboard.tsx)

三个核心面板：

| 面板 | 显示内容 |
|------|---------|
| EventStreamView | 实时事件流、Agent 活动、任务进度 |
| TaskProgress | 运行/排队/完成的任务统计 |
| SystemStatus | 系统健康度、资源占用、错误率 |
| AlertPanel | 告警事件列表 |

### DashboardData 字段

| 字段 | 描述 |
|------|------|
| request_trend | 请求量趋势数据 |
| latency_trend | 延迟趋势数据 |
| alerts | 告警事件列表 |

## 拓扑可视化 (Topology.tsx)

组件/Agent 之间的依赖和调用关系图：

- Agent ↔ Tool 调用关系
- Agent ↔ Knowledge 关联
- Agent → Agent 协作链路
- DAG 工作流执行路径

## 3D 可视化 (Visualization3D)

Agent 调用链和工作流执行路径渲染为 3D 力导向图：

| 元素 | 3D 表示 |
|------|---------|
| 节点 (Node) | Agent / 任务 / 数据流 |
| 边 (Edge) | 调用关系 / 数据流向 |
| 权重 | 边粗细表示调用频次 |

## 成本中心 (CostCenter.tsx)

LLM API 成本追踪：

| 指标 | 描述 |
|------|------|
| Token 消耗 | 各模型/Agent 消耗量 |
| 费用统计 | 按时间/Agent/用户聚合 |
| 预算管理 | check_budget、设置配额和告警阈值 |
| 成本趋势 | 日/周/月趋势图 |
| BudgetDecision | 超限决策枚举（继续/降级/暂停） |

## 信任评分 (TrustScorer)

Agent 信任评级系统：

| 等级 | 评分范围 | 权限 |
|------|---------|------|
| A+ | 90-100 | 完全自主 |
| A | 80-89 | 自动执行 |
| B | 60-79 | 需通知 |
| C | 40-59 | 需审批 |
| D | 0-39 | 需人工全程监督 |

评分基于：任务成功率、用户反馈、合规记录、历史表现。

## 治理策略 (Governance.tsx)

安全和合规配置：

| 功能 | 描述 |
|------|------|
| 权限管理 | 用户/角色/Agent 权限矩阵 |
| 审计日志 | 所有操作记录和查询 |
| 隐私策略 | 数据过滤和脱敏规则 |
| 安全策略 | Dual-LLM / 4 层宪法配置 |

## 系统信息 (SystemInfo.tsx)

| 信息 | 来源 |
|------|------|
| 版本 | Cargo.toml version |
| 运行时间 | uptime |
| 内存/CPU | 系统 API |
| SQLite 状态 | 数据库大小/表统计 |
| API 延迟 | 最近请求耗时统计 |
