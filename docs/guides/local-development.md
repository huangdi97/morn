# Local Development Guide（Morn v11.5-R1）

## 环境

- Rust（MSRV 1.85；本机 rustc 1.97.1）
- MSVC：会话级 `INCLUDE`/`LIB`（MSVC 14.50.35717 + Windows SDK 10.0.26100.0）
- Node 20+（web/：Vite + vitest + playwright）
- SQLite（rusqlite bundled——无外部 DB 依赖）

## 常用命令

```bash
cargo test --workspace              # 全量 Rust 测试（~1434）
cargo fmt --check                   # 格式检查
cargo clippy --lib -- -D warnings   # lint（CI 标准）
cd web && npm run test              # vitest
cd web && npm run build             # UI build
```

## 构建环境脚本

Windows 下 cargo 需要 MSVC 环境。会话级：

```powershell
$msvcRoot = "D:\Code\Visual Studio\Visual Studio\VC\Tools\MSVC\14.50.35717"
$sdkRoot = "C:\Program Files (x86)\Windows Kits\10"
$env:INCLUDE = "$msvcRoot\include;$sdkRoot\Include\10.0.26100.0\ucrt;$sdkRoot\Include\10.0.26100.0\shared;$sdkRoot\Include\10.0.26100.0\um"
$env:LIB = "$msvcRoot\lib\x64;$sdkRoot\Lib\10.0.26100.0\ucrt\x64;$sdkRoot\Lib\10.0.26100.0\um\x64"
```

## 工程规范

- 文件 300 行硬限（250 WARNING）；函数 ≤40 行（复杂 domain ≤60）
- 禁止 `except Exception: pass` / silent failure / unjustified any
- 注释解释 WHY/INVARIANT/SAFETY，不解释显而易见的 WHAT
- TODO 必须带 issue/backlog ref
- 生产写默认 FORBIDDEN（REAL_PRODUCTION_WRITE = NO）

## 仓库结构

见 `docs/canonical/CURRENT.md`（canonical 指针）与 `ARCHITECTURE.md`（六 Plane 映射）。
