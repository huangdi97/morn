# Morn v11.5-R1 全量连续执行 Goal

**用途：**直接交给具备代码仓库读写、终端、测试、浏览器/文档能力的 Coding Agent / WorkBuddy / OpenCode / Codex 等执行。  
**目标：**在不伪造外部条件、不越过真实生产权限、不推倒已有有效工程资产的前提下，把《Morn v11.5-R1｜协议驱动工作控制平面 + Cordis 参考组合运行时 + Capability Supply Chain + Domain Profiles｜统一全量母版》所定义的**当前可工程化设计**持续落实到代码、测试、文档、UI、发布与验收证据中，直到所有本地可完成 Gate 通过，剩余项只允许是被真实外部条件阻塞的项目。

> 本 Goal 不是“做一个 Demo”。它要求形成可持续开发、可回放、可验证、可替换 Provider、可继续接入真实客户的 Morn Reference Implementation。  
> 不允许为了“跑通”降低 Gate、删除测试、静默跳过失败、伪造真实数据/客户/外部回执、把 mock 成功写成真实成功。

---

# 0. 唯一 Canonical 与优先级

执行前先定位并完整阅读下列文件，不得只读摘要：

1. **最高优先级设计规范：**  
   `Morn_v11.5-R1_协议驱动控制平面_Cordis参考运行时_CapabilitySupplyChain_统一全量母版_2026-09-24.md`
2. 若仓库中仍存在 v11.4 / v11.3 / 2026-08-31 母版：
   - v11.4/v11.3 仅用于上游设计追溯；
   - 2026-08-31 文档仅用于历史工程事实、RV/TF01/LC01/BIOFERM、Brownfield 证据和旧接口追溯；
   - **不得因为旧文档“没有冲突”就静默继承为现行设计。**
3. **仓库真实代码、测试、Git 历史、当前构建状态是工程事实源。** 文档不能把未实现能力写成已完成；代码事实与设计目标冲突时，先记录 Reality Gap，再按本 Goal 迁移。

如果仓库里没有 v11.5 文件，但本 Goal 与 v11.5 文件一起提供，则把 v11.5 复制到仓库合适的 canonical/docs 目录，并在 `CURRENT`/README 中明确其状态；不得删除旧母版。

---

# 1. 总体执行原则

## 1.1 不要阶段性停下来问我“是否继续”

从开始执行后，按：

```text
Reality Audit
→ Architecture Map
→ Migration Plan
→ Implement
→ Test
→ Evidence
→ Commit
→ Next Phase
```

持续运行。

遇到单个问题时：定位根因 → 修复 → 继续；不要因为一个非关键失败就结束整个 Goal。

只有以下情况可以暂停等待人类：

- 需要真实生产系统写权限、真实客户凭据、付费云资源或秘密密钥；
- 会删除不可恢复的数据、重写远端 Git 历史、force push、破坏客户系统；
- 法律/合规/生产安全责任必须由人类作决定；
- canonical 文件存在无法通过代码/历史判定的根本冲突，且两条路线会造成不可逆架构分叉。

除此之外，采用**最小风险、最小改动、最符合 canonical** 的方案，记录 ADR/decision log 后继续。

## 1.2 External Blocker 不得阻塞整个工程

若客户数据、真实 MES/CMMS、真实云账号等不可得：

```text
BLOCKED_EXTERNAL
```

并明确缺什么、谁才能解除、解除后的验收步骤；然后继续执行所有不依赖该外部条件的任务。

最终允许状态：

```text
ENGINEERING_COMPLETE_WITH_EXTERNAL_BLOCKERS
```

但不允许把 external blocker 伪装成 PASS。

## 1.3 禁止“为了完成而完成”

禁止：

- 降低已有测试阈值；
- 删除/skip/xpass 真实失败用例来制造绿色；
- 把 mock/fake/synthetic 数据写成 real customer evidence；
- 把 HTTP 200、Agent 自述、工具返回文本当成 ObservedOutcome；
- 将 Cordis Fiber 生命周期等同于 Work 生命周期；
- 将 DSH/Pi Session 作为 Morn canonical business state；
- 让 Agent 直接持有生产凭据绕过 Action Gateway；
- 因新架构漂亮而推倒已经通过验证的旧 Core；
- 直接 fork Cordis/DSH，除非先给出 upstream 不可满足需求的最小证据；
- 创建第二套与 Six Records 重复的事实账本；
- 无版本迁移地改变已发布 contract/profile；
- Provider hot-swap 偷偷改变 active Attempt 的执行语义。

---

# 2. 最终必须实现的目标架构

最终工程应收敛为：

```text
Morn
=
Semantic Specification
+ Durable Work Control Plane
+ Cordis Reference Composition Runtime
+ Provider Fabric
+ Trust/Governance Plane
+ Capability Supply Chain
+ Domain Profiles
```

六 Plane：

```text
1. Semantic / Specification Plane
2. Durable Control Plane
3. Composition Plane (Cordis reference runtime)
4. Execution Plane
5. Trust & Governance Plane
6. Capability Supply Chain
```

Domain Products（Factory / Research / BioLab / Work / Personal 等）建立在以上 Plane 之上，不得形成互不兼容的独立平台。

最高层原则必须在代码、测试和文档中都可观察：

```text
Work is the unit of coordination.
Capability is the unit of composition.
Accepted Outcome is the unit of value.
Agent is one kind of executor.
Everything may evolve; nothing may silently rewrite a committed fact or contract.
```

---

# 3. 目标仓库结构（允许结合现仓实际做等价映射）

不要为了目录漂亮强制大迁移；先映射现状。如果现有结构不冲突，可逐步收敛为：

```text
morn/
  packages/
    spec/                       # language-neutral semantic contracts / schemas
    contracts/                  # protocol interfaces / provider interfaces
    control-plane/              # durable work state + API + projections
    controllers/                # reconciliation controllers
    profiles/                   # lite / enterprise / factory / research ...
    capability/                 # manifest, qualification, resolver, package metadata
    events/                     # CloudEvents mapping / event contracts
    telemetry/                  # OTel semantic conventions/helpers
    trust/                      # authority request/decision, gateway contracts

  runtime/
    cordis/                     # ONLY place directly coupled to Cordis API where practical

  providers/
    harness-dsh/
    harness-pi/
    policy-opa-or-reference/
    solver-ortools/
    durable-local/
    sandbox-local/
    connector-fake-mes/
    connector-fake-cmms/

  domains/
    factory/
      profile/
      work-packages/
      adapters/

  studio/
    artifact2capability/
    blueprint/
    qualification/

  apps/
    workbench/
    console/
    studio/

  conformance/
    protocol/
    profile/
    provider/

  docs/
    canonical/
    adr/
    architecture/
    evidence/
    reports/
```

**约束：** `packages/spec` / domain semantic code 不应到处直接 import Cordis、DSH、Pi、Temporal 等实现类型。实现耦合应收敛在 adapter/provider 层。

---

# 4. Phase 0 — Reality Audit：先证明现在是什么，不要先改

## 4.1 Git / workspace reality

收集并落盘：

- repo root / branch / HEAD / origin；
- tracked/untracked/modified；
- 最近 30~100 个相关 commit；
- package/workspace 结构；
- 当前语言/运行时/框架版本；
- CI 配置；
- lockfiles；
- migrations；
- docs canonical 指针；
- Core / Factory / Agent / runtime / plugin / workflow / policy / DB / UI 的实际位置。

输出：

`docs/reports/V11_5_REALITY_AUDIT.md`

必须区分：

```text
EXISTS_AND_TESTED
EXISTS_PARTIAL
DOC_ONLY
MISSING
LEGACY
EXTERNAL_BLOCKED
UNKNOWN_NEEDS_PROOF
```

## 4.2 全量基线测试

自动发现并运行仓库已有：

- unit tests；
- integration tests；
- lint/format；
- typecheck；
- build；
- UI build；
- e2e/smoke；
- migrations/schema checks；
- security/dependency audit（只执行仓库现有或无需付费的）；
- any existing Morn gates。

记录真实结果，不要先修再写成“baseline all green”。

输出：

`docs/reports/V11_5_BASELINE_TEST_REPORT.md`

## 4.3 v11.3/v11.4/v11.5 → Current Code 映射

必须做对象级矩阵：

```text
canonical concept
existing code
existing tests
current maturity
v11.5 target
migration action
risk
```

至少覆盖：

- Six Records；
- World/Work/Decision/Execution/Outcome/Workforce；
- Authority/Delegation；
- Attempt/Receipt/Reconcile；
- Capability/Skill/Harness；
- Workcell；
- Studio/SolutionPackage；
- Factory；
- TF/RV/LC/BIOFERM 历史接口；
- UI/Workbench/Console；
- persistence/event state。

输出：

`docs/reports/V11_5_CODE_MAPPING_AND_MIGRATION_MATRIX.md`

### G0 Reality Gate

只有完成 Reality Audit、基线测试、代码映射后才允许进入大规模实现。

---

# 5. Phase 1 — Semantic Specification：先稳定语义，不先绑定运行时

建立/整理 language-neutral schema/contracts。优先 JSON Schema / OpenAPI-compatible schema / typed IDL；具体语言类型从 schema 生成或映射。

至少定义：

```text
ObjectRef
Observation
Situation
Work
WorkSpec
WorkStatus
Condition
Commitment
Principal
Actor
Role
AuthorityRequest
AuthorityDecision
Capability
CapabilityManifest
CapabilityRelease
QualificationRecord
SiteAdmission
ExecutionBinding
ActionAttempt
Receipt
ReconciliationRecord
ObservedOutcome
AcceptanceDecision
ValueAssessment
Profile
SolutionPackage
```

## 5.1 Work 必须是 spec/status

`Work.spec`：

- goal / desired state；
- target/scope；
- constraints；
- acceptance requirements；
- authority requirements；
- budget；
- profile ref；
- source refs；
- owner/commitment；
- version/generation。

`Work.status`：

- observedGeneration；
- phase；
- conditions；
- active bindings/attempts；
- unresolved unknowns；
- observed outcome refs；
- acceptance state；
- blocker refs；
- timestamps。

必须写 schema validation tests。

## 5.2 Conditions 是语义 Gate

至少提供条件模型，并能表达：

```text
CapabilityResolved
AuthoritySatisfied
EvidenceSufficient
ExecutionBound
ActionCommitted
ExternalActionVerified
OutcomeObserved
AcceptanceSatisfied
ProfileConformant
```

Profile 要求 Condition；Provider/Controller 负责产生 Condition。不要通过硬编码某个具体 Provider 达成。

## 5.3 历史语义

实现：

```text
current projection       mutable
released contract        versioned/content-addressed
historical fact          append/supersede/retract/redact
active execution binding pinned in scope
```

至少支持：

- `supersedes` / `retracts`；
- valid_time；
- recorded_at/transaction time；
- reason / actor / provenance；
- contract version reference。

### G1 Semantic Mapping Gate

- schema round-trip；
- old record compatibility/mapping tests；
- no duplicate canonical ledger；
- protocol types 不依赖 Cordis/DSH/Pi implementation types。

---

# 6. Phase 2 — Durable Control Plane Minimum

第一版优先利用现有 DB；若没有合理持久层，默认 PostgreSQL。不要因追求“未来分布式”先引入 Kafka/K8s/Temporal。

## 6.1 Minimum store

实现/映射：

```text
work
work_condition
semantic_event / fact
execution_binding
action_attempt
receipt
reconciliation
observed_outcome
acceptance
capability_release
qualification
profile_release
outbox
controller_lease/checkpoint
```

物理表可合并，但协议语义必须保持。

## 6.2 Event history + current projection

- append/supersede event/fact；
- deterministic projection；
- replay test；
- crash/restart 后 projection 一致；
- event idempotency；
- transaction/outbox；
- schema/version metadata。

CloudEvents 用作外部/跨组件 Event Envelope；内部不需要为追求标准强行把所有行存成 CloudEvents JSON。

## 6.3 Controllers

实现 reference controllers：

```text
WorkController
SituationController (minimum)
CapabilityResolverController
BindingController
AuthorityController
ExecutionController
ReceiptController
ReconciliationController
OutcomeController
AcceptanceController
QualificationController
ProfileConformanceController
```

允许部分为简单 deterministic worker，但必须：

- 幂等；
- 可重入；
- crash-safe；
- 不靠进程内 memory 保存 canonical progress；
- 有 lease/checkpoint；
- 输出 evidence/condition changes。

## 6.4 Unknown 是一等状态

必须实现并测试：

```text
OUTCOME_UNKNOWN
→ RECONCILING
→ VERIFIED / FAILED / STILL_UNKNOWN
```

不能把 timeout 自动当 failure 再 retry 外部 side effect。

### G2 Durable State Gate

至少证明：

1. 进程在 Attempt 后 kill；
2. 重启；
3. Work 仍存在；
4. Attempt/Binding 不丢；
5. Reconciliation 能继续；
6. 不重复执行已提交副作用。

---

# 7. Phase 3 — Cordis Reference Composition Runtime

## 7.1 先做 Cordis Spike，不先 fork

要求：

- 查明当前可用 upstream Cordis exact version/commit；
- exact pin；
- 保存版本/来源/许可证；
- 不使用宽泛 `latest`/无上限 semver；
- 先以 dependency + adapter 使用；只有出现可复现 upstream blocker 才考虑 vendor/fork。

## 7.2 Cordis 只负责 node-local composition

建立 `runtime/cordis` adapter。只有该层（及极少必要基础层）直接依赖 Cordis API。

映射：

```text
Morn service slot
↔ Cordis service
Morn runtime scope
↔ Cordis context/scope
provider lifecycle
↔ Fiber/effect lifecycle
```

但是明确禁止：

```text
Fiber == Work
Effect disposer == business rollback
service active == capability qualified
provider ready == site admitted
```

## 7.3 Service slots

至少定义/挂载：

```text
capabilities
harnesses
solvers
executionEnvironments
connectors
policy
identity
telemetry
durableExecution(optional)
```

Work/Receipt/Outcome 等 durable business truth 仍在 Control Plane。

## 7.4 Runtime lifecycle tests

测试：

- provider register/unregister；
- dependency unavailable/recover；
- scoped service；
- plugin cleanup；
- provider upgrade；
- active Attempt 不因 Cordis provider reload 被偷偷换 Binding。

### G3 Cordis Isolation Gate

通过代码扫描/依赖图证明：domain/spec/control-plane 没有大面积 Cordis coupling；Cordis 可被 adapter 隔离。

---

# 8. Phase 4 — Execution Plane：DSH + Pi + Solver + Human + Connector

## 8.1 DSH Provider

优先把 DeepSeek Harness 作为**独立进程 Harness Provider**接入，使用其公开/稳定的 SDK、JSON-RPC、ACP 或正式接口；不要依赖 DSH 内部私有 Cordis tree。

统一 `HarnessProvider` 最小合同：

```text
capabilities()
startSession(binding/context)
resumeSession(sessionRef)
submit(input)
cancel()
checkpoint()/resumeRef()
getEvents()
getUsage()
health()
close()
```

实际接口可调整，但业务层只能依赖 Morn contract。

DSH SessionEvent 可以保留为 Harness evidence，不得直接成为 Work truth。

## 8.2 Pi Provider

接入 Pi，目标不是“多支持一个模型”，而是证明：

```text
same Work contract
same capability intent
same business acceptance
```

可以换 Harness。

如果 Pi/DSH 暂时不可访问：

- 完成真实 adapter skeleton + contract tests；
- 使用 deterministic fake provider 验证控制平面；
- 标记 REAL_PROVIDER_INTEGRATION_BLOCKED；
- 不得声称 Harness swap 已真实 PASS。

## 8.3 Solver Provider

提供至少一个 deterministic solver provider（若现有项目已有，复用；否则 OR-Tools/简化 solver）。

验证 Workcell 不要求 Agent。

## 8.4 Human Provider

至少支持：

- assignment；
- approval；
- reject/request-more-evidence；
- acceptance；
- timeout/escalation。

## 8.5 Connector / Action Provider

第一阶段实现 Fake MES / Fake CMMS / Fake APS 或仓库已有 sandbox adapter。

所有 side-effect action 必须经过：

```text
Work
→ Action proposal
→ Action Gateway / PEP
→ AuthorityDecision
→ ExecutionBinding
→ Provider
→ Receipt/Reconciliation
```

Agent 不得直接调用“真实生产写”路径。

### G4 / G5 Harness Gates

- G4: DSH Provider contract + smoke；
- G5: 第二 Harness（Pi）contract + smoke；
- 无真实 Provider 时必须留 BLOCKED，不假装 pass。

---

# 9. Phase 5 — ExecutionBinding：安全绑定与 provider swap

实现 `ExecutionBinding` 一等资源。

至少包含：

```text
binding_id
work_ref
capability_release_digest
provider_id/version
runtime/harness ref
execution_environment ref
authority_decision ref
profile ref
site ref
created_at
binding_generation
```

规则：

1. Attempt 开始前必须有 Binding；
2. active Attempt pin 到 Binding；
3. Cordis hot reload 只影响 future resolution，不能静默改变 active Binding；
4. Provider 故障：Attempt → SUSPENDED/UNKNOWN；
5. 若换 Provider：创建新 Binding；需要时新 Attempt；
6. 对已发生 side effect 的 Attempt，不得“换 Provider 后从头重跑”；先 reconcile。

### G6 Binding Pin Gate

编写故障注入测试：执行中卸载 DSH / restart provider / load Pi，证明 active Attempt 不被偷偷迁移。

---

# 10. Phase 6 — Action / Receipt / Reconciliation

保留并强化状态：

```text
PROPOSED
AUTHORIZED
DISPATCHED
ACKNOWLEDGED
COMMITTED
OBSERVED
VERIFIED
```

失败/不确定支路：

```text
FAILED_BEFORE_COMMIT
OUTCOME_UNKNOWN
RECONCILING
COMPENSATING
COMPENSATED
MANUAL_INTERVENTION_REQUIRED
```

不是所有动作都必须支持全部状态，但语义不能混淆。

实现：

- business idempotency key；
- request hash；
- external business key；
- transport ack != business commit；
- receipt source/provenance；
- reconciliation query；
- duplicate prevention；
- compensation contract（仅适用时）。

至少有真实自动化测试：

```text
Fake CMMS POST
→ server committed
→ client timeout
→ Morn OUTCOME_UNKNOWN
→ reconcile by business key
→ find MO-xxxxx
→ VERIFIED
→ no duplicate order
```

### G7 Receipt/Reconciliation Gate

必须通过 crash + timeout + retry storm + duplicate message 测试。

---

# 11. Phase 7 — Outcome / Acceptance / Value

## 11.1 Outcome

`ObservedOutcome` 必须来自可识别 source：external system, sensor, human observation, validated computation 等。

禁止：

```text
Agent: “已经完成”
→ Outcome = success
```

## 11.2 Acceptance

`AcceptanceDecision` 至少包含：

```text
accept/reject/conditional
criteria_ref
outcome_refs
actor/principal
reason
time
```

支持多角色验收要求。

## 11.3 Value Assessment

可记录：

- total duration；
- human minutes；
- model/runtime cost；
- retries/unknown；
- baseline comparison；
- accepted/reopened；
- operational KPI。

不得把模拟 KPI 当客户真实商业价值。

---

# 12. Phase 8 — Trust & Governance Plane

## 12.1 Authority contract

Authority 必须表达：

```text
principal
acting_for
work/scope
site
resource
action
parameter envelope
time window
budget
preconditions
approval refs
delegation depth
revocation
```

## 12.2 Policy Decision / Enforcement 分离

定义 Provider-neutral：

```text
AuthorityRequest
AuthorityDecision
AuthorityEvidence
```

实现至少一个 reference provider；若已有 OPA/Cedar，可接入一个真实 provider。不要为 Morn 发明复杂 policy DSL。

## 12.3 PEP 强制性

对有副作用 Connector，写测试证明绕过 Action Gateway/PEP 不可达或不可被正常 capability 调用。

## 12.4 Secrets

- 不写入 git；
- capability 只获取工作范围/最小权限 credential；
- Harness 不天然持有生产 credential；
- logs 默认 redact secret；
- 提供 credential provider interface。

SPIFFE/IAM 等先进 identity 先保留 contract，若仓库已有条件再实现，不为完成 Goal 引入巨大基础设施。

---

# 13. Phase 9 — Execution Environment：DSec 抽象落地

实现 `ExecutionEnvironmentSpec`：

```text
isolation level
OS/runtime
CPU/memory/GPU
filesystem
network policy
secret bindings
persistence scope
timeout
side-effect policy
```

第一阶段 Provider 至少：

```text
local-process (dev only)
container (preferred default)
```

microVM/full VM/DSec-like/cloud sandbox 先定义 provider contracts 与 qualification fields；没有实际环境时标 `NOT_IMPLEMENTED / FUTURE_PROVIDER`，不要模拟成完成。

Capability admission 必须检查 execution requirements 与 provider guarantees 是否匹配。

---

# 14. Phase 10 — Capability Supply Chain

## 14.1 Capability Manifest

至少支持：

```text
identity/version/digest
kind
provides
input/output contract
preconditions/postconditions
side effects
idempotency/compensation
runtime/harness compatibility
execution requirements
authority ceiling
cost/latency envelope
qualification refs
site admission refs
provenance/license/owner
```

## 14.2 四种能力状态必须分离

```text
Declared
Observed
Qualified
Admitted
```

不得：

```text
registered == qualified == admitted
```

## 14.3 Lifecycle

兼容 canonical：

```text
DISCOVERED
EXTRACTED
WRAPPED
EXECUTABLE
VERIFIED
QUALIFIED
RELEASED
SITE_BOUND / ADMITTED
DEPRECATED / REVOKED
```

将 asset lifecycle 与 site admission 分开。

## 14.4 Qualification

至少实现：

- deterministic test suite refs；
- environment digest；
- input scope；
- expected outputs/properties；
- failure modes；
- cost/latency evidence；
- evaluator identity；
- qualification status/version。

### G8 Capability Qualification Gate

未通过 qualification 的 capability 不允许进入 Factory strict profile 的关键 binding。

---

# 15. Phase 11 — Capability Resolver / Workcell

Resolver 流程：

```text
Work requirements
→ hard constraints
→ eligible capabilities
→ qualification/site admission
→ authority compatibility
→ execution environment compatibility
→ cost/latency/risk scoring
→ minimum sufficient executor mix
→ bindings
```

硬约束与软评分必须分开。

LLM 可以辅助解析自然语言 requirement，但最终 eligibility 不能只靠 LLM 猜测。

至少测试：

- 0 Agent Workcell；
- 1 Agent + Rule；
- Agent + Solver + Human；
- capability 缺失；
- capability qualification 过期；
- site not admitted；
- authority ceiling 不足；
- budget 不足；
- provider unavailable fallback。

`Workcell` 是某个 Work 的动态执行组合，不是永久多 Agent 部门。

---

# 16. Phase 12 — Artifact2Capability MVP

不要一次实现所有 Compiler。先至少做 2 个真实可验证 compiler，再扩展。

优先级：

```text
1. OpenAPI2Capability
2. Repo2Capability
3. SOP2Procedure / Paper2Capability（二选一按仓库资产）
```

Pipeline：

```text
Artifact discovery
→ extraction
→ candidate manifest
→ wrapper/interface
→ environment
→ tests
→ evaluation
→ candidate qualification
→ package
```

失败必须有结构化结果：

```text
RESOURCE_ONLY
BLOCKED_MISSING_ARTIFACT
BLOCKED_ENVIRONMENT
WRAPPED_BUT_UNVERIFIED
NON_GENERALIZABLE
FAILED_TEST
```

绝不能：

```text
读文档 → LLM 生成一个 tool → RELEASED
```

保留 Paper2Agent 的核心原则：生成 executable wrapper 必须能追到真实 source artifact/code；evaluation 要区分 tool execution、result correctness、cost。

---

# 17. Phase 13 — Capability Package / Registry / Provenance

如果本地环境允许，优先按 OCI/ORAS 构造 package；若没有 registry，至少生成内容寻址的本地 package layout，并确保未来可映射 OCI。

Package 至少包含：

```text
capability manifest
schemas
runtime metadata
qualification evidence
tests
provenance
policy/authority envelope
docs
SBOM if build produces software artifact
```

实现 digest/content address。

Sigstore/SLSA：

- 能真实运行则接；
- 不能运行则建立 provider/metadata contract + local unsigned status；
- 不伪造 signature/provenance。

---

# 18. Phase 14 — Profiles：从插件列表变成 Guarantee Profile

实现 Profile schema + conformance engine。

至少：

```text
morn.lite
morn.enterprise
morn.factory.readonly
morn.research
```

其中 Factory Readonly 至少要求：

```text
durable_work_state = required
source_of_truth_binding = required
provenance = required
authority_before_external_side_effect = required
receipt_after_external_action = required
reconciliation_on_unknown = required
accepted_outcome_semantics = required
profile/version pinning = required
production_write = forbidden in initial profile
```

Profile 是 guarantee requirements，不是 provider 名单。

同一 Profile 应允许：

```text
DSH or Pi
OPA or Cedar/reference policy
local durable store or future Temporal
SAP or fake/custom connector
```

### G9 Profile Conformance Gate

写 conformance suite，给出机器可读报告。

---

# 19. Phase 15 — Morn Factory Thin Slice：首个真正产品楔子

仍然只做：

> **停机 / 插单后的异常—产能—交期影响评审。**

初始模式：

```text
READ
MAP
ANALYZE
CANDIDATE
EVIDENCE
EXPERT REVIEW
NO PRODUCTION WRITE
```

## 19.1 CNC-17 Reference Scenario

构建 deterministic reference scenario：

```text
SCADA/Alarm event
→ Situation
→ Work creation
→ Capability resolution
→ Equipment Investigator (DSH/Pi)
→ vibration evidence
→ scheduling solver
→ candidate plans
→ human decision
→ Fake CMMS action (sandbox only)
→ intentional timeout after commit
→ OUTCOME_UNKNOWN
→ reconciliation
→ Receipt verified
→ fake MES/QMS/APS observed states
→ ObservedOutcome
→ Acceptance
→ ValueAssessment
```

必须同时提供：

- deterministic fixture；
- replay；
- restart/resume；
- DSH→Pi provider failure scenario；
- binding pin evidence；
- no duplicate external action；
- audit timeline；
- UI walkthrough。

### G10 Factory Thin Slice Gate

整个 reference scenario 一键可跑，产生可验证 evidence bundle。

---

# 20. Phase 16 — Workbench / Console / Studio UI

如果仓库已有 UI 技术栈，延续现栈；不要为重写 UI 换框架。

## 20.1 Workbench

至少：

- Work list；
- Work detail；
- goal/constraints/acceptance；
- conditions；
- current situation/evidence；
- candidates；
- active workcell；
- authority status；
- attempts/receipts/reconciliation；
- outcome；
- accept/reject/request-evidence；
- timeline/audit；
- provider/runtime info 只作为次级信息，不让用户围绕 Agent Session 操作。

## 20.2 Console

至少：

- providers health；
- Cordis service slots；
- capabilities/releases；
- qualification；
- site admission；
- profiles/conformance；
- bindings/attempts；
- external blockers；
- telemetry links/metrics。

## 20.3 Studio

至少：

- “你希望完成什么工作？”入口；
- WorkBlueprint editor；
- RoleBlueprint editor；
- capability import/compiler flow；
- qualification run；
- SolutionPackage preview/release；
- 不以“创建 Agent”为唯一入口。

## 20.4 UX

- 中文默认；
- 错误/Unknown/Blocked 必须显式；
- 不用“AI 已完成”掩盖外部事实未验证；
- read-only Factory profile 上所有真实 write 明确禁用；
- 关键审批要展示对象、动作、范围、后果、证据。

UI 完成后跑真实浏览器 smoke/e2e，并保存截图到 `docs/evidence/ui/`。

---

# 21. Phase 17 — Event / Telemetry / Observability

优先复用 OpenTelemetry；定义 Morn semantic attrs：

```text
morn.work.id
morn.work.type
morn.profile.id
morn.binding.id
morn.attempt.id
morn.capability.id
morn.capability.digest
morn.provider.id
morn.authority.decision_id
morn.site.id
morn.outcome.id
```

关键 trace 至少覆盖：

```text
Work created
→ resolve
→ bind
→ authorize
→ dispatch
→ receipt/reconcile
→ outcome
→ acceptance
```

日志必须能通过 work_id 关联，但不要记录 secret/full sensitive payload。

---

# 22. Phase 18 — Security / Threat Model / Failure Engineering

产出 `docs/security/MORN_V11_5_THREAT_MODEL.md`。

至少覆盖：

- prompt/tool injection；
- malicious capability package；
- provider impersonation；
- credential leakage；
- action replay；
- duplicate write；
- stale authority；
- binding substitution；
- qualification spoofing；
- provenance tampering；
- external source conflict；
- unknown outcome；
- compromised Harness；
- sandbox escape assumptions；
- UI approval confusion；
- cross-site capability misuse；
- event reordering/duplication；
- clock/time semantics。

做 fault injection：

```text
kill process
provider crash
DB restart where feasible
network timeout
reordered event
duplicate event
slow provider
malformed receipt
expired authority
revoked capability
profile upgrade during work
```

所有结果留报告。

---

# 23. Phase 19 — Conformance / Compatibility / Provider Swap

建立真正的 conformance suite，而不是只做 unit test。

## 23.1 Protocol conformance

验证：

- versioning；
- Work spec/status；
- Condition gates；
- Binding pin；
- authority-before-side-effect；
- receipt semantics；
- outcome vs acceptance；
- supersede/retract；
- unknown/reconciliation。

## 23.2 Provider conformance

所有 HarnessProvider 使用同一 contract test。

## 23.3 Harness swap benchmark

同一 CNC scenario：

```text
Run A = DSH
Run B = Pi
```

允许模型推理文本不同，但业务不可变条件必须保持：

- 同一 Work contract；
- 相同 authority boundary；
- 不绕过 receipt；
- 不重复 side effect；
- Outcome/Acceptance 语义一致；
- Provider 失败不会让 Work 丢失。

记录成本/延迟/成功率，但不要为了跑通调低 acceptance。

### G11 Provider Swap Gate

只有两个真实 provider 都跑过才能标真实 PASS；否则 `PASS_WITH_LIMITATION` 或 `BLOCKED_EXTERNAL`。

---

# 24. Phase 20 — Documentation / ADR / Developer Experience

至少更新/新增：

```text
docs/canonical/CURRENT.md
README.md
ARCHITECTURE.md
docs/adr/ADR-MORN-001...007+
docs/guides/provider-authoring.md
docs/guides/capability-authoring.md
docs/guides/profile-authoring.md
docs/guides/local-development.md
docs/guides/factory-reference-scenario.md
docs/api/
docs/evidence/
docs/reports/
```

至少冻结/确认 ADR：

```text
ADR-001 不自研通用 composition framework，reference runtime 采用 Cordis
ADR-002 DSH/Pi 是 Harness Provider，不是 Work Control Plane
ADR-003 Durable Work Truth 独立于 Cordis Fiber / Agent Session
ADR-004 Provider hot-swap 不改变 active Attempt
ADR-005 Capability 进入 qualification/supply-chain governance
ADR-006 标准优先：MCP/A2A/CloudEvents/OTel/OPA/OCI 等
ADR-007 Factory 首包 read-first / no production write
ADR-008 Control loop + reconciliation 优先于 all-in-one agent/DAG
ADR-009 Current projection 可变，released contracts 版本化，history non-destructive
ADR-010 Domain semantic code 不绑定 Cordis implementation types
```

---

# 25. Phase 21 — Build / Test / Quality 收口

必须提供统一命令，最好类似：

```bash
make verify
# 或
./scripts/verify-all
```

一条命令至少串联当前仓库适用的：

```text
format check
lint
typecheck
unit
integration
conformance
migration/schema
build
UI build
e2e/smoke
reference scenario
security/static checks
```

如果某项依赖外部资源，verify 输出必须明确 `SKIP_EXTERNAL`，不能悄悄 success。

代码要求：

- 禁止新大面积重复 abstraction；
- public contract 有文档/类型；
- error code/structured error；
- concurrency/idempotency tests；
- no secret；
- dependency pin/lockfile；
- migration 有 rollback/forward strategy；
- 所有 TODO 都必须有 issue/backlog ref 或明确 reason，禁止大量“以后再说”。

---

# 26. Phase 22 — Release / Evidence Bundle

生成本地/reference release candidate，不宣称 customer production ready。

产出：

```text
docs/reports/V11_5_FINAL_ENGINEERING_REPORT.md
docs/reports/V11_5_GATE_MATRIX.md
docs/reports/V11_5_KNOWN_GAPS_AND_EXTERNAL_BLOCKERS.md
docs/reports/V11_5_PROVIDER_SWAP_REPORT.md
docs/reports/V11_5_FACTORY_REFERENCE_SCENARIO_REPORT.md
docs/reports/V11_5_SECURITY_AND_FAILURE_REPORT.md
docs/reports/V11_5_MIGRATION_REPORT.md
```

Evidence bundle 至少包含：

- Git HEAD；
- exact dependency/runtime versions；
- test counts/results；
- build artifacts/hashes；
- profile conformance result；
- reference scenario run ID；
- timeline/receipt/reconciliation evidence；
- screenshots；
- provider swap evidence；
- baseline vs final diff；
- remaining blockers。

如项目有版本机制，可创建 `v11.5-reference-rc*` 或项目既有命名方式；不要未经授权推生产 release/tag。

---

# 27. Git / Commit 策略

全程：

- 不 force push；
- 不无故 rebase 已发布历史；
- 不删除旧证据；
- 每个 Phase/重大根因修复做可回退 checkpoint commit；
- commit message 清楚区分 spec/control/runtime/provider/factory/ui/test/docs；
- 推远端前确认 branch policy；如果没有明确 push 权限，保留本地 commit 并在 final report 给出 push plan。

如果 worktree 开始不是 clean：

- 先辨认已有用户改动；
- 不覆盖；
- 保存 diff；
- 尽量在其基础上继续；
- 不要用 reset --hard 清理未知工作。

---

# 28. 工程 Gate 总表

最终逐项给出 `PASS / PASS_WITH_LIMITATIONS / BLOCKED_EXTERNAL / FAIL`：

```text
G0  Reality Audit
G1  Semantic Mapping
G2  Durable State / Restart / Replay
G3  Cordis Isolation
G4  DSH Provider
G5  Pi / Second Harness Provider
G6  Active Binding Pin
G7  Receipt / Unknown / Reconciliation
G8  Capability Qualification
G9  Profile Conformance
G10 Factory Thin Slice
G11 Provider Swap Benchmark
G12 Customer Gate
```

## G12 的特殊规则

如果没有真实客户/现场/授权数据：

```text
G12 = BLOCKED_EXTERNAL
```

这不是工程失败，也绝不能被 fake data 改成 PASS。

生产写仍默认：

```text
REAL_PRODUCTION_WRITE = NO
CONTROLLED_REAL_WRITE = NO
```

除非人类单独授权并提供现场安全/回滚/审批条件。

---

# 29. “所有设计都做完”的定义

不要把“所有设计”理解成无限扩张，把未来想法全部编码。

本 Goal 的 **DONE** 定义是：

1. v11.5 中属于 Reference Implementation 的核心语义、Control Plane、Cordis reference composition、Provider contracts、DSH/Pi、Capability Supply Chain 基础、Profiles、Factory reference thin slice、Workbench/Console/Studio 基础路径均已落地或被明确、真实地标记为 external/future provider；
2. 旧 Core/Factory 能力经过显式映射与迁移，没有为了新架构无证据推倒；
3. 所有本地可执行 Gate 通过；
4. reference CNC scenario 可以一键重复运行；
5. crash/restart/unknown/reconcile/provider-failure 路径有自动测试；
6. 至少一个非 Agent executor 路径证明 `Agent is one executor`；
7. 两 Harness 真实可用时完成 swap；若一方外部不可用则诚实阻塞；
8. UI 可完成至少一个 Work 从创建/观察到验收的全链路；
9. 文档、ADR、报告和代码一致；
10. 没有虚假“客户价值/生产写/商业成熟”声明。

未来大型能力（大规模 DSec、K8s、SPIFFE 全栈、WASM 全组件化、真实 ERP/MES 全矩阵、全行业 Domain Pack）不因存在 contract 就要求在此次全部实现；除非仓库原本已经具备或实现它们是完成当前 Gate 的必要条件。

---

# 30. Agent 自主运行机制：避免上下文耗尽后失控

从开始即维护：

`docs/reports/V11_5_EXECUTION_STATE.md`

每完成一个 Phase 更新：

```text
Current HEAD
Completed phases
Current gate status
Tests last run
Architecture decisions
Files changed
Open root causes
External blockers
Next exact actions
```

如果 Agent 即将换会话/上下文不足：

1. 先确保代码处于可恢复状态；
2. 跑与本轮改动相关的测试；
3. commit 或至少保存 clean diff；
4. 更新 `V11_5_EXECUTION_STATE.md`；
5. 写 `docs/reports/V11_5_HANDOFF.md`；
6. 下一 Agent 必须从这两个文件和 canonical 开始，不从聊天记忆猜测。

不要因为换 Agent 重做已通过的 Phase，除非 evidence stale 或代码已改变。

---

# 31. 执行时的优先级算法

任何时候有多个任务，按以下优先级：

```text
P0 数据损坏 / 安全绕过 / 假通过 / canonical truth 问题
P1 Durable state / Binding / Receipt / Reconciliation / Authority
P2 Protocol / compatibility / provider isolation
P3 Factory thin slice / end-to-end product path
P4 Capability supply chain / Studio
P5 UX polish / optional providers / future scale
```

先修根因，不堆补丁。

---

# 32. 最终交付格式

最终不要只回复“完成”。必须给出：

## A. Repository State

```text
branch
HEAD
origin relationship
worktree status
```

## B. Architecture Completion

六 Plane 每一层：

```text
IMPLEMENTED
PARTIAL
EXTERNAL_BLOCKED
DEFERRED_BY_DESIGN
```

并给证据文件/测试路径。

## C. Gate Matrix

G0-G12 完整状态。

## D. Test Matrix

真实数量：

```text
unit
integration
conformance
e2e
build/lint/typecheck
failure-injection
```

不得写模糊“全部通过”。

## E. Factory Scenario

CNC-17 完整运行证据，特别说明：

```text
DSH/Pi
binding
unknown
reconciliation
receipt
outcome
acceptance
no duplicate action
```

## F. External Blockers

逐项写解除条件。

## G. Remaining Debt

只保留真实债务；每项给优先级、原因、是否阻塞发布。

## H. Next Product Step

只有在本 Goal 工程收口后，才提出下一阶段真实客户 read-only/shadow 计划。

---

# 33. 开始执行

现在开始，不要先重写设计文档，也不要先做大规模重构。

第一步执行：

```text
1. 找 repo root、HEAD、branch、worktree
2. 定位 v11.5 canonical
3. 完整读取 canonical
4. 运行现有 baseline gates
5. 生成 V11_5_REALITY_AUDIT.md
6. 生成 CODE_MAPPING_AND_MIGRATION_MATRIX.md
7. 只有 G0 证据齐全后进入 Phase 1
```

之后按照本 Goal 持续推进直到：

```text
ALL_LOCAL_GATES_PASS
or
ENGINEERING_COMPLETE_WITH_EXTERNAL_BLOCKERS
```

**不要在中途因为“这一轮做了很多”就自行停止。**
