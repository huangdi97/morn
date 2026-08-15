# Goal 5 恢复与强制继续指令

## 上下文重置
```text
Goal5 上下文已重置。立即读取 GOAL5_CODEX_MASTER_PROMPT.md、GOAL5_PLAN.md、GOAL5_ACCEPTANCE.md、STATUS_GOAL5.md、DECISIONS.md、BLOCKERS.md、KNOWN_FAILURES.md，以及已生成的 goal5 audit/report。查看 git status/diff/recent commits/tests。不要重做已完成 milestone，从 STATUS_GOAL5.md 第一个未完成项继续。普通错误自己修，不征求许可。
```

## 只分析不执行
```text
停止输出计划。选择当前第一个未完成 milestone，打开真实代码和测试，先完成一个 vertical slice 并跑绿，然后继续下一 slice。只有真实外部 blocker 才能停止。
```

## Domain Neutralization 卡住
```text
先生成 Core→Domain dependency graph。任何 concrete domain import 先抽 public contract/interface，让 reference pack 适配 SDK。优先让 pure-core build/test/start green。
```

## Distributed Runtime 卡住
```text
不需要真实集群。必须用两个本地/simulated nodes 跑 lease/failover/checkpoint/dedupe 集成测试。一个 worker 不能算 distributed proof。
```

## 最终验收
```text
逐条执行 GOAL5_ACCEPTANCE.md，为每项记录代码、测试、命令、结果。运行 Goal1–4 regression、全部 conformance、安全、chaos、Pure Core E2E、Reference Pack E2E。所有本地可修项修到 green，生成 reports/goal5_final_report.md，并明确 CORE COMPLETE = YES/NO。
```
