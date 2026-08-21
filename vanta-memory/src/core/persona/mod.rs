//! L3 persona: trigger heuristics + first/incremental generation (MEM-15,
//! F4).
//!
//! The persona is the distilled L3 document the host prepends to its system
//! prompt. [`persona_trigger`] decides WHEN to regenerate (LLM-free
//! heuristics, TDAM priorities P1-P4); [`persona_generator`] decides HOW —
//! mode First/Incremental derived from the store, LLM output validated,
//! escaped and persisted under the `persona/<session>` namespace.
//!
//! Source: `docs/research/tdam/02-scene-persona.md` §26, §41 (TDAM
//! `persona-generator.ts`, `persona-trigger.ts`).

pub mod persona_generator;
pub mod persona_trigger;

pub use persona_generator::{
    escape_xml_tags, generate_persona, get_persona, has_persona_body, persona_namespace,
    PersonaError, PersonaGenerateParams, PersonaGenerationResult, PersonaRecord, PERSONA_KEY,
};
pub use persona_trigger::{evaluate_persona_trigger, PersonaTriggerInput, TriggerResult};
