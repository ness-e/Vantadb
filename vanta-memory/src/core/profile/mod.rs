//! Profile layer: team+agent scoping and sync of L2/L3 profile content.
//! MEM-18 — see plan file.

pub mod profile_sync;

pub use profile_sync::{
    build_profile_isolation_scope, parse_profile_isolation_scope, profile_namespace,
    read_scoped_persona, sync_persona_to_scope, PersonaSyncOutcome, ProfileIsolation,
    ProfileSyncError, ScopedPersonaRecord, DEFAULT_PROFILE_SCOPE,
};
