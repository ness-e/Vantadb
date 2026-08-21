//! Wire protocol handlers. Each handler is a thin wrapper over the shared
//! verbatim forwarder — no business logic (that lands in MEM-26).

pub mod anthropic;
pub mod openai;
pub mod responses;
