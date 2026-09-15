---
title: vanta-proxy Reference (Endpoints, Opt-in Features, Config)
type: api
status: active
tags: [vantadb, api, proxy]
last_reviewed: 2026-09-15
aliases: []
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
(`router`, lines 741-772). Config source of truth:
[`vanta-proxy/src/config.rs`](../../vanta-proxy/src/config.rs) (decision D31:
TOML + serde, line 1).

## Endpoints

All routes are registered in `router()` (`server.rs:741-772`): 10 route
registrations covering **8 logical endpoints** (`/v1/models` and
`/v1/messages/count_tokens` each have a plain and an
`{agent}/{spaceId}`-prefixed shape, `server.rs:758-770`).

| Method | Path | Handler | Source |
|--------|------|---------|--------|
| GET | `/health` | returns `{"status":"ok"}` | `server.rs:743,774-776` |
| GET | `/snapshot` | live ops snapshot: recent turn reports, active sessions, write-back queue, rate-limit telemetry, cost snapshot + budget/enforce flags | `server.rs:744,816-840` |
| POST | `/session/advance` | explicit session-stage trigger; body `{ "target": "team"\|"agent"\|"task", "entity_id": "<id>" }`; requires auth (401), bad target/key → 400 | `server.rs:745,778-811` |
| POST | `/{agent}/{spaceId}/v1/chat/completions` | OpenAI chat completions (forward) | `server.rs:746-749` |
| POST | `/{agent}/{spaceId}/v1/messages` | Anthropic messages (forward) | `server.rs:750-753` |
| POST | `/v1/responses` | Responses API (forward) | `server.rs:754` |
| GET | `/v1/models`, `/{agent}/{spaceId}/v1/models` | model discovery (Claude Code picker); IDs come **only** from `upstream.models` in config — empty (default) → `{"object":"list","data":[]}` | `server.rs:758-762`, `config.rs:235-238` |
| POST | `/v1/messages/count_tokens`, `/{agent}/{spaceId}/v1/messages/count_tokens` | token counting | `server.rs:763-770` |

Wire paths (`/v1/...`) are appended to the upstream base URL as received from
the client (`config.rs:227-229`).

## Opt-in features (8)

Invariant: the wire stays a transparent proxy unless explicitly opted in —
each feature below is **off by default** (`enabled: false`, or empty endpoint).
Config keys live on `ProxyConfig` (`config.rs:17-56`).

| TOML section | What it does | Default (off) | Source |
|--------------|--------------|---------------|--------|
| `[mem_command]` | in-band `mem:*` message interception (D33); off → forwarded verbatim | `enabled = false` | `config.rs:78-86` |
| `[cache]` | exact + semantic response cache (PRX-09) | `enabled = false` | `config.rs:108-122` |
| `[report]` | per-turn span export to Langfuse/OTel over OTLP-JSON (MEM-56) | `langfuse_endpoint = ""` (empty disables export) | `config.rs:140-155` |
| `[routing]` | task-aware routing by tier (PRX-06); `Shadow` (default) vs `Enforce` | `enabled = false`, `mode = Shadow` | `config.rs:41-43`, `routing.rs:59-95` |
| `[redact]` | PII/secret redaction on egress (PRX-07) | `enabled = false`, `mode = Mask` | `config.rs:44-46`, `redact.rs:41-63` |
| `[context]` | conversation-history trimming to a token budget (PRX-13); never blocks (`Pass` only) | `enabled = false`, `mode = Performance` | `config.rs:47-49`, `context.rs:58-84` |
| `[guardrails]` | per-key model allowlists (PRX-10); unknown key or empty entry allows | `enabled = false` | `config.rs:50-52`, `guardrails.rs:13-21,30-48` |
| `[translate]` | Anthropic→OpenAI request translation (PRX-11); off → byte-identical forward | `enabled = false` | `config.rs:53-55`, `translate.rs:29-50` |

Related but **not** opt-in (documented here to avoid confusion):

| Section | Behavior | Source |
|---------|----------|--------|
| `[cost]` | **tracking ON, enforcement OFF** by default (`enabled = true`, `default_budget_usd = None`, `enforce = false` — log-first: over budget warns + allows unless `enforce = true` → 429) | `config.rs:157-184` |
| `[auth]` | local VantaDB store path for auth/sessions | `db_path = "vantadb_data"` (`config.rs:206-221`) |
| `[writeback]` | L0 write-back crash-audit file | `persist_path = "vanta-proxy-writeback-pending.json"` (`config.rs:88-103`) |
| `[[upstreams]]` | PRX-02 failover list, tried in order after `upstream`; empty (default) → legacy single-upstream behavior | `config.rs:22-25,66-75` |

## Defaults

### `[server]` / `[upstream]`

These are the only two sections present in the shipped
[`vanta-proxy/config.toml`](../../vanta-proxy/config.toml) (lines 4-19).

| Key | Default | Source |
|-----|---------|--------|
| `server.host` | `"0.0.0.0"` | `config.rs:196-204` |
| `server.port` | `8096` (`DEFAULT_PORT`) | `config.rs:11-12,196-204` |
| `server.rate_limit_per_minute` | `60` (placeholder, enforced in MEM-27) | `config.rs:192-193`, `config.toml:8` |
| `upstream.url` | `"http://127.0.0.1:8096"` (code default; shipped TOML sets `"https://api.anthropic.com"`) | `config.rs:269-278`, `config.toml:12` |
| `upstream.api_key` | `""` (empty → incoming `Authorization` header passes through verbatim) | `config.rs:230-232` |
| `upstream.forward_timeout_secs` | `600` (`DEFAULT_FORWARD_TIMEOUT_SECS`, TDAM parity) | `config.rs:9-10,233-234` |
| `upstream.models` | `[]` (empty → `GET /v1/models` returns `{"object":"list","data":[]}`) | `config.rs:235-238` |

> Note: the code default `upstream.url` points at the proxy's own default
> port; a self-looping URL is detected by `points_at_self` (PRX-08 S2,
> `config.rs:241-252`). The shipped `config.toml` overrides it with the real
> upstream, so normal deployments never hit this.

### Feature tuning defaults

| Key | Default | Source |
|-----|---------|--------|
| `cache.max_entries` | `128` | `config.rs:124-134` |
| `cache.ttl_secs` | `0` (entries never expire) | `config.rs:124-134` |
| `cache.semantic_enabled` | `false` | `config.rs:124-134` |
| `cache.similarity_threshold` | `0.90` (`DEFAULT_SIMILARITY_THRESHOLD`) | `config.rs:131`, `cache.rs:32` |
| `cache` body cap | `4 MiB` (`MAX_CACHEABLE_BODY_BYTES` — larger bodies not cached) | `cache.rs:25-26` |
| `redact.max_scan_bytes` | `2 MiB` (larger bodies fail open) | `redact.rs:17-20,54-63` |
| `context.max_input_tokens` | `8000` | `context.rs:25-28,74-84` |
| `context.max_scan_bytes` | `2 MiB` (larger bodies fail open) | `context.rs:30-33,74-84` |
| `translate` token clamp | default `1024` (`DEFAULT_MAX_TOKENS`), hard cap `128_000` (`MAX_MAX_TOKENS`) | `translate.rs:15-23` |

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
| `VANTA_EMBED_BASE_URL` | Ollama endpoint for semantic-cache embeddings | `"http://localhost:11434"` | `cache.rs:592-599` |
| `VANTA_EMBED_MODEL` | embedding model for semantic-cache | `"nomic-embed-text"` | `cache.rs:596-598` |
| `RUST_LOG` | log filter (tracing `EnvFilter`; not proxy config) | `"info"` | `main.rs:11-16` |

Usage: `vanta-proxy [path/to/config.toml]` (`config.toml:2`, `main.rs:18`).
On shutdown the proxy drains pending L0 writes (10 s grace) before exiting
(`main.rs:44-49`).

## Keeping this page in sync

This page diverges from the code within a month if untended (pre-mortem).
Full checklist in [`docs/tasks/FIND-68.md`](../tasks/FIND-68.md)
(§ Checklist anti-drift); the mechanical check is:

```powershell
rg -c "\.route\(" vanta-proxy/src/server.rs   # must equal endpoint rows above
```
