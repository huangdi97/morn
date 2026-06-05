# Morn Desktop

[![build](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/huangdi97/morn)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

> Morn — 你的桌面 AI 创作系统

Morn 是一个运行在桌面的 AI 平台，集工作台、创作台、管理台于一体，支持从原子组件到多 Agent 团队的四层组合，以及完整的市场生态。

## 功能概览

- **三台一体** — 工作台（对话交互）、创作台（组件管理 / Agent 组装 / 测试发布）、管理台（成本监控 / 治理策略 / 系统健康）
- **四层组合** — 原子组件（6 类）→ Agent → 多 Agent 团队 → 工作流引擎
- **多 Agent 团队** — 7 种协作模式：链式 / 主管-工人 / 广播 / 投票 / 路由 / Agent 即工具 / 共享黑板
- **六类原子组件** — Tool / Knowledge / Skill / Persona / Memory / Model
- **市场** — 组件 / Agent / 工作流的上架、下载、评分、许可证管理
- **渠道适配** — CLI、Telegram、企业微信、钉钉、飞书、REST API、QQ Bot、微信小程序、微信公众号、Webhook、SMTP
- **四层安全宪法** — L1 硬拦截 / L2 需审批 / L3 需通知 / L4 自由，Dual-LLM 双重审查
- **电脑操控** — 桌面操作 / 文件系统 / 浏览器控制 / 应用管理 / 系统管理 / 感知（当前为模拟阶段）

## 快速开始

```bash
# 1. 克隆
git clone https://github.com/huangdi97/morn.git
cd morn-desktop

# 2. 构建 CLI
cargo build --release --bin morn

# 3. 运行（需要 API Key）
MORN_API_KEY=sk-xxx cargo run --release -- cli

# 4. CLI 内直接输入文本对话，输入 /exit 退出
```

## CLI 命令一览

| 命令 | 功能 |
|------|------|
| 直接输入文本 | 对话 |
| `/exit` | 退出 |
| `/clear` | 清除历史 |
| `/status` | 显示会话状态 |
| `/mode` | 设置 COO 模式（active/safe/auto） |
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

## 技术栈

- **语言**: Rust (edition 2021)
- **桌面框架**: Tauri v2
- **前端**: React 18 + TypeScript + Vite
- **LLM**: OpenAI 兼容 API（默认 DeepSeek）
- **存储**: SQLite (rusqlite)
- **HTTP**: reqwest (rustls-tls)

## 路线图

| 阶段 | 状态 | 内容 |
|------|------|------|
| Phase 0 | ✅ 完成 | 骨架：单人·单 Agent·桌面端·CLI |
| Phase 1 | ✅ 完成 | 六类组件 + 创作台 + IM 渠道 + COO 完整决策树 |
| Phase 2+ | ✅ 完成 | 多 Agent 团队 + 工作流 + 管理台 + 电脑操控 + 市场 |

---

[贡献指南](CONTRIBUTING.md) · [MIT 许可证](LICENSE)