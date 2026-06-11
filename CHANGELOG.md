# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.0] - 2026-06-11

### Added
- 80 项功能补齐：渠道真实化（TG轮询/企微/钉钉/推送/小程序/浏览器扩展）
- ModelRouter 生产化（主线接入/Ollama API/动态provider/Hybrid增强）
- COO 决策树完整（6步理解/权重决策/VLM真实/自动升降级）
- Studio 创作台（Canvas拖拽缩放快照/16节点/7协作模式/MCP全工具/手机画布）
- Dual-LLM 安全管线 + E2E加密 + Security授权链
- 管理台（成本超限决策/拓扑拖拽/趋势图表/仪表盘告警）
- 基础设施（config.toml/daemon模式/Web独立化/1键启动脚本/CI）
- 跨设备 A2A 发现协议 + 同步

### Changed
- 代码质量：统一 MornError、retry 指数退避、storage 持久化、macOS 支持
- 架构：EventBus 合并（SimpleEventBus 统一）、app_ops/desktop_ops 接线到 CLI
- 依赖：reqwest/serde_json 永久依赖、删除 serde_yaml、删除 office feature
- 文档：10 个 docs/ 文件同步更新到当前代码状态

### Fixed
- executor.rs match 遗漏（16 节点类型覆盖不全）
- onboarding 测试路径不一致（改为 $HOME/.config）
- 10 处硬编码 /tmp/ 路径改为 temp_dir()
- 4 处静默吞错误改为 tracing::warn
- clippy 12 个 warning 修复至 5 个
- 修复 5 处 clippy warning 至 6 个预留、清理 15 处静默吞错误、27 个文件补模块文档、tower-http 改为 optional

## [0.1.0] - 2026-05-30

### Added

- Phase 0 骨架：COO Supervisor、CLI 通道、ChatAgent
- Phase 1 组件体系：Tool / Knowledge / Skill / Persona / Memory / Model
- Phase 1 渠道适配：Telegram、企业微信、钉钉、飞书、REST API
- Phase 1 创作台：StudioManager
- Phase 2+ 多 Agent 团队：7 种协作模式 + 工作流引擎
- Phase 2+ 管理台：Console — 成本 / 治理 / 监控
- Phase 2+ 电脑操控：桌面 / 文件 / 浏览器 / 应用 / 系统 / 感知
- Phase 2+ 市场：Marketplace — 上架 / 下载 / 评分 / 许可证
- Phase 2+ 四层安全宪法：L1-L4 安全策略 + Dual-LLM
- Rust + Tauri + React 技术栈，MIT 许可证