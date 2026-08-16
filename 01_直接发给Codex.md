# 直接发给 Codex

```text
当前本地 Morn 是从头开发的全新工程。

GitHub 上已有一个很久以前的旧 morn 仓库，
但旧仓库与当前工程不是继承关系。

现在请执行最终 Goal：

MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER

读取并严格执行根目录 MASTER_CODEX_PROMPT.md。

不要 merge 旧 GitHub 源码。
不要 pull unrelated history。
不要使用 --allow-unrelated-histories。

先完成所有本地最终收口：

- Goal1–5 regression
- 全仓 TODO/FIXME/mock/placeholder/ignored 审计
- MUST_FIX_NOW 清零
- 架构检查
- Rust/backend 代码质量与风格
- API/数据库/migration
- Provider/Runtime/Connector/Distributed Runtime
- Domain/Plugin/SDK
- Workbench/Studio/Console/Hub 全部 UI
- 共享 UX
- CLI/DX
- Security
- Chaos
- Performance baseline
- Full Test Matrix
- Playwright/Tauri
- Docs
- GitHub Actions
- Packaging
- Git cleanup
- logical commits
- final local regression
- git status clean

然后处理 GitHub：

1. fetch 旧 morn；
2. 识别旧 default branch；
3. 将旧版 tip 归档为：
   legacy/pre-rewrite
4. 创建并 push：
   legacy-pre-rewrite annotated tag
5. 验证 legacy branch/tag；
6. 当前全新 Morn 先 push 到：
   morn-v1
7. 验证 morn-v1 == LOCAL_FINAL_COMMIT；
8. fresh fetch；
9. 确认旧 main 在归档后没有被别人更新；
10. 不 merge 两套历史；
11. 在所有安全条件满足后，使用：
    git push --force-with-lease origin HEAD:main
    让当前全新 Morn 接管正式 main；
12. 禁止普通 --force；
13. push 后再次 fetch 验证 origin/main == LOCAL_FINAL_COMMIT；
14. 如果版本策略明确，创建并 push v1.0.0-rc.1；
15. 生成 reports/morn_v1_ga_final_report.md。

只有真实 GitHub auth/permission/branch protection/remote/network，
真实 Secret、许可证、真实外部系统、硬件或数据风险
才允许成为 blocker。

任何 external blocker 出现后，继续完成其他所有任务。

最终必须明确：

MUST_FIX_NOW=0
ALL_LOCAL_TESTS=PASS
GIT_STATUS=CLEAN
MORN_V1_GA_LOCAL_COMPLETE=YES
LEGACY_GITHUB_BACKUP=SUCCESS
NEW_MORN_BRANCH_PUSH=SUCCESS
NEW_MAIN_TAKEOVER=SUCCESS

或者 main takeover 只剩唯一真实 GitHub external blocker。

不要只给计划。
不要停下来问下一步。
现在直接连续执行到结束。
```
