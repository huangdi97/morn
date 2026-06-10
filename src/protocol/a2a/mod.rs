//! A2A (Agent-to-Agent) protocol for inter-agent communication.
//!
//! ## Protocol Design
//! Inspired by Google A2A spec and OpenAI MCP concepts:
//! - Each message is wrapped in an `A2AEnvelope` with routing metadata.
//! - Supports three delivery modes: Direct, Relay, Broadcast.
//! - Transport layer abstracts WebSocket and HTTP SSE.
//!
//! ## Message Flow
//! 1. Sender agent creates `A2AMessage` and wraps in `A2AEnvelope`.
//! 2. Router dispatches based on routing mode and recipient capabilities.
//! 3. Recipient agent receives and processes the message.

pub mod protocol;
pub mod router;

pub use protocol::*;
pub use router::*;
