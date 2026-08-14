# BIOLAB_V01.md — 第一块真实样板

## 定位

BioLab v0.1 是 Morn Work/World/Evidence/Human-in-loop 的验证域，不是假装全自动实验室。

## 最小对象

```text
Dataset
Sample（可选）
AnalysisRun
QCResult
ScientificClaim
Figure/Report Artifact
Review
Approval
```

## 最小动作

```text
register_dataset
start_analysis
submit_analysis_artifact
request_review
approve_claim
exclude_sample（可选，用于 E2/E3 policy 演示）
release_claim
```

## 最小角色

- Analyst：Actor 或 Human
- Pipeline：Deterministic Worker
- Statistical Reviewer：Actor/Human
- PI：Human approver

## 必跑闭环

```text
Dataset registered
→ WorkPackage: dataset_to_reviewed_claim
→ AnalysisRun
→ Artifact v1
→ Reviewer review
→ PI approval
→ governed release action
→ StateDiff
→ Outcome: Reviewed Scientific Claim
```

## AcceptanceSpec

至少：
- dataset locked/versioned
- analysis artifact exists
- reviewer decision exists
- human approval exists
- provenance complete
- claim links dataset + analysis + artifact + decision
- no forbidden failure condition

## UI 演示

Workbench：
- 当前 workcell 与 attention

Studio：
- 查看/编辑 WorkPackage template

Console：
- approval / event / receipt / state diff

Hub：
- BioLab DomainPack / WorkPackageTemplate / EvaluationPack

## 数据

使用小型可提交 fixture。
不要引入大科学数据集。
重点验证工作语义和证据链，而不是算法性能。
