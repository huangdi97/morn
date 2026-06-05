# Morn Desktop

> 你的桌面 AI 创作系统 · Rust + Tauri + React

Morn 是一个运行在 Windows 桌面的 AI 平台，集工作台、创作台、管理台于一体，支持从原子组件到多 Agent 团队的四层组合，以及完整的市场生态。

## 架构总览

```
┌──────────────────────────────────────────────────────────┐
│                    Morn Desktop                           │
│         Tauri (Rust) · Windows 原生 .exe                  │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  接入层                                                   │
│  Desktop UI | CLI | Channel Adapter                       │
│                                                          │
│  ┌── COO (Supervisor) ──────────────────────────────┐   │
│  │  1. Intent Parser (NL → 结构化意图)              │   │
│  │  2. Planner (意图分解为 DAG 计划)                 │   │
│  │  3. Router (匹配组件 + 信任评分加权)              │   │
│  │  4. Scheduler (DAG 调度)                          │   │
│  │  6 级决策树: 直接答→单工具→Agent→团队→模板→创作台 │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  ┌── 6 类原子组件 ────────────────────────────────┐    │
│  │  Tool / Knowledge / Skill / Persona / Memory / Model │   │
│  │  可自由组合成 Agent → 组合成团队 → 发布           │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  ┌── 多 Agent 团队 ───────────────────────────────┐    │
│  │  7 种协作模式: 链式/主管-工人/广播/投票/路由/    │   │
│  │  Agent即工具/共享黑板 + 工作流引擎               │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  SQLite (agents/capabilities/tasks/executions/decisions) │
└──────────────────────────────────────────────────────────┘
```

## 三阶段路线图

| 阶段 | 状态 | 内容 |
|------|------|------|
| Phase 0 | ✅ 完成 | 骨架：单人·单 Agent·桌面端·CLI |
| Phase 1 | ✅ 完成 | 六类组件 + 创作台 + IM 渠道 + COO 完整决策树 |
| Phase 2+ | ✅ 完成 | 多 Agent 团队 + 工作流 + 管理台 + 电脑操控 + 市场 |

## 技术栈

- **语言**: Rust (edition 2021)
- **桌面框架**: Tauri v2
- **前端**: React 18 + TypeScript + Vite
- **LLM**: OpenAI 兼容 API（默认 DeepSeek）
- **存储**: SQLite (rusqlite)
- **HTTP**: reqwest (rustls-tls)

## 快速开始

### 前提

- Rust toolchain (1.75+)
- Node.js (18+)
- `MORN_API_KEY` 环境变量（DeepSeek 或 OpenAI 兼容的 API key）

### 构建与运行

```bash
# 克隆
git clone https://github.com/huangdi97/morn.git
cd morn-desktop

# 构建
cargo build --release --bin morn

# CLI 模式
MORN_API_KEY=sk-xxx cargo run --release -- cli

# Tauri 桌面端（需要 Windows + WebView2）
cargo build -p morn-desktop --release
```

### CLI 命令

| 命令 | 功能 |
|------|------|
| 直接输入文本 | 对话 |
| `/exit` | 退出 |
| `/clear` | 清除历史 |
| `/status` | 显示会话状态 |
| `/help` | 帮助 |

## 项目结构

```
morn-desktop/
├── src/
│   ├── main.rs              # 入口
│   ├── lib.rs               # 模块声明
│   ├── core/                # 内核
│   │   ├── supervisor.rs    # COO 主管（6 级决策树）
│   │   ├── registry.rs      # 能力注册中心
│   │   ├── storage.rs       # SQLite 存储（7 表）
│   │   ├── engine.rs        # 执行引擎（DAG 调度）
│   │   ├── event_bus.rs     # 事件总线
│   │   ├── security.rs      # 四层安全宪法
│   │   ├── component.rs     # Component trait 体系
│   │   ├── assembler.rs     # Agent 组装器
│   │   ├── orchestrator.rs  # 多 Agent 团队编排
│   │   ├── workflow.rs      # 工作流引擎
│   │   ├── trust_evaluator.rs # 四层信任评分
│   │   ├── dual_llm.rs      # Dual-LLM 安全
│   │   └── worker.rs        # Worker 线程池
│   ├── component/           # 六类原子组件
│   │   ├── tool.rs
│   │   ├── knowledge.rs
│   │   ├── skill.rs
│   │   ├── persona.rs
│   │   ├── memory.rs
│   │   └── model.rs
│   ├── bridge/
│   │   └── chat_agent.rs    # LLM API 适配器
│   ├── channel/             # 渠道
│   │   ├── adapter.rs       # 统一消息适配器
│   │   ├── cli.rs           # CLI 通道
│   │   ├── wecom.rs         # 企业微信
│   │   ├── dingtalk.rs      # 钉钉
│   │   ├── feishu.rs        # 飞书
│   │   ├── rest_api.rs      # REST API
│   │   └── ...
│   ├── studio/              # 创作台后端
│   ├── console/             # 管理台后端
│   ├── computer/            # 电脑操控
│   └── market/              # 市场
├── src-tauri/               # Tauri 桌面入口
└── web/                     # React 前端
    └── src/
        ├── App.tsx          # 工作台聊天界面
        ├── studio/          # 创作台前端
        └── console/         # 管理台前端
```

## 许可证

MIT
