# 管理台文档

## Console 管理台是什么

`src/console/` 目录提供系统管理功能，包含三个模块：

- `mod.rs` — 仪表盘、拓扑、系统信息、安全日志、审计日志
- `cost.rs` — 成本监控
- `governance.rs` — 治理策略

## 仪表盘

`ConsoleBackend::get_dashboard()` 返回 `DashboardData`：

| 字段 | 说明 |
|------|------|
| total_tasks | 任务总数 |
| success_rate | 任务成功率（当前固定 0.95） |
| avg_latency_ms | 平均延迟 |
| today_cost | 今日成本 |
| agent_count | 已注册能力数 |
| active_channels | 活跃渠道数 |
| uptime_hours | 系统运行时间 |

`get_system_info()` 返回系统信息（CPU 使用率、内存、磁盘、OS）。

## 成本监控

`src/console/cost.rs::CostCenter` 管理预算和成本报告：

```rust
pub struct CostReport {
    pub total_cost: f64,
    pub by_agent: Vec<CostBreakdown>,   // 按 Agent 分解
    pub by_tool: Vec<CostBreakdown>,    // 按工具分解
    pub by_model: Vec<CostBreakdown>,   // 按模型分解
    pub daily_trend: Vec<DailyCost>,
    pub monthly_trend: Vec<MonthlyCost>,
    pub budget: f64,
    pub budget_exceeded: bool,
}
```

- `get_report()` — 获取当前成本报告
- `set_budget(budget)` — 设置预算上限
- `set_budget_action(action)` — 超预算行为（warn / block）

## 治理策略

`src/console/governance.rs::Governance` 管理：

| 功能 | 说明 |
|------|------|
| API Key 管理 | 添加 / 删除密钥，显示 masked key |
| 策略例外 | 添加 / 移除安全策略例外 |
| 渠道绑定 | 管理渠道 webhook 绑定 |
| 信任阈值 | 设置全局信任分阈值（0-100） |
| 审批队列 | 批准 / 拒绝待审批项 |
| 安全策略列表 | 从 SecurityGuard 读取当前策略 |

## 系统健康检查

`ConsoleBackend` 提供额外监控接口：

- `get_topology()` — 系统拓扑（已注册能力节点列表）
- `get_security_logs()` — 安全日志（认证、策略执行记录）
- `get_audit_log(limit)` — 审计日志（决策记录）