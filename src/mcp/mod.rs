//! mcp — Model Context Protocol integration for external tool execution.
//! MCP (Model Context Protocol) 模块 — 提供组件端口与 MCP 工具之间的双向转换能力。
//! 子模块 `adapter` 实现了转换逻辑，`tools` 提供了内置 MCP 工具实现。
//! MCP (Model Context Protocol) module — bidirectional conversion between component ports and MCP tools.
//! Submodules: `adapter` for conversion logic, `tools` for built-in MCP tool implementations.

pub mod adapter;
pub mod http;
pub mod manager;
pub mod server;
pub mod stdio;
pub mod tools;
