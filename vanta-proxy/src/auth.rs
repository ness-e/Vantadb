//! Proxy auth against the local RBAC entity store (D25/D34).
//!
//! Port of the MEM-05 L3 pattern (`src/cli_server.rs::resolve_user_key`):
//! every request MUST carry a valid `x-vanta-user-key` resolved against the
//! local `user` entity collection — there is no open mode (D34).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

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
    /// Cache-aside `user_key → identity` snapshot (PRX-08 S1): the first
    /// miss scans once and memoizes EVERY user, so steady-state resolves
    /// are O(1) HashMap lookups instead of a 10k `list` scan per
    /// request. Ceiling: per-process snapshot — external writers MUST call
    /// [`AuthDb::invalidate`] (the proxy itself never writes `user`
    /// entities, so steady state is exact).
    index: Arc<Mutex<std::collections::HashMap<String, UserIdentity>>>,
    indexed: Arc<AtomicBool>,
}

impl AuthDb {
    /// Wrap an already-open storage engine (tests / shared handles).
    pub fn new(engine: Arc<StorageEngine>) -> Self {
        Self {
            engine,
            index: Arc::new(Mutex::new(std::collections::HashMap::new())),
            indexed: Arc::new(AtomicBool::new(false)),
        }
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
            index: Arc::new(Mutex::new(std::collections::HashMap::new())),
            indexed: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Underlying engine (shared with the embedded memory handle).
    pub fn engine(&self) -> Arc<StorageEngine> {
        self.engine.clone()
    }

    /// WIRE-09: count provisioned `user` entities for the refuse-to-start
    /// gate. Uses the page `total` so a single-row page suffices — no full
    /// collection scan at startup.
    ///
    /// # Errors
    /// [`ProxyError::Storage`] on local read failures.
    pub(crate) fn provisioned_user_count(&self) -> Result<usize, ProxyError> {
        let store = EntityStore::new(&self.engine);
        store
            .list(AUTH_ENTITY_NS, "user", 1, 0)
            .map(|page| page.total)
            .map_err(|e| ProxyError::Storage(format!("list user: {e}")))
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

    /// Drop the cached user index so the next resolve re-scans (call
    /// after any external write to the `user` entity collection).
    pub fn invalidate(&self) {
        self.index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.indexed.store(false, Ordering::Relaxed);
    }

    /// Resolve a user key to its identity — O(1) once warm (PRX-08 S1).
    ///
    /// First call scans the `user` entity collection (comparing keys in
    /// constant time, MEM-05 parity) and memoizes every user first-wins
    /// (list order preserved via `or_insert`, so duplicate keys keep the
    /// pre-index semantics); later calls are HashMap lookups.
    ///
    /// # Errors
    /// [`ProxyError::Storage`] on local read failures.
    pub fn resolve_user_key(&self, user_key: &str) -> Result<Option<UserIdentity>, ProxyError> {
        if self.indexed.load(Ordering::Relaxed) {
            return Ok(self
                .index
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(user_key)
                .cloned());
        }
        let store = EntityStore::new(&self.engine);
        let page = store
            .list(AUTH_ENTITY_NS, "user", USER_SCAN_LIMIT, 0)
            .map_err(|e| ProxyError::Storage(format!("list user: {e}")))?;
        let mut snapshot = std::collections::HashMap::with_capacity(page.items.len());
        for entity in &page.items {
            let Some(FieldValue::String(candidate)) = entity.fields.get("user_key") else {
                continue;
            };
            // First-wins preserves the pre-index duplicate-key semantics.
            snapshot.entry(candidate.clone()).or_insert_with(|| {
                let is_system_admin = matches!(
                    entity.fields.get("user_type"),
                    Some(FieldValue::String(t)) if t == "system_admin"
                );
                UserIdentity {
                    user_id: entity.id.clone(),
                    is_system_admin,
                }
            });
        }
        let found = snapshot.get(user_key).cloned();
        *self
            .index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = snapshot;
        self.indexed.store(true, Ordering::Relaxed);
        Ok(found)
    }

    /// Whether an entity exists in the given collection (session state
    /// machine validation contra entity_*). Malformed ids count as absent.
    ///
    /// # Errors
    /// [`ProxyError::Storage`] on unexpected local failures.
    pub fn entity_exists(&self, collection: &str, entity_id: &str) -> Result<bool, ProxyError> {
        let store = EntityStore::new(&self.engine);
        match store.get(AUTH_ENTITY_NS, collection, entity_id) {
            Ok(found) => Ok(found.is_some()),
            // ponytail: invalid ids (empty / '{' ':' '}') surface as InvalidInput —
            // treat as not-found so callers can reject with 400 instead of 500.
            Err(vantadb::error::Error::InvalidInput(_)) => Ok(false),
            Err(e) => Err(ProxyError::Storage(format!("get {collection}: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use vantadb::entity::EntityWrite;

    fn in_memory_db() -> AuthDb {
        let config = vantadb::config::Config {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            read_only: false,
            ..vantadb::config::Config::default()
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
            .set(EntityWrite {
                namespace: AUTH_ENTITY_NS,
                collection: "user",
                id,
                fields,
            })
            .expect("seed user");
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

    #[test]
    fn index_serves_o1_after_warm_and_invalidate_refreshes() {
        // PRX-08 S1 RED: cache-aside index — warm once, then O(1).
        let db = in_memory_db();
        seed_user(&db, "usr-1", Some("sk-1"), None);
        assert!(db.resolve_user_key("sk-1").expect("resolve").is_some());

        // Seeded after warm: invisible until invalidate (documented ceiling).
        seed_user(&db, "usr-2", Some("sk-2"), None);
        assert!(db.resolve_user_key("sk-2").expect("resolve").is_none());
        db.invalidate();
        let identity = db
            .resolve_user_key("sk-2")
            .expect("resolve")
            .expect("found after invalidate");
        assert_eq!(identity.user_id, "usr-2");
    }
}
