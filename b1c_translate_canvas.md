# Batch 1c — 英文翻译：画布 + 工作流 + Console

将以下文件中的硬编码中文替换为 i18n t() 调用。

## 编码准则（同前）

Key 命名空间：`t('node_canvas.xxx')`, `t('workflow_builder.xxx')`, `t('admin_dashboard.xxx')`, `t('business_templates.xxx')`, `t('topology.xxx')`

## 任务列表

### T1: NodeCanvas.tsx — 画布编辑器

文件：`web/src/studio/NodeCanvas.tsx`

硬编码中文：
- 节点类型标签（side panel 中的分类名称）
- "撤销" "重做" "自动排列" 按钮
- "连线设置" "删除连线" 上下文菜单项
- 提示文字和占位符
- 节点右键菜单选项
- 画布状态提示（"拖拽组件到画布"等）

### T2: MobileView.tsx — 移动端视图

文件：`web/src/studio/canvas/MobileView.tsx`

硬编码中文：
- 视图标题
- 操作按钮文字
- 提示文本

### T3: EditorPanel.tsx — 编辑器面板

文件：`web/src/studio/canvas/EditorPanel.tsx`

硬编码中文：
- 面板标题
- 字段标签
- 按钮文字
- 配置项名称

### T4: WorkflowBuilder.tsx — 工作流构建器

文件：`web/src/studio/WorkflowBuilder.tsx`

硬编码中文：
- 节点分类标题："流程节点" "控制节点" "交互节点"
- 节点类型标签："触发" "Agent" "工具" "代码" "LLM" "知识检索"
- 控制节点："条件" "循环" "等待" "并行" "路由" "聚合"
- 交互节点："人工审批" "人工输入" "通知"
- 按钮和提示文本

### T5: AdminDashboard.tsx — 控制台仪表盘

文件：`web/src/console/AdminDashboard.tsx`

硬编码中文：
- 引导步骤文字（9+ 条）
- 功能描述文本
- 卡片标题
- 快速操作按钮

### T6: BusinessTemplates.tsx — 业务模板

文件：`web/src/console/BusinessTemplates.tsx`

硬编码中文：
- 5 个业务模板名称
- 每个模板的描述
- 按钮文本

### T7: Topology.tsx — 拓扑图

文件：`web/src/console/Topology.tsx`

硬编码中文：
- "节点状态实时更新" 等拼接文案
- 图例标签
- 状态文字

### T8: en.json + zh.json 补充

为所有新 key 添加翻译。检查现有翻译是否有遗漏不完整的地方一并补上。

## 验证

- `npm run build` ✅
- `tsc --noEmit` ✅
- Canvas/Workflow/Console 页面显示正常
