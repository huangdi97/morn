# Morn — 轮A：基础设施完善

## 模块A1：server.py 集成新模块

### 修改文件
- `morn_core/server.py`

### 规格
将以下新模块挂载到 server.py 的启动循环和 MornState：

| 模块 | 初始化位置 | 后台循环 |
|------|-----------|---------|
| **BondTracker** | `emotion/bond_tracker.py` → `init_subsystems()` | 每次对话后更新bond |
| **RetrievalEngine** | `memory/retrieval.py` → `init_subsystems()` | 替换旧检索路径 |
| **LayeredRetrievalEngine** | 同上，mode默认为"balanced" | — |
| **DynamicPermissions** | `security/dynamic_permissions.py` → `init_subsystems()` | — |
| **ExternalBoundary** | `security/external_boundary.py` → `init_subsystems()` | — |
| **IntentDriftDetector** | `security/intent_drift.py` → `init_subsystems()` | 后台漂移检测循环 |
| **ChallengeMode** | `consciousness/challenge_mode.py` → `init_subsystems()` | — |
| **AuditAgent** | `memory/audit_agent.py` → `init_subsystems()` | 后台审计循环 |

集成要求：
- 新增模块添加到 MornState 的字段
- 在 init_subsystems() 中初始化（try/except 包裹，失败不阻塞整体启动）
- 需要后台循环的模块启动对应 asyncio.create_task
- 所有新增模块默认启用，可通过配置关闭
- config 中添加对应配置项（默认值）

### 新增测试
`tests/test_server_new_modules.py` — 8+测试
- 各模块导入正常
- 初始化不抛异常
- 配置可禁用
- 后台循环可启动/停止

---

## 模块A2：安装 chromadb

### 操作
```bash
pip install chromadb
```

### 验证
`python3 -c "from morn_core.memory.vector_store import VectorStore; print('chromadb OK')"` — 不再输出"chromadb not installed"

---

## 模块A3：pyproject.toml 补依赖

### 修改文件
`pyproject.toml`

### 规格
补全当前代码实际依赖但缺失的包：
- chromadb
- matplotlib（已通过 pip 安装，但未写入 pyproject.toml）
- 检查缺失的其他包

### 新增测试
- 已有 test_project_structure.py 验证 dependencies 可解析

---

## 验收
1. `python3 -m pytest tests/ -q` — 全部通过
2. `python3 -m morn_core.server --instance test & sleep 3 && kill %1` — 正常启动退出
3. chromadb 不再报"not installed"
4. pyproject.toml 包含所有运行时依赖
