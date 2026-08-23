//! L0 turn capture (MEM-50 / D47): a completed proxied request is tracked
//! through [`crate::writeback::WriteBack`] so the conversation turn lands in
//! memory without ever blocking or failing the wire. This is the single write
//! path for L0 turns — the same one the capture tool uses.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde_json::Value;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryMetadata, VantaMemoryRecord,
};

use crate::writeback::L0Job;

/// Namespace holding proxied conversation turns.
pub const TURNS_NAMESPACE: &str = "proxy-turns";

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
pub fn turn_job(
    memory: VantaEmbedded,
    session_key: &str,
    protocol: &str,
    space_id: &str,
    model: &str,
    text: &str,
) -> L0Job {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let key = format!("{now_ms}-{}", TURN_SEQ.fetch_add(1, Ordering::Relaxed));
    let payload = serde_json::json!({
        "session": session_key,
        "protocol": protocol,
        "space": space_id,
        "model": model,
        "text": text,
    })
    .to_string();

    Arc::new(move || {
        let memory = memory.clone();
        let input = VantaMemoryInput {
            namespace: TURNS_NAMESPACE.into(),
            key: key.clone(),
            payload: payload.clone(),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        };
        Box::pin(async move {
            // ponytail: sync storage op inside async — proxy scale tolerates it;
            // move to spawn_blocking if puts ever show in latency profiles.
            memory.put(input).map(|_| ()).map_err(|e| e.to_string())
        })
    })
}

/// Read back every persisted turn record (tests / tooling).
pub fn list_turns(db: &VantaEmbedded) -> Vec<VantaMemoryRecord> {
    db.list(
        TURNS_NAMESPACE,
        VantaMemoryListOptions {
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

    fn memory() -> VantaEmbedded {
        let config = vantadb::config::VantaConfig {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            ..Default::default()
        };
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
            .map(|engine| VantaEmbedded::from_engine(engine.into()))
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
        let inner = turn_job(
            db.clone(),
            "sess-d19",
            "openai",
            "space",
            "m",
            "hello world",
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
}
