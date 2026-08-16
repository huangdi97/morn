# Codex Context Reset Recovery

```text
继续执行 MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER。

读取：
MASTER_CODEX_PROMPT.md
FINAL_ACCEPTANCE.md
STATUS.md
DECISIONS.md
BLOCKERS.md
KNOWN_FAILURES.md
reports/final_reality_audit.md
reports/final_backlog_audit.md
以及已开始的 final report。

然后执行：
git status --short --branch
git remote -v
git log -10 --oneline
git fetch origin --prune（仅在 remote 已配置时）

不要重做已完成项。
不要 merge 旧 GitHub Morn。
从 STATUS 第一个未完成项目继续。

最终目标不变：
MUST_FIX_NOW=0
ALL_LOCAL_TESTS=PASS
GIT_STATUS=CLEAN
LEGACY backup success
morn-v1 push success
main takeover success 或只剩真实 GitHub external blocker。
```
