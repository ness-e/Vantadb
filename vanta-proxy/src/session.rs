//! Session resolution + local state machine (D26).
//!
//! sessionKey comes from the first present header alias (TDAM
//! `session-key.ts:9-19` order). The local form state machine progresses
//! monotonically `team → agent → task`, validating each target entity against
//! the local entity_* collections. Pending states (team/agent) expire after a
//! 30-minute TTL; the sweep runs lazily on every request.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use serde::Serialize;

pub mod claude_code;

use crate::auth::AuthDb;
use crate::error::ProxyError;

/// Header aliases consulted in order; first present wins (D26).
///
/// `x-vanta-session` (PRX-01) is the proxy's own explicit trigger and takes
/// top priority; the TDAM parity order below it is unchanged.
pub const SESSION_HEADER_ALIASES: [&str; 6] = [
    "x-vanta-session",
    "x-conversation-id",
    "x-session-id",
    "x-claude-code-session-id",
    "x-chat-id",
    "x-thread-id",
];

/// TTL for pending states only (TDAM `store.ts:31,116` — 30 min).
pub const PENDING_TTL_MS: u64 = 30 * 60 * 1000;

/// Hard cap on live sessions (PRX-08 S3): the store was unbounded.
pub const MAX_SESSIONS: usize = 10_000;

/// Current point of the local form state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Waiting for / bound to a team.
    Team,
    /// Bound to an agent within the team.
    Agent,
    /// Bound to a task — terminal state, never expires.
    Task,
}

impl Stage {
    fn entity_collection(self) -> &'static str {
        match self {
            Stage::Team => "team",
            Stage::Agent => "agent",
            Stage::Task => "task",
        }
    }

    fn is_pending(self) -> bool {
        self != Stage::Task
    }

    /// Stable wire label (`/snapshot`, `/session/advance`).
    pub(crate) fn label(self) -> &'static str {
        match self {
            Stage::Team => "team",
            Stage::Agent => "agent",
            Stage::Task => "task",
        }
    }
}

/// One session's wire shape for `/snapshot` (DESKTOP-38).
#[derive(Debug, Clone, Serialize)]
pub struct SessionSnapshot {
    pub key: String,
    /// Current stage of the team→agent→task state machine.
    pub stage: &'static str,
    pub updated_at_ms: u64,
    /// Only pending stages expire (30-min TTL); terminal Task never does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
}

struct Entry {
    stage: Stage,
    updated_at_ms: u64,
}

/// In-process session store. Local-first by design (D26) — no remote gateway.
#[derive(Default)]
pub struct SessionStore {
    sessions: Mutex<HashMap<String, Entry>>,
}

/// Parse a `POST /session/advance` body target (PRX-01). Case-insensitive,
/// surrounding whitespace ignored; anything else is `None` (→ 400).
pub fn parse_stage(target: &str) -> Option<Stage> {
    match target.trim().to_ascii_lowercase().as_str() {
        "team" => Some(Stage::Team),
        "agent" => Some(Stage::Agent),
        "task" => Some(Stage::Task),
        _ => None,
    }
}

/// Extract the session key from headers (alias priority order, trimmed).
pub fn session_key_from_headers(headers: &HeaderMap) -> Option<String> {
    SESSION_HEADER_ALIASES.iter().find_map(|alias| {
        headers
            .get(*alias)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    })
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Cap the store at [`MAX_SESSIONS`] by dropping the stalest entries
/// (oldest activity first). Linear scan, but it only runs on the insert
/// that overflows the cap — steady state pays nothing.
fn evict_oldest_past_cap(sessions: &mut HashMap<String, Entry>) {
    while sessions.len() > MAX_SESSIONS {
        let Some(oldest) = sessions
            .iter()
            .min_by_key(|(_, e)| e.updated_at_ms)
            .map(|(k, _)| k.clone())
        else {
            break;
        };
        sessions.remove(&oldest);
    }
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop expired pending sessions (lazy sweep, per request).
    fn sweep(&self, now: u64) {
        let mut sessions = self.lock();
        sessions.retain(|_, e| {
            !e.stage.is_pending() || now.saturating_sub(e.updated_at_ms) < PENDING_TTL_MS
        });
    }

    /// Resolve (or create at [`Stage::Team`]) the session and refresh its
    /// activity timestamp. Init is clean: no prior session simply starts at
    /// Team.
    pub fn ensure(&self, key: &str) -> Stage {
        let now = now_ms();
        self.sweep(now);
        let mut sessions = self.lock();
        let entry = sessions.entry(key.to_string()).or_insert(Entry {
            stage: Stage::Team,
            updated_at_ms: now,
        });
        entry.updated_at_ms = now;
        let stage = entry.stage;
        evict_oldest_past_cap(&mut sessions);
        stage
    }

    /// Advance the session to `target` (monotonic team→agent→task), requiring
    /// `entity_id` to exist in the matching local entity collection.
    ///
    /// # Errors
    /// - [`ProxyError::InvalidRequest`] — backwards/skipping transition or
    ///   unknown entity
    /// - [`ProxyError::Storage`] — local read failure
    pub fn advance(
        &self,
        db: &AuthDb,
        key: &str,
        target: Stage,
        entity_id: &str,
    ) -> Result<Stage, ProxyError> {
        let now = now_ms();
        self.sweep(now);
        if !db.entity_exists(target.entity_collection(), entity_id)? {
            return Err(ProxyError::InvalidRequest(format!(
                "unknown {} entity `{entity_id}`",
                target.entity_collection()
            )));
        }
        let mut sessions = self.lock();
        let entry = sessions.entry(key.to_string()).or_insert(Entry {
            stage: Stage::Team,
            updated_at_ms: now,
        });
        let allowed = matches!(
            (entry.stage, target),
            (Stage::Team, Stage::Team)
                | (Stage::Team, Stage::Agent)
                | (Stage::Agent, Stage::Agent)
                | (Stage::Agent, Stage::Task)
                | (Stage::Task, Stage::Task)
        );
        if !allowed {
            return Err(ProxyError::InvalidRequest(format!(
                "cannot move session from {:?} to {target:?}",
                entry.stage
            )));
        }
        entry.stage = target;
        entry.updated_at_ms = now;
        evict_oldest_past_cap(&mut sessions);
        Ok(target)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Entry>> {
        self.sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// All live sessions (expired pending swept first), unordered (`/snapshot`).
    pub fn snapshot(&self) -> Vec<SessionSnapshot> {
        let now = now_ms();
        self.sweep(now);
        self.lock()
            .iter()
            .map(|(key, e)| SessionSnapshot {
                key: key.clone(),
                stage: e.stage.label(),
                updated_at_ms: e.updated_at_ms,
                expires_at_ms: e
                    .stage
                    .is_pending()
                    .then(|| e.updated_at_ms + PENDING_TTL_MS),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use vantadb::entity::{EntityStore, EntityWrite};

    fn in_memory_db() -> AuthDb {
        let config = vantadb::config::Config {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            read_only: false,
            ..vantadb::config::Config::default()
        };
        let engine = vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
            .expect("in-memory engine");
        AuthDb::new(std::sync::Arc::new(engine))
    }

    fn seed_entity(db: &AuthDb, collection: &str, id: &str) {
        EntityStore::new(&db.engine())
            .set(EntityWrite {
                namespace: "default",
                collection,
                id,
                fields: HashMap::new(),
            })
            .expect("seed entity");
    }

    #[test]
    fn x_vanta_session_header_wins_over_aliases() {
        // PRX-01: the proxy's own trigger header has top priority.
        let mut h = HeaderMap::new();
        h.insert("x-conversation-id", "from-conv".parse().expect("hv"));
        h.insert("x-vanta-session", "from-vanta".parse().expect("hv"));
        assert_eq!(session_key_from_headers(&h).as_deref(), Some("from-vanta"));
    }

    #[test]
    fn parse_stage_accepts_team_agent_task() {
        // PRX-01: `POST /session/advance` body target values.
        assert_eq!(parse_stage("team"), Some(Stage::Team));
        assert_eq!(parse_stage("agent"), Some(Stage::Agent));
        assert_eq!(parse_stage("task"), Some(Stage::Task));
        assert_eq!(parse_stage("bogus"), None);
        assert_eq!(parse_stage(""), None);
    }

    #[test]
    fn session_key_first_present_alias_wins_in_priority_order() {
        let mut h = HeaderMap::new();
        assert!(session_key_from_headers(&h).is_none());

        h.insert("x-thread-id", "from-thread".parse().expect("hv"));
        assert_eq!(session_key_from_headers(&h).as_deref(), Some("from-thread"));

        h.insert("x-chat-id", "from-chat".parse().expect("hv"));
        assert_eq!(session_key_from_headers(&h).as_deref(), Some("from-chat"));

        h.insert("x-claude-code-session-id", "from-cc".parse().expect("hv"));
        h.insert("x-session-id", "from-session".parse().expect("hv"));
        h.insert("x-conversation-id", "from-conv".parse().expect("hv"));
        assert_eq!(
            session_key_from_headers(&h).as_deref(),
            Some("from-conv"),
            "x-conversation-id has highest priority"
        );
    }

    #[test]
    fn empty_alias_value_is_skipped() {
        let mut h = HeaderMap::new();
        h.insert("x-conversation-id", "  ".parse().expect("hv"));
        h.insert("x-thread-id", "real".parse().expect("hv"));
        assert_eq!(session_key_from_headers(&h).as_deref(), Some("real"));
    }

    #[test]
    fn init_clean_starts_at_team_and_advances_monotonically() {
        let db = in_memory_db();
        for c in ["team", "agent", "task"] {
            seed_entity(&db, c, &format!("{c}-1"));
        }
        let store = SessionStore::new();

        // (f) no prior session → clean init at Team.
        assert_eq!(store.ensure("sess-a"), Stage::Team);

        assert_eq!(
            store
                .advance(&db, "sess-a", Stage::Agent, "agent-1")
                .expect("advance"),
            Stage::Agent
        );
        assert_eq!(
            store
                .advance(&db, "sess-a", Stage::Task, "task-1")
                .expect("advance"),
            Stage::Task
        );
        assert_eq!(store.ensure("sess-a"), Stage::Task);
    }

    #[test]
    fn skipping_or_backwards_transitions_rejected() {
        let db = in_memory_db();
        for c in ["team", "agent", "task"] {
            seed_entity(&db, c, &format!("{c}-1"));
        }
        let store = SessionStore::new();

        // Team → Task skips Agent.
        assert!(matches!(
            store.advance(&db, "sess-b", Stage::Task, "task-1"),
            Err(ProxyError::InvalidRequest(_))
        ));
        // Fresh session still sits at Team afterwards.
        assert_eq!(store.ensure("sess-b"), Stage::Team);

        store
            .advance(&db, "sess-b", Stage::Agent, "agent-1")
            .expect("to agent");
        // Backwards not allowed.
        assert!(matches!(
            store.advance(&db, "sess-b", Stage::Team, "team-1"),
            Err(ProxyError::InvalidRequest(_))
        ));
    }

    #[test]
    fn unknown_entity_rejected_before_transition() {
        let db = in_memory_db();
        seed_entity(&db, "team", "team-1");
        let store = SessionStore::new();

        assert!(matches!(
            store.advance(&db, "sess-c", Stage::Agent, "agent-missing"),
            Err(ProxyError::InvalidRequest(_))
        ));
        // Session was created at Team but never advanced.
        assert_eq!(store.ensure("sess-c"), Stage::Team);
    }

    #[test]
    fn pending_ttl_sweeps_but_task_persists() {
        let store = SessionStore::new();
        let now = now_ms();

        {
            let mut sessions = store.sessions.lock().expect("lock");
            sessions.insert(
                "old-pending".into(),
                Entry {
                    stage: Stage::Team,
                    updated_at_ms: now - PENDING_TTL_MS - 1,
                },
            );
            sessions.insert(
                "fresh-pending".into(),
                Entry {
                    stage: Stage::Agent,
                    updated_at_ms: now,
                },
            );
            sessions.insert(
                "old-task".into(),
                Entry {
                    stage: Stage::Task,
                    updated_at_ms: now - PENDING_TTL_MS - 1,
                },
            );
        }

        store.sweep(now);

        let sessions = store.sessions.lock().expect("lock");
        assert!(
            !sessions.contains_key("old-pending"),
            "expired pending swept"
        );
        assert!(sessions.contains_key("fresh-pending"));
        assert!(
            sessions.contains_key("old-task"),
            "terminal task never expires"
        );
    }

    #[test]
    fn snapshot_reports_stage_and_ttl_only_for_pending() {
        let store = SessionStore::new();
        let db = in_memory_db();
        for c in ["team", "agent", "task"] {
            seed_entity(&db, c, &format!("{c}-1"));
        }
        store.ensure("sess-a");
        store
            .advance(&db, "sess-a", Stage::Agent, "agent-1")
            .expect("advance");
        store.ensure("sess-b");
        store
            .advance(&db, "sess-b", Stage::Agent, "agent-1")
            .expect("advance");
        store
            .advance(&db, "sess-b", Stage::Task, "task-1")
            .expect("advance");

        let mut snap = store.snapshot();
        snap.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(snap.len(), 2);

        assert_eq!(snap[0].key, "sess-a");
        assert_eq!(snap[0].stage, "agent");
        assert!(snap[0].expires_at_ms.is_some());
        assert_eq!(
            snap[0].expires_at_ms,
            Some(snap[0].updated_at_ms + PENDING_TTL_MS)
        );

        assert_eq!(snap[1].key, "sess-b");
        assert_eq!(snap[1].stage, "task");
        assert_eq!(snap[1].expires_at_ms, None, "task never expires");
    }

    #[test]
    fn store_evicts_oldest_past_cap() {
        // PRX-08 S3 RED: unbounded HashMap must drop oldest past MAX_SESSIONS.
        let store = SessionStore::new();
        let now = now_ms();
        {
            let mut sessions = store.sessions.lock().expect("lock");
            for i in 0..=MAX_SESSIONS {
                sessions.insert(
                    format!("sess-{i:05}"),
                    Entry {
                        stage: Stage::Team,
                        // Fresh (sweep-proof): oldest is still < TTL old.
                        updated_at_ms: now - i as u64,
                    },
                );
            }
        }
        assert_eq!(store.ensure("fresh"), Stage::Team);
        let sessions = store.sessions.lock().expect("lock");
        assert_eq!(sessions.len(), MAX_SESSIONS);
        assert!(sessions.contains_key("fresh"));
        assert!(
            !sessions.contains_key("sess-10000"),
            "oldest evicted past cap"
        );
    }
}
