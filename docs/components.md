# Morn 组件体系

> 六类原子组件 → Agent → 多 Agent 团队 → 工作流

## 六类原子组件

### 1. Tool（工具）
外部能力适配器。每个 Tool 有唯一 ID、名称、输入输出 schema。

```rust
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
}
```

内置工具：文件系统、浏览器控制、应用管理、桌面操作、系统管理、感知模块。

### 2. Knowledge（知识）
知识库检索模块。支持向量检索和关键词检索。

| 类型 | 存储 | 检索方式 |
|------|------|---------|
| 本地文件 | SQLite + FTS5 | 关键词 + 语义 |
| 远程 API | REST 端点 | API 查询 |

### 3. Skill（技能）
标准化的能力描述文件 (SKILL.md)。通过 SkillLoader 自治发现。

| 字段 | 说明 |
|------|------|
| name | 技能名称 |
| description | 描述 |
| trigger | 触发条件 |
| steps | 执行步骤 |
| tools | 依赖工具 |
| models | 推荐模型 |

### 4. Persona（人格）
Agent 的个性模板。52 个预置人格。

```rust
pub struct Persona {
    pub name: String,
    pub role: String,
    pub tone: String,
    pub expertise: Vec<String>,
    pub principles: Vec<String>,
    pub decision_framework: Vec<String>,
}
```

**预设模板（52 个）：**

| 类别 | 模板 |
|------|------|
| 通用 | 助理、助手、导师 |
| 技术 | 软件工程师、QA 工程师、运维工程师、数据工程师、安全工程师 |
| 分析 | 数据分析师、市场分析师、量化分析师、研究员 |
| 创意 | 作家、编剧、翻译、UI 设计师、视频剪辑师 |
| 管理 | 产品经理、项目经理、团队主管、HR |
| 行业 | 医生、律师、金融顾问、教育专家 |
| 更多 | 共 52 个预置人格 |

### 5. Memory（记忆）
三层记忆架构：

| 层 | 存储 | 生命周期 | API |
|----|------|---------|-----|
| Working | 当前会话 | 会话结束清除 | save/load/clear |
| Episodic | SQLite | 永久 + 压缩 | store/recall/prune |
| Semantic | SQLite | 永久 + 融合 | infer/update/merge |

自编辑记忆 (`memory_self_edit.rs`) 支持修正、压缩、合并旧记忆。

### 6. Model（模型）
LLM 模型配置。支持任意 OpenAI 兼容 API。

```rust
pub struct ModelConfig {
    pub provider: String,    // "deepseek", "openai", "custom"
    pub model_name: String,  // "deepseek-chat", "gpt-4"
    pub base_url: String,
    pub api_key: String,
    pub parameters: ModelParameters,
    pub fallback: Option<String>,
}
```

## Agent 组装

组件 → Agent 的组装过程：

1. 选择人格 (Persona)
2. 选择模型 (Model)
3. 装配工具集 (Tools)
4. 绑定知识库 (Knowledge)
5. 加载技能 (Skills)
6. 配置记忆 (Memory)

```rust
AgentDef {
    id, name,
    persona, model,
    tools: Vec<String>,
    knowledge: Vec<String>,
    skills: Vec<String>,
    memory: Option<MemoryConfig>,
}
```

## 变量系统 (VariableStore)

工作流变量系统。支持：

| 操作 | 描述 |
|------|------|
| set | 设置变量 |
| get | 读取变量 |
| resolve | 模板解析（`{{var}}` → 值） |
| scope | 变量作用域管理 |

## 工作流模板

预置 5 个模板：
1. 数据处理管道
2. 多角度分析
3. 报告生成
4. 定时监控
5. Agent 团队协作
