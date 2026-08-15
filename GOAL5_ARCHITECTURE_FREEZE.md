# GOAL5_ARCHITECTURE_FREEZE.md

## 十条不可违反的 Core Laws
1. Work-first：Agent 永远不是首要业务对象。
2. Domain-neutral：Core 不依赖行业 ontology。
3. Stable Kernel：Provider/Runtime/Domain 不能重定义 canonical identity/work/artifact/outcome。
4. Governed Side Effects：外部/不可逆动作必须经过 Action Gateway / Policy / Approval / Verify。
5. Provider-neutral：Codex/Claude/DeepSeek/Python/Solver/Human/Device 都是可替换 provider/capability。
6. Production/Evolution Separation：Evolution 只提案，生产变更需 evaluation/certification/promotion。
7. Historical Truth：upgrade/uninstall/rollback 不删除历史 provenance。
8. Minimum Sufficient Intelligence：确定性方法足够时优先 Rule/Program/Solver。
9. No Instance Backdependency：Core 不 import/reference concrete domain pack。
10. Explicit Failure：blocked/degraded/partial 必须显式建模，禁止 fake green。

## 自动架构守卫
Codex 必须新增自动测试/lint，使以下行为 CI 失败：
- core crate imports domain pack
- core schema depends on concrete domain enum
- core UI directly imports reference-domain module
- provider bypasses permission/action gateway
- plugin/connector direct writes canonical DB tables outside public application/repository boundary

## 建议层次
按真实仓库适配，不强制改名：
`kernel → world/artifact/work/organization → capability/harness/runtime/integration/node → foundry/assurance/evolution → package/domain-sdk/plugin-sdk → app → domain packs`。
