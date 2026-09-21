//! Offload hooks (TDAM `MC/offload/hooks/`).
//!
//! MEM-20 ports only the after-tool-call decision core; the L3 compression
//! machinery of TDAM (`llm-input-l3.ts`, token counters, MMD injection) is
//! out of scope until its F5 consumers exist.

pub mod after_tool_call;
