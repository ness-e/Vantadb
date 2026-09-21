//! Host adapters: standalone vs. host (OpenClaw-style) LLM runners.
//!
//! Both implement the host-neutral [`crate::core::abstractions::LlmRunner`]:
//! - `standalone` — direct OpenAI-compatible HTTP, no host (real transport
//!   under the `llm-driver` feature).
//! - `openclaw` — delegates to an OpenClaw-style host via the [`OpenClawHost`]
//!   port (no dependency on real OpenClaw).
//! - `mock` (feature `mock`) — deterministic fake for tests (D19).
//!
//! [`OpenClawHost`]: openclaw::llm_runner::OpenClawHost

pub mod openclaw;
pub mod standalone;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "mock")]
pub use mock::{MockLlmRunner, MockScript};
