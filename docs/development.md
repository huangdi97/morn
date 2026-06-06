# Morn 开发者指南

> 构建 · 测试 · 贡献 · 项目结构

## 开发环境

| 工具 | 版本要求 |
|------|---------|
| Rust | 1.75+ (推荐 stable) |
| Node.js | 18+ |
| npm | 9+ |
| Tauri CLI | v2 (通过 cargo install) |

## 快速开始

```bash
# 克隆
git clone https://github.com/huangdi97/morn.git
cd morn-desktop

# 构建库
cargo build

# 运行测试
cargo test

# 构建前端
cd web && npm install && npm run build

# CLI 启动
MORN_API_KEY=sk-xxx cargo run
```

## 项目结构

```
morn-desktop/
├── src/                          # Rust 核心库
│   ├── main.rs                   # CLI 入口
│   ├── lib.rs                    # 模块声明
│   ├── core/                     # 内核 (30+ 模块)
│   │   ├── supervisor.rs         # COO 主管
│   │   ├── registry.rs           # 组件注册中心
│   │   ├── storage.rs            # SQLite 存储
│   │   ├── event_bus.rs          # 事件总线
│   │   ├── security.rs           # 安全体系
│   │   └── ... (25+ 更多)
│   ├── component/                # 6 类原子组件
│   ├── bridge/                   # LLM API 适配器
│   ├── channel/                  # 多渠道适配器
│   ├── studio/                   # 创作台后端
│   ├── console/                  # 管理台后端
│   ├── api/                      # REST API
│   ├── computer/                 # 电脑操控
│   └── market/                   # 组件市场
├── src-tauri/                    # Tauri 桌面入口
│   ├── src/lib.rs                # 28 个 Tauri 命令
│   ├── src/main.rs               # Windows 入口
│   ├── tauri.conf.json           # 桌面配置
│   └── icons/                    # 应用图标
├── web/                          # React + TypeScript 前端
│   └── src/
│       ├── App.tsx               # 工作台聊天界面
│       ├── studio/               # 创作台 UI
│       ├── dashboard/            # 仪表盘
│       ├── console/              # 管理台 UI
│       └── store/                # Bot 商店
├── docs/                         # 文档
├── DESIGN.md                     # 设计总纲 (本地)
└── Cargo.toml                    # 工作区配置
```

## 构建命令

| 命令 | 说明 |
|------|------|
| `cargo build` | 构建调试版本 |
| `cargo build --release` | 构建发布版本 |
| `cargo test` | 运行全部测试 (417 tests) |
| `cargo fmt` | 格式化代码 |
| `cargo clippy` | 静态分析 |
| `cd web && npm run build` | 构建前端 |
| `cargo tauri build` | 构建桌面安装包 |

## 添加新模块

1. 在 `src/core/` 下创建 `.rs` 文件
2. 在 `src/lib.rs` 的模块声明中添加
3. 实现核心功能 + 单元测试
4. 运行 `cargo build && cargo test`
5. 运行 `cargo fmt` 格式化代码

## 测试

| 测试类型 | 数量 | 运行方式 |
|---------|------|---------|
| 单元测试 | 417+ | `cargo test` |
| 前端类型检查 | - | `cd web && tsc --noEmit` |
| 前端构建 | - | `cd web && npm run build` |
| 格式检查 | - | `cargo fmt --check` |

## CI/CD

GitHub Actions 自动运行：

| Job | Runner | 内容 |
|-----|--------|------|
| build-and-test | ubuntu-latest | cargo build + cargo test + cargo fmt |
| build-tauri | windows-latest | npm build + cargo tauri build → NSIS/MSI |

## 贡献指南

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/xxx`)
3. 提交改动 (`git commit -m "feat: xxx"`)
4. 推送到分支 (`git push origin feature/xxx`)
5. 创建 Pull Request

### 代码风格
- Rust: `cargo fmt` 自动格式化
- TypeScript: Prettier 标准配置
- Commit: Conventional Commits (feat/fix/docs/chore)

### 许可
MIT License — 参见仓库 LICENSE 文件。
