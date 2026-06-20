# Batch 1a — 英文翻译：Welcome + QuickActions + Store

将以下文件中的硬编码中文替换为 i18n t() 调用，并补充 en.json + zh.json 翻译条目。

## 编码准则

1. 【Think Before Coding】先看完整文件理解结构，再动手替换
2. 【最小改动原则】只改文案部分，不改逻辑/样式/JSX结构
3. 【无副作用】不引入新依赖，不改功能行为
4. 【提取 Key 规则】`t('welcome.xxx')` / `t('quick_actions.xxx')` / `t('store.xxx')` 按目录命名空间
5. 【翻译填充】en.json 写真实英文，zh.json 保留原中文
6. 【验证】每改完一个文件立即检查语法无误

## 任务列表

### T1: WelcomeNoKey.tsx — 无 API Key 欢迎页

文件：`web/src/welcome/WelcomeNoKey.tsx`

硬编码中文：
- "欢迎使用 Morn"
- "你需要配置一个 AI 模型才能开始"
- "我有 API Key"
- "→ 打开设置页"
- "配置 API Key"
- "或者先逛逛："
- "🏪 Bot Store · 📖 Studio · ⚙️ 设置"

替换为 `t('welcome_no_key.title')` 等，补充 en.json + zh.json

### T2: WelcomeReady.tsx — 有 Key 欢迎页

文件：`web/src/welcome/WelcomeReady.tsx`

硬编码中文：
- "你好，我是 Morn"
- "桌面 AI 系统已就绪 ✅"
- "试试说："
- "📄 \"帮我写一份周报\""
- "💻 \"查一下电脑配置\""
- "🔍 \"搜索 AI Agent 最新消息\""
- "📊 \"分析这组数据\""
- "或者去 Store 安装预置 Bot"

### T3: WelcomeError.tsx — 调用失败页

文件：`web/src/welcome/WelcomeError.tsx`

硬编码中文：
- "API 调用失败"
- "当前模型的连接出了问题"
- "检查 API Key → 设置"
- "切换 Provider"

### T4: QuickActions.tsx — 快捷操作按钮

文件：`web/src/QuickActions.tsx`

硬编码中文：
- "数据分析"
- "帮我分析一下当前的市场趋势和热点"
- "写作"
- "帮我写一份关于 AI Agent 发展的深度分析报告"
- "搜索研究"
- "\"帮我搜索并总结 DeepSeek 最新技术突破\""
- "系统管理"
- "检查系统状态和资源使用情况"
- "自定义"
- 其他按钮文本

### T5: BotStore.tsx — 商店页面

文件：`web/src/store/BotStore.tsx`

硬编码中文：
- "Bot Store"
- "搜索 Bot..."
- "加载中..."
- "免费"
- "¥{price}/次"
- "免费安装"
- "¥{price} 安装"
- "已安装 ✓"
- "安装中..."
- "没有找到匹配的 Bot"
- 类别筛选标签
- 各 Bot 的硬编码名称和描述

### T6: CheckoutModal.tsx — 支付弹窗

文件：`web/src/store/CheckoutModal.tsx`

硬编码中文：
- "确认购买"
- "支付方式"
- "支付宝"
- "模拟支付"
- "取消"
- "支付成功！"
- "支付失败"

### T7: en.json + zh.json 补充

文件：`web/src/i18n/en.json` 和 `web/src/i18n/zh.json`

为上述所有新 key 添加翻译条目。en.json 写地道英文，zh.json 保留原中文。

## 验证

- `npm run build` ✅
- `tsc --noEmit` ✅
- 中文界面显示不变
- en.json 和 zh.json key 完全对齐

## 执行规则

- 不改动任何逻辑代码
- 不改 CSS/样式
- 不对功能产生任何影响
- 先改 tsx 文件，再补翻译文件
