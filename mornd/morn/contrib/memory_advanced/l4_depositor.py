import logging

_L4_CONFIDENCE_THRESHOLD = 0.75
_L4_MIN_SOURCES = 3
_L4_MIN_VERIFIED = 2

_logger = logging.getLogger("morn.l4_depositor")


async def check_and_deposit(memory_store) -> int:
    try:
        all_triples = await memory_store.query_knowledge()
        if not all_triples:
            return 0

        groups = {}
        for t in all_triples:
            key = (t["subject"], t["relation"], t["object"])
            if key not in groups:
                groups[key] = []
            groups[key].append(t)

        existing_beliefs = await memory_store.query_personality(category="belief", limit=1000)
        existing_contents = {b["content"] for b in existing_beliefs}

        count = 0
        for (subject, relation, object), rows in groups.items():
            max_conf = max(r["confidence"] for r in rows)
            if max_conf < _L4_CONFIDENCE_THRESHOLD:
                continue

            source_ids = set()
            verified_count = 0
            skip_ltz = False
            for r in rows:
                if r.get("source_event_id"):
                    source_ids.add(r["source_event_id"])
                    capsule = await memory_store.get_capsule(r["source_event_id"])
                    if capsule and capsule.get("trust_level") == "ltz":
                        skip_ltz = True
                        break
                if r.get("verified_at") is not None:
                    verified_count += 1

            if skip_ltz:
                continue
            if len(source_ids) < _L4_MIN_SOURCES:
                continue
            if verified_count < _L4_MIN_VERIFIED:
                continue

            content = f"{subject} {relation} {object}"
            if content in existing_contents:
                continue

            await memory_store.add_personality("belief", content, max_conf)
            count += 1

        return count
    except Exception as e:
        _logger.warning(f"l4 deposit check failed: {e}")
        return 0