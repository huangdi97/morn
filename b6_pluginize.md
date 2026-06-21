# Batch 6 — 功能插件化

将零散挂载的功能归入标准 MornPlugin 生命周期，通过 PluginContext 统一管理。

## 设计

合并成 **3 个新插件**：

| 插件 ID | 职责 | 包含 | 依赖 | 优先级 |
|---------|------|------|------|--------|
| `morn:observability` | 可观测性 | 成本追踪 / 分析面板 / 可靠性 / 记忆可见 | data-layer | 175 |
| `morn:voice` | 语音 | Whisper 集成 / 音频设备 | data-layer | 170 |
| `morn:sync` | 跨设备同步 | SyncEngine 管理 | data-layer | 130 |

现有的 `ProactiveEngine` 已在 `SupervisorPlugin` 或 AppState 中接线，不另建插件。

## 任务列表

### T1: ObservabilityPlugin

文件：`src/core/plugin_manager/plugins/observability_plugin.rs`（新建）

```rust
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;
use std::sync::Arc;

pub struct ObservabilityPlugin(pub Option<Arc<Storage>>);

impl Default for ObservabilityPlugin { fn default() -> Self { Self::new() } }
impl ObservabilityPlugin { pub fn new() -> Self { Self(None) } }

impl MornPlugin for ObservabilityPlugin {
    fn id(&self) -> &str { "morn:observability" }
    fn deps(&self) -> Vec<&str> { vec!["morn:data-layer"] }
    fn priority(&self) -> i32 { 175 }
    
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage")
            .ok_or_else(|| PluginError::LoadFailed("morn:observability".into(), "morn:storage not found".into()))?;
        ctx.register("morn:observability", storage.clone());
        self.0 = Some(storage);
        Ok(())
    }
    
    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        // 验证 daily_costs 表存在
        let storage = ctx.get::<Storage>("morn:observability")
            .ok_or_else(|| PluginError::ActivateFailed("morn:observability".into(), "missing".into()))?;
        // 执行一次测试查询来验证表
        let _ = storage.get_cost_summary(30);
        Ok(())
    }
    
    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
```

### T2: VoicePlugin

文件：`src/core/plugin_manager/plugins/voice_plugin.rs`（新建）

```rust
pub struct VoicePlugin;
impl MornPlugin for VoicePlugin {
    fn id(&self) -> &str { "morn:voice" }
    fn deps(&self) -> Vec<&str> { vec!["morn:data-layer"] }
    fn priority(&self) -> i32 { 170 }
    fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
}
```

VoicePlugin 主要是生命周期标记——实际功能在 Tauri 命令中（whisper.rs），它们通过 AppState 访问存储。

### T3: SyncPlugin

文件：`src/core/plugin_manager/plugins/sync_plugin.rs`（新建）

```rust
use crate::bridge::sync::SyncEngine;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;
use std::sync::{Arc, Mutex};

pub struct SyncPlugin(pub Option<Arc<Mutex<SyncEngine>>>);

impl Default for SyncPlugin { fn default() -> Self { Self::new() } }
impl SyncPlugin { pub fn new() -> Self { Self(None) } }

impl MornPlugin for SyncPlugin {
    fn id(&self) -> &str { "morn:sync" }
    fn deps(&self) -> Vec<&str> { vec!["morn:data-layer"] }
    fn priority(&self) -> i32 { 130 }
    
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage")
            .ok_or_else(|| PluginError::LoadFailed("morn:sync".into(), "morn:storage not found".into()))?;
        // 从 settings 读取 server_url
        let server_url = storage.get_setting("sync_server_url")
            .unwrap_or_else(|| "http://localhost:3000".to_string());
        let engine = SyncEngine::new(storage, &server_url);
        let shared = Arc::new(Mutex::new(engine));
        ctx.register("morn:sync-engine", shared.clone());
        self.0 = Some(shared);
        Ok(())
    }
    
    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<Mutex<SyncEngine>>>("morn:sync-engine")
            .ok_or_else(|| PluginError::ActivateFailed("morn:sync".into(), "engine missing".into()))?;
        Ok(())
    }
    
    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
```

### T4: 注册到 plugin 列表

文件：`src/core/plugin_manager/manager.rs` 中的 `load_plugins()` 函数（或 `run()` 中的 plugin vec）

将新插件加入 vec：
```rust
pub fn load_plugins() -> Vec<Box<dyn MornPlugin>> {
    vec![
        Box::new(DataLayerPlugin::new()),
        Box::new(RegistryPlugin::new()),
        Box::new(SandboxPlugin::new()),
        Box::new(EnginePlugin::new()),
        Box::new(ChannelBusPlugin::new()),
        Box::new(SupervisorPlugin::new()),
        Box::new(StudioPlugin::new()),
        Box::new(BridgePlugin::new(plugin_dir)),
        // 新插件
        Box::new(ObservabilityPlugin::new()),
        Box::new(VoicePlugin::new()),
        Box::new(SyncPlugin::new()),
    ]
}
```

需要找到实际加载插件的代码位置并修改。搜索 `load_plugins` 或 plugin vec 创建处。

### T5: AppState 从 PluginContext 读取

文件：`src-tauri/src/lib.rs`

当前 AppState 直接构造各个字段。改为接收 PluginContext 后从中读取：

```rust
pub struct AppState {
    // 现有字段保持（从 PluginContext 读取）
    pub storage: Arc<Mutex<Storage>>,
    // ...
    // 新插件字段
    pub sync_engine: Option<Arc<Mutex<SyncEngine>>>,
}

impl AppState {
    pub fn from_ctx(ctx: &PluginContext) -> Self {
        let storage = ctx.get::<Storage>("morn:storage")
            .expect("morn:storage must be registered");
        let sync_engine = ctx.get::<Arc<Mutex<SyncEngine>>>("morn:sync-engine");
        // ...
        AppState { storage, sync_engine, ... }
    }
}
```

### T6: 验证

- `cargo check -p morn` ✅
- `cargo test --lib` 全部通过
- `cargo clippy -p morn` 0 warnings
- 所有功能通过插件生命周期初始化
- 启动日志显示新插件 init/activate 成功
