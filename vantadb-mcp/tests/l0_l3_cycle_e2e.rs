//! MEM-73 — the composed L0→L3 cycle, in a **real process**.
//!
//! The wave shipped each layer with its own proof: MCP-44 (capture + enqueue),
//! MCP-43 (the scheduler drains), MEM-71 (the pass consolidates), MEM-72 (a
//! runner-less pipeline reads as broken). Nobody had run the whole cycle in one
//! process from the artifact we ship. This test does exactly that:
//!
//! `vantadb-server --mcp` (writer) → `thread_send` → **scheduler tick** →
//! L1 extraction → L2 scenes → L3 persona → `memory_recall` returns the turn,
//! with no `memory_put` anywhere.
//!
//! # Why an ignored test
//!
//! It spawns a child process and a fake LLM endpoint; that is exactly the class
//! of test that poisons a default gate when it flakes (FIND-727/739). It is
//! `#[ignore]` and runs from an explicit lane:
//! `pwsh dev-tools/e2e-memory-cycle.ps1` (or `--run-ignored` by hand).
//!
//! # What the LLM is (and is not)
//!
//! There is no local model and no API key on this machine, so the *model* is a
//! scripted HTTP endpoint. The **runner is real**: the shipped binary builds
//! `ConcreteRunner::Ollama` from `VANTADB_INGEST_*`, and the real
//! `StandaloneLlmRunner` speaks HTTP to the fake. What is proven is the
//! composition (capture → queue → pass → extraction → scenes → persona →
//! recall); what is NOT proven is model quality. Declared, not blurred.
//!
//! # FIND-741 (stale binary)
//!
//! Before running, the binary's `--version` must name the current `HEAD`. A
//! stale artifact answers with the old contract shape; here it would also fail
//! the `ingest` assertions below, but the SHA check fails first and says why.
//! A `dirty` worktree only warns: `VANTADB_BUILD_COMMIT` proves what the
//! artifact came from, and the contract-shape assertions prove it is recent.
//!
//! # FIND-745 (out of contract)
//!
//! Durability across restarts (backfill of L0 into the queue) is deliberately
//! NOT covered: it is known debt, and this cycle runs in one process lifetime.
// ponytail: blanket allow — assertions with documented invariants; one binary.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use axum::{routing::post, Json, Router};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// One turn whose extraction yields 5 memories — the default
/// `auto_l2_threshold` (TEC-18), so L1 cascades to L2 → L3 in the same pass.
const TURN: &str = "I prefer the fjall storage backend and want deploys to stay local";

/// The first extracted memory: the recall assertion matches it EXACTLY, so a
/// raw-turn echo (or any other record) cannot satisfy the cycle.
const EXTRACTED_MEMORY: &str = "The user prefers the fjall storage backend";

/// Five distinct memories, all above L1's quality gate.
const EXTRACTION_JSON: &str = r#"[{"scene_name":"Storage","message_ids":[],"memories":[
  {"content":"The user prefers the fjall storage backend","type":"persona","priority":80,"source_message_ids":[]},
  {"content":"The user wants deploys to stay local","type":"instruction","priority":75,"source_message_ids":[]},
  {"content":"The user keeps VantaDB memory inside the editor session","type":"episodic","priority":60,"source_message_ids":[]},
  {"content":"The user prefers Rust for the storage engine","type":"persona","priority":70,"source_message_ids":[]},
  {"content":"The user avoids cloud dependencies for personal data","type":"persona","priority":80,"source_message_ids":[]}
]}]"#;

/// L2 contract: `[{scene_name, summary, content, merge_sources[], source_episode_ids[]}]`.
const SCENE_JSON: &str = r#"[{"scene_name":"Storage","summary":"storage prefs","content":"The user prefers fjall and local deploys.","merge_sources":[],"source_episode_ids":[]}]"#;

// ── scripted model ────────────────────────────────────────────────────────

/// The HTTP body carries no task id, so the prompt is the discriminator: each
/// pipeline stage has a distinct system prompt (checked against
/// `vanta-memory/src/core/prompts/*`).
fn respond(system: &str, user: &str) -> String {
    if system.contains("Persona Architect") {
        return json!({ "persona": "# User persona\n\n- Prefers the fjall storage backend.\n- Wants deploys to stay local.\n" })
            .to_string();
    }
    if system.contains("long-term memory deduplication") {
        // One `store` decision per judged record (ids are derived from the
        // prompt; `batch_dedup` only calls the LLM when candidates exist).
        let decisions: Vec<Value> = user
            .split("\"record_id\": \"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .map(|id| json!({ "record_id": id, "action": "store" }))
            .collect();
        return serde_json::to_string(&decisions).expect("decision json");
    }
    if system.contains("L2 scene organizer") {
        return SCENE_JSON.to_string();
    }
    // Fail loud on an unknown stage: a fourth pipeline layer must not be
    // silently answered with an L1 payload.
    if system.contains("scene segmentation and memory extraction") {
        return EXTRACTION_JSON.to_string();
    }
    panic!("unexpected system prompt (pipeline stage not modelled): {system}");
}

async fn chat(Json(body): Json<Value>) -> Json<Value> {
    let messages = body["messages"].as_array().cloned().unwrap_or_default();
    let text_of = |role: &str| -> String {
        messages
            .iter()
            .filter(|m| m["role"] == role)
            .filter_map(|m| m["content"].as_str())
            .next_back()
            .unwrap_or_default()
            .to_string()
    };
    let content = respond(&text_of("system"), &text_of("user"));
    Json(json!({ "choices": [ { "message": { "content": content } } ] }))
}

/// Serve the scripted model on 127.0.0.1:<ephemeral> from its own thread.
fn start_scripted_llm() -> String {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("scripted llm runtime");
        rt.block_on(async move {
            // The Ollama/OpenAI runner appends `/chat/completions` to base_url.
            let app = Router::new().route("/chat/completions", post(chat));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind scripted llm");
            let addr = listener.local_addr().expect("scripted llm addr");
            tx.send(format!("http://127.0.0.1:{}", addr.port()))
                .expect("share scripted llm addr");
            axum::serve(listener, app)
                .await
                .expect("serve scripted llm");
        });
    });
    rx.recv().expect("scripted llm ready")
}

// ── the shipped binary ────────────────────────────────────────────────────

/// `<target>/debug/vantadb-server[.exe]`, derived from this test binary's own
/// path so a custom `CARGO_TARGET_DIR` keeps working.
fn server_binary() -> PathBuf {
    if let Ok(explicit) = std::env::var("VANTADB_SERVER_BIN") {
        return PathBuf::from(explicit);
    }
    let exe = std::env::current_exe().expect("test exe path");
    let debug_dir = exe
        .parent()
        .and_then(Path::parent)
        .expect("target/debug from target/debug/deps");
    debug_dir.join(if cfg!(windows) {
        "vantadb-server.exe"
    } else {
        "vantadb-server"
    })
}

/// FIND-741 gate: the artifact must declare the commit under test.
fn assert_binary_is_head(bin: &Path) {
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .expect("run --version");
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let head = Command::new("git")
        .args([
            "-C",
            env!("CARGO_MANIFEST_DIR"),
            "rev-parse",
            "--short",
            "HEAD",
        ])
        .output()
        .expect("git rev-parse");
    let head = String::from_utf8_lossy(&head.stdout).trim().to_string();
    let commit = line
        .split("commit ")
        .nth(1)
        .and_then(|rest| rest.split(',').next())
        .unwrap_or("")
        .to_string();
    assert_eq!(
        commit, head,
        "stale binary: `{}` says commit {commit}, HEAD is {head} — rebuild \
         (`cargo build -p vantadb-server --bin vantadb-server`) before trusting this run",
        line
    );
    if !line.contains("worktree clean") {
        // Uncommitted tracked changes at build time: the SHA still pins the
        // commit, and the `ingest` assertions below cannot pass on a binary
        // predating MCP-43/44 — recorded, not fatal.
        eprintln!("MEM-73 NOTE: binary built from a dirty worktree ({line})");
    }
}

/// A stdio MCP client: newline-delimited JSON-RPC (server contract).
struct Server {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
    /// Every stderr line the server wrote, in order (FIND-747). Echoed to the
    /// operator as it arrives, so the log is not lost by capturing it.
    stderr: Arc<Mutex<Vec<String>>>,
    next_id: u64,
}

impl Server {
    fn spawn(bin: &Path, storage: &Path, llm_base: &str) -> Self {
        let mut child = Command::new(bin)
            .arg("--mcp")
            .env("VANTADB_STORAGE_PATH", storage)
            .env("VANTADB_MCP_PROFILE", "brain")
            .env("VANTADB_INGEST_PROVIDER", "ollama")
            .env("VANTADB_INGEST_BASE_URL", llm_base)
            .env("VANTADB_INGEST_MODEL", "scripted")
            // Default is 5000 ms. 1500 ms leaves room to OBSERVE the enqueue
            // before the first tick after the turn (the loop's first tick is
            // immediate, so a shorter interval could drain it before we look)
            // while still finishing the whole cycle well inside the 5 s bound.
            .env("VANTADB_SCHEDULER_INTERVAL_MS", "1500")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // FIND-747: captured (not inherited) so the cycle can assert that
            // the healthy path does not train the reader to ignore stderr. The
            // reader thread echoes every line, so `--no-capture` output is what
            // `inherit` used to be.
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn vantadb-server --mcp");
        let stdin = child.stdin.take().expect("child stdin");
        let stdout = child.stdout.take().expect("child stdout");
        let stderr_pipe = child.stderr.take().expect("child stderr");
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&stderr);
        std::thread::spawn(move || {
            for line in BufReader::new(stderr_pipe).lines() {
                let Ok(line) = line else { break };
                eprintln!("{line}");
                sink.lock().expect("stderr buffer").push(line);
            }
        });
        Self {
            child,
            stdin,
            rx,
            stderr,
            next_id: 1,
        }
    }

    /// Snapshot of the server's stderr so far (FIND-747).
    fn stderr_lines(&self) -> Vec<String> {
        self.stderr.lock().expect("stderr buffer").clone()
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        writeln!(self.stdin, "{msg}").expect("write request");
        self.stdin.flush().expect("flush request");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(left) {
                Ok(line) => {
                    let Ok(value) = serde_json::from_str::<Value>(&line) else {
                        continue; // not JSON-RPC (never expected on stdout)
                    };
                    if value["id"] == json!(id) {
                        return value;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    panic!("no response to {method} (id {id}) in 30s")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("server exited while waiting for {method} (id {id})")
                }
            }
        }
    }

    /// `tools/call` → parsed payload. Error content comes back as
    /// `{"_error": "<text>"}` so polls can express "not there yet".
    fn call(&mut self, tool: &str, args: Value) -> Value {
        let resp = self.request("tools/call", json!({ "name": tool, "arguments": args }));
        let result = &resp["result"];
        let text = result["content"][0]["text"].as_str().unwrap_or_default();
        if result["isError"].as_bool().unwrap_or(false) {
            return json!({ "_error": text });
        }
        serde_json::from_str(text).unwrap_or_else(|_| json!({ "_text": text }))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── the contract ──────────────────────────────────────────────────────────

/// The whole wave, composed: capture → queue → scheduled pass → L1 → L2 → L3 →
/// recall — observed from outside the process through the public tool surface.
#[test]
#[ignore = "spawns the shipped binary + a scripted LLM; run via dev-tools/e2e-memory-cycle.ps1"]
fn e2e_l0_to_l3_cycle_in_the_shipped_binary() {
    let bin = server_binary();
    assert!(
        bin.exists(),
        "missing {} — build it first: cargo build -p vantadb-server --bin vantadb-server",
        bin.display()
    );
    assert_binary_is_head(&bin);

    let llm_base = start_scripted_llm();
    let dir = tempfile::tempdir().expect("tempdir");
    let mut server = Server::spawn(&bin, dir.path(), &llm_base);

    let init = server.request(
        "initialize",
        json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "mem-73-e2e", "version": "1" }
        }),
    );
    assert!(init.get("error").is_none(), "initialize failed: {init}");

    // The writer hosts the loop and knows which runner it has (MCP-43/72).
    let caps = server.call("capabilities", json!({}));
    assert_eq!(caps["ingest"]["runner"], json!("ollama"), "caps: {caps}");
    assert_eq!(
        caps["ingest"]["scheduler_running"],
        json!(true),
        "the --mcp writer must host the loop: {caps}"
    );
    assert_eq!(caps["ingest"]["degraded"], json!(false), "caps: {caps}");

    let thread_id = server.call("thread_create", json!({ "title": "mem-73" }))["thread_id"]
        .as_str()
        .expect("thread id")
        .to_string();

    // ── L0 capture + one L1 enqueue, from one turn ──
    let sent = server.call(
        "thread_send",
        json!({ "thread_id": thread_id, "role": "user", "content": TURN }),
    );
    assert_eq!(
        sent["memory"],
        json!({ "captured": true, "queued": true }),
        "thread_send must feed the pipeline: {sent}"
    );
    let sent_at = Instant::now();

    // ADR-0065: `capabilities.ingest.queue` is a live read taken when the
    // answer is built, so the enqueued turn is visible here right away. The
    // proof below still uses the EFFECT (the drained queue + the recalled
    // turn) rather than a single sample, because the loop may drain between
    // two calls.

    // ── the scheduled pass drains it, L1 → L2 → L3 (no manual run_pass) ──
    //
    // The contract is the EFFECT, not a stopwatch: a wall-clock assert here
    // would be FIND-739 reborn (a bound that fails a run whose work already
    // succeeded). The budget only bounds how long a broken run takes to fail;
    // the observed elapsed is logged for the record.
    let mut caps = Value::Null;
    let mut recall_hit: Option<Value> = None;
    let mut scenes = 0usize;
    let mut persona = false;
    let mut pass_ran = false;
    while sent_at.elapsed() < Duration::from_secs(20) {
        caps = server.call("capabilities", json!({}));
        pass_ran = caps["ingest"]["last_pass"]["skipped"] == json!(false);

        if recall_hit.is_none() {
            let recalled = server.call(
                "memory_recall",
                json!({ "query": "fjall storage backend", "scope": "agent" }),
            );
            recall_hit = recalled["recalled"].as_array().and_then(|hits| {
                hits.iter()
                    .find(|h| {
                        // The extracted memory — NOT the raw turn ("I prefer
                        // ..."). Recall decorates content with an activity
                        // suffix, so equality is impossible by design;
                        // `starts_with` still separates extracted from raw.
                        h["content"]
                            .as_str()
                            .is_some_and(|c| c.starts_with(EXTRACTED_MEMORY))
                    })
                    .cloned()
            });
        }
        if scenes == 0 {
            scenes = server.call("scene_list", json!({ "session_key": thread_id }))["scenes"]
                .as_array()
                .map(Vec::len)
                .unwrap_or(0);
        }
        if !persona {
            let record = server.call(
                "memory_get",
                json!({ "namespace": format!("persona/{thread_id}"), "key": "persona.md" }),
            );
            persona = record["payload"].as_str().is_some_and(|p| !p.is_empty());
        }
        if recall_hit.is_some() && scenes > 0 && persona && pass_ran {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let elapsed = sent_at.elapsed();

    // L1 → recall: the turn became memory WITHOUT `memory_put`.
    let hit = recall_hit.unwrap_or_else(|| {
        let recalled = server.call(
            "memory_recall",
            json!({ "query": "fjall storage backend", "scope": "agent" }),
        );
        panic!("no exact recall within the budget — recalled: {recalled} (caps: {caps})")
    });
    assert!(
        hit["source_namespace"]
            .as_str()
            .unwrap_or_default()
            .starts_with("l1/"),
        "recall must come from the extracted L1 layer, got: {hit}"
    );
    // L2: the 5 extracted memories became a scene block.
    assert!(
        scenes > 0,
        "no L2 scene was written (caps: {caps}); queue: {}",
        caps["ingest"]["queue"]
    );
    // L3: the persona exists and carries the scripted body.
    let record = server.call(
        "memory_get",
        json!({ "namespace": format!("persona/{thread_id}"), "key": "persona.md" }),
    );
    assert!(
        record["payload"]
            .as_str()
            .is_some_and(|p| p.contains("fjall")),
        "no L3 persona record: {record}"
    );
    // The pass really ran with the runner: not skipped, queue drained, no
    // degraded warning (MEM-72's `is_broken()` false).
    assert!(pass_ran, "last_pass.skipped must be false: {caps}");
    assert_eq!(
        caps["ingest"]["queue"]["depth"],
        json!(0),
        "the queue must be drained: {caps}"
    );
    assert_eq!(
        caps["ingest"]["health"],
        json!("idle"),
        "a drained pipeline with a live loop and a real runner is idle: {caps}"
    );
    assert!(
        caps["ingest"]["notice"].is_null(),
        "a healthy pipeline carries no notice: {caps}"
    );

    // FIND-747: the healthy cycle must not emit errors it will discard. An
    // `ERROR add_edge … Node not found` per pass trains every client reading
    // stderr to ignore the channel, and hides a real lineage failure behind
    // the same text. The pass above is the real one (no `run_pass`), so this
    // is measured on the shipped path.
    let stderr = server.stderr_lines();
    let noise: Vec<&String> = stderr
        .iter()
        .filter(|l| l.contains("Node not found"))
        .collect();
    assert!(
        noise.is_empty(),
        "the healthy cycle wrote {} lineage error line(s) to stderr: {:?}\nfull stderr:\n{}",
        noise.len(),
        noise,
        stderr.join("\n")
    );

    eprintln!(
        "MEM-73 e2e: L0→L3 in {elapsed:?} (pass ran, queue drained, scene={scenes}, persona=true, stderr lines={})",
        stderr.len()
    );
}
