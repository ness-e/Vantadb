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
| GET | `/snapshot` | live ops snapshot: recent turn reports, active sessions, write-back queue, rate-limit telemetry, cost snapshot + budget/enforce flags; **requires auth (API-05)**, no loopback bypass | `server.rs:791,871` |
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

## Opt-in features (8)

Invariant: the wire stays a transparent proxy unless explicitly opted in —
each feature below is **off by default** (`enabled: false`, or empty endpoint).
Config keys live on `ProxyConfig` (`config.rs:17-56`).

| TOML section | What it does | Default (off) | Source |
|--------------|--------------|---------------|--------|
| `[mem_command]` | in-band `mem:*` message interception (D33); off → forwarded verbatim | `enabled = false` | `config.rs:78-86` |
| `[cache]` | exact + semantic response cache (PRX-09); active only when `enabled` **and** `ttl_secs > 0` | `enabled = false`, `ttl_secs = 0` | `config.rs:137-166` |
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
| `[auth]` | local VantaDB store path for auth/sessions; every route (incl. `/snapshot`) resolves `x-vanta-user-key` against its `user` collection | `db_path = "vantadb_data"` (`config.rs:298-310`) |
| `[writeback]` | L0 write-back crash-audit file | `persist_path = "vanta-proxy-writeback-pending.json"` (`config.rs:88-103`) |
| `[[upstreams]]` | PRX-02 failover list, tried in order after `upstream`; empty (default) → legacy single-upstream behavior | `config.rs:22-25,66-75` |

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
