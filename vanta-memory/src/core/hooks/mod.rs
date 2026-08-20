//! Automatic capture hooks that plug the memory pipeline into a host
//! conversation stream (OpenClaw-style, host-neutral).
//!
//! Filled by MEM-09 (see plan file).

pub mod auto_capture;

pub use auto_capture::{AutoCaptureConfig, AutoCaptureHook, AutoCaptureResult, RawMessage};
