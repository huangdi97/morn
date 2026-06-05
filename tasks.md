# Morn 用户体验层全面实现

## 14 条核心准则编程原则

**1. Think Before Coding** — 理解全貌再动手
**2. Simplicity First** — 简单优先，不加不必要的抽象层
**3. Surgical Changes** — 一次一事
**4. Goal-Driven Execution** — 先明确"要什么"再"怎么写"
**5. 架构优先，拒绝补丁** — 不堆补丁，架构不合理就重构
**6. 面向组件的构建** — 模块化，每个组件职责清晰
**7. 显式优于隐式** — 明确的数据流和依赖
**8. 代码整洁与自文档化** — 代码即文档
**9. 单一职责** — 一个函数/类只做一件事
**10. 组合优于委托** — 组合模式 > 继承
**11. 单一状态源** — 状态只在一个地方管理
**12. 避免语法糖** — 可读性 > 炫技
**13. 命名一致性** — 同一概念用同一命名
**14. 文件不超过 300 行** — 超了拆

## 低耦合原则
1. **模块间只通过公开接口通信** — 不要跨模块直接访问内部结构体字段
2. **测试独立于实现细节** — 测试公共 API 而非私有函数
3. **新测试模块互相独立** — 每个 `#[cfg(test)] mod tests` 只测试本模块功能

## 执行方式
- 所有代码修改必须通过 **opencode run** 执行
- 每阶段完成后运行 `cargo fmt && cargo build && cargo test && cargo fmt --check`
- 全部完成后运行全量验证：`cd web && npm run build`

---

## 阶段一：一句话构建 Agent（NL → COO 自动组装）

### 背景
目前 AgentBuilder 是表单填写（选 tools/knowledge/skills/persona/model），用户需要写代码般的操作。改成：输入一句自然语言描述，COO 自动分析并生成完整的 Agent 定义。

### 技术方案
1. COO/Supervisor 新增 `create_agent_from_nl(nl: &str) -> Result<AgentDef>` 方法
2. 该方法调用 ChatAgent（现有的 chat_agent.rs）用 LLM 分析自然语言描述，返回 AgentDef JSON
3. 前端 AgentBuilder 新增「自然语言输入」模式 vs 「表单编辑」模式

### 任务 1.1: 后端 NL→AgentDef 流水线

**文件:** `src/core/supervisor.rs`

在 `Supervisor` 结构体中添加：

```rust
pub fn create_agent_from_nl(&self, nl: &str) -> Result<AgentDef, String> {
    // 1. 构造 LLM prompt，要求分析自然语言描述
    //    输入示例："帮我创建一个股票分析 Agent，能够获取 K 线、计算技术指标、生成报告"
    // 2. 调用 chat_fn 获取 LLM 返回的 AgentDef JSON
    // 3. 解析并验证返回的 AgentDef
    // 4. 检查 Registry 中是否存在匹配的组件
    // 5. 返回 AgentDef
}
```

AgentDef 结构（agentbuilder.rs 中已有类似定义）：
```rust
pub struct AgentDef {
    pub name: String,
    pub persona: String,
    pub model: String,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
}
```

LLM prompt 要足够结构化，示例格式：
```
用户需求：{nl}
请从以下可用组件中匹配：
工具：{tools_list}
知识：{knowledge_list}
技能：{skills_list}
人格：{personas}
返回 JSON：
{ name: string, persona: string, model: string, tools: string[], knowledge: string[], skills: string[] }
```

### 任务 1.2: Tauri 命令暴露 NL 接口

**文件:** `src/main.rs`

新增 Tauri 命令：
```rust
#[tauri::command]
fn create_agent_from_description(nl: String, state: tauri::State<AppState>) -> Result<String, String> {
    // 调用 supervisor.create_agent_from_nl(&nl)
    // 返回 AgentDef JSON 字符串
}
```

### 任务 1.3: 前端 AgentBuilder 增加 NL 输入模式

**文件:** `web/src/studio/AgentBuilder.tsx`

改动：
1. 页面顶部增加模式切换：「自然语言描述」/「手动编辑」
2. 自然语言模式：一个大输入框 + 「生成 Agent」按钮
3. 调用 `invoke("create_agent_from_description", { nl })`
4. 返回结果后自动填充到表单，用户可进一步编辑
5. 表单模式保留现有 UI

同时添加一个文案提示框，示例输入：
```
"创建一个股票分析助手，能获取行情数据、计算 MACD/RSI 指标、分析市场情绪并生成报告"
"帮我写一个生物文献翻译 Agent，能查 PubMed，翻译论文并总结要点"
```

---

## 阶段二：可视化节点编辑器（ComfyUI 风格）

### 背景
目前 AgentBuilder 是固定表单（选 Persona/Tools/Knowledge/Skills/Model），用户无法自由组合组件的关系。改成拖拽式节点编辑器——像搭积木一样连接组件。

### 技术方案
1. 安装 `reactflow` 作为节点编辑器引擎
2. 创建 `NodeCanvas.tsx` 组件作为编辑器主体
3. 定义节点类型：PersonaNode、ToolNode、KnowledgeNode、SkillNode、ModelNode、AgentNode
4. 节点输入/输出端口：每个组件类型有标准 IO 端口
5. 连线逻辑：从输出端口拖到输入端口建立连接
6. 将节点图序列化为 AgentDef → 可发布

### 任务 2.1: 安装 reactflow 依赖

```bash
cd web && npm install reactflow @types/reactflow
```

### 任务 2.2: 创建 NodeCanvas 组件

**文件:** `web/src/studio/NodeCanvas.tsx`

功能：
- 使用 ReactFlow 作为底层
- 左侧面板是组件库面板（Tool/Knowledge/Skill/Persona/Model）
- 拖拽组件到画布创建节点
- 节点显示：标题 + IO 端口（圆形输入/输出点）
- 连线：从输出端口拖到输入端口
- 右键菜单：删除节点/编辑/配置
- 右上角按钮：导出为 AgentDef / 发布 / 保存

节点颜色方案（暗色主题与现有 UI 一致）：
- PersonaNode: 紫色 (#7c3aed)
- ToolNode: 蓝色 (#3b82f6)
- KnowledgeNode: 绿色 (#22c55e)
- SkillNode: 橙色 (#f59e0b)
- ModelNode: 红色 (#ef4444)
- AgentNode: 青色 (#06b6d4)

### 任务 2.3: 定义节点类型和 IO 端口

每种组件类型定义标准的 IO 端口：

```
PersonaNode:
  输入: [system_prompt]
  输出: [persona_config]

ToolNode:
  输入: [params]
  输出: [result]

KnowledgeNode:
  输入: [query]
  输出: [data]

SkillNode:
  输入: [input]
  输出: [output]

ModelNode:
  输入: [prompt]
  输出: [response]

AgentNode:
  输入: [user_input, persona_config, tool_result, knowledge_data, skill_result, model_response]
  输出: [agent_output]
```

### 任务 2.4: 图形→AgentDef 序列化

编写 `serializeGraph(graph: Node[]) -> AgentDef` 函数：
1. 遍历所有节点
2. 提取每个节点的配置数据
3. 按类型归类（Persona → agent.persona, Tools → agent.tools 等）
4. 返回标准的 AgentDef 结构
5. 兼容现有的 AgentBuilder 发布流程

### 任务 2.5: 集成到 Studio 页面

**文件:** `web/src/studio/AgentBuilder.tsx`

在 AgentBuilder 中新增「可视化编辑」tab，与「表单编辑」tab 并列：
- Tab 1: 表单编辑（现有模式）
- Tab 2: 可视化编辑（NodeCanvas）

现有发布按钮同时在两种模式下可用。

---

## 阶段三：Marvis 式开箱即用体验

### 背景
要让非程序员拿到就能用，需要：预置模板、引导流程、快捷操作、系统常驻。

### 任务 3.1: 预置 Agent 模板

**文件:** `src/core/registry.rs`

在 `register_defaults()` 中注册 6 个开箱即用的 Agent 模板（作为预置能力）：

1. **research-assistant** — 研究助手：web_search + knowledge + summarization
2. **data-analyst** — 数据分析师：get_kline + calc_macd + chart + report
3. **writing-assistant** — 写作助手：translate + grammar + format + style
4. **coding-helper** — 编码助手：code_review + debug + format + test
5. **translation-agent** — 翻译 Agent：translate + proofread + glossary
6. **general-assistant** — 通用助手（默认）：混合工具集

每个模板包含预配置的 tools、skills、persona、model。

### 任务 3.2: 前端模板选择器

**文件:** `web/src/studio/TemplateSelector.tsx`

改动：从硬编码卡片改为从后端加载模板列表，展示每个模板的：
- 名称 + Emoji 图标
- 简短描述（2-3 行）
- 包含的组件列表（标签形式）
- "使用此模板"按钮

点击后自动填充到 AgentBuilder（两种模式均支持），用户可直接发布。

### 任务 3.3: 工作台快捷命令

**文件:** `web/src/App.tsx` + 新增 `web/src/QuickActions.tsx`

在聊天输入框上方增加快捷按钮行：
- 📊 数据分析
- 📝 写作
- 🔍 搜索研究
- 💻 编码
- 🌐 翻译

点击快捷按钮自动发送预配置 prompt 到聊天 Agent。
这些同时展示在 Dashboard 首页作为欢迎快捷卡片。

### 任务 3.4: 启动指引（Onboarding）

**文件:** `web/src/console/Dashboard.tsx`

新增首次运行检测：
1. 检查 localStorage 的 `morn_onboarded` 标记
2. 如果未标记，展示欢迎对话框：
   - Step 1: "欢迎使用 Morn" — 一句话介绍
   - Step 2: "选择你的第一个 Agent" — 从模板列表选
   - Step 3: "试试吧" — 打开工作台，预填第一条消息
3. 完成后设置标记

---

## 全量验证

```bash
cd ~/morn-desktop
cargo build 2>&1 | grep -E "^(warning|error)" | wc -l   # = 0
cargo test 2>&1 | grep "test result"                     # all passed
cargo fmt --check                                         # no diff
cd web && npm run build                                   # web 构建成功
```
