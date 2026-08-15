# Goal 3 恢复指令

如果 Codex 上下文重置：

```text
读取 AGENTS.md、GOAL3.md、GOAL3_PLAN.md、GOAL3_ACCEPTANCE.md、STATUS_GOAL3.md、DECISIONS.md、BLOCKERS.md、KNOWN_FAILURES.md，以及 Goal1/Goal2 final report。
查看 git status、git diff、最近提交与测试。
不要重做已经完成的 Goal 3 milestone。
从 STATUS_GOAL3.md 第一个未完成 milestone 继续。
普通失败直接修，不要停在分析。
```

最终收尾：

```text
逐条执行 GOAL3_ACCEPTANCE.md。
每一条给出代码位置、测试位置、命令、结果。
运行 Goal1+Goal2 全量 regression。
生成 reports/goal3_final_report.md。
```
