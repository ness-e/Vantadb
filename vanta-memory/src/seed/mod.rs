//! Seed/import: bootstrap a VantaDB memory store with initial skills and a
//! persona document (MEM-39, plan P29 Task 4).
//!
//! Idempotent by content-hash (MEM-06/MEM-17 pattern): re-importing the same
//! seed file never duplicates records — identical skill content is skipped,
//! an unchanged persona is left untouched. Counts report what each run
//! actually did (`created` / `updated` / `unchanged`).
//!
//! Persistence parity:
//! - skills → `skills_extract/<sanitized scope>` with the exact [`StoredSkill`]
//!   payload shape written by MEM-06's sink, so existing readers see them.
//! - persona → `persona/<sanitized session_key>` / key `persona.md` as a
//!   [`PersonaRecord`], readable by [`get_persona`].
//!
//! Note: seeded persona content is operator-provided (trusted file), so it
//! bypasses the LLM-output XML escaping of the generator.

pub mod input;

pub use input::{parse_seed_input, SeedInput, SeedPersona, SeedSkill};

use crate::core::conversation::{now_ms, sanitize_component, sanitize_key};
use crate::core::persona::{get_persona, persona_namespace, PersonaRecord, PERSONA_KEY};
use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::core::skill::conversation_add::sink::StoredSkill;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use thiserror::Error;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};

/// Errors surfaced by the seed/import layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SeedError {
    /// Seed file could not be read.
    #[error("seed file io: {0}")]
    Io(#[from] std::io::Error),
    /// Seed document failed to parse as JSON.
    #[error("seed json: {0}")]
    Json(#[from] serde_json::Error),
    /// Seed document is structurally invalid.
    #[error("seed validation: {0}")]
    Validation(String),
    /// Underlying VantaDB storage error.
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::VantaError),
    /// Persona layer error (read/write of the persona record).
    #[error("persona: {0}")]
    Persona(#[from] crate::core::persona::PersonaError),
}

/// What one import run actually did (MEM-06 `SkillSinkCounts` parity).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SeedCounts {
    /// New records written.
    pub created: usize,
    /// Existing records whose content changed.
    pub updated: usize,
    /// Records skipped because their content hash matched.
    pub unchanged: usize,
}

impl std::fmt::Display for SeedCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "created={}, updated={}, unchanged={}",
            self.created, self.updated, self.unchanged
        )
    }
}

/// Load, validate and import a seed JSON file.
pub fn import_seed_file(
    db: &VantaEmbedded,
    path: &std::path::Path,
) -> Result<SeedCounts, SeedError> {
    let raw = std::fs::read_to_string(path)?;
    import_seed_str(db, &raw)
}

/// Validate and import a seed document from raw JSON text.
pub fn import_seed_str(db: &VantaEmbedded, raw: &str) -> Result<SeedCounts, SeedError> {
    let seed = parse_seed_input(raw)?;
    import_seed(db, &seed)
}

/// Import an already-parsed seed document. Idempotent: replaying the same
/// document returns all-`unchanged` counts without writing.
pub fn import_seed(db: &VantaEmbedded, seed: &SeedInput) -> Result<SeedCounts, SeedError> {
    let mut counts = SeedCounts::default();
    for skill in &seed.skills {
        apply_skill(db, &seed.scope, skill, now_ms(), &mut counts)?;
    }
    if let Some(persona) = &seed.persona {
        apply_persona(db, persona, now_ms(), &mut counts)?;
    }
    Ok(counts)
}

/// `skills_extract/<scope>` — same namespace as the MEM-06 skill sink.
fn skills_namespace(scope: &str) -> String {
    format!("skills_extract/{}", sanitize_component(scope, 64, false))
}

/// Deterministic 64-bit content hash (same pattern as the skill sink).
fn content_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

fn apply_skill(
    db: &VantaEmbedded,
    scope: &str,
    skill: &SeedSkill,
    now: u64,
    counts: &mut SeedCounts,
) -> Result<(), SeedError> {
    let ns = skills_namespace(scope);
    let key = sanitize_key(&skill.name);
    let hash = content_hash(&skill.content);

    let existing_hash = match db.get(&ns, &key)? {
        Some(record) => {
            let existing: StoredSkill = serde_json::from_str(&record.payload)?;
            Some(existing.content_hash)
        }
        None => None,
    };
    if existing_hash == Some(hash) {
        counts.unchanged += 1; // identical content → skip
        return Ok(());
    }

    let stored = StoredSkill {
        name: skill.name.clone(),
        description: skill.description.clone(),
        content: skill.content.clone(),
        content_hash: hash,
        updated_at_ms: now,
    };
    let mut metadata = VantaMemoryMetadata::new();
    metadata.insert(
        "kind".into(),
        vantadb::sdk::VantaValue::String("skill".into()),
    );
    metadata.insert(
        "name".into(),
        vantadb::sdk::VantaValue::String(stored.name.clone()),
    );
    db.put(VantaMemoryInput {
        namespace: ns,
        key,
        payload: serde_json::to_string(&stored)?,
        metadata,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })?;
    if existing_hash.is_some() {
        counts.updated += 1;
    } else {
        counts.created += 1;
    }
    Ok(())
}

fn apply_persona(
    db: &VantaEmbedded,
    persona: &SeedPersona,
    now: u64,
    counts: &mut SeedCounts,
) -> Result<(), SeedError> {
    let existing = get_persona(db, &persona.session_key)?;
    if existing.as_ref().map(|p| p.content.as_str()) == Some(persona.content.as_str()) {
        counts.unchanged += 1; // identical persona → skip
        return Ok(());
    }

    // Mode reflects what happened: fresh seed = First, overwrite = Incremental.
    let record = PersonaRecord {
        content: persona.content.clone(),
        mode: if existing.is_some() {
            crate::core::abstractions::PersonaMode::Incremental
        } else {
            crate::core::abstractions::PersonaMode::First
        },
        generated_at_ms: now,
        generated_at: epoch_ms_to_rfc3339(now),
    };
    let mut metadata = VantaMemoryMetadata::new();
    metadata.insert(
        "kind".into(),
        vantadb::sdk::VantaValue::String("persona".into()),
    );
    db.put(VantaMemoryInput {
        namespace: persona_namespace(&persona.session_key),
        key: sanitize_key(PERSONA_KEY),
        payload: serde_json::to_string(&record)?,
        metadata,
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })?;
    if existing.is_some() {
        counts.updated += 1;
    } else {
        counts.created += 1;
    }
    Ok(())
}
