//! Seed input schema and validation (MEM-39).
//!
//! Deliberate deviation from TDAM `core/seed/input.ts`: that schema imports
//! full conversation transcripts (sessions/rounds/messages) and is coupled to
//! the OpenClaw host capture format. Our scope is **skills + initial persona
//! only** (host-specific seeds are out of scope per plan P29 Task 4), so this
//! is a minimal purpose-built schema:
//!
//! ```json
//! {
//!   "scope": "my-agent",
//!   "skills": [
//!     { "name": "deploy", "description": "...", "content": "..." }
//!   ],
//!   "persona": { "session_key": "user-1", "content": "# Profile ..." }
//! }
//! ```
//!
//! `scope` (default `"seed"`) selects the `skills_extract/<scope>` namespace;
//! `skills` and `persona` are both optional but at least one must be present.
//! JSON only — no YAML parser exists in the workspace and adding one violates
//! the no-new-deps constraint.

use serde::Deserialize;

/// Default skills scope when the input omits one.
pub const DEFAULT_SCOPE: &str = "seed";

/// Top-level seed document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SeedInput {
    /// Skills namespace scope (sanitized before use).
    #[serde(default = "default_scope")]
    pub scope: String,
    /// Initial skills to import.
    #[serde(default)]
    pub skills: Vec<SeedSkill>,
    /// Optional initial persona document.
    #[serde(default)]
    pub persona: Option<SeedPersona>,
}

fn default_scope() -> String {
    DEFAULT_SCOPE.to_string()
}

/// A single seeded skill (parity with MEM-06 [`StoredSkill`] inputs).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SeedSkill {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub content: String,
}

/// A seeded persona document for one session key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SeedPersona {
    pub session_key: String,
    pub content: String,
}

/// Parse and validate a seed document from raw JSON text.
pub fn parse_seed_input(raw: &str) -> Result<SeedInput, super::SeedError> {
    if raw.trim().is_empty() {
        return Err(super::SeedError::Validation("seed input is empty".into()));
    }
    let seed: SeedInput = serde_json::from_str(raw)?;
    validate(&seed)?;
    Ok(seed)
}

fn validate(seed: &SeedInput) -> Result<(), super::SeedError> {
    if seed.skills.is_empty() && seed.persona.is_none() {
        return Err(super::SeedError::Validation(
            "seed input must contain at least one skill or a persona".into(),
        ));
    }
    if seed.scope.trim().is_empty() {
        return Err(super::SeedError::Validation(
            "\"scope\" must be a non-empty string when present".into(),
        ));
    }
    for (idx, skill) in seed.skills.iter().enumerate() {
        if skill.name.trim().is_empty() {
            return Err(super::SeedError::Validation(format!(
                "skills[{idx}]: \"name\" must be a non-empty string"
            )));
        }
        if skill.content.trim().is_empty() {
            return Err(super::SeedError::Validation(format!(
                "skills[{}]: \"content\" must be a non-empty string",
                idx
            )));
        }
    }
    if let Some(persona) = &seed.persona {
        if persona.session_key.trim().is_empty() {
            return Err(super::SeedError::Validation(
                "persona: \"session_key\" must be a non-empty string".into(),
            ));
        }
        if persona.content.trim().is_empty() {
            return Err(super::SeedError::Validation(
                "persona: \"content\" must be a non-empty string".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_skills_only_document() {
        let seed = parse_seed_input(r#"{"skills":[{"name":"a","content":"b"}]}"#)
            .expect("valid minimal seed");
        assert_eq!(seed.scope, DEFAULT_SCOPE);
        assert_eq!(seed.skills.len(), 1);
        assert!(seed.persona.is_none());
    }

    #[test]
    fn rejects_empty_document_and_missing_fields() {
        assert!(matches!(
            parse_seed_input("{}"),
            Err(super::super::SeedError::Validation(_))
        ));
        assert!(matches!(
            parse_seed_input(r#"{"skills":[{"name":" ","content":"b"}]}"#),
            Err(super::super::SeedError::Validation(_))
        ));
        assert!(matches!(
            parse_seed_input("not json"),
            Err(super::super::SeedError::Json(_))
        ));
        assert!(matches!(
            parse_seed_input("   "),
            Err(super::super::SeedError::Validation(_))
        ));
    }
}
