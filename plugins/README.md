# Morn Plugins

Morn 的插件体系分两个层次：

**1. Rust 原生插件（MornPlugin trait）** — 内核级，全权限
- 编译进二进制，性能零开销
- 可访问 Storage/Supervisor/Engine 等所有内核服务
- 生命周期由 `load_plugins()` 管理（init → activate → deactivate）
- **开发者：** 需要 Rust 知识，实现 `MornPlugin` trait

**2. 外部脚本插件（Python/JS + manifest.json）** — 用户态，低门槛
- 丢到 `plugins/` 目录即可自动发现
- 通过 stdin/stdout JSON-RPC 与 BridgePlugin 通信
- 权限由 `manifest.json` 的 `permissions` 字段控制
- **开发者：** 需要 Python 或 JavaScript 基础

---

## 目录结构

每个插件 = `plugins/<plugin-name>/` 子目录 + `manifest.json`：

```
plugins/
├── README.md
├── manifest.template.json
└── my-cool-plugin/
    ├── manifest.json       # 必填：插件元数据
    ├── main.py             # 入口脚本（Python 插件）
    ├── theme.css           # 主题插件可选
    └── assets/             # 其他资源文件
```

---

## manifest.json 字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | ✅ | 插件唯一名称 |
| `version` | string | ✅ | 语义版本号 |
| `description` | string | ✅ | 简短说明 |
| `author` | string | ❌ | 作者名 |
| `plugin_type` | string | ✅ | `theme`/`channel`/`tool`/`knowledge`/`ui_panel`/`protocol` |
| `entry` | string | ✅ | 入口脚本路径（相对插件目录） |
| `icon` | string | ❌ | 图标 URL 或路径 |
| `permissions` | string[] | ❌ | 所需权限：`network`, `filesystem`, `storage`, `notification` |
| `dependencies` | string[] | ❌ | 依赖的其他插件名称 |

---

## Python 插件开发

插件入口脚本通过 stdin/stdout 与 BridgePlugin JSON-RPC 通信：

```python
#!/usr/bin/env python3
import sys
import json

def handle_request(method, params):
    """处理来自 Morn 的请求"""
    if method == "ping":
        return {"pong": True}
    elif method == "execute":
        # 执行插件逻辑
        return {"result": "done"}
    return {"error": f"unknown method: {method}"}

for line in sys.stdin:
    try:
        req = json.loads(line.strip())
        response = handle_request(req.get("method"), req.get("params", {}))
        response["id"] = req.get("id")
        print(json.dumps(response), flush=True)
    except json.JSONDecodeError as e:
        print(json.dumps({"error": str(e), "id": None}), flush=True)
    except Exception as e:
        print(json.dumps({"error": str(e), "id": None}), flush=True)
```

---

## 安装方式

1. **手动安装：** 把插件目录放到 `plugins/` 下，重启 Morn
2. **Hub 安装：** 从 Hub 商店点击「安装」，自动下载到 `plugins/`
3. **CLI 安装：** `plugin_install` 命令

扫描自动发现，无需额外配置。
