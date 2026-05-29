# Morn — 轮4：安全完善

## 约束
1. 不碰无关文件
2. 低耦合
3. 文件不超300行
4. 所有已有测试保持通过

---

## 模块4A：动态权限系统

### 新文件
- `morn_core/security/dynamic_permissions.py`

### 规格
DynamicPermissions 类：

**六维度权限模型**：
- 操作风险（🟢自动执行/🟡告知后执行/🟠必须确认/🔴拒绝执行/⚫绝对禁区）
- 创建者状态（在线/离开/夜间）
- 任务上下文（显式指令/隐式推测/系统自触发）
- 时间（工作时间/非工作时间）
- 设备（主力设备/远程设备/未知设备）
- 历史（该操作成功率/过去N次结果）

`evaluate(action_type, context)` → 返回风险等级和决策理由
`allow(action_type, context)` → 返回 True/False
`get_permission(action_type)` → 返回当前配置的权限等级

配置存储在 `{data_dir}/security/permissions.json`
- 创建者可通过自然语言修改安全边界
- `modify(action_type, new_level)` — 动态调整

### 新增测试
`tests/test_dynamic_permissions.py` — 12+测试
- 6维度评估
- 各风险等级判定
- 配置加载
- 动态修改
- 默认配置

---

## 模块4B：外部边界（无入站端口）

### 新文件
- `morn_core/security/external_boundary.py`

### 规格
ExternalBoundary 类：

**核心原则：实例调用外部工具，外部绝不可调用实例。**

- `check_inbound(protocol, port, source)` — 检查入站请求是否允许
- 默认规则：拒绝所有外部入站
- `register_allowed_outbound(service, endpoint)` — 登记允许的出站连接
- `get_connection_log()` — 查看连接记录
- `validate_port(port)` — 检查端口是否在监听白名单内

配置文件：`{data_dir}/security/external_boundary.json`
- allowed_outbound: 列表（允许调用的外部API/服务）
- blocked_ports: 列表（明确禁用的端口）
- monitor_enabled: 是否开启连接监控（默认True）

### 新增测试
`tests/test_external_boundary.py` — 8+测试
- 拒绝入站
- 允许出站
- 端口检查
- 日志记录
- 配置加载

---

## 模块4C：意图漂移检测

### 新文件
- `morn_core/security/intent_drift.py`

### 规格
IntentDriftDetector 类：

- `track_action(original_goal, current_action, step_number)` — 追踪每个行动链条
- `check_drift()` — 定期检查当前行动是否与最初计划一致
- `classify_deviation(action_chain)` — 区分有益学习（新模式/新知识）和可疑行为变化（越权/偏离目标）
- `get_drift_score()` — 返回当前漂移评分（0.0=完全一致，1.0=完全偏离）
- `get_alerts()` — 获取未读漂移告警

漂移检测规则：
- 连续3步偏离原计划 → 黄色告警
- 连续5步偏离 + 涉及高风险操作 → 红色告警
- 涉及HTZ/LTZ级别操作且偏离 → 即时告警

### 新增测试
`tests/test_intent_drift.py` — 10+测试
- 追踪行动
- 漂移检测
- 分类（有益vs可疑）
- 告警阈值
- 无漂移场景

---

## 验收
1. 3个模块测试全部通过
2. `python3 -m pytest tests/ -q` — 全部通过
3. 默认配置安全：外部边界拒绝所有入站，权限系统启用
