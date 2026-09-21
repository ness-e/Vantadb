//! OpenClaw-style host adapters.
//!
//! The [`OpenClawLlmRunner`] delegates to an [`OpenClawHost`] — the port shape
//! of a host's embedded-agent runtime. No dependency on real OpenClaw.
//!
//! [`OpenClawLlmRunner`]: llm_runner::OpenClawLlmRunner
//! [`OpenClawHost`]: llm_runner::OpenClawHost

pub mod llm_runner;
