// ERR-CORE-02: keep clippy deny on prod code; tests may `unwrap`/`expect`
// freely (same pattern as vantadb/src/lib.rs).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Vanta proxy library: transparent forwarding of LLM wire protocols.
//!
//! Protocols supported (verbatim forward, no business logic — see MEM-26):
//! - OpenAI Chat Completions: `/{agent}/{spaceId}/v1/chat/completions`
//! - Anthropic Messages:      `/{agent}/{spaceId}/v1/messages`
//! - Generic Responses subset: `/v1/responses` (no Codex/WorkBuddy adapters)

pub mod auth;
pub mod capture;
pub mod config;
pub mod error;
pub mod forward;
pub mod handlers;
pub mod inject;
pub mod langfuse;
pub mod mem_command;
pub mod memory_tools;
pub mod rate_limit;
pub mod report;
pub mod server;
pub mod session;
pub mod sse_intercept;
pub mod writeback;
