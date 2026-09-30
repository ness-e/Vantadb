//! L0 turn capture (MEM-50 / D47): a completed proxied request is tracked
//! through [`crate::writeback::WriteBack`] so the conversation turn lands in
//! memory without ever blocking or failing the wire. This is the single write
//! path for L0 turns — the same one the capture tool uses.
//!
//! VER-03: both destinations (`proxy-turns` and `l1/{session}`) persist the
//! **redacted** text (same detector kinds as the egress [`Redactor`]); the
//! pre-redaction original only survives inside the per-namespace AEAD
//! envelope attached to the audit payload — never in clear.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde_json::Value;
use vanta_memory::core::abstractions::{MemoryRecord as L1Record, MemoryType};
use vanta_memory::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use vanta_memory::core::record::l1_reader::l1_namespace;
use vantadb::sdk::{
    Embedded, MemoryInput, MemoryListOptions, MemoryMetadata, MemoryRecord, Value as SdkValue,
};

use crate::envelope::Envelope;
use crate::redact::Redactor;
use crate::writeback::L0Job;

/// Namespace holding proxied conversation turns.
pub const TURNS_NAMESPACE: &str = "proxy-turns";

/// Write-path guard (VER-03): the on-write redaction policy plus the optional
/// per-namespace envelope that keeps the pre-redaction original.
pub struct WriteGuard<'a> {
    /// Redaction policy (the egress `[redact]` config), reused on write.
    pub redactor: &'a Redactor,
    /// Per-namespace AEAD envelope (off/disarmed → redacted-only).
    pub envelope: &'a Envelope,
}

/// Monotonic disambiguator so two turns in the same millisecond keep both
/// records (`key = {now_ms}-{seq}`; upsert semantics would drop one otherwise).
static TURN_SEQ: AtomicU64 = AtomicU64::new(0);

/// Extract the plain text of the LAST user message from a `messages` array
/// (string content, or array of blocks → the LAST `{type:"text"}` block).
///
/// Refined by MEM-57: delegates array extraction to the Claude Code adapter
/// (`session::claude_code::extract_last_user_text`), which scans backwards so
/// CC's prepended `<system-reminder>` metadata blocks no longer pollute the
/// captured turn. Non-JSON bodies yield `None`.
pub fn last_user_text(body: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let messages = value.get("messages")?.as_array()?;
    let last = messages.last()?;
    if last.get("role")?.as_str()? != "user" {
        return None;
    }
    crate::session::claude_code::extract_last_user_text(last.get("content")?)
}

/// Build the retryable L0 job persisting one conversation turn record.
///
/// Dual-write (WIRE-01 opción B): the turn lands in `proxy-turns` (L0 audit
/// trail, unchanged) AND as an `Episodic` record in `l1/{session}` so the
/// search path (`perform_auto_recall`, `vanta_memory_search` tool) retrieves
/// it — that closes the capture→recall loop without perturbing the injected
/// system block (PRX-04 prefix stability + PRX-09 exact cache keep working).
/// Tenancy stays `None` (LLM08: session-only, never cross-session). Either
/// put failing returns `Err` so WriteBack retries both (idempotent upsert by
/// key — a retry overwrites the same two records).
///
/// VER-03: `guard` applies redaction-on-write over `text`; both persisted
/// records carry the masked version and the pre-redaction original only
/// survives inside the per-namespace envelope (audit payload). This function
/// never blocks or drops a turn (D47) and never persists a clear original.
pub fn turn_job(
    memory: Embedded,
    session_key: &str,
    protocol: &str,
    space_id: &str,
    model: &str,
    text: &str,
    guard: &WriteGuard<'_>,
) -> L0Job {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let key = format!("{now_ms}-{}", TURN_SEQ.fetch_add(1, Ordering::Relaxed));
    // VER-03: persist the redacted text — the egress policy, applied on write.
    let write = guard.redactor.for_write(text);
    let mut payload = serde_json::json!({
        "session": session_key,
        "protocol": protocol,
        "space": space_id,
        "model": model,
        "text": write.text.clone(),
    });
    if guard.redactor.enabled() {
        // Kind labels only (never values): traceability of what was redacted.
        payload["redacted"] = serde_json::json!(write.kinds);
    }
    // Envelope only when masking changed the text: if nothing was redacted the
    // original IS the persisted text, so sealing would duplicate it.
    let original_envelope = if write.masked {
        guard.envelope.seal(TURNS_NAMESPACE, text)
    } else {
        None
    };
    if let Some(blob) = original_envelope {
        payload["original_envelope"] = serde_json::to_value(blob).unwrap_or(Value::Null);
    }
    let payload = payload.to_string();
    // Canonical L1 shape (mirrors `l1_writer::put_record`: namespace
    // `l1/{session}`, key = record id, payload = serialized record,
    // metadata tags {type, priority}). Keys are `[0-9-]` by construction,
    // so the canonical `sanitize_key` is identity here — no dependency on
    // the `pub(crate)` helper.
    let now_rfc = epoch_ms_to_rfc3339(now_ms);
    let l1_record = L1Record {
        id: key.clone(),
        content: write.text.clone(),
        memory_type: MemoryType::Episodic,
        priority: 50,
        scene_name: String::new(),
        source_message_ids: Vec::new(),
        metadata: Value::Null,
        timestamps: vec![now_rfc.clone()],
        created_at: now_rfc.clone(),
        updated_at: now_rfc,
        version: 1,
        session_key: session_key.to_string(),
        session_id: String::new(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    };
    let l1_namespace = l1_namespace(session_key);

    Arc::new(move || {
        let memory = memory.clone();
        let input = MemoryInput {
            namespace: TURNS_NAMESPACE.into(),
            key: key.clone(),
            payload: payload.clone(),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        };
        let l1_record = l1_record.clone();
        let l1_namespace = l1_namespace.clone();
        Box::pin(async move {
            // ponytail: sync storage op inside async — proxy scale tolerates it;
            // move to spawn_blocking if puts ever show in latency profiles.
            memory.put(input).map_err(|e| e.to_string())?;
            let l1_payload = serde_json::to_string(&l1_record).map_err(|e| e.to_string())?;
            let mut metadata = MemoryMetadata::new();
            metadata.insert("type".into(), SdkValue::String("episodic".into()));
            metadata.insert("priority".into(), SdkValue::Int(50));
            memory
                .put(MemoryInput {
                    namespace: l1_namespace,
                    key: l1_record.id.clone(),
                    payload: l1_payload,
                    metadata,
                    vector: None,
                    sparse_vector: None,
                    ttl_ms: None,
                    ..Default::default()
                })
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    })
}

/// Read back every persisted turn record (tests / tooling).
pub fn list_turns(db: &Embedded) -> Vec<MemoryRecord> {
    db.list(
        TURNS_NAMESPACE,
        MemoryListOptions {
            limit: 100,
            ..Default::default()
        },
    )
    .map(|page| page.records)
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use crate::writeback::DEFAULT_ATTEMPTS;

    fn memory() -> Embedded {
        let config = vantadb::config::Config {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            ..Default::default()
        };
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
            .map(|engine| Embedded::from_engine(engine.into()))
            .expect("in-memory engine")
    }

    #[test]
    fn extracts_last_user_string() {
        let body = br#"{"messages":[{"role":"system","content":"s"},{"role":"user","content":"hi there"}]}"#;
        assert_eq!(last_user_text(body).as_deref(), Some("hi there"));
    }

    #[test]
    fn extracts_last_user_block_array_last_text_block_wins() {
        // MEM-57 refinement: the LAST text block is the typed input; earlier
        // blocks (e.g. CC <system-reminder> metadata) are not captured.
        let body = br#"{"messages":[{"role":"user","content":[{"type":"text","text":"<system-reminder>x</system-reminder>"},{"type":"text","text":"real input"}]}]}"#;
        assert_eq!(last_user_text(body).as_deref(), Some("real input"));
    }

    #[test]
    fn ignores_non_user_tail_and_garbage() {
        let assistant_tail =
            br#"{"messages":[{"role":"user","content":"u"},{"role":"assistant","content":"a"}]}"#;
        assert_eq!(last_user_text(assistant_tail), None);
        assert_eq!(last_user_text(b"not json"), None);
    }

    /// D19 mechanics through the real WriteBack coordinator: first attempt
    /// fails → retries exhaust → job visible in pending queue → flush runs it
    /// again and persists the record.
    #[tokio::test]
    async fn failed_enqueue_lands_in_pending_and_flush_persists() {
        let db = memory();
        let wb = crate::writeback::WriteBack::new(None);
        // Fail exactly the 3 retry attempts so the job exhausts them and lands
        // in the pending queue; the post-flush invocation succeeds.
        let remaining = Arc::new(AtomicU64::new(u64::from(DEFAULT_ATTEMPTS)));
        let left = remaining.clone();
        // Default guard: redaction off + envelope off — legacy write shape.
        let redactor = crate::redact::Redactor::new(&crate::redact::RedactConfig::default())
            .expect("redactor");
        let envelope = crate::envelope::Envelope::with_master(
            &crate::envelope::EnvelopeConfig::default(),
            None,
        );
        let guard = WriteGuard {
            redactor: &redactor,
            envelope: &envelope,
        };
        let inner = turn_job(
            db.clone(),
            "sess-d19",
            "openai",
            "space",
            "m",
            "hello world",
            &guard,
        );
        let job: L0Job = Arc::new(move || {
            let left = left.clone();
            let inner = inner.clone();
            Box::pin(async move {
                if left.fetch_sub(1, Ordering::SeqCst) >= 1 {
                    return Err("simulated l0 failure".into());
                }
                inner().await
            })
        });
        wb.track("turn:sess-d19", job);

        // Retries back off 500ms→1s→2s; poll until exhaustion lands the job
        // in the pending queue (fixed sleeps are flaky under load).
        let mut queued = false;
        for _ in 0..100 {
            if wb.pending_count() == 1 {
                queued = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(queued, "failed enqueue visible in queue");
        assert!(list_turns(&db).is_empty(), "nothing persisted yet");

        wb.flush(Duration::from_secs(5)).await;
        assert_eq!(wb.pending_count(), 0, "flush drained the queue");
        let turns = list_turns(&db);
        assert_eq!(turns.len(), 1);
        assert!(turns[0].payload.contains("hello world"));
        assert!(turns[0].payload.contains("sess-d19"));
    }

    /// WIRE-01 (opción B) RED: el job L0 persiste el turno TAMBIÉN como
    /// registro L1 `Episodic` en `l1/{session}` para que el search lo
    /// recupere. Hoy FAIL: solo escribe `proxy-turns` → search da 0 hits.
    #[tokio::test]
    async fn turn_job_dual_writes_l1_record_for_recall() {
        use vanta_memory::core::record::l1_reader::read_session_records;
        let db = memory();
        let redactor = crate::redact::Redactor::new(&crate::redact::RedactConfig::default())
            .expect("redactor");
        let envelope = crate::envelope::Envelope::with_master(
            &crate::envelope::EnvelopeConfig::default(),
            None,
        );
        let guard = WriteGuard {
            redactor: &redactor,
            envelope: &envelope,
        };
        let job = turn_job(
            db.clone(),
            "sess-l1",
            "openai",
            "space",
            "m",
            "xylophone-quasar-7429 prefers concise answers",
            &guard,
        );
        job().await.expect("l0 job runs");
        let records = read_session_records(&db, "sess-l1").expect("read l1");
        assert_eq!(records.len(), 1, "turn must land in l1 per session");
        assert!(records[0].content.contains("xylophone-quasar-7429"));
        assert_eq!(records[0].session_key, "sess-l1");
    }
}
