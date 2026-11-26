//! Memory-prompt layer: custom strategy prompts (types, resolver, composer).
//! MEM-18 — see plan file.

pub mod composer;
pub mod resolver;
pub mod types;

pub use composer::{compose_memory_system_prompt, escape_closing_tags};
pub use resolver::{resolve_memory_prompt, MemoryPromptStore, ResolveTarget};
pub use types::{
    build_memory_prompt_setting_id, MemoryPromptError, MemoryPromptLayer, MemoryPromptRecord,
    MemoryPromptSettingRecord, MemoryPromptSource, PromptStatus, ResolvedMemoryPrompt,
};
