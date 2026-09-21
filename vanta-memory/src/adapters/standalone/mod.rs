//! Standalone (host-less) adapters.
//!
//! The [`StandaloneLlmRunner`] calls an external OpenAI-compatible endpoint
//! directly — no host integration required.
//!
//! [`StandaloneLlmRunner`]: llm_runner::StandaloneLlmRunner

pub mod llm_runner;
