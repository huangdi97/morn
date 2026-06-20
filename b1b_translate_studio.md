# Batch 1b — 英文翻译：Studio 向导 + 模板 + Agent 构建

将以下 Studio 核心文件中的硬编码中文替换为 i18n t() 调用，并补充翻译条目。

## 编码准则（同 B1a）

1. 只改文案，不改逻辑/样式/JSX结构
2. Key 按目录命名空间：`t('step_wizard.xxx')`, `t('template_selector.xxx')`, `t('agent_builder.xxx')`
3. 每改完一个文件立即检查语法
4. 最后补充 en.json + zh.json

## 任务列表

### T1: StepWizard.tsx — Studio 构建向导（~40条硬编码）

文件：`web/src/studio/StepWizard.tsx`

硬编码中文区域：
- 步骤标题："选择组件类型" "配置基础信息" "添加工具" "配置知识" ...
- 组件类型标签："工具" "知识" "技能" "人格" "记忆" "模型" "渠道"
- 记忆类型选项："工作记忆" "情景记忆" "语义记忆" "长程经验" "图谱记忆" "闪存"
- 工具选项："搜索" "读取文件" "写入文件" "代码执行" "HTTP请求" "消息发送"
- 知识选项："向量数据库" "全文搜索" "结构化数据" "文件知识" "SQLite知识"
- 人格选项："分析师" "研究员" "写手" "程序员" "翻译官" "系统管家" "审查员"
- 模型选项："DeepSeek Chat" "DeepSeek Reasoner" "GPT-4o" "Claude 3"
- 渠道选项："Telegram" "企业微信" "钉钉" "飞书" "桌面"
- 描述文字和提示文本
- 按钮："上一步" "下一步" "完成" "取消"

### T2: TemplateSelector.tsx — Agent 模板选择

文件：`web/src/studio/TemplateSelector.tsx`

硬编码中文：
- 6 个模板名称："数据分析师" "研究员" "写作者" "程序员" "翻译官" "系统管家"
- 每个模板的描述文本
- "从模板创建" "自定义构建" 按钮
- "选择一个模板开始" 标题
- 加载、空状态提示

### T3: TeamTemplateSelector.tsx — 团队模板选择

文件：`web/src/studio/TeamTemplateSelector.tsx`

硬编码中文：
- 7 个团队模板名称："股票研究团队" "软件开发团队" "内容生产团队" "市场调研团队" "风控团队" "客服团队" "监控团队"
- 每个模板的描述
- 协作模式标签："链式" "并行" "会诊" "招标" "接力" "师徒" "黑板"
- 标题和按钮

### T4: TeamBuilder.tsx — 团队构建器

文件：`web/src/studio/TeamBuilder.tsx`

硬编码中文：
- 协作模式选择
- 成员配置
- 按钮文本
- 提示信息

### T5: AgentBuilder.tsx — Agent 构建器

文件：`web/src/studio/AgentBuilder.tsx`

硬编码中文：
- 模式选择按钮："自然语言描述" "手动编辑"
- 示例 NL prompt（4 条长中文文本需要抽 key）
- "创建 Agent" 按钮
- "保存" "测试" "发布" 按钮
- 表单标签："名称" "人格" "模型" "工具" "知识" "技能"
- 提示文本
- 加载/成功/错误状态消息
- 编辑标签："表单编辑" "可视化编辑"

### T6: en.json + zh.json 补充

为上述所有新 key 添加翻译。

## 验证

- `npm run build` ✅
- `tsc --noEmit` ✅
- Studio 页面中文显示不变
- en.json 和 zh.json key 完全对齐
