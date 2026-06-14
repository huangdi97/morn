import { useState, useCallback } from 'react';

type Locale = 'en' | 'zh';

const messages: Record<Locale, Record<string, string>> = {
  en: {
    'app.title': 'Morn',
    'nav.workbench': 'Workbench',
    'nav.studio': 'Studio',
    'nav.store': 'Store',
    'nav.console': 'Console',
    'workbench.placeholder': 'Type a message...',
    'workbench.send': 'Send',
    'workbench.clear': 'Clear',
    'workbench.settings': 'Settings',
    'workbench.welcome': 'Welcome to Morn',
    'workbench.welcome.desc': 'Select a quick action or type your question',
    'store.title': 'Bot Store',
    'store.install': 'Install',
    'store.purchase': 'Purchase',
    'store.installed': 'Installed',
    'store.search': 'Search bots...',
    'studio.editor': 'Component Editor',
    'studio.builder': 'Agent Builder',
    'studio.teams': 'Teams',
    'studio.test': 'Test Runner',
    'welcome.title': 'Welcome to Morn',
    'welcome.subtitle': 'Your Desktop AI Creation System',
    'welcome.get_started': 'Get Started',
    'welcome.step1': 'Choose a Persona',
    'welcome.step2': 'Install from Store',
    'welcome.step3': 'Start Chatting',
    'dev.scaffold': 'Scaffold Plugin',
    'dev.generate': 'Generate with AI',
    'dev.examples': 'Example Plugins',
    'system.storage': 'Storage',
    'system.api': 'API Connection',
    'system.memory': 'Memory',
    'system.plugins': 'Plugins',
    'system.agents': 'Agents',
    'system.workflows': 'Workflows',
    'roi.title': 'ROI Calculator',
    'roi.saved': 'You save',
    'journey.day': 'Day',
    'journey.progress': 'Progress',
    'settings.backup': 'Backup',
    'settings.export': 'Export .mornpack',
    'settings.import': 'Import .mornpack',
  },
  zh: {
    'app.title': 'Morn',
    'nav.workbench': '工作台',
    'nav.studio': '创作台',
    'nav.store': '商店',
    'nav.console': '控制台',
    'workbench.placeholder': '输入消息...',
    'workbench.send': '发送',
    'workbench.clear': '清空',
    'workbench.settings': '设置',
    'workbench.welcome': '欢迎使用 Morn',
    'workbench.welcome.desc': '选择快捷任务或直接输入你的问题',
    'store.title': 'Bot 商店',
    'store.install': '安装',
    'store.purchase': '购买',
    'store.installed': '已安装',
    'store.search': '搜索 Bot...',
    'studio.editor': '组件编辑器',
    'studio.builder': 'Agent 构建器',
    'studio.teams': '团队',
    'studio.test': '测试运行器',
    'welcome.title': '欢迎使用 Morn',
    'welcome.subtitle': '你的桌面 AI 创作系统',
    'welcome.get_started': '开始使用',
    'welcome.step1': '选择人格',
    'welcome.step2': '从商店安装',
    'welcome.step3': '开始聊天',
    'dev.scaffold': '脚手架插件',
    'dev.generate': 'AI 生成',
    'dev.examples': '示例插件',
    'system.storage': '存储',
    'system.api': 'API 连接',
    'system.memory': '内存',
    'system.plugins': '插件',
    'system.agents': 'Agent',
    'system.workflows': '工作流',
    'roi.title': 'ROI 计算器',
    'roi.saved': '每月节省',
    'journey.day': '第',
    'journey.progress': '进度',
    'settings.backup': '备份',
    'settings.export': '导出 .mornpack',
    'settings.import': '导入 .mornpack',
  },
};

const STORAGE_KEY = 'morn-locale';

export function getLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'en' || stored === 'zh') return stored;
  return 'zh'; // Default to Chinese
}

export function setLocale(locale: Locale) {
  localStorage.setItem(STORAGE_KEY, locale);
}

export function t(key: string, locale?: Locale): string {
  const l = locale || getLocale();
  return messages[l]?.[key] || key;
}

export function useLocale(): [Locale, (l: Locale) => void] {
  const [locale, setLocaleState] = useState<Locale>(getLocale);
  const setter = useCallback((l: Locale) => {
    setLocale(l);
    setLocaleState(l);
  }, []);
  return [locale, setter];
}
