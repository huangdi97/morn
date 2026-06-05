# 安装指南

## 系统要求

| 组件 | 要求 |
|------|------|
| OS | Linux（CLI 模式），Windows 10+（Tauri 桌面端） |
| Rust | 1.75+ |
| Node.js | 18+（仅构建前端时需要） |
| WebView2 | Windows 10+ 自带（Tauri 桌面端） |

## 依赖安装

### Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable  # 确保 1.75+
```

### Node.js

```bash
# 使用 nvm 安装
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 18
```

## 环境变量

| 变量 | 必填 | 说明 |
|------|------|------|
| `MORN_API_KEY` | 是 | DeepSeek / OpenAI 兼容 API Key |

启动时未设置 `MORN_API_KEY` 会进入帮助模式，显示可用命令清单。

## 构建方式

### CLI 模式（推荐开发测试）

```bash
cargo build --release --bin morn
MORN_API_KEY=sk-xxx ./target/release/morn cli
```

### 桌面端（Tauri）

```bash
cargo build -p morn-desktop --release
# 需要 Windows + WebView2
```

### 前端开发

```bash
cd web
npm install
npm run dev
```

## 故障排查

| 问题 | 原因 | 解决 |
|------|------|------|
| `MORN_API_KEY` not set | 环境变量未配置 | `export MORN_API_KEY=sk-xxx` |
| Tauri 构建失败 | 缺少 WebView2 | Windows 10+ 自动包含，检查系统更新 |
| SQLite 编译错误 | 缺少 C 编译器 | `apt install build-essential` (Linux) |
| LLM 返回空 | API Key 过期或余额不足 | 检查 DeepSeek 账户 |