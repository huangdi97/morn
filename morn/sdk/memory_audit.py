import difflib
import json
import logging
import re

logger = logging.getLogger("morn.audit")

_PATTERN_TRIPLES = [
    (re.compile(r"([^，。；]+?)是([^，。；]+)"), "是"),
    (re.compile(r"([^，。；]+?)喜欢([^，。；]+)"), "喜欢"),
    (re.compile(r"([^，。；]+?)拥有([^，。；]+)"), "拥有"),
    (re.compile(r"([^，。；]+?)想要([^，。；]+)"), "想要"),
    (re.compile(r"([^，。；]+?)在([^，。；]+)"), "在"),
    (re.compile(r"([^，。；]+?)会([^，。；]+)"), "会"),
    (re.compile(r"([^，。；]+?)说([^，。；]+)"), "说"),
    (re.compile(r"([^，。；]+?)提到([^，。；]+)"), "提到"),
    (re.compile(r"([^，。；]+?)擅长([^，。；]+)"), "擅长"),
    (re.compile(r"([^，。；]+?)知道([^，。；]+)"), "知道"),
]

_AUDIT_KEYWORDS_PASS = {
    "是", "喜欢", "拥有", "想要", "在", "会", "说", "提到", "擅长", "知道",
    "研究", "正在", "今天", "昨天", "明天",
}

_LLM_AUDIT_PROMPT = """审计以下知识三元组是否被源文本充分支持。

源文本：
{source_text}

三元组：
主体: {subject}
关系: {relation}
客体: {object}

判断规则：
- "pass": 证据充分，关系合理
- "fail": 主体或客体在源文本中找不到（幻觉），关系明显错误
- "uncertain": 有一定依据但不完全确定

严格要求：仅基于源文本判断，不要使用外部知识。

只输出 JSON：{{"verdict": "pass|fail|uncertain", "reason": "简短理由"}}"""

_LLM_EXTRACT_PROMPT = """从下面这段对话描述中提取知识三元组（subject-relation-object）。

要求：
- 每个三元组的 subject 和 object 必须是原文中明确提到的名词
- relation 用短谓语：喜欢, 擅长, 在, 有, 是, 想要, 住在, 使用, 玩, 吃, 读, 做, 去, 觉得, 希望, 可以, 知道, 会
- confidence 0-1，反映信息可靠程度
- evidence 是从原文中截取的最相关短句作为证据
- 如果没有任何可提取的知识，返回空数组 []

只输出 JSON 数组，不要其他文字。

对话描述：
{description}"""


class AuditAgent:
    def __init__(self, memory_store, llm_caller=None, config=None):
        self.store = memory_store
        self.llm_caller = llm_caller
        self._extracted_events = set()
        config = config or {}
        self.llm_audit_enabled = config.get("llm_audit_enabled", True)
        self.llm_extract_enabled = config.get("llm_extract_enabled", True)
        self._unhealable: list = []
        self._heal_enabled: bool = config.get("heal_enabled", True)

    async def _llm_audit(self, triple: dict, source_text: str) -> tuple:
        prompt = _LLM_AUDIT_PROMPT.format(
            source_text=source_text,
            subject=triple.get("subject", ""),
            relation=triple.get("relation", ""),
            object=triple.get("object", ""),
        )
        try:
            text = await self.llm_caller([{"role": "user", "content": prompt}])
            if not text or not text.strip():
                return ("uncertain", "LLM returned empty response")
            json_match = text.strip()
            if "{" not in json_match:
                return ("uncertain", "LLM returned non-JSON response")
            json_str = json_match[json_match.index("{"):json_match.rindex("}")+1]
            data = json.loads(json_str)
            verdict = data.get("verdict", "uncertain")
            reason = data.get("reason", "")
            if verdict not in ("pass", "fail", "uncertain"):
                verdict = "uncertain"
            return (verdict, reason)
        except Exception as e:
            logger.warning("LLM audit call failed: %s", e)
            raise

    async def _llm_extract(self, description: str) -> list:
        if not description:
            return []
        prompt = _LLM_EXTRACT_PROMPT.format(description=description)
        try:
            text = await self.llm_caller([{"role": "user", "content": prompt}])
            if not text or not text.strip():
                return []
            json_match = text.strip()
            if "[" not in json_match:
                return []
            json_str = json_match[json_match.index("["):json_match.rindex("]")+1]
            triples = json.loads(json_str)
            if not isinstance(triples, list):
                return []
            for t in triples:
                if not all(k in t for k in ("subject", "relation", "object", "confidence", "evidence")):
                    return []
            return triples
        except Exception as e:
            logger.warning("LLM extract call failed: %s", e)
            raise

    async def extract_triples(self, event_capsule: dict) -> list[dict]:
        event_id = event_capsule.get("event_id")
        if event_id and event_id in self._extracted_events:
            return []
        description = event_capsule.get("description", "")
        if not description:
            return []

        if self.llm_extract_enabled and self.llm_caller:
            try:
                llm_triples = await self._llm_extract(description)
                if llm_triples:
                    for t in llm_triples:
                        t["source_event_id"] = event_id
                        t["source_text"] = description
                    if event_id:
                        self._extracted_events.add(event_id)
                    return llm_triples
            except Exception:
                logger.warning("LLM extract failed, falling back to regex extraction")

        triples = []
        seen = set()
        for pattern, relation in _PATTERN_TRIPLES:
            for match in pattern.finditer(description):
                subject = match.group(1).strip()
                obj = match.group(2).strip()
                if not subject or not obj:
                    continue
                key = (subject, relation, obj)
                if key in seen:
                    continue
                seen.add(key)
                triples.append({
                    "subject": subject,
                    "relation": relation,
                    "object": obj,
                    "confidence": 0.7,
                    "evidence": match.group(0),
                    "source_event_id": event_id,
                    "source_text": description,
                })

        if event_id:
            self._extracted_events.add(event_id)
        return triples

    async def audit(self, triple: dict) -> dict:
        source_text = triple.get("source_text", triple.get("evidence", ""))

        if self.llm_audit_enabled and self.llm_caller:
            try:
                verdict, reason = await self._llm_audit(triple, source_text)
                return {"verdict": verdict, "reason": reason, "triple": triple}
            except Exception:
                logger.warning("LLM audit failed, falling back to rule engine")

        evidence = triple.get("evidence", "")
        subject = triple.get("subject", "")
        relation = triple.get("relation", "")
        obj = triple.get("object", "")

        if not source_text or not evidence:
            return {"verdict": "fail", "reason": "缺少源文本或证据", "triple": triple}

        if relation in _AUDIT_KEYWORDS_PASS:
            return {"verdict": "pass", "reason": "关系匹配已知模式", "triple": triple}

        if subject in source_text and obj in source_text:
            return {"verdict": "pass", "reason": "主体和客体均在源文本中找到", "triple": triple}

        if subject in source_text or obj in source_text:
            return {"verdict": "uncertain", "reason": "部分匹配源文本", "triple": triple}

        return {"verdict": "fail", "reason": "主体和客体均未在源文本中找到", "triple": triple}

    async def extract_and_deposit(self, event_capsule: dict) -> int:
        triples = await self.extract_triples(event_capsule)
        count = 0
        for triple in triples:
            result = await self.audit(triple)
            verdict = result.get("verdict", "fail")
            if verdict == "pass":
                await self.store.add_knowledge(
                    subject=triple["subject"],
                    relation=triple["relation"],
                    object=triple["object"],
                    confidence=triple.get("confidence", 0.5),
                    source_event_id=triple.get("source_event_id"),
                    source="audit_agent",
                )
                count += 1
            elif verdict == "uncertain":
                triple["pending_review"] = True
                await self.store.add_knowledge(
                    subject=triple["subject"],
                    relation=triple["relation"],
                    object=triple["object"],
                    confidence=triple.get("confidence", 0.5) * 0.8,
                    source_event_id=triple.get("source_event_id"),
                    source="audit_agent",
                    verified=False,
                )
                count += 1
            else:
                logger.info("audit rejected triple: subject=%s relation=%s object=%s reason=%s",
                            triple.get("subject"), triple.get("relation"),
                            triple.get("object"), result.get("reason"))
        return count

    async def self_heal_scan(self) -> int:
        if not self._heal_enabled:
            return 0
        healed = 0
        self._unhealable.clear()
        from datetime import datetime, timezone, timedelta
        cutoff = datetime.now(timezone.utc) - timedelta(days=30)
        cutoff_str = cutoff.strftime("%Y-%m-%dT%H:%M:%S")
        try:
            cursor = await self.store.db.execute(
                "SELECT s.* FROM raw_snapshots s WHERE s.timestamp >= ? ORDER BY s.timestamp DESC",
                (cutoff_str,)
            )
            rows = await cursor.fetchall()
        except Exception as e:
            logger.warning("self_heal_scan: failed to query raw_snapshots: %s", e)
            return 0
        for row in rows:
            raw_text = row["raw_text"]
            if not raw_text or not raw_text.strip():
                snap_id = row["snapshot_id"]
                self._unhealable.append(snap_id)
                continue
            snapshot_id = row["snapshot_id"]
            capsule = None
            try:
                capsule = await self.store.raw_snapshot_store.get_snapshot_by_capsule(snapshot_id)
                if not capsule:
                    cap_cursor = await self.store.db.execute(
                        "SELECT * FROM capsules WHERE event_id = ?", (snapshot_id,)
                    )
                    cap_row = await cap_cursor.fetchone()
                    if cap_row:
                        capsule = cap_row
            except Exception:
                pass
            if not capsule:
                try:
                    raw_meta = row["metadata"]
                    meta = json.loads(raw_meta) if raw_meta else {}
                    capsule_id = meta.get("capsule_id", "")
                    if capsule_id:
                        cap_cursor = await self.store.db.execute(
                            "SELECT * FROM capsules WHERE event_id = ?", (capsule_id,)
                        )
                        cap_row = await cap_cursor.fetchone()
                        if cap_row:
                            capsule = cap_row
                except Exception:
                    pass
            description = None
            if capsule:
                if isinstance(capsule, dict):
                    description = capsule.get("description", "")
                else:
                    try:
                        description = capsule["description"]
                    except Exception:
                        description = None
            if not description:
                continue
            ratio = difflib.SequenceMatcher(None, raw_text, description).ratio()
            if ratio >= 0.7:
                continue
            if self.llm_extract_enabled and self.llm_caller:
                try:
                    triples = await self._llm_extract(raw_text)
                    if triples:
                        new_desc_parts = []
                        for t in triples:
                            subj = t.get("subject", "")
                            rel = t.get("relation", "")
                            obj = t.get("object", "")
                            if subj and rel and obj:
                                new_desc_parts.append(f"{subj}{rel}{obj}")
                        if new_desc_parts:
                            new_description = "，".join(new_desc_parts)
                            new_description += f" [已修正] 旧描述: {description[:50]}"
                            try:
                                if isinstance(capsule, dict):
                                    eid = capsule.get("event_id", "")
                                else:
                                    try:
                                        eid = capsule["event_id"]
                                    except Exception:
                                        eid = ""
                                if eid:
                                    await self.store.db.execute(
                                        "UPDATE capsules SET description = ? WHERE event_id = ?",
                                        (new_description, eid)
                                    )
                                    await self.store.db.commit()
                                    healed += 1
                            except Exception as e:
                                logger.warning("self_heal_scan: failed to update capsule: %s", e)
                except Exception as e:
                    logger.warning("self_heal_scan: LLM extract failed: %s", e)
        return healed

    async def get_unhealable(self) -> list:
        return list(self._unhealable)

    def reset_extraction_tracking(self):
        self._extracted_events.clear()
