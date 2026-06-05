# 组件体系文档

## Component Trait 定义

`src/core/component.rs` 定义了组件系统的三个核心 Trait：

```rust
pub trait Component: Send {
    fn id(&self) -> &str;
    fn type_name(&self) -> &str;
    fn init(&mut self) -> Result<(), String>;
    fn run(&mut self) -> Result<(), String>;
    fn pause(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn health_check(&self) -> HealthStatus;
}

pub trait IOComponent: Component {
    fn ports(&self) -> Vec<Port>;
    fn send(&mut self, port: &str, data: Data) -> Result<(), String>;
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String>;
}

pub trait SecureComponent: Component {
    fn required_permissions(&self) -> Vec<Permission>;
}
```

## 六类原子组件

`src/component/` 目录下 6 个文件对应 6 种组件：

| 组件 | 文件 | 说明 |
|------|------|------|
| Tool | `tool.rs` | 外部工具（搜索、计算、API 调用） |
| Knowledge | `knowledge.rs` | 知识库检索 |
| Skill | `skill.rs` | 复合技能 |
| Persona | `persona.rs` | 角色人格定义 |
| Memory | `memory.rs` | 记忆存储 |
| Model | `model.rs` | 模型配置（provider / model_name / parameters） |

另有 `ComponentType` 枚举额外包含 `Agent` 和 `Pipeline` 两种复合类型。

## Port 和 Data 模型

```rust
pub struct Port {
    pub id: String,
    pub direction: PortDirection,  // Input / Output / Bidirectional
    pub data_type: String,
    pub description: String,
}

pub struct Data {
    pub content: Value,     // serde_json::Value
    pub mime_type: String,  // text/plain, application/json, ...
}
```

`Data` 提供便捷构造方法：
- `Data::text("...")` — 纯文本数据
- `Data::json(value)` — JSON 数据
- `Data::new(content, mime_type)` — 自定义数据

## AgentAssembler（组装器）

`src/core/assembler.rs` 的 `AgentAssembler` 负责将组件组装为 Agent：

```rust
pub struct AgentDef {
    pub id: String,
    pub name: String,
    pub persona: Persona,
    pub model: ModelConfig,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
    pub memory: Option<String>,
}
```

- `assemble(def)` — 根据 AgentDef 构造一个实现了 Component + IOComponent + SecureComponent 的 Agent
- `natural_language_build(description)` — 自然语言描述自动生成 AgentDef（通过关键词匹配选择 Persona 和 Tools）

## 组件生命周期

```
init → run → pause → stop
        ↑______________|
```

- **init** — 初始化资源，返回 Ok(()) 或 Err
- **run** — 开始运行
- **pause** — 暂停运行
- **stop** — 停止并释放资源
- **health_check** — 返回 Healthy / Degraded(msg) / Unhealthy(msg)