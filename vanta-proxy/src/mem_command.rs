//! `mem:` in-band commands intercepted before forwarding (D33, TDAM parity:
//! `mem-command/parser.ts`). Disabled by default; enabled via config, a turn
//! whose last user message starts with `mem:` is answered locally instead of
//! reaching the upstream LLM.
//!
//! Known commands (TDAM `KNOWN_COMMANDS`, index.ts:24):
//! - `mem:sync`           — refresh session memory
//! - `mem:create-skill …` — create a skill from the given prompt
//! - `mem:help`           — command reference
//!
//! Strict-args rule (parser.ts:37-41): `help`/`sync` take NO arguments, so
//! `mem:help what is rust` is ordinary conversation passed through verbatim.
//! Unknown commands (`mem:foo`) still parse and get a typo-fallback message.

use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};

/// Namespace holding skills created via `mem:create-skill` (PRX-01).
pub const SKILLS_NAMESPACE: &str = "proxy-skills";

/// Monotonic disambiguator so two skills in the same millisecond keep both
/// records (same upsert-collision rationale as capture's `TURN_SEQ`).
static SKILL_SEQ: AtomicU64 = AtomicU64::new(0);

/// The three TDAM commands this proxy understands (D33).
pub const KNOWN_COMMANDS: [&str; 3] = ["sync", "create-skill", "help"];

/// A parsed `mem:` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemCommand {
    pub command: String,
    pub args: String,
}

impl MemCommand {
    pub fn is_known(&self) -> bool {
        KNOWN_COMMANDS.contains(&self.command.as_str())
    }
}

/// Detect a `mem:` command in a request body's last user message.
///
/// Text extraction is conservative (no per-client adapters here): string
/// content → itself; array content → concatenation of `text` blocks.
pub fn parse(body: &[u8]) -> Option<MemCommand> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let messages = value.get("messages")?.as_array()?;
    let last = messages.last()?;
    if last.get("role")?.as_str()? != "user" {
        return None;
    }
    let text = extract_text(last.get("content")?)?;
    parse_text(&text)
}

/// Plain text of a message `content`: string → itself; array → concatenation
/// of `text` blocks. Shared with turn capture (MEM-50).
pub(crate) fn extract_text(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            let mut text = String::new();
            for block in blocks {
                if block.get("type")?.as_str()? == "text" {
                    text.push_str(block.get("text")?.as_str()?);
                }
            }
            (!text.is_empty()).then_some(text)
        }
        _ => None,
    }
}

/// Classify already-extracted user text (TDAM `parseCommandFromText`).
pub fn parse_text(text: &str) -> Option<MemCommand> {
    let trimmed = text.trim();
    if !trimmed.to_ascii_lowercase().starts_with("mem:") {
        return None;
    }
    let after_prefix = trimmed[4..].trim_start();
    let (command, args) = match after_prefix.split_once(' ') {
        None => (after_prefix, ""),
        Some((c, rest)) => (c, rest.trim()),
    };
    if command.is_empty() {
        return None;
    }
    // Strict-args: help/sync accept none — trailing prose means conversation.
    if !args.is_empty() && matches!(command.to_ascii_lowercase().as_str(), "help" | "sync") {
        return None;
    }
    Some(MemCommand {
        command: command.to_ascii_lowercase(),
        args: args.to_string(),
    })
}

const HELP_TEXT: &str = "**VantaDB mem: commands**\n\n\
| Command | Effect |\n|---|---|\n\
| `mem:sync` | Refresh session memory (skills / knowledge / tasks) |\n\
| `mem:create-skill [prompt]` | Create a Skill from a prompt |\n\
| `mem:help` | Show this reference |\n\n\
Examples:\n```\nmem:sync\nmem:create-skill summarize database migration notes\nmem:help\n```";

/// Execute a parsed command against the real memory pipeline (PRX-01) and
/// return the reply text shown to the user.
/// - `sync` reads back this session's persisted turns (the D47 write path)
///   and reports the real count — empty session scopes to all turns.
/// - `create-skill` puts a real record in [`SKILLS_NAMESPACE`].
/// - `help` / unknown keep static text (no storage involved).
///
/// Storage failures degrade into message text: the wire never breaks.
pub fn execute(memory: &VantaEmbedded, session_key: &str, cmd: &MemCommand) -> String {
    match cmd.command.as_str() {
        "sync" => {
            let turns = crate::capture::list_turns(memory);
            let n = if session_key.is_empty() {
                turns.len()
            } else {
                turns
                    .iter()
                    .filter(|r| r.payload.contains(session_key))
                    .count()
            };
            format!(
                "✅ Session memory refreshed: {n} turn(s) stored \
                 for session `{session_key}` (skills / knowledge / tasks)."
            )
        }
        "create-skill" => {
            if cmd.args.trim().is_empty() {
                return "❌ `mem:create-skill` needs a prompt, e.g. \
                    `mem:create-skill summarize database migration notes`."
                    .to_string();
            }
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let key = format!("{now_ms}-{}", SKILL_SEQ.fetch_add(1, Ordering::Relaxed));
            let payload = serde_json::json!({
                "session": session_key,
                "prompt": cmd.args,
            })
            .to_string();
            let input = VantaMemoryInput {
                namespace: SKILLS_NAMESPACE.into(),
                key,
                payload,
                metadata: VantaMemoryMetadata::new(),
                vector: None,
                sparse_vector: None,
                ttl_ms: None,
            };
            match memory.put(input) {
                Ok(record) => format!(
                    "✅ Skill saved from prompt: “{}” ({} v{}).",
                    cmd.args, record.key, record.version
                ),
                Err(e) => format!("❌ couldn't save skill: {e}"),
            }
        }
        "help" => HELP_TEXT.to_string(),
        other => format!("❌ Unknown command `mem:{other}` — type `mem:help`."),
    }
}

/// Build the local HTTP response for an intercepted command. Unlike TDAM's
/// protocol-specific builders, this returns a plain JSON envelope on every
/// wire protocol — clients read `message`.
pub fn respond(message: &str) -> Response<Body> {
    let request_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let body = serde_json::json!({
        "id": format!("mem-cmd-{request_id}"),
        "object": "mem.command",
        "message": message,
    });
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        axum::Json(body),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn disabled_by_default_config_flag() {
        assert!(!crate::config::MemCommandConfig::default().enabled);
    }

    #[test]
    fn parses_sync_help_and_create_skill_from_last_user_message() {
        let body = json!({
            "model": "m",
            "messages": [
                {"role": "user", "content": "earlier question"},
                {"role": "assistant", "content": "answer"},
                {"role": "user", "content": "  MEM:SYNC  "}
            ]
        });
        let bytes = serde_json::to_vec(&body).expect("serialize");
        assert_eq!(
            parse(&bytes),
            Some(MemCommand {
                command: "sync".into(),
                args: String::new()
            }),
            "case-insensitive prefix, trimmed"
        );

        let body = json!({ "messages": [{"role": "user", "content": "mem:help"}] });
        let bytes = serde_json::to_vec(&body).expect("serialize");
        let parsed = parse(&bytes).expect("parsed");
        assert_eq!(parsed.command, "help");
        assert!(
            execute(&test_memory(), "", &parsed).contains("mem:create-skill"),
            "help lists commands"
        );

        let body = json!({ "messages": [{"role": "user", "content": "mem:create-skill db migration summary"}] });
        let bytes = serde_json::to_vec(&body).expect("serialize");
        let parsed = parse(&bytes).expect("parsed");
        assert_eq!(parsed.args, "db migration summary");
        assert!(execute(&test_memory(), "", &parsed).contains("db migration summary"));
    }

    #[test]
    fn strict_args_and_non_commands_pass_through() {
        // help/sync with trailing prose = ordinary conversation (TDAM parity).
        assert_eq!(parse_text("mem:help what is rust"), None);
        assert_eq!(parse_text("mem:sync now please"), None);
        // Not a command at all.
        assert_eq!(parse_text("remember mem: is fun"), None);
        assert_eq!(parse_text(""), None);
        // Bare prefix without a command name.
        assert_eq!(parse_text("mem:"), None);
        // Non-user last message is ignored.
        let bytes = serde_json::to_vec(&json!({
            "messages": [{"role": "assistant", "content": "mem:help"}]
        }))
        .expect("serialize");
        assert_eq!(parse(&bytes), None);
    }

    #[test]
    fn unknown_command_gets_typo_fallback_message() {
        let parsed = parse_text("mem:synk").expect("unknown still parses");
        assert!(!parsed.is_known());
        assert!(execute(&test_memory(), "", &parsed).contains("Unknown command"));
        assert!(execute(&test_memory(), "", &parsed).contains("mem:help"));
    }

    #[test]
    fn array_content_blocks_extracted_conservatively() {
        let body = json!({
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": "meta: ignore"},
                    {"type": "text", "text": "mem:help"}
                ]
            }]
        });
        let bytes = serde_json::to_vec(&body).expect("serialize");
        // Concatenated text does NOT start with mem: → passthrough (conservative).
        assert_eq!(parse(&bytes), None);

        let body = json!({
            "messages": [{
                "role": "user",
                "content": [{"type": "text", "text": "mem:help"}]
            }]
        });
        let bytes = serde_json::to_vec(&body).expect("serialize");
        assert_eq!(
            parse(&bytes),
            Some(MemCommand {
                command: "help".into(),
                args: String::new()
            })
        );
    }

    #[tokio::test]
    async fn respond_returns_200_json_envelope_with_message() {
        let resp = respond("hello");
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let value: Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(value["object"], "mem.command");
        assert_eq!(value["message"], "hello");
        assert!(value["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("mem-cmd-")));
    }

    #[test]
    fn known_commands_constant_matches_tdam_trio() {
        assert_eq!(KNOWN_COMMANDS, ["sync", "create-skill", "help"]);
    }

    fn test_memory() -> vantadb::sdk::VantaEmbedded {
        let config = vantadb::config::VantaConfig {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            ..Default::default()
        };
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
            .map(|engine| vantadb::sdk::VantaEmbedded::from_engine(engine.into()))
            .expect("in-memory engine")
    }

    #[tokio::test]
    async fn sync_reports_real_turn_count_for_session() {
        // PRX-01: `mem:sync` runs the real pipeline (stored turns), not a stub.
        let memory = test_memory();
        for text in ["first turn", "second turn"] {
            crate::capture::turn_job(memory.clone(), "sess-sync", "anthropic", "sp", "m", text)()
                .await
                .expect("seed turn");
        }
        crate::capture::turn_job(memory.clone(), "other", "anthropic", "sp", "m", "elsewhere")()
            .await
            .expect("seed turn");

        let sync = MemCommand {
            command: "sync".into(),
            args: String::new(),
        };
        let msg = execute(&memory, "sess-sync", &sync);
        assert!(msg.contains('2'), "real count for this session, got: {msg}");
        assert!(msg.contains("sess-sync"));

        let msg = execute(&memory, "nobody", &sync);
        assert!(msg.contains('0'), "empty session reports zero, got: {msg}");
    }

    #[test]
    fn create_skill_persists_a_real_record() {
        // PRX-01: `mem:create-skill` puts a real record in `proxy-skills`.
        let memory = test_memory();
        let cmd = MemCommand {
            command: "create-skill".into(),
            args: "summarize db migration notes".into(),
        };
        let msg = execute(&memory, "sess-skill", &cmd);
        assert!(msg.contains("summarize db migration notes"), "got: {msg}");

        let page = memory
            .list(
                SKILLS_NAMESPACE,
                vantadb::sdk::VantaMemoryListOptions {
                    limit: 10,
                    ..Default::default()
                },
            )
            .expect("list skills");
        assert_eq!(page.records.len(), 1);
        assert!(page.records[0]
            .payload
            .contains("summarize db migration notes"));
        assert!(page.records[0].payload.contains("sess-skill"));
    }

    #[test]
    fn create_skill_rejects_empty_prompt_and_degrades_storage_errors() {
        let memory = test_memory();
        let empty = MemCommand {
            command: "create-skill".into(),
            args: String::new(),
        };
        assert!(execute(&memory, "s", &empty).contains("needs a prompt"));

        // Read-only engine → put fails → descriptive text, never a panic.
        let ro_config = vantadb::config::VantaConfig {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            read_only: true,
            ..Default::default()
        };
        let ro = vantadb::storage::StorageEngine::open_with_config(":memory:", Some(ro_config))
            .map(|engine| vantadb::sdk::VantaEmbedded::from_engine(engine.into()))
            .expect("ro engine");
        let cmd = MemCommand {
            command: "create-skill".into(),
            args: "something".into(),
        };
        let msg = execute(&ro, "s", &cmd);
        assert!(msg.contains("couldn't save"), "got: {msg}");
    }
}
