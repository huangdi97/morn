# 数据分析面板插件 (ui-panel)

在 Console 中新增一个数据分析仪表盘 Tab 页，显示 API 调用量、延迟、成功率等指标。

## 安装

1. 将本目录复制到 `~/.hermes/plugins/plugin-panel-analytics/`
2. 在 Morn 中启用：`PluginManager::activate("plugin-panel-analytics")`

## 结构

```
plugin-panel-analytics/
├── plugin.json   — 插件清单
├── panel.html    — 面板入口（HTML + 内嵌 JS）
└── README.md     — 本文档
```

## 权限

- `ui:slot-console` — 在 Console 中注册 Tab 页
