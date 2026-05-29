# Morn v0.4 第五轮：安全验证器 EventBus 集成 + 热重载

## 编程原则（同前四轮）

---

## 任务 1：创建 EventBus 驱动的 SecurityValidator

**文件**：`morn_core/security/security_validator.py`（新建）

将现有分散的安全逻辑（rules.py 的规则集、risk_guard.py 的风险分级、cli_executor.py 的模式拦截）统一到 EventBus 驱动的 SecurityValidator 中。

### 核心类

```python
class SecurityValidator:
    """安全验证器：EventBus 驱动的统一安全校验门。
    
    对应架构文档 §3.5 行动指令协议 + §8 安全体系。
    作为内核组件存在，不依赖任何插件。
    依赖创建者的持久化配置（风险偏好、权限规则）作为校验输入。
    """

    def __init__(self, config: dict, event_bus: Optional[EventBus] = None):
        self._config = config
        self._event_bus = event_bus
        self._rules = self._load_rules(config)
        self._risk_levels = self._load_risk_config(config)
        self._stats = {"allowed": 0, "blocked": 0, "confirmed": 0}
        self._last_reload = time.time()
        self._config_path = None  # 由 set_config_path() 设置

    def validate(self, action_type: str, params: dict,
                 source_plugin: str, risk_level: str,
                 risk_preference: str) -> ValidationResult:
        """核心校验函数——同步、无状态、硬编码规则。
        
        返回 ValidationResult(action, reason, suggested_level)
        """
        ...

    async def publish_alert(self, result: ValidationResult,
                            source_plugin: str,
                            risk_level: str) -> None:
        """如果 event_bus 存在，发布 security.alert 事件"""
        ...

    def set_config_path(self, path: str) -> None:
        """设置配置路径，供热重载使用"""
        ...

    def reload_config(self) -> int:
        """从 config_path 重新加载规则和风险配置。
        返回新加载的规则数量。配置未变化时返回 0。
        """
        ...

    def get_stats(self) -> dict:
        """返回累计统计"""
        ...
```

### ValidationResult

```python
@dataclass
class ValidationResult:
    action: str           # "allow" | "block" | "confirm"
    reason: str           # 人类可读原因
    suggested_level: str  # 建议的 risk_level（原始值可能被验证器修正）
    rule_id: Optional[str]  # 匹配的规则 ID（如有）
```

### 校验逻辑

1. 检查 `risk_level` 是否匹配创建者的 `risk_preference`（来自 config.yaml）
   - `green`：自动放行
   - `yellow`：放行并记录日志
   - `orange`：如果创建者风险偏好 ≤ `orange` 则要求确认
   - `red`：拦截
   - `black`：绝对禁区，直接拦截不记录

2. 检查 `params` 是否匹配白名单/黑名单规则（来自 rules.py 的 SecurityRule 列表）
   - 命中黑名单 → block + publish security.alert
   - 不在白名单内（如果该 action_type 有白名单要求）→ block

3. 检查 `source_plugin` 是否有当前 `action_type` 的权限
   - 基于创建者配置的权限规则（simple ACL: plugin_id → [allowed_action_types]）

4. 检查是否为高风险操作（`risk_level == "red"` 或包含 `DANGER_*` 规则命中）
   - 高风险 → block + publish security.alert

5. 累计统计

### 热重载（ADR-006 companion）

```python
async def watch_config_reload(validator: SecurityValidator, interval: float = 5.0):
    """每 5 秒检查配置文件变化，变化时调用 validator.reload_config()
    配置文件路径通过 validator.set_config_path() 设置
    """
    last_mtime = 0
    while True:
        try:
            mtime = os.path.getmtime(validator._config_path)
            if mtime != last_mtime:
                last_mtime = mtime
                count = validator.reload_config()
                if count > 0 and validator._event_bus:
                    await validator._event_bus.publish(Event(
                        type="security.config_reloaded",
                        payload={"rules_loaded": count},
                        source="security_validator",
                        priority=Priority.MEDIUM,
                    ))
        except (OSError, AttributeError):
            pass
        await asyncio.sleep(interval)
```

---

## 任务 2：行动指令协议集成

**文件**：`morn_core/action/cli_executor.py`

### 改动

1. 添加事件发布：CLI 命令执行前通过 SecurityValidator 校验
2. 校验通过才执行，校验不通过发布 security.alert 事件

```python
class CLIExecutor:
    def __init__(self, config=None, validator: Optional[SecurityValidator] = None,
                 event_bus: Optional[EventBus] = None):
        self._validator = validator
        self._event_bus = event_bus
        ...

    async def execute(self, command: str, source_plugin: str = "cli",
                      risk_level: str = "yellow") -> ExecResult:
        # 1. 安全检查
        if self._validator:
            result = self._validator.validate(
                action_type="execute_command",
                params={"cmd": command},
                source_plugin=source_plugin,
                risk_level=risk_level,
                risk_preference=self._config.get("risk_preference", "yellow"),
            )
            if result.action == "block":
                if self._event_bus:
                    await self._validator.publish_alert(result, source_plugin, risk_level)
                return ExecError(f"blocked: {result.reason}")
            elif result.action == "confirm":
                # 需要创建者确认——发布 security.alert 等待确认
                if self._event_bus:
                    await self._event_bus.publish(Event(
                        type="security.confirm_required",
                        payload={"command": command, "reason": result.reason},
                        source="security_validator",
                        priority=Priority.HIGH,
                    ))
                return ExecError("pending_confirmation")

        # 2. 现有 DANGEROUS_PATTERNS 检查保留（双保险）
        for pattern in _DANGEROUS_PATTERNS:
            if pattern in command:
                return ExecError("blocked_by_pattern")

        # 3. 执行命令
        ...
```

---

## 任务 3：Server 集成安全验证器

**文件**：`morn_core/server.py`

### 改动

1. 在创建 EventBus 后、注册插件前，创建 SecurityValidator
2. 传入 config 和 event_bus
3. 启动热重载协程
4. 将 validator 传递给 CLIExecutor 等需要安全检查的组件

```python
# 在 event_bus.start() 之后、register_all_plugin_hooks 之前添加：
from morn_core.security.security_validator import SecurityValidator
from morn_core.security.security_validator import watch_config_reload

# 创建安全验证器
security_validator = SecurityValidator(config, event_bus)
security_validator.set_config_path(str(data_dir / "config.json"))
await event_bus.publish(Event(
    type="security.config_reloaded",
    payload={"rules_loaded": len(security_validator._rules)},
    source="security_validator",
    priority=Priority.MEDIUM,
))

# 启动热重载
tasks.append(
    asyncio.create_task(
        watch_config_reload(security_validator),
        name="morn-sec-reload"
    )
)

# 创建 CLIExecutor 时传入 validator
from morn_core.action.cli_executor import CLIExecutor
state.cli_executor = CLIExecutor(config, validator=security_validator, event_bus=event_bus)
```

---

## 任务 4：测试

**文件**：`tests/test_security_validator.py`（新建）

测试覆盖：
1. **validate allow**：green/yellow 风险级别 → action="allow"
2. **validate block**：red/black 风险级别 → action="block"
3. **validate confirm**：orange 风险级别 + 创建者偏好 ≤ orange → action="confirm"
4. **黑名单匹配**：命令命中 DANGER_001（rm -rf）→ block + security.alert
5. **热重载**：修改配置文件 → 5 秒内规则更新
6. **CLIExecutor 集成**：通过 CLIExecutor 执行命令 → SecurityValidator 校验
7. **EventBus 集成**：security.alert 事件被发布和接收
8. **权限 ACL**：插件无权限执行 action_type → block
9. **累计统计**：allowed/blocked/confirmed 计数正确
10. **并发安全**：多个 validate 同时调用，统计一致

---

## 验收标准

1. ✅ `pytest tests/test_security_validator.py -v` 全部通过（至少 8 个测试）
2. ✅ `pytest tests/test_eventbus.py tests/test_eventbus_integration.py -v` 无回归
3. ✅ `python -c "from morn_core.security.security_validator import SecurityValidator; print('OK')"`
4. ✅ `python -c "from morn_core.server import main; print('OK')"` — import 正常
5. ✅ 已有 security 测试（如果有）无回归
6. ✅ config.json 修改后 5 秒内 validator 自动加载新规则
7. ✅ CLIExecutor 执行被拦截的命令时返回 ExecError 并发布 security.alert
