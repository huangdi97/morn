# Batch 1 — Agent 人格深度设计 + 端口连接系统

> 底层基础设施已有：TypeRegistry ✅ / Component trait ✅ / IOComponent ✅ / Assembly模块 ✅
> 此 Batch 补两样底层缺口：人格5层Prompt系统和端口可视化连接

---

## 任务清单

### B1-1: Agent 人格 5 层 Prompt 系统

**位置：** `src/core/component/persona.rs`（新建或扩展现有）

```
DESIGN.md §4.3 人格的深度设计：

Persona 需要包含：
1. 核心思维模型（5-7 条原则）
2. 决策框架（收到请求时的思考步骤）
3. 反模式（不该做什么）
4. 可量化参数（temperature, style, verbosity, proactivity）
5. 5 层 Prompt（L1核心身份 / L2技能指令 / L3格式模板 / L4约束规则 / L5对话风格）
6. 沟通风格
```

**具体工作：**

1. 扩展 `PersonaConfig`（或新建 `DeepPersona`）struct：
   ```rust
   pub struct DeepPersona {
       pub name: String,
       pub core_principles: Vec<String>,          // 核心思维模型
       pub decision_framework: Vec<String>,        // 决策框架步骤
       pub anti_patterns: Vec<String>,             // 反模式
       pub temperature: f64,
       pub style: PersonaStyle,                    // professional/creative/friendly/academic
       pub verbosity: f64,                         // 0.0=concise, 1.0=detailed
       pub proactivity: f64,                       // 0.0=passive, 1.0=proactive
       pub prompt_layers: [String; 5],             // 5 层 Prompt
       pub communication_style: String,
   }
   
   pub enum PersonaStyle {
       Professional, Creative, Friendly, Academic, Custom(String),
   }
   ```

2. `merge_prompt()` 方法 — 将 5 层 Prompt + 核心原则 + 决策框架 + 反模式 + 参数合并为最终 system prompt

3. 从已有的 `persona` 字段（现有代码里 agent 配置文件中的 `persona`）无缝兼容——旧 person 字段自动转为简单 DeepPersona（只有 L1）

4. 从 `persona_presets.json` 加载 52 种预置人格（analyst/researcher/writer/coder/translator/assistant/reviewer 等）

5. 每条人格带完整的 5 层 Prompt + 核心原则 + 决策框架

**验证：**
- 旧 person 代码不受影响（向后兼容）
- `merge_prompt()` 输出合理的完整 system prompt
- 52 种预置人格全部加载无错误
- `cargo test --lib` 全通过

---

### B1-2: 端口匹配 + 组件连接系统

**位置：** `src/core/assembly/`（扩展现有模块）

现有 `core/assembly/` 有：
- `builder.rs` — `AssemblyBuilder`, `ComponentSelector`, `DefaultCompleter`, `GuidedBuildSteps`
- `graph.rs` — `AtomicComponentDef`, `ComponentConnection`, `ComponentGraph`, `ConnectionValidator`
- `rules.rs` — 已有一些匹配规则
- `validator.rs` — `AssemblyValidator`

**具体工作：**

1. 检查 `graph.rs` 中的 `ConnectionValidator` 是否已有端口类型匹配功能。如果没有，添加：
   - `fn validate_port_types(conn: &Connection) -> Result<(), MornError>` — 检查 source port data_type == target port data_type
   - `fn suggest_compatible_ports(source: &str) -> Vec<Port>` — 给定一个端口，推荐兼容的端口

2. 检查 `ComponentGraph` 是否已有存储组件连接的能力：
   - 需支持 `add_component()`, `connect()`, `disconnect()`, `get_connections()`, `to_agent_def()`（将图转为 AgentDef）
   - 如果已有，确认功能完整；如果不完整，补齐

3. `ComponentGraph::to_agent_def()` — 将画布上的组件连接图转为可执行的 AgentDef（人格+工具+知识+技能+记忆+模型）

4. `to_agent_def()` 的组装规则：
   - 图中至少 1 个人格 + 1 个工具 + 1 个 LLM
   - 缺少的组件自动补默认值
   - 端口不匹配提示 Transformer 建议

**验证：**
- `to_agent_def()` 能正确处理多组件画布
- `validate_port_types` 能正确报告类型不匹配
- 现有测试不受影响
- `cargo test --lib` 全通过

---

### B1-3: Studio UI — 人格编辑器

**位置：** `web/src/studio/PersonaEditor.tsx`

**具体工作：**

1. 创建 `PersonaEditor` 组件：
   - 5 层 Prompt 输入框（L1-L5），每层有占位提示
   - Personality 参数滑块：温度 / 详细度 / 主动度
   - 风格选择器（Professional / Creative / Friendly / Academic）
   - 核心思维模型列表（可增删条目）
   - 决策框架列表（可增删）
   - 反模式列表（可增删）
   - 实时预览合并后的 system prompt

2. 人格模板预设：
   - 从 `list_preset_personas` Tauri 命令获取列表
   - 点击预设 → 自动填充所有字段
   - "Custom" 选项 → 空白编辑

3. 集成到 Studio 面板：
   - 在 Agent Builder 中，选择人格时打开 `PersonaEditor`

**注意：** 后端 Tauri 命令 `list_preset_personas` 和 `get_preset_persona` 已有（DESIGN.md 附录 A），调用它们即可。

**验证：**
- `npm run build` ✅
- `tsc --noEmit` ✅
- 人格预览能正确显示合并 prompt

---

### B1-4: 52 种预置人格 JSON 文件

**位置：** `src/core/component/`（参考 `market/builtin_listings.json` 和 `workflow_templates.json` 的数据化模式）

**具体工作：**

1. 创建 `src/core/component/persona_presets.json` — `include_str!` 加载（不硬编码）

2. 预置人格清单（每种带完整 5 层 Prompt）：
   - 分析师（analyst）— data-driven, 先大盘后个股
   - 研究员（researcher）— deep dive, citation-first
   - 写手（writer）— 结构清晰, 读者视角
   - 程序员（coder）— 先设计后编码, 测试先行
   - 翻译官（translator）— 忠实原文, 自然语言
   - 系统管家（assistant）— 友好高效, 安全优先
   - 审查员（reviewer）— 挑剔但建设性
   - 教师（teacher）— Socratic, 引导式
   - 以及扩展到 52 种

3. 每种人格包含：
   - `name`, `display_name`, `category`
   - `core_principles` (3-7条)
   - `decision_framework` (3-5步)
   - `anti_patterns` (2-4条)  
   - `temperature`, `style`, `verbosity`, `proactivity`
   - `prompt_layers` [5 strings]
   - `communication_style`

4. 加载路径：
   - 在 `DeepPersona::load_presets()` 中 `include_str!` 解析
   - 返回 `HashMap<String, DeepPersona>` — 按 name 索引
   - Tauri 命令 `list_preset_personas` 从此加载
   - Tauri 命令 `get_preset_persona(name)` 返回完整定义

**验证：**
- 52 种人格全部加载无解析错误
- JSON schema 验证通过
- 每种人格的合并 prompt 不为空
- `cargo test --lib` 全通过

---

## 执行顺序

1. B1-4 → B1-1 → B1-2 → B1-3
   （先做人格JSON数据，再做后端代码，再做端口连接，最后做UI）

2. 每步完成立即 `cargo check -p morn`

## 验证门禁

- `cargo build --lib` ✅
- `cargo test --lib` 全部通过
- `cargo clippy -p morn` 0 warnings
- `npm run build` ✅
- 新代码严格遵守 Rust Component trait 接口（不改现有 Supervisor/Scheduler 一行代码）
