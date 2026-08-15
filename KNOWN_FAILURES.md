# KNOWN_FAILURES.md

记录已知失败/缺陷。普通 bug 不要写成 blocker 后停止，继续修。

### KF-001 — vitest/esbuild `spawn EPERM`（沙箱内）
Status: Fixed-with-workaround

Reproduction:
`npm test`（在 Codex sandbox 内）→ `Error: spawn EPERM`（esbuild service spawn 被沙箱拒绝）。

Expected: 测试在沙箱内直接运行。

Actual: 沙箱阻止 esbuild 子进程；在获得审批的运行环境下（`require_escalated`）`npm test` 通过（2 passed）。

Root cause:
环境 sandbox 对子进程 spawn 的限制（已知环境边界），非产品缺陷。

Fix/Decision:
`scripts/run_all.ps1` 的 frontend test 步骤在审批运行环境下执行；沙箱内单独运行 frontend test 会失败属预期环境边界。

Test added:
`frontend/src/App.test.ts`（2 用例）——在审批环境通过。

### KF-002 — Tauri 桌面壳（已构建并验证）
Status: Fixed

Reproduction:
ACCEPTANCE A1 提到 “Tauri app 可启动或至少 desktop build 通过”。

Expected: Tauri desktop build 通过。

Actual: `src-tauri`（tauri v2 + WebView2）已添加；`cargo build -p morn-desktop` 通过（纳入 run_all）；二进制在沙箱外可启动（WebView2 runtime 151.0.4129.78）。

Root cause:
先打通同一真实后端的四个表面与全量验证；随后补齐 Tauri 壳。

Fix/Decision:
Tauri 壳作为边界层，不堆业务逻辑；复用同一 HTTP/domain API。

Test added:
`run_all.ps1` 新增 `tauri desktop build` 步骤；沙箱外启动 smoke（进程存活 6s）。

### KF-005 — 沙箱 token 拒绝 GUI/WebView2 启动
Status: Open (environment boundary)

Reproduction:
在 Codex sandbox 内运行 `target\debug\morn-desktop.exe`：
```text
thread 'main' panicked at tauri-2.11.5/src/app.rs:1425:11:
Failed to setup app: 拒绝访问。 (os error 5)
```

Expected: 桌面窗口正常启动。

Actual: 沙箱 token 下 WebView2/COM 初始化返回 “Access is denied (os error 5)”；在审批（非沙箱）环境下同一二进制启动正常（进程存活 6s），WebView2 runtime 151.0.4129.78 已安装。

Root cause:
沙箱对 GUI/COM 初始化的权限限制（环境边界），非产品缺陷。

Fix/Decision:
GUI 启动验证在非沙箱环境执行；构建验证（`cargo build -p morn-desktop`）在 run_all 中覆盖。

Test added:
沙箱外启动 smoke（见 KF-002）。

### KF-003 — 浏览器级 UI runtime QA（已自动化）
Status: Fixed

Reproduction:
ACCEPTANCE A10 要求“关键页面无明显 overflow/console error”。

Expected: headless 浏览器 smoke 捕获 console error / overflow。

Actual: 已加入 Playwright（chromium 151.0.7922.34）UI smoke：`frontend/scripts/ui_smoke.mjs` 加载
/workbench、/studio、/console、/hub 四个表面并点击 BioLab E2E 按钮，断言 0 console error / 0 pageerror；
`npm run ui-smoke` 通过（exit 0）。已纳入 `scripts/run_all.ps1`（playwright 浏览器存在时执行）。

Root cause:
早期无 headless 浏览器测试夹具。

Fix/Decision:
UI smoke 由 `run_all.ps1` 覆盖；Playwright 为 devDependency，浏览器需 `npx playwright install chromium`。

Test added:
`frontend/scripts/ui_smoke.mjs`（4 surfaces + BioLab E2E）。

### KF-004 — 早期文件带 UTF-8 BOM 导致严格 JSON 解析失败（fixed）
Status: Fixed

Reproduction:
`npm test` 曾报 `SyntaxError: Unexpected token '\uFEFF' ... not valid JSON`（package.json）。

Expected: JSON 正常解析。

Actual: PowerShell 5.1 `Set-Content -Encoding UTF8` 写入 BOM。

Root cause:
写文件编码选择。

Fix/Decision:
全仓库文本文件去除 BOM（UTF-8 no-BOM 重写）；后续写文件统一用 no-BOM UTF-8。

Test added:
`npm test` / `npm run build` / Tauri build（tauri.conf.json 严格 JSON 解析）均通过。
### KF-006 — UI smoke 的 innerText 大小写问题（fixed）
Status: Fixed

Reproduction:
`frontend/scripts/ui_smoke.mjs` 断言 `body.includes("Durable Work Runtime")` 失败；实际卡片已渲染。

Expected: 断言通过。

Actual: CSS `text-transform: uppercase` 使 `innerText` 返回 "DURABLE WORK RUNTIME (V0.2)"，大小写不匹配。

Root cause:
断言使用混合大小写字符串与 CSS 变换后的 innerText 比较。

Fix/Decision:
smoke 断言改为 toLowerCase() 双向比较。

Test added:
`frontend/scripts/ui_smoke.mjs`（case-insensitive 断言）。
### KF-007 — Playwright strict-mode 按钮冲突（fixed）
Status: Fixed

Reproduction:
`ui_smoke.mjs` 中 `/Shadow Compare/i` 同时匹配 "Shadow Compare" 与 "Shadow Compare Baseline vs Candidate"。

Expected: 只点击目标按钮。

Actual: strict mode violation。

Fix/Decision:
Goal 2 按钮改为 exact:true；点击循环透传 exact。

Test added:
`frontend/scripts/ui_smoke.mjs`。
### KF-008 — PowerShell 保留变量 `$pid` 干扰 API smoke（fixed）
Status: Fixed-with-workaround

Reproduction:
API smoke 脚本用 `$pid` 保存 prediction id，触发 “Cannot overwrite variable PID” 错误。

Expected: 脚本正常运行。

Actual: PowerShell 将 `$pid` 视为只读自动变量。

Fix/Decision:
改用非保留变量名（`$predId`）。

Test added:
手工 smoke；产品代码无影响。### KF-009 — demo smoke 404 + 过期 world_objects 断言（G5 中立化回归，fixed）
Status: Fixed (2026-08-16)

Reproduction:
`scripts/run_all.ps1` demo smoke 在 G5 领域中立后失败：
1. 默认 feature 构建的 server 无 `/api/biolab/run` 路由（`morn-app` default features = []，
   BioLab 路由在 `domain-biolab` feature 下注册）→ 404 Not Found；
2. 改用 all-features server 后，workbench `world_objects >= 3` 断言失败——中立化后共享 world
   不再默认预置 3+ 对象（BioLab 对象在其自身 WorldService 内）。

Expected: demo smoke 验证 server + BioLab E2E 真实可用。

Actual: 上述两步均失败。

Root cause:
G5 M1-M2 领域中立后 `morn-app` 默认零领域，demo smoke 未同步更新（构建 feature 与断言模型）。

Fix/Decision:
- demo smoke 用 `--all-features` 构建 server（与 UI smoke 一致）；
- flow 改为 POST `/api/demo/bootstrap`（零领域通用种子）→ POST `/api/biolab/run`（领域包 E2E）→
  校验 workbench world_objects >= 1 与 work_packages >= 1（对齐中立化架构；见 DECISIONS D-026）；
- `crates/morn-app/src/app.rs` 将 `WorkspaceId` import/binding 置于 `domain-biolab` 下，消除
  零领域构建的 unused-variable warning。

Test added:
`scripts/run_all.ps1` 全绿（exit 0）；手工 smoke 复现通过：
health=ok / bootstrap objects=1 work_packages=1 / biolab all_ok=True steps=7 / workbench objects=1 work_packages=1。

### KF-010 — Core 编译器领域启发式（M1 残留，fixed）
Status: Fixed (2026-08-16)

Reproduction:
`morn-foundry::compiler` decompose_nodes 用 `goal.contains("claim") || goal.contains("hypothesis") ||
request.domain.contains("biolab")` 触发 evidence/analysis/review/approval/release 模板——Core 认识领域词，
违反 Domain-neutral（M1 审计 CORE_BUG）。

Fix/Decision:
改为通用 governed-deliverable 信号（goal 含 review/approv，或约束声明 governed release）；
领域名单独不再改变分解；release 措辞通用化（见 DECISIONS D-027）。

Test added:
`governed_template_is_domain_neutral`（morn-foundry）；`check_domain_boundary.ps1` 新增领域词启发式守卫。

### KF-011 — Core UI 硬编码 BioLab 卡片/按钮（M14 残留，fixed）
Status: Fixed (2026-08-16)

Reproduction:
Workbench.tsx 无条件渲染 "Run BioLab E2E" / "BioLab Dream Factory" 卡片并直接调用 /biolab/*；
Studio.tsx 默认 domain="biolab"——零 Domain 下 UI 仍显示领域入口（M14 违约）。

Fix/Decision:
`/api/workbench` 广告 `domain_packs`；前端 `biolabEnabled()` 数据驱动门控（见 DECISIONS D-028）；
Studio 默认通用；compiler/run 默认 domain="generic"。

Test added:
`frontend/src/Workbench.gating.test.ts`（2 tests）；zero-domain server 实测 domain_packs=[] 且 /api/biolab/run 404。
