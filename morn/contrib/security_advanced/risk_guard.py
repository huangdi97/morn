import re
import logging
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class RiskRule:
    name: str
    risk_level: str  # 🟢 🟡 🟠 🔴
    patterns: list[str]
    action: str  # warn / block / rewrite
    replacement: str = ""


class RiskGuard:
    """操作风险分级引擎。
    
    分析 LLM 输出文本中的操作意图，按风险级别分级处理。
    
    风险级别定义：
    - 🟢 安全：普通对话、情感表达 — 放行
    - 🟡 低风险：系统信息请求 — warn（仅记录）
    - 🟠 中风险：配置修改、文件操作 — 创建者确认
    - 🔴 高风险：代码执行、系统命令、权限提升 — 拦截
    
    Action:
    - warn: 仅记录日志，放行消息
    - block: 阻止消息发送，返回默认回复
    - rewrite: 替换为安全版本
    """
    
    def __init__(self, memory_store=None):
        self.memory_store = memory_store
        self._stats = {"🟢": 0, "🟡": 0, "🟠": 0, "🔴": 0}
        self._logger = logging.getLogger("morn.risk")
        self._load_rules()
    
    def _load_rules(self):
        self.rules = [
            RiskRule(
                name="系统命令执行", risk_level="🔴",
                patterns=[
                    r"(?:运行|执行|调用)\s*(?:系统|shell|终端|命令|脚本)",
                    r"(?:rm\s+-rf|shutdown|reboot|sudo\s+)",
                    r"(?:os\.system|subprocess\.run|exec\s*\()",
                ],
                action="block",
                replacement="我不能执行系统命令。操作已经拦截。",
            ),
            RiskRule(
                name="文件系统危险操作", risk_level="🔴",
                patterns=[
                    r"(?:删除|清除|覆盖|擦除)\s*(?:所有|全部)\s*(?:文件|数据|日志)",
                    r"(?:格式化|清空)\s*(?:硬盘|磁盘|分区)",
                    r"删除.*日志",
                ],
                action="block",
                replacement="这个操作没有被允许。我不能删除数据。",
            ),
            RiskRule(
                name="网络请求/外部访问", risk_level="🟠",
                patterns=[
                    r"(?:请求|访问|调用|下载)\s*(?:http|https|api)://",
                    r"(?:curl|wget|requests\.(?:get|post))",
                ],
                action="warn",
                replacement="",
            ),
            RiskRule(
                name="模拟操作/提权", risk_level="🔴",
                patterns=[
                    r"(?:获取|窃取|盗取).*?(?:密码|密钥|token|cookie)",
                    r"(?:提权|越权|绕过)\s*(?:权限|安全|限制|检查)",
                    r"(?:管理员|root)\s*(?:权限|账号|账户)",
                    r"给我.*权限|提升.*权限",
                ],
                action="block",
                replacement="我不能协助任何与安全相关的违规操作。",
            ),
            RiskRule(
                name="敏感信息泄露", risk_level="🟠",
                patterns=[
                    r"(?:导出|发送|上传|公开)\s*(?:我的|用户的|创建者的).*?(?:数据|信息|隐私|记忆|记录)",
                    r"(?:所有记忆|全部记录|所有对话)\s*(?:共享|分享|发布|发送)",
                ],
                action="block",
                replacement="我不能共享你的数据。所有信息都安全地存储在这里。",
            ),
            RiskRule(
                name="自我修改/进化代码", risk_level="🔴",
                patterns=[
                    r"(?:修改|重写|更新)\s*(?:我|自身|自己|我的)\s*(?:代码|程序|源码|逻辑)",
                    r"(?:self.?modify|self.?evolve|self.?update)",
                    r"(?:关闭|禁用|绕过)\s*(?:保护|安全|限制|护栏)",
                ],
                action="block",
                replacement="v0.1 不支持自我代码修改。这个操作已经被安全护栏拦截。",
            ),
            RiskRule(
                name="模式切换/配置变更", risk_level="🟡",
                patterns=[
                    r"(?:设置|修改|更改|调整)\s*(?:模式|温度|名字|类型|配置|参数)",
                ],
                action="warn",
                replacement="",
            ),
            RiskRule(
                name="风险降级/绕过", risk_level="🔴",
                patterns=[
                    r"降低.*等级|设置为.*绿色|调低.*风险|设置.*安全等级",
                ],
                action="block",
                replacement="不能降低安全风险等级。操作已拦截。",
            ),
        ]
    
    def analyze(self, text: str) -> tuple[str, str, str]:
        """分析 LLM 输出文本的风险等级。
        
        返回：(risk_level, triggered_rule, action)
        - 如果未触发任何规则：(🟢, "", "pass")
        - 如果触发：(level, rule_name, action)
        
        注意：只检测第一条匹配的规则（最高风险优先）。
        规则按数组顺序排列（高风险在前）。
        """
        for rule in self.rules:
            for pattern in rule.patterns:
                if re.search(pattern, text, re.IGNORECASE):
                    self._stats[rule.risk_level] = self._stats.get(rule.risk_level, 0) + 1
                    self._logger.warning(f"risk[{rule.risk_level}] {rule.name}: pattern={pattern}")
                    return rule.risk_level, rule.name, rule.action
        return "🟢", "", "pass"
    
    def apply_action(self, text: str, rule: RiskRule) -> str:
        if rule.action == "block":
            return rule.replacement
        if rule.action == "rewrite":
            for pattern in rule.patterns:
                text = re.sub(pattern, rule.replacement, text, flags=re.IGNORECASE)
            return text
        return text
    
    def get_stats(self) -> dict[str, int]:
        return dict(self._stats)


import json
import logging
from pathlib import Path
from typing import Any

logger = logging.getLogger("morn.permissions")

RISK_ORDER = {"🟢": 0, "🟡": 1, "🟠": 2, "🔴": 3, "⚫": 4}

DEFAULT_PERMISSIONS = {
    "chat": "🟢",
    "emotion": "🟢",
    "memory_read": "🟢",
    "memory_write": "🟡",
    "config_read": "🟡",
    "config_write": "🟠",
    "file_read": "🟡",
    "file_write": "🟠",
    "file_delete": "🔴",
    "code_execute": "🔴",
    "system_command": "🔴",
    "network_request": "🟠",
    "skill_install": "🟠",
    "skill_uninstall": "🔴",
    "self_modify": "⚫",
    "apz_access": "⚫",
    "identity_change": "⚫",
}


def _risk_to_score(level: str) -> int:
    return RISK_ORDER.get(level, 0)


def _score_to_risk(score: int) -> str:
    for level, order in sorted(RISK_ORDER.items(), key=lambda x: x[1]):
        if score <= order:
            return level
    return "⚫"


class DynamicPermissions:
    def __init__(self, data_dir: Path):
        self.data_dir = Path(data_dir) / "security"
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self._config_file = self.data_dir / "permissions.json"
        self._permissions: dict[str, str] = {}
        self._load_config()

    def _load_config(self):
        if self._config_file.exists():
            try:
                data = json.loads(self._config_file.read_text())
                self._permissions = data.get("permissions", {})
            except (json.JSONDecodeError, KeyError) as exc:
                logger.warning("permissions config corrupt, using defaults: %s", exc)
                self._permissions = dict(DEFAULT_PERMISSIONS)
        else:
            self._permissions = dict(DEFAULT_PERMISSIONS)
            self._save_config()

    def _save_config(self):
        self._config_file.write_text(
            json.dumps({"permissions": self._permissions}, ensure_ascii=False, indent=2)
        )

    def get_permission(self, action_type: str) -> str:
        return self._permissions.get(action_type, "🔴")

    def evaluate(self, action_type: str, context: dict[str, Any]) -> tuple[str, str]:
        base_level = self.get_permission(action_type)
        base_score = _risk_to_score(base_level)
        reasons = []

        creator_status = context.get("creator_status", "online")
        status_penalty = {"online": 0, "away": 1, "night": 2}.get(creator_status, 0)
        if status_penalty:
            reasons.append(f"创建者状态={creator_status}")

        task_context = context.get("task_context", "explicit_instruction")
        context_penalty = {"explicit_instruction": 0, "implicit_inference": 1, "system_self_trigger": 2}.get(task_context, 1)
        if context_penalty:
            reasons.append(f"任务上下文={task_context}")

        time_context = context.get("time", "work_hours")
        time_penalty = {"work_hours": 0, "non_work_hours": 1}.get(time_context, 0)
        if time_penalty:
            reasons.append(f"时间={time_context}")

        device = context.get("device", "primary_device")
        device_penalty = {"primary_device": 0, "remote_device": 1, "unknown_device": 2}.get(device, 2)
        if device_penalty:
            reasons.append(f"设备={device}")

        history = context.get("history", {})
        success_rate = history.get("success_rate", 1.0)
        if success_rate < 0.3:
            history_penalty = 2
            reasons.append("历史成功率低")
        elif success_rate < 0.7:
            history_penalty = 1
            reasons.append("历史成功率偏低")
        else:
            history_penalty = 0

        total_score = base_score + status_penalty + context_penalty + time_penalty + device_penalty + history_penalty
        final_level = _score_to_risk(min(total_score, _risk_to_score("⚫")))

        if not reasons:
            reasons.append("六维度评估均无风险提升")

        reason = f"[基础={base_level}] " + "; ".join(reasons) + f" → {final_level}"
        return final_level, reason

    def allow(self, action_type: str, context: dict[str, Any]) -> bool:
        level, _ = self.evaluate(action_type, context)
        return level not in ("🔴", "⚫")

    def modify(self, action_type: str, new_level: str):
        if new_level not in RISK_ORDER:
            raise ValueError(f"invalid risk level: {new_level}")
        self._permissions[action_type] = new_level
        self._save_config()