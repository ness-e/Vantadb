//! One-shot MCP tool call bridge (WIRE-10): `vanta-cli mcp-call`.
//!
//! Spawns `vantadb-server --mcp` with piped stdio, sends `initialize` +
//! `tools/call` as line-delimited JSON-RPC 2.0 (the wire format
//! `vantadb-mcp::server::serve_lines` speaks), and prints the tool `result`
//! verbatim to stdout. Lets hook templates call MCP tools without `pwsh`.
//!
//! Exit codes: 0 ok · 1 infra (spawn/io/timeout/protocol) ·
//! 2 tool-level error (MCP error channel or `isError` result).
//! Errors never leak secrets: only tool names, arg shapes and server
//! stderr passthrough (which the server keeps secret-free).

use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::time::Duration;

use crate::error::{ChainedError, Result};

/// Protocol version offered at `initialize` (server negotiates forward).
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Outcome of one JSON-RPC response line, mapped to process exit codes.
#[derive(Debug, PartialEq, Eq)]
pub enum CallOutcome {
    /// Tool result to print verbatim on stdout (exit 0).
    Ok(serde_json::Value),
    /// Tool-level error: message for stderr (exit 2).
    ToolError(String),
    /// Infra/protocol failure: message for stderr (exit 1).
    Infra(String),
}

/// Parse `--args` into a JSON object (must be an object, not any value).
pub fn parse_tool_args(raw: &str) -> Result<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
        crate::error::Error::Cli(ChainedError::msg(format!(
            "mcp-call: --args is not valid JSON: {e}"
        )))
    })?;
    if !v.is_object() {
        return Err(crate::error::Error::Cli(ChainedError::msg(
            "mcp-call: --args must be a JSON object, e.g. '{\"query\":\"x\"}'",
        )));
    }
    Ok(v)
}

/// Cap for hook-input stdin reads (hooks pass small JSON blobs; bound the
/// read so a stray pipe can't OOM the one-shot).
const MAX_STDIN_BYTES: usize = 1_048_576;

/// Resolve `{{dotted.path}}` placeholders in an `--args` template against a
/// hook-input JSON object. Values must be strings and are JSON-escaped into
/// the template. No placeholders → template returned verbatim (stdin is
/// never touched, so TTY runs can't block).
pub fn resolve_args_template(
    template: &str,
    stdin_json: Option<&serde_json::Value>,
) -> Result<String> {
    if !template.contains("{{") {
        return Ok(template.to_string());
    }
    let input = stdin_json.ok_or_else(|| {
        crate::error::Error::Cli(ChainedError::msg(
            "mcp-call: --args has {{placeholders}} but no hook-input JSON was provided on stdin",
        ))
    })?;
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after_open = &rest[start + 2..];
        match after_open.find("}}") {
            None => {
                return Err(crate::error::Error::Cli(ChainedError::msg(
                    "mcp-call: unterminated {{placeholder}} in --args",
                )));
            }
            Some(end) => {
                out.push_str(&rest[..start]);
                let path = after_open[..end].trim();
                if path.is_empty() {
                    return Err(crate::error::Error::Cli(ChainedError::msg(
                        "mcp-call: empty {{}} placeholder in --args",
                    )));
                }
                let mut value = input;
                for key in path.split('.') {
                    value = value.get(key).ok_or_else(|| {
                        crate::error::Error::Cli(ChainedError::msg(format!(
                            "mcp-call: hook input has no field '{{{path}}}'"
                        )))
                    })?;
                }
                let s = value.as_str().ok_or_else(|| {
                    crate::error::Error::Cli(ChainedError::msg(format!(
                        "mcp-call: hook input field '{{{path}}}' is not a string"
                    )))
                })?;
                // JSON-escape (without surrounding quotes) for safe inline splice.
                let escaped = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
                out.push_str(escaped.trim_matches('"'));
                rest = &after_open[end + 2..];
            }
        }
    }
    out.push_str(rest);
    Ok(out)
}

/// Read hook-input JSON from stdin (capped). `None` on empty stdin.
fn read_stdin_json() -> Result<Option<serde_json::Value>> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin()
        .take((MAX_STDIN_BYTES + 1) as u64)
        .read_to_string(&mut buf)
        .map_err(|e| {
            crate::error::Error::Cli(ChainedError::msg(format!(
                "mcp-call: failed reading stdin: {e}"
            )))
        })?;
    if buf.len() > MAX_STDIN_BYTES {
        return Err(crate::error::Error::Cli(ChainedError::msg(
            "mcp-call: stdin exceeds 1 MiB; refusing to parse as hook input",
        )));
    }
    if buf.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(&buf).map_err(|e| {
        crate::error::Error::Cli(ChainedError::msg(format!(
            "mcp-call: stdin is not valid hook-input JSON: {e}"
        )))
    })
}

/// Build one line-delimited JSON-RPC 2.0 request line (no trailing newline).
pub fn build_request(id: u64, method: &str, params: serde_json::Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    })
    .to_string()
}

/// Classify one server response line for the expected `id`.
pub fn classify_response(line: &str, expect_id: u64) -> CallOutcome {
    let v: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return CallOutcome::Infra(format!("mcp-call: server sent non-JSON response: {e}"));
        }
    };
    let id_ok = v.get("id").is_some_and(|id| id.as_u64() == Some(expect_id));
    if !id_ok {
        return CallOutcome::Infra(format!(
            "mcp-call: response id mismatch (expected {expect_id}): {}",
            truncate(line, 200)
        ));
    }
    if let Some(err) = v.get("error") {
        let code = match err.get("code") {
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => "?".to_string(),
        };
        return CallOutcome::ToolError(format!(
            "mcp-call: MCP error {code}: {}",
            err.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("(no message)")
        ));
    }
    match v.get("result") {
        None => CallOutcome::Infra("mcp-call: response has neither result nor error".to_string()),
        Some(result) => {
            if result.get("isError").and_then(serde_json::Value::as_bool) == Some(true) {
                let text = result
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|b| b.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("(no text)");
                CallOutcome::ToolError(format!("mcp-call: tool returned isError: {text}"))
            } else {
                CallOutcome::Ok(result.clone())
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // P2-01: cortar en char-boundary (slicing en byte no-boundary panica).
    let end = s.floor_char_boundary(max);
    format!("{}…", &s[..end])
}

/// Locate `vantadb-server[.exe]`: PATH first, then next to this executable.
/// Mirrors `cmd_server_mcp` resolution without touching its spawn path.
fn resolve_server_binary() -> Result<std::path::PathBuf> {
    let exe_name = if cfg!(windows) {
        "vantadb-server.exe"
    } else {
        "vantadb-server"
    };
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let cand = dir.join(exe_name);
        if cand.is_file() {
            return Ok(cand);
        }
    }
    if let Ok(mut current_exe) = std::env::current_exe() {
        current_exe.set_file_name(exe_name);
        if current_exe.is_file() {
            return Ok(current_exe);
        }
    }
    Err(crate::error::Error::Cli(ChainedError::msg(format!(
        "mcp-call: vantadb-server binary not found. Searched PATH for '{exe_name}' \
         and the CLI directory. Build it with 'cargo build --bin vantadb-server' \
         or place it alongside vanta-cli."
    ))))
}

/// Run one tool call end-to-end. Returns the process exit code
/// (0 ok · 1 infra · 2 tool error); prints result to stdout.
pub fn cmd_mcp_call(db_path: &str, tool: &str, args_raw: &str, timeout_secs: u64) -> Result<i32> {
    // Hook templates splice client event fields (e.g. the verbatim user
    // prompt) as {{dotted.path}} against the hook-input JSON on stdin.
    // Stdin is read only when the template actually has placeholders.
    let args_resolved = if args_raw.contains("{{") {
        let stdin_json = read_stdin_json()?;
        match resolve_args_template(args_raw, stdin_json.as_ref()) {
            Ok(resolved) => resolved,
            Err(e) => {
                eprintln!("{e}");
                return Ok(2);
            }
        }
    } else {
        args_raw.to_string()
    };
    let args = match parse_tool_args(&args_resolved) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}");
            return Ok(2);
        }
    };
    let server_bin = resolve_server_binary()?;
    let timeout = Duration::from_secs(timeout_secs.max(1));

    let mut child = std::process::Command::new(&server_bin)
        .env("VANTADB_STORAGE_PATH", db_path)
        .arg("--mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| {
            crate::error::Error::Cli(ChainedError::msg(format!(
                "mcp-call: failed to spawn {}: {e}",
                server_bin.display()
            )))
        })?;

    let mut stdin = child.stdin.take().ok_or_else(|| {
        crate::error::Error::Cli(ChainedError::msg("mcp-call: server stdin unavailable"))
    })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        crate::error::Error::Cli(ChainedError::msg("mcp-call: server stdout unavailable"))
    })?;

    // Reader thread: pump response lines into a channel (main applies timeout).
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if l.trim().is_empty() {
                        continue;
                    }
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut roundtrip =
        |id: Option<u64>, method: &str, params: serde_json::Value| -> Result<String> {
            let req = match id {
                // JSON-RPC notification (no id): write + flush, never wait.
                None => serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": method,
                    "params": params,
                })
                .to_string(),
                Some(id) => build_request(id, method, params),
            };
            writeln!(stdin, "{req}").map_err(|e| {
                crate::error::Error::Cli(ChainedError::msg(format!(
                    "mcp-call: failed writing {method} request: {e}"
                )))
            })?;
            stdin.flush().map_err(|e| {
                crate::error::Error::Cli(ChainedError::msg(format!(
                    "mcp-call: failed flushing {method} request: {e}"
                )))
            })?;
            let id = match id {
                Some(id) => id,
                None => return Ok(String::new()),
            };
            // Skip lines that don't carry our id (e.g. answers to notifications
            // never arrive; pipelined background responses might).
            let deadline = std::time::Instant::now() + timeout;
            loop {
                let now = std::time::Instant::now();
                if now >= deadline {
                    // P2-01: reap tras kill (evita zombie hasta la salida).
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(crate::error::Error::Cli(ChainedError::msg(format!(
                        "mcp-call: timed out after {timeout_secs}s waiting for {method} response"
                    ))));
                }
                match rx.recv_timeout(deadline - now) {
                    Ok(line) => {
                        if line_contains_id(&line, id) {
                            return Ok(line);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // P2-01: reap tras kill (evita zombie hasta la salida).
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(crate::error::Error::Cli(ChainedError::msg(format!(
                        "mcp-call: timed out after {timeout_secs}s waiting for {method} response"
                    ))));
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        let _ = child.wait();
                        return Err(crate::error::Error::Cli(ChainedError::msg(format!(
                            "mcp-call: server closed stdout while waiting for {method}"
                        ))));
                    }
                }
            }
        };

    let exit_code = run_session(&mut roundtrip, tool, args)?;

    // EOF lets the server drain in-flight work and exit (serve_lines breaks
    // on Ok(None)); then reap it so no orphan holds the DB lock.
    drop(stdin);
    let _ = child.wait();
    Ok(exit_code)
}

/// Best-effort id match without full parse (cheap pre-filter; the
/// authoritative check is `classify_response`). Guards against prefix
/// collisions (`"id":2` inside `"id":22`) by requiring a non-digit after.
fn line_contains_id(line: &str, id: u64) -> bool {
    for needle in [format!("\"id\":{id}"), format!("\"id\": {id}")] {
        let mut rest = line;
        while let Some(pos) = rest.find(&needle) {
            let after = rest[pos + needle.len()..].chars().next();
            match after {
                Some(c) if c.is_ascii_digit() => rest = &rest[pos + needle.len()..],
                _ => return true,
            }
        }
    }
    false
}

fn run_session(
    roundtrip: &mut dyn FnMut(Option<u64>, &str, serde_json::Value) -> Result<String>,
    tool: &str,
    args: serde_json::Value,
) -> Result<i32> {
    let init_line = roundtrip(
        Some(1),
        "initialize",
        serde_json::json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "vanta-cli-mcp-call", "version": env!("CARGO_PKG_VERSION")},
        }),
    )?;
    if let CallOutcome::Infra(msg) = classify_response(&init_line, 1) {
        eprintln!("{msg}");
        return Ok(1);
    }
    // `notifications/initialized` is a pure lifecycle ack (no id, no answer).
    let _ = roundtrip(None, "notifications/initialized", serde_json::json!({}));

    let call_line = roundtrip(
        Some(2),
        "tools/call",
        serde_json::json!({"name": tool, "arguments": args}),
    )?;
    match classify_response(&call_line, 2) {
        CallOutcome::Ok(result) => {
            println!(
                "{}",
                serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            );
            Ok(0)
        }
        CallOutcome::ToolError(msg) => {
            // Tool errors still carry payload: print result for hook parsing.
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&call_line) {
                if let Some(result) = v.get("result") {
                    println!(
                        "{}",
                        serde_json::to_string(result).unwrap_or_else(|_| "{}".to_string())
                    );
                }
            }
            eprintln!("{msg}");
            Ok(2)
        }
        CallOutcome::Infra(msg) => {
            eprintln!("{msg}");
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_must_be_json_object() {
        assert!(parse_tool_args(r#"{"query":"x","top_k":5}"#).is_ok());
        assert!(parse_tool_args("{}").is_ok());
        assert!(parse_tool_args("[1,2]").is_err());
        assert!(parse_tool_args("42").is_err());
        assert!(parse_tool_args("not json").is_err());
    }

    #[test]
    fn request_is_line_jsonrpc_with_id() {
        let line = build_request(2, "tools/call", serde_json::json!({"name": "t"}));
        assert!(!line.contains('\n'));
        let v: serde_json::Value = serde_json::from_str(&line).expect("valid json");
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 2);
        assert_eq!(v["method"], "tools/call");
    }

    #[test]
    fn ok_result_passes_through_verbatim() {
        let payload = serde_json::json!({"prepend_context": "ctx", "recalled": []});
        let line = serde_json::json!({"jsonrpc": "2.0", "id": 2, "result": payload}).to_string();
        assert_eq!(classify_response(&line, 2), CallOutcome::Ok(payload));
    }

    #[test]
    fn jsonrpc_error_is_tool_error() {
        let line =
            r#"{"jsonrpc":"2.0","id":2,"error":{"code":-32602,"message":"Missing 'query'"}}"#;
        let out = classify_response(line, 2);
        assert!(matches!(out, CallOutcome::ToolError(_)));
        if let CallOutcome::ToolError(msg) = out {
            assert!(msg.contains("-32602") && msg.contains("Missing 'query'"));
        }
    }

    #[test]
    fn is_error_result_is_tool_error() {
        let line = r#"{"jsonrpc":"2.0","id":2,"result":{"isError":true,"content":[{"type":"text","text":"boom"}]}}"#;
        assert!(matches!(
            classify_response(line, 2),
            CallOutcome::ToolError(_)
        ));
    }

    #[test]
    fn id_mismatch_and_garbage_are_infra() {
        let line = r#"{"jsonrpc":"2.0","id":99,"result":{}}"#;
        assert!(matches!(classify_response(line, 2), CallOutcome::Infra(_)));
        assert!(matches!(
            classify_response("not json", 2),
            CallOutcome::Infra(_)
        ));
        let no_result = r#"{"jsonrpc":"2.0","id":2}"#;
        assert!(matches!(
            classify_response(no_result, 2),
            CallOutcome::Infra(_)
        ));
    }

    #[test]
    fn id_mismatch_with_multibyte_never_panics() {
        // P2-01: truncate cortaba en byte 200 sin respetar char-boundary.
        let big = "memoria con emojis 🧠 y CJK 漢字 ".repeat(20);
        let line = format!(r#"{{"jsonrpc":"2.0","id":99,"result":"{big}"}}"#);
        assert!(matches!(classify_response(&line, 2), CallOutcome::Infra(_)));
    }

    #[test]
    fn id_prefilter_matches_both_spacings() {
        assert!(line_contains_id(r#"{"id":2,"x":1}"#, 2));
        assert!(line_contains_id(r#"{"id": 2, "x": 1}"#, 2));
        assert!(!line_contains_id(r#"{"id":22}"#, 2));
    }

    #[test]
    fn template_without_placeholders_passes_through() {
        let tpl = r#"{"query":"x","top_k":5}"#;
        assert_eq!(resolve_args_template(tpl, None).expect("passthrough"), tpl);
    }

    #[test]
    fn template_substitutes_hook_fields_json_escaped() {
        let input: serde_json::Value =
            serde_json::from_str(r#"{"prompt":"say \"hi\"\nnow","a":{"b":"deep"}}"#)
                .expect("fixture");
        let out = resolve_args_template(
            r#"{"query":"{{prompt}}","nested":"{{a.b}}","top_k":5}"#,
            Some(&input),
        )
        .expect("resolve");
        // Valid JSON after splice, values escaped, nesting works.
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid json");
        assert_eq!(v["query"], "say \"hi\"\nnow");
        assert_eq!(v["nested"], "deep");
        assert_eq!(v["top_k"], 5);
    }

    #[test]
    fn template_errors_are_typed() {
        let input: serde_json::Value = serde_json::from_str(r#"{"n":1}"#).expect("fixture");
        assert!(resolve_args_template(r#"{"q":"{{missing}}"}"#, Some(&input)).is_err());
        assert!(resolve_args_template(r#"{"q":"{{n}}"}"#, Some(&input)).is_err());
        assert!(resolve_args_template(r#"{"q":"{{oops}"}"#, Some(&input)).is_err());
        assert!(resolve_args_template(r#"{"q":"{{}}"}"#, Some(&input)).is_err());
        assert!(resolve_args_template(r#"{"q":"{{prompt}}"}"#, None).is_err());
    }
}
