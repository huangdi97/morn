# Morn 市场生态

> 组件市场 · 模板商店 · 模板市场 · 搜索启动器

## 组件市场 (Marketplace)

组件和 Agent 的发布、搜索、安装平台。

### 数据结构

```rust
pub struct ComponentListing {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub rating: f64,
    pub downloads: u64,
    pub license: String,
}
```

### 功能

| 功能 | 描述 |
|------|------|
| 发布组件 | publish_agent / publish_component |
| 浏览列表 | 按类型/评分/下载量排序 |
| 下载安装 | 一键安装到本地 Registry |
| 版本管理 | 版本号 + 更新检查 |
| 评分系统 | 用户评分 + 自动信任评分 |

### Gateway trait

```rust
pub trait Gateway {
    fn process_payment(&self, amount: f64, currency: &str) -> Result<PaymentResult>;
    fn refund(&self, payment_id: &str) -> Result<RefundResult>;
}
```

| 实现 | 用途 |
|------|------|
| MockGateway | 测试用模拟支付 |
| StripeGateway | 生产用 Stripe 支付 |

### License / Billing

| 功能 | 描述 |
|------|------|
| License 验证 | 检查组件许可证有效性 |
| Billing 计费 | 按使用量/订阅模式计费 |
| 免费/付费标记 | 组件可设置免费或付费 |

## 工作流模板商店 (WorkflowTemplateStore)

预置工作流模板：

| 模板 | 用途 |
|------|------|
| 数据处理管道 | 采集→清洗→分析→输出 |
| 多角度分析 | 同一数据多个 Agent 独立分析→汇总 |
| 报告自动生成 | 数据采集→图表→排版→导出 |
| 定时监控告警 | 轮询→阈值判断→通知 |
| Agent 团队协作 | 主管拆任务→工人执行→结果合并 |

```rust
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub steps: Vec<WorkflowStep>,
    pub variables: Vec<TemplateVariable>,
}
```

## 社区模板市场 (CommunityTemplateRegistry)

从远程仓库拉取模板：

| 功能 | 描述 |
|------|------|
| fetch_registry | 从远程拉取模板列表 |
| install_templates | 批量安装模板 |
| check_updates | 检查本地模板更新 |
| 远程源 | 可配置的 URL 端点 |

## Cortex 推理引擎

MCP 协议兼容的模型和技能仓库：

```rust
pub fn mcp_market() -> Vec<MarketItem> {
    // 从社区仓库更新可用模型/技能列表
}
```

| 类型 | 来源 |
|------|------|
| LLM 模型 | MCP 社区仓库 |
| 推理技能 | 社区贡献的推理链 |
| 工具适配器 | 社区贡献的工具包装 |

## 搜索启动器 (SearchLauncher)

系统级快速搜索入口（Alt+Space）：

```rust
pub struct SearchIndex {
    pub apps: Vec<AppEntry>,
    pub files: Vec<FileEntry>,
    pub commands: Vec<CommandEntry>,
    pub agent_skills: Vec<SkillEntry>,
}
```

搜索结果排序：模糊匹配 → 最近使用 → 评分。
