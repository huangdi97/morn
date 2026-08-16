# Morn 最终 Codex MASTER 指令
## Re-Foundation + GitHub Takeover

Goal ID:
`MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER`

当前事实：

1. Goal1–5 已经完成。
2. 当前本地 Morn 是全新的、从头开发的工程。
3. GitHub 上已经存在一个旧的 `morn` 仓库。
4. 旧 GitHub Morn 与当前工程不是继承关系。
5. 不要 merge 两套源码或 Git 历史。
6. 最终目标是：旧版本安全归档，当前全新 Morn 接管正式 `main`。

你必须持续执行，直到所有本地可完成项全部清零，并完成 GitHub 安全迁移或仅剩真实外部 GitHub blocker。

---

# 第一阶段：真实仓库与 Goal1–5 复验

先读取：

- AGENTS.md
- 最新 Morn 设计母版
- Goal1–5 final reports
- STATUS*.md
- DECISIONS.md
- BLOCKERS.md
- KNOWN_FAILURES.md
- 本包所有 spec / acceptance 文档

然后执行：

```bash
git status --short --branch
git branch --show-current
git log -20 --oneline
git remote -v
```

运行当前仓库全部官方回归脚本。

如果 Goal5 并非真正 CORE COMPLETE：
先把所有本地可修项目补完。

生成：

`reports/final_reality_audit.md`

---

# 第二阶段：全仓最终 Backlog Audit

搜索：

```text
TODO
FIXME
XXX
HACK
TEMP
placeholder
mock
stub
dummy
todo!()
unimplemented!()
ignored
allow(dead_code)
```

同时检查：

- API 501 / NotImplemented
- 空页面
- 静态假成功数据
- 生产代码 fixture
- dead route
- duplicate service/repository/runtime
- deprecated but live
- debug prints
- commented-out production implementation
- 临时代码
- 巨型 god modules
- 明显 copy-paste 逻辑

生成：

`reports/final_backlog_audit.md`

分类：

```text
MUST_FIX_NOW
EXTERNAL_BLOCKED
INSTANCE_LAYER_FUTURE
TEST_FIXTURE
SAFE
```

最终：

`MUST_FIX_NOW = 0`

---

# 第三阶段：架构审计

验证：

```text
Core 不依赖 concrete Domain
Domain/Plugin 只能依赖 public Morn SDK
UI 不直接写数据库
Provider 不成为 canonical source of truth
Runtime 不拥有业务事实
Connector 写操作必须经过 Action Gateway
Plugin 不绕过 permission/policy
Evolution 不直接修改 production
Distributed Runtime 不复制 canonical business state
```

检查是否存在：

- BioLab/Factory 特殊分支
- Core import concrete domain
- UI hardcode domain nav
- duplicate canonical implementations
- old architecture path 和 new architecture path 同时存活

发现问题时：

1. 先补 architecture test；
2. 迁移调用方；
3. 删除 obsolete path；
4. 重新跑 regression。

禁止创建 V2/V3 平行空壳。

---

# 第四阶段：Rust / Backend 全仓审查

修复所有本地 Must-fix：

- fmt/clippy
- compile warnings
- 不必要 unwrap/expect/panic
- silent error swallowing
- weak error context
- duplicated business logic
- 巨型职责混杂模块
- hidden mutable globals
- missing transaction
- missing idempotency
- missing cancellation/timeout
- missing validation
- missing provenance/audit
- inconsistent naming
- secrets/log leakage
- deprecated APIs
- dead code

原则：

```text
small modules
high cohesion
low coupling
one canonical implementation
typed errors
explicit state transitions
```

---

# 第五阶段：Persistence / Migration / API

必须重新验证：

- fresh DB
- existing DB upgrade
- migration version
- migration failure handling
- restart hydration
- workspace isolation
- idempotency
- version conflict
- pagination/filter/sort
- request validation
- typed API errors
- health/readiness
- graceful shutdown/restart

所有本地失败全部修复。

---

# 第六阶段：Capability / Provider / Runtime / Connector

重新跑全部 conformance。

至少验证：

```text
Provider:
health
scope
invoke
cancel
timeout
normalized events/errors
secret redaction

Runtime:
start
pause
resume
checkpoint
restore
signal
cancel

Connector:
read
query
event
mapping
retry
rate limit
idempotency
governed write
teardown
```

Provider/Connector 不得直接修改 canonical DB。

---

# 第七阶段：Node / Distributed Durable Runtime

重新实际跑本地双节点 E2E：

```text
Node A claims work
→ checkpoint
→ Node A failure
→ lease expiry
→ Node B takeover
→ restore
→ duplicate event arrives
→ dedupe
→ external effect not duplicated
→ work completes
→ audit failover record
```

额外验证：

- stale worker completion rejected
- capability mismatch
- signal routing
- checkpoint drift
- retry exhausted
- duplicate signal
- reordered event

禁止单 worker 测试冒充分布式。

---

# 第八阶段：Domain / Plugin / Package / SDK

验证：

- Core 在 zero-domain 状态可启动；
- generic work E2E；
- hello/reference domain install；
- enable/disable；
- upgrade；
- uninstall；
- historical provenance preserved；
- broken plugin isolation；
- permissions；
- compatibility；
- dynamic UI extension。

不做正式行业实例。

---

# 第九阶段：Workbench UI 最终收口

逐项检查真实后端绑定和 Playwright：

- Mission
- Work / WorkPackage
- WorkGraph
- Workcell
- Execution Timeline
- Checkpoint
- Wait
- Retry
- Approval
- Attention
- Artifact
- Decision
- Outcome
- Delivery/Managed Work where generic

必须覆盖：

```text
loading
empty
error
blocked
permission denied
degraded
external verification pending
```

修死按钮、死链、console error、静态假数据。

---

# 第十阶段：Studio UI 最终收口

完整检查：

- WorkPackage Builder
- WorkContract
- Workflow
- Actor / Role / Workcell
- Capability
- Provider / Harness / Runtime
- Connector
- Solution Compiler
- Evaluation
- Simulation
- Domain / Plugin builder

要求：

- invalid form cannot save
- server errors shown
- version/edit semantics consistent
- no mock production backend

---

# 第十一阶段：Console UI 最终收口

检查：

- Workspace
- Identity
- Policy
- Permissions
- Approvals
- Nodes
- Topology
- Runtimes
- Providers
- Connectors
- Plugins
- Domain Packs
- Migration
- Version
- Audit
- Incidents
- Health/Readiness
- Secret health

危险操作：
必须确认、权限检查或审批。

---

# 第十二阶段：Hub UI 最终收口

检查：

- Packages
- Providers
- Plugins
- Domains
- Templates
- Versions
- Dependencies
- Compatibility
- Trust
- Install / Enable / Disable / Upgrade

不能使用 fake marketplace 数据宣称真实 marketplace。

---

# 第十三阶段：Shared UI / UX

统一：

- design tokens
- spacing
- typography
- buttons
- forms
- tables
- status badges
- dialogs
- notifications
- loading skeleton
- errors
- empty state

基础 accessibility：

- labels
- keyboard reachable
- focus
- semantic controls
- readable contrast under existing theme

不无意义重做品牌视觉。

---

# 第十四阶段：CLI / DX

保证新开发者能：

```text
clone
→ install dependencies
→ doctor
→ migrate/init
→ run
→ test
→ package
```

Windows PowerShell 必须能用。

修：

- scripts
- env example
- doctor/status
- missing dependency messages
- migration/start/test/package commands

---

# 第十五阶段：Security

检查并修：

- cross-workspace leakage
- secrets in logs/UI/errors
- unauthorized connector writes
- plugin permissions
- node identity
- stale lease
- path traversal
- package extraction
- process/shell invocation boundary
- malformed inputs
- unsafe debug endpoints
- duplicate E3 action

本地可修安全问题必须清零。

---

# 第十六阶段：Chaos / Reliability

实际注入：

- process crash
- DB busy/locked
- duplicate event
- reordered event
- provider timeout
- malformed provider result
- connector timeout
- node loss
- stale lease
- checkpoint mismatch
- migration failure
- plugin init failure
- duplicate action

每种情况必须：

```text
recover
或
block
或
escalate
或
compensate
```

并保留 audit。

禁止 silent corruption。

---

# 第十七阶段：Performance / Resource Baseline

记录：

- cold startup
- server readiness
- representative API
- representative Work E2E
- DB hot-path observations
- frontend production build size
- memory/process if practical

修明显问题：

- N+1
- unbounded retry
- unbounded loop
- hot-path full table scan
- accidental huge payload
- obvious bundle explosion

不做无数据的过度优化。

---

# 第十八阶段：Full Test Matrix

最终必须跑全部适用项：

- backend fmt/check/clippy
- unit
- integration
- contract
- architecture
- migrations
- API
- Provider conformance
- Runtime conformance
- Connector conformance
- Plugin conformance
- DomainPack conformance
- Distributed E2E
- Security
- Chaos
- frontend typecheck/lint/test/build
- Playwright
- Tauri/Desktop
- Pure Core E2E
- Reference Pack E2E
- Goal1–5 regression

规则：

- 不删除失败测试
- 不 ignored
- 不降低断言
- flaky 找根因
- external-only smoke 单独记录

---

# 第十九阶段：Docs / Release Docs

更新：

- README
- Quickstart
- Architecture
- Core Concepts
- Domain SDK
- Plugin SDK
- Provider/Runtime/Connector
- Distributed Runtime
- Deployment
- Migration/Upgrade
- Security
- Troubleshooting
- CHANGELOG
- Release Notes

验证文档命令。

---

# 第二十阶段：CI/CD

检查 `.github/workflows`。

至少覆盖：

- fmt
- lint/clippy
- backend tests
- frontend test/build
- conformance
- key E2E where feasible
- package/build

禁止用 continue-on-error 掩盖核心失败。

---

# 第二十一阶段：Packaging

验证/构建：

- Rust server/CLI
- Tauri/Desktop
- frontend production assets
- migrations/config/examples
- SDK/package artifacts if existing architecture supports

记录 release artifact path。

---

# 第二十二阶段：本地 Git 清理

在任何远端迁移前：

```bash
git status --short --branch
git diff
```

检查：

- `.env`
- tokens/secrets
- local DB
- logs
- caches
- target
- node_modules
- dist
- test artifacts

修 `.gitignore`。

将最终变更组织成合理逻辑 commits。

不要 force push。
不要 destructive reset。

最终关键测试重新跑。

本地应达到：

```text
GIT_STATUS = CLEAN
LOCAL_FINAL_COMMIT = <sha>
```

---

# 第二十三阶段：识别旧 GitHub `morn`

重要：

当前本地工程和远端旧 Morn 是完全不同的工程历史。

如果当前 repo 已有 remote：

```bash
git remote -v
git fetch origin --prune
```

如果没有 remote：
先检查已有文档/git config/environment 是否能可靠确定旧 `morn` URL。

如果无法可靠确定：
记录：

`EXTERNAL_GITHUB_REMOTE_REQUIRED`

不要猜 URL。

识别远端默认分支：

- `origin/main`
- 或 `origin/master`
- 或 remote HEAD 指向的实际分支。

生成：

```text
REMOTE_OLD_DEFAULT=<branch>
```

不要执行 pull。
不要 merge。
不要 allow-unrelated-histories。

---

# 第二十四阶段：旧仓库 Legacy 归档

对旧 GitHub 默认分支的当前 tip：

创建远端归档分支：

`legacy/pre-rewrite`

如果本地名称冲突，可使用明确 refspec，不破坏当前工作分支。

要求：

1. `legacy/pre-rewrite` 精确指向旧远端默认分支 tip；
2. push 成功；
3. fetch 再次验证 remote legacy ref；
4. 记录 commit SHA。

再创建 annotated tag：

`legacy-pre-rewrite`

指向同一个旧 commit。

push tag。

如果 tag 已存在：
验证它是否已经正确指向该旧 commit；
不要覆盖不明 tag。

最终记录：

```text
LEGACY_BRANCH=legacy/pre-rewrite
LEGACY_COMMIT=<sha>
LEGACY_BRANCH_PUSH=SUCCESS
LEGACY_TAG=legacy-pre-rewrite
LEGACY_TAG_PUSH=SUCCESS
```

旧 Morn 到此安全封存。

---

# 第二十五阶段：推送当前全新 Morn 到验证分支

不要立即替换 main。

先：

```bash
git push -u origin HEAD:morn-v1
```

如果 `morn-v1` 已存在：

- fetch；
- 判断是否就是当前新工程的早期 push；
- 若是同一新历史，正常 fast-forward push；
- 如果是未知/冲突历史，不覆盖，使用安全临时分支名并记录 blocker/decision。

push 后：

```bash
git fetch origin --prune
```

验证：

`origin/morn-v1` 指向 `LOCAL_FINAL_COMMIT`。

记录：

```text
NEW_MORN_BRANCH=morn-v1
NEW_MORN_BRANCH_COMMIT=<sha>
NEW_MORN_BRANCH_PUSH=SUCCESS
```

---

# 第二十六阶段：远端新版本安全检查

在覆盖 main 前，确认：

- 当前 final commit 已推到 morn-v1；
- legacy branch/tag 已成功；
- 没有 secrets/local data；
- CI workflow 配置存在；
- 当前本地 final tests green；
- Git worktree clean；
- remote old default commit 与上次 fetch 一致。

重新：

```bash
git fetch origin --prune
```

记录当前 `origin/main`/旧 default SHA。

如果旧 main 在 legacy 归档之后被别人更新：
不要覆盖。
记录：

`REMOTE_CHANGED_AFTER_BACKUP`

需要重新归档/重新评估。

---

# 第二十七阶段：当前全新 Morn 接管 main

仅在：

```text
legacy backup verified
+
morn-v1 verified
+
local final tests PASS
+
git clean
+
remote main unchanged after fresh fetch
```

时进行。

因为历史完全无关，不 merge。

如果远端正式主分支应为 `main`：

```bash
git push --force-with-lease origin HEAD:main
```

必须使用 `--force-with-lease`。

禁止：

```bash
git push --force
```

如果 branch protection 阻止：
不要绕过。
记录：

`EXTERNAL_GITHUB_BRANCH_PROTECTION`

并在 final report 给出需要用户在 GitHub UI 做的唯一调整。

如果成功：

重新 fetch，并验证：

```text
origin/main == LOCAL_FINAL_COMMIT
```

记录：

```text
NEW_MAIN_TAKEOVER=SUCCESS
REMOTE_MAIN_COMMIT=<sha>
```

---

# 第二十八阶段：默认分支与版本标签

如果旧仓库原默认分支不是 `main`：
不要擅自通过 Git 命令假设 GitHub 默认分支已经切换。

如果 `gh` CLI 已安装且已认证，可在不违反 repo policy 的情况下检查/更新 repository default branch；
否则记录：

`GITHUB_DEFAULT_BRANCH_UI_CHECK_REQUIRED`

但这不影响代码 push 完成。

版本标签：

如果仓库现有版本策略明确，且没有冲突：

```text
v1.0.0-rc.1
```

创建 annotated tag 指向当前新 main，并 push。

如果版本策略不明确：
不要乱打 tag；
在 final report 中写推荐 tag。

---

# 第二十九阶段：Post-push 验证

执行：

```bash
git fetch origin --prune
git status
git log --decorate --oneline -10
```

验证：

- current branch clean；
- origin/main 指向 final commit；
- legacy/pre-rewrite 仍指向旧 commit；
- legacy tag 未改变；
- morn-v1 指向 final commit；
- no accidental merge commit between old/new histories。

如 `gh` CLI 可用且已登录：
检查 Actions 最近 workflow run。
失败则分析可本地修的 workflow 配置；
如果只是 runner/quota/account issue，记录 external blocker。

---

# 第三十阶段：最终报告

生成：

`reports/morn_v1_ga_final_report.md`

必须包含：

```text
STARTING_COMMIT=
LOCAL_FINAL_COMMIT=

MUST_FIX_NOW=0
ALL_LOCAL_TESTS=PASS
ALL_APPLICABLE_UI_TESTS=PASS
GIT_STATUS=CLEAN

REMOTE_URL=
REMOTE_OLD_DEFAULT=
OLD_REMOTE_COMMIT=

LEGACY_BRANCH=legacy/pre-rewrite
LEGACY_COMMIT=
LEGACY_BRANCH_PUSH=SUCCESS
LEGACY_TAG_PUSH=SUCCESS

NEW_MORN_BRANCH=morn-v1
NEW_MORN_BRANCH_PUSH=SUCCESS
NEW_MORN_BRANCH_COMMIT=

NEW_MAIN_TAKEOVER=SUCCESS|EXTERNAL_BLOCKED
REMOTE_MAIN_COMMIT=

MORN_V1_GA_LOCAL_COMPLETE=YES
GITHUB_MIGRATION_COMPLETE=YES|EXTERNAL_BLOCKED
```

还要包括：

- reality audit
- backlog audit
- architecture audit
- code quality changes
- backend/API/migration
- distributed proof
- provider/runtime/connector conformance
- SDK/plugin/domain conformance
- Workbench
- Studio
- Console
- Hub
- security
- chaos
- performance
- test matrix
- docs
- CI/CD
- packaging
- logical commits
- GitHub migration steps
- remaining external blockers
- deferred instance-layer work

---

# 允许停止的唯一条件

只有：

- GitHub auth/permission
- GitHub branch protection
- remote URL无法可靠确定
- network
- 真实 Secret
- 许可证
- 真实外部系统/硬件
- 不可逆业务操作
- 数据不可恢复风险

出现外部 blocker：
记录后继续所有其他任务。

普通编译、测试、lint、UI、migration、依赖、merge conflict in local ref setup、CI YAML、Git ignore、脚本问题，都自己处理。

---

# 最终行为要求

不要只输出计划。

不要反复问我是否继续。

不要 merge unrelated histories。

不要删除旧 GitHub 历史。

不要普通 force push。

不要伪造 green。

现在从第一阶段开始持续执行，直到：

```text
MUST_FIX_NOW=0
ALL_LOCAL_TESTS=PASS
MORN_V1_GA_LOCAL_COMPLETE=YES
LEGACY_GITHUB_BACKUP=SUCCESS
NEW_MORN_BRANCH_PUSH=SUCCESS
NEW_MAIN_TAKEOVER=SUCCESS
```

或者远端最终一步只剩真实 GitHub external blocker。
