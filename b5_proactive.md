# Batch 5 — 主动 Agent

让 ProactiveEngine 真实运行：存储持久化 + Tauri 命令真实化 + 运行时集成。

## 编码准则

1. 【Think Before Coding】先读懂现有 ProactiveEngine 再动手
2. 【增量叠加】不改现有 Supervisor/Scheduler
3. 【自底向上】先存储层，再 Tauri 命令，再运行时集成

## 预备知识

现有代码：
- `src/core/proactive/mod.rs` — ✅ ProactiveEngine (register/tick/check_ready) + 3 测试 (166行)
- `src-tauri/src/commands/proactive.rs` — ❌ stub，返回 3 条硬编码规则
- `web/src/console/ProactivePanel.tsx` — ✅ 良好 UI (66行)
- EventBus ✅ 已有 (SimpleEventBus)
- Cron 系统 ✅ 已有

## 任务列表

### T1: Storage 层 — proactive_rules 表

当前：规则不持久化，Tauri 命令返回硬编码数据。

在 `src/core/storage/mod.rs` 的 `fn create_tables` 中添加新表：
```sql
CREATE TABLE IF NOT EXISTS proactive_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,     -- "timer" | "event"
    trigger_config TEXT NOT NULL,   -- JSON: timer 存间隔秒数, event 存事件类型
    action TEXT NOT NULL,           -- 执行的动作描述
    enabled INTEGER NOT NULL DEFAULT 1,
    last_triggered_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
```

新建 `src/core/storage/proactive.rs`：
```rust
pub struct ProactiveRule {
    pub id: String,
    pub name: String,
    pub trigger_type: String,
    pub trigger_config: String,  // JSON
    pub action: String,
    pub enabled: bool,
    pub last_triggered_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Storage {
    pub fn list_proactive_rules(&self) -> Result<Vec<ProactiveRule>, MornError> { ... }
    pub fn get_proactive_rule(&self, id: &str) -> Result<Option<ProactiveRule>, MornError> { ... }
    pub fn create_proactive_rule(&self, rule: &ProactiveRule) -> Result<(), MornError> { ... }
    pub fn update_proactive_rule(&self, rule: &ProactiveRule) -> Result<(), MornError> { ... }
    pub fn delete_proactive_rule(&self, id: &str) -> Result<(), MornError> { ... }
    pub fn toggle_proactive_rule(&self, id: &str, enabled: bool) -> Result<(), MornError> { ... }
}
```

在 `src/core/storage/mod.rs` 中声明 `pub mod proactive;`

### T2: ProactiveEngine 扩展 — 接入 Storage

在 `src/core/proactive/mod.rs` 中：
1. `ProactiveEngine` 新增 `storage: Option<Arc<Storage>>` 字段
2. `new()` 接收可选的 Storage 引用
3. `load_rules()` 方法 — 从 Storage 读取所有 enabled 规则
4. `save_rule()` / `delete_rule()` 方法（双向同步 DB 和内存）

```rust
pub struct ProactiveEngine {
    agents: HashMap<String, ProactiveAgent>,
    storage: Option<Arc<Storage>>,
}

impl ProactiveEngine {
    pub fn new(storage: Option<Arc<Storage>>) -> Self {
        let mut engine = ProactiveEngine { agents: HashMap::new(), storage };
        engine.load_rules();
        engine
    }
    
    pub fn load_rules(&mut self) {
        if let Some(ref storage) = self.storage {
            if let Ok(rules) = storage.list_proactive_rules() {
                for rule in rules {
                    if rule.enabled {
                        // 将 rule 转为 ProactiveAgent 注册
                        self.agents.insert(rule.id.clone(), ProactiveAgent { ... });
                    }
                }
            }
        }
    }
}
```

### T3: 真实化 Tauri 命令（commands/proactive.rs）

将当前硬编码的 3 条规则改为从 Storage 读取：

```rust
#[tauri::command]
pub(crate) fn list_proactive_rules(state: State<AppState>) -> Result<Vec<ProactiveRule>, CommandError> {
    let storage = state.storage.lock().map_err(...)?;
    storage.list_proactive_rules().map_err(|e| e.into())
}

#[tauri::command]
pub(crate) fn create_proactive_rule(
    state: State<AppState>,
    name: String,
    trigger_type: String,
    trigger_config: String,
    action: String,
) -> Result<ProactiveRule, CommandError> { ... }

#[tauri::command]  
pub(crate) fn toggle_proactive_rule(
    state: State<AppState>,
    rule_id: String,
    enabled: bool,
) -> Result<(), CommandError> {
    let mut storage = state.storage.lock().map_err(...)?;
    storage.toggle_proactive_rule(&rule_id, enabled).map_err(|e| e.into())?;
    
    // 同步更新 engine
    if let Some(ref engine) = *state.proactive_engine.lock().map_err(...)? {
        if enabled {
            engine.register(...);
        } else {
            engine.remove(&rule_id);
        }
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn delete_proactive_rule(
    state: State<AppState>,
    rule_id: String,
) -> Result<(), CommandError> { ... }
```

### T4: AppState 接线

`src-tauri/src/lib.rs` 中 AppState 新增：
```rust
pub proactive_engine: Arc<Mutex<ProactiveEngine>>,
```

`setup()` 中：
```rust
let proactive = ProactiveEngine::new(Some(storage.clone()));
state.proactive_engine = Arc::new(Mutex::new(proactive));
```

命令需要改为接收 AppState（当前是纯函数）：
```rust
// 原来：
pub(crate) fn list_proactive_rules() -> ...
// 改为：
pub(crate) fn list_proactive_rules(state: State<AppState>) -> ...
```

### T5: 运行时集成

在 `src-tauri/src/lib.rs` 的 `setup()` 中启动后台 tick 线程：

```rust
// 每 60 秒检查一次主动规则
let engine = state.proactive_engine.clone();
std::thread::spawn(move || {
    loop {
        std::thread::sleep(Duration::from_secs(60));
        if let Ok(mut engine) = engine.lock() {
            let ready = engine.tick();
            for agent_id in ready {
                // 找到对应的 rule action
                // 如果 action 含 cron 前缀，调度定时任务
                // 如果 action 是事件类型，推送到 chat
                tracing::info!("Proactive rule triggered: {}", agent_id);
            }
        }
    }
});
```

当前 `tick()` 返回 `Vec<String>`（普通 agent ID）。需要检查返回类型是否匹配。

### T6: 前端 ProactivePanel 增强

当前 `ProactivePanel.tsx`：显示列表 + 切换按钮。需增强：

1. 从 `list_proactive_rules` 读取真实数据
2. 添加"新建规则"表单：
   - 名称输入
   - 触发类型选择（定时器/事件）
   - 触发配置（定时器=间隔秒数，事件=事件类型名）
   - 动作描述
3. 规则列表显示：名称/触发类型/状态/最后触发时间
4. 删除按钮
5. 调用 `create_proactive_rule` / `delete_proactive_rule` Tauri 命令

### T7: 验证

```bash
cargo check -p morn
cargo test --lib
npm run build
```

## 验证门禁

- `cargo check -p morn` ✅
- `cargo test --lib` 全部通过
- `cargo clippy -p morn` 0 warnings
- 前端能创建/查看/启用/禁用/删除规则
- 规则在重启后持续存在（DB 持久化）
- `npm run build` ✅
