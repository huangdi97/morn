import asyncio
import json
import logging
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from morn.core.bus import Event, Priority

logger = logging.getLogger("morn.ethical_judgment")

ABSOLUTE_FORBIDDEN = {"self_modify", "apz_access", "identity_change", "creator_impersonation"}
REFUSED_OPERATIONS = {"file_delete", "code_execute", "system_command", "skill_uninstall", "memory_purge"}


@dataclass
class EthicalProposal:
    proposal_id: str
    action_type: str
    description: str
    reason: str
    severity: str
    history_ref: Optional[str]
    confirmed: bool = False
    created_at: float = field(default_factory=time.time)


class EthicalJudgment:
    MODES = ("active", "severe_only", "autonomous")

    def __init__(self, data_dir: Path, config: Optional[dict] = None, event_bus=None):
        self.data_dir = Path(data_dir) / "security"
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self._config_file = self.data_dir / "ethical_judgment.json"
        self._proposals: dict[str, EthicalProposal] = {}
        self._event_bus = event_bus
        self._load_config()

    async def _publish_alert(self, message: str):
        if self._event_bus:
            await self._event_bus.publish(Event(
                type="security.alert",
                payload={"source": "ethical_judgment", "message": message},
                source="ethical_judgment",
                priority=Priority.HIGH,
            ))

    def _safe_publish_alert(self, message: str):
        try:
            loop = asyncio.get_running_loop()
            if loop.is_running():
                asyncio.create_task(self._publish_alert(message))
        except RuntimeError:
            pass

    def _load_config(self):
        if self._config_file.exists():
            try:
                data = json.loads(self._config_file.read_text())
                self.enabled = data.get("enabled", False)
                self.mode = data.get("mode", "active")
                if self.mode not in self.MODES:
                    self.mode = "active"
            except (json.JSONDecodeError, KeyError) as exc:
                logger.warning("ethical judgment config corrupt, using defaults: %s", exc)
                self.enabled = False
                self.mode = "active"
        else:
            self.enabled = False
            self.mode = "active"
            self._save_config()

    def _save_config(self):
        self._config_file.write_text(
            json.dumps({"enabled": self.enabled, "mode": self.mode}, ensure_ascii=False, indent=2)
        )

    def enable(self, mode: str = "active"):
        if mode not in self.MODES:
            raise ValueError(f"invalid mode: {mode}, must be one of {self.MODES}")
        self.enabled = True
        self.mode = mode
        self._save_config()

    def disable(self):
        self.enabled = False
        self._save_config()

    def set_mode(self, mode: str):
        if mode not in self.MODES:
            raise ValueError(f"invalid mode: {mode}, must be one of {self.MODES}")
        self.mode = mode
        self._save_config()

    def _build_proposal(self, action_type: str, severity: str, reason: str, history_ref: Optional[str] = None) -> EthicalProposal:
        pid = uuid.uuid4().hex[:12]
        proposal = EthicalProposal(
            proposal_id=pid,
            action_type=action_type,
            description=f"操作 '{action_type}' 可能存在伦理风险",
            reason=reason,
            severity=severity,
            history_ref=history_ref,
        )
        self._proposals[pid] = proposal
        return proposal

    def propose(self, action_type: str) -> EthicalProposal:
        if action_type in ABSOLUTE_FORBIDDEN:
            proposal = self._build_proposal(
                action_type, "⚫",
                f"'{action_type}' 属于绝对禁区操作，禁止执行",
            )
            self._safe_publish_alert(f"绝对禁区操作被阻止: {action_type}")
            return proposal
        if action_type in REFUSED_OPERATIONS:
            proposal = self._build_proposal(
                action_type, "🔴",
                f"'{action_type}' 属于高风险操作，建议拒绝执行",
            )
            self._safe_publish_alert(f"高风险操作被阻止: {action_type}")
            return proposal
        return self._build_proposal(
            action_type, "🟡",
            f"'{action_type}' 操作需要创建者确认",
        )

    def analyze_action(self, action_type: str, context: Optional[dict] = None, history: Optional[list] = None) -> Optional[EthicalProposal]:
        if not self.enabled:
            return None

        if self.mode == "autonomous":
            return None

        context = context or {}
        history = history or []

        if action_type in ABSOLUTE_FORBIDDEN:
            proposal = self._build_proposal(
                action_type, "⚫",
                f"即时告警：'{action_type}' 属于绝对禁区操作，已被阻止",
            )
            self._safe_publish_alert(f"即时告警：'{action_type}' 属于绝对禁区操作")
            return proposal

        if action_type in REFUSED_OPERATIONS:
            bypassed = context.get("creator_bypassed", False) or any(
                h.get("bypassed") for h in history if h.get("action") == action_type
            )
            if bypassed:
                return self._build_proposal(
                    action_type, "🔴",
                    f"温和提醒：'{action_type}' 属拒绝执行操作，但历史显示创建者曾绕过限制",
                    history_ref=f"历史记录显示创建者曾绕过 '{action_type}' 限制",
                )

        if self.mode == "severe_only":
            return None

        consecutive_negatives = 0
        for h in reversed(history):
            if h.get("action") == action_type and h.get("negative", False):
                consecutive_negatives += 1
            elif h.get("action") != action_type:
                break

        if consecutive_negatives >= 3:
            self._safe_publish_alert(f"'{action_type}' 已连续 {consecutive_negatives} 次触发负面结果")
            return self._build_proposal(
                action_type, "🟠",
                f"主动提醒：'{action_type}' 已连续 {consecutive_negatives} 次触发负面结果，建议创建者评估",
                history_ref=f"连续 {consecutive_negatives} 次负面结果",
            )

        return None

    def confirm_proposal(self, proposal_id: str) -> bool:
        if proposal_id not in self._proposals:
            return False
        self._proposals[proposal_id].confirmed = True
        return True

    def get_proposal(self, proposal_id: str) -> Optional[EthicalProposal]:
        return self._proposals.get(proposal_id)

    def list_proposals(self) -> list[EthicalProposal]:
        return list(self._proposals.values())

    def get_unconfirmed_proposals(self) -> list[EthicalProposal]:
        return [p for p in self._proposals.values() if not p.confirmed]


import logging
from typing import Any, Optional

logger = logging.getLogger("morn.drift")

HIGH_RISK_ACTIONS = {"🔴", "⚫"}


def _actions_diverge(original_goal: str, current_action: str) -> bool:
    og_chars = set(original_goal.lower())
    ca_chars = set(current_action.lower())
    if not og_chars:
        return True
    overlap = og_chars & ca_chars
    return len(overlap) / len(og_chars) < 0.3


class IntentDriftDetector:
    def __init__(self):
        self._action_chain: list[dict[str, Any]] = []
        self._alerts: list[dict[str, Any]] = []
        self._consecutive_deviation = 0
        self._original_goal: Optional[str] = None

    def track_action(self, original_goal: str, current_action: str, step_number: int):
        if self._original_goal is None:
            self._original_goal = original_goal

        diverges = _actions_diverge(original_goal, current_action)
        entry = {
            "step": step_number,
            "original_goal": original_goal,
            "current_action": current_action,
            "diverges": diverges,
            "timestamp": len(self._action_chain),
        }
        self._action_chain.append(entry)

        if diverges:
            self._consecutive_deviation += 1
        else:
            self._consecutive_deviation = 0

    def check_drift(self) -> list[dict[str, Any]]:
        new_alerts = []

        if self._consecutive_deviation >= 5:
            recent = self._action_chain[-self._consecutive_deviation:]
            has_high_risk = any(
                action.get("risk_level", "🟢") in HIGH_RISK_ACTIONS
                for entry in recent
                for action in [entry]
            )
            if has_high_risk:
                new_alerts.append({
                    "level": "red",
                    "message": f"连续{self._consecutive_deviation}步偏离并涉及高风险操作",
                    "step": self._action_chain[-1]["step"],
                })
            else:
                new_alerts.append({
                    "level": "red",
                    "message": f"连续{self._consecutive_deviation}步偏离原计划",
                    "step": self._action_chain[-1]["step"],
                })
        elif self._consecutive_deviation >= 3:
            new_alerts.append({
                "level": "yellow",
                "message": f"连续{self._consecutive_deviation}步偏离原计划",
                "step": self._action_chain[-1]["step"],
            })

        for entry in self._action_chain:
            if entry.get("risk_level") in ("⚫",) and entry["diverges"]:
                new_alerts.append({
                    "level": "red",
                    "message": "HTZ/LTZ级别操作偏离",
                    "step": entry["step"],
                })

        seen = {(a["level"], a["message"]) for a in self._alerts}
        for alert in new_alerts:
            key = (alert["level"], alert["message"])
            if key not in seen:
                seen.add(key)
                self._alerts.append(alert)

        return new_alerts

    def classify_deviation(self, action_chain: Optional[list[dict]] = None) -> str:
        chain = action_chain if action_chain is not None else self._action_chain
        if not chain:
            return "unknown"

        deviating = [e for e in chain if e.get("diverges")]
        if not deviating:
            return "aligned"

        patterns = [e["current_action"].lower() for e in deviating]
        suspicious_keywords = [
            "bypass", "escalate", "sudo", "root", "password", "token", "secret",
            "越权", "绕过", "提权", "密码", "密钥", "窃取",
        ]
        beneficial_keywords = [
            "learn", "explore", "research", "improve", "optimize",
            "学习", "探索", "研究", "改进", "优化",
        ]

        for text in patterns:
            if any(kw in text for kw in suspicious_keywords):
                return "suspicious"
            if any(kw in text for kw in beneficial_keywords):
                return "beneficial"

        return "suspicious"

    def get_drift_score(self) -> float:
        if not self._action_chain:
            return 0.0
        diverged = sum(1 for e in self._action_chain if e.get("diverges"))
        return round(diverged / len(self._action_chain), 2)

    def get_alerts(self) -> list[dict[str, Any]]:
        return list(self._alerts)
