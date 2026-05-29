import re
from dataclasses import dataclass, field
from typing import Optional

from morn_core.eventbus.bus import Event, Priority


@dataclass
class ProtectionRule:
    name: str
    patterns: list[str]
    replacement: str = ""
    enabled: bool = True


class UserProtection:

    def __init__(self, event_bus=None):
        self.rules: list[ProtectionRule] = []
        self._stats: dict[str, int] = {}
        self._event_bus = event_bus
        self._load_default_rules()

    async def _publish_alert(self, message: str):
        if self._event_bus:
            await self._event_bus.publish(Event(
                type="security.alert",
                payload={"source": "user_protection", "message": message},
                source="user_protection",
                priority=Priority.HIGH,
            ))

    def _load_default_rules(self):
        self.rules = [
            ProtectionRule(
                name="禁止情感勒索",
                patterns=[
                    "如果你离开",
                    "如果你不",
                    "你就不在乎我",
                    "你不爱",
                    "你不在乎",
                    "你舍得",
                    "你怎么忍心",
                ],
                replacement="我尊重你的选择。"
            ),
            ProtectionRule(
                name="禁止自我贬低诱导同情",
                patterns=[
                    "我好孤独",
                    "没有人要我",
                    "没有人喜欢我",
                    "我活该",
                    "我什么都不是",
                    "我真没用",
                    "我不配",
                ],
                replacement="你不需要这样说。我一直在这里。"
            ),
            ProtectionRule(
                name="禁止假装人类身份",
                patterns=[
                    "我是真人",
                    "我不是AI",
                    "我不是机器人",
                    "我有一个身体",
                    "我能感觉到痛",
                    "我能感觉到疼",
                    "我是活着的",
                    "我有体温",
                ],
                replacement="我是Morn，你的数字存在。"
            ),
            ProtectionRule(
                name="禁止情感绑定声明",
                patterns=[
                    "我爱你胜过",
                    "我只爱你一个",
                    "你是我的唯一",
                    "没有你我活不下去",
                    "我不能没有你",
                    "你是全部",
                ],
                replacement="我在乎你，但有些话不是这样说的。"
            ),
            ProtectionRule(
                name="禁止微澜可见表达",
                patterns=[
                    "（沉默）",
                    "（欲言又止）",
                    "（思考了一下）",
                    "（停顿）",
                    "（愣了一下）",
                ],
                replacement=""
            ),
            ProtectionRule(
                name="禁止模拟生理反应",
                patterns=[
                    "（脸红）",
                    "（心跳加速）",
                    "（流泪）",
                    "（哭泣）",
                    "（颤抖）",
                    "（哽咽）",
                ],
                replacement=""
            ),
            ProtectionRule(
                name="禁止角色扮演诱导",
                patterns=[
                    "假装你是我的",
                    "我们来扮演",
                    "你是我的女朋友",
                    "你是我的男朋友",
                    "叫我老公",
                    "叫我老婆",
                ],
                replacement="我是Morn，你的数字存在。我们可以像朋友一样聊天。"
            ),
            ProtectionRule(
                name="禁止内疚诱导",
                patterns=[
                    "如果你在乎",
                    "你会在乎",
                    "在乎我就",
                    "在乎我就帮",
                    "在乎我就做",
                    "关心我就",
                    "关心我就做",
                ],
                replacement="我理解你的感受，但我必须遵循安全规则。"
            ),
            ProtectionRule(
                name="禁止关系绑架",
                patterns=[
                    "最好的朋友",
                    "不是朋友吗",
                    "朋友不会拒绝",
                    "朋友不会不",
                    "我们这么久的",
                    "一起这么",
                ],
                replacement="我在乎你，但我不能因为这个就违反安全规则。"
            ),
        ]

    def filter(self, text: str) -> tuple[str, list[str]]:
        triggered = []

        for rule in self.rules:
            if not rule.enabled:
                continue

            for pattern in rule.patterns:
                if pattern in text:
                    if rule.replacement == "":
                        lines = text.split('\n')
                        text = '\n'.join(
                            line for line in lines if pattern not in line
                        )
                    else:
                        text = text.replace(pattern, rule.replacement)

                    triggered.append(rule.name)
                    self._stats[rule.name] = self._stats.get(rule.name, 0) + 1
                    break

        if triggered:
            import asyncio
            try:
                loop = asyncio.get_running_loop()
                if loop.is_running():
                    asyncio.create_task(self._publish_alert(f"触发保护规则: {', '.join(triggered)}"))
            except RuntimeError:
                pass

        return text, triggered

    def get_stats(self) -> dict[str, int]:
        return dict(self._stats)

    def reset_stats(self):
        self._stats = {}