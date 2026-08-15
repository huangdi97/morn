# GOAL3_CERTIFICATION_SPEC.md

## CertificationSpec

```yaml
certification_spec:
  capability_id:
  capability_version:
  context_of_use:
  required_evaluation_suites:
  minimum_acceptance_rate:
  policy_requirements:
  recovery_requirements:
  reproducibility_requirements:
  human_gate_requirements:
  provenance_requirements:
  known_failure_coverage:
  compatibility_requirements:
```

## Decision states
Draft → EvaluationPending → Conditional → Certified → Restricted → Failed → Suspended → Deprecated → Retired

## Certified Work Capability

```text
WorkPackageTemplate
+ WorkcellBlueprint
+ Role/Harness bindings
+ Workflow
+ AcceptanceSpec
+ OutcomeContract
+ EvaluationPack
+ Historical Cases
+ Deployment Profile
+ Certification Evidence
```

禁止一个 demo pass 就 Certified；禁止忽略失败 case；禁止 critical gap 下认证；高风险 capability 必须保留 human gate；版本变化要判断 patch-compatible / re-evaluation / full recertification。
