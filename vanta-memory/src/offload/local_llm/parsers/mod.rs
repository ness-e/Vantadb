//! Parsers for local-LLM JSON output (port of TDAM
//! `offload/local-llm/parsers/`).

/// Generic JSON extraction/repair utilities for raw LLM responses.
pub mod json_utils;

/// L1 extraction response → typed scene segments.
pub mod l1_parser;
