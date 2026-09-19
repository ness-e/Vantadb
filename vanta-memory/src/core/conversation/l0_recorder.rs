//! L0 conversation capture — LLM-free, idempotent, persisted in VantaDB.
//!
//! L0 stores raw conversation turns as stable-key records under the
//! `l0/<session>` namespace (SDK `put`/`get`/`list`), with a persistent
//! cursor in the separate `l0_cursor/<session>` namespace. The cursor holds
//! `{"after_timestamp_ms": u64}` so replaying the same turn never duplicates
//! records (MEM-09 contract, D19).
//!
//! The recorder is deliberately LLM-free (Principle 4): it never blocks and
//! never loses data regardless of LLM availability.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use vantadb::error::Error;
use vantadb::sdk::{
    Embedded, MemoryInput, MemoryListOptions, MemoryListPage, MemoryMetadata, MemoryRecord, Value,
};

/// Conversational role of an L0 message. Serde snake_case keeps the wire
/// format host-neutral (`"user"`, `"assistant"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum L0Role {
    User,
    Assistant,
}

impl FromStr for L0Role {
    type Err = L0Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(L0Role::User),
            "assistant" => Ok(L0Role::Assistant),
            other => Err(L0Error::InvalidRole(other.to_string())),
        }
    }
}

impl fmt::Display for L0Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            L0Role::User => write!(f, "user"),
            L0Role::Assistant => write!(f, "assistant"),
        }
    }
}

/// A single captured L0 message.
///
/// `id` is the stable key used for SDK upsert idempotency. When `None`, the
/// recorder derives a stable `t{timestamp_ms}_{index}_{fnv}` key from the
/// message position plus an FNV-1a hash of role+content (see `derived_key`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L0Message {
    pub id: Option<String>,
    pub role: L0Role,
    pub content: String,
    pub timestamp_ms: u64,
}

/// One capture unit: the messages of a single conversation turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L0Capture {
    pub session_id: String,
    pub messages: Vec<L0Message>,
}

/// Outcome of a capture attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L0CaptureResult {
    /// Messages that were actually persisted (post cursor-filter + dedup).
    pub recorded: Vec<L0Message>,
    pub recorded_count: usize,
    /// New cursor value (max recorded timestamp, or the previous cursor).
    pub cursor_ms: u64,
}

/// Errors surfaced by the L0 recorder. Wraps the SDK error so callers only
/// depend on one error type for the whole L0/L1 surface.
#[derive(Debug, Error)]
pub enum L0Error {
    #[error("vantadb: {0}")]
    Vanta(#[from] Error),
    #[error("invalid L0 role: {0}")]
    InvalidRole(String),
    #[error("malformed cursor payload: {0}")]
    Cursor(#[from] serde_json::Error),
}

/// Stable derived L0 key: `t{timestamp_ms}_{index}_{fnv}` where `fnv` is the
/// FNV-1a-64 hex of `role\0content`. Identical input always yields the same
/// key — in any thread, in any call — so concurrent same-ms captures never
/// overwrite each other (distinct content → distinct keys) while exact
/// redeliveries stay idempotent (same key → upsert + cursor skip).
fn derived_key(timestamp_ms: u64, index: usize, role: &str, content: &str) -> String {
    format!("t{timestamp_ms}_{index}_{}", fnv1a_hex(role, content))
}

/// FNV-1a 64-bit, hex-encoded. Deterministic across runs (unlike SipHash),
/// so derived keys double as restart-stable idempotency keys. Not
/// cryptographic — keys are identifiers, never secrets.
fn fnv1a_hex(role: &str, content: &str) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in role
        .bytes()
        .chain(std::iter::once(0))
        .chain(content.bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

/// Outcome of the same-millisecond tie-break: either the message is a
/// replay (`Skip`) or it is new and persists under the returned key.
enum TieBreak {
    Skip,
    Accept(String),
}

/// Metadata keys used on L0 records (none use the reserved `__vanta_` prefix).
const META_ROLE: &str = "role";
const META_SESSION: &str = "session_id";
const META_TS: &str = "timestamp_ms";
const META_RECORDED_AT: &str = "recorded_at";

/// Cursor record key and namespace prefix.
const CURSOR_KEY: &str = "__cursor";

/// Character set allowed in VantaDB namespaces: `[A-Za-z0-9._/-]`, ≤128 bytes.
pub(crate) fn sanitize_component(s: &str, max_bytes: usize, allow_slash: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if out.len() + ch.len_utf8() > max_bytes {
            break;
        }
        let keep = ch.is_ascii_alphanumeric()
            || ch == '.'
            || ch == '_'
            || ch == '-'
            || (allow_slash && ch == '/');
        out.push(if keep { ch } else { '_' });
    }
    out
}

/// Message keys use the same safe set minus `/` (namespace separator).
pub(crate) fn sanitize_key(s: &str) -> String {
    sanitize_component(s, 512, false)
}

/// Persistent L0 recorder over the VantaDB SDK. Owns the [`Embedded`]
/// handle; the host must keep the recorder alive for the DB lifetime.
pub struct L0Recorder {
    db: Embedded,
}

impl L0Recorder {
    /// Open a recorder over an already-open embedded database.
    pub fn new(db: Embedded) -> Self {
        Self { db }
    }

    /// Capture a turn: filter by the persistent cursor, dedup in-batch by id,
    /// upsert each message under `l0/<session>`, then advance the cursor to
    /// the max recorded timestamp.
    ///
    /// `plugin_start_timestamp_ms` is the floor when no cursor exists yet
    /// (avoids dumping the whole session on first capture).
    pub fn record_turn(
        &self,
        capture: &L0Capture,
        plugin_start_timestamp_ms: Option<u64>,
    ) -> Result<L0CaptureResult, L0Error> {
        let session_ns = l0_namespace(&capture.session_id);
        let cursor_ns = cursor_namespace(&capture.session_id);

        let persisted_cursor = self.read_cursor(&cursor_ns)?;
        let cursor_ms = persisted_cursor.unwrap_or(plugin_start_timestamp_ms.unwrap_or(0));

        // Filter: messages older than the cursor are replays. A message AT the
        // cursor is a replay too — except when it is genuinely new (same-ms
        // race: two captures inside one millisecond, e.g. back-to-back
        // `/conversation/add`). The tie-break below tells them apart by
        // stored key+content, so no message is ever lost.
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut pending: Vec<(String, &L0Message)> = Vec::new();
        for (idx, msg) in capture.messages.iter().enumerate() {
            if msg.timestamp_ms < cursor_ms {
                continue;
            }
            let base = msg.id.clone().unwrap_or_else(|| {
                derived_key(msg.timestamp_ms, idx, &msg.role.to_string(), &msg.content)
            });
            let base = sanitize_key(&base);
            if msg.timestamp_ms == cursor_ms {
                match self.tie_break(&session_ns, &base, msg, persisted_cursor.is_some())? {
                    TieBreak::Skip => continue,
                    TieBreak::Accept(key) => {
                        if !seen.insert(key.clone()) {
                            continue; // duplicate within the same batch
                        }
                        pending.push((key, msg));
                    }
                }
                continue;
            }
            if !seen.insert(base.clone()) {
                continue; // duplicate within the same batch
            }
            pending.push((base, msg));
        }

        let mut recorded = Vec::with_capacity(pending.len());
        let mut new_cursor = cursor_ms;
        let recorded_at = now_ms();
        for (key, msg) in pending {
            let mut metadata = MemoryMetadata::new();
            metadata.insert(META_ROLE.into(), Value::String(msg.role.to_string()));
            metadata.insert(
                META_SESSION.into(),
                Value::String(capture.session_id.clone()),
            );
            metadata.insert(META_TS.into(), Value::Int(msg.timestamp_ms as i64));
            metadata.insert(META_RECORDED_AT.into(), Value::Int(recorded_at as i64));

            self.db.put(MemoryInput {
                namespace: session_ns.clone(),
                key,
                payload: msg.content.clone(),
                metadata,
                vector: None,
                sparse_vector: None,
                ttl_ms: None,
            })?;

            new_cursor = new_cursor.max(msg.timestamp_ms);
            recorded.push(msg.clone());
        }

        if !recorded.is_empty() {
            self.write_cursor(&cursor_ns, new_cursor)?;
        }

        let recorded_count = recorded.len();
        Ok(L0CaptureResult {
            recorded,
            recorded_count,
            cursor_ms: new_cursor,
        })
    }

    /// Read back all captured messages for a session, oldest first. The
    /// cursor record never appears here (it lives in a separate namespace;
    /// filtering `__cursor` is a defensive belt-and-suspenders).
    pub fn read_messages(&self, session_id: &str) -> Result<Vec<L0Message>, L0Error> {
        let session_ns = l0_namespace(session_id);
        let mut messages = Vec::new();
        let mut cursor: Option<usize> = None;

        loop {
            let options = MemoryListOptions {
                limit: 1000,
                cursor,
                ..Default::default()
            };
            let page: MemoryListPage = self.db.list(&session_ns, options)?;
            for record in page.records {
                if record.key == CURSOR_KEY {
                    continue;
                }
                if let Some(msg) = l0_message_from_record(&record) {
                    messages.push(msg);
                }
            }
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }

        messages.sort_by_key(|m| m.timestamp_ms);
        Ok(messages)
    }

    /// Read the persisted cursor for a session (if any).
    fn read_cursor(&self, cursor_ns: &str) -> Result<Option<u64>, L0Error> {
        match self.db.get(cursor_ns, CURSOR_KEY)? {
            Some(record) => {
                let value: serde_json::Value = serde_json::from_str(&record.payload)?;
                Ok(value
                    .get("after_timestamp_ms")
                    .and_then(serde_json::Value::as_u64))
            }
            None => Ok(None),
        }
    }

    /// Same-millisecond tie-break (CODEX-132): a message stamped exactly at
    /// the cursor is a replay when its key already stores the same
    /// role+content, and a genuinely new message otherwise. New messages
    /// reuse the free derived key, or probe a `_<n>` sequence suffix on
    /// collision — so back-to-back captures in one millisecond lose nothing.
    /// Runs only when a cursor was already persisted; without one (fresh
    /// session under a plugin-start floor) ties keep the legacy skip.
    fn tie_break(
        &self,
        session_ns: &str,
        base: &str,
        msg: &L0Message,
        cursor_persisted: bool,
    ) -> Result<TieBreak, L0Error> {
        if !cursor_persisted {
            return Ok(TieBreak::Skip);
        }
        match self.db.get(session_ns, base)? {
            None => Ok(TieBreak::Accept(base.to_string())),
            Some(stored) => {
                let same = l0_message_from_record(&stored)
                    .is_some_and(|m| m.role == msg.role && m.content == msg.content);
                if same {
                    return Ok(TieBreak::Skip);
                }
                for n in 1..1000u32 {
                    let candidate = format!("{base}_{n}");
                    if self.db.get(session_ns, &candidate)?.is_none() {
                        return Ok(TieBreak::Accept(candidate));
                    }
                }
                Err(L0Error::Vanta(Error::InvalidInput(format!(
                    "l0 key space exhausted for base {base:?}: 1000 same-ms collisions"
                ))))
            }
        }
    }

    fn write_cursor(&self, cursor_ns: &str, after_timestamp_ms: u64) -> Result<(), L0Error> {
        let payload = serde_json::json!({ "after_timestamp_ms": after_timestamp_ms }).to_string();
        self.db.put(MemoryInput {
            namespace: cursor_ns.into(),
            key: CURSOR_KEY.into(),
            payload,
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(())
    }
}

/// `l0/<sanitized-session>` — message records namespace.
fn l0_namespace(session_id: &str) -> String {
    format!("l0/{}", sanitize_component(session_id, 128, false))
}

/// `l0_cursor/<sanitized-session>` — cursor records namespace.
fn cursor_namespace(session_id: &str) -> String {
    format!("l0_cursor/{}", sanitize_component(session_id, 128, false))
}

/// Rebuild an [`L0Message`] from a stored record. Records missing role or
/// timestamp metadata are skipped (tracing::debug), never fatal.
fn l0_message_from_record(record: &MemoryRecord) -> Option<L0Message> {
    let role_str = match record.metadata.get(META_ROLE)? {
        Value::String(s) => s.clone(),
        _ => {
            tracing::debug!(key = %record.key, "l0 record missing role metadata");
            return None;
        }
    };
    let role = L0Role::from_str(&role_str).ok()?;
    let timestamp_ms = match record.metadata.get(META_TS)? {
        Value::Int(v) => u64::try_from(*v).ok()?,
        _ => {
            tracing::debug!(key = %record.key, "l0 record missing timestamp_ms metadata");
            return None;
        }
    };
    Some(L0Message {
        id: Some(record.key.clone()),
        role,
        content: record.payload.clone(),
        timestamp_ms,
    })
}

/// Current unix-epoch milliseconds; falls back to 0 if the clock is bogus
/// (never panics — a 0 `recorded_at` is a harmless anomaly, not data loss).
pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
