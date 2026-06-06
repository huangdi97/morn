# Morn 创作台 (Studio)

> 组件管理 · Agent 组装 · 即时测试 · 发布市场

## StudioManager

组件生命周期管理：

| 操作 | 描述 |
|------|------|
| list_components | 按类型过滤列表 |
| get_component | 获取组件详情 |
| create_component | 创建新组件 |
| update_component | 更新组件配置 |
| delete_component | 删除组件 |

## Agent 组装 (Assembler)

通过自然语言或 GUI 组装 Agent：

### NL 组装
```bash
"帮我创建一个数据分析 Agent，人格用 analyst，模型用 deepseek-chat"
```
→ 自动匹配组件 → 创建 Agent → 返回 ID

### GUI 组装 (AgentBuilder.tsx)
- 卡片式人格选择（52 个预置模板）
- 拖拽配置工具集和知识库
- 实时预览 Agent 配置

### 组件类型一览

| 类型 | 图标 | 说明 |
|------|------|------|
| Agent | 🤖 | 组装好的 Agent |
| Tool | 🔧 | 能力工具 |
| Workflow | ⚙️ | 工作流 |
| Knowledge | 📚 | 知识库 |
| Persona | 🧑 | 人格模板 |

## 测试面板 (TestPanel.tsx)

即时测试组件和 Agent：

| 功能 | 描述 |
|------|------|
| TestRunner | 运行测试并测量性能 |
| 展开 IO 详情 | 查看输入/输出完整数据 |
| 编辑重跑 | 修改输入后重新测试 |
| 测试报告 | 生成结构化测试结果 |

```rust
pub struct TestResult {
    pub id: String,
    pub input: Data,
    pub output: Data,
    pub steps: Vec<TestStep>,
    pub duration: Duration,
    pub success: bool,
}
```

## 看板调度 (KanbanBoard)

任务可视化状态管理：

```
待办 (Todo) ─→ 进行中 (InProgress) ─→ 审查 (Review) ─→ 完成 (Done)
```

| 操作 | 描述 |
|------|------|
| 创建任务 | 添加到待办列 |
| 开始执行 | 移动到进行中 |
| 提交审查 | 移动到审查列 |
| 完成/驳回 | 移动到完成或返回待办 |

## Bot 商店 (BotStore.tsx)

浏览、搜索、安装预构建的 Agent：

| 功能 | 描述 |
|------|------|
| 浏览 | 卡片式 Agent 展示 |
| 搜索 | 关键词/分类过滤 |
| 安装 | 一键添加到工作台 |
| 详情 | 能力描述、评分、版本 |

## 搜索启动器 (SearchLauncher)

Alt+Space 快速搜索入口：

| 可搜索类型 | 描述 |
|-----------|------|
| 应用 | 系统已安装应用 |
| 文件 | 文件系统搜索 |
| 命令 | Morn CLI 命令 |
| Agent 技能 | 已安装的 Agent 能力 |
