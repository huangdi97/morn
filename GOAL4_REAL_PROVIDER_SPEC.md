# Goal4 Real Provider Spec

EvolutionPlannerProvider：EvidenceWindow→provider proposal→schema parse→rule validation→policy validation→capability/authority validation→EvolutionCandidate。Provider proposal 永远不能直接 mutate production。

Real LLM：若已有可用 provider/secret，做真实 smoke 并记录 provider/model/config/version；secret 走现有 secret store/env，禁止写进仓库/log。无 secret 则 contract + deterministic test provider 全绿，并保留真实 smoke blocker。

DeepSeek Harness：沿用 B-001；目标只做 reconnect/health/scoped work/normalized events/Action Gateway mediation/teardown/provenance。缺官方安装物/凭据就保留 Active，不 fake success。
