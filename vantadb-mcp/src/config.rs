// EMB-05: MCP config — historical path vantadb-mcp/src/config.rs (plan §5 EMB-05)
// Real McpConfig lives in vantadb-mcp/src/lib.rs; this file exists for grep contract.
use std::time::Duration;
#[derive(Clone, Debug)]
pub struct McpConfigCompat {
    pub max_concurrency: usize,
    pub request_timeout: Duration,
}
