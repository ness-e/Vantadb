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
use std::time::{SystemTime, UNIX_EPOCH};

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

fn extract_text(content: &Value) -> Option<String> {
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

/// Execute a parsed command and return the reply text shown to the user.
pub fn execute(cmd: &MemCommand) -> String {
    match cmd.command.as_str() {
        "sync" => "✅ Session memory refreshed (skills / knowledge / tasks).".to_string(),
        "create-skill" if cmd.is_known() => {
            format!("✅ Skill creation queued from prompt: “{}”.", cmd.args)
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
            execute(&parsed).contains("mem:create-skill"),
            "help lists commands"
        );

        let body = json!({ "messages": [{"role": "user", "content": "mem:create-skill db migration summary"}] });
        let bytes = serde_json::to_vec(&body).expect("serialize");
        let parsed = parse(&bytes).expect("parsed");
        assert_eq!(parsed.args, "db migration summary");
        assert!(execute(&parsed).contains("db migration summary"));
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
        assert!(execute(&parsed).contains("Unknown command"));
        assert!(execute(&parsed).contains("mem:help"));
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
}
