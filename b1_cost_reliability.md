# Batch 1 — 成本透明 + 可靠性指标（基础设施层）

从最底层开始：建 token_count 列、daily_costs 表、ObservabilityManager 接 LLM 调用、ReliabilityPanel 接入 UI

## 编码准则

1. 【最小改动】不破坏现有存储/路由逻辑
2. 【增量叠加】只加不删
3. 【错误处理】所有 DB 操作返回 Result，不使用 unwrap

## 任务列表

### T1: executions 表加 token_count 列

文件：`src/core/storage/mod.rs`

在 `create_tables` 的 executions 表定义中加：
```sql
-- 原表
executions (id, agent_id, task_id, action, status, latency_ms, error_msg, created_at)
-- 改为
executions (id, agent_id, task_id, action, status, latency_ms, error_msg, token_count INTEGER DEFAULT 0, created_at)
```

用 `ALTER TABLE IF NOT EXISTS` 或 `CREATE TABLE IF NOT EXISTS` 保证迁移安全。

### T2: 建 daily_costs 表

文件：`src/core/storage/mod.rs`

```sql
CREATE TABLE IF NOT EXISTS daily_costs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,              -- '2024-01-01' 格式
    agent_id TEXT NOT NULL,
    model TEXT NOT NULL,
    token_count INTEGER NOT NULL DEFAULT 0,
    cost_usd REAL NOT NULL DEFAULT 0.0,
    call_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(date, agent_id, model)
);
```

### T3: Storage CRUD for daily_costs

文件：`src/core/storage/costs.rs`（新建）

```rust
impl Storage {
    pub fn record_call_cost(&self, agent_id: &str, model: &str, token_count: u64, cost_usd: f64) -> Result<(), MornError>;
    pub fn get_daily_costs(&self, date: &str) -> Result<Vec<DailyCostRow>, MornError>;
    pub fn get_agent_costs(&self, agent_id: &str, days: u32) -> Result<Vec<DailyCostRow>, MornError>;
    pub fn get_total_cost(&self, days: u32) -> Result<f64, MornError>;
    pub fn get_cost_summary(&self, days: u32) -> Result<CostSummary, MornError>;
}
```

在 `src/core/storage/mod.rs` 声明 `pub mod costs;`

### T4: ObservabilityManager 接 LLM 调用

文件：`src/core/observability.rs`

当前：`record_token_usage()` 仅在内存中，无持久化。

1. 给 `ObservabilityManager` 加 `storage: Option<Arc<Storage>>` 字段
2. 在 `record_token_usage()` 中调用 `storage.record_call_cost()`
3. 在 `start_span()`/`end_span()` 中写 `executions` 表（含 `latency_ms` 和 `token_count`）

LLM 调用路径追踪：
- `end_span(name)` 时：从 `active_spans` 取出 span，计算耗时
- 如果 span 有 token 数据 → 调 `record_call_cost`
- 同时写一条 `executions` 记录

### T5: 替换硬编码成本后端

文件：`src-tauri/src/commands/cost.rs`

当前 `get_cost_summary` 通过 `调用次数 × 500 tokens × $0.005/1k` 估算。

改为从 `Storage::get_cost_summary()` 读取真实数据。

`get_cost_details` 命令不存在（当前搜索结果为 0），新增它并调用 `get_agent_costs`。

### T6: CostCenter 前端对接

文件：`web/src/console/CostCenter.tsx`

当前：期望从 `getSystemStatus` 获取 `agent_costs` 和 `daily_costs`。

改为直接调 Tauri 命令：
- `get_cost_summary` → 显示总成本和趋势
- `get_cost_details` → 显示各 Agent/模型细分

### T7: ReliabilityPanel 接入 UI

文件：`web/src/App.tsx`

当前：ReliabilityPanel 有完整的后端 SQL 查询（`get_reliability_metrics`），但 Console tab 里没有它。

在 `consoleTab` 类型加 `"reliability"`，在 nav 加按钮，在渲染加：
```tsx
{consoleTab === "reliability" && <ReliabilityPanel />}
```

补 i18n key：`console_tab.reliability`

## 验证

- `cargo check -p morn` ✅
- `cargo test --lib` 全部通过
- `cargo clippy -p morn` 0 warnings
- `npm run build` ✅
- CostCenter 显示真实数据而非硬编码
- ReliabilityPanel 在 Console 导航中可见
