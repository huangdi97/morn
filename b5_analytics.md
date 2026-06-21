# Batch 5 — 分析面板

AnalyticsPanel 前端已有，需要后端数据管道 + 接到 Console 导航。

## 任务列表

### T1: 分析数据后端

文件：`src-tauri/src/commands/analytics.rs`（新建）

分析面板需要的数据：

```rust
#[derive(Serialize)]
pub struct AnalyticsData {
    pub daily_calls: Vec<DailyCallStat>,     // 每日调用次数趋势
    pub daily_tokens: Vec<DailyTokenStat>,   // 每日 token 消耗趋势
    pub top_agents: Vec<AgentStat>,          // 最活跃 Agent 排行
    pub error_rates: Vec<ErrorRateStat>,     // 错误率趋势
    pub avg_latency: Vec<LatencyStat>,       // 平均延迟趋势
    pub active_users: u64,                   // 活跃用户数
    pub total_executions: u64,               // 总执行次数
}

#[tauri::command]
pub(crate) fn get_analytics_data(days: u64) -> Result<AnalyticsData, CommandError> {
    // 从 executions 表和 daily_costs 表聚合数据
    // 按天分组
    // 返回近 days 天的趋势
}
```

SQL 查询参考：
```sql
-- 每日调用次数
SELECT DATE(created_at) as date, COUNT(*) as count FROM executions
WHERE created_at >= strftime('%s', 'now', ?) GROUP BY date;

-- 每日 token 消耗（从 daily_costs 表）
SELECT date, SUM(token_count) as tokens, SUM(cost_usd) as cost FROM daily_costs
WHERE date >= DATE('now', ?) GROUP BY date;

-- 最活跃 Agent
SELECT agent_id, COUNT(*) as calls, AVG(latency_ms) as avg_latency
FROM executions GROUP BY agent_id ORDER BY calls DESC LIMIT 10;

-- 错误率趋势
SELECT DATE(created_at) as date,
    SUM(CASE WHEN status = 'error' OR status = 'failed' THEN 1 ELSE 0 END) as errors,
    COUNT(*) as total
FROM executions GROUP BY date;
```

在 `src-tauri/src/lib.rs` 注册命令。

### T2: AnalyticsPanel 对接

文件：`web/src/console/AnalyticsPanel.tsx`

当前：前端组件存在，但需要确认它调用了正确的 Tauri invoke。

如果它已经是 `invoke('get_analytics_data', { days: 30 })` 并且渲染图表，就不需要改。如果写死了模拟数据，就改为调后端。

### T3: 接到 Console 导航

文件：`web/src/App.tsx`

检查 AnalyticsPanel 是否已在 Console tab 中：
- `consoleTab` 类型中已有 `"analytics"` 吗？
- nav 按钮已有吗？
- 渲染 `<AnalyticsPanel />` 了吗？

如果已有 → 跳过。
如果没有 → 补上（同 B1 T7 的 pattern）。

## 验证

- `cargo check -p morn` ✅
- `npm run build` ✅
- `tsc --noEmit` ✅
- AnalyticsPanel 显示真实执行数据而非空状态
