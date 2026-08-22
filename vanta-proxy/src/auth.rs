//! Proxy auth against the local RBAC entity store (D25/D34).
//!
//! Port of the MEM-05 L3 pattern (`src/cli_server.rs::resolve_user_key`):
//! every request MUST carry a valid `x-vanta-user-key` resolved against the
//! local `user` entity collection — there is no open mode (D34).

use std::sync::Arc;

use axum::http::HeaderMap;
use vantadb::entity::EntityStore;
use vantadb::node::FieldValue;
use vantadb::storage::StorageEngine;

use crate::error::ProxyError;

/// Header carrying the caller's user key (TDAM `x-tdai-user-key` port).
pub const USER_KEY_HEADER: &str = "x-vanta-user-key";

/// Namespace holding auth entities (MEM-05 parity: fixed `"default"`).
const AUTH_ENTITY_NS: &str = "default";

/// Max users scanned per key resolution (MEM-05 parity).
const USER_SCAN_LIMIT: usize = 10_000;

/// A resolved caller identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserIdentity {
    pub user_id: String,
    pub is_system_admin: bool,
}

/// Handle over the local VantaDB store used for auth + session validation.
///
/// The same [`StorageEngine`] backs the embedded memory handle used by
/// [`crate::inject`] — one open database serves both APIs.
#[derive(Clone)]
pub struct AuthDb {
    engine: Arc<StorageEngine>,
}

impl AuthDb {
    /// Wrap an already-open storage engine (tests / shared handles).
    pub fn new(engine: Arc<StorageEngine>) -> Self {
        Self { engine }
    }

    /// Open the local store at `path`.
    ///
    /// # Errors
    /// [`ProxyError::Storage`] when the database cannot be opened.
    pub fn open(path: &str) -> Result<Self, ProxyError> {
        let engine = StorageEngine::open(path)
            .map_err(|e| ProxyError::Storage(format!("open {}: {e}", path)))?;
        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    /// Underlying engine (shared with the embedded memory handle).
    pub fn engine(&self) -> Arc<StorageEngine> {
        self.engine.clone()
    }

    /// D34: resolve the request identity from headers. Missing, empty or
    /// unknown user keys all fail closed with [`ProxyError::Unauthorized`].
    ///
    /// # Errors
    /// - [`ProxyError::Unauthorized`] — no/unknown key (D34)
    /// - [`ProxyError::Storage`] — local read failure
    pub fn authenticate(&self, headers: &HeaderMap) -> Result<UserIdentity, ProxyError> {
        let key = headers
            .get(USER_KEY_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(ProxyError::Unauthorized)?;

        self.resolve_user_key(key)?.ok_or(ProxyError::Unauthorized)
    }

    /// Resolve a user key to its identity by scanning the `user` entity
    /// collection and comparing keys in constant time (MEM-05 parity).
    ///
    /// # Errors
    /// [`ProxyError::Storage`] on local read failures.
    pub fn resolve_user_key(&self, user_key: &str) -> Result<Option<UserIdentity>, ProxyError> {
        let store = EntityStore::new(&self.engine);
        let page = store
            .entity_list(AUTH_ENTITY_NS, "user", USER_SCAN_LIMIT, 0)
            .map_err(|e| ProxyError::Storage(format!("entity_list user: {e}")))?;
        for entity in page.items {
            let Some(FieldValue::String(candidate)) = entity.fields.get("user_key") else {
                continue;
            };
            if ct_eq(candidate.as_bytes(), user_key.as_bytes()) {
                let is_system_admin = matches!(
                    entity.fields.get("user_type"),
                    Some(FieldValue::String(t)) if t == "system_admin"
                );
                return Ok(Some(UserIdentity {
                    user_id: entity.entity_id,
                    is_system_admin,
                }));
            }
        }
        Ok(None)
    }

    /// Whether an entity exists in the given collection (session state
    /// machine validation contra entity_*). Malformed ids count as absent.
    ///
    /// # Errors
    /// [`ProxyError::Storage`] on unexpected local failures.
    pub fn entity_exists(&self, collection: &str, entity_id: &str) -> Result<bool, ProxyError> {
        let store = EntityStore::new(&self.engine);
        match store.entity_get(AUTH_ENTITY_NS, collection, entity_id) {
            Ok(found) => Ok(found.is_some()),
            // ponytail: invalid ids (empty / '{' ':' '}') surface as InvalidInput —
            // treat as not-found so callers can reject with 400 instead of 500.
            Err(vantadb::error::VantaError::InvalidInput(_)) => Ok(false),
            Err(e) => Err(ProxyError::Storage(format!("entity_get {collection}: {e}"))),
        }
    }
}

/// Constant-time byte equality (no external dep): accumulates the XOR of all
/// byte differences so branch timing does not leak the match position.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn in_memory_db() -> AuthDb {
        let config = vantadb::config::VantaConfig {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            read_only: false,
            ..vantadb::config::VantaConfig::default()
        };
        let engine =
            StorageEngine::open_with_config(":memory:", Some(config)).expect("in-memory engine");
        AuthDb::new(Arc::new(engine))
    }

    fn seed_user(db: &AuthDb, id: &str, key: Option<&str>, user_type: Option<&str>) {
        let mut fields: HashMap<String, FieldValue> = HashMap::new();
        if let Some(k) = key {
            fields.insert("user_key".into(), FieldValue::String(k.to_string()));
        }
        if let Some(t) = user_type {
            fields.insert("user_type".into(), FieldValue::String(t.to_string()));
        }
        EntityStore::new(&db.engine)
            .entity_set(AUTH_ENTITY_NS, "user", id, fields)
            .expect("seed user");
    }

    #[test]
    fn ct_eq_basic() {
        assert!(ct_eq(b"abc", b"abc"));
        assert!(!ct_eq(b"abc", b"abd"));
        assert!(!ct_eq(b"abc", b"ab"));
    }

    #[test]
    fn valid_key_resolves_and_admin_flag_set() {
        let db = in_memory_db();
        seed_user(&db, "usr-1", Some("sk-good"), Some("system_admin"));
        let identity = db
            .resolve_user_key("sk-good")
            .expect("resolve")
            .expect("found");
        assert_eq!(identity.user_id, "usr-1");
        assert!(identity.is_system_admin);
    }

    #[test]
    fn unknown_key_fails_closed_d34() {
        let db = in_memory_db();
        seed_user(&db, "usr-1", Some("sk-good"), None);
        assert!(matches!(
            db.authenticate(&HeaderMap::new()),
            Err(ProxyError::Unauthorized)
        ));
        let mut headers = HeaderMap::new();
        headers.insert(USER_KEY_HEADER, "sk-bad".parse().expect("hv"));
        assert!(matches!(
            db.authenticate(&headers),
            Err(ProxyError::Unauthorized)
        ));
    }

    #[test]
    fn empty_or_whitespace_key_rejected() {
        let db = in_memory_db();
        let mut headers = HeaderMap::new();
        headers.insert(USER_KEY_HEADER, "   ".parse().expect("hv"));
        assert!(matches!(
            db.authenticate(&headers),
            Err(ProxyError::Unauthorized)
        ));
    }
}
