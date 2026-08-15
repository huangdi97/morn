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