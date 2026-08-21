//! Profile sync (MEM-18): team+agent scoping of L2/L3 profile content.
//!
//! TDAM `MC/core/profile/profile-sync.ts` syncs `persona.md` / scene blocks
//! between a local fs and remote stores with MD5 verification. In
//! vanta-memory everything already lives in VantaDB (Principio 2), so the
//! exercised core is: the isolation scope (`team:{t}|agent:{a}`), an
//! idempotent copy of the persona body into the scoped namespace, and scoped
//! reads for the auto-recall hook.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::conversation::{now_ms, sanitize_component};
use crate::core::persona::persona_generator::{get_persona, PersonaError};
use crate::core::scene::scene_navigation::strip_scene_navigation;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};

/// Scope used when no isolation is provided (TDAM `DEFAULT_PROFILE_SCOPE`).
pub const DEFAULT_PROFILE_SCOPE: &str = "global";

/// Team+agent isolation for L2/L3 profiles. User/session/task dimensions are
/// intentionally excluded (TDAM parity): one team's agent memory accumulates
/// across sessions and users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileIsolation {
    pub team_id: String,
    pub agent_id: String,
}

impl Default for ProfileIsolation {
    fn default() -> Self {
        Self {
            team_id: "default".into(),
            agent_id: "default".into(),
        }
    }
}

/// Build the scope string `team:{t}|agent:{a}` (TDAM
/// `buildProfileIsolationScope`).
pub fn build_profile_isolation_scope(iso: &ProfileIsolation) -> String {
    format!("team:{}|agent:{}", iso.team_id, iso.agent_id)
}

/// Parse a scope string back into an isolation (TDAM
/// `parseProfileIsolationScope`). Returns `None` on malformed input.
pub fn parse_profile_isolation_scope(scope: &str) -> Option<ProfileIsolation> {
    let mut team = None;
    let mut agent = None;
    for part in scope.split('|') {
        let (key, value) = part.split_once(':')?;
        match key {
            "team" => team = Some(value.to_string()),
            "agent" => agent = Some(value.to_string()),
            _ => return None,
        }
    }
    Some(ProfileIsolation {
        team_id: team?,
        agent_id: agent?,
    })
}

/// `profile/{sanitized-scope}` — persisted scoped-profile namespace.
pub fn profile_namespace(scope: &str) -> String {
    format!("profile/{}", sanitize_component(scope, 128, false))
}

/// Record key of the scoped persona inside the profile namespace.
const PERSONA_KEY: &str = "persona";

/// A synced persona snapshot in the scoped namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScopedPersonaRecord {
    /// Persona body with the scene navigation stripped (TDAM stores the
    /// stripped body; recall re-attaches fresh navigation per turn).
    pub content: String,
    pub synced_at_ms: u64,
}

/// Errors surfaced by the profile-sync layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProfileSyncError {
    /// Underlying VantaDB storage error.
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::VantaError),
    /// Persona layer failure.
    #[error("persona: {0}")]
    Persona(#[from] PersonaError),
    /// Payload (de)serialization failure.
    #[error("profile record: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Outcome of a [`sync_persona_to_scope`] pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonaSyncOutcome {
    /// `true` when the scoped record was written this pass.
    pub updated: bool,
    /// `true` when nothing was done because there is no persona body yet.
    pub skipped_no_persona: bool,
}

/// Idempotently copy the session persona body into the scoped namespace:
/// re-running with unchanged content does NOT rewrite the record (string
/// equality — no hash needed when both sides live in the same store).
///
/// MD5-mismatch safety of TDAM translates here to: a malformed existing
/// record is simply overwritten by the verified session persona — local data
/// loss is impossible because the source of truth is never deleted.
pub fn sync_persona_to_scope(
    db: &VantaEmbedded,
    session_key: &str,
    isolation: &ProfileIsolation,
) -> Result<PersonaSyncOutcome, ProfileSyncError> {
    let Some(persona) = get_persona(db, session_key)? else {
        return Ok(PersonaSyncOutcome {
            updated: false,
            skipped_no_persona: true,
        });
    };
    let body = strip_scene_navigation(&persona.content).trim().to_string();
    if body.is_empty() {
        return Ok(PersonaSyncOutcome {
            updated: false,
            skipped_no_persona: true,
        });
    }

    let ns = profile_namespace(&build_profile_isolation_scope(isolation));
    let key = crate::core::conversation::sanitize_key(PERSONA_KEY);
    if let Some(existing) = db.get(&ns, &key)? {
        if let Ok(record) = serde_json::from_str::<ScopedPersonaRecord>(&existing.payload) {
            if record.content == body {
                return Ok(PersonaSyncOutcome {
                    updated: false,
                    skipped_no_persona: false,
                });
            }
        }
    }

    let record = ScopedPersonaRecord {
        content: body,
        synced_at_ms: now_ms(),
    };
    db.put(VantaMemoryInput {
        namespace: ns,
        key,
        payload: serde_json::to_string(&record)?,
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })?;
    Ok(PersonaSyncOutcome {
        updated: true,
        skipped_no_persona: false,
    })
}

/// Read the scoped persona body (navigation already stripped).
pub fn read_scoped_persona(
    db: &VantaEmbedded,
    isolation: &ProfileIsolation,
) -> Result<Option<String>, ProfileSyncError> {
    let ns = profile_namespace(&build_profile_isolation_scope(isolation));
    let key = crate::core::conversation::sanitize_key(PERSONA_KEY);
    match db.get(&ns, &key)? {
        Some(record) => Ok(Some(
            serde_json::from_str::<ScopedPersonaRecord>(&record.payload)?.content,
        )),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_roundtrip_and_malformed_input() {
        let iso = ProfileIsolation {
            team_id: "team-a".into(),
            agent_id: "agent 1".into(),
        };
        let scope = build_profile_isolation_scope(&iso);
        assert_eq!(scope, "team:team-a|agent:agent 1");
        assert_eq!(parse_profile_isolation_scope(&scope), Some(iso));
        assert_eq!(parse_profile_isolation_scope("global"), None);
        assert_eq!(parse_profile_isolation_scope("team:x"), None);
        assert_eq!(parse_profile_isolation_scope("user:x|agent:y"), None);
    }

    #[test]
    fn namespace_is_sanitized() {
        assert_eq!(
            profile_namespace("team:a/b|agent:c"),
            "profile/team_a_b_agent_c"
        );
    }
}
