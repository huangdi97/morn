# Morn 插件开发指南

## 概述

Morn 插件系统允许开发者通过标准化的插件结构扩展 Morn 的功能。插件可以是 CSS 主题、通信渠道、UI 组件、协议适配或后端工具。

## 插件结构

```
plugins/
└── your-plugin-name/
    ├── manifest.json    # 必需：插件元数据
    ├── main.js          # 入口文件（JavaScript 插件）
    └── ...              # 其他资源文件
```

## Manifest 格式

```json
{
  "name": "my-awesome-plugin",
  "version": "1.0.0",
  "description": "What your plugin does",
  "author": "Your Name",
  "plugin_type": "theme",
  "entry": "main.js"
}
```

### 字段说明

| 字段 | 必需 | 说明 |
|------|------|------|
| `name` | ✅ | 全局唯一标识，小写+连字符 |
| `version` | ✅ | 语义化版本号 |
| `description` | ✅ | 一行的功能描述 |
| `author` | ❌ | 作者名 |
| `plugin_type` | ✅ | 插件类型（见下表） |
| `entry` | ✅ | 入口文件名（.js 或 .py） |

### 插件类型

| 类型 | 说明 | 入口建议 |
|------|------|---------|
| `theme` | CSS 主题，修改界面外观 | `theme.css` |
| `channel` | 通信渠道（IM/邮件等） | `main.js` |
| `ui_slot` | 前端 UI 扩展 | `main.js` |
| `protocol` | 协议适配 | `main.py` |
| `tool` | 后端工具 | `main.py` |

## 开发步骤

### 1. 手动创建

```
mkdir -p plugins/my-plugin
cd plugins/my-plugin
```

创建 `manifest.json` 和入口文件。

### 2. AI 生成（推荐）

在 Morn Studio → Dev 标签页中：
1. 切换到 "AI 生成" 模式
2. 输入自然语言描述（如 "一个天气工具插件，显示天气预报"）
3. 点击 "Generate with AI"
4. 系统自动生成 manifest + 入口文件

### 3. 文件放置

将插件目录放到 `~/.local/share/morn/plugins/` 下：
```
~/.local/share/morn/plugins/
├── my-plugin/
│   ├── manifest.json
│   └── main.js
└── another-plugin/
    ├── manifest.json
    └── theme.css
```

### 4. 验证加载

启动 Morn，在 Console → System 中查看插件状态。或在 Dev Zone 中查看已发现的插件列表。

## 示例

### 示例 1：最小主题插件

`plugins/minimal-dark/manifest.json`:
```json
{
  "name": "minimal-dark",
  "version": "1.0.0",
  "plugin_type": "theme",
  "entry": "theme.css"
}
```

`plugins/minimal-dark/theme.css`:
```css
:root[data-theme="minimal-dark"] {
  --bg-primary: #0d1117;
  --bg-secondary: #161b22;
  --text-primary: #c9d1d9;
  --accent: #58a6ff;
}
```

### 示例 2：Python 工具插件

`plugins/hello-tool/manifest.json`:
```json
{
  "name": "hello-tool",
  "version": "1.0.0",
  "plugin_type": "tool",
  "entry": "main.py"
}
```

`plugins/hello-tool/main.py`:
```python
#!/usr/bin/env python3
"""Morn tool plugin — prints system info."""
import json, os, platform

info = {
    "plugin": os.environ.get("MORN_PLUGIN_NAME", "hello-tool"),
    "platform": platform.system(),
    "python": platform.python_version(),
}
print(json.dumps(info))
```

Python 插件通过 stdout 输出 JSON 数据返回给 Morn。支持的环境变量：
- `MORN_PLUGIN_DIR` — 插件目录路径
- `MORN_PLUGIN_NAME` — 插件名称
- `MORN_API_URL` — Morn API 地址

## 常见问题

**Q: 插件加载后显示 "Error" 状态？**
检查 manifest.json 是否是合法的 JSON，entry 文件是否存在。

**Q: 如何让插件在 Morn 启动时自动加载？**
插件放在 `~/.local/share/morn/plugins/` 目录下，Morn 启动时自动扫描。

**Q: Python 插件超时了？**
Python 插件默认超时 30 秒。复杂的任务请拆分成多次调用。

**Q: 插件更新后需要重启 Morn 吗？**
目前需要重启。热加载功能将在后续版本支持。
