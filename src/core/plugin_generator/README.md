# PluginGenerator

模块路径: `src/core/plugin_generator/`

## 功能

从自然语言描述自动生成插件骨架代码。使用 LLM 解析用户意图，创建 manifest.json 和入口文件。

## 流程

```
用户描述 → LLM 解析 → PluginSpec → scaffold → 插件文件
```

## PluginSpec 结构

```rust
pub struct PluginSpec {
    pub name: String,           // 插件名称
    pub plugin_type: String,    // theme|channel|ui_slot|protocol|tool
    pub description: String,    // 描述
    pub entry_content: String,  // 入口文件内容
    pub entry_filename: String, // 入口文件名（main.js 或 main.py）
}
```

## 函数

- `parse_nl_to_spec(nl, chat_fn)` — 用 LLM 解析自然语言描述，返回结构化的 PluginSpec
- `scaffold_plugin(spec, output_dir)` — 创建插件目录和文件
- `generate_plugin_from_nl(nl, output_dir, chat_fn)` — 组合上述两步，返回 manifest 路径

## Tauri 命令

前端通过 `invoke("generate_plugin_from_nl", { nl: "..." })` 调用，返回生成的 manifest 路径。
