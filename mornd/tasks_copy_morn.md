# 任务：将 morn/ 框架包复制到项目根目录

将 `/home/hermes/morn/mornd/morn/` 复制到 `/home/hermes/morn/morn/`（与 mornd 平级），同时分三大块组织：

```
/home/hermes/morn/
├── morn/                    ← pip 包
│   ├── __init__.py          ← 公开 API 门面
│   ├── core/                ← ① 内核
│   │   ├── __init__.py
│   │   ├── bus.py           ← EventBus
│   │   ├── plugin.py        ← MornPlugin 基类
│   │   ├── plugin_loader.py ← PluginLoader
│   │   ├── plugin_registry.py
│   │   ├── hooks.py
│   │   ├── heartbeat.py
│   │   ├── security.py      ← SecurityValidator
│   │   └── skill_store_interface.py
│   ├── sdk/                 ← ② SDK 服务接口层
│   │   ├── __init__.py
│   │   ├── chat.py
│   │   ├── memory.py
│   │   ├── security.py
│   │   └── presence.py
│   ├── plugins/             ← ③ 11 个插件
│   │   ├── __init__.py
│   │   ├── health_monitor.py
│   │   ├── dream_engine.py
│   │   ├── self_reflection.py
│   │   ├── identity_affirmer.py
│   │   ├── self_pruner.py
│   │   ├── bond_tracker.py
│   │   ├── intent_drift.py
│   │   ├── audit.py
│   │   ├── thinking_evolution.py
│   │   ├── milestones.py
│   │   └── hindsight.py
│   ├── cli/                 ← ④ CLI Presence
│   │   ├── __init__.py
│   │   └── main.py
│   └── contrib/             ← ⑤ 高级组件
│       ├── __init__.py
│       ├── memory_advanced/
│       └── security_advanced/
│
├── mornd/                   ← 原项目（保留不动）
│   ├── morn_core/
│   ├── morn/                ← 此目录将被复制走，源保留不动
│   └── ...
│
├── docs/                    ← ⑥ 文档
│   ├── MORN_QUICKSTART.md
│   ├── API_REFERENCE.md
│   └── PLUGIN_DEV_GUIDE.md
│
└── pyproject.toml           ← 更新 packages.find 包含 morn*
```

## 执行步骤

### Step 1: 创建目录结构

```bash
mkdir -p /home/hermes/morn/morn/core
mkdir -p /home/hermes/morn/morn/sdk
mkdir -p /home/hermes/morn/morn/plugins
mkdir -p /home/hermes/morn/morn/cli
mkdir -p /home/hermes/morn/morn/contrib/memory_advanced
mkdir -p /home/hermes/morn/morn/contrib/security_advanced
mkdir -p /home/hermes/morn/docs
```

### Step 2: 复制 core/

从 `/home/hermes/morn/mornd/morn/kernel/` 复制到 `/home/hermes/morn/morn/core/`：
- bus.py, plugin.py, plugin_loader.py, plugin_registry.py, hooks.py, heartbeat.py, security.py, skill_store_interface.py, __init__.py

注意：`core/__init__.py` 需要修正内部 import 路径——原来是 `morn.kernel.xxx` 引用其他子包的地方，改为 `morn.core.xxx`。

具体来说，检查 core/ 下的每个文件的 `from .xxx` 或 `from morn.kernel.xxx` 引用，应改为 `from morn.core.xxx` 或 `from .xxx`。

### Step 3: 复制 sdk/

从 `/home/hermes/morn/mornd/morn/sdk/` 复制到 `/home/hermes/morn/morn/sdk/`：
- 5 个文件：__init__.py, chat.py, memory.py, security.py, presence.py

检查 import：`sdk/chat.py` 引用 `morn_core.chat.engine.ChatEngine` — 这个引用路径不变，因为仍然指向旧的 morn_core 包中的实际实现。

### Step 4: 复制 plugins/

从 `/home/hermes/morn/mornd/morn/plugins/` 复制到 `/home/hermes/morn/morn/plugins/`：
- 12 个文件全部复制

检查 import：`plugins/` 中的文件引用 `from morn.kernel.xxx` — 改为 `from morn.core.xxx`。

### Step 5: 复制 cli/

从 `/home/hermes/morn/mornd/morn/cli/` 复制到 `/home/hermes/morn/morn/cli/`：
- 2 个文件

检查 import：`cli/main.py` 引用 `morn.sdk.presence.MornPresence` — 改为 `morn.sdk.presence.MornPresence`（不变）。

### Step 6: 复制 contrib/

从 `/home/hermes/morn/mornd/morn/contrib/` 复制到 `/home/hermes/morn/morn/contrib/`：
- 所有文件和子目录

检查 import：`contrib/` 中文件引用 `morn_core.xxx` 的不变。

### Step 7: 创建新的 __init__.py

`/home/hermes/morn/morn/__init__.py` 的内容需要修正 import：
- `from morn.kernel import ...` → `from morn.core import ...`
- `from morn.sdk import ...` → 保持不变

### Step 8: 复制文档

将三份文档从 `/home/hermes/morn/mornd/morn/` 复制到 `/home/hermes/morn/docs/`：
- MORN_QUICKSTART.md
- API_REFERENCE.md  
- PLUGIN_DEV_GUIDE.md

### Step 9: 更新 pyproject.toml

更新 `/home/hermes/morn/pyproject.toml`（注意：是根目录的 pyproject.toml，如果不存在则从 mornd/ 复制一份过来修改）。

```toml
[tool.setuptools.packages.find]
include = ["morn*", "morn.core*", "morn.sdk*", "morn.plugins*", "morn.cli*", "morn.contrib*"]
```

如果根目录没有 pyproject.toml，从 mornd/ 复制一份。

### Step 10: 验证

```bash
cd /home/hermes/morn
pip install -e .

python -c "from morn import EventBus, ChatEngine, MemoryStore; print('✓ morn OK')"
python -c "from morn.core.plugin import MornPlugin; print('✓ core OK')"
python -c "from morn.plugins import HealthMonitorPlugin; print('✓ plugins OK')"
python -c "from morn.sdk.presence import MornPresence; print('✓ sdk OK')"
python -c "from morn.cli import CLIPresence; print('✓ cli OK')"
```
