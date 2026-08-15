# Goal 2 恢复与分阶段指令

## A 上下文重置
读取 AGENTS.md、GOAL2.md、GOAL2_PLAN.md、GOAL2_ACCEPTANCE.md、STATUS_GOAL2.md、DECISIONS.md、BLOCKERS.md、KNOWN_FAILURES.md，并查看 git status/diff/最近提交/测试。不要重做已完成 Milestone，从第一个未完成阶段继续。

## B Compiler 卡住
读取 GOAL2_COMPILER_SPEC.md。先做 Goal→ProblemSpec→WorkGraph→WorkPackage→Validation vertical slice，补 contract tests，再扩 ExecutionMode/MemberType/CapabilityResolver/HarnessPlan/EvaluationPlan。缺 capability 必须 CapabilityGap。

## C Durable 卡住
读取 GOAL2_DURABLE_RUNTIME_SPEC.md。先保证 checkpoint→process restart→load→drift check→resume 自动测试，再补 signal/wait、retry、compensation、escalation、budget、attention。内存状态不算 durable。

## D Replay/Simulation 卡住
确认 runner 使用隔离 state store，绝不写 production。先 tool failure + approval missing 两个 scenario，输出结构化 EvaluationReport，再扩全部 scenario。

## E BioLab 卡住
先让 Loop B 通过 Goal 2 durable/evaluation，再做 Loop A/C。科学内容不硬编码 UI；每个 Claim 有 provenance/review/approval/version。

## F UI 卡住
Studio 先 Goal→Compiler→Review→Manifest；Workbench timeline+WorkGraph；Console durable/eval/representation；Hub Solution/Domain/Evaluation/Simulation assets。必须接真实 backend。

## G 最终验收
逐条 GOAL2_ACCEPTANCE，每条给代码位置、测试位置、命令、结果。修完所有本地可修复失败，跑 Goal 1 全量 regression，生成 reports/goal2_final_report.md。
