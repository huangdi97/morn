# Morn 架构文档

> 版本：v0.1.0

## 三台一体

Morn 三个工作台共享同一套底层系统：

```
┌─────────────────────────────────────────────────────────────┐
│                    Morn Desktop v0.1.0                      │
├─────────────────┬─────────────────┬─────────────────────────┤
│   🛠 工作台      │   🎨 创作台      │   📋 管理台             │
│  (Workbench)    │  (Studio)       │  (Console)              │
├─────────────────┼─────────────────┼─────────────────────────┤
│ NL 对话交互      │ 组件管理/组装    │ 系统监控                │
│ 一句话指挥 COO   │ Agent 组装/测试  │ 成本中心                │
│ 后台任务/定时    │ 发布到市场       │ 治理策略                │
│ 多任务并行       │ 画布拖拽连线     │ 安全事件                │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## 四层组合

```
┌──────────────────────────────────────┐
│  ④ 工作流 (Workflow)                  │
│  NL→Workflow · 变量系统 · 模板商店    │
├──────────────────────────────────────┤
│  ③ 多 Agent 团队                     │
│  7种协作模式 · 主管-专家 · 集群       │
├──────────────────────────────────────┤
│  ② Agent                             │
│  人格+记忆+工具+技能+知识+模型 → Agent│
├──────────────────────────────────────┤
│  ① 原子组件 (6类)                    │
│  Tool/Knowledge/Skill/Persona/Memory/Model │
└──────────────────────────────────────┘
```

### 7 种协作模式

| 模式 | 描述 | 适用场景 |
|------|------|---------|
| 链式 (Chain) | A→B→C 串行执行 | 数据处理管道 |
| 主管-工人 (Supervisor-Worker) | 主管拆任务 → 工人并行 | 复杂任务分解 |
| 广播 (Broadcast) | 同一输入发所有 Agent | 多角度分析 |
| 投票 (Voting) | 多 Agent 独立决策→投票 | 质量审核 |
| 路由 (Router) | 按输入内容路由到指定 Agent | 分流处理 |
| Agent 即工具 (Agent-as-Tool) | Agent 注册为其他 Agent 的工具 | 嵌套协作 |
| 共享黑板 (Shared Blackboard) | 所有 Agent 读写同一上下文 | 持续协作 |

## 10 大模块集群

### 1. 核心运行时 (Core Runtime)
| 模块 | 文件 | 职责 |
|------|------|------|
| COO 主管 | `supervisor.rs` | 6 级决策树、任务拆解、COO 学习 |
| Registry | `registry.rs` | 组件注册中心、生命周期管理 |
| Storage | `storage.rs` | SQLite 持久化、12+ 数据表 |
| 执行引擎 | `engine.rs` | DAG 调度、任务编排 |
| 事件总线 | `event_bus.rs` | publish/subscribe/replay |
| 安全体系 | `security.rs` | 4 层宪法安全 |

### 2. 组件体系 (Components)
| 模块 | 文件 | 职责 |
|------|------|------|
| Tool | `tool.rs` | 工具注册与调用 |
| Knowledge | `knowledge.rs` | 知识库检索 |
| Skill | `skill.rs` / `skill_manifest.rs` | SKILL.md 标准化 |
| Persona | `persona.rs` | 52 个预置人格模板 |
| Memory | `memory.rs` / `memory_three_layer.rs` | 三层记忆 + 自编辑 |
| Model | `model.rs` | 模型配置与切换 |

### 3. 创作台 (Studio)
| 模块 | 文件 | 职责 |
|------|------|------|
| StudioManager | `manager.rs` | 组件增删改查 |
| StudioTester | `tester.rs` | 组件测试（TestRunner） |
| StudioPublisher | `publisher.rs` | 发布到市场 |
| AgentAssembly | `assembler.rs` | Agent 组装器 |

### 4. 渠道与通信 (Channels)
| 模块 | 文件 | 职责 |
|------|------|------|
| ChannelAdapter | `adapter.rs` | 统一消息适配器 |
| CLI | `cli.rs` | 终端 REPL |
| 企业微信 | `wecom.rs` | 企微信道 |
| 钉钉 | `dingtalk.rs` | 钉钉信道 |
| 飞书 | `feishu.rs` | 飞书信道 |
| REST API | `rest_api.rs` | axum 6 端点 |
| SMTP | `smtp.rs` | 邮件通知 |

### 5. Agent 高级能力
| 模块 | 文件 | 职责 |
|------|------|------|
| 三阶段 Agent | `triphase_agent.rs` | Plan→Implement→Review |
| 主管调度 | `orchestrator.rs` | 动态主管-专家调度 |
| Agent 集群 | `agent_pool.rs` | 大规模 Agent 管理 |
| 信任评分 | `trust_scorer.rs` | Agent 信任评级 |
| 共识协作 | `consensus.rs` | 多 Agent 共识接力 |
| Agent 组装 | `assembler.rs` | NL→Agent 组装 |

### 6. 记忆系统
| 模块 | 文件 | 职责 |
|------|------|------|
| 三层记忆 | `memory_three_layer.rs` | Working / Episodic / Semantic |
| 自编辑记忆 | `memory_self_edit.rs` | 记忆修正/压缩/合并 |

### 7. 平台功能
| 模块 | 文件 | 职责 |
|------|------|------|
| REST API | `rest_api.rs` | axum 服务器 (6 端点) |
| 看板调度 | `kanban.rs` | 待办→进行中→审查→完成 |
| Code-as-Tool | `code_tool.rs` | 沙箱执行 Python/Shell |
| 搜索启动器 | `search_launcher.rs` | Alt+Space 快速搜索 |
| 模板商店 | `template_store.rs` | 工作流模板 CRUD |

### 8. 认知与智能
| 模块 | 文件 | 职责 |
|------|------|------|
| PikoSoul 性格引擎 | `personality_engine.rs` | 五维性格分析 |
| PC Tracker 录制 | `demo_recorder.rs` | 操作录制/回放 |
| NL→Workflow | `nl_workflow.rs` | 自然语言转工作流 |
| SOP→Prompt | `sop_template.rs` | 标准作业程序转提示 |

### 9. 高级能力
| 模块 | 文件 | 职责 |
|------|------|------|
| 视觉 GUI 操控 | `visual_agent.rs` | VLM 截图元素检测 |
| 跨渠道身份 | `identity_bridge.rs` | IM 用户统一绑定 |
| 3D 可视化 | `visualization_3d.rs` | 力导向图数据 |
| 超长任务引擎 | `long_task_engine.rs` | 13h+ 断点续传 |
| Office 处理 | `office_handler.rs` | PPT/Excel (纯 Rust) |
| Cortex 引擎 | `cortex_engine.rs` | MCP 模型仓库 |
| 社区模板市场 | `community_templates.rs` | 远程模板仓库 |

### 10. 安全与治理
| 模块 | 文件 | 职责 |
|------|------|------|
| SecurityGuard | `security.rs` | 4 层宪法安全 |
| 隐私闸门 | `privacy_gate.rs` | 敏感数据过滤 |
| 审计日志 | `audit.rs` | 操作记录 |
| 权限检查 | `permissions.rs` | 用户/角色权限 |
| Dual-LLM | `dual_llm.rs` | 双重审查 |

## 技术栈

| 层 | 技术 |
|----|------|
| 语言 | Rust 2021 edition |
| 桌面框架 | Tauri v2 (NSIS 安装器 + 自动更新) |
| 前端 | React 18 + TypeScript + Vite 5 |
| LLM 协议 | OpenAI 兼容 API (默认 DeepSeek) |
| 存储 | SQLite (rusqlite) |
| HTTP | reqwest + axum |
| 协议 | MCP (Model Context Protocol) |
| 前端组件 | reactflow (节点编辑器) |

## 构建状态

| 指标 | 数值 |
|------|------|
| cargo build | 0 errors, 3 minor warnings |
| cargo test | 417 passed, 0 failed |
| npm run build | 0 errors |
| 总代码量 | ~13,000+ 行 Rust |
| 前端代码 | ~2,000+ 行 TypeScript/React |
