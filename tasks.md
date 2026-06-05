# Morn 收尾阶段：OS 层 + 渠道做真 + 手机端

## 14 条核心准则编程原则

**1. Think Before Coding** — 理解全貌再动手
**2. Simplicity First** — 简单优先
**3. Surgical Changes** — 一次一事
**4. Goal-Driven Execution** — 先明确"要什么"再"怎么写"
**5. 架构优先，拒绝补丁**
**6. 面向组件的构建**
**7. 显式优于隐式**
**8. 代码整洁与自文档化**
**9. 单一职责** — 一个函数/类只做一件事
**10. 组合优于委托**
**11. 单一状态源**
**12. 避免语法糖**
**13. 命名一致性**
**14. 文件不超过 300 行**

## 低耦合原则
1. 模块间只通过公开接口通信
2. 测试独立于实现细节
3. 新测试模块互相独立

## 执行方式
- 所有代码修改必须通过 **opencode run**
- 每阶段完后运行 `cargo fmt && cargo build && cargo test`
- 全部完成后全量验证：`cargo build + cargo test + cargo fmt --check + cd web && npm run build`

---

## 阶段一：深度 OS 层（系统托盘 + 开机自启）

### 背景
目前 Morn 只是一个 CLI 应用 + Tauri 窗口。设计文档要求：开机自启动、系统托盘常驻、后台运行。

### 任务 1.1: 添加 Tauri 系统托盘

**文件:** `src-tauri/src/lib.rs` + `src-tauri/Cargo.toml`

在 Cargo.toml 中添加：
```toml
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-notification = "2"
```

Tauri 2 的系统托盘实现：
```rust
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, SubmenuBuilder};

// 在 setup 中创建托盘图标
// 右键菜单：显示窗口 / 隐藏窗口 / 退出
// 左键点击：切换窗口显示状态
// 托盘图标使用内嵌图标或默认图标
```

### 任务 1.2: Windows 开机自启

**文件:** `src-tauri/src/lib.rs`

通过 Windows 注册表实现开机自启（不需要额外依赖）：
```rust
use std::os::windows::process::CommandExt;

fn set_auto_start(enabled: bool) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| e.to_string())?;
    let key = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let app_name = "Morn";
    // 使用 reg add / reg delete 命令
}
```

### 任务 1.3: 主窗口最小化到托盘

**文件:** `src-tauri/src/lib.rs`

实现最小化时隐藏到系统托盘而非任务栏：
- 点击关闭按钮 → 隐藏到托盘
- 托盘图标左键 → 显示/隐藏切换
- 退出确认对话框

---

## 阶段二：渠道 stub 做真

### 背景
目前 QQ 机器人/微信公众号/微信小程序三个渠道只有骨架 struct，没有任何 HTTP 通信。类似 DingTalk/Feishu/WeCom 的 webhook POST 实现。

### 任务 2.1: QQ 机器人渠道做真

**文件:** `src/channel/qqbot.rs`

腾讯 QQ 机器人 API（基于 QQ 开放平台）：
```rust
fn send_message(&self, user_id: &str, content: &str) -> Result<(), String> {
    let url = format!("https://api.qq.com/v1/robots/{}/messages", self.bot_id);
    let body = json!({
        "user_id": user_id,
        "content": content,
        "msg_type": "text"
    });
    // reqwest::blocking POST with auth header
    // 类似 dingtalk.rs 的模式
}
```

核心实现：
1. POST 消息到 QQ 机器人 API endpoint
2. 带 Bot Token 认证（从环境变量读取）
3. 接收消息模型（轮询或 webhook 回调）
4. 错误处理 + 单元测试
5. 注册到 ChannelAdapter

### 任务 2.2: 微信公众号渠道做真

**文件:** `src/channel/wechat_mp.rs`

微信公众平台 API（服务号）：
1. 获取 access_token（使用 AppID + AppSecret）
2. 发送模板消息/客服消息
3. 接收消息回调验证
4. 消息格式转换（xml ↔ ChannelMessage）

```rust
fn get_access_token(&self) -> Result<String, String> {
    let url = format!(
        "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
        self.app_id, self.app_secret
    );
    // reqwest get → 返回 access_token
}

fn send_custom_message(&self, open_id: &str, content: &str) -> Result<(), String> {
    let token = self.get_access_token()?;
    let url = format!("https://api.weixin.qq.com/cgi-bin/message/custom/send?access_token={}", token);
    let body = json!({
        "touser": open_id,
        "msgtype": "text",
        "text": { "content": content }
    });
    // reqwest POST
}
```

### 任务 2.3: 微信小程序渠道做真

**文件:** `src/channel/miniprogram.rs`

小程序云开发 / 消息推送 API：
1. 获取 access_token（同微信公众号，使用 AppID + AppSecret）
2. 订阅消息推送
3. 客服消息接口

结构类似上面两个渠道，使用 reqwest::blocking。

---

## 阶段三：手机端 PWA

### 背景
设计文档要求手机 App 实时看 Agent 干活。最快速方案：把现有 Web 前端改为 PWA（无需原生开发）。

### 任务 3.1: Service Worker

**文件:** `web/public/sw.js`（新增）

基本的 service worker：
```javascript
const CACHE = "morn-v1";
self.addEventListener("install", (e) => {
  e.waitUntil(caches.open(CACHE).then(c => c.addAll(["/", "/index.html"])));
});
self.addEventListener("fetch", (e) => {
  e.respondWith(
    caches.match(e.request).then(r => r || fetch(e.request))
  );
});
```

在 `web/index.html` 中注册：
```html
<script>
if ("serviceWorker" in navigator) {
  navigator.serviceWorker.register("/sw.js");
}
</script>
```

### 任务 3.2: manifest.json

**文件:** `web/public/manifest.json`（新增）

```json
{
  "name": "Morn Agent Platform",
  "short_name": "Morn",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#0d1117",
  "theme_color": "#161b22",
  "icons": []
}
```

在 `index.html` 中添加 link tag。

需要生成一个简单的 SVG 图标作为 PWA 图标（用内联 SVG 或 base64）。

### 任务 3.3: 响应式 CSS 适配

**文件:** `web/src/App.css`

添加移动端响应式适配：
- 媒体查询：`@media (max-width: 768px)` 和 `(max-width: 480px)`
- 调整聊天界面在手机上的布局（全宽、大按钮、底部输入固定）
- Studio NodeCanvas 在手机上的简化视图
- Console 页面卡片堆叠

核心改动：
```css
@media (max-width: 768px) {
  .chat-container { padding: 0; border-radius: 0; }
  .sidebar { display: none; }
  .mobile-menu { display: block; }
  .console-grid { grid-template-columns: 1fr; }
}
```

### 任务 3.4: manifest 生成脚本

在 vite.config.ts 中配置 PWA 插件或手动生成 manifest。

最简单方式：手动创建 `web/public/manifest.json` 和 `web/public/sw.js`，vite 自动复制到构建输出。

---

## 全量验证

```bash
cd ~/morn-desktop
cargo build 2>&1 | grep -E "^(warning|error)" | wc -l   # = 0
cargo test 2>&1 | grep "test result"                     # all passed
cargo fmt --check                                         # no diff
cd web && npm run build                                   # web build success
```
