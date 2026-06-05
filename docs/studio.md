# 创作台文档

## Studio 创作台是什么

`src/studio/` 目录下的三个模块提供组件和 Agent 的创建、管理、组装、测试、发布功能：

- `manager.rs` — 组件 CRUD + Agent 组装
- `publisher.rs` — 发布 / 下架 Agent 到工作台
- `tester.rs` — 组件测试运行器

## 组件管理

`StudioManager` 提供完整的 CRUD：

| 方法 | 功能 |
|------|------|
| `list_components(type_filter)` | 列出组件，支持按类型过滤 |
| `get_component(id)` | 获取组件详情 |
| `create_component(def)` | 创建新组件，返回 ID |
| `update_component(id, def)` | 更新组件名称、配置、状态 |
| `delete_component(id)` | 删除组件 |

数据结构：

- `ComponentSummary` — ID、名称、类型、状态、信任分
- `ComponentDetail` — 完整信息（含 config_json）
- `CreateComponentDef` — 创建参数（name、component_type、config_json）
- `UpdateComponentDef` — 更新参数（name、config_json、status）

## Agent 组装流程

1. 调用 `StudioManager::assemble_agent(def)` 传入 `AgentDef`
2. 内部委托 `AgentAssembler::assemble()` 生成实现了 Component Trait 的 Agent
3. 组装后的 Agent 存入存储，状态设为 `active`

`AgentDef` 包含：persona、model、tools、knowledge、skills、memory 的组合。

## 测试与发布

### 测试

`StudioTester` 提供组件模拟测试：

```rust
let result = tester.run_test(component_type, input, config);
```

测试步骤根据类型不同：
- persona / agent → person 注入 + LLM 调用
- knowledge / agent → 知识检索 + LLM 调用
- tool → LLM 调用 + 工具执行

返回 `TestResult` 包含分步耗时、token 数、成本、输出。

### 发布

`StudioPublisher` 控制 Agent 的生命周期状态：

- `publish_agent(agent_id)` — 设为 `active`
- `unpublish_agent(agent_id)` — 设为 `inactive`