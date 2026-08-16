# GOAL — Morn v1.0 GA Re-Foundation + GitHub Takeover

Goal ID:

`MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER`

## 最终 Definition of Done

只有满足以下条件才算本 Goal 完成：

```text
MUST_FIX_NOW = 0

ALL_LOCAL_TESTS = PASS

ALL_APPLICABLE_UI_TESTS = PASS

CORE_ARCHITECTURE = CLEAN

ZERO_DOMAIN_CORE = PASS

GIT_STATUS = CLEAN

LOCAL_FINAL_COMMIT = RECORDED

LEGACY_GITHUB_BACKUP = SUCCESS

NEW_MORN_BRANCH_PUSH = SUCCESS

NEW_MAIN_TAKEOVER = SUCCESS
或仅因真实 branch protection/auth/network 被 EXTERNAL_BLOCKED

FINAL_REMOTE_MAIN = 当前全新 Morn
或在 external blocker 情况下明确记录尚未完成的唯一远端步骤
```

## GitHub 最终目标

```text
GitHub repo: morn

main
└── 当前全新 Morn v1 主线

legacy/pre-rewrite
└── 很久以前旧 Morn

tag:
legacy-pre-rewrite
v1.0.0-rc.1  # 若版本策略允许
```

## 严禁

```text
git pull --allow-unrelated-histories
git merge --allow-unrelated-histories
普通 git push --force
删除旧 GitHub 历史
把旧源码复制进当前工程
把当前工程 merge 到旧 main
```

如果需要替换旧 main，只允许：

```text
legacy backup verified
+ fetch fresh
+ remote main unchanged since fetch
+ git push --force-with-lease
```
