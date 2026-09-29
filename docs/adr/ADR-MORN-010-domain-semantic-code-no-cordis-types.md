# ADR-MORN-010：Domain semantic code 不绑定 Cordis implementation types

**状态：** Accepted（2026-09-28，依据 v11.5 canonical §8.2）
**上下文：** `packages/spec` / domain semantic code 若直接 import Cordis/DSH/Pi/Temporal 等实现类型，会形成锁定，Provider 替换要求重写 Domain。
**决定：** 语义/控制平面代码只用 Morn contract 类型；实现耦合收敛在 adapter/provider 层（`src/runtime/cordis/`、`src/providers/`）。依赖方向：work/control 不依赖 runtime/cordis。
**证据：** `tests/work_module_isolation.rs`（扫描 src/work + src/control 全部文件，import-path 级检测 cordis::/deepseek/dsh::/temporal::/kafka::/sled:: = 0 violations）。
