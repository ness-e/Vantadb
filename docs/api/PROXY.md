---
title: "vanta-proxy Reference (Endpoints, Opt-in Features, Config)"
kind: reference
status: active
description: "vanta-proxy is a transparent LLM wire proxy: by default it forwards bytes"
tags: [vantadb, api, proxy]
---

# vanta-proxy Reference

`vanta-proxy` is a transparent LLM wire proxy: by default it forwards bytes
unchanged to a single upstream, and every behavior beyond forwarding is an
explicit opt-in via `config.toml`
([`vanta-proxy/config.toml`](../../vanta-proxy/config.toml)).
This page documents the HTTP surface, the opt-in features, and every default
with its source. It is **not** a tutorial — for the config file itself, read
the TOML; for the code, follow the cited paths.

Router source of truth: [`vanta-proxy/src/server.rs`](../../vanta-proxy/src/server.rs)
(`router`, lines 788-823). Config source of truth:
[`vanta-proxy/src/config.rs`](../../vanta-proxy/src/config.rs) (decision D31:
TOML + serde, line 1).

## Endpoints

All routes are registered in `router()` (`server.rs:788-823`): 11 route
registrations covering **8 logical endpoints** (`/v1/models`,
`/v1/messages/count_tokens` and `/v1/responses` each have a plain and an
`{agent}/{space_id}`-prefixed shape, `server.rs:801-821`).

| Method | Path | Handler | Source |
|--------|------|---------|--------|
| GET | `/health` | returns `{"status":"ok"}` | `server.rs:790,825` |
| GET | `/snapshot` | live ops snapshot: recent turn reports, active sessions, write-back queue, rate-limit telemetry, cost snapshot + budget/enforce flags, envelope mode (VER-03); **requires auth (API-05)**, no loopback bypass | `server.rs:791,871` |
| POST | `/sessions/advance` | explicit session-stage trigger; body `{ "target": "team"\|"agent"\|"task", "entity_id": "<id>" }`; requires auth (401), bad target/key → 400 | `server.rs:792,834` |
| POST | `/{agent}/{space_id}/v1/chat/completions` | OpenAI chat completions (forward) | `server.rs:793-796` |
| POST | `/{agent}/{space_id}/v1/messages` | Anthropic messages (forward) | `server.rs:797-800` |
| POST | `/v1/responses`, `/{agent}/{space_id}/v1/responses` | Responses API (forward); the prefixed shape keys limiter/reports by `space_id` (API-05 parity) | `server.rs:801-804` |
| GET | `/v1/models`, `/{agent}/{space_id}/v1/models` | model discovery (Claude Code picker); IDs come **only** from `upstream.models` in config — empty (default) → `{"object":"list","data":[]}` | `server.rs:809-813`, `config.rs:324-327` |
| POST | `/v1/messages/count_tokens`, `/{agent}/{space_id}/v1/messages/count_tokens` | token counting | `server.rs:814-821` |

Wire paths (`/v1/...`) are appended to the upstream base URL as received from
the client (`config.rs:316-318`).

### Authentication (API-05)

Every route — including `GET /snapshot` — requires a valid `x-vanta-user-key`
header resolved against the local `user` entity collection (`auth.rs`, D34);
missing or unknown keys get `401`. There is deliberately **no loopback
bypass**: local silent exposure of sessions/cost was the API-05 finding.
The desktop Proxy Dashboard
([`desktop/src/components/proxy/ProxyDashboard.tsx`](../../desktop/src/components/proxy/ProxyDashboard.tsx))
does not send the header yet — tracked as `FIND-155`.

## Opt-in features (9)

Invariant: the wire stays a transparent proxy unless explicitly opted in —
each feature below is **off by default** (`enabled: false`, or empty endpoint).
Config keys live on `ProxyConfig` (`config.rs:17-61`).

| TOML section | What it does | Default (off) | Source |
|--------------|--------------|---------------|--------|
| `[mem_command]` | in-band `mem:*` message interception (D33); off → forwarded verbatim | `enabled = false` | `config.rs:78-86` |
| `[cache]` | exact + semantic response cache (PRX-09); active only when `enabled` **and** `ttl_secs > 0` | `enabled = false`, `ttl_secs = 0` | `config.rs:137-166` |
| `[report]` | per-turn span export to Langfuse/OTel over OTLP-JSON (MEM-56) | `langfuse_endpoint = ""` (empty disables export) | `config.rs:140-155` |
| `[routing]` | task-aware routing by tier (PRX-06); `Shadow` (default) vs `Enforce` | `enabled = false`, `mode = Shadow` | `config.rs:41-43`, `routing.rs:59-95` |
| `[redact]` | PII/secret redaction on egress (PRX-07) — and, when enabled, the same policy redacts captured turns **before they are persisted** (VER-03) | `enabled = false`, `mode = Mask` | `config.rs:44-46`, `redact.rs:41-63` |
| `[context]` | conversation-history trimming to a token budget (PRX-13); never blocks (`Pass` only) | `enabled = false`, `mode = Performance` | `config.rs:47-49`, `context.rs:58-84` |
| `[guardrails]` | per-key model allowlists (PRX-10); unknown key or empty entry allows | `enabled = false` | `config.rs:50-52`, `guardrails.rs:13-21,30-48` |
| `[translate]` | Anthropic→OpenAI request translation (PRX-11); off → byte-identical forward | `enabled = false` | `config.rs:53-55`, `translate.rs:29-50` |
| `[envelope]` | per-namespace AEAD envelope preserving the pre-redaction original of a captured turn (VER-03); requires `[redact] enabled` + a 32-byte `VANTADB_ENCRYPTION_KEY` | `enabled = false`, `key_version = 1` | `config.rs:59-61`, `envelope.rs` |

Related but **not** opt-in (documented here to avoid confusion):

| Section | Behavior | Source |
|---------|----------|--------|
| `[cost]` | **tracking ON, enforcement OFF** by default (`enabled = true`, `default_budget_usd = None`, `enforce = false` — log-first: over budget warns + allows unless `enforce = true` → 429) | `config.rs:157-184` |
| `[auth]` | local VantaDB store path for auth/sessions; every route (incl. `/snapshot`) resolves `x-vanta-user-key` against its `user` collection | `db_path = "vantadb_data"` (`config.rs:298-310`) |
| `[writeback]` | L0 write-back crash-audit file | `persist_path = "vanta-proxy-writeback-pending.json"` (`config.rs:88-103`) |
| `[[upstreams]]` | PRX-02 failover list, tried in order after `upstream`; empty (default) → legacy single-upstream behavior | `config.rs:22-25,66-75` |
| `[injection]` | `<vanta-memory>` block budget is **on by default** (`max_tokens = 2000`, `0` disables injection); the VER-04 ACL + audit sub-keys are opt-in — see § Injection governance | `config.rs` (`InjectionConfig`) |

## Redaction-on-write & encrypted namespaces (VER-03)

`[redact]` applies to **persistence** too: a captured turn — automatic L0
capture and the `vanta_memory_capture` tool both go through
[`capture::turn_job`](../../vanta-proxy/src/capture.rs) — is stored redacted.
Both destinations carry the masked text: `proxy-turns` (audit payload) and
`l1/{session}` (the search-facing record, the only surface recall/injection
reads). The audit payload also records the kind labels found
(`"redacted": ["email", "aws_key", …]` — labels only, never values).

Mode mapping on write: `Mask` and `Block` mask (a completed turn is never
dropped or rejected — masking is the only lossless outcome); `Log` observes
only, the same semantic as the wire. With `[redact] disabled` nothing is
scanned and persistence is byte-identical to the pre-VER-03 behavior.

Scan cap (shared with egress): turns above `redact.max_scan_bytes` (2 MiB)
fail open — a warning is logged, the turn is persisted **unscanned** and gets
**no envelope**. The zero-cleartext guarantee holds under the cap; shorten
turn capture or raise the cap if your traffic exceeds it.

When `[envelope] enabled = true`, the **pre-redaction original** of a turn
additionally survives — and only — inside an AEAD envelope attached to the
`proxy-turns` audit payload:

```json
{ "v": 1, "k": 1, "ns": "proxy-turns", "ct": "<hex>" }
```

| Field | Meaning |
|-------|---------|
| `v` | envelope framing version (bump on format change) |
| `k` | key version used for the derivation (rotation marker) |
| `ns` | namespace whose derived key seals the payload |
| `ct` | hex of AES-256-GCM `[nonce ‖ ciphertext ‖ tag]` |

Key derivation: `key(ns, k) = HKDF-SHA256(master, salt =
"vanta-namespace-envelope-v1", info = "ns" ‖ ns ‖ [k])`, where `master` is the
project-wide 32-byte `VANTADB_ENCRYPTION_KEY` (same AEAD primitive as storage
encryption, domain-separated by salt + info). Namespace and version are part
of the key: a blob re-labelled with another `ns` or `k` does not authenticate.

**Rotation (v1):** bump `key_version` — new writes seal under the new derived
key while previous envelopes stay readable under the same master key.
Rotating the master key itself requires re-encrypting existing envelopes (not
automated yet — declared limit). **Recovery** is programmatic via
`Envelope::open` (same crate); there is no CLI for it yet.

**Degradation is explicit — never a silent fallback to clear text:**

| `[envelope]` | key | `[redact]` | Result |
|--------------|-----|------------|--------|
| off (default) | — | any | redacted text is the only copy (or nothing redacted when `[redact]` is off) |
| on | valid 32-byte | on | envelopes active: the original is recoverable only with its key |
| on | missing / invalid / passphrase | on | **disarmed**: startup warning, envelopes skipped, the original is NOT persisted — it is never written in clear |
| on | valid | off | no redaction ran → no envelope (there is nothing to hide) |

`mode = "log"` observes without mutating (same semantic as the wire): the
persisted text stays as-is and **no envelope is written** — the envelope only
ever preserves an original that redaction actually masked (`Mask`/`Block`).

The effective mode and key version are visible in `GET /snapshot` under
`"envelope"` (`redacted_only` · `active` · `disarmed`).

### PII audit (`vanta-pii-audit`)

The zero-cleartext guarantee is auditable on a real store with the versioned
scan binary — the same built-in detectors as `[redact]`, byte-oriented
(binary-safe: non-UTF-8 files are scanned byte-wise, never skipped) so
records, derived indexes, WAL and column stores are covered in one sweep:

```bash
cargo run -p vanta-proxy --bin vanta-pii-audit -- --json ./db ./export-v2.jsonl
```

Output is value-free (`path`, `kind`, `offset` — never the matched text).
`--pattern <regex>` adds the operator patterns the `[redact]` config uses.
Exit `0` = every byte of every file scanned, nothing matched; `1` = findings
and/or `unscanned` files (oversize > 64 MiB, symlink, unreadable) — **not
audited is not clean**; `2` = usage error. On Windows, run it against a closed
store: an open engine byte-range-locks its lock files. Full walkthrough:
[PRIVACY.md](../user/PRIVACY.md).

## Injection governance (VER-04)

The `<vanta-memory>` system-prompt block (persona + scene navigation, WIRE-01)
is built under four governance rules. All four are enforced per request:

**1. Budget.** `[injection] max_tokens` (default `2000`) caps the block with
the canonical `estimate_text_tokens` heuristic (`ceil(len/4)`, wrapper tags
included). Sections enter in priority order — persona, current scene, scene
index — and the first one that would overflow is dropped with everything below
it; when even the top section alone overflows it is hard-truncated to fit with
a visible `…[truncated]` marker. `max_tokens = 0` disables injection entirely
(empty block → the request is forwarded untouched). The same budget bounds the
`vanta_memory_search` tool result (char cap = `max_tokens * 4`).

**2. Namespace ACL (opt-in).** `[injection] namespace_allow_prefixes = ["l1/",
"persona/", "scene/"]` restricts which namespaces the block (and the search
tool) may read from. Empty (default) allows everything — current behavior.
Matching is boundary-aware: `l1/sess-1` allows `l1/sess-1` and
`l1/sess-1/...` but never `l1/sess-12`. Anything not matched is skipped
(`deny fuera de scope`) and recorded as a `denied` audit event — never silent
(the governance lists are bounded to 16 entries; when more namespaces were
involved the last slot degrades to `…overflow`, so the bound never hides the
ACL's existence). A pass where the ACL denied every source still audits: the
`denied` events are emitted even though nothing was injected.

**3. Trust classes (opt-in; MGR-04).** `[injection] tainted_namespaces =
["l1/scratch/", "imports/"]` classifies namespace prefixes as **tainted**:
their content is never injected by default (the block skips them and the skip
is recorded as a `denied` audit event). This is the content-trust gate — it is
orthogonal to the ACL and both compose with AND (to inject, a source must pass
the ACL *and* be trusted). Set `[injection] include_tainted = true` to opt back
in for review workflows (the ACL above still applies). Empty (default) = every
namespace trusted — current behavior. Classifying a namespace is an explicit
operator act (a config change): nothing is promoted or demoted automatically by
time or content.

**4. Injection audit (opt-in).** `[injection] audit_log_path = "injection-audit.jsonl"`
turns on an append-only JSONL audit (rotated like the core audit log: 10 MiB ×
5 files) where every injected memory and every denial (ACL or trust) leaves one
event:

```json
{"timestamp":"2026-09-29T12:00:00Z","op":"injection","namespace":"persona/sess-1","key":"persona.md","outcome":"ok","reason":"surface=proxy;tool=prompt_block;session=sess-1;kind=persona;budget=42/2000;truncated=false;acl=allow"}
```

Metadata only — namespace, key, score, budget, truncation and ACL decision;
the memory content never lands in the audit (pre-mortem F1 of VER-04).

**Consulting the audit** (``rg``/`jq` over the JSONL; no UI by design):

```bash
# every memory injected into a session's prompts, with scores
rg '"op":"injection"' injection-audit.jsonl

# only ACL denials (nothing was injected from those namespaces)
jq -c 'select(.op=="injection" and .outcome=="denied")' injection-audit.jsonl

# injections that hit the budget cap
jq -c 'select(.reason | contains("truncated=true"))' injection-audit.jsonl
```

This JSONL is WORM-ready (append-only + rotation). Cryptographic chaining of
the audit is deliberately out of scope here: the tamper-evident hash-chain of
the WAL (`vanta-cli verify`, VER-01) is the chained evidence surface — cited,
not duplicated.

## North Star metric (ICP-01)

The product North Star — *sessions with put + search in a 7-day window*
(`SPEC.md` §North Star) — is measurable directly from the proxy store. Both
halves are persisted in the proxy's own database (`[auth] db_path`):

| Half | Where it lands | Shape |
|------|----------------|-------|
| **PUT** | `proxy-turns` — every completed proxied turn (automatic L0 capture) | key `{ms}-{seq}`, payload `{session, protocol, space, model, text, …}` |
| **SEARCH** | `proxy-memory-events` — one record per executed `vanta_memory_search` tool call | key `{ms}-{seq}`, payload `{session, kind:"search", hits}` |

The search event is written through the same fire-and-forget L0 write-back as
turn capture (`WriteBack::track`) and is **metadata-only**: session, kind and
hit count — the query text and the recalled content never land there. It adds
no latency to the wire (the tool result returns before the write settles).

The metric is the intersection of sessions present in **both** namespaces
(search side with `hits >= 1`) inside the same window, filtered client-side by
the `{ms}` head of each key:

```bash
# Requires a vanta-cli with the `mcp-call` subcommand (0.8.0 train; on `develop` today).
python scripts/north_star_metric.py --db <proxy-db-path> [--days 7] [--json]
```

`--self-test` exercises the windowing logic offline. Under the hood the script
issues two read-only `memory_list` MCP calls —
`vanta-cli mcp-call --db <path> --tool memory_list --args '{"namespace":"proxy-turns"}'`
and the same for `proxy-memory-events` — and intersects them; any MCP-capable
client can run the equivalent queries by hand.

## Defaults

### `[server]` / `[upstream]`

These are the only two sections present in the shipped
[`vanta-proxy/config.toml`](../../vanta-proxy/config.toml) (lines 4-19).

| Key | Default | Source |
|-----|---------|--------|
| `server.host` | `"0.0.0.0"` | `config.rs:219-231` |
| `server.port` | `8096` (`DEFAULT_PORT`) | `config.rs:11-12,219-231` |
| `server.rate_limit_per_minute` | `60` (single enforcement point: the in-process sliding window in `process_inner` step 2; `/snapshot` only reports telemetry) | `config.rs:222`, `config.toml:9` |
| `upstream.url` | `""` — **no default**: `validate_startup` refuses to start without an explicit URL (API-05; the old default self-looped into this proxy). Shipped TOML sets `"https://api.anthropic.com"` | `config.rs:253,358-370`, `config.toml:14` |
| `upstream.api_key` | `""` (empty → incoming `Authorization` header passes through verbatim) | `config.rs:319-321` |
| `upstream.forward_timeout_secs` | `600` (`DEFAULT_FORWARD_TIMEOUT_SECS`, TDAM parity) | `config.rs:9-10,322-323` |
| `upstream.models` | `[]` (empty → `GET /v1/models` returns `{"object":"list","data":[]}`) | `config.rs:324-327` |

> Note: the code default `upstream.url` is empty and refuses to start
> (`validate_startup`, `config.rs:253`); an explicit URL pointing back at the
> proxy's own port is refused by `points_at_self` (`config.rs:334-340`).

### Feature tuning defaults

| Key | Default | Source |
|-----|---------|--------|
| `cache.max_entries` | `128` | `config.rs:137-166` |
| `cache.ttl_secs` | `0` — caching is active only when `enabled` **and** `ttl_secs > 0`; 0 (default) means no TTL configured → cache disabled (API-05 X5: the old "never expires" convention was inverted) | `config.rs:137-166` |
| `cache.semantic_enabled` | `false` | `config.rs:137-166` |
| `cache.similarity_threshold` | `0.90` (`DEFAULT_SIMILARITY_THRESHOLD`) | `cache.rs:32` |
| `cache` body cap | `4 MiB` (`MAX_CACHEABLE_BODY_BYTES` — larger bodies not cached) | `cache.rs:29` |
| `redact.max_scan_bytes` | `2 MiB` (larger bodies fail open) | `redact.rs:17-20,54-63` |
| `context.max_input_tokens` | `8000` | `context.rs:25-28,74-84` |
| `context.max_scan_bytes` | `2 MiB` (larger bodies fail open) | `context.rs:30-33,74-84` |
| `translate` token clamp | default `1024` (`DEFAULT_MAX_TOKENS`), hard cap `128_000` (`MAX_MAX_TOKENS`) | `translate.rs:15-23` |
| `injection.max_tokens` | `2000` (`DEFAULT_INJECTION_MAX_TOKENS`); `0` disables memory injection | `config.rs` (`InjectionConfig`) |
| `injection.namespace_allow_prefixes` | `[]` — empty = allow-all (VER-04 ACL off) | `config.rs` (`InjectionConfig`) |
| `injection.tainted_namespaces` | `[]` — empty = every namespace trusted (MGR-04 trust gate off); listed prefixes never inject by default | `config.rs` (`InjectionConfig`) |
| `injection.include_tainted` | `false` — opt-in to inject tainted namespaces (review workflows; ACL still applies) | `config.rs` (`InjectionConfig`) |
| `injection.audit_log_path` | `""` — empty = injection audit disabled | `config.rs` (`InjectionConfig`) |

### `[cost]` price table (USD per 1K tokens)

Default `PriceTable` (`cost.rs:50-74`); unknown models fall back to
`__default__` (`cost.rs:76-84`). Prices go stale — they are configurable, and
the fallback is documented in code (`cost.rs:41-43`).

| Model | Input / 1K | Output / 1K | Source |
|-------|-----------|-------------|--------|
| `gpt-4o` | 0.0025 | 0.010 | `cost.rs:53` |
| `gpt-4o-mini` | 0.00015 | 0.0006 | `cost.rs:54` |
| `claude-3-5-sonnet` | 0.003 | 0.015 | `cost.rs:55` |
| `claude-3-haiku` | 0.00025 | 0.00125 | `cost.rs:56` |
| `__default__` (unknown models) | 0.002 | 0.008 (`ModelPrice::default`) | `cost.rs:32-38,71` |

Token estimates use `ceil(len/4)` over raw bytes — fine for budget
guardrails, not billing (`cost.rs:86-90`; ±30% vs real tokenizers by design).

## Environment variables

The only `std::env::var` reads in `vanta-proxy/src` (plus the tracing filter):

| Variable | Effect | Default when unset | Source |
|----------|--------|-------------------|--------|
| `VANTA_PROXY_CONFIG` | config file path (CLI arg takes precedence) | `"config.toml"` | `main.rs:18-22` |
| `VANTA_EMBED_BASE_URL` | Ollama endpoint for semantic-cache embeddings | `"http://localhost:11434"` | `cache.rs:599-606` |
| `VANTA_EMBED_MODEL` | embedding model for semantic-cache | `"nomic-embed-text"` | `cache.rs:602-604` |
| `RUST_LOG` | log filter (tracing `EnvFilter`; not proxy config) | `"info"` | `main.rs:11-16` |

Usage: `vanta-proxy [path/to/config.toml]` (`config.toml:2`, `main.rs:18`).
On shutdown the proxy drains pending L0 writes (10 s grace) before exiting
(`main.rs:44-49`).

## Keeping this page in sync

This page diverges from the code within a month if untended (pre-mortem).
Full checklist in [`docs/dev/tasks/FIND-68.md`](../dev/tasks/FIND-68.md)
(§ Checklist anti-drift); the mechanical check is:

```powershell
rg -c "\.route\(" vanta-proxy/src/server.rs   # must equal 11 registrations
```
