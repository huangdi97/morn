# GAP 审计执行计划

## 阶段划分

### Phase 1 — 独立无依赖（3路并行）

| # | 任务 | 操作 | 执行工具 | 预估 |
|---|------|------|----------|------|
| 1.1 | clippy fix（12个warnings） | 函数重命名+type alias+args struct | Codex CLI | 15min |
| 1.2 | computer/加测试（7文件） | 追加 `#[cfg(test)] mod tests` | Codex CLI | 20min |
| 1.3 | dead_code审查（28处） | 加注释说明保留理由 | Codex CLI | 15min |

**全部只改注释 / 追加测试 / 修 warning，不改变外部行为。**

### Phase 2 — 大文件拆分（串行，需Phase 1产出）

| # | 文件 | 行数 | 拆分方案 | 执行工具 |
|---|------|------|----------|----------|
| 2.1 | persona/mod.rs | 730 | → subdir: core.rs + communication.rs + ...（已有预设子模块） | Codex CLI |
| 2.2 | supervisor.rs | 662 | → subdir: delegation.rs + lifecycle.rs + ... | Codex CLI |
| 2.3 | workflow/templates.rs | 659 | → subdir: code_review.rs + deploy.rs + ... | Codex CLI |
| 2.4 | storage/mod.rs | 584 | 已拆过，仅检查是否需要补充 | Codex CLI |
| 2.5 | office_handler/slides.rs | 523 | → subdir: charts.rs + slides_layout.rs | Codex CLI |

**每拆一个跑 `cargo build` + `cargo test` 验证。**

### Phase 3 — 架构/依赖（高风险，后做）

| # | 任务 | 风险 | 前提条件 |
|---|------|------|----------|
| 3.1 | 升级 axum 0.7→0.8 + tower 0.4→0.5 | ⚠️ 路由API破坏变更 | Phase 2 完成 |
| 3.2 | 移除 component→core 反向依赖 | ⚠️ 需引入 trait 接口层 | Phase 2 完成 |
| 3.3 | 更新 README/文档 | 低 | Phase 1-3 完成后 |

### Phase 4 — Design Gap 实现（长期）

GAP 分析中的 25 项未实现 + 28 项部分实现，按 P0→P4 优先级逐项推进，不在本轮范围内。

## 约束条件

- 每个阶段执行前，先写 tasks.md 给老板确认
- 等老板说「出」才执行
- 每个子任务完成后报告结果，不自动推进下一步
- 全部用 Codex CLI 执行代码改动（不用 delegate_task）
