# Batch 2 — 用户可见记忆

将 7 层内存系统持久化到 SQLite，Tauri 命令真实化，MemoryManager 面板显示真实数据。

## 任务列表

### T1: 记忆持久化表

文件：`src/core/storage/mod.rs`

```sql
CREATE TABLE IF NOT EXISTS memory_entries (
    id TEXT PRIMARY KEY,
    layer TEXT NOT NULL,             -- 'working' | 'episodic' | 'semantic' | 'experiential' | 'graph' | 'flash' | 'long_term'
    agent_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    metadata TEXT DEFAULT '{}',      -- JSON
    priority REAL DEFAULT 0.0,
    expires_at INTEGER,             -- unix timestamp, NULL = 不过期
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_memory_agent ON memory_entries(agent_id);
CREATE INDEX IF NOT EXISTS idx_memory_layer ON memory_entries(layer);
```

### T2: Storage CRUD for memory

文件：`src/core/storage/memory.rs`（新建）

```rust
impl Storage {
    pub fn list_memories(&self, agent_id: Option<&str>, layer: Option<&str>, limit: u64) -> Result<Vec<MemoryEntry>, MornError>;
    pub fn search_memories(&self, query: &str, agent_id: Option<&str>) -> Result<Vec<MemoryEntry>, MornError>;
    pub fn delete_memory(&self, id: &str) -> Result<(), MornError>;
    pub fn store_memory(&self, entry: &MemoryEntry) -> Result<(), MornError>;
    pub fn get_memory_layers(&self) -> Result<Vec<String>, MornError>;  // 所有有数据的 layer 名
}
```

### T3: 真实化 Tauri 命令

文件：`src-tauri/src/commands/memory.rs`

当前：全是存根（`list_memories` 返回空 Vec，`delete_memory` 返回 "deleted"）。

改为真实调用 Storage：
```rust
#[tauri::command]
pub(crate) fn list_memories(
    state: State<AppState>,
    agent_id: Option<String>,
    layer: Option<String>,
    limit: Option<u64>,
) -> Result<Vec<MemoryEntry>, CommandError> {
    let storage = state.storage.lock().map_err(|e| ...)?;
    storage.list_memories(agent_id.as_deref(), layer.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.into())
}

#[tauri::command]
pub(crate) fn search_memories(...) { ... }

#[tauri::command]
pub(crate) fn delete_memory(...) { ... }
```

命令需要接入 AppState（当前是纯函数），加 `State<AppState>` 参数。

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册命令。

### T4: MemoryManager 前端

文件：`web/src/console/MemoryManager.tsx`

当前：UI 已完善（列表、类型过滤、搜索、展开详情、删除确认），但后端返回空所以显示为空状态。

只需确认：
1. `list_memories` 参数传递正确（agent_id、layer、limit）
2. `search_memories` 搜索框触发调用
3. `delete_memory` 删除后刷新列表

不需要改 UI 代码 — 它已经调对了 invoke 命令。

### T5: 验证

- `cargo check -p morn` ✅
- `cargo test --lib` 全部通过
- 前端 MemoryManager 面板显示记忆条目（至少非空）
- 搜索/过滤/删除功能正常
