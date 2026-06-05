# Morn 项目文档 — 任务清单

## 核心准则 (编程原则)

### 14条核心准则

1. **Think Before Coding** — 阅读整个任务文件再动手，理解项目结构和代码风格
2. **The Code Works** — 每次修改后确保 `cargo build` 通过，`cargo test` 全绿
3. **Small Batches** — 每写一个文档文件就检查一次，不堆砌批量文件
4. **No Dead Code** — 不添加未使用的代码或注释
5. **Single Source of Truth** — 文档内容必须与实际代码一致，不要写代码里不存在的伪API
6. **Test the Paths** — 所有 git 操作（add/commit/push）实际执行，不模拟
7. **Fail Fast** — 遇到错误立即停止并输出错误信息，不继续执行后面的任务
8. **Leave It Better** — 文档格式统一，中英文混排加空格，markdown 规范
9. **Never Guess the Stack** — 从 src/ 下的实际代码中提取 API 和架构信息
10. **Read Before You Write** — 每个模块先读对应的源文件再写文档
11. **Prefer Friction Logs** — 记录构建/测试中遇到的问题
12. **Respect the Dependency** — 文档间互相引用时用相对链接
13. **No Ambiguous Names** — 文档标题清晰不含糊
14. **Document Decisions, Not Drama** — 写开发者需要的技术文档，不是设计哲学

### 低耦合3条

1. **每个文档独立可读** — 不依赖其他文档作为前置阅读条件
2. **避免重复** — 公共信息（如安装步骤）只在一个文件写，其他用链接引用
3. **接口稳定** — 描述公共 API 时只写 pub 接口签名，不写内部实现细节

### 执行规则

- 严格按照任务顺序执行
- 每个任务完成后检查 `cargo build` 是否还正常（只编译，不需要运行二进制）
- 所有文件以 `~/morn-desktop/` 为根目录
- 最终 git add/commit/push 到 main 分支

---

## 任务列表

### 任务1: README.md — 重构

重构根目录的 README.md，使其成为专业开源项目的门面。

要求：
- 顶部徽章行（build passing, license MIT, Rust version, 用 shields.io 的徽章语法，不请求网络只加文字链接）
- 一句话标语：Morn — 你的桌面 AI 创作系统
- 功能概览（三台 + 四层组合 + 多 Agent 团队 + 市场）
- 快速开始（clone → 构建 → 运行，3-5步）
- CLI 命令一览表
- 项目结构树（保留现有）
- 技术栈
- 三阶段路线图状态（全部 ✅ 完成）
- 底部指向 CONTRIBUTING.md 和 LICENSE

注意：设计文档不推 GitHub，README 中不引用设计文档。

位置：`~/morn-desktop/README.md`

### 任务2: CONTRIBUTING.md — 新建

写贡献指南。

要求：
- Issue 提交流程（Bug / Feature / RFC 模板）
- PR 流程（fork → branch → commit → PR → review → merge）
- 编码规范（Rust fmt、clippy、命名约定）
- 测试要求（cargo test 全绿）
- 文档要求（新模块必须有 Rustdoc 注释）
- 行为准则（Contributor Covenant）

位置：`~/morn-desktop/CONTRIBUTING.md`

### 任务3: CHANGELOG.md — 新建

初始化变更日志。

格式：Keep a Changelog 格式。
初始版本 v0.1.0 (2026-05-30)：
- Added: Phase 0 骨架（COO Supervisor、CLI 通道、ChatAgent）
- Added: Phase 1 组件体系（Tool/Knowledge/Skill/Persona/Memory/Model）
- Added: Phase 1 渠道适配（Telegram、企微、钉钉、飞书、REST API）
- Added: Phase 1 创作台（StudioManager）
- Added: Phase 2+ 多 Agent 团队（7 种协作模式 + 工作流引擎）
- Added: Phase 2+ 管理台（Console — 成本/治理/监控）
- Added: Phase 2+ 电脑操控（桌面/文件/浏览器/应用/系统/感知）
- Added: Phase 2+ 市场（Marketplace — 上架/下载/评分/许可证）
- Added: Phase 2+ 四层安全宪法（L1-L4 安全策略 + Dual-LLM）
- Added: Rust + Tauri + React 技术栈，MIT 许可证

位置：`~/morn-desktop/CHANGELOG.md`

### 任务4: docs/architecture.md — 新建

写架构总览文档。

先读以下源文件理解架构：
- src/lib.rs（模块声明）
- src/core/supervisor.rs（COO 主管）
- src/core/component.rs（Component trait）
- src/core/engine.rs（执行引擎）
- src/core/storage.rs（SQLite 存储）

内容：
- 四层架构图（接入层 → COO → 组件体系 → 存储层）
- COO 主管的 6 级决策树流程
- 组件 trait 体系
- 安全宪法四层模型
- 数据流（用户输入 → 意图解析 → 计划 → 执行 → 输出）
- 事件总线

注意：开发者视角的架构说明，不含设计哲学或竞品分析。

位置：`~/morn-desktop/docs/architecture.md`

### 任务5: docs/setup.md — 新建

写详细安装指南。

先读：
- src/main.rs（入口，理解环境变量和启动方式）
- Cargo.toml（依赖和技术栈）

内容：
- 系统要求（Windows 10+ for Tauri，Linux for CLI）
- 依赖安装（Rust toolchain 1.75+、Node.js 18+、WebView2）
- 环境变量配置（MORN_API_KEY）
- 三种构建方式（CLI / 桌面端 / 前端）
- 故障排查（常见错误及解决）

位置：`~/morn-desktop/docs/setup.md`

### 任务6: docs/development.md — 新建

写开发指南。

内容：
- 项目结构详解（每个目录的职责）
- 开发流程（改代码 → 编译 → 测试 → 提交）
- 测试体系（cargo test 有哪些测试分类）
- Rustdoc 说明
- 代码风格（rustfmt、clippy 配置）

位置：`~/morn-desktop/docs/development.md`

### 任务7: docs/coo.md — 新建

写 COO Supervisor 详细文档。

先读：
- src/core/supervisor.rs（主逻辑 + 6 级决策树）
- src/core/registry.rs（能力注册中心）
- src/core/trust_evaluator.rs（信任评分）

内容：
- COO 是什么（主管大脑）
- 6 级决策树详解（L1-L6 每级的含义、触发条件、成本）
- CooMode（Active/Safe/Auto 三种模式）
- 工作流引擎（WorkflowTemplate 和内建模板）
- 信任评分机制

位置：`~/morn-desktop/docs/coo.md`

### 任务8: docs/components.md — 新建

写组件体系文档。

先读：
- src/core/component.rs（Component trait + Port + Data）
- src/component/ 目录下所有文件

内容：
- Component trait 定义
- 六类组件：Tool / Knowledge / Skill / Persona / Memory / Model
- Port 和 Data 模型
- AgentAssembler（组装器）
- 组件的生命周期（init → run → pause → stop）

位置：`~/morn-desktop/docs/components.md`

### 任务9: docs/studio.md — 新建

写创作台文档。

先读：
- src/studio/manager.rs
- src/studio/publisher.rs
- src/studio/tester.rs

内容：
- Studio 创作台是什么
- 组件管理（CRUD 操作）
- Agent 组装流程
- 测试与发布

位置：`~/morn-desktop/docs/studio.md`

### 任务10: docs/console.md — 新建

写管理台文档。

先读：
- src/console/mod.rs
- src/console/cost.rs
- src/console/governance.rs

内容：
- Console 管理台是什么
- 成本监控
- 治理策略
- 系统健康检查

位置：`~/morn-desktop/docs/console.md`

### 任务11: docs/channels.md — 新建

写渠道适配文档。

先读：
- src/channel/adapter.rs（统一适配器）
- src/channel/ 目录下所有通道文件

内容：
- 统一消息适配器（ChannelAdapter + ChannelMessage）
- 已支持的渠道（CLI / Telegram / 企微 / 钉钉 / 飞书 / REST API / QQ Bot / 微信小程序 / 微信公众号 / Webhook / SMTP / Desktop）
- 如何添加新渠道

位置：`~/morn-desktop/docs/channels.md`

### 任务12: docs/market.md — 新建

写市场文档。

先读：
- src/market/marketplace.rs

内容：
- Marketplace 是什么
- Listing（上架商品）
- Transaction（交易）
- License（许可证管理）
- 评分与下载

位置：`~/morn-desktop/docs/market.md`

### 任务13: docs/computer-control.md — 新建

写电脑操控文档。

先读：
- src/computer/ 目录下所有文件

内容：
- 电脑操控概览（桌面操作 / 文件系统 / 浏览器控制 / 应用管理 / 系统管理 / 感知）
- 安全分级体系
- 当前状态（模拟阶段，所有操作返回 [simulated]）

位置：`~/morn-desktop/docs/computer-control.md`

### 任务14: Git 提交

```bash
cd ~/morn-desktop
git add -A
git commit -m "docs: add full project documentation suite

- README.md: restructured with badges, features, quick start
- CONTRIBUTING.md: contribution guide with PR/issue workflow
- CHANGELOG.md: v0.1.0 initial release changelog
- docs/: 10 developer documents (architecture, setup, development,
  coo, components, studio, console, channels, market, computer-control)"
git push origin main
```

执行后输出 git log 确认。
