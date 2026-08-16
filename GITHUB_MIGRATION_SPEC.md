# GitHub 旧 Morn → 全新 Morn 安全接管规范

## 原则

旧仓库与新工程完全无关，因此：

```text
NO MERGE
NO PULL OF OLD CODE
NO ALLOW_UNRELATED_HISTORIES
```

## 安全链

```text
fetch old repo
→ identify old default
→ archive old default to legacy/pre-rewrite
→ tag old commit
→ verify
→ push new project to morn-v1
→ verify
→ fresh fetch
→ ensure old main unchanged
→ force-with-lease new project to main
→ verify
```

## 归档旧版本

旧 default 假设 `origin/main`：

```bash
git fetch origin --prune

git push origin origin/main:refs/heads/legacy/pre-rewrite

git tag -a legacy-pre-rewrite <OLD_SHA> -m "Legacy Morn before full rewrite"
git push origin refs/tags/legacy-pre-rewrite
```

如果旧 default 是 master，使用对应 old ref。

## 推新工程

```bash
git push -u origin HEAD:morn-v1
```

## 新 main takeover

在 fresh fetch + backup verified 后：

```bash
git push --force-with-lease origin HEAD:main
```

仅允许 `--force-with-lease`。

## 注意

如果 remote main 在最后一次 fetch 后发生改变，
force-with-lease 应失败。
这是正确的安全行为。

不得改用 `--force` 绕过。

## Branch protection

若保护规则拒绝 takeover：

- 不绕过；
- 记录 blocker；
- 保留 `morn-v1`；
- 用户只需在 GitHub 调整保护/默认分支后再执行最终一步。
