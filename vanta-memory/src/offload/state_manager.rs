//! Persistent per-session offload state (port of TDAM
//! `MC/offload/state-manager.ts`, MEM-20).
//!
//! TDAM keeps an in-memory [`PluginState`] mirrored to a per-agent
//! `state.json`. Here the single source of truth is the VantaDB store: the
//! state is one JSON record under `offload_state/<session>` keyed `__state`
//! (same pattern as the L0 cursor of MEM-09). A missing or corrupt record
//! falls back to the default state — never fatal, never blocking.
//!
//! The F4 cursor [`PluginState::last_offloaded_tool_call_id`] marks how far
//! the compact context has summarized; re-processing the same tool call must
//! never duplicate work (D19).

use crate::offload::types::PluginState;
use crate::utils::sanitize::{sanitize_component, sanitize_key};
use vantadb::error::VantaError;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput};

/// Errors surfaced by the offload surface (state manager, storage, hooks).
/// Wraps the SDK error so callers only depend on one error type.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OffloadError {
    #[error("vantadb: {0}")]
    Vanta(#[from] VantaError),
    #[error("malformed offload state payload: {0}")]
    State(#[from] serde_json::Error),
}

/// State record key inside the `offload_state/<session>` namespace.
const STATE_KEY: &str = "__state";

/// Persistent offload state over the VantaDB SDK. Owns the [`VantaEmbedded`]
/// handle; the host must keep it alive for the DB lifetime.
pub struct OffloadStateManager {
    db: VantaEmbedded,
}

impl OffloadStateManager {
    /// Open a state manager over an already-open embedded database.
    pub fn new(db: VantaEmbedded) -> Self {
        Self { db }
    }

    /// Load the persisted state for a session. Missing record → default
    /// state; corrupt payload → default state with a warning (mirrors TDAM's
    /// `readStateFile` catch-and-default).
    pub fn load_state(&self, session_id: &str) -> Result<PluginState, OffloadError> {
        let ns = state_namespace(session_id);
        match self.db.get(&ns, STATE_KEY)? {
            None => Ok(PluginState::default()),
            Some(record) => match serde_json::from_str(&record.payload) {
                Ok(state) => Ok(state),
                Err(err) => {
                    tracing::warn!(session = %session_id, "corrupt offload state, using default: {err}");
                    Ok(PluginState::default())
                }
            },
        }
    }

    /// Persist the full state for a session (upsert).
    pub fn save_state(&self, session_id: &str, state: &PluginState) -> Result<(), OffloadError> {
        let payload = serde_json::to_string(state)?;
        self.db.put(VantaMemoryInput {
            namespace: state_namespace(session_id),
            key: sanitize_key(STATE_KEY),
            payload,
            metadata: vantadb::sdk::VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(())
    }

    /// Read the persistent L3 cursor for a session.
    pub fn last_offloaded_tool_call_id(
        &self,
        session_id: &str,
    ) -> Result<Option<String>, OffloadError> {
        Ok(self.load_state(session_id)?.last_offloaded_tool_call_id)
    }

    /// Advance (or clear) the cursor: load-modify-save so unrelated fields
    /// (`mmd_counter`, active MMD…) survive the update.
    pub fn set_last_offloaded_tool_call_id(
        &self,
        session_id: &str,
        tool_call_id: Option<&str>,
    ) -> Result<(), OffloadError> {
        let mut state = self.load_state(session_id)?;
        state.last_offloaded_tool_call_id = tool_call_id.map(sanitize_key);
        self.save_state(session_id, &state)
    }
}

/// `offload_state/<sanitized-session>` — cursor/state records namespace.
fn state_namespace(session_id: &str) -> String {
    format!(
        "offload_state/{}",
        sanitize_component(session_id, 128, false)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    fn open_db() -> VantaEmbedded {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    #[test]
    fn missing_session_yields_default_state() {
        let mgr = OffloadStateManager::new(open_db());
        let state = mgr.load_state("sess-1").expect("load");
        assert_eq!(state, PluginState::default());
        assert_eq!(state.last_offloaded_tool_call_id, None);
    }

    #[test]
    fn cursor_persists_across_reopen() {
        let db = open_db();
        {
            let mgr = OffloadStateManager::new(db.clone());
            mgr.set_last_offloaded_tool_call_id("sess-1", Some("call_42"))
                .expect("set cursor");
        }
        // "Reopen": fresh manager over the same DB handle.
        let mgr = OffloadStateManager::new(db);
        assert_eq!(
            mgr.last_offloaded_tool_call_id("sess-1").expect("cursor"),
            Some("call_42".to_string())
        );
    }

    #[test]
    fn cursor_update_preserves_other_state_fields() {
        let db = open_db();
        let mgr = OffloadStateManager::new(db);
        let state = PluginState {
            mmd_counter: 7,
            active_mmd_file: Some("mmds/001.md".into()),
            ..PluginState::default()
        };
        mgr.save_state("sess-1", &state).expect("save");

        mgr.set_last_offloaded_tool_call_id("sess-1", Some("call_9"))
            .expect("set cursor");

        let reloaded = mgr.load_state("sess-1").expect("reload");
        assert_eq!(reloaded.mmd_counter, 7);
        assert_eq!(reloaded.active_mmd_file.as_deref(), Some("mmds/001.md"));
        assert_eq!(
            reloaded.last_offloaded_tool_call_id.as_deref(),
            Some("call_9")
        );
    }

    #[test]
    fn corrupt_state_payload_falls_back_to_default() {
        let db = open_db();
        // Write garbage directly under the state key.
        db.put(VantaMemoryInput {
            namespace: state_namespace("sess-x"),
            key: sanitize_key(STATE_KEY),
            payload: "not-json{".into(),
            metadata: vantadb::sdk::VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .expect("seed corrupt state");

        let mgr = OffloadStateManager::new(db);
        let state = mgr.load_state("sess-x").expect("fallback, not fatal");
        assert_eq!(state, PluginState::default());
    }

    #[test]
    fn namespaces_and_keys_are_sanitized() {
        assert_eq!(state_namespace("a/b c"), "offload_state/a_b_c");
        let mgr = OffloadStateManager::new(open_db());
        mgr.set_last_offloaded_tool_call_id("s", Some("call id/weird\n"))
            .expect("set");
        let stored = mgr.last_offloaded_tool_call_id("s").expect("get");
        assert_eq!(stored.as_deref(), Some("call_id_weird_"));
    }
}
