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

### KF-002 — Tauri 桌面壳本轮未构建（deferred）
Status: Deferred

Reproduction:
ACCEPTANCE A1 提到 “Tauri app 可启动或至少 desktop build 通过”。

Expected: 提供 Tauri desktop build。

Actual: 本轮未添加 `src-tauri`；四个产品表面通过同一 axum 后端 + React/Vite web 前端交付（`frontend/` + `crates/morn-app`）。

Root cause:
时间/范围决策（D-007）：先打通同一真实后端的四个表面与全量验证；Tauri 壳需要额外 tauri 依赖树与打包配置，且当前无浏览器级 UI runtime QA 工具链。

Fix/Decision:
不声明 Tauri build 已通过。下一步 P1：在 `morn-app` 之上加 `src-tauri`（tauri v2 + WebView2），复用同一 HTTP/domain API。

Test added:
无（未实现，不伪造）。

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
全仓库文本文件去除 BOM（UTF-8 no-BOM 重写）。

Test added:
`npm test` / `npm run build` 通过。
