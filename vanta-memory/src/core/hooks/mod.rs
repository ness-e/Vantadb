//! Automatic capture hooks that plug the memory pipeline into a host
//! conversation stream (OpenClaw-style, host-neutral).
//!
//! Filled by MEM-09 (capture) and MEM-18 (recall) — see plan file.

pub mod auto_capture;
pub mod auto_recall;

pub use auto_capture::{AutoCaptureConfig, AutoCaptureHook, AutoCaptureResult, RawMessage};
pub use auto_recall::{
    perform_auto_recall, AutoRecallParams, RecallConfig, RecallError, RecallMode, RecallResult,
    RecalledMemory, MEMORY_TOOLS_GUIDE,
};
