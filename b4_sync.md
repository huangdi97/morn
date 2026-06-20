# Batch 4 — 跨设备同步

SyncEngine 真实接线 + 后端 API 端点，实现两台设备间的数据同步。

## 编码准则

1. 【Think Before Coding】先读懂现有 SyncEngine + sync events 存储层
2. 【增量叠加】不改现有非同步代码
3. 【最小依赖】使用已有 reqwest crate 做 HTTP 通信
4. 【安全】同步数据需加密传输

## 预备知识

现有代码：
- `src/core/storage/sync.rs` — ✅ 完整 CRUD (258行, 含 tests)
  - `get_unsynced_events()`, `mark_events_synced()`, `insert_sync_event()`, `clear_synced_events()`
  - `save_device()`, `list_devices()`, `remove_device()`
- `src/bridge/sync/mod.rs` — ✅ SyncEngine 编排 (402行)
  - `sync_once()`, `push_events()`, `pull_events()`, `resolve_conflicts()`, `start_sync_loop()`
  - HTTP 客户端已实现（调用外部 API）
- `src/bridge/sync/events.rs` — ✅ 冲突解决 (136行)
  - 按时间戳 last-writer-wins
- `web/src/console/SyncPanel.tsx` — ⚠️ 有 UI 但只调 sync_now
- `src-tauri/src/commands/market.rs` — ⚠️ sync_now 仅本地标记
- 后端 API: ❌ 无 /sync/push 和 /sync/pull 端点
- AppState: ❌ 无 SyncEngine

## 任务列表

### T1: 后端 API 服务端端点

在 `src/channel/rest_api.rs`（或新建 `src/bridge/sync/server.rs`）中添加：

```rust
// POST /sync/push
// 接收事件数组，按 last-writer-wins 合并
async fn sync_push(events: Vec<SyncEvent>) -> Result<Vec<SyncEvent>, ...> {
    for event in events {
        // 查询是否有冲突事件（同 entity_type + entity_id）
        // 如果本地事件更新 → 保留本地
        // 如果远程事件更新 → 替换本地
        // 记录合并结果
    }
    // 返回本设备需要的远程事件
}

// GET /sync/pull?since=<timestamp>&device_id=<id>
// 返回自 since 以来的所有未同步事件
async fn sync_pull(since: i64, device_id: String) -> Result<Vec<SyncEvent>, ...> {
    // 查询 sync_events 表中 timestamp > since 且 device_id != 请求设备
}
```

如果 `rest_api.rs` 还没有 axum 路由，需要先初始化 axum server。

**检查现状：** `src/channel/rest_api.rs` 中已有 REST API 服务器吗？如果有，在其基础上添加路由。如果没有，创建一个轻量 axum 服务器（或使用已有的 HTTP 服务框架）。

### T2: SyncEngine 接线到 AppState

`src-tauri/src/lib.rs` 中 AppState 新增：
```rust
pub sync_engine: Arc<Mutex<Option<SyncEngine>>>,
```

`setup()` 中初始化：
```rust
let sync_engine = SyncEngine::new(storage.clone(), server_url);
state.sync_engine = Arc::new(Mutex::new(Some(sync_engine)));
```

`server_url` 从设置读取（默认 `http://localhost:3000`）。

### T3: 启动后台同步线程

在 `setup()` 中：
```rust
let sync_engine = state.sync_engine.clone();
std::thread::spawn(move || {
    loop {
        std::thread::sleep(Duration::from_secs(300)); // 每5分钟
        if let Ok(mut engine) = sync_engine.lock() {
            if let Some(ref mut engine) = *engine {
                if let Err(e) = engine.sync_once() {
                    tracing::warn!("Sync failed: {}", e);
                }
            }
        }
    }
});
```

### T4: 真实化 sync_now 命令

当前在 `src-tauri/src/commands/market.rs` 中：

```rust
#[tauri::command]
fn sync_now(state: State<AppState>) -> Result<String, MornError> {
    // 简化实现：mark all as synced
    // ...
}
```

改为调用 SyncEngine：
```rust
#[tauri::command]  
fn sync_now(state: State<AppState>) -> Result<String, MornError> {
    let mut engine = state.sync_engine.lock()
        .map_err(|e| MornError::Internal(format!("lock error: {}", e)))?;
    if let Some(ref mut engine) = *engine {
        let result = engine.sync_once()?;
        Ok(format!("Synced {} events", result))
    } else {
        Err(MornError::Internal("Sync engine not initialized".into()))
    }
}
```

### T5: 前端 SyncPanel 增强

`web/src/console/SyncPanel.tsx` 当前调用 `sync_now` 返回时间戳。

增强为：
1. 显示当前设备 ID
2. 显示已注册设备列表（从 `list_devices` 后端读取）
3. 显示待同步事件数
4. 远程服务器 URL 配置输入框（保存到 settings）
5. 同步状态指示器（上次同步时间、是否成功）
6. 手动触发同步按钮

需要新增 Tauri 命令：
- `list_sync_devices()` — 返回注册设备列表
- `get_sync_status()` — 返回待同步事件数 + 最后同步时间
- `set_sync_server_url(url)` — 保存服务器 URL

### T6: 验证

```bash
cargo check -p morn
cargo test --lib
npm run build
```

## 验证门禁

- `cargo check -p morn` ✅
- `cargo test --lib` 全部通过
- `cargo clippy -p morn` 0 warnings
- SyncEngine 在 AppState 中初始化
- 生产代码中 start_sync_loop 被调用
- 前端 SyncPanel 显示设备列表和事件数
- `npm run build` ✅
