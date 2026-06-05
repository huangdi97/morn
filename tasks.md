# Morn 三大方向 — 渠道真实化 + 电脑操控 + 企业多人版

## 核心准则 (编程原则)

14条核心准则（Think Before Coding、The Code Works、Small Batches、No Dead Code、Single Source of Truth、Test the Paths、Fail Fast、Leave It Better、Never Guess the Stack、Read Before You Write、Prefer Friction Logs、Respect the Dependency、No Ambiguous Names、Document Decisions, Not Drama）
+ 低耦合3条（模块独立、接口一致、渐进增强）
+ 执行规则（按任务顺序、每个任务后 cargo build && cargo test、git push 到 main）

---

## 前置阅读

- `src/channel/telegram.rs` — 现有 Telegram stub
- `src/channel/adapter.rs` — 通道适配器
- `src/computer/` 下所有文件
- `src/core/storage.rs` — Storage 模块（企业版需要新表）
- Cargo.toml — 已有 reqwest + tokio 依赖，足够做 HTTP 请求

---

## 阶段一：渠道真实接入

### 任务1: Telegram 通道做真

改造 `src/channel/telegram.rs`，从 stub 变成真实 Telegram Bot API 调用。

**技术要求：**
- 使用 `reqwest` 发 HTTP POST 到 `https://api.telegram.org/bot<token>/sendMessage`
- 从 `TELEGRAM_BOT_TOKEN` 环境变量读取 bot token
- `send()` 方法真实发送消息
- `send_message()` 支持 parse_mode（Markdown/MarkdownV2/HTML）
- `set_webhook()` 真实设置 webhook
- 错误处理：网络超时、API 错误码、token 无效

**新增文件：** 无，改造现有 `telegram.rs`
**验证：** cargo build 通过 + cargo test 通过 + 实际发一条消息（可选）

### 任务2: SMTP 通道做真

改造 `src/channel/smtp.rs`，使用 `lettre` crate 发送真实邮件。

**技术要求：**
- 在 Cargo.toml 添加 `lettre` 依赖（带 smtp-transport + rustls-tls feature）
- 从环境变量读取 SMTP 配置：SMTP_HOST、SMTP_PORT、SMTP_USERNAME、SMTP_PASSWORD、SMTP_FROM
- `send_report()` 真实发送邮件
- 支持 TLS 加密连接
- 错误处理

### 任务3: REST API 通道做真

改造 `src/channel/rest_api.rs`，添加真实的 HTTP 服务器。

**技术要求：**
- 使用 `actix-web` 或 `axum` 启动 HTTP 服务器
- 支持 POST `/chat` — 接受消息文本，返回 AI 回复
- 支持 GET `/status` — 返回系统状态 JSON
- 支持 POST `/clear` — 清除对话历史
- 服务器端口通过 `API_PORT` 环境变量配置（默认 8080）
- `axum` 更轻量，优先

### 任务4-8: 其它渠道 stub 标注

对 dingtalk/feishu/wecom/qqbot/wechat_mp/miniprogram/webhook 不做真实改造（需要对应平台的真实账号/应用），但在每个文件顶部加文档注释说明：
```rust
//! 注意：此通道需要 [平台名称] 真实应用注册才能使用
//! 配置方式：xxx
```

---

## 阶段二：电脑操控做真

### 任务9: 文件系统操作做真

改造 `src/computer/fs_ops.rs`：
- `read()` — 已真实 ✅
- `write()` — 已真实 ✅
- `move()` — 已真实 ✅
- `delete()` — 已真实 ✅
- `search()` — 已真实 ✅
- `compress()` — 从 `[simulated]` 改为调用系统 `tar` 或 `zip` 命令实现真实压缩

### 任务10: 桌面操作做真（PowerShell 桥接）

改造 `src/computer/desktop_ops.rs`：

在 Windows 环境下（通过 `cfg!(target_os = "windows")` 判断），调用 PowerShell 命令：
- `mouse_move(x, y)` — 用 PowerShell `[System.Windows.Forms.Cursor]::Position`
- `mouse_click(button)` — 用 user32.dll `mouse_event`
- `keyboard_type(text)` — 用 `SendKeys`
- `screenshot()` — 用 `[System.Windows.Forms.Screen]::PrimaryScreen` 截图
- 注：需要添加 `windows` 或使用 PowerShell 桥接

在非 Windows 环境（Linux/WSL）保持 `[simulated]` 并打印提示。

**更实用的做法：** 
从 WSL 调用 PowerShell.exe 执行命令：
- `powershell.exe -Command "[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point(x,y)"`
- `powershell.exe -Command "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('text')"`
- 截图用 `powershell.exe -Command "Add-Type -AssemblyName System.Windows.Forms; ..."`

### 任务11: 浏览器操控做真

改造 `src/computer/browser_ops.rs`：

`navigate(url)` — 调用系统默认浏览器打开 URL：
- Windows: `cmd.exe /c start <url>`
- Linux: `xdg-open <url>`

`content_extract(url)` — 使用 reqwest 抓取并提取网页内容文本

### 任务12: 应用操控做真

改造 `src/computer/app_ops.rs`：

`launch(app_name)` — 启动应用：
- Windows: `cmd.exe /c start <app_name>` 或 PSH `Start-Process`
- Linux: `which <app> && <app> &`

`list()` — 列出正在运行的进程：
- Windows: `tasklist`
- Linux: `ps aux`

`close(app_name)` — 结束进程：
- Windows: `taskkill /IM <name>`
- Linux: `pkill <name>`

### 任务13: 系统操控做真

改造 `src/computer/sys_ops.rs`：

`network_status()` — 使用 `ping` 或 reqwest 检查网络连通性
`power_status()` — 读取 `/sys/class/power_supply/` (Linux) 或 PSH `Get-WmiObject`
`get_volume()` / `set_volume(level)` — WSL 中可用 pactl/amixer（Linux）或 PSH 调用 Windows

注意：shutdown/sleep/restart 保持 `[simulated]` 并打印警告，避免误操作。

### 任务14: 感知模块做真

改造 `src/computer/perception.rs`：

`pixel_screenshot()` — 截取屏幕像素（依赖 screenshot 实现）
`ocr()` — 调用 tesseract CLI（如果已安装）或保持 simulated
`accessibility_tree()` — 保持 simulated（需要系统级权限）

---

## 阶段三：企业多人版

### 任务15: 用户与团队数据模型

在 `src/core/storage.rs` 中新增表：
- `users` — id, username, display_name, role(admin/user/viewer), created_at, last_login
- `teams` — id, name, description, owner_id, created_at
- `team_members` — id, team_id, user_id, role(owner/admin/member), joined_at
- `agent_permissions` — id, agent_id, user_id, team_id(optional), permission(read/use/manage/admin), granted_at
- `audit_log` — id, user_id, action, target_type, target_id, details_json, created_at

添加对应的 CRUD 方法到 Storage。

### 任务16: 团队管理模块

新建 `src/org/mod.rs` 和 `src/org/team.rs`：

TeamManager：
- `create_team(name, description, owner_id)` — 创建团队
- `add_member(team_id, user_id, role)` — 添加成员
- `remove_member(team_id, user_id)` — 移除成员
- `list_teams(user_id)` — 用户的团队列表
- `list_members(team_id)` — 团队成员列表
- `transfer_ownership(team_id, new_owner_id)` — 转移所有权

UserManager：
- `register(username, display_name, role)` — 注册用户
- `get_user(id)` — 获取用户信息
- `list_users()` — 所有用户

### 任务17: 权限系统

新建 `src/org/permissions.rs`：

PermissionChecker：
- `check(user_id, action, target)` — 检查是否有权限
- `grant(user_id, agent_id, permission)` — 授权
- `revoke(user_id, agent_id)` — 撤销
- `list_permissions(agent_id)` — 查看谁有什么权限

权限等级：read < use < manage < admin
- read：只看 Agent 信息，不能调用
- use：可以调用 Agent
- manage：可以修改 Agent 配置
- admin：可以删除/转移 Agent + 管理权限

### 任务18: Agent 共享

改造 `src/core/registry.rs` 和 `src/core/supervisor.rs`：

- Agent 可以标记为 public（所有人可用）/ team（团队可用）/ private（仅自己）
- Supervisor 执行时根据用户身份过滤可用 Agent
- 团队共享的 Agent 可以设置每日调用额度

### 任务19: 审计日志

新建 `src/org/audit.rs`：

- 记录：谁在什么时间做了什么操作
- `AuditLogger` — 自动记录所有关键操作（用户登录/登出、Agent 创建/修改/删除、权限变更、团队变更）
- `query(user_id, date_range, action_type)` — 查询审计日志
- 从 `src/console/governance.rs` 关联审计数据

### 任务20: 管理台 — 组织管理页面（Tauri 命令）

在 `src-tauri/src/lib.rs` 添加 Tauri 命令：
- `create_user` / `list_users`
- `create_team` / `list_teams` / `add_member` / `remove_member`
- `grant_permission` / `revoke_permission`
- `get_audit_log`

### 任务21: 测试

为所有新增模块写测试：
- 用户 CRUD 测试
- 团队增删改测试
- 权限检查测试（read < use < manage < admin 层级）
- Agent 共享测试（public/team/private）
- 审计日志测试
- 确保 0 新增 warning

### 任务22: 验证与提交

```bash
cd ~/morn-desktop
cargo build 2>&1
cargo test 2>&1
# 检查是否有新增 warning（新代码应 0 warning）
cargo build 2>&1 | grep "warning" | wc -l
git add -A
git commit -m "feat: channel real implementation + computer control + org system

- Telegram, SMTP, REST API channels now use real APIs
- Computer control uses PowerShell/OS calls instead of simulated
- Add org module: users, teams, permissions, audit log
- Add enterprise multi-user support"
git push origin main
```
