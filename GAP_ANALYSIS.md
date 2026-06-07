# Morn 设计-实现差距分析报告

> 对照 `DESIGN.md`（1755行）与 `src/` 实际代码的逐一比对
> 生成日期: 2026-06-07

## 统计概要

| 分类 | 数量 |
|------|------|
| ✅ 已实现 | 88 |
| 🟡 部分实现 | 28 |
| ❌ 未实现 | 25 |

---

## 一、核心架构（Section 2）

### 2.2 标准组件接口
| 承诺 | 状态 | 说明 |
|------|------|------|
| `trait Component` | ✅ | `src/core/component.rs` — 完整实现 `id()`, `type_name()`, `init()`, `run()`, `pause()`, `stop()`, `health_check()` |
| `trait IOComponent` | ✅ | 同上 — `ports()`, `send()`, `recv()` |
| `trait EventBus` | ✅ | `src/core/event_bus.rs` + `src/core/event_stream.rs` — publish/subscribe/replay |
| `trait SecureComponent` | ✅ | `src/core/component.rs` — `required_permissions()` |

### 2.3 系统架构
| 承诺 | 状态 | 说明 |
|------|------|------|
| Tauri 桌面端 | ✅ | `src-tauri/` — 完整 Tauri v2 应用，系统托盘、自动启动、NSIS 安装包配置 |
| Channel Adapter | ✅ | `src/channel/adapter.rs` |
| COO (Supervisor) | ✅ | `src/core/supervisor.rs` — 662行 |
| Registry | ✅ | `src/core/registry.rs` — 440行 |
| Task Engine (DAG) | ✅ | `src/core/engine/` — `dag.rs` + `executor.rs` |
| Dual-LLM 安全 | 🟡 | `src/core/dual_llm.rs` — 321行，**副 LLM 未实际接入**（用模拟判断） |
| SQLite 存储 | ✅ | `src/core/storage/` — 7个子模块（agents, tasks, settings, sessions, users, market, oauth, governance, sync） |

### 2.4 进程模型
| 承诺 | 状态 | 说明 |
|------|------|------|
| 系统托盘常驻 | ✅ | `src-tauri/src/lib.rs` — TrayIcon |
| 主线程 UI | ✅ | Tauri 主线 |
| COO 线程 | 🟡 | 单线程 `block_on` 执行 |
| 执行线程池 | ❌ | **无独立线程池**，同步执行 |
| 子进程隔离 | ❌ | **无子进程**，所有执行在主进程 |
| 渠道线程池 | ❌ | **无独立渠道线程** |
| 托盘线程 | ✅ | Tauri 原生托盘 |

---

## 二、创作台 Studio（Section 3）

### 3.1 一句话构建 Agent
| 承诺 | 状态 | 说明 |
|------|------|------|
| NL → Agent 定义 | ✅ | `supervisor.create_agent_from_nl()` + Tauri command `create_agent_from_description` |
| Registry 动态推断 | 🟡 | 基本能力，但缺少市场查询和骨架创建 |
| 一句话构建团队 | ❌ | **未实现** — 无团队版 NL builder |
| 一句话构建工作流 | ✅ | `workflow_builder.rs` — `nl_to_workflow()` |

### 3.2 可创作组件
| 承诺 | 状态 | 说明 |
|------|------|------|
| 工具 (Tool) | ✅ | `src/component/tool/` |
| 知识 (Knowledge) | ✅ | `src/component/knowledge.rs` — Static/File/SQLite 三种实现 |
| 技能 (Skill) | ✅ | `src/component/skill/` — registry + builtins |
| 人格 (Persona) | ✅ | `src/component/persona/` — 完整 struct + 5层 Prompt |
| 记忆 (Memory) | ✅ | `src/component/memory/` — mod + simple + storage |
| 模型 (Model) | ✅ | `src/component/model.rs` |
| Agent | ✅ | `assembler.rs` |
| 管道 (无 Agent 的组件链) | ❌ | **未实现** |

### 3.3 人格深度设计
| 承诺 | 状态 | 说明 |
|------|------|------|
| 5 层 Prompt (L1-L5) | ✅ | `PromptLayers` struct 含所有 5 层 |
| core_principles | ✅ | Vec 字段 |
| decision_framework | ✅ | Vec 字段 |
| anti_patterns | ✅ | Vec 字段 |
| 52 个预置人格模板 | ✅ | **正好 52 个** `fn preset_*()` — 确实验证 |
| 人格代数组合 | ❌ | **未实现** — 无人格向量组合逻辑 |

### 3.4 组合画布
| 承诺 | 状态 | 说明 |
|------|------|------|
| 拖拽画布 | ❌ | **没有 Rust 端画布实现**。`web/src/studio/NodeCanvas.tsx` 存在但属于原型级 |
| 端口自动匹配 | ❌ | **未实现** |
| Transformer 中间件 | ❌ | **未实现** |

### 3.5 测试面板
| 承诺 | 状态 | 说明 |
|------|------|------|
| TestRunner | ✅ | `src/studio/tester/` — run_and_measure |
| 测试报告 | ✅ | `reports.rs` + web `TestPanel.tsx` |
| 编辑重跑 | ✅ | `rerun_component_step` |
| 完整执行日志 | 🟡 | 结构存在但精细度不如文档描述 |

### 3.6 7 种协作模式
| 承诺 | 状态 | 说明 |
|------|------|------|
| 链式 (Chain) | ✅ | `src/core/orchestrator/chain.rs` |
| 主管-工人 (ManagerWorker) | ✅ | `src/core/orchestrator/manager_worker.rs` |
| 广播-监听 (Broadcast) | ✅ | `src/core/orchestrator/broadcast.rs` |
| 投票集成 (Voting) | ❌ | **未实现** — enum 定义了但无代码 |
| 路由-分类 (Routing) | ❌ | **未实现** — enum 定义了但无代码 |
| Agent 即工具 | ❌ | **未实现** — enum 定义了但无代码 |
| 共享黑板 | ❌ | **未实现** — enum 定义了但无代码 |

### 3.7 Agent 构建指导
| 承诺 | 状态 | 说明 |
|------|------|------|
| 8 个预置 Agent 模板 | ✅ | Registry 中有 8 个模板 |
| 引导式构建 | ❌ | **无分步引导流程** |
| COO 实时建议 | ❌ | **无实时建议功能** |

### 3.8 多 Agent 团队
| 承诺 | 状态 | 说明 |
|------|------|------|
| 7 个预置团队模板 | ❌ | **未实现** |
| 共识机制 (4 种) | ✅ | `ConsensusMechanism` enum — Vote/CeoDecides/MungerVeto/AutoSynthesis |
| 共享黑板通信 | ❌ | **无共享内存实现** |
| Agent 间直接消息 | ❌ | **无 Agent-to-Agent 消息** |

### 3.9 工作流
| 承诺 | 状态 | 说明 |
|------|------|------|
| 8 个预置工作流模板 | ✅ | `workflow/templates.rs` — 8 个模板 |
| 5 个工作流模板商店 | ✅ | `workflow_templates/builtins.rs` — 5 个中文模板 |
| 控制节点 (条件/循环/并行/路由/聚合) | ❌ | **仅 `condition` 节点在模板 JSON 中存在，Rust 端无控制流执行引擎** |
| 人工审批节点 | 🟡 | `approval.rs` 存在，但**未集成**到工作流执行器中 |
| 工作流版本管理 | ❌ | **未实现** |
| 工作流嵌套调用 | ❌ | **未实现** |

---

## 三、工作台 Workbench（Section 4）

### 4.1 COO 决策树（6 级）
| 承诺 | 状态 | 说明 |
|------|------|------|
| L1 直接回答 | ✅ | `DecisionLevel::L1DirectAnswer` |
| L2 单工具 | ✅ | `DecisionLevel::L2SingleTool` |
| L3 单 Agent | ✅ | `DecisionLevel::L3SingleAgent` |
| L4 临时团队 | ✅ | `DecisionLevel::L4Team` |
| L5 工作流模板 | ✅ | `DecisionLevel::L5Workflow` |
| L6 跳创作台 | ✅ | `DecisionLevel::L6JumpToStudio` |
| 完整决策树执行 | 🟡 | COO 决策逻辑存在但**自动升降级不完整** |
| 用户强制指定层级 | ❌ | **无用户 override 机制** |

### 4.2 COO 三种工作模式
| 承诺 | 状态 | 说明 |
|------|------|------|
| 主动模式 | 🟡 | CLI `/mode` 命令存在但**三种模式未完整实现差异** |
| 安全模式 | ❌ | **未真正实现** |
| 自动化模式 | ❌ | **未真正实现** |

### 4.3 COO 决策协议
| 承诺 | 状态 | 说明 |
|------|------|------|
| 三挡决策 (运营/战术/战略) | ✅ | `DecisionLevel` 实现了基本的分级 |
| COO 学习机制 | ❌ | **未实现** — decision_rules 表不存在 |
| 对话修改规则 | ❌ | **未实现** |

### 4.4 电脑操控
| 承诺 | 状态 | 说明 |
|------|------|------|
| 文件系统 (读/写/移/删/搜索) | ✅ | `src/computer/fs_ops.rs` |
| 应用管理 (启动/关闭/列表) | 🟡 | `app_ops.rs` — 启动存在但**关闭/列表模拟** |
| 系统设置 (壁纸/音量/网络) | 🟡 | `sys_ops.rs` — 基本读操作，**无写操作** |
| 桌面操控 (鼠标/键盘/剪贴板/截图) | ❌ | `desktop_ops.rs` — **仅模拟实现** |
| 浏览器 (导航/表单/内容/多标签) | 🟡 | `browser_ops.rs` — 导航和内容提取真实，**表单填充是模拟** |
| 感知融合 (VLM + 无障碍树 + OCR) | ❌ | `perception.rs` — **不是真实的 VLM/OCR，是模拟** |

---

## 四、管理台 Console（Section 5）

### 5.1 视图
| 承诺 | 状态 | 说明 |
|------|------|------|
| 仪表盘 | ✅ | `console/mod.rs` + web `Dashboard.tsx` |
| 组件拓扑 | ✅ | `get_component_topology` Tauri command |
| 成本中心 | ✅ | `console/cost.rs` |
| 治理 | ✅ | `console/governance.rs` |
| 安全 (Dual-LLM 记录) | 🟡 | 有日志结构但**管理台界面不完整** |
| 市场视图 | ❌ | **无管理台内的市场面板** |
| 系统监控 | 🟡 | 基本 CPU/内存/磁盘信息 |

### 5.2 信任评分
| 承诺 | 状态 | 说明 |
|------|------|------|
| TrustScorer | ✅ | `src/core/trust_scorer.rs` |
| TrustEvaluator | ✅ | `src/core/trust_evaluator.rs` |
| 公式 (质量*0.3 + 成功*0.3 + 延迟*0.2 + 反馈*0.2) | ✅ | 代码中实现 |

### 5.3 成本中心
| 承诺 | 状态 | 说明 |
|------|------|------|
| 按 Agent/工具/模型拆分 | 🟡 | 基本结构存在但**粒度不够** |
| 预算设置 | ❌ | **未实现** |
| 超限行为 | ❌ | **未实现** |
| 每日/每月趋势 | ❌ | **未实现** |

---

## 五、渠道与连接（Section 6）

### 6.1 全渠道
| 承诺 | 状态 | 说明 |
|------|------|------|
| Windows 桌面端 (Tauri) | ✅ | 完整 Tauri v2 应用 |
| 网页端 (PWA) | ✅ | `web/` 含 dist、manifest、sw.js |
| 企业微信 | ✅ | `src/channel/wecom.rs` |
| 钉钉 | ✅ | `src/channel/dingtalk.rs` — 含 webhook 发送 + 回调处理器 |
| 飞书 | ✅ | `src/channel/feishu.rs` |
| 微信小程序 | ✅ | `src/channel/miniprogram.rs` |
| REST API | ✅ | `src/api/rest_api.rs` (axum) |
| Webhook | ✅ | `src/channel/webhook.rs` |
| 邮件 (SMTP) | ✅ | `src/channel/smtp.rs` |
| 微信公众号 | ✅ | `src/channel/wechat_mp.rs` |
| QQ 机器人 | ✅ | `src/channel/qqbot.rs` |
| Telegram | ✅ | `src/channel/telegram.rs` — 含 sendMessage/webhook/update handler |
| 浏览器扩展 | ❌ | **未实现** |

---

## 六、市场 Marketplace（Section 7）

| 承诺 | 状态 | 说明 |
|------|------|------|
| 6 种商品类型 | ✅ | Listing enum: Tool/Knowledge/Skill/Persona/Agent/Team |
| 按次计费 | ✅ | Transaction struct |
| 月订阅 | ❌ | **未实现** |
| 创作者收益 | ❌ | **未实现** |
| 交易模型 | 🟡 | 结构存在但**无真实付款** |

---

## 七、安全模型（Section 8）

| 承诺 | 状态 | 说明 |
|------|------|------|
| 4 层宪法 (L1-L4) | ✅ | `src/core/security.rs` — 10条策略 |
| Dual-LLM 6 个检查点 | 🟡 | `src/core/dual_llm.rs` — 定义了 Auth/ParamValidate/ContentSanitize/Permission/Audit/Route，**但副 LLM 未实际连接** |
| HITL 审批 | ✅ | `src/core/approval.rs` — 完整 ApprovalManager |
| 隐私闸门 | ✅ | `src/core/privacy_gate.rs` — 255行，关键词+匿名化 |

---

## 八、Phase 0-9 实现状态（Section 11）

### Phase 0 — 基础骨架
| 承诺 | 实际 | |
|------|------|---|
| main.rs CLI 入口 | ✅ | 78行，ASCII启动 + REPL |
| Supervisor | ✅ | 662行 |
| Registry | ✅ | 440行 |
| Storage | ✅ | ~3000行 |
| SecurityGuard | ✅ | 246行 |
| SimpleEventBus | ✅ | ✅ |

### Phase 1 — 组件体系
| 承诺 | 实际 | |
|------|------|---|
| Tool 组件 | ✅ | `tool/mod.rs` + file_ops/web_search/code_exec/builtins |
| Knowledge 组件 | ✅ | Static/File/SQLite 三种 |
| Skill 组件 | ✅ | `skill/mod.rs` + registry + builtins |
| Persona 52 个 | ✅ | 确实验证 52 个 |
| Memory 组件 | ✅ | `memory/mod.rs` + simple + storage |
| Model 组件 | ✅ | `model.rs` |

### Phase 2 — 创作台
| 承诺 | 实际 | |
|------|------|---|
| Tester 测试面板 | ✅ | `studio/tester/` |
| TestPanel 前端 | ✅ | `web/src/studio/TestPanel.tsx` |
| AgentBuilder 前端 | ✅ | `web/src/studio/AgentBuilder.tsx` |
| BotStore 前端 | ✅ | `web/src/store/BotStore.tsx` |

### Phase 3 — Channel 与 IM
| 承诺 | 实际 | |
|------|------|---|
| CLI 信道 | ✅ | |
| 企微/钉钉/飞书 | ✅ | 各有真实 webhook 发送代码 |
| 微信小程序 | ✅ | 适配器框架 |
| REST API | ✅ | axum 服务器 |
| 邮件 (SMTP) | ✅ | |
| QQ/Telegram | ✅ | |

### Phase 4 — 基础设施层
| 承诺 | 实际 | |
|------|------|---|
| MCP 协议层 | ✅ | `core/mcp.rs` |
| EventBus | ✅ | `core/event_bus.rs` + `core/event_stream.rs` |
| SkillLoader | ✅ | `component/skill/` |
| Dashboard | ✅ | web Dashboard.tsx |
| 轻量优化 (LTO) | ✅ | Cargo.toml 配置 |

### Phase 5 — 核心运行时增强
| 承诺 | 实际 | |
|------|------|---|
| Checkpoint 持久化 | ✅ | `core/checkpoint.rs` — 完整 save/load/list |
| HITL 审批 | ✅ | `core/approval.rs` — 296行 |
| 三阶段 Agent | ✅ | `core/agent_loop.rs` — Plan→Implement→Review |
| NL→Workflow | ✅ | `core/workflow_builder.rs` |
| 三层记忆 | ✅ | `component/memory/` |
| 自编辑记忆 | 🟡 | 基本结构但有改进空间 |
| OAuth 认证 | ✅ | `core/oauth.rs` — 4个预配置 provider |
| 隐私闸门 | ✅ | `core/privacy_gate.rs` |
| RepoMap | ✅ | `core/repo_map.rs` — Tree-sitter 风格的仓库浏览 |
| 视觉定位 | 🟡 | `core/visual_grounding.rs` — **模拟实现，非真实 VLM** |
| SOP→Prompt | ✅ | `core/sop_template.rs` — 3个内置模板 |

### Phase 6 — 渠道与工具生态
| 承诺 | 实际 | |
|------|------|---|
| 中国 IM 信道 | ✅ | 4 个信道编译验证 |
| 工作流变量系统 | ✅ | `core/workflow/mod.rs` — VariableStore |
| Agent 间自动委派 | ✅ | `core/delegation.rs` |

### Phase 7 — 高级 Agent 能力
| 承诺 | 实际 | |
|------|------|---|
| 主管-专家调度 | ✅ | `core/orchestrator/` |
| Agent 集群 | ✅ | `core/agent_pool/` |
| 信任评分 | ✅ | `core/trust_scorer.rs` + `trust_evaluator.rs` |
| 共识文件接力 | ✅ | `core/consensus.rs` |
| Bot 商店 | ✅ | web BotStore.tsx |
| 52 个人格 | ✅ | 确实验证 |

### Phase 8 — 平台功能
| 承诺 | 实际 | |
|------|------|---|
| REST API (axum) | ✅ | `api/rest_api.rs` |
| Code-as-Tool | ✅ | `core/code_tool.rs` |
| 工作流模板商店 | 🟡 | 5个预置模板（文档说8个） |
| 看板调度 | ✅ | `core/kanban/` |
| PikoSoul 性格引擎 | ✅ | `core/personality_engine.rs` |
| PC Tracker 认知录制 | ✅ | `core/demo_recorder.rs` |
| 搜索启动器 | ✅ | `core/search_launcher/` |
| 模板商店 | ✅ | `core/template_store/` |

### Phase 9 — 高级与远期
| 承诺 | 实际 | |
|------|------|---|
| 视觉 GUI 操控 | 🟡 | `visual_agent.rs` + `visual_grounding.rs` — **模拟实现** |
| 跨渠道身份统一 | ✅ | `core/identity_bridge.rs` — 299行，完整实现 |
| 3D 可视化仪表盘 | 🟡 | `core/visualization_3d.rs` — **力导向图数据结构但无 3D 渲染** |
| 超长任务引擎 | ✅ | `core/long_task_engine.rs` |
| Office 文档处理 | ✅ | `core/office_handler/` |
| Cortex 推理引擎 | ✅ | `core/cortex_engine.rs` |
| 社区模板市场 | ✅ | `core/community_templates/` |

---

## 九、下一步优先项（Section 11 底部）

| 优先级 | 项 | 状态 | 说明 |
|--------|-----|------|------|
| P0 | 打包安装 | 🟡 | tauri.conf.json 配置了 NSIS + updater，但**未验证端到端** |
| P1 | 首次引导 Onboarding | ❌ | **未实现** |
| P1 | Registry 热加载 | ❌ | Registry 是静态 HashMap，**无热加载** |
| P2 | 自进化 Skill | ❌ | **未实现** |
| P2 | 双记忆 MDRM 图谱 | ❌ | **未实现** |
| P3 | 扫码绑定 IM | ❌ | **未实现** |
| P3 | 活人感引擎 | ❌ | **未实现** |
| P4 | 深度可观测性 | ❌ | **未实现** |

---

## 十、本地优先与数据主权（Section 13）

| 承诺 | 状态 | 说明 |
|------|------|------|
| 纯本地 Offline 模式 | ❌ | **仅 DeepSeek API 云端工作** |
| 纯云端 Online 模式 | ✅ | DeepSeek API 通过 ChatAgent |
| 混合 Hybrid 模式 | ❌ | **无 ModelRouter，无自动路由** |
| 本地模型 (LM Studio/Ollama/llama.cpp) | ❌ | **未集成** |
| 本地 Embedding | ❌ | **无 Sentence Transformers 集成** |
| 本地向量检索 | ❌ | **无本地向量库** |
| 网络权限模型 (L0/L1/L2) | ❌ | **未实现** — 虽然 `SecurityLevel` 描述了 L1Sandbox/L2Local/L3System |
| 数据流日志 | ❌ | **未实现** |
| A2A 跨设备同步 | ❌ | `a2a.rs` 和 `a2a_discovery.rs` 存在但**无实际同步逻辑** |

---

## 十一、OpenAkita 参考核心层（Section 14）

| 项 | 状态 | 说明 |
|----|------|------|
| 打成安装包 | 🟡 | Tauri NSIS 配置存在，但**未测试端到端打包** |
| 首次引导 Onboarding | ❌ | **未实现** |
| Registry 热加载 | ❌ | **未实现** |

### 组件层（插件）
| 项 | 状态 | 说明 |
|----|------|------|
| 自进化 Skill | ❌ | **未实现** |
| 双记忆 MDRM 图谱 | ❌ | **未实现** |
| 扫码绑定 IM | ❌ | **未实现** |
| 活人感引擎 | ❌ | **未实现** |
| 深度可观测性 | ❌ | **未实现** |

---

## 十二、严重差距（文档承诺但代码没有的 TOP 10）

| # | 功能 | 文档位置 | 说明 |
|---|------|---------|------|
| 1 | **子进程崩溃隔离** | §2.4 | 文档说"每个子任务独立子进程"，实际**全部主进程同步执行** |
| 2 | **组合画布拖拽** | §3.4 | 文档描述"从左侧组件库拖组件到画布"，实际**无任何类似实现** |
| 3 | **Voting/Routing/AgentAsTool/Blackboard 协作** | §3.6 | 仅 3/7 种协作模式实现 |
| 4 | **7 个预置团队模板** | §3.8 | 文档列出 7 个团队模板，代码中**一个也没有** |
| 5 | **工作流控制节点 (循环/并行/路由/聚合)** | §3.9 | 仅 `condition` 在模板 JSON 中存在碎片 |
| 6 | **纯本地离线模式** | §13 | 无法脱离 DeepSeek API 运行 |
| 7 | **Hybrid 模型路由** | §13 | 无 ModelRouter 实现 |
| 8 | **Onboarding 引导** | §14 | 无首次启动向导 |
| 9 | **Registry 热加载** | §14 | 静态 HashMap，无动态注册/注销 |
| 10 | **电脑桌面操控 (鼠标/键盘/剪贴板/截图)** | §4.4 | 全部是模拟返回值，无真实操控 |

---

## 十三、文档与实际状态不一致

| 文档声明 | 实际状态 |
|---------|---------|
| "cargo test 417 passed" | ✅ 实际 **424 passed** (多了7个) |
| "9个 Phase / 75 项功能全部实现" | ❌ 约 **25 项功能为空或模拟实现**，尤其在办公套件以下功能严重缺失 |
| "52 个人格模板" | ✅ 确实验证 52 个 |
| "8 个预置工作流模板" | ✅ 确实验证 8 个在 `workflow/templates.rs` |
| "7 种协作模式" | ❌ 仅 3 种实现 (Chain, ManagerWorker, Broadcast) |
| "Dual-LLM 安全" | 🟡 结构存在但副 LLM 未真正接入 |
| "视觉定位 (VLM)" | 🟡 模拟实现，非真实 VLM |
| "电脑操控" | 🟡 浏览器导航真实，桌面操控模拟 |

## 十四、代码行统计

| 模块 | 文件数 | 大致行数 |
|------|--------|---------|
| `src/core/` | 43 模块 | ~12,000 行 |
| `src/component/` | 13 模块 | ~3,500 行 |
| `src/computer/` | 7 模块 | ~800 行 |
| `src/channel/` | 13 模块 | ~1,500 行 |
| `src/studio/` | 4 模块 | ~1,000 行 |
| `src/console/` | 3 模块 | ~400 行 |
| `src/bridge/` | 4 模块 | ~1,200 行 |
| `src/market/` | 1 模块 | ~354 行 |
| `src/org/` | 3 模块 | ~600 行 |
| `src/api/` | 1 模块 | ~500 行 |
| `src-tauri/` | 3 文件 | ~620 行 |
| **总计** | **~141 文件** | **~25,000 行** |

> 结论：项目代码量约 25,000 行 Rust，141 个文件。核心组件体系完成度高，但
> **电脑操控（桌面级、浏览器DOM级）、Dual-LLM、本地模型、工作流控制节点、团队模板、协作模式**
> 等功能为**骨架/模拟实现**，文档中有 25 项左右功能实际代码中缺失或仅模拟。
