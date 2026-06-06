# Morn

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/huangdi97/morn)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **你的桌面 AI 创作系统** — From一个人的工位开始

Morn 是一个跑在 Windows 桌面的 AI 操作系统。集 **工作台**（对话交互）、**创作台**（组件搭建）、**管理台**（运营监控）于一体。

支持从原子组件到多 Agent 团队的四层组合，以及完整的市场生态。

## 功能矩阵

### 三台一体
- **🛠 工作台** — NL 对话交互，一句话指挥 COO 拆任务、派活、跟进度
- **🎨 创作台** — 拖拽式 Agent 组装 + 组件管理 + 即时测试
- **📋 管理台** — 系统监控、拓扑可视化、成本中心、治理策略

### 四层组合架构
| 层 | 描述 |
|----|------|
| ① 原子组件 (6 类) | Tool / Knowledge / Skill / Persona / Memory / Model |
| ② Agent | 组件组装成单一 Agent，可配置人格、记忆、工具集 |
| ③ 多 Agent 团队 | 7 种协作模式：链式 / 主管-工人 / 广播 / 投票 / 路由 / 工具 / 黑板 |
| ④ 工作流 | NL→Workflow 自动编排，支持变量系统、模板商店 |

### 10 大模块集群

| 模块 | 功能 |
|------|------|
| **核心运行时** | COO 主管决策树、DAG 引擎、事件总线、Registry 注册中心 |
| **组件体系** | 6 类原子组件 + 52 个预置人格模板 |
| **安全体系** | 4 层宪法（硬拦截→审批→通知→自由）+ Dual-LLM + 隐私闸门 |
| **渠道适配** | CLI / Telegram / 企微 / 钉钉 / 飞书 / 微信小程序 / REST API / SMTP |
| **Agent 能力** | 三阶段 Agent(Plan→Implement→Review)、主管-专家调度、Agent 集群 |
| **记忆系统** | 三层记忆(Working/Episodic/Semantic) + 自编辑记忆 |
| **平台功能** | REST API、看板调度、Code-as-Tool、搜索启动器、模板商店 |
| **认知智能** | PikoSoul 性格引擎、认知录制、信任评分、共识协作 |
| **高级能力** | 视觉GUI操控、跨渠道身份统一、3D 可视化、超长任务引擎 |
| **工具生态** | MCP 协议、Office 处理(PPT/Excel)、Cortex推理引擎、社区模板市场 |

## 快速开始

```bash
# 1. 克隆
git clone https://github.com/huangdi97/morn.git
cd morn-desktop

# 2. 构建 CLI
cargo build --release --bin morn

# 3. 运行（需要 API Key）
MORN_API_KEY=sk-xxx cargo run --release -- cli

# 4. 直接输入文本对话
```

### CLI 命令

| 命令 | 功能 |
|------|------|
| 直接输入文本 | 对话 |
| `/exit` | 退出 |
| `/clear` | 清除历史 |
| `/status` | 显示会话状态 |
| `/mode` | 设置 COO 模式 (active/safe/auto) |
| `/market` | 浏览组件市场 |
| `/help` | 帮助 |

### 桌面端 (Tauri)

完整桌面端需要 WSL/Windows 上的 GTK 系统库，或直接在 Windows 上构建：

```bash
# 前端构建
cd web && npm run build

# 完整桌面构建（需系统依赖）
cargo build --workspace --release
```

## 项目结构

```
morn-desktop/
├── src/                          # Rust 核心库 (lib)
│   ├── main.rs                   # CLI 入口
│   ├── lib.rs                    # 模块声明
│   ├── core/                     # 内核 (30+ 模块)
│   │   ├── supervisor.rs         # COO 主管（6 级决策树）
│   │   ├── registry.rs           # 能力注册中心
│   │   ├── storage.rs            # SQLite 存储（12+ 表）
│   │   ├── event_bus.rs          # 事件总线
│   │   ├── security.rs           # 四层安全宪法
│   │   ├── orchestrator.rs       # 多 Agent 团队编排
│   │   ├── workflow.rs           # 工作流引擎
│   │   ├── mcp.rs                # MCP 协议通信
│   │   ├── hitl.rs               # Human-in-the-Loop 审批
│   │   ├── checkpoint.rs         # 任务持久化
│   │   ├── agent_pool.rs         # Agent 集群管理
│   │   ├── consensus.rs          # 共识协作机制
│   │   └── ... (20+ 更多模块)
│   ├── component/                # 6 类原子组件
│   ├── bridge/                   # LLM API 适配器
│   ├── channel/                  # 多渠道适配
│   ├── studio/                   # 创作台后端
│   ├── console/                  # 管理台后端
│   ├── api/                      # REST API
│   ├── computer/                 # 电脑操控
│   └── market/                   # 组件市场
├── src-tauri/                    # Tauri 桌面入口 (NSIS 安装器 + 自动更新)
│   ├── src/lib.rs                # 28 个 Tauri 命令
│   └── tauri.conf.json           # 桌面配置
├── web/                          # React + TypeScript + Vite 前端
│   └── src/
│       ├── App.tsx               # 工作台聊天界面
│       ├── studio/               # 创作台 UI (AgentBuilder + TestPanel)
│       ├── dashboard/            # 仪表盘
│       ├── store/                # BotStore
│       └── console/              # 管理台 UI
└── DESIGN.md                     # 设计总纲（本地专属）
```

## 技术栈

- **语言**: Rust (edition 2021)
- **桌面框架**: Tauri v2 (NSIS 安装器)
- **前端**: React 18 + TypeScript + Vite 5
- **LLM**: OpenAI 兼容 API（默认 DeepSeek）
- **存储**: SQLite (rusqlite)
- **HTTP**: reqwest + axum
- **协议**: MCP (Model Context Protocol)

## 路线图

| 优先级 | 项 |
|--------|-----|
| **P0** | 🏗️ 打成安装包 + 自动更新 |
| **P1** | 🎯 首次引导 (Onboarding) + Registry 热加载 |
| **P2** | 🧬 自进化 Skill + MDRM 图谱记忆 |
| **P3** | 📱 扫码绑定 IM + 活人感引擎 |
| **P4** | 📊 深度可观测性 |

---

[贡献指南](CONTRIBUTING.md) · [更新日志](CHANGELOG.md) · [MIT 许可证](LICENSE)
