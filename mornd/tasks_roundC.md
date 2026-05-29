# Morn — 轮C：系统操控 CLI/API 基础

## 约束
1. 不碰无关文件
2. 低耦合，传ID不传对象
3. 文件不超300行
4. 所有已有测试保持通过

---

## 模块C1：多路径自动选择框架

### 新文件
- `morn_core/action/multi_path.py`

### 规格
ActionRouter 类：

**多路径自动选择**（分层响应策略）：

| 优先级 | 路径 | 适用场景 | 延迟 |
|--------|------|---------|------|
| 1 | **CLI** | 本地命令，有终端访问权 | 毫秒级 |
| 2 | **API** | HTTP API 调用 | 百毫秒级 |
| 3 | **浏览器** | Playwright 自动化（标记为v1.0+能力） | 秒级 |
| 4 | **人工建议** | 以上都不可行 | — |

`route(action_type, params, context)` → 选择最优路径并执行
`get_available_paths()` → 返回当前可用路径列表
`get_fallback_chain(action_type)` → 返回降级链

路径选择逻辑：
- 优先选延迟最低的可行路径
- 记录每次路径选择结果（成功/失败、耗时）
- 后续选择时参考历史成功率
- 可通过配置强制指定路径

### 新增测试
`tests/test_action_router.py` — 10+测试
- 自动选择CLI
- 自动选择API
- CLI失败自动降级API
- 全部不可行返回人工建议
- 历史成功率影响选择
- 强制指定路径

---

## 模块C2：CLI 执行器

### 新文件
- `morn_core/action/cli_executor.py`

### 规格
CLIExecutor 类：

`execute(command, timeout=30, workdir=None)` → 执行本地命令
`is_available()` → 检查终端是否可用
`validate(command)` → 安全检查（拒绝rm -rf /等危险命令）
`get_shell()` → 返回当前shell路径

安全规则：
- 黑名单：rm -rf /, dd if=, :(){ :|:& };:, chmod 777 /, sudo 无确认
- 所有命令执行前通过 validate()
- 超时自动终止（默认30秒）
- 记录执行日志

### 新增测试
`tests/test_cli_executor.py` — 10+测试
- 执行简单命令
- 安全检查拦截危险命令
- 安全命令放行
- 超时终止
- is_available
- 执行日志

---

## 模块C3：API 调用器

### 新文件
- `morn_core/action/api_caller.py`

### 规格
APICaller 类：

`call(method, url, headers=None, body=None, timeout=10)` → 发起HTTP请求
`is_available()` → 检查网络是否可用
`validate(url)` → 安全检查（拒绝内网/敏感端点）

安全规则：
- 黑名单：127.0.0.1, localhost, 10.x.x.x, 172.16-31.x.x, 192.168.x.x
- 只允许 HTTPS（除非配置明确允许HTTP）
- 超时自动终止（默认10秒）
- 记录调用日志

### 新增测试
`tests/test_api_caller.py` — 8+测试
- GET请求
- POST请求
- 超时终止
- 内网拦截
- 非HTTPS拦截
- is_available

---

## 验收
1. `python3 -m pytest tests/ -q` — 全部通过
2. CLI危险命令被拦截
3. API内网请求被拦截
4. 降级链正常工作
