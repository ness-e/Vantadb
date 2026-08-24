#![warn(missing_docs)]
#![recursion_limit = "512"]

//! VantaDB Model Context Protocol (MCP) Server.
//!
//! This module provides a complete MCP server implementation for VantaDB,
//! exposing tools, resources, and prompts for AI agent integration.

mod axioms;
mod code;
mod config;
mod context;
mod error;
mod handlers;
mod metrics;
mod protocol;
mod scenes;
mod server;
mod skills;
mod validation;
mod wiki;

/// Tuning knobs for the MCP server.
pub use config::McpConfig;
/// MCP error type used across the server.
pub use error::McpError;
/// Handle the `initialize` request, returning protocol version, server info and capabilities.
pub use handlers::initialize::handle_initialize;
/// Get a specific prompt and its arguments.
pub use handlers::prompts::handle_prompts_get;
/// List available prompts.
pub use handlers::prompts::handle_prompts_list;
/// List available resources exposed by the server.
pub use handlers::resources::handle_resources_list;
/// Read the content of a resource by URI.
pub use handlers::resources::handle_resources_read;
/// Call a tool by name with the given arguments.
pub use handlers::tools::handle_tools_call;
/// List available tools.
pub use handlers::tools::handle_tools_list;
/// Run the MCP server over stdin/stdout (JSON-RPC 2.0).
pub use server::run_stdio_server;
/// Lifecycle state + progress of an async wiki build, by run_id (MEM-31).
pub use wiki::ingest_status;
/// Start an async wiki ingest build, returning its run_id immediately (MEM-52).
pub use wiki::start_ingest;
/// LLM-free runner for [`start_ingest`] (P4 degraded mode).
pub use wiki::NoLlm;
