# GOAL3.md

## Goal ID

`MORN-V10.2-G3-DELIVERY-EVOLUTION-REPLACEMENT`

## 北极星

把 Morn 从“能编译和运行 Work System”推进到：

> **能够认证 Work Capability、托管 Outcome Delivery、从真实运行轨迹中形成受治理进化候选，并完成至少一个可量化 Shadow Replace / Partial Replace 决策的 Morn v0.3。**

## G3.1 Certified Work Capability

实现：WorkCapabilityCandidate、CertificationSpec、CertificationRun、CertificationDecision、CertifiedWorkCapability、CapabilityRelease、CompatibilityProfile、DeprecationPolicy。

认证至少检查：Acceptance pass、Policy/safety、Reproducibility、Recovery、Provenance、Human intervention profile、Cost/latency、Known failure coverage、Version compatibility。

Certified 不等于 L5 Autonomous。

## G3.2 Evolution Flywheel v0.2

```text
Trace / Process / Outcome / Human Correction
→ Pattern
→ Bottleneck / Repetition / Failure
→ Candidate
→ Replay
→ Evaluation
→ Shadow
→ Certification
→ New Capability / Workflow / Harness Patch
```

至少支持：Skill candidate、Workflow candidate、Harness patch candidate、deterministic distillation candidate、Workcell candidate、Role suggestion、software replacement suggestion。生产版本不得自修改。

## G3.3 Minimum Sufficient Intelligence / Distillation

建立：

```text
Repeated Agent Path
→ Stable Decision Boundary?
→ Extract Rule / State Machine / Program
→ Regression Evaluation
→ Shadow
→ Candidate Deterministic Capability
```

至少做一个真实示例：一个原本由 Actor 执行的稳定步骤被蒸馏为 deterministic worker，保留长尾 fallback 给 Actor，并比较质量、成本、延迟、人类负担。

## G3.4 Managed Work / Outcome Delivery

实现 ManagedWorkService、ManagedWorkcell、DeliveryLifecycle、DeliveryReceipt、AcceptanceDecision、RetryLiability、Escalation、HumanFallback、SLO / Outcome metric tracking。

```text
Certified Work Capability
→ Managed Workcell
→ OutcomeContract
→ Execute
→ Verify
→ DeliveryReceipt
→ Acceptance
```

## G3.5 Delivery SLO

至少支持：quality SLO、deadline、acceptance method、business/scientific metric、target、retry liability、human fallback、evidence required。保留 billing basis schema，但不做复杂计费。

## G3.6 Replacement Pilot

默认试点：**BioLab Dataset → Reviewed Scientific Claim**。

对比：Existing/Manual Baseline vs Morn Orchestrated vs Morn-native Candidate。

```text
R0 Observe
R1 Assist
R2 Orchestrate
R3 Shadow Replace
R4 Partial Replace
```

Goal 3 至少做到 R3，并在指标明确优于/不劣于 baseline 时生成 R4 Decision Candidate。

## G3.7 Value Validation

至少比较：Quality、Acceptance pass rate、Cycle time、Human intervention、Error/rework、Cost estimate、Evidence coverage、Recovery success、Policy violations、Outcome metric。Replacement 决策不能只比较文本输出。

## G3.8 Software Absorption Record

实现 ExistingSystemMapping、ObservedWorkGraph、NativeCandidate、ShadowComparison、ReplacementDecision、ReplacementRecord、RetirementCandidate。Goal 3 不真正退役关键生产系统。

## G3.9 Evolution Center UI

新增 Candidate discovery、Evidence window、Expected benefit、Risk、Required evaluation、Shadow status、Certification gate、Promotion/Rollback、Distillation opportunity、Replacement opportunity。

## G3.10 Managed Work UI

Workbench：delivery queue、SLO、active managed workcell、human fallback、acceptance。
Console：certification、delivery lifecycle、replacement records、capability release/version。
Hub：Certified Work Capability、release history、compatibility、evaluation evidence。

## Final E2E

```text
Goal2 Solution
→ Production/fixture runs
→ Trace/Outcome collection
→ Evolution Candidate
→ Distillation/Workflow improvement
→ Replay/Evaluation
→ Shadow
→ Certification
→ Certified Work Capability
→ Managed Delivery
→ DeliveryReceipt
→ Human Acceptance
→ Baseline Comparison
→ Replacement Decision Candidate
→ Feedback to Evolution
```
