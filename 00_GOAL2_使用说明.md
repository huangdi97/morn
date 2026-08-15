# Morn v10.2-R1 — Goal 2 使用说明

## Goal 2
**Organization/Solution Compiler + Durable Work Runtime + Simulation/Evaluation + BioLab Dream Factory v0.1**

Goal ID：`MORN-V10.2-G2-SOLUTION-FACTORY`

## 前置条件
默认 Goal 1 已完成：Kernel、Operational World L0、Artifact/Decision/Outcome、WorkPackage/Acceptance、Harness/Runtime 解耦、Action Gateway、E0-E3、durable minimum、Evolution guard、BioLab 最小闭环及基础 UI 都已存在。

Goal 2 **只做增量**，不得重写 Goal 1 已验证语义。

## Goal 2 要解决的问题

Goal 1：Morn 能否可靠执行受治理 Work？

Goal 2：Morn 能否把一个自然语言目标/业务问题，编译成可审查、可执行、可恢复、可回放、可仿真、可评测的 Work System / Solution，并在 BioLab 中跑通三个科研闭环？

目标链路：

```text
Goal / Problem
→ ProblemSpec
→ WorkGraph
→ WorkPackage / WorkContract
→ RoleSlots / Workcell
→ ExecutionMode
→ Capability Resolution / Gap
→ HarnessBinding
→ Workflow
→ Evaluation Plan
→ ProposedSolution
→ Human Review
→ SolutionPackage
→ Replay / Simulation
→ Shadow
→ BioLab execution
→ Outcome
```

## 不属于 Goal 2
- Operational World L2/L3；
- 通用 Predictive/Causal World Model；
- 企业级分布式 Durable Runtime；
- 自动真实岗位重构/软件退役；
- 真实机器人/仪器自治；
- 多行业 Dream Factory；
- Managed Work / Work-as-a-Service 商业交付；
- ERP/MES/APS Replacement Pilot。

## 使用
复制整个包到 Morn 仓库根目录，然后给 Codex：

```text
Goal 1 已完成。请读取并严格执行 GOAL2_CODEX_MASTER_PROMPT.md。先验证 Goal 1 基线，然后从 Goal 2 M0 开始连续执行。除真实外部 blocker 外不要停下来问我下一步。
```
