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

### KF-003 — 浏览器级 UI runtime QA 未自动化（deferred）
Status: Deferred

Reproduction:
ACCEPTANCE A10 要求“关键页面无明显 overflow/console error”。

Expected: headless 浏览器 smoke 捕获 console error / overflow。

Actual: 前端 typecheck/lint/test/build 全通过，`dist/` 可构建；未配置 headless 浏览器（Playwright 等）做运行时 console 检查。

Root cause:
当前环境无现成 headless 浏览器测试夹具；配置它属于下一步工程。

Fix/Decision:
前端组件已有 loading/empty/error/blocked 状态；浏览器级 smoke 列为下一步 P1 项（Playwright + 真实 server）。

Test added:
无（不伪造）。

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