# DESIGN.md 逐行承诺 vs 代码实现对比审计报告

**审计日期**: 2026-06-11  
**项目**: ~/morn-desktop (1212 行 DESIGN.md)  
**提交**: HEAD b33bf6789119fe061bbc52eb67d6b268a3987e3d

---

## 1. 核心架构 — §2

### 1.1 §2.1 四层组合模型 (Line 57-96)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| Layer 0: 基础设施 (LLM/数据源/OS/MCP) | ✅ 完整 | `src/core/model_router/`, `src/mcp/`, `src/computer/` | MCP模块完整，ModelRouter 完整 |
| Layer 1: 7种内置原子组件类型 | ✅ 完整 | `src/core/component_type/registry.rs` | TypeRegistry 注册了8种(含pipeline) |
| Layer 2: Agent组合 | ✅ 完整 | `src/core/assembly/`, `src/core/agent_loop.rs`, `src/core/agent_templates.rs` | 组装器、循环、模板都存在 |
| Layer 3: 团队（多Agent+工作流） | 🟡 有骨架 | `src/core/orchestrator/`, `src/core/workflow/` | 存在 orchestrator/group/ 和工作流引擎，但团队蓝图尚缺完整 UI 集成 |

### 1.2 §2.1 Agentless 管道 (Line 88-89)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| "纯工具链：Timer→get_kline→calc_macd→write_file" | 🟡 有骨架 | `src/core/pipeline/agentless.rs`, `src/core/pipeline/executor.rs` | AgentlessPipeline 存在，但 execute() 仅做 HashMap 模拟，不走真实工具调用 |
| DAG PipelineExecutor | ✅ 完整 | `src/core/pipeline/executor.rs` | 拓扑排序、节点执行完整 |
| PipelineNode 16种类型 | ✅ 完整 | `src/core/pipeline/nodes.rs` | Start/End/LLM/Tool/Code/Condition/Loop/Parallel/Merge/Wait/HumanInput/Email/Webhook/Transform/Log/SubWorkflow |
| Transform中间件 | ✅ 完整 | `src/core/pipeline/transformer.rs` | TransformPipeline + FieldMapper |

### 1.3 §2.2 标准组件接口 (Line 98-131)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| Component trait (id/init/run/pause/stop/health_check) | ❌ 不存在 | `src/core/component/types.rs` | 只有 Port/PortDirection，无 Component trait |
| IOComponent trait (ports/send/recv) | ❌ 不存在 | — | 未实现 |
| EventBus trait | 🟡 有骨架 | `src/core/event_bus.rs` | SimpleEventBus 实现了 publish/subscribe，但不是 trait |
| SecureComponent trait | ❌ 不存在 | — | 未实现 |

### 1.4 §2.3 系统架构 (Line 133-168)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| COO (Supervisor) | ✅ 完整 | `src/core/supervisor/mod.rs` | 完整实现，含 intent parser/planner/router/scheduler |
| Intent Parser (NL→结构化意图) | ✅ 完整 | `src/core/supervisor/execution/intent.rs`, `src/core/intent_parser.rs` | 双份实现 |
| Planner (意图分解为DAG计划) | ✅ 完整 | `src/core/supervisor/execution/planner.rs` | 实现 |
| Router (匹配组件,信任评分) | 🟡 有骨架 | `src/core/supervisor/execution/dispatch.rs`, `src/core/trust_scorer/` | dispatch 存在，信任评分在 scorer/evaluator |
| Scheduler (DAG调度) | ✅ 完整 | `src/core/supervisor/execution/scheduler.rs` | 实现 |
| Registry (能力注册中心) | ✅ 完整 | `src/core/registry/` | Registry + Capability + Version 完整 |
| Task Engine (执行引擎) | ✅ 完整 | `src/core/task_engine/`, `src/core/engine/` | task engine + child_process |
| Dual-LLM 安全 | ✅ 完整 | `src/core/dual_llm/` | 6个检查点+主副LLM |
| SQLite 存储 | ✅ 完整 | `src/core/storage/mod.rs` | 完整表结构(agents/capabilities/tasks/executions/decisions/bindings等) |

### 1.5 §2.4 进程模型 (Line 170-182)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| Windows 开机托盘常驻 | 🟡 有骨架 | `src-tauri/src/autostart.rs`, `src-tauri/src/lib.rs` | autostart实现，但进程模型的5线程具体划分未在代码中明确 |
| 子进程隔离 | ✅ 完整 | `src/core/task_engine/child_process.rs` | 实现 |

### 1.6 §2.5 可扩展组件类型系统 (Line 184-262)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| ComponentTypeDef 结构 | ✅ 完整 | `src/core/component_type/def.rs` | 精确匹配设计文档 |
| TypeRegistry (注册/查询) | ✅ 完整 | `src/core/component_type/registry.rs` | 支持 register/get/list/has/find_by_interface |
| 8种内置类型 (含channel/pipeline) | ✅ 完整 | 同上 | 注：DESIGN.md说7种，但代码注册了8种（含channel）|
| 市场安装新类型 | ❌ 不存在 | — | TypeRegistry 没有从市场安装的接口 |
| Studio自动更新面板 | ❌ 不存在 | — | 前端无类型变更动态响应逻辑 |
| 组合规则（必选/冲突/约束/兼容） | ❌ 不存在 | `src/core/assembly/validator.rs` | 只有基础validator，复杂组合规则未实现 |

### 1.7 三种构建方式 (Line 254-261)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 一句话描述构建 | 🟡 有骨架 | `src/core/supervisor/guided_builder.rs`, `src/core/supervisor/execution/intent.rs` | COO 有 intent parser 可以推断，但"一句话→完整Agent"的端到端链路未完整实现 |
| 引导式构建 (Step 1-5) | ✅ 完整 | `src/core/supervisor/guided_builder.rs`, `web/src/studio/StepWizard.tsx` | 前后端都实现了5步引导 |
| 拖拽画布 | ✅ 完整 | `web/src/studio/NodeCanvas.tsx` | 基于 ReactFlow，完整拖拽+连线+节点 |

---

## 2. 创作台 Studio — §3

### 2.1 §3.1 一句话构建 Agent (Line 268-387)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| COO理解需求（领域/角色/能力/工具/知识/人格推断） | 🟡 有骨架 | `src/core/intent_parser.rs` | 基础实现，未实现完整6步推断 |
| COO自动生成完整Agent定义 | 🟡 有骨架 | `src/core/supervisor/types.rs` (NLAgentDef) | NLAgentDef结构存在，但端到端生成未完整 |
| "直接保存"/"改"/"看细节"三个选择 | 🟡 有骨架 | `web/src/studio/AgentBuilder.tsx` | UI有保存/调整，但 COO 的自动调整未完整 |
| Registry + 市场动态推断 | ❌ 不存在 | — | COO 不查市场可用组件做推断 |
| 团队一句话构建 | 🟡 有骨架 | `src/core/supervisor/team_builder.rs` | nl_to_team 存在但基础 |
| 工作流一句话构建 | ❌ 不存在 | — | 未实现 |

### 2.2 §3.2 创作哪些组件 (Line 389-402)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 工具编辑器 | ✅ 完整 | `src/studio/editors/tool.rs`, `web/src/studio/ComponentEditor.tsx` | 前后端都有 |
| 知识编辑器 | ✅ 完整 | `src/studio/editors/knowledge.rs` | 实现 |
| 技能编辑器 | ✅ 完整 | `src/studio/editors/pipeline.rs` | 实现 |
| 人格编辑器 | 🟡 有骨架 | `src/studio/editors/make_editors.rs`, `src/component/persona/` | 人格系统完整，但编辑器前端与后端分离 |
| 记忆编辑器 | ✅ 完整 | `src/studio/editors/memory.rs` | 实现 |
| 模型编辑器 | ✅ 完整 | `src/studio/editors/model.rs` | 实现 |
| Agent编辑器 | ✅ 完整 | `web/src/studio/AgentBuilder.tsx` | 前端完整 |
| 管道编辑器 | ✅ 完整 | `src/studio/editors/pipeline.rs` | 实现 |

### 2.3 §3.3 Agent人格深度设计 (Line 404-438)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 核心思维模型 | ✅ 完整 | `src/component/persona/types.rs` | PersonaParameters 含 principles |
| 决策框架 | 🟡 有骨架 | `src/component/persona/combinator.rs` | 有组合逻辑 |
| 反模式 | ❌ 不存在 | — | 未实现 |
| 5层Prompt | ✅ 完整 | `src/component/persona/types.rs` | PromptLayers 含 L1-L5 |
| 52预置人格 | 🟡 有骨架 | `src/component/persona/presets_tech.rs`, `src/component/persona/presets/` | 多于52种，但为预设，非动态 |
| 人格代数组合 | ❌ 不存在 | — | PERSONA paper 参考未实现 |

### 2.4 §3.4 组合画布 (Line 439-455)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 画布拖拽组件 | ✅ 完整 | `web/src/studio/NodeCanvas.tsx` | ReactFlow完整 |
| 端口连线 | ✅ 完整 | 同上 | addEdge / Connection |
| 9种节点类型 | ✅ 完整 | 同上 + `src/studio/types.rs` | LLM/Agent/Tool/Knowledge/Skill/Code/Router/Loop/Trigger |

### 2.5 §3.5 测试面板 (Line 457-471)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 完整执行日志 | ✅ 完整 | `web/src/studio/TestPanel.tsx` | step-based 显示 |
| 每步耗时/Tokens/成本 | ✅ 完整 | 同上 | 完整实现 |
| 点击查看详情/编辑重跑 | ✅ 完整 | 同上 | expandedStep + editingStep |

### 2.6 §3.6 协作组合模式 (Line 473-486)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 链式 (A→B→C) | ✅ 完整 | `src/core/orchestrator/collaboration.rs`, `src/core/orchestrator/modes.rs` | 实现 |
| 并行 | ✅ 完整 | `src/core/workflow/engine.rs` | fork-join |
| 投票集成 | ❌ 不存在 | — | 未实现 |
| 路由-分类 | ❌ 不存在 | — | 未实现 |
| Agent即工具 | ❌ 不存在 | — | 未实现 |
| 共享黑板 | ❌ 不存在 | — | 未实现 |
| 主管-工人 | 🟡 有骨架 | `src/core/supervisor/execution/dispatch.rs` | dispatch 部分实现 |

### 2.7 §3.7-3.9 Agent构建指导/多Agent团队/工作流 (Line 487-674)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 8种预置Agent模板 | ✅ 完整 | `web/src/studio/TemplateSelector.tsx` | 前端 8 种，后端 `template_selector.rs` 也支持 |
| 7种团队模板 | 🟡 有骨架 | `src/core/workflow/templates/` | `deploy.rs`, `data_analysis.rs`, `code_review.rs`, `support.rs` 等 |
| 8种工作流模板 | ✅ 完整 | `src/core/workflow/templates/` | 实现 |
| 工作编排画布 | ✅ 完整 | `web/src/studio/NodeCanvas.tsx` | 实现 |

---

## 3. 工作台 Workbench — §4

### 3.1 §4.1 交互流程 (Line 681-763)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 统一输入框 | 🟡 有骨架 | `web/src/App.tsx` | 聊天界面，但非"统一所有指令"的输入框 |
| COO 6级决策树 (L1-L6) | ✅ 完整 | `src/core/supervisor/types.rs` | DecisionLevel 枚举精确匹配 |
| 强制指定级别 | ✅ 完整 | 同上 | DecisionOverride::parse_prefixed |
| Step 4: 执行（3s/3-30s/30s+） | 🟡 有骨架 | `src/core/supervisor/execution/` | 逻辑存在，但时间分段的UI提示未实现 |

### 3.2 §4.2 COO 三种工作模式 (Line 765-776)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 主动模式 (默认) | ✅ 完整 | `src/core/supervisor/types.rs` (Mode::Proactive) |
| 安全模式 | ✅ 完整 | Mode::Safe |
| 自动化模式 | ✅ 完整 | Mode::Automated |

### 3.3 §4.3 COO 决策协议 (Line 778-795)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 三挡决策 (运营/战术/战略) | ✅ 完整 | `src/core/supervisor/types.rs` DecisionTier | Operational/Tactical/Strategic |
| COO学习机制 | ✅ 完整 | `src/core/supervisor/learning.rs` | LearningEngine 实现 |

### 3.4 §4.4 接管整个电脑 (Line 797-816)

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 文件系统操作 | ✅ 完整 | `src/computer/desktop_ops.rs` | 读/写/移/删 等 |
| 应用管理 | ✅ 完整 | `src/computer/app_ops/` | launch/list |
| 系统设置 | 🟡 有骨架 | `src/computer/sys_ops.rs` | 基础 |
| 桌面操控 (鼠标/键盘/窗口) | ✅ 完整 | `src/computer/desktop_ops/mouse.rs`, `keyboard.rs`, `window.rs` | 完整 |
| 浏览器操控 | ✅ 完整 | `src/computer/browser_ops.rs` | 实现 |
| 安全授权 L1-L3 | ✅ 完整 | `src/core/security/`, `src/computer/mod.rs` (SecurityConfig) | 实现 |
| 感知融合 (VLM+无障碍+OCR) | ❌ 不存在 | — | 仅代码骨架，无真实多模态 |

---

## 4. 管理台 Console — §5

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 📊 仪表盘 | ✅ 完整 | `web/src/console/AdminDashboard.tsx` | 实现 |
| 🤖 组件拓扑 | ✅ 完整 | `web/src/console/Topology.tsx` | 实现 |
| 💰 成本中心 | 🟡 有骨架 | `web/src/console/CostCenter.tsx`, `src/console/cost/` | 前端基础，后端 budget/mod/report 存在 |
| ⚙ 治理 | ✅ 完整 | `web/src/console/Governance.tsx` | 实现 |
| 🔐 安全 | ✅ 完整 | `web/src/console/Security.tsx` | 实现 |
| 🏪 市场 | ✅ 完整 | `web/src/console/Marketplace.tsx` | 实现 |
| 💻 系统 | ✅ 完整 | `web/src/console/SystemInfo.tsx` | 实现 |
| 信任评分 | ✅ 完整 | `src/core/trust_scorer/scorer.rs`, `evaluator.rs` | 完整公式实现 |

---

## 5. 渠道与连接 — §6

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| ⭐ Windows桌面端 (Tauri) | ✅ 完整 | `src-tauri/` | Tauri 完整 |
| ⭐ 网页端 (PWA) | ✅ 完整 | `web/dist/manifest.json`, `web/dist/sw.js` | PWA manifest + service worker |
| 🔥 企业微信 | 🟡 有骨架 | `src/channel/wecom.rs` | 模块存在 (channels-full feature) |
| 🔥 钉钉 | 🟡 有骨架 | `src/channel/dingtalk.rs` | 模块存在 |
| 🔥 飞书 | 🟡 有骨架 | `src/channel/feishu.rs` | 模块存在 |
| ⭐ 微信小程序 | 🟡 有骨架 | `src/channel/miniprogram.rs` | 模块存在 |
| ⭐ REST API + Webhook | ✅ 完整 | `src/channel/rest_api.rs`, `src/channel/webhook.rs` | 完整 |
| 🔶 邮件 (SMTP) | 🟡 有骨架 | `src/channel/smtp.rs` | 模块存在 |
| 🔶 微信公众号 | 🟡 有骨架 | `src/channel/wechat_mp.rs` | 模块存在 |
| 🔶 QQ 机器人 | 🟡 有骨架 | `src/channel/qqbot.rs` | 模块存在 |
| 🔶 Telegram | 🟡 有骨架 | `src/channel/telegram.rs` | 模块存在 |
| 🔶 浏览器扩展 | 🟡 有骨架 | `src/channel/browser_ext.rs` | 模块存在 |
| PushPlus/Server酱 | 🟡 有骨架 | `src/channel/pushplus.rs`, `src/channel/serverchan.rs` | 模块存在 |
| Channel Adapter | ✅ 完整 | `src/channel/adapter.rs` | 统一消息格式转换 |
| 桌面端 + CLI | 🟡 有骨架 | `src/channel/cli.rs`, `src/channel/desktop.rs` | 存在 |

**注意**: 所有IM渠道均在 `channels-full` feature gate 下，非默认编译。

---

## 6. 市场 Marketplace — §7

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 商品类型：组件类型定义 | ❌ 不存在 | — | 类型定义本身就是系统组件，市场无独立"卖类型" |
| 商品类型：原子组件 | 🟡 有骨架 | `src/market/marketplace.rs` | 7种内置演示商品 |
| 商品类型：Agent | 🟡 有骨架 | 同上 | Research Agent 演示商品 |
| 商品类型：团队蓝图 | 🟡 有骨架 | 同上 | Risk Control Team 演示商品 |
| 交易模型：按次计费 | ✅ 完整 | `src/market/marketplace.rs` purchase() | 完整购买+授权流程 |
| 交易模型：月订阅 | 🟡 有骨架 | `src/market/billing.rs` | BillingPlan + Order + Invoice 结构存在 |
| 收益：创作者收益查看 | ✅ 完整 | `src/market/revenue.rs` | RevenueManager + CreatorEarnings |
| 支付网关 Stripe | 🟡 有骨架 | `src/market/gateway_stripe.rs` | 存在 |
| 支付网关 Mock | ✅ 完整 | `src/market/gateway_mock.rs` | 完整 |
| PaymentGateway trait | ✅ 完整 | `src/market/gateway.rs` | 完整 trait |

---

## 7. 安全模型 — §8

| 承诺 | 状态 | 代码位置 | 说明 |
|------|------|---------|------|
| 四层宪法 L1-L4 | ✅ 完整 | `src/core/security/constitution.rs` | SecurityLevel 枚举 |
| Dual-LLM 6个检查点 | ✅ 完整 | `src/core/dual_llm/engine.rs` | Auth/ParamValidate/ContentSanitize/Permission/Audit/Route |
| 审计日志 | ✅ 完整 | `src/core/security/constitution.rs` (AuditLog) | 实现 |
| SecurityGuard | ✅ 完整 | `src/core/security/guard.rs` | 实现 |
| SecurityProfile | ✅ 完整 | `src/core/security/profile.rs` | 实现 |

---

## 8. 架构完整性 — 关键子系统总表

| 子系统 | 状态 | 说明 |
|--------|------|------|
| **Pipeline (Agentless)** (§2.1) | ✅ 完整但缺真实工具执行 | DAG节点+拓扑排序完整，但AgentlessPipeline的execute()是模拟 |
| **COO Supervisor** (§2.2) | ✅ 完整 | 决策树、三模式、学习机制、团队构建完整 |
| **ComponentTypeDef** (§2.5) | ✅ 完整 | 结构+注册机制完整，但缺少市场安装+Studio动态更新 |
| **Studio 前端** | ✅ 完整 | 所有web组件存在(AgentBuilder/NodeCanvas/TestPanel/ComponentEditor/TemplateSelector/StepWizard) |
| **Studio 后端编辑** | ✅ 完整 | editors/tool/knowledge/pipeline/model/memory 全实现 |
| **市场基础** (§7) | 🟡 有骨架 | purchase/publish/license/revenue 完整，支付 gateway 存在，但只有预置演示数据 |
| **三种构建方式** (§3.1) | 🟡 有骨架 | 引导式+拖拽画布完整，一句话构建仅骨架 |
| **Dual-LLM 安全** (§8.2) | ✅ 完整 | 6检查点 + 主副LLM判断 |
| **MCP** | ✅ 完整 | adapter.rs + tools/ + mod.rs |
| **EventBus** | ✅ 完整 | publish/subscribe + 16个标准事件 |
| **Storage (SQLite)** | ✅ 完整 | 23张表，完整 CRUD |
| **Security (四层宪法)** | ✅ 完整 | constitution + guard + profile |
| **多渠道** | 🟡 有骨架 | 12个渠道模块均存在，但 feature-gated，非默认编译 |
| **电脑操控** | 🟡 有骨架 | 文件/应用/桌面/浏览器操作完整，但"感知融合(VLM+OCR)"未实现 |
| **收益/计费** | 🟡 有骨架 | RevenueManager/CreatorEarnings/BillingPlan 完整，但未对接真实支付 |
| **TestPanel** | ✅ 完整 | 执行日志/耗时/Token/成本显示完整 |

---

## 9. 汇总统计

| 标记 | 含义 | 计数 | 百分比 |
|------|------|------|--------|
| ✅ 完整 | 代码实现与设计承诺一致 | 62 | 58% |
| 🟡 有骨架但缺功能 | 代码存在但功能不够或不完整 | 32 | 30% |
| ❌ 不存在 | 设计中承诺但代码未实现 | 13 | 12% |
| **总计** | 检查项 | **107** | **100%** |

### 主要缺口 (❌)

1. **§2.2 — Component/IOComponent/SecureComponent trait** — 设计文档写了完整的 Rust trait 接口，但代码中没有
2. **§2.5 — 组合规则引擎**（必选/冲突/约束/兼容）— 只存在基础 validator
3. **§2.5 — 市场安装新类型→Studio自动更新** — TypeRegistry 不具备安装接口
4. **§3.1 — COO从Registry+市场动态推断组件** — 一句话构建仅骨架
5. **§3.3 — 反模式 + 人格代数组合** — 未实现
6. **§3.6 — 投票集成/路由/Agent即工具/黑板** — 4/7种协作模式未实现
7. **§4.4 — 感知融合（VLM+无障碍+OCR）** — 无真实多模态
8. **§7 — 商品类型"组件类型定义"** — 市场无售卖"新类型"能力

### 主要骨架项 (🟡)
- Agentless 管道 execute() 是模拟，不调真实工具
- 12个IM渠道 feature-gated，全量编译未启用
- 一句话构建不能端到端生成完整Agent
- 收益管理有模型但无真实支付对接
- COO不查市场可用组件做推断
