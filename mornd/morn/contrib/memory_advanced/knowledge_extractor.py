import json
import logging

logger = logging.getLogger("morn.knowledge")

EXTRACT_PROMPT = """从下面这段对话描述中提取知识三元组（subject-relation-object）。

要求：
- 每个三元组的 subject 和 object 必须是原文中明确提到的名词
- relation 用短谓语：喜欢, 擅长, 在, 有, 是, 想要, 住在, 使用, 玩, 吃, 读, 做, 去, 觉得, 希望, 可以, 知道, 会
- confidence 0-1，反映信息可靠程度
- evidence 是从原文中截取的最相关短句作为证据
- 如果没有任何可提取的知识，返回空数组 []

只输出 JSON 数组，不要其他文字。

对话描述：
{description}"""

AUDIT_PROMPT = """审计以下知识三元组是否被源文本充分支持。

源文本：
{source_text}

三元组：
主体: {subject}
关系: {relation}
客体: {object}
置信度: {confidence}
证据: {evidence}

判断规则：
- "pass": 证据充分，关系合理，置信度恰当
- "reject": 主体或客体在源文本中找不到（幻觉），关系明显错误，或置信度虚高
- "unsure": 有一定依据但不完全确定

严格要求：仅基于源文本判断，不要使用外部知识。

只输出 JSON：{{"verdict": "pass|reject|unsure", "reason": "简短理由"}}"""


async def extract_knowledge(capsule: dict, llm_caller: callable) -> list[dict]:
    description = capsule.get("description", "")
    if not description:
        return []
    prompt = EXTRACT_PROMPT.format(description=description)
    try:
        text = await llm_caller([{"role": "user", "content": prompt}])
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
        logger.warning("extract_knowledge failed: %s", e)
        return []


async def audit_triple(triple: dict, source_text: str, llm_caller: callable) -> tuple[str, str]:
    subject = triple.get("subject", "")
    relation = triple.get("relation", "")
    obj = triple.get("object", "")
    confidence = triple.get("confidence", 0.5)
    evidence = triple.get("evidence", "")
    prompt = AUDIT_PROMPT.format(
        source_text=source_text,
        subject=subject,
        relation=relation,
        object=obj,
        confidence=confidence,
        evidence=evidence,
    )
    try:
        text = await llm_caller([{"role": "user", "content": prompt}])
        if not text or not text.strip():
            return ("unsure", "audit LLM returned empty response")
        json_match = text.strip()
        if "{" not in json_match:
            return ("unsure", "audit LLM returned non-JSON response")
        json_str = json_match[json_match.index("{"):json_match.rindex("}")+1]
        data = json.loads(json_str)
        verdict = data.get("verdict", "unsure")
        reason = data.get("reason", "")
        if verdict not in ("pass", "reject", "unsure"):
            verdict = "unsure"
        return (verdict, reason)
    except Exception as e:
        logger.warning("audit_triple failed: %s", e)
        return ("unsure", "audit LLM call failed")


async def auto_extract(memory_store, capsule: dict, llm_caller: callable) -> int:
    try:
        triples = extract_knowledge(capsule, llm_caller)
        count = 0
        for triple in triples:
            verdict, reason = audit_triple(triple, capsule.get("description", ""), llm_caller)
            subject = triple.get("subject", "")
            relation = triple.get("relation", "")
            obj = triple.get("object", "")
            confidence = triple.get("confidence", 0.5)
            if verdict == "pass":
                await memory_store.add_knowledge(
                    subject, relation, obj, confidence,
                    source_event_id=capsule.get("event_id"),
                )
                count += 1
            elif verdict == "reject":
                logger.info("audit rejected triple: %s — reason: %s", triple, reason)
            else:
                await memory_store.add_knowledge(
                    subject, relation, obj, confidence * 0.8,
                    source_event_id=capsule.get("event_id"), verified=False,
                )
                count += 1
        return count
    except Exception as e:
        logger.warning("auto_extract failed: %s", e)
        return 0