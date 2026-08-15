# GOAL2_BIOLAB_DREAM_FACTORY.md

## 定位
BioLab v0.1 是第一块可复用 Dream Factory 样板，输出的是经过验证的 Work Capability 候选，而不是一个“科研 Agent”。

## Domain Pack
至少：Dataset、Sample、LiteratureSource、EvidenceItem、Hypothesis、ExperimentDesign、AnalysisRun、QCResult、ScientificClaim、Figure、Table、Manuscript、Review、Approval。

## Loop A Literature → Experiment Design
成员：Evidence Research Actor、Evidence Validator、Experiment Designer、Statistical/Method Reviewer、Human PI。

Artifacts：EvidenceMap、EvidenceSummary、HypothesisSpec、ExperimentDesign、ReviewReport。

Acceptance：sources versioned、hypothesis traceable、conflicting evidence recorded、design has criteria、physical experiment 前 PI approval。

## Loop B Dataset → Reviewed Claim
成员：Analyst Actor、deterministic pipeline、Data Steward、Statistical Reviewer、Human PI。

Artifacts：DatasetSnapshot、QCReport、AnalysisReport、Figure/Table、ClaimPackage、ReviewReport。

Acceptance：locked dataset/version、code/env/params provenance、independent review、claim evidence links、approval、reproducibility。

## Loop C Result → Manuscript Consistency
成员：Scientific Writer Actor、Figure/Table Worker、Claim-Evidence Checker、Reviewer、Human PI。

Artifacts：FigureSet、TableSet、ManuscriptDraft、ClaimEvidenceMatrix、ConsistencyReport、ReleaseCandidate。

检查：figure/data mismatch、unsupported claim、stale artifact、conflicting statistics、missing provenance、claims beyond approved evidence。

## Wet Lab
保持 Human / Device placeholder；可以有 planned action/execution receipt schema，但不伪造机器人控制。

## 可导出资产
BioLab DomainPack、3 个 WorkPackageTemplate、RoleBlueprints、EvaluationPack、SimulationScenarios、SolutionTemplate、BioLab manifest。
