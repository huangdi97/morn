# Morn 安装与配置

> CLI 模式 · Tauri 桌面端 · NSIS 安装包 · 自动更新

## CLI 模式（开发/快速体验）

```bash
# 克隆
git clone https://github.com/huangdi97/morn.git
cd morn-desktop

# 构建 CLI
cargo build --release --bin morn

# 运行
MORN_API_KEY=sk-xxx cargo run --release -- cli
```

### CLI 命令

| 命令 | 功能 |
|------|------|
| 直接输入文本 | 对话 |
| `/exit` | 退出 |
| `/clear` | 清除对话历史 |
| `/status` | 显示系统状态（轮次/模式/连接） |
| `/mode` | 设置 COO 模式 (active/safe/auto) |
| `/market` | 浏览组件市场 |
| `/help` | 帮助信息 |
| `--daemon` | 以守护进程模式启动 |

## Tauri 桌面端（完整体验）

### 系统依赖

**Windows:** 无需额外依赖（所有库捆绑在 NSIS 安装包中）。

**Linux (开发):**
```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
```

### 构建

```bash
# 1. 构建前端
cd web && npm run build

# 2. 构建桌面端
cd .. && cargo build --workspace --release
```

## NSIS 安装包

通过 CI 自动构建：

1. 创建 GitHub Release → 打 tag
2. CI 在 Windows runner 上自动构建
3. 产出：`Morn_0.1.0_x64-setup.exe` + `Morn_0.1.0_x64.msi`
4. Artifact 上传至 Release 页面

支持语言：简体中文、English

安装模式：当前用户 (currentUser)，无需管理员权限。

## 自动更新

| 组件 | 配置 |
|------|------|
| 更新插件 | tauri-plugin-updater v2 |
| 端点 | GitHub Releases (`update.json`) |
| 检查频率 | 每次启动 + 后台定时检查 |
| 交互 | 有可用更新时弹出对话框 |

### 发布更新流程

1. 构建新版本 → 打 tag → GitHub Release
2. CI 构建 NSIS 安装包 + MSI
3. CI 自动生成 `update.json` 上传到 Release
4. 已安装的 Morn 桌面端自动检测到更新

## 环境变量

| 变量 | 必需 | 说明 |
|------|------|------|
| `MORN_API_KEY` | 是 | LLM API 密钥 |
| `MORN_API_BASE` | 否 | API 端点 (默认 https://api.deepseek.com) |
| `MORN_MODEL` | 否 | 模型名 (默认 deepseek-chat) |
| `MORN_CONFIG_DIR` | 否 | 配置文件目录 (默认 ~/.config/morn) |
| `MORN_DATA_DIR` | 否 | 数据存储目录 (默认 ~/.local/share/morn) |
| `MORN_LOG_LEVEL` | 否 | 日志级别 (默认 info) |
| `MORN_DAEMON_PORT` | 否 | 守护进程端口 (默认 3000) |

## 配置文件 (config.toml)

```toml
[api]
key = "sk-xxx"              # LLM API 密钥
base_url = "https://api.deepseek.com"
model = "deepseek-chat"

[daemon]
enabled = true
port = 3000
host = "127.0.0.1"

[storage]
data_dir = "~/.local/share/morn"
db_name = "morn.db"

[logging]
level = "info"
file = "~/.local/share/morn/morn.log"

[security]
guard_model = "deepseek-chat"    # Guard LLM 模型
judge_model = "deepseek-chat"    # Judge LLM 模型
encryption_key = ""              # E2E 加密密钥（自动生成）
```
