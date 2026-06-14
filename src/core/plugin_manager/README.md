# PluginManager

模块路径: `src/core/plugin_manager/`

## 架构

PluginManager 管理 Morn 的插件系统，支持扫描、加载、激活、停用四个生命周期。

```
扫描 (scan) → 加载 (load) → 激活 (activate) → 停用 (deactivate)
```

## 插件类型

| 类型 | plugin_type | 说明 |
|------|-------------|------|
| 主题 | theme | CSS 主题，通过 theme.css 注入 |
| 渠道 | channel | 通信渠道集成 |
| UI 插槽 | ui_slot | 前端界面扩展 |
| 协议 | protocol | 通信协议扩展 |
| 工具 | tool | 后端工具扩展 |

## Manifest 格式

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "Plugin description",
  "author": "Author Name",
  "plugin_type": "theme",
  "entry": "main.js"
}
```

- `entry`: 插件入口文件，相对路径。`.js` 为 JavaScript，`.py` 为 Python 脚本。
- `name`: 全局唯一，用于标识插件。

## 使用

```rust
let mut mgr = PluginManager::new(plugin_dir);
mgr.scan()?;                    // 发现新插件
mgr.load("my-plugin")?;         // 加载
mgr.activate("my-plugin")?;     // 激活（CSS 注入或脚本执行）
mgr.deactivate("my-plugin")?;   // 停用
mgr.list();                     // 列出全部
mgr.get("my-plugin");           // 获取单个
```

## 主题 CSS

对于 `plugin_type = "theme"` 的插件，`activate()` 会自动读取 entry 指向的 `.css` 文件内容并缓存，通过 `get_theme_css(name)` 获取。前端通过 Tauri 命令 `apply_theme` 获取 CSS 内容并注入到页面。

## 错误类型

使用 `PluginError` 枚举:
- `NotFound(String)` — 插件未找到
- `InvalidManifest(String, String)` — manifest 格式错误
- `Scaffold(String)` — 创建插件失败
- `Llm(String)` — LLM 调用错误
- `Io(std::io::Error)` — IO 错误（自动转换）
- `Other(String)` — 其他错误
