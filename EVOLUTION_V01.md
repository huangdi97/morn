# EVOLUTION_V01.md — 受治理 Evolution Engine v0.1

## 今晚范围

只做：

```text
Trace / Outcome / Human Correction
→ Candidate
→ Branch
→ Evaluation
→ Promotion Decision
→ New Version / Rollback Point
```

不做：
- 自动重构真实岗位；
- 自动退役软件；
- 自动改组织；
- 自动部署高风险能力。

## Candidate 类型

今晚可支持 enum/schema：
- Capability
- Workflow
- HarnessPatch
- Workcell
- RoleSuggestion
- SoftwareReplacementSuggestion

后两类只建议，不执行。

## Candidate 字段

至少：
- id
- candidate_type
- source_window
- evidence_refs
- current_version
- proposed_change
- expected_benefit
- risk
- required_evaluations
- status
- created_by
- created_at

## Branch

- 基于 production version；
- isolated config/state；
- 不共享生产写权限；
- 记录 base_version。

## Evaluation

最少：
- correctness
- regression
- policy
- cost/latency（可有数据才评）
- acceptance outcome
- pass/fail + evidence

## Promotion

只有：
- evaluation passed；
- required approvals satisfied；
- policy allows；

才创建新的 production version。

`promote()` 不能被单个 Actor 绕过 Policy/Certification。

## Rollback

Promotion 保存：
- previous_version
- new_version
- migration/changeset ref
- rollback eligibility
- rollback action/ref

## 最关键测试

1. Candidate 直接调用 production mutation → fail
2. failed evaluation promote → fail
3. no approval for high risk → fail
4. successful promote → new version
5. rollback → previous version becomes active 或产生等价 rollback record（按仓库设计）
