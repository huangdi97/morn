# Batch 3 — OAuth 端到端

完整 OAuth 授权码流程：Tauri 命令 → OAuth 握手 → 存储 token → callback 路由

## 编码准则

1. 【Think Before Coding】先读懂现有 OAuthManager 和 storage/oauth.rs 再动手
2. 【增量叠加】不破坏现有代码结构
3. 【最小改动】storage/oauth.rs 不动（已完整），主要改 Tauri 命令和前端
4. 【安全】不在前端暴露 client_secret

## 预备知识

现有代码结构：
- `src/core/storage/oauth.rs` — ✅ CRUD + schema + tests (172行)
- `src/core/oauth.rs` — ⚠️ OAuthManager 结构完整但 handle_callback 是 mock
- `src-tauri/src/commands/oauth.rs` — ❌ stub, 返回假 URL
- `web/src/console/Connections.tsx` — ⚠️ 有 UI 但无 callback 路由
- 无 oauth2 crate 依赖

OAuth 授权码流程：
1. 前端请求 authorize → 后端返回 provider 授权页 URL
2. 用户浏览器打开 URL → 同意授权 → provider 回调到 redirect_uri
3. 前端 callback 页面拿到 `?code=XXX&state=YYY`
4. 前端调用 `oauth_callback(code, state)` → 后端用 code 换 token
5. 后端存储 token 到 `oauth_tokens` 表
6. 前端显示已连接

## 任务列表

### T1: 引入 oauth2 crate 或使用 reqwest

方案：使用已有的 `reqwest` crate 做 HTTP token exchange，不引入新依赖。

需要的 HTTP 调用：
- `POST https://github.com/login/oauth/access_token` — 用 code 换 token
- `POST https://oauth2.googleapis.com/token` — Google token exchange
- `POST https://slack.com/api/oauth.v2.access` — Slack token exchange

### T2: 真实化 OAuthManager（core/oauth.rs）

当前 `handle_callback` 返回 mock：
```rust
fn handle_callback(&self, provider: &str, code: &str) -> Result<String, MornError> {
    Ok(format!("mock_{}_token_{}", provider, code))
}
```

改为真实 HTTP 请求：
```rust
fn handle_callback(&self, provider: &str, code: &str) -> Result<String, MornError> {
    let config = self.providers.get(provider)
        .ok_or_else(|| MornError::Internal(format!("unknown provider: {}", provider)))?;
    
    // 根据 provider 选择不同 endpoint
    let (token_url, params) = match provider {
        "github" => (
            "https://github.com/login/oauth/access_token",
            json!({"client_id": config.client_id, "client_secret": config.client_secret, "code": code})
        ),
        "google" => (
            "https://oauth2.googleapis.com/token",
            json!({"client_id": config.client_id, "client_secret": config.client_secret, "code": code, "grant_type": "authorization_code", "redirect_uri": config.redirect_uri})
        ),
        // ... 其他 provider
        _ => return Err(MornError::Internal("unsupported provider".into()))
    };
    
    // HTTP POST 获取 token
    let client = reqwest::blocking::Client::new();
    let resp = client.post(token_url)
        .header("Accept", "application/json")
        .json(&params)
        .send()
        .map_err(|e| MornError::Internal(format!("OAuth request failed: {}", e)))?;
    
    let token_data: Value = resp.json().map_err(|e| MornError::Internal(format!("parse failed: {}", e)))?;
    let access_token = token_data["access_token"].as_str()
        .ok_or_else(|| MornError::Internal("no access_token in response".into()))?;
    
    Ok(access_token.to_string())
}
```

同时添加 `provider_configs` 持久化方法：允许用户通过设置页输入各 provider 的 client_id / client_secret。

### T3: 真实化 Tauri 命令（commands/oauth.rs）

当前文件 `src-tauri/src/commands/oauth.rs` 只有两个桩函数。

需要：
1. `oauth_authorize(provider: String)` — 调用 `OAuthManager.get_auth_url()`，返回真实授权 URL
2. `oauth_callback(provider: String, code: String, state: String)` — 新命令，完成 token exchange 并存储
3. `oauth_list_providers()` — 从 OAuthManager 读取配置，返回完整 provider 列表（含是否已配 client_id）
4. `oauth_save_config(provider: String, client_id: String, client_secret: String)` — 保存 provider 凭据

所有命令需要接入 AppState：
```rust
#[tauri::command]
pub(crate) async fn oauth_authorize(
    state: State<'_, AppState>,
    provider: String,
) -> Result<String, CommandError> {
    let oauth = state.oauth_manager.lock().map_err(|e| ...)?;
    if let Some(manager) = &*oauth {
        manager.get_auth_url(&provider).map_err(|e| ...)
    } else {
        Err("OAuth not configured".into())
    }
}
```

### T4: AppState 接线

`src-tauri/src/lib.rs` 中 AppState 新增：
```rust
pub struct AppState {
    // ... 现有字段
    pub oauth_manager: Arc<Mutex<Option<OAuthManager>>>,
}
```

`setup()` 中初始化：
```rust
let oauth_manager = OAuthManager::new(storage.clone());
// 从 settings 表加载 provider 凭据
if let Some(config_json) = storage.get_setting("oauth_providers") {
    if let Ok(configs) = serde_json::from_str::<HashMap<String, OAuthConfig>>(&config_json) {
        for (name, config) in configs {
            oauth_manager.add_provider(name, config);
        }
    }
}
app_state.oauth_manager = Arc::new(Mutex::new(Some(oauth_manager)));
```

### T5: 前端 callback 页面

新建 `web/src/oauth/Callback.tsx`：
```tsx
// 从 URL 参数读取 code 和 state
// 调 invoke('oauth_callback', { provider, code, state })
// 成功后：localStorage 标记 + window.close()
// 失败时：显示错误
```

在 App.tsx 中检测是否在 callback 页面：
```tsx
// 在 AppInner 顶部：
const urlParams = new URLSearchParams(window.location.search);
const code = urlParams.get('code');
if (code) {
    // 这是 OAuth callback，渲染 Callback 组件
    return <CallbackHandler />;
}
```

### T6: Connections.tsx 适配

当前 `web/src/console/Connections.tsx` 已有基本 UI，需确保：
1. `oauth_list_providers()` 正确显示可用 provider
2. 点击连接按钮 → `oauth_authorize(provider)` → `window.open(url)`
3. redirect_uri 指向 Morn 的 callback URL（`http://localhost:1420/oauth/callback` 或 Tauri 自定义协议）

### T7: 验证

```bash
cargo check -p morn
cargo test --lib
```

## 验证门禁

- `cargo check -p morn` ✅
- `cargo test --lib` 全部通过
- `cargo clippy -p morn` 0 warnings
- `oauth_authorize("github")` 返回真实 GitHub 授权 URL
- 前端浏览器打开 OAuth 页面后能回调
- token 存储到 `oauth_tokens` 表
