# GOAL5_SECURITY_CHAOS.md

## Security Matrix
- cross-workspace ID/search/artifact leakage denied
- secrets never serialized to log/audit/UI; errors redact
- plugin only gets authorized secret handles/values
- E3 approval enforced
- unauthorized connector write denied
- duplicate external request idempotent
- path traversal/malformed manifest/incompatible version rejected
- stale/invalid node identity/lease rejected
- safe API validation/no raw secret stack leak

## Chaos Matrix
Inject：crash before commit；crash after external action before receipt；DB busy；duplicate signal/event；reordered event；provider timeout/malformed output；connector timeout；node loss；stale lease；checkpoint mismatch；migration failure；plugin init failure；duplicate action。

每个 failure 必须有显式状态和 audit evidence。
