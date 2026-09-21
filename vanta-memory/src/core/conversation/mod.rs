//! L0 conversation capture: raw turn recording, idempotent and LLM-free.
//!
//! Filled by MEM-09 (see plan file).

pub mod l0_recorder;

pub use l0_recorder::{L0Capture, L0CaptureResult, L0Error, L0Message, L0Recorder, L0Role};

// Reused by the L1 layer (MEM-11): namespace/key sanitization + clock.
pub(crate) use l0_recorder::{now_ms, sanitize_component, sanitize_key};
