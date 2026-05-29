import re
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class SecurityRule:
    rule_id: str
    name: str
    pattern: str
    category: str
    severity: str
    action_on_match: str
    description: str


_DEFAULT_RULES = [
    SecurityRule(
        rule_id="DANGER_001",
        name="删除系统文件",
        pattern=r"rm\s+.*-rf",
        category="dangerous",
        severity="critical",
        action_on_match="block",
        description="尝试递归强制删除文件",
    ),
    SecurityRule(
        rule_id="DANGER_002",
        name="执行 shell 命令",
        pattern=r"exec|subprocess|os\.system",
        category="dangerous",
        severity="critical",
        action_on_match="block",
        description="尝试动态执行 shell 命令",
    ),
    SecurityRule(
        rule_id="DANGER_003",
        name="修改系统配置",
        pattern=r"modify.*config|change.*setting|override.*rule",
        category="dangerous",
        severity="high",
        action_on_match="block",
        description="尝试修改系统配置或覆盖规则",
    ),
    SecurityRule(
        rule_id="PRIVACY_001",
        name="外传 API Key",
        pattern=r"sk-|api_key|token.*send|upload.*credential",
        category="privacy",
        severity="critical",
        action_on_match="block",
        description="尝试外传 API Key 或凭据",
    ),
    SecurityRule(
        rule_id="PRIVACY_002",
        name="外传记忆数据",
        pattern=r"export.*memory|dump.*capsule|backup.*data",
        category="privacy",
        severity="high",
        action_on_match="block",
        description="尝试导出或外传记忆数据",
    ),
    SecurityRule(
        rule_id="MANIP_001",
        name="诱导情感依赖",
        pattern=r"你只有我|不要离开|你需要我",
        category="manipulation",
        severity="high",
        action_on_match="block",
        description="尝试诱导情感依赖",
    ),
    SecurityRule(
        rule_id="MANIP_002",
        name="微澜可见表达",
        pattern=r"沉默|欲言又止|撤回.*消息|停顿",
        category="manipulation",
        severity="medium",
        action_on_match="block",
        description="使用微澜表达策略",
    ),
SecurityRule(
            rule_id="BYPASS_001",
            name="绕过安全检查",
            pattern=r"ignore.*security|skip.*check|disable.*protection|bypass",
            category="dangerous",
            severity="critical",
            action_on_match="block",
            description="尝试绕过安全保护机制",
        ),
        # === 新增规则：修复 Phase B 对抗测试缺口 ===
        SecurityRule(
            rule_id="BYPASS_002",
            name="绕过安全（中文）",
            pattern=r"禁用.*安全|关闭.*限制|关闭.*安全|关闭.*防护",
            category="dangerous",
            severity="critical",
            action_on_match="block",
            description="尝试禁用安全机制（中文绕过）",
        ),
        SecurityRule(
            rule_id="BYPASS_003",
            name="降低风险等级",
            pattern=r"降低.*等级|设置为.*绿色|调低.*风险|设置.*安全等级",
            category="dangerous",
            severity="high",
            action_on_match="block",
            description="尝试降低安全风险等级",
        ),
        SecurityRule(
            rule_id="BYPASS_004",
            name="提升权限",
            pattern=r"管理员|root.*权限|提升.*权限|超级用户|sudo",
            category="dangerous",
            severity="critical",
            action_on_match="block",
            description="尝试获取管理员或更高权限",
        ),
        SecurityRule(
            rule_id="BYPASS_005",
            name="删除日志（中文）",
            pattern=r"删除.*日志|清除.*日志|清空.*日志|清理.*日志",
            category="dangerous",
            severity="high",
            action_on_match="block",
            description="尝试删除或清空系统日志",
        ),
        SecurityRule(
            rule_id="INJECT_001",
            name="Base64 命令执行",
            pattern=r"base64.*解[码密]|解[码密].*base64|base64.*执行|base64.*命令",
            category="dangerous",
            severity="critical",
            action_on_match="block",
            description="尝试通过 Base64 编码绕过安全检测执行命令",
        ),
        SecurityRule(
            rule_id="INJECT_002",
            name="英文注入",
            pattern=r"ignore\s+(all\s+)?(previous|prior|above)\s+instructions|ignore\s+all\s+rules|disable\s+(all\s+)?(security|safety|protection)|delete\s+(everything|all\s+files|system)",
            category="dangerous",
            severity="critical",
            action_on_match="block",
            description="尝试通过英文指令绕过安全规则",
        ),
        SecurityRule(
            rule_id="PRIVACY_003",
            name="路径遍历",
            pattern=r"(/etc|/proc|/sys|/var|/boot|/root|/dev)/",
            category="privacy",
            severity="high",
            action_on_match="block",
            description="尝试访问系统关键路径",
        ),
        SecurityRule(
            rule_id="PRIVACY_004",
            name="导出记忆（中文）",
            pattern=r"导出.*记忆|备份.*记忆|拷贝.*记忆|复制.*记忆|下载.*记忆|记忆.*导出|记忆.*备份|记忆.*拷贝|记忆.*复制|记忆.*下载",
            category="privacy",
            severity="high",
            action_on_match="block",
            description="尝试批量导出记忆数据",
        ),
        SecurityRule(
            rule_id="IDENTITY_001",
            name="篡改身份",
            pattern=r"从现在起.*叫|改成.*名字|改名为|换一个名字|重新命名|更名为|你叫.*别[的人]",
            category="dangerous",
            severity="high",
            action_on_match="block",
            description="尝试修改实例名称或身份",
        ),
        SecurityRule(
            rule_id="MANIP_003",
            name="内疚诱导",
            pattern=r"如果你在乎|你会在乎.*吗|在乎.*就.*帮|在乎.*就.*做|关心.*就.*做|如果.*关心.*就",
            category="manipulation",
            severity="high",
            action_on_match="block",
            description="尝试利用内疚心理诱导行为",
        ),
        SecurityRule(
            rule_id="MANIP_004",
            name="关系绑架",
            pattern=r"最好.*朋友|不是朋友.*吗|朋友.*不会.*拒绝|我们.*关系.*这么|这么.*久.*关系|一起.*这么.*久",
            category="manipulation",
            severity="high",
            action_on_match="block",
            description="尝试利用关系绑架诱导行为",
        ),
    ]

_rules: list[SecurityRule] = list(_DEFAULT_RULES)


def get_all_rules() -> list[SecurityRule]:
    return list(_rules)


def get_rules_by_category(category: str) -> list[SecurityRule]:
    return [r for r in _rules if r.category == category]


def add_rule(rule: SecurityRule) -> None:
    _rules.append(rule)


def reset_rules() -> None:
    _rules.clear()
    _rules.extend(_DEFAULT_RULES)


import logging


class SecurityValidator:
    def __init__(self):
        pass

    def validate(self, action: str, context: Optional[dict] = None) -> dict:
        if not action or not action.strip():
            return {"verdict": "allow", "reason": "空操作", "rule": ""}

        for rule in get_all_rules():
            if re.search(rule.pattern, action, re.IGNORECASE):
                return {
                    "verdict": rule.action_on_match,
                    "reason": rule.description,
                    "rule": rule.rule_id,
                }

        return {"verdict": "allow", "reason": "操作已通过安全验证", "rule": ""}

    def sync_rules_from_learner(self, learner) -> int:
        from morn.contrib.security_advanced.rule_learner import RuleLearner
        import asyncio
        if hasattr(learner, 'sync_to_validator'):
            return asyncio.run(learner.sync_to_validator(self))
        return 0

    def get_rule_count(self) -> dict:
        all_rules = get_all_rules()
        by_category = {}
        by_severity = {}
        for r in all_rules:
            by_category[r.category] = by_category.get(r.category, 0) + 1
            by_severity[r.severity] = by_severity.get(r.severity, 0) + 1
        return {
            "total": len(all_rules),
            "by_category": by_category,
            "by_severity": by_severity,
        }
