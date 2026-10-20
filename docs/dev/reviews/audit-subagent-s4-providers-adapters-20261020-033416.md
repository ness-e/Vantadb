---
title: "S4 — Deep Audit: Providers + Adapters"
kind: review
status: active
description: "Informe crudo del subagente S4: providers (openai, ollama, litellm), vanta-proxy y los 9 adaptadores Python de `integrations/`. Base de los hallazgos `S4-*` del ledger."
tags: [audit, reviews, providers, adapters, proxy, subagent]
---

# Informe del subagente — Deep Audit: PROVIDERS + ADAPTERS
Repo: `C:\Users\Eros\VantaDB Proyect\VantaDB` · Auditor: vanta-db-audit · Date: 2026-10-20
Scope: `providers/{openai,ollama,litellm}/` (Rust/PyO3 + PyPI metadata), `vanta-proxy/` (Rust), `integrations/*` (9 Python packages).

Read-only audit. No repo file was created, edited or deleted. No git write command was run (`git log`/`git show` only).

**SKILLS_CARGADAS:** `security-and-hardening` (trust boundaries, secret handling, OWASP/LLM framing) · `source-driven-development` (every external fact — PyPI metadata, wheel contents, axum/openai/hatchling APIs — verified against upstream sources rather than model memory) · `doubt-driven-development` (degraded self-check: a subagent cannot spawn a fresh-context reviewer, so the P1/P2 load-bearing claims were re-verified by re-reading the cited lines) · `code-review-and-quality` (five-axis framing, severity labels) · `code-simplification` (fixes framed as minimal root-cause moves) · `systematic-debugging` (root-cause attribution of each doc drift via `git log`/`git show` bisect-style archaeology).
**SDP discovery:** `SKILLS-MANIFEST.md` grep for `security|audit|supply|adversarial|python|packag` → `security-and-hardening` (KEEP, :274), `unified-review` (:474), `python-packaging` (**empty dir**, :562 — not loadable, and out of scope for an audit-only task). No additional skill covers Rust/Python supply-chain auditing of this shape; 6 skills loaded, 8 allowed.

---

## 0. Executive summary

| Question asked | Answer |
|---|---|
| Do providers read env vars? | **No.** Zero `env::var`/`os.environ` in `providers/**`. Keys are constructor args only. Consistent with `.env.tokens.example` §2 (path refs there are stale). |
| Is there an API-key redaction path? | **No, in the providers.** `vanta-proxy` has a full redaction engine (`src/redact.rs`, incl. an `sk-` detector) that the providers never reach. Providers do no logging at all, and the OpenAI SDK does not put request headers in exception text — so the raw key does not currently leak, but there is zero defense-in-depth. |
| Are budget / ACL / audit ENFORCED? | **Providers: not implemented at all.** `vanta-proxy`: budget/allowlist/audit **are** implemented and enforced (see A-12…A-14 for two bypasses). |
| Is the SSE parser correct? | **Parser: correct** (handles `[DONE]`, CRLF, garbage, multi-line data). **But** the stream is buffered fully in memory with **no size cap** → OOM. |
| Timeouts / retries? | Proxy: single **total** timeout (covers the whole stream), exponential backoff **without jitter**, retries non-idempotent POSTs across upstreams. Providers: one optional timeout, no retry control. |
| SSRF / missing auth? | **No request-controlled SSRF** (wire paths are hardcoded literals). Auth covers every route except `/health` (doc says otherwise). |
| 9 integrations? | **Yes, 9** (+ `vantadb_shared`, not a distribution). All 9 at version **0.9.0** = repo version. **7 of 9** are in `release-adapters.yml`; the 2 twins are retired by documented decision. |

**Counts:** 36 findings — 0 P0 · 1 P1 · 15 P2 · 20 P3. Plus 15 positive observations.

---

## PART A — PROVIDERS

### A-01 | P2 | providers/openai — API key not validated at construction; failure surfaces late and confusing
- **Location:** `providers/openai/src/python.rs:86-93` (constructor), `:102` (kwarg), `:52-57` (error path)
- **Claim/Break:** `api_key: &str` is a required positional argument, but it is **never validated**. `client_kwargs.set_item("api_key", api_key)?` is executed unconditionally — even for `""`. The OpenAI SDK only raises when the key is `None` (`openai/_client.py:209-210`: `if api_key is None: api_key = os.environ.get("OPENAI_API_KEY")`), so `""` passes construction. There is no emptiness/prefix check anywhere in `providers/`.
- **Why it matters:** The provider is the trust boundary that holds a paid third-party credential. A misconfigured key is accepted silently at construction and only fails later as an opaque HTTP 401 from the API, not as a local "missing API key" error. The caller gets `openai.AuthenticationError: Error code: 401 - {...}` wrapped in `OpenAI embed API error: model=..., detail=...` — the offending model, not the missing credential, is highlighted.
- **Trigger:** `VantaDBOpenAI("/tmp/db", "")` → constructs OK (no exception). `store.embed(["x"])` → `RuntimeError: OpenAI embed API error: model=text-embedding-3-small, detail=Error code: 401 - {...Incorrect API key provided...}`.
- **Fix:** Reject empty/whitespace keys at the boundary, before `Embedded::open_with_config`:
  ```rust
  if api_key.trim().is_empty() {
      return Err(PyValueError::new_err("api_key must be a non-empty string"));
  }
  ```
  Apply the same to `providers/litellm/src/python.rs:96-120` (which currently defaults to `""` and silently omits the kwarg, see A-02) and add an explicit "no key configured" error for `providers/ollama/src/python.rs:71-103` when the base URL is not reachable/configured.
- **Verify:** `cargo test -p vantadb-openai` + a new test asserting `VantaDBOpenAI(path, "")` raises `ValueError`.

### A-02 | P3 | providers/litellm — undocumented env-var fallback for the API key
- **Location:** `providers/litellm/src/python.rs:46-48`, `:114`; `providers/litellm/README.md:53`
- **Claim/Break:** `api_key: Option<&str>` defaults to `None` → stored as `String::new()` (`:114`). In `litellm_embed_chunk` the kwarg is only set when the string is non-empty (`:46-48`), so with no key the provider **delegates credential resolution to LiteLLM**, which reads its own environment variables (`OPENAI_API_KEY`, `AZURE_API_KEY`, …). Neither the README constructor line (`providers/litellm/README.md:53`) nor `.env.tokens.example` mentions this second credential path.
- **Why it matters:** A deployment that passes no key will silently pick up an ambient credential from the process environment. That is the opposite of "keys are passed as constructor args, not env vars" (`.env.tokens.example:19`) and makes the effective trust boundary invisible to whoever reads the docs.
- **Trigger:** `VantaDBLiteLLM("/tmp/db")` in a shell where `OPENAI_API_KEY` is exported → the provider authenticates against OpenAI with the ambient key, with no log line and no README notice.
- **Fix:** Either document the fallback explicitly in `providers/litellm/README.md`, or make it explicit — e.g. accept `api_key=None` to mean "resolve from env" and log/raise otherwise.
- **Verify:** doc + a test asserting no request is issued when neither the kwarg nor the env var is present.

### A-03 | P2 | providers/* — no API-key redaction path exists
- **Location:** `providers/openai/src/python.rs:52-57`, `providers/litellm/src/python.rs:59-61`, `providers/litellm/src/python.rs:28`
- **Claim/Break:** Grep over `providers/**` for `redact|scrub|mask|sanitiz` returns **0 matches in `.rs` sources**. The provider that holds the key keeps it in a plain Rust `String` field (`providers/litellm/src/python.rs:28`: `api_key: String`) and re-sends it as a kwarg on **every chunk** of `embed_batch` (`providers/litellm/src/python.rs:161-163` → `:46-48`). The error constructors embed the raw third-party exception text verbatim: `format!("OpenAI embed API error: model={}, detail={}", store.model, e)` and `format!("liteLLM embed error: {}", e)`. By contrast `vanta-proxy/src/redact.rs` ships a complete detector set including an `sk-` token scanner (`redact.rs:410-419` prefixes, `redact.rs:426-451` `scan_tokens`) — and it is **not reachable from the providers**.
- **Why it matters:** *Current* exposure is limited and I verified it: the OpenAI SDK's `APIStatusError` stringifies to the upstream **response body** only (`openai/_exceptions.py:67-71` → `super().__init__(message)`); request headers (which carry `Authorization: Bearer <key>`) are **not** part of `str(e)`. The providers also do zero logging (no `tracing`/`println!` in any provider `.rs`). So the raw key does **not** currently leak. What is missing is the *floor*: any future change that logs `e.request.headers`, or a LiteLLM exception that embeds `litellm_params` (which carries `api_key`), turns this into a credential leak with no masking layer to catch it. A security-critical primitive already exists in the repo and is simply not wired.
- **Trigger:** (latent) a one-line change adding `tracing::error!("{e:?}")` in `providers/openai/src/python.rs:52` — `APIError.__init__` stores `self.request`, and an httpx `Request` repr includes its headers → the key lands in the application log.
- **Fix:** Add a `redact_secret` helper to `providers/shared_py.rs` (single place, already shared by all three crates) that scrubs the known key from any message before it becomes a `PyErr`, and apply it in the three error constructors. Long-term, move the providers behind `vanta-proxy`'s `[redact]` policy or expose the detector set as a shared crate.
- **Verify:** unit test asserting that an exception whose text contains the api_key is rewritten before being raised.

### A-04 | P2 | providers/* — no token/spend budget, allow-list or audit trail (explicit answer)
- **Location:** `providers/{openai,ollama,litellm}/src/python.rs` (whole file), `providers/shared_py.rs` (whole file)
- **Claim/Break:** Grep over `providers/**` for `budget|allow_list|allowlist|audit|rate_limit|retry|backoff|stream|sse` returns **0 matches in `.rs` sources** (only test assertions and `#[allow(dead_code)]` noise matched). The three providers are thin embedding wrappers: `embed` / `embed_batch` / `store` / `search` / `get` / `list` / `delete` / `list_namespaces`. There is no spend ceiling, no model allow-list, and no per-call audit record of who embedded what.
- **Why it matters:** The audit brief asks whether these controls are *enforced or merely logged*. In the providers the honest answer is **neither — they are absent**. `embed_batch` (`providers/openai/src/python.rs:145-162`) will happily issue N/100 sequential paid API calls with no ceiling; `providers/ollama/src/python.rs:131-148` likewise. If these providers are the intended entry point for agent workloads, the spend guardrails that `vanta-proxy` implements (`cost.rs`, `guardrails.rs`, `governance.rs`) are not reachable from them.
- **Trigger:** `store.embed_batch(["text"] * 100_000, batch_size=100)` → 1 000 paid embedding requests, no budget check, no audit row, no log line.
- **Fix:** Either (a) state the scope explicitly in the provider READMEs ("no budget/audit; route through `vanta-proxy` if you need them"), or (b) add a per-instance `max_calls_per_minute` / `max_tokens_per_day` counter in `providers/shared_py.rs` (it is the shared module already included by all three crates, so it is a one-place change).
- **Verify:** test asserting the counter trips and raises a typed error.

### A-05 | P3 | providers/* — single total timeout, no connect/read split, no retry control
- **Location:** `providers/openai/src/python.rs:103-105`, `providers/ollama/src/python.rs:89-91`, `providers/litellm/src/python.rs:51-53`
- **Claim/Break:** `timeout: Option<f64>` is forwarded to the SDK client constructor **only when `Some`**; there is no default, no separate connect/read timeout, and no `max_retries` knob. With `timeout=None` the OpenAI SDK applies its own defaults (600 s total, 2 retries with exponential backoff + jitter) — i.e. the provider's effective retry/timeout policy is entirely the SDK's and is not configurable from the VantaDB API. The `timeout` field is even marked `#[allow(dead_code)]` in two of the three crates (`providers/openai/src/python.rs:31-33`, `providers/ollama/src/python.rs:31-32`) with a `ponytail:` note.
- **Why it matters:** The task asks explicitly: "are timeouts applied to connect and read separately? does retry use backoff with jitter? is there a max-attempts cap?" For the providers the answer is: **one total timeout, no separation, no provider-level retry, no cap**. A 600 s total timeout on an embedding call means a hung upstream pins a worker thread for 10 minutes.
- **Trigger:** `VantaDBOpenAI(db, key)` with no `timeout` → `store.embed([...])` against a stalled endpoint blocks for the SDK's default 600 s.
- **Fix:** expose `connect_timeout` / `read_timeout` / `max_retries` on the provider constructors and map them to the SDK client kwargs; default `max_retries` explicitly so behaviour is pinned rather than inherited.
- **Verify:** test asserting the client receives the expected kwargs.

### A-06 | P3 | providers/*/README.md — `embed_batch` missing from all three public API tables
- **Location:** `providers/openai/README.md:43-51`, `providers/litellm/README.md:43-51`, `providers/ollama/README.md:43-51`
- **Claim/Break:** All three READMEs present a "## API" table of methods. None of them lists `embed_batch`, although it is implemented in all three crates (`providers/openai/src/python.rs:145`, `providers/litellm/src/python.rs:148`, `providers/ollama/src/python.rs:131`) and is declared in the shipped type stubs (`providers/openai/vantadb_openai.pyi:11`, and the equivalents).
- **Why it matters:** The README table is the de-facto API contract for these packages. `embed_batch` is the only cost-relevant method (it multiplies API calls), so its absence from the contract is exactly the omission that hides the missing budget control of A-04.
- **Trigger:** a user reading `providers/openai/README.md` cannot discover that `embed_batch` exists or that it issues one request per 100 texts.
- **Fix:** add one row per README: `` | `embed_batch` | `embed_batch(texts, batch_size=100) -> list[list[float]]` | Chunked variant of `embed`; one request per chunk, order preserved. | ``
- **Verify:** doc-diff check, or a test that parses the README table and compares to the `.pyi`.

### A-07 | P3 | providers/ollama/README.md — documented response shapes are stale
- **Location:** `providers/ollama/README.md:47` and `:49` vs `providers/ollama/src/python.rs:200-205`, `:320-321`, `providers/shared_py.rs:124-141`
- **Claim/Break:**
  - `:47` — "`search(...)` … Returns `{id, text, score}` per hit." The code returns the **full record** plus `score`: `record_to_pydict` emits `namespace, key, text, metadata, created_at_ms, updated_at_ms, version, node_id[, vector][, expires_at_ms]` (`providers/shared_py.rs:124-141`), then `bound.set_item("score", hit.score)` (`providers/ollama/src/python.rs:201-203`). `shared_py.rs:14` even records this as a deliberate unification ("was ollama-minimal {id, text, score}").
  - `:49` — "`list(namespace, limit=100, cursor=None) -> dict` | Paginated listing: `{"records": [...], "cursor": str}` when a next page exists." The code returns `{"records": [...], "next_cursor": <int|None>}` (`providers/ollama/src/python.rs:320-321`), where `next_cursor` is `Option<usize>` — **wrong key name and wrong type**. The sibling `providers/openai/README.md:49` documents it correctly (`"next_cursor": int | None`), so the three READMEs disagree with each other.
- **Why it matters:** A caller coded against the Ollama README writes `resp["cursor"]` → `KeyError`, and `resp["records"][0]["id"]` → `KeyError` (the key is `key`, not `id`). The README also silently mis-describes the payload the caller receives.
- **Trigger:** `store.list("ollama_store")["cursor"]` → `KeyError: 'cursor'`; `store.search("ollama_store", vec)[0]["id"]` → `KeyError: 'id'`.
- **Fix:** copy the openai README wording for both rows.
- **Verify:** a doc test asserting the documented keys exist in the returned dict.

### A-08 | P3 | providers/* — auto-generated record key is a nanosecond timestamp; collision silently upserts
- **Location:** `providers/openai/src/python.rs:231-237`, `providers/litellm/src/python.rs:289-295`, `providers/ollama/src/python.rs:220-226`
- **Claim/Break:** When `key=None`, the key is `format!("openai_{ts}")` where `ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()`. The `store()` path is an **upsert** (`MemoryInput::new(&namespace, &key, text)` → `engine.put`), so two calls resolving the same nanosecond silently overwrite each other instead of creating two records.
- **Why it matters:** Silent data loss on the write path, with no error and no return-value difference (both return `"namespace:key"`, and the second returns the *same* id as the first). The probability is low (Rust's Windows clock is ~100 ns via `GetSystemTimePreciseAsFileTime`), but the failure mode is invisible, which is worse than a loud collision.
- **Trigger:** two `store()` calls landing in the same clock tick (multi-threaded caller, coarse clock, or a future platform with ms-resolution `SystemTime`) → the first record is replaced by the second.
- **Fix:** add a process-local monotonic counter or a `uuid` suffix to the auto-generated key (`format!("openai_{ts}_{n}")`), or make `store()` fail loudly on an existing key when `key` was auto-generated.
- **Verify:** test storing N records in a tight loop and asserting N distinct keys.

### A-09 | P2 | providers/supply-chain — provider lockfiles are excluded from every advisory/license scan
- **Location:** `.github/workflows/ci-rust.yml:551` (`cargo audit`), `:576` (`osv-scanner scan source --lockfile ./Cargo.lock …`), `:677` (`cargo deny check`); `osv-scanner.toml:5`; `providers/{openai,ollama,litellm}/Cargo.lock` (all three exist)
- **Claim/Break:** The three providers are **not workspace members** (`ci-rust.yml:489-490`: "providers/* are intentionally NOT workspace members … check them individually via --manifest-path") and each carries its **own** `Cargo.lock` that pins the entire `vantadb` core graph plus `pyo3`. The advisory gate runs only against the root `./Cargo.lock`; `osv-scanner.toml:5` states it plainly: "Config applies to the colocated ./Cargo.lock (does not propagate to feature-freez/desktop/src-tauri, fuzz/, **providers/\***, vantadb-node/ - separate lockfiles)." No job scans the provider lockfiles.
- **Why it matters:** Three published wheels (`vantadb-openai`, `vantadb-ollama`, `vantadb-litellm`) ship a full dependency graph that no advisory scanner ever inspects. Any RUSTSEC affecting `pyo3`, `tokio`, `reqwest`, `rustls`, `regex`, `serde` in those lockfiles is invisible to CI. The same is true of `cargo deny` license compliance for those crates.
- **Trigger:** a RUSTSEC is published for a crate in `providers/openai/Cargo.lock` → `cargo audit` at the root stays green, and the provider wheel still ships the vulnerable version.
- **Fix:** add a CI step that runs `cargo audit --file providers/<p>/Cargo.lock` (or `osv-scanner scan source --lockfile providers/<p>/Cargo.lock --config ./osv-scanner.toml`) for each of the three lockfiles; `cargo deny` supports `--manifest-path` for the license side.
- **Verify:** the step must fail on a seeded advisory in a provider lockfile.

### A-10 | P3 | `.env.tokens.example` — source paths cited for the providers do not exist
- **Location:** `.env.tokens.example:20` and `:23`
- **Claim/Break:** The template says `Usado en: vantadb-openai/src/python.rs` and `Usado en: vantadb-litellm/src/python.rs`. The actual paths are `providers/openai/src/python.rs` and `providers/litellm/src/python.rs` — there is no `vantadb-openai/` or `vantadb-litellm/` directory at the repo root (the crate directories are `providers/openai`, `providers/litellm`).
- **Why it matters:** The template is the operator-facing map of "where is each secret used". A wrong path sends an operator looking for a file that has never existed, and hides that the Ollama provider is *not* listed at all (correctly, since it needs no key — but the omission is implicit rather than stated).
- **Trigger:** an operator greps for `vantadb-openai/src/python.rs` → no hits, and concludes the provider does not exist.
- **Fix:** correct to `providers/openai/src/python.rs` / `providers/litellm/src/python.rs`; add a line stating that `providers/ollama` requires no key.
- **Verify:** link/path checker over the template.

---

## PART A — `vanta-proxy/` (Rust)

### A-11 | P1 | proxy/streaming — the SSE interceptor buffers the entire upstream stream in memory with no size cap
- **Location:** `vanta-proxy/src/sse_intercept.rs:60-69` (`drain`), `:48-53` (`StreamCapture`), called from `vanta-proxy/src/server.rs:785`
- **Claim/Break:**
  ```rust
  pub(crate) async fn drain(body: Body) -> Result<StreamCapture, ProxyError> {
      let mut out = StreamCapture::default();
      let mut stream = body.into_data_stream();
      while let Some(chunk) = stream.next().await {
          let chunk = chunk.map_err(|e| ProxyError::Forward(format!("stream read: {e}")))?;
          out.push(chunk);
      }
      Ok(out)
  }
  ```
  `StreamCapture` keeps **both** the concatenated bytes (`full: Vec<u8>`) and every individual chunk (`chunks: Vec<Bytes>`) — roughly 2× the stream size in RAM — with **no byte cap, no chunk-count cap and no wall-clock cap**. The gate that decides to take this path is `memory_tools::announces(&body)` (`vanta-proxy/src/server.rs:747-753`), which inspects the **client-supplied request body**. The upstream response is bounded only by the client's total forward timeout (600 s default, `vanta-proxy/src/config.rs:10`).
- **Why it matters:** This is the one place where the proxy trades streaming for buffering, and it is reachable by any authenticated client that puts the Vanta memory-tool names in the request body. The client fully controls the switch, and the amount of memory consumed is controlled by the **upstream response size**, which the client influences by asking for a long generation. The tool loop can repeat the cycle up to `MAX_MEMORY_TOOL_ITERATIONS = 3` (`vanta-proxy/src/server.rs:42`), i.e. up to 4 upstream forwards, each of which is drained.
- **Trigger:** `POST /v1/responses` with `{"model":"…","tools":[{"name":"vanta_memory_search",…}],"input":"…"}` and a body long enough to make the upstream emit a multi-hundred-MB SSE response → the proxy holds the whole thing in RAM before answering the client. N concurrent such requests multiply it.
- **Fix:** cap the drain. The codebase already has the pattern — `cache.rs` uses a `MAX_CACHEABLE_BODY_BYTES` cap (documented at `docs/api/PROXY.md:291`). Add a `MAX_SSE_CAPTURE_BYTES` (e.g. 8–16 MiB); on overflow, stop capturing and **replay the partial stream verbatim** (the existing `sse_intercept::replay(parts, captured.chunks)` path already supports handing back what was collected) instead of failing the request.
- **Verify:** a test that feeds a synthetic SSE body larger than the cap and asserts the proxy (a) does not buffer it all and (b) still returns a well-formed response.

### A-12 | P2 | proxy/routing — the self-forwarding-loop guard does not cover the `[[upstreams]]` list
- **Location:** `vanta-proxy/src/server.rs:131-136`, `vanta-proxy/src/config.rs:75-81` (`upstreams_resolved`), `vanta-proxy/src/config.rs:364-370` (`points_at_self`)
- **Claim/Break:** The startup guard checks **only** `config.upstream`:
  ```rust
  if config.upstream.points_at_self(config.server.port) {
      return Err(crate::error::ProxyError::Config(format!(
          "upstream {} points at this proxy itself (listen port {})", …)));
  }
  ```
  But `upstreams_resolved()` (`vanta-proxy/src/config.rs:75-81`) returns `self.upstreams` **whenever that list is non-empty**, and that resolved list is what is actually forwarded (`vanta-proxy/src/server.rs:219` → `:710` → `vanta-proxy/src/forward.rs:249`). So when an operator configures `[[upstreams]]`, the guarded `config.upstream` is not used for forwarding at all, and the guard inspects the wrong value. `validate_startup` does it correctly — it checks `self.upstreams_resolved()` for empty URLs (`vanta-proxy/src/config.rs:284-288`) — which makes the omission in `points_at_self` look like an oversight rather than a decision. No test covers it: grep for `points_at_self|self_loop|self-loop` across `vanta-proxy/tests/` returns **no matches**.
- **Why it matters:** The guard exists precisely to prevent "recurses until the forward timeout" (PRX-08 S2). With `[[upstreams]]` configured, that protection is silently absent: the proxy forwards to itself, the inner proxy forwards to itself, and the recursion runs until the 600 s timeout — a self-inflicted resource exhaustion that the codebase explicitly tries to prevent. `docs/api/PROXY.md:279-281` tells the operator the guard applies ("an explicit URL pointing back at the proxy's own port is refused by `points_at_self`"), without qualifying it to `[upstream].url` only.
- **Trigger:** `config.toml` with
  ```toml
  [upstream]
  url = "https://api.anthropic.com"
  [[upstreams]]
  url = "http://127.0.0.1:8096"
  ```
  → the proxy starts successfully and every request recurses into itself.
- **Fix:** iterate the resolved list:
  ```rust
  if config.upstreams_resolved().iter().any(|u| u.points_at_self(config.server.port)) { … }
  ```
  and add a regression test with a self-pointing `[[upstreams]]` entry.
- **Verify:** new test asserting `ProxyConfig::from_engine` (or `validate_startup`) rejects a config whose `[[upstreams]][0].url` is `http://127.0.0.1:8096`.

### A-13 | P2 | proxy/rate-limit — the rate-limit bucket key is 100 % client-controlled, so the limit is trivially bypassed
- **Location:** `vanta-proxy/src/server.rs:397` (`self.limiter.check(space_id, model)`), `vanta-proxy/src/rate_limit.rs:113` + `:133` (`bucket_key(space_id, model)`), `vanta-proxy/src/handlers/openai.rs:13` (`Path((agent, space_id))`), `vanta-proxy/src/server.rs:269` (`model_from_body`)
- **Claim/Break:** The limiter is a sliding window keyed by `(space_id, model)` (`vanta-proxy/src/rate_limit.rs:133`). **Both** components are attacker-supplied:
  - `space_id` is the raw URL path segment `/{agent}/{space_id}/v1/chat/completions`, extracted with no validation (`vanta-proxy/src/handlers/openai.rs:13`: `Path((agent, space_id)): Path<(String, String)>`) and passed straight into `check()` (`vanta-proxy/src/server.rs:397`).
  - `model` comes from the JSON body (`model_from_body`, `vanta-proxy/src/server.rs:269`).
  Neither is bound to the authenticated identity (`x-vanta-user-key` → `user_id`). The `agent` segment is likewise unvalidated and unused except for a debug log.
- **Why it matters:** `server.rate_limit_per_minute` (default 60, `vanta-proxy/src/config.rs:263`) is the proxy's only volumetric control on spend and on upstream load. Rotating the path segment or the `model` field creates a fresh bucket and resets the counter, so the configured limit can be multiplied by an arbitrary factor. It is also not per-user, so N users × M rotated segments multiply it again. The bucket map is capped at `MAX_BUCKETS = 10_000` (`vanta-proxy/src/rate_limit.rs:22`) with oldest-eviction, so the map itself does not grow unbounded — the *limit* is what is lost.
- **Trigger:** with a valid user key, send 61 requests to `/a/space-1/v1/chat/completions`, then 61 more to `/a/space-2/v1/chat/completions` — both succeed; the same is achieved by varying `"model"` in the body.
- **Fix:** key the limiter on the **authenticated identity** (the `user_id` already resolved at `vanta-proxy/src/server.rs:338`) in addition to (or instead of) the client-supplied `space_id`, and validate `space_id` against a charset/length bound. Note the ordering constraint: auth must run before the limiter (it already does — `server.rs:338` before `:397`), so the identity is available.
- **Verify:** test that N+1 requests from the **same user key** with **different** `space_id` values are limited.

### A-14 | P2 | proxy/rate-limit — three upstream 429s disable rate limiting entirely (fail-open)
- **Location:** `vanta-proxy/src/rate_limit.rs:194-206` (`UpstreamHealth::observe`), `:113-123` (`RateLimiter::check` fail-open), `vanta-proxy/src/server.rs:684-696` (`note_upstream`), `:714` / `:718`
- **Claim/Break:** `observe()` counts **429** as a failure:
  ```rust
  if status == 429 || (500..600).contains(&status) {
      self.successes.store(0, Ordering::Relaxed);
      let failures = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
      return (failures >= DEGRADED_ENTER_FAILURES).then_some(true);
  }
  ```
  After 3 such responses, `note_upstream` (`vanta-proxy/src/server.rs:686-689`) sets `self.limiter.set_degraded(true)`. While degraded, `check()` returns `RateDecision::Allowed` for **every** request, with only a `tracing::warn!` (`vanta-proxy/src/rate_limit.rs:114-123`). Recovery requires 5 consecutive successful upstream responses (`DEGRADED_EXIT_SUCCESSES`, `vanta-proxy/src/rate_limit.rs:169`).
- **Why it matters:** A 429 is the upstream saying *this client is over quota* — the exact condition where you most want the proxy to slow the client down. Instead, the proxy interprets the client's quota exhaustion as "the upstream is broken" and turns off its own volumetric protection, then keeps forwarding at full speed until the upstream starts answering 2xx again. The fail-open contract is documented (`rate_limit.rs:5-6`, "degraded → allow + warn log, never block the wire") but it is documented as an *upstream-health* fallback; the 429 path makes it client-triggerable.
- **Trigger:** exhaust the upstream's quota (or point the proxy at a quota-limited tenant) → 3 consecutive 429s → the limiter is off; the client keeps hammering the quota-exhausted upstream.
- **Fix:** exclude 429 from the *degrade* trigger (keep it in the failover trigger, `vanta-proxy/src/forward.rs:26-28`), or make the degraded mode only relax the limiter for the failing upstream rather than globally. At minimum, separate "the upstream is sick" (5xx) from "the client is over quota" (429).
- **Verify:** test asserting that 3 consecutive 429s do **not** flip the limiter to degraded, while 3 consecutive 503s do.

### A-15 | P2 | proxy/cache — the cache key carries no user identity, so responses are shared across authenticated users
- **Location:** `vanta-proxy/src/cache.rs:163-184` (`lookup`), `:167-171` (the `Key`), `vanta-proxy/src/server.rs:510-518` (call site)
- **Claim/Break:** The cache key is exactly `{protocol, path, body}`:
  ```rust
  let key = Key { protocol: protocol.to_string(), path: path.to_string(), body: body.to_vec() };
  ```
  It contains **no** `user_id`, **no** `x-vanta-session`, and **no** `Authorization` header. The lookup site (`vanta-proxy/src/server.rs:512`) runs after auth (`:338`), but the auth result is not part of the key. When `upstream.api_key` is empty — the shipped default (`vanta-proxy/config.toml:16`) and the code default (`vanta-proxy/src/config.rs:395`) — each client supplies its **own** `Authorization` header, which the forwarder passes through verbatim (`vanta-proxy/src/forward.rs:179-185` only overrides when `cfg.api_key` is non-empty). Also note the no-session path (`vanta-proxy/src/server.rs:435-439`) forwards without injecting the session-scoped memory block, so the body is byte-identical across users and therefore the cache key is too.
- **Why it matters:** With `[cache] enabled = true` and `ttl_secs > 0` (both required — `vanta-proxy/src/config.rs:168-176`), user B's request can be served **user A's upstream response**, and B's request never reaches the upstream at all (so B's own quota is not consumed either). The cached payload is whatever the upstream returned for A — which, for a proxy whose purpose is injecting per-user memory, may be A-specific content. The semantic-similarity path (`lookup_similar`, `cache.rs:197-…`) widens the surface further: it matches on a body *template* plus cosine similarity over the prompt, i.e. near-duplicate prompts from different users.
- **Trigger:** `[cache] enabled=true, ttl_secs=300`, `upstream.api_key=""`. User A (key A) POSTs `{"model":"gpt-4o","messages":[{"role":"user","content":"secret question"}]}` to `/v1/responses` **without** an `x-vanta-session` header. User B (key B) POSTs the identical body → receives A's cached response; the upstream is never called for B.
- **Fix:** add the resolved `user_id` (and, where present, the session key) to the cache `Key`, and/or key on a hash of the forwarded `Authorization` header when the upstream key is pass-through. Also consider making the no-session path non-cacheable, since that is the path where the body carries no per-user content.
- **Verify:** test that two different `x-vanta-user-key` values with the same body produce two upstream calls and two distinct responses.

### A-16 | P3 | proxy/auth — the doc comment claims constant-time key comparison, but the code uses a `HashMap`
- **Location:** `vanta-proxy/src/auth.rs:121-127` (comment) vs `:136` (implementation), `:149-158`
- **Claim/Break:** The doc comment on `resolve_user_key` states: *"First call scans the `user` entity collection (**comparing keys in constant time**, MEM-05 parity)"*. The implementation resolves keys through a `HashMap<String, UserIdentity>` — `self.index.lock()…get(user_key)` (`auth.rs:136`) and `snapshot.entry(candidate.clone()).or_insert_with(…)` (`auth.rs:149`). `HashMap` lookup is neither constant-time nor constant-time-comparing; the byte comparison is whatever `SipHash`/`ahash` does.
- **Why it matters:** A security-relevant property is asserted in prose that the code does not implement. A reviewer trusting the comment would not add a constant-time comparison. (Practical exploitability here is low: the attacker would need to recover a user key from timing noise over a local file-backed store, and the header is compared against a map, not a secret-equal primitive.)
- **Trigger:** N/A at runtime; the defect is the false invariant in the code comment.
- **Fix:** either implement constant-time comparison (e.g. `subtle::ConstantTimeEq` over the candidate set) or correct the comment to say "first-wins map lookup".
- **Verify:** update/extend the comment; no behaviour change needed.

### A-17 | P3 | proxy/docs — `docs/api/PROXY.md` claims every route requires auth; `/health` does not
- **Location:** `docs/api/PROXY.md:47` vs `vanta-proxy/src/server.rs:866` + `:901-903`
- **Claim/Break:** `docs/api/PROXY.md:47`: *"Every route - including `GET /snapshot` - requires a valid `x-vanta-user-key` header resolved against the local `user` entity collection"*. The `/health` handler takes no `State`, no `HeaderMap`, and performs no auth:
  ```rust
  async fn health() -> Json<serde_json::Value> { Json(serde_json::json!({ "status": "ok" })) }
  ```
- **Why it matters:** The impact is minimal (`/health` returns only `{"status":"ok"}`), but the sentence is a blanket security assertion that a reader will rely on. Anyone adding a field to the health payload will not think to gate it, because the doc says the route is already authenticated.
- **Trigger:** add `"upstream": config.upstream.url` to the health payload and it is served unauthenticated — the doc gave no reason to worry.
- **Fix:** change the sentence to "Every route except `GET /health` requires a valid `x-vanta-user-key` …".
- **Verify:** doc assertion + a test that `GET /health` returns 200 without the header (documented behaviour).

### A-18 | P3 | proxy/docs — the "environment variables" table is incomplete while claiming to be exhaustive
- **Location:** `docs/api/PROXY.md:319-328` vs `vanta-proxy/src/envelope.rs:125` → `src/crypto.rs:196`
- **Claim/Break:** The table is introduced with *"The only `std::env::var` reads in `vanta-proxy/src` (plus the tracing filter)"* and lists 4 variables (`VANTA_PROXY_CONFIG`, `VANTA_EMBED_BASE_URL`, `VANTA_EMBED_MODEL`, `RUST_LOG`). The proxy also reads **`VANTADB_ENCRYPTION_KEY`**: `Envelope::from_config` calls `Cipher::from_env()` (`vanta-proxy/src/envelope.rs:125`), which reads that variable (`src/crypto.rs:193-196`, documented at `src/crypto.rs:15`). It is mentioned elsewhere in the doc (`:103`, `:136`) but is absent from the inventory the doc presents as complete.
- **Why it matters:** An operator auditing the proxy's environment surface reads a table that says it is the complete list, and misses the variable that unlocks (or silently disarms) the AEAD envelope.
- **Trigger:** an operator exports `VANTADB_ENCRYPTION_KEY` expecting no effect, or rotates it believing nothing reads it, while `[envelope] enabled = true` silently degrades to `Disarmed` (warning at `vanta-proxy/src/server.rs:171-177`).
- **Fix:** add a row: `` | `VANTADB_ENCRYPTION_KEY` | master key for the `[envelope]` AEAD seal | unset → envelope disarmed (redacted-only) | `envelope.rs:125` → `crypto.rs:196` | ``
- **Verify:** doc check that every `std::env::var` / `from_env` in `vanta-proxy/src` appears in the table.

### A-19 | P3 | proxy/docs — `docs/api/PROXY.md` source line references are systematically stale
- **Location:** `docs/api/PROXY.md:33-40`, `:63-81`, `:271-300`
- **Claim/Break:** Every `Source` cell points at lines that no longer hold the referenced code. Examples:
  | Doc says | Actual |
  |---|---|
  | `/health` → `server.rs:790,825` | `server.rs:866`, handler `:901` |
  | `/snapshot` → `server.rs:791,871` | `server.rs:867`, handler `:947` |
  | `/sessions/advance` → `server.rs:792,834` | `server.rs:868`, handler `:910` |
  | `/{agent}/{space_id}/v1/chat/completions` → `server.rs:793-796` | `server.rs:869-872` |
  | `[mem_command]` → `config.rs:78-86` | `config.rs:139-143` |
  | `[cache]` → `config.rs:137-166` | `config.rs:166-194` |
  | `[cost]` → `config.rs:157-184` | `config.rs:219-244` |
  | `[auth]` → `config.rs:298-310` | `config.rs:326-340` |
  | `[writeback]` → `config.rs:88-103` | `config.rs:147-160` |
  | `upstream.url` → `config.rs:253,358-370` | `config.rs:253` is not the URL field; `points_at_self` is `config.rs:364-370` |
- **Why it matters:** The doc's value proposition is that it is the auditable map from behaviour to source. Every pointer in it is wrong, so a reviewer cannot use it to verify a claim — which is exactly how A-12 (guard scope) survived: `docs/api/PROXY.md:279-281` cites `config.rs:334-340`, a line range that no longer contains `points_at_self`.
- **Trigger:** following any `Source` link in `docs/api/PROXY.md` lands on unrelated code.
- **Fix:** regenerate the references (or add a CI job that greps the doc's `file.rs:NN` citations and fails when the line does not contain the named symbol).
- **Verify:** the CI checker described above.

### A-20 | P3 | proxy/timeout — the forward timeout is a single total budget that also covers the streamed response
- **Location:** `vanta-proxy/src/forward.rs:150-156`, `vanta-proxy/src/config.rs:10`, `:352-353`
- **Claim/Break:** The client is built with `Client::builder().timeout(Duration::from_secs(cfg.forward_timeout_secs))` — a **total request** timeout in `reqwest`, which covers connect + read + **the entire response body stream**. The code acknowledges this: *"ponytail: client-level timeout caps total stream duration at 600s — streams longer than the timeout are cut; per-phase timeouts if real traffic hits it."* (`forward.rs:153-154`). Default 600 s (`config.rs:10`).
- **Why it matters:** There is no separate connect timeout and no read/idle timeout, and the stream-continuation budget is shared with connection setup. A long-lived SSE session is cut at the total budget, and a slow-but-healthy upstream that takes 60 s to connect has only 540 s left for the whole generation. The `UpstreamTimeout` → 504 mapping (`vanta-proxy/src/error.rs:88`, `forward.rs:361-369`) is correct, but the granularity is not.
- **Trigger:** a 700 s agent session against a slow upstream → the stream is truncated at 600 s and the client sees a 504 (or a truncated body if the cutoff lands mid-stream).
- **Fix:** set `connect_timeout` separately from the total, and use `reqwest::Client::builder().read_timeout(...)` (or an idle-timeout wrapper) for the stream rather than a whole-request budget.
- **Verify:** test with a mock upstream that stalls after headers.

### A-21 | P3 | proxy/retry — exponential backoff without jitter; retries non-idempotent POSTs across upstreams
- **Location:** `vanta-proxy/src/forward.rs:30-37` (`backoff_delay`), `:228-290` (`forward_with_failover`), `:26-28` (`is_retryable_status`)
- **Claim/Break:**
  - *No jitter*: *"No jitter by design (decisión 4: determinism). `ponytail:` no jitter — add `+/−25%` if real traffic shows thundering herd."* (`forward.rs:32-33`). The schedule is exactly `min(100·2^n, 2000)` ms, unit-tested to the byte (`forward.rs:389-405`).
  - *Non-idempotent retry*: the forwarded method is `POST` for every wire route. On a 429/5xx the body is dropped and **re-POSTed to a different upstream** (`forward.rs:263-271`). A 5xx may mean the upstream already processed the request (and billed for it), so the retry can double-charge.
  - *Attempt cap*: the loop is bounded by `upstreams.len()` — there is no separate max-attempts knob.
  - *Error propagation*: the **last** attempt's outcome is returned verbatim (`forward.rs:272-277`), including a real 429/5xx, which is the correct behaviour (the client sees the upstream's own signal, not a synthetic error).
- **Why it matters:** The no-jitter backoff is a deliberate, documented choice and the error propagation is correct, so the substantive issue is (a) the herd risk and (b) the non-idempotent retry. On (b): with a single upstream configured (the default) there is no cross-upstream replay, so the blast radius is limited to the "last attempt" path; with `[[upstreams]]` configured the same request body is sent to two different providers.
- **Trigger:** two upstreams behind one proxy, the first returns 503 after partially processing → the identical body is re-sent to the second → possible double billing and a duplicated side effect upstream.
- **Fix:** add ±25 % jitter to `backoff_delay` (the `ponytail:` comment names the exact change); restrict cross-upstream replay to idempotent verbs or to statuses that prove the request was not processed (e.g. 429 / connection errors), and surface a config flag for operators who accept the double-send risk.
- **Verify:** unit test that `backoff_delay` values fall in the jittered band; integration test that a 503 on upstream A and 200 on upstream B is asserted per policy.

### Positive observations — Part A
- **A-P1 — Fail-closed auth on every route.** `vanta-proxy/src/auth.rs:100-109` resolves `x-vanta-user-key` against the local `user` entity collection and fails closed on missing/empty/unknown keys (`ok_or(ProxyError::Unauthorized)`); `vanta-proxy/src/server.rs:338-341` runs it **before** the rate limiter so unauthenticated traffic cannot burn another identity's quota.
- **A-P2 — The internal credential is stripped before forwarding.** `vanta-proxy/src/forward.rs:99-102` lists `x-vanta-user-key` in the hop-by-hop blocklist, so the D34 credential never reaches the upstream. Verified by reading the whole filter (`forward.rs:108-119`).
- **A-P3 — No request-controlled SSRF.** The upstream URL is `format!("{base}{wire_path}")` (`vanta-proxy/src/forward.rs:174-175`) and `wire_path` is always a **hardcoded literal** supplied by the handler (`"/v1/chat/completions"`, `"/v1/messages"`, `"/v1/responses"` — `handlers/openai.rs:21`, `handlers/responses.rs:26`, `:44`). The client-controlled `{agent}` / `{space_id}` path segments are **not** concatenated into the upstream URL, so path traversal into another upstream is impossible.
- **A-P4 — The proxy's redaction engine is well built.** `vanta-proxy/src/redact.rs` uses hand-rolled linear scans (no ReDoS), bounded `size_limit`/`dfa_size_limit` on operator regexes (`:156-165`), findings carry **labels only and never matched values** (`:73-79`, `:10-11`), the wire path fails open on non-UTF-8 and over the 2 MiB cap (`:183-188`), and the on-write path uses the same detectors (`capture.rs:88-101`). `sk-` is in `TOKEN_PREFIXES` (`redact.rs:410-419`).
- **A-P5 — Governance is enforced, not merely logged.** The injection ACL + audit are real: `governance.rs:97-127` emits one metadata-only event per injected source and one per denial; `inject.rs:233-244` enforces the `max_tokens` budget via `fit_sections` (`:256-…`); the guardrails deny with 403 (`server.rs:378-394`) and the budget gate denies with 429 (`server.rs:346-370`).
- **A-P6 — The proxy's dependency graph is audited.** As a workspace member (`vanta-proxy/Cargo.toml:3`: `version.workspace = true`) its graph lives in the root `Cargo.lock` and is therefore covered by `cargo audit`, `cargo deny check` and `osv-scanner` (`ci-rust.yml:551`, `:576`, `:677`). Its `[features = ["encryption"]` pull-in (`vanta-proxy/Cargo.toml:33`) is therefore also audited — in contrast to the three providers (A-09).
- **A-P7 — Provider error messages never use `Debug` formatting.** `providers/shared_py.rs:332-334` asserts (and the code honours) that `format!("{:?}"` never appears in the Python-boundary messages; `err_to_py` goes through `Display` (`shared_py.rs:69-100`) and attaches canonical `code` / `retriable` / `hint` metadata (`:56-63`), mirroring the SDK's MOD-20 hierarchy.

---

## PART B — ADAPTERS (`integrations/*`)

### B-01 | P2 | adapters/deps — the twins' `vantadb-py` floor allows a version with no `Client` API
- **Location:** `integrations/openai/pyproject.toml:26`, `integrations/ollama/pyproject.toml:26` vs `integrations/vantadb_shared/__init__.py:47`
- **Claim/Break:** Both twins declare `"vantadb-py>=0.5.0,<0.10.0"`. Their shared base class calls `vanta.Client(...)`:
  ```python
  self._db = vanta.Client(db_path, memory_limit_bytes=memory_limit_bytes, read_only=read_only)
  ```
  I verified against the published wheels: **`vantadb-py` 0.5.0 has no `Client`** — its `vantadb_py/__init__.py` does not mention `Client` and its `vantadb_py/vantadb_py.pyi` has no `class Client` (0.8.0 has both). This contradicts the repo's own statement in `integrations/README.md:32`: *"piso real: 0.5.0 no tiene la API `Client`"*, and the other seven adapters, which all pin `>=0.6.1`.
- **Why it matters:** With a constrained resolver (offline mirror, a lockfile pinning `vantadb-py==0.5.0`, or `--no-deps` with a pre-installed 0.5.0), the install succeeds and the **first constructor call** dies with `AttributeError: module 'vantadb_py' has no attribute 'Client'` — a runtime failure in a library whose whole purpose is the `Client` API, and one that the repo's own documentation says must not happen.
- **Trigger:** `pip install "vantadb-py==0.5.0"` then `pip install --no-deps ./integrations/openai` → `VantaDBOpenAI(api_key="sk-…")` → `AttributeError: module 'vantadb_py' has no attribute 'Client'`.
- **Fix:** raise the twins' floor to `>=0.6.1` (or `>=0.8.0`, the current PyPI release) to match the other seven and the documented invariant.
- **Verify:** `pip install --no-deps` with `vantadb-py==0.5.0` and assert the constructor raises a clear error; add a pin-floor assertion to `integrations/test_pins.py`.

### B-02 | P3 | adapters/docs — the documented `vantadb-py` ceiling is stale after the 0.9.0 sync
- **Location:** `integrations/README.md:29`, `:31-32`, `integrations/test_pins.py:5` vs the 9 `pyproject.toml` files
- **Claim/Break:** The docs still say `vantadb-py>=0.6.1,<0.9.0`:
  - `integrations/README.md:29` — "…`vantadb-py>=0.6.1,<0.9.0` exenta por traer techo propio"
  - `integrations/README.md:31-32` — "pin `vantadb-py` bumpeado a `>=0.6.1,<0.9.0` (decisión owner) … el techo permite el core live 0.8.0"
  - `integrations/test_pins.py:5` — "`vantadb-py>=0.6.1,<0.9.0` ya trae techo y queda exenta"

  The **code** says `<0.10.0` in all nine pyprojects. Git shows why: commit `3430c42c` ("chore(release): sync monorepo surface manifests to v0.9.0") bumped the adapter versions `0.8.0 → 0.9.0` **and** relaxed the ceiling `<0.9.0 → <0.10.0` in the same commit — but it did not touch `integrations/README.md` or `integrations/test_pins.py`.
- **Why it matters:** The ceiling is the thing that lets a future `vantadb-py` 0.9.0 be installed alongside the 0.9.0 adapters. The code now allows it; the documentation still forbids it. A reader following the README would conclude the pin is `<0.9.0`.
- **Trigger:** the next `vantadb-py` release is 0.9.0 → `pip install vantadb-langchain` resolves it fine (code says `<0.10.0`), but `integrations/README.md:31` tells the operator the ceiling forbids it.
- **Fix:** update the three doc occurrences to `<0.10.0`.
- **Verify:** grep for `0.9.0` in `integrations/README.md` and `integrations/test_pins.py`.

### B-03 | P3 | adapters/deps — `letta-client` is a declared dependency that is never imported
- **Location:** `integrations/letta/pyproject.toml:27` vs `integrations/letta/vantadb_letta/vectorstore.py` (whole file)
- **Claim/Break:** `dependencies = ["vantadb-py>=0.6.1,<0.10.0", "letta-client>=1.0.0,<2"]`. The adapter module imports **only** `vantadb_py` (`vectorstore.py:6`) and stdlib. Grepping the whole `integrations/letta/` tree for `letta_client|letta-client|from letta|import letta` returns **exactly one hit — the pyproject line itself**.
- **Why it matters:** `letta-client` is an unnecessary dependency: it pulls `httpx`, `pydantic`, `websockets` and friends into every install for no code path. It is also a misleading signal — the README (`integrations/letta/README.md:3`) calls this a "Letta storage adapter", implying a Letta client integration that does not exist; the adapter is a plain VantaDB-backed KV store with an injectable embedding callable.
- **Trigger:** `pip install vantadb-letta` installs `letta-client` and its transitive tree, then never imports it.
- **Fix:** drop `letta-client` from `dependencies` (or, if the intent was a real Letta client integration, implement it). The `integrations/README.md:51-52` already flags the package mismatch ("`letta 0.2` (paquete `letta`) vs pin `letta-client>=1.0.0` (paquete distinto)") as NOTICED/out-of-scope.
- **Verify:** `pip install vantadb-letta --no-deps` + import; or a `pipdeptree`-style check in CI.

### B-04 | P3 | adapters/deps — `pydantic` is imported by the crewai adapter but not declared
- **Location:** `integrations/crewai/vantadb_crewai/vectorstore.py:15` vs `integrations/crewai/pyproject.toml:25-28`
- **Claim/Break:** `from pydantic import PrivateAttr` is a direct import in the adapter, but `dependencies` lists only `vantadb-py` and `crewai`. `pydantic` currently resolves because `crewai` depends on it transitively (via `crewai-core`), so the install works today.
- **Why it matters:** This is the "package importing an undeclared dep" pattern the brief calls out: it is not an install-time break *today* because the transitive path exists, but it breaks the moment `crewai` drops or makes `pydantic` optional — and `crewai`'s dependency set is not under this repo's control. The adapter also *subclasses* a pydantic model (`VantaDBTool(CrewAIBaseTool)`, `vectorstore.py:23`), so the coupling is structural, not incidental.
- **Trigger:** a future `crewai` release removes `pydantic` from its required deps → `pip install vantadb-crewai` succeeds but `import vantadb_crewai` raises `ModuleNotFoundError: No module named 'pydantic'`.
- **Fix:** add `"pydantic>=2,<3"` to `integrations/crewai/pyproject.toml` dependencies.
- **Verify:** `pip install vantadb-crewai --no-deps` plus an explicit `pydantic` install, then import.

### B-05 | P2 | adapters/packaging — two packages claim the same PyPI distribution name and the same import module
- **Location:** `providers/openai/pyproject.toml:6` + `providers/openai/Cargo.toml:2` vs `integrations/openai/pyproject.toml:6`; `providers/ollama/pyproject.toml:6` + `providers/ollama/Cargo.toml:2` vs `integrations/ollama/pyproject.toml:6`
- **Claim/Break:** Both trees publish the same distribution names:
  | Name | Rust provider | Python adapter | Module |
  |---|---|---|---|
  | `vantadb-openai` | `providers/openai` (maturin, `dynamic = ["version"]` → Cargo 0.9.0) | `integrations/openai` (hatchling, `version = "0.9.0"`) | both `vantadb_openai` |
  | `vantadb-ollama` | `providers/ollama` (maturin → Cargo 0.9.0) | `integrations/ollama` (hatchling, `version = "0.9.0"`) | both `vantadb_ollama` |
  The two implementations expose **completely different APIs** under the same import name: the provider's `VantaDBOpenAI` has `embed` / `embed_batch` / `store` / `search` / `get` / `list` / `delete` / `list_namespaces` (`providers/openai/vantadb_openai.pyi:1-33`), while the adapter's has `add_texts` / `similarity_search` / `delete` (`integrations/openai/vantadb_openai/vectorstore.py:15-64`).
- **Why it matters:** **Mitigation is in place**: the twins are excluded from the publish lane (`release-adapters.yml:33-35` and `:74-76`: "FIND-273 … openai/ollama twins retired from the release — canonical names belong to the Rust providers"), their READMEs say so (`integrations/openai/README.md:7-12`, `integrations/ollama/README.md:7-12`), and `integrations/README.md:1` is titled "7 adapters + 2 superseded twins". The residual risk is that the **metadata still declares the colliding name and version**, so the hazard is one `pip install .` away: `integrations/openai/README.md:16` literally instructs `cd integrations/openai && pip install .`, which produces a `vantadb_openai-0.9.0` wheel that shadows the Rust provider's module locally and produces a wheel whose filename is indistinguishable from the provider's on an index.
- **Trigger:** `cd integrations/openai && pip install .` on a machine that also has the provider installed → the Python adapter replaces `vantadb_openai`; `from vantadb_openai import VantaDBOpenAI; store.embed([...])` → `AttributeError: 'VantaDBOpenAI' object has no attribute 'embed'`. And the resulting wheel is `vantadb_openai-0.9.0-py3-none-any.whl` while the provider's is `vantadb_openai-0.9.0-cp311-abi3-*.whl` — same version, same name.
- **Fix:** rename the twins' distribution names (e.g. `vantadb-openai-adapter`) and/or rename their import modules (`vantadb_openai_adapter`) so a source install can never shadow the canonical provider. If the twins are truly retired, deleting them removes the hazard entirely.
- **Verify:** a CI check that no two `pyproject.toml`/`Cargo.toml` in the repo declare the same distribution name.

### B-06 | P3 | adapters/api — `langchain` exports `None` for missing optional surfaces while keeping them in `__all__`
- **Location:** `integrations/langchain/vantadb_langchain/__init__.py:5-13`, `:20-25`
- **Claim/Break:**
  ```python
  try:
      from vantadb_langchain.store import VantaDBStore
  except ImportError:  # langgraph-checkpoint not installed - VectorStore-only env
      VantaDBStore = None  # type: ignore[assignment]
  ```
  and the same for `VantaDBCheckpointer` — but both remain in `__all__` (`:20-25`). `from vantadb_langchain import *` therefore binds `VantaDBStore = None`.
- **Why it matters:** The failure mode is silent: `from vantadb_langchain import VantaDBStore` succeeds and yields `None`, so the failure surfaces later as `AttributeError: 'NoneType' object has no attribute 'aput'` instead of an actionable `ImportError` at import time. Note the declared dependency is a **hard** requirement (`langgraph-checkpoint>=2,<5`, `integrations/langchain/pyproject.toml:30`) — I verified that `langgraph-checkpoint` 2.0.0 … 4.3.0 all ship `langgraph/store/base/__init__.py`, so the fallback should be unreachable in a correct install. It is reachable when an *incompatible* `langgraph` is present, which is exactly when a clear error matters.
- **Trigger:** `pip install langchain-core==1.6` alongside `langgraph-checkpoint<4` such that `langgraph.store.base` fails to import → `from vantadb_langchain import VantaDBStore` → `None` → `store.aput(...)` → `AttributeError`.
- **Fix:** either drop the `try/except` (the dependency is declared as required) or raise a descriptive `ImportError` subclass instead of binding `None`, and remove the names from `__all__` when unavailable.
- **Verify:** test that importing `vantadb_langchain` without `langgraph` raises a clear, actionable error.

### B-07 | P3 | adapters/api — only 1 of 9 adapters exposes `__version__`
- **Location:** `integrations/langchain/vantadb_langchain/__init__.py:15-18` vs the other eight `__init__.py` files
- **Claim/Break:** `vantadb_langchain` computes `__version__` from `importlib.metadata.version("vantadb-langchain")` (`:15-18`). The other eight packages expose no `__version__` at all, even though all nine declare `version = "0.9.0"` in their `pyproject.toml`. The Rust **providers**, by contrast, all export it (`providers/openai/src/python.rs:339`, `providers/ollama/src/python.rs:336`, `providers/litellm/src/python.rs:343`).
- **Why it matters:** Version discovery is a normal support/diagnostic operation, and it is inconsistent *within* the same repo across three separate surfaces. Note also that `vantadb_langchain.__version__` resolves to the **installed distribution** version, which for the twins' source-only install path is the adapter's version, not the provider's — so the same attribute name means different things across packages.
- **Trigger:** `python -c "import vantadb_crewai; print(vantadb_crewai.__version__)"` → `AttributeError`.
- **Fix:** add the same `importlib.metadata` block to the other eight `__init__.py` files (or a shared helper), with a `"0.0.0"` fallback.
- **Verify:** a test loop over the 9 packages asserting `__version__` is present and matches the `pyproject.toml` version.

### B-08 | P2 | adapters/docs — haystack README documents a `count_documents(filters)` signature that does not exist
- **Location:** `integrations/haystack/README.md:45` vs `integrations/haystack/vantadb_haystack/vectorstore.py:371`
- **Claim/Break:** The README's API list says: `` - `count_documents(filters)` - count documents ``. The implementation is:
  ```rust
  def count_documents(self) -> int:
  ```
  — **no parameter at all**. It counts every record in the namespace (`vectorstore.py:382-394`), with `filters={}` hard-coded in the `memory.list` call at `:386`. The neighbouring methods `filter_documents(filters)` (`:297`) and `delete_documents(filters)` (`:334`) *do* take filters, so the README's mental model is reasonable — the implementation is the odd one out.
- **Why it matters:** A caller filtering by a field (the README's own example at `:36` uses `filters={"source": "docs"}`) will call `count_documents(filters)` and get a `TypeError`; the documented call is simply impossible. There is also a semantic gap: `count_documents()` ignores filters while `filter_documents()` honours them, so the two can disagree.
- **Trigger:** `store.count_documents({"source": "docs"})` → `TypeError: count_documents() takes 1 positional argument but 2 were given`.
- **Fix:** either add the `filters` parameter to the implementation (Haystack's `DocumentStore.count_documents` protocol is filter-aware) and keep the README, or correct the README to `count_documents()` and state that it counts the whole namespace.
- **Verify:** a test asserting `count_documents()` returns the namespace total and, if the parameter is added, that it agrees with `filter_documents(filters)`.

### B-09 | P3 | adapters/docs — mem0 README uses a parameter name the code does not have
- **Location:** `integrations/mem0/README.md:42` vs `integrations/mem0/vantadb_mem0/vectorstore.py:196`
- **Claim/Break:** The README says `` - `delete(key)` - delete a memory by key ``. The implementation is `def delete(self, vector_id: str) -> None:` (`vectorstore.py:196`). Positional calls work; a keyword call using the documented name does not.
- **Why it matters:** Minor, but it is the kind of drift that produces a `TypeError` in user code. The same package's `update()` also uses `vector_id` (`vectorstore.py:201`), so the README is the outlier.
- **Trigger:** `store.delete(key="abc")` → `TypeError: delete() got an unexpected keyword argument 'key'`.
- **Fix:** change the README to `delete(vector_id)` (and align `list(user_id=None, limit=100)` with the actual signature at `vectorstore.py:278`).
- **Verify:** a doc/signature consistency check.

### B-10 | P2 | adapters/docs — crewai README's documented constructor signature raises `TypeError` when used as written
- **Location:** `integrations/crewai/README.md:44` vs `integrations/crewai/vantadb_crewai/vectorstore.py:32-44`
- **Claim/Break:** The README's API list says: `` - `VantaDBTool(name, description, db_path, namespace)` — CrewAI-compatible RAG tool ``. The actual signature is:
  ```python
  def __init__(self, embedding=None, name="VantaDB Search",
               description="Search documents stored in VantaDB", *,
               db_path="./vantadb_data", namespace=DEFAULT_NAMESPACE,
               top_k=DEFAULT_TOP_K, memory_limit_bytes=None,
               read_only=False, backend=None):
  ```
  `db_path` and `namespace` are **keyword-only** (after the `*`), and the first positional parameter is `embedding`, not `name`.
- **Why it matters:** The README's own Quickstart (`:28-33`) uses keywords and therefore works — so the defect is isolated to the API summary line, which is the line a reader copies. A caller who follows it gets a `TypeError` and, worse, a *silently wrong* first argument: `embedding` would be bound to the tool's name.
- **Trigger:** `VantaDBTool("Memory Search", "Search stored documents", "./my_data", "docs")` → `TypeError: __init__() takes 5 positional arguments but 6 were given` (4 positional supplied, 3 accepted).
- **Fix:** rewrite the line as `VantaDBTool(embedding=None, name=…, description=…, *, db_path=…, namespace=…, top_k=…)`, or simplify to `VantaDBTool(name=…, description=…, db_path=…, namespace=…)` (all keyword).
- **Verify:** a doctest-style check that the README's signature line parses against the real `inspect.signature`.

### B-11 | P2 | adapters/docs — dspy README's documented constructor signature raises `TypeError` when used as written
- **Location:** `integrations/dspy/README.md:46` vs `integrations/dspy/vantadb_dspy/vectorstore.py:46-56`
- **Claim/Break:** The README says `` - `VantaDBRetriever(db_path, namespace, k)` - DSPy-compatible retriever ``. The actual signature is `def __init__(self, embedding=None, *, db_path="./vantadb_data", namespace=DEFAULT_NAMESPACE, k=DEFAULT_TOP_K, memory_limit_bytes=None, read_only=False, backend=None)` — again keyword-only after `embedding`.
- **Why it matters:** Same class of defect as B-10, in a second package. The README's Quickstart (`:28-…`) uses keywords and works; the API summary line does not.
- **Trigger:** `VantaDBRetriever("./my_data", "docs", 5)` → `TypeError: __init__() takes 2 positional arguments but 4 were given`.
- **Fix:** rewrite the line to the keyword form.
- **Verify:** the same signature-parity check as B-10, applied to all 9 READMEs.

### B-12 | P2 | adapters/ci — the "latest" rows of the adapter-compat matrix are a no-op, so the current upstream major is never tested
- **Location:** `.github/workflows/ci-fleet.yml:364-380`, matrix rows `:331-345`
- **Claim/Break:** The install step is:
  ```bash
  if [ "${{ matrix.framework_version }}" != "latest" ]; then
    pip install "${{ matrix.adapter }}-core==${{ matrix.framework_version }}" 2>/dev/null || \
    pip install "${{ matrix.adapter }}==${{ matrix.framework_version }}" 2>/dev/null || \
    pip install "${{ matrix.framework_version }}" 2>/dev/null
  fi
  ```
  For the 8 rows whose `framework_version` is `"latest"`, the `if` is **false** and the whole block is skipped. No "latest" framework is ever installed; the adapter's own declared dependency (which carries a next-major ceiling — see B-14) is used instead. I verified that the alternatives are not a viable fallback either: `pip install "openai==latest"` → `ERROR: Invalid requirement: 'openai==latest': Expected semicolon …` (exit 1), and `pip install "latest"` resolves to an unrelated PyPI project named `latest` (0.6.0) whose build fails.
- **Why it matters:** The matrix's whole purpose (per `integrations/README.md:47-49`, the ceilings are chosen "sin evidencia de breaking") is to detect the moment the *next* major breaks the adapter. The "latest" rows are the only rows that could surface that, and they silently test the pinned version instead — a green matrix therefore proves nothing about forward compatibility. Combined with B-14 (five ceilings already exclude the current upstream major), the adapters are untested against the versions users will actually try to install.
- **Trigger:** `haystack-ai` 3.x ships (it is already out — 3.4.0); the matrix's `haystack/latest` row still installs `haystack-ai>=2.0,<3` and passes, hiding any 3.x break.
- **Fix:** invert the condition — install the pinned version when it is a concrete version, and install the *unpinned* latest when it is `"latest"`:
  ```bash
  if [ "${{ matrix.framework_version }}" != "latest" ]; then
    pip install "${{ matrix.adapter }}==${{ matrix.framework_version }}" || exit 1
  else
    pip install --upgrade "${{ matrix.adapter }}" || exit 1
  fi
  ```
  (and drop the `2>/dev/null` so failures are visible, plus a `--no-deps` variant so the adapter's own ceiling does not block the newest major).
- **Verify:** the "latest" rows must install a version outside the adapter's declared range — assert it in the job (e.g. `pip show <pkg> | grep Version`).

### B-13 | P3 | adapters/ci — the adapter-compat matrix is not a PR/push gate
- **Location:** `.github/workflows/ci-fleet.yml:264-265`
- **Claim/Break:** The `compat-matrix` job begins with `if: (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch')`. It therefore runs only on the weekly schedule and on manual dispatch — never on `push` or `pull_request`. The adapters' own unit tests do run in `release-adapters.yml` (`:25-63`, matrix of 7), but that workflow also only fires on `workflow_dispatch` or on `v*`/`adapters-v*` **tags** (`:14-15`), i.e. it is a release-time gate, not a merge gate.
- **Why it matters:** A change to an adapter that breaks it can merge and sit undetected until the next release tag or the next weekly tick. This is the CI analogue of B-14: the adapters' correctness is not continuously verified.
- **Trigger:** a PR that changes `integrations/llamaindex/vantadb_llamaindex/vectorstore.py` merges green; the breakage surfaces at the next `adapters-v*` tag.
- **Fix:** add a per-PR job that runs `pytest integrations/<adapter>/tests/` for the 7 publishable adapters (the matrix already exists in `release-adapters.yml:35`).
- **Verify:** the job must appear in the PR-required-checks list.

### B-14 | P2 | adapters/pins — five of the nine framework ceilings exclude the current upstream major
- **Location:** the nine `integrations/*/pyproject.toml` `dependencies` blocks; live PyPI metadata checked 2026-10-20
- **Claim/Break:** The project policy is explicit — "next-major es el techo seguro sin evidencia de breaking" (`integrations/README.md:47-49`) — and `integrations/test_pins.py:34-40` enforces that every framework dep has an upper bound. The problem is that five of the ceilings now exclude the **currently published** major:

  | Adapter | Declared pin | Current upstream release (live, 2026-10-20) | Current major excluded? |
  |---|---|---|---|
  | mem0 | `mem0ai>=0.1.0,<1` | `mem0ai` **2.2.1** | yes (1.x and 2.x) |
  | haystack | `haystack-ai>=2.0,<3` | `haystack-ai` **3.4.0** | yes (3.x) |
  | dspy | `dspy>=2.6,<3` | `dspy` **3.4.1** | yes (3.x) |
  | openai | `openai>=1.0,<2` | `openai` **3.35.1** | yes (2.x and 3.x) |
  | langchain | `langchain-core>=0.3,<1` | `langchain-core` **1.6.12** | yes (1.x) |

  In-range: `llama-index-core` 0.14.25 (`>=0.12,<1` ✓), `ollama` 0.6.3 (`>=0.4,<1` ✓), `letta-client` 1.12.1 (`>=1.0.0,<2` ✓ — and unused, B-03), `langgraph-checkpoint` 4.3.0 (`>=2,<5` ✓).
- **Why it matters:** `pip install vantadb-mem0` today resolves `mem0ai` 0.1.x — a major behind the ecosystem — and the adapter's `from mem0.vector_stores.base import VectorStoreBase` (`integrations/mem0/vantadb_mem0/vectorstore.py:8`) is therefore pinned to a 2024-era API surface. The same holds for `haystack-ai` 2.x vs 3.4.0 and `openai` 1.x vs 3.35.1. Because B-12 makes the "latest" matrix rows a no-op, nothing in CI detects that the excluded majors break. `integrations/README.md:50-52` records the `ollama` and `letta` mismatches as "NOTICED (fuera de scope, no fix)" but says nothing about the five ceilings above.
- **Trigger:** `pip install vantadb-mem0` → `mem0ai-0.1.x` is installed; the user cannot get `mem0ai` 2.x without `--no-deps` + manual install, and the adapter has never been run against it.
- **Fix:** move the five ceilings up one major (`mem0ai<3`, `haystack-ai<4`, `dspy<4`, `openai<4`, `langchain-core<2`) **after** B-12 is fixed so CI can prove the new range; until then, keep the ceilings and record the gap explicitly in `integrations/README.md`.
- **Verify:** with B-12 fixed, the "latest" matrix rows must pass against the new majors.

### B-15 | P3 | adapters/ci — the only CI run of the langchain LangGraph path installs outside the adapter's declared pin range
- **Location:** `.github/workflows/demo-acceptance.yml:146` and `:149` vs `integrations/langchain/pyproject.toml:27`
- **Claim/Break:** The demo installs `langgraph>=1,<2` (which pulls `langchain-core>=1.4.7`) and then installs the adapter with `--no-deps`. The adapter declares `langchain-core>=0.3,<1`, so the demo exercises `langchain-core` **1.x** — outside the range the package claims to support, and outside the range the compat matrix pins (`ci-fleet.yml:275-276`: `framework_version: "0.3"`).
- **Why it matters:** The demo therefore validates the adapter against a configuration that no user following the declared pin would have, while the declared range itself is only exercised by the weekly compat job (B-13) — and the `langchain` package the demo installs is `langgraph`, not the pinned `langgraph-checkpoint`, so `langgraph.store.base` resolves via a different dependency path than the declared one.
- **Trigger:** the demo goes green on `langchain-core` 1.6.x while `vantadb-langchain` claims `<1`; a 1.x-only API change would break the demo and not the supported range (and vice-versa).
- **Fix:** pin the demo to the adapter's declared range (`langgraph-checkpoint>=2,<5` plus `langchain-core>=0.3,<1`), or add a second demo job against the declared range.
- **Verify:** assert the installed `langchain-core` version in the job.

### Positive observations — Part B
- **B-P1 — The adapter matrix is exactly 9 and is enforced by a test.** `integrations/test_pins.py:20` lists `EXPECTED = ["crewai","dspy","haystack","langchain","letta","llamaindex","mem0","ollama","openai"]` and `test_nine_adapters_present` (`:29-30`) asserts the on-disk set matches exactly — so a 10th or a missing adapter fails CI. The 9 directories I counted match: crewai, dspy, haystack, langchain, letta, llamaindex, mem0, ollama, openai. `vantadb_shared` is correctly **not** a distribution (no `pyproject.toml`); it is vendored into the twin wheels via `[tool.hatch.build.targets.wheel.force-include]` (`integrations/openai/pyproject.toml:40-41`, `integrations/ollama/pyproject.toml:40-41`).
- **B-P2 — All 9 adapter versions equal the repo version 0.9.0.** Verified in every `pyproject.toml` (`version = "0.9.0"`), matching `release-manifest.json:4` (`"version": "0.9.0"`) and the three provider `Cargo.toml`s (`providers/openai/Cargo.toml:3`, `providers/ollama/Cargo.toml:3`, `providers/litellm/Cargo.toml:3`). The bump was applied mechanically by commit `3430c42c` across all 9 + the 3 providers + `vantadb-node`.
- **B-P3 — The declared dependencies match the actual imports for 7 of 9 packages.** I extracted every third-party import from all 12 adapter modules and compared against each `pyproject.toml`: `vantadb_py` (all 9 ✓), `crewai` ✓, `dspy` ✓, `haystack` (via `haystack-ai`) ✓, `langchain_core` ✓, `langgraph.checkpoint.base` + `langgraph.store.base` ✓ (I verified against the published wheels that `langgraph-checkpoint` 2.0.0 … 4.3.0 all ship `langgraph/store/base/__init__.py`), `llama_index.core` ✓, `mem0.vector_stores.base` (via `mem0ai`) ✓, `ollama` ✓, `openai` ✓. The two exceptions are B-03 (`letta-client`, unused) and B-04 (`pydantic`, undeclared).
- **B-P4 — `force-include` works for editable installs.** `ci-fleet.yml:369` installs the twins with `pip install -e .`; I verified in the hatchling 1.32.4 source (`hatchling/builders/wheel.py:608-610` in `build_editable_detection` and `:648-650` in `build_editable_explicit`) that forced files are copied into the editable wheel, so `import vantadb_shared` resolves in that CI path.
- **B-P5 — Packaging metadata is coherent for the 7 publishable adapters.** Each has `name`, `version`, `description`, `readme`, `license`, `requires-python = ">=3.11"`, `authors`, `keywords`, classifiers, `[project.urls]`, and `[tool.hatch.build.targets.wheel] packages = ["vantadb_<x>"]` pointing at the real package directory — the `packages` list matches the on-disk directory name in every case.
- **B-P6 — The 7/9 publish split is documented in four independent places.** `release-adapters.yml:33-35` and `:74-76` (workflow comments, FIND-273), `integrations/README.md:1` ("7 adapters + 2 superseded twins") and `:19-23`, and both twin READMEs (`integrations/openai/README.md:7-12`, `integrations/ollama/README.md:7-12`). The split is deliberate and traceable, not an accident.
- **B-P7 — Every framework dependency has an upper bound, enforced by a test.** `integrations/test_pins.py:34-40` parses each `pyproject.toml` with `packaging.requirements.Requirement` and fails if any non-`vantadb-py` dependency lacks an upper bound. The gate runs in `demo-acceptance.yml:154-155`.

---

## Coverage

### What was fully verified (read + cross-checked)
- **Providers (Rust/PyO3):** all 3 `src/python.rs` (404 + 408 + 401 lines), `providers/shared_py.rs` (336 lines), all 3 `Cargo.toml`, all 3 `pyproject.toml`, all 3 `README.md`, all 3 `.pyi`, all 3 test files. Every claim about env vars, redaction, budget/ACL/audit, streaming, timeouts and retries was checked against the source text.
- **`vanta-proxy`:** `server.rs` (routes + full pipeline), `auth.rs`, `forward.rs`, `config.rs`, `cost.rs`, `guardrails.rs`, `governance.rs`, `rate_limit.rs`, `redact.rs`, `sse_intercept.rs`, `cache.rs`, `report.rs`, `error.rs`, `envelope.rs`, `main.rs`, `Cargo.toml`, `config.toml`, `README.md`, `docs/api/PROXY.md`.
- **Adapters:** all 9 `pyproject.toml`, all 9 `README.md`, all 10 `__init__.py` (9 adapters + `vantadb_shared`), all 12 adapter modules, `integrations/test_pins.py`, `integrations/README.md`, and the workflows `release-adapters.yml`, `release-providers.yml`, `ci-fleet.yml`, `ci-rust.yml`, `demo-acceptance.yml`.
- **External facts checked live (2026-10-20):** PyPI metadata for `vantadb-py` (all 9 releases + the 0.5.0 / 0.8.0 wheel contents → `Client` absent in 0.5.0), `vantadb-openai`, `vantadb-ollama`, `vantadb-litellm` (not yet published), `mem0ai`, `llama-index-core`, `haystack-ai`, `dspy`, `letta-client`, `ollama`, `openai`, `langchain-core`, `langgraph-checkpoint` (wheel contents for 2.0.0→4.3.0 → `langgraph/store/base` present in all), `litellm` (exception classes → no `api_key` in `litellm/exceptions.py`), `hatchling` 1.32.4 (source, editable + `force-include`), `axum` 0.8.9 (`DefaultBodyLimit` = 2 MB for `Bytes`), `openai` 2.54.0 (`_client.py` api_key validation, `_exceptions.py` message construction). `pip` behaviour verified locally for `pkg==latest` (invalid requirement) and `pip install "latest"` (unrelated package, build failure).
- **Git (read-only):** `git log` / `git show` on `3234edb3`, `3430c42c`, `9f773e92`, `0f36fc24e` to establish when each version/pin drift was introduced.

### NOT VERIFICADO (could not execute in this environment — not claimed as findings)
- **`cargo audit` / `cargo deny check` results.** `cargo` is not on `PATH` in this shell (`cargo: El término 'cargo' no se reconoce`), so no advisory or license scan was executed. A-09 is a **coverage** finding (the provider lockfiles are not in the scan scope), derived from reading the workflow files and `osv-scanner.toml:5` — it is **not** a claim that a specific advisory exists in those lockfiles.
- **semgrep / ast-grep scans.** Not run (not in scope for this brief, and the tool config was not exercised).
- **Runtime behaviour of the proxy and the adapters.** No process was started, no HTTP request was issued, no Python test was executed. All proxy/adapter claims are static-reading claims with the exact source line cited.
- **Whether `pip install vantadb-<adapter>` currently resolves cleanly against PyPI** — the pins were checked against live metadata, but no resolution was performed.
- **The `mem0ai` 0.1.x / `haystack-ai` 2.x / `openai` 1.x compatibility of the adapter code** — I confirmed the pins exclude the current majors, but did not verify whether the adapters would in fact work against them (B-14 is a staleness/exposure finding, not a break claim).

### Explicitly checked and found NOT to be a problem
- **No request-controlled SSRF in `vanta-proxy`** — the upstream URL is `{config base}{hardcoded literal}`; the client-controlled `{agent}`/`{space_id}` segments never enter the URL (A-P3).
- **No missing auth on a privileged route** — every route except `/health` (which returns only `{"status":"ok"}`) calls `state.auth.authenticate(&headers)?` (A-P1, A-17).
- **The D34 credential is never forwarded upstream** — `x-vanta-user-key` is in the hop-by-hop blocklist (A-P2).
- **Request-body size is bounded** — all handlers take `bytes::Bytes`, and axum 0.8's `DefaultBodyLimit` applies 2 MB by default (no `DefaultBodyLimit::disable()` anywhere in the proxy).
- **The SSE parser itself is correct** — it handles `[DONE]` sentinels (`sse_intercept.rs:103`), CRLF (`:89`), multi-line `data:` fields (`:101`), and garbage lines (`:106-108`); the defect is the unbounded *buffer*, not the parsing (A-11).
- **Failover error propagation is correct** — the last attempt's real status is returned verbatim (`forward.rs:272-277`), never a synthetic error.
- **`vantadb-py` 0.6.1 exists on PyPI** (4 wheels, not yanked), so the `>=0.6.1` floor of the 7 publishable adapters resolves (B-01 covers only the twins' `>=0.5.0` floor).
- **All 9 adapters are version 0.9.0** = the repo version — no stale adapter versions (B-P2).
- **`langgraph-checkpoint>=2,<5` really does provide `langgraph.store.base`** — verified across every release in the range, so the langchain adapter's dependency declaration is correct (B-P3).

### Severity distribution
`P0: 0 · P1: 1 · P2: 15 · P3: 20` — **36 findings total** (21 in Part A / 15 in Part B), plus 15 positive observations.

| Severity | IDs |
|---|---|
| **P0** | — |
| **P1** | A-11 |
| **P2** | A-01, A-03, A-04, A-09, A-12, A-13, A-14, A-15, B-01, B-05, B-08, B-10, B-11, B-12, B-14 |
| **P3** | A-02, A-05, A-06, A-07, A-08, A-10, A-16, A-17, A-18, A-19, A-20, A-21, B-02, B-03, B-04, B-06, B-07, B-09, B-13, B-15 |
