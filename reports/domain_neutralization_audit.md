# Domain Neutralization Audit

Date: 2026-08-15

## Dependency graph（Core → domain）
- `morn-app` (Core app) → `morn-biolab`（违反 Core→Domain 禁令；第一优先修复）
- `morn-biolab` → 仅 public SDK crates（kernel/world/artifact/work/org/actor/harness/runtime/capability）—— 已满足“pack 只依赖 public Morn SDK”

## Domain term scan（Core crates）
| Term | Location | Classification |
|---|---|---|
| biolab | kernel/policy.rs:124（测试 policy 名） | TEST_FIXTURE |
| biolab | world/service.rs:300,338（测试对象类型名） | TEST_FIXTURE |
| DatasetTag/SampleTag/AnalysisRunTag/QCResultTag/ScientificClaimTag | kernel/ids.rs | CORE_BUG（kernel 拥有具体领域 ID，需移出） |
| scientific (ScientificClaimId) | kernel/ids.rs:226,368 | CORE_BUG |
| hypothesis/claim/biolab | foundry/compiler.rs decompose_nodes | CORE_BUG（compiler 内置 biolab 工作模板启发式，需领域无关化） |
| experiment | capability/effect.rs（"physical experiment" 示例字符串） | SAFE_GENERIC（不可逆动作示例） |
| sample (SampleId) | kernel/ids.rs | CORE_BUG |
| qc_status | world/service.rs（测试） | TEST_FIXTURE |

## Plan
1. kernel：移除 biolab 领域 ID（Dataset/Sample/AnalysisRun/QCResult/ScientificClaim）→ 迁到 biolab-reference pack。
2. foundry：compiler 去 biolab 启发式 → 中性分解 + WorkTemplateRegistry（领域 pack 注入）。
3. app：移除对 biolab 的直接依赖；领域功能经 DomainRegistry（M11/M12）注入。
4. biolab：迁移到 domain-packs/biolab-reference（crate 名 morn-biolab-reference）。
5. 增加架构守卫测试（Core 不得依赖 domain pack）。


---

## Resolution (2026-08-16)

- CORE_BUG（kernel 领域 ID）→ 已移除，迁至 domain-packs/biolab-reference（morn-biolab-reference）。
- CORE_BUG（compiler 领域启发式）→ 已移除（DECISIONS D-027）：governed-deliverable 模板由通用信号触发；
  `check_domain_boundary.ps1` 新增领域词启发式守卫，run_all 通过。
- TEST_FIXTURE（各 crate 测试中 biolab 字符串）→ 保留为 conformance/reference fixture，不构成 Core→domain 依赖。
- 依赖方向：Core crates 无（非 optional）domain pack 依赖；morn-biolab-reference 仅依赖 public SDK crates。
- 证明：
  - `cargo check -p morn-app`（零领域）通过；
  - zero-domain server：/api/workbench domain_packs=[]，/api/biolab/run 404；
  - all-features server：domain_packs=[biolab-reference]，BioLab E2E all_ok=True；
  - `ArchitectureConformance::core_has_no_domain_pack` + 领域词守卫通过。
