---
title: "ADR-029 review guide — human articulation gate for D21-D36"
type: adr-guide
status: active
tags: [vantadb, architecture, adr, vanta-memory, vanta-proxy, governance]
created: 2026-08-22
related: [ADR-029-vanta-memory-context-engine.md]
---

# Review Guide: ADR-029 articulation + decisions D24-D37

> **Purpose (Regla 5 / D41).** Architectural decisions are not closed until the
> **human author** articulates them in their own words. This document is prep
> material produced by the AI: trade-off context, technical evidence, accepted
> consequences, and socratic questions per decision. It deliberately does NOT
> draft any decision in first person — the act of articulation IS the decision.
>
> Your deliverable: edit `ADR-029-vanta-memory-context-engine.md` (and decide
> where D24-D37 live — see checklist) with your own words, then sign and commit.
> Answering the socratic questions out loud before writing is the fastest way
> to find what you still don't understand.

Evidence notation: `file:line` verified against the working tree on 2026-08-22;
TDAM references are the TypeScript origin (`MemoryKnowledge/`) cited by plans
P27/P29/P30 task files.

---

## D21 — Token estimation: `chars / 3`, no tiktoken

**Trade-off context.** Alternatives on the table were (a) real BPE/tiktoken,
(b) per-model tokenizer tables, (c) heuristic chars/token. The estimator only
decides *when to trigger compression* and *how much to cut* — it never gates
correctness or persistence.

**Evidence.**
- `vanta-memory/src/context_engine/token_estimator.rs::TokenEstimator` — `chars_per_token: 3` over role-line + content (TDAM parity `extractLlmVisibleText`).
- ADR-029 §D21 documents the rejected tiktoken alternative (new heavy dep, versioned vocabularies, coupling of a generic crate to one tokenizer).
- Compression always cuts *below* budget (never exact), so estimation error only moves the trigger point.

**Consequences assumed.**
- Systematic ±20% error tolerated; ~2x off for CJK text (~1.5 chars/token real). Confirmed accepted risk in D37.
- Upgrade path already exposed: configurable `chars_per_token`; per-script estimator later.

**Socratic questions.**
1. ¿Podés explicar por qué un error sistemático de ±20% es aceptable acá pero sería fatal en un contador de facturación?
2. ¿Qué clase de texto degrada peor con chars/3 y cuál es exactamente la consecuencia visible para el usuario (compresión tardía vs datos corruptos)?
3. Si mañana activás tiktoken opt-in, ¿qué parte del contrato del pipeline cambia y qué queda intacto?

---

## D22 — `recall_scope` híbrido: `session | agent | team`, default `agent`

**Trade-off context.** Alternatives: session-only forever (pre-MEM-40 behavior),
team-wide default (max recall, max leak), agent default (TDAM de-facto behavior
without cross-agent leak).

**Evidence.**
- `vanta-memory/src/core/hooks/auto_recall.rs::RecallScope` — three variants; default `Agent`.
- Visibility rules: own-session records always visible; legacy records without `agent_id`/`team_id` stay session-only (never disappear when scope widens).
- Implementation: full scan of `l1/*` namespaces excluding own — O(#sessions + #records) per recall, acceptable at hundreds of sessions. Documented upgrade path: sessions-per-agent index.

**Consequences assumed.**
- Linear scan cost grows with session count until the index lands.
- Team scope is opt-in per call — an operator misconfiguration can widen exposure.

**Socratic questions.**
1. ¿Por qué default `agent` y no `team`? ¿Qué leak concreto evitás y qué conveniencia pagás?
2. ¿Podés explicar por qué los records legacy sin metadata permanecen session-only en vez de heredar el scope ampliado?
3. ¿En qué punto de #sesiones el full scan deja de ser aceptable y qué señal te avisa antes?

---

## D23 — MMD META format `{created, updated, summary, heat}`

**Trade-off context.** Alternatives: rich schema from day one, no META at all
(raw blobs), minimal META without heat. Chosen: fixed small contract shared
with `SceneMeta`/`SceneNode` core conventions.

**Evidence.**
- `vanta-memory/src/context_engine/mmd.rs::SceneMeta` / `TaskMemory`.
- Body cap 4000 chars (`MAX_MMD_CONTENT_CHARS`, ~1300 tokens); history keys FNV-1a(content+updated) idempotent.
- Heat semantics: CREATE = 1, UPDATE = old + 1. The MMD store is CRUD-dumb — it never mutates META; semantics live in the strategy layer.

**Consequences assumed.**
- `summary` is a descriptive placeholder until L1 produces real semantic summaries.
- Dedup fingerprint `{len}:{first 64 chars}` accepts theoretical collisions for identical-prefix content.

**Socratic questions.**
1. ¿Podés explicar por qué el store es CRUD tonto y la semántica de META vive en la estrategia? ¿Qué bug evita ese split?
2. ¿Qué pasa con `created` si un merge LLM reescribe el body completo — quién lo preserva?
3. ¿Cuándo se vuelve insuficiente el fingerprint de dedup por prefijo?

---

## D24 — Rate limit: sliding window **in-process**, fail-open, no Redis

**Trade-off context.** Alternatives: Redis-backed shared limiter (multi-instance
correct, new infra), fail-closed limiter (protects upstream, breaks availability),
in-process fail-open (chosen). Local-first product → single instance by design.

**Evidence.**
- `vanta-proxy/src/rate_limit.rs:1` — header doc names D24 (no Redis) explicitly.
- `rate_limit.rs:19` `WINDOW_MS = 60_000`; sliding window keyed `spaceId×model` (TDAM parity `redis-store.ts:324-326`).
- Fail-open on degraded mechanism with warn log (TDAM parity `guard.ts:40-51`); 429 carries `Retry-After` + `x-ratelimit-*` headers.

**Consequences assumed.**
- Multi-instance deployments share no state — N instances ≈ N× the limit (accepted risk, D37).
- During degraded mode the limit is advisory: excess requests pass.

**Socratic questions.**
1. ¿Por qué fail-open y no fail-closed cuando el mecanismo de rate-limit se degrada? ¿Qué falla concreta preferís tener en producción?
2. Si mañana corrés 3 instancias detrás de un balanceador, ¿qué comportamiento exacto esperás del límite y por qué lo aceptamos igual?
3. ¿Por qué la key es `spaceId×model` y no global ni por user_key?

---

## D25 + D34 — Proxy auth: mandatory local RBAC via `entity_*`

**Trade-off context.** Alternatives: remote Gateway `/v3/meta/auth/verify`
(TDAM's path), open/no-auth mode, local RBAC against the entity store (chosen).
D25 fixes *where* auth lives (local); D34 fixes that it is *mandatory* (every
request without a valid key → 401, no open mode).

**Evidence.**
- `vanta-proxy/src/auth.rs:1` — header names D25/D34.
- `auth.rs:78` missing/unknown key → `ProxyError::Unauthorized`; `auth.rs:86-90` resolution queries `entity_list(AUTH_ENTITY_NS, "user", ...)` — port of the MEM-05 pattern `src/cli_server.rs::resolve_user_key`.

**Consequences assumed.**
- Auth availability == local VantaDB storage availability; a storage outage blocks the proxy at the auth hop.
- No distributed revocation — user_keys live and die in the local entity store.

**Socratic questions.**
1. ¿Podés explicar la diferencia entre D25 y D34 sin mirar las notas? (pista: dónde vive vs es opcional)
2. Cuando el storage local está caído, ¿el proxy responde 401 o 5xx? Mirá `auth.rs:86-90`: ¿qué error propaga `entity_list` y qué status produce?
3. ¿Qué modelo de amenaza hace inaceptable un modo open, incluso para desarrollo?

---

## D26 — sessionKey from headers + local state machine, TTL 30min pending-only

**Trade-off context.** Alternatives: server-side session ids minted by the
proxy (breaks client compatibility), TDAM's exact header cascade + local state
machine (chosen). TTL applies ONLY to pending states — established stages are
sticky for the conversation lifetime.

**Evidence.**
- `vanta-proxy/src/session.rs:20-28` — header aliases `x-conversation-id`, `x-session-id`, ... (TDAM parity `session-key.ts:9-19`).
- `session.rs:28` `PENDING_TTL_MS = 30*60*1000`; `session.rs:90-94` lazy sweep drops expired pendings on every request (state machine team→agent→task, TDAM `store.ts:31,116`).

**Consequences assumed.**
- Lazy sweep means expired pendings can survive between requests if traffic stops — memory bounded by request frequency, not by wall clock.
- Header alias order is a compatibility contract: changing precedence silently remaps conversations.

**Socratic questions.**
1. ¿Por qué el TTL aplica solo a estados pending y no a todo el ciclo team→agent→task?
2. ¿Podés explicar el orden exacto de aliases de headers y qué rompería invertir dos de ellos?
3. ¿Qué leak de memoria evita el sweep lazy y cuándo sería insuficiente?

---

## D27 — F7 as a single worker ("1 servicio, no 2")

**Trade-off context.** Alternative rejected: copying TDAM's Knowledge Service +
Knowledge Panel as two separate services. Chosen: ONE worker orchestrating
extract+commit+index; wiki store LLM-free in core, ingest-with-LLM in vanta-memory.

**Evidence.**
- `docs/research/tdam/08-knowledge-panel-sdk.md:102` — "Ingest wiki = 1 servicio, no 2: un solo worker (`realWikiWorker`, module.ts:174-202)"; `:116` main finding repeats it.
- `src/wiki/{mod,store,state}.rs` — LLM-free state machine `pending→processing→ready|failed` in core (MEM-28).
- `vanta-memory/src/ingest/` — serial merge worker under global LLM concurrency limit (MEM-30).

**Consequences assumed.**
- Core gains a wiki module that is inert without vanta-memory driving it.
- One worker = single serialization point for ingest throughput.

**Socratic questions.**
1. ¿Por qué el store vive en core (LLM-free) y el ingest en vanta-memory (con LLM)? ¿Qué línea de dependencias se rompería al revés?
2. ¿Qué costo operacional evitó no copiar los 2 servicios de TDAM y qué capacidad futura se sacrificó?
3. Si dos builds de ingest corren en paralelo, ¿qué garantiza el run_id/state machine sobre cuál gana?

---

## D28 — code_* tools over our OWN graphrag (no external codegraph dep)

**Trade-off context.** Alternative: bridge npm `@colbymchenry/codegraph` like
TDAM does (platform-pkg resolution, external binary). Chosen: expose the
existing in-house graph primitives as MCP tools.

**Evidence.**
- `src/graph.rs:61 bfs_traverse`, `:234 dfs_traverse`, `:258 topological_sort` — pre-existing primitives (verified in P30 Task 1 audit).
- `vantadb-mcp/src/code.rs` — 8 query-only code_* tools (MEM-32), each mapped to a local primitive; rejected dep documented in research `08-knowledge-panel-sdk.md:45`.

**Consequences assumed.**
- Tool semantics (impact/callers) differ from TDAM's codegraph — mapping is approximate, not byte-compatible.
- Query-only surface: no write tools exposed.

**Socratic questions.**
1. ¿Qué semántica de TDAM codegraph NO replica nuestro graphrag y cómo lo documentaste como recorte honesto?
2. ¿Por qué query-only? ¿Qué trust boundary separaría un tool code_write?
3. Si graphrag está vacío o sin indexar, ¿qué ve el usuario del tool — panic, error claro, resultado vacío?

---

## D29 — L2/L3 injection system-prompt-only; L0/L1 exposed as tools

**Trade-off context.** Alternatives: inject memory into every history message
(destroys provider KV-cache economics), tools-only (weak persona grounding),
hybrid chosen by TDAM and ported verbatim.

**Evidence.**
- `vanta-proxy/src/inject.rs:1-5` — header states D29; injection only at the system-prompt position, never into conversation history; non-JSON bodies pass through untouched.
- `inject.rs:157` `merge_tools` merges L0/L1 vantage tools into the request `tools` array (TDAM README:28 rationale).

**Consequences assumed.**
- Memory updates mid-conversation don't appear until the next turn rebuilds the prompt.
- Persona/scenes consume system-prompt budget every turn.

**Socratic questions.**
1. Podés explicar mecánicamente por qué inyectar en el history invalida KV-cache y qué cuesta en latencia/costo por turno.
2. ¿Qué tipo de memoria pertenece a system prompt y cuál a tools? ¿Dónde trazás la línea entre L0/L1 y L2/L3?
3. Si el sistema prompt crece hasta comerse el budget del modelo, ¿qué degrada primero y cómo lo medirías?

---

## D30 — SSRF blocklist https-only NON-disablable (when remote fetch lands)

**Trade-off context.** TDAM ships its fetcher with `KNOWLEDGE_SSRF_CHECK=off`
env escape hatch. Research flagged: do NOT propagate that env-off. Decision:
when a remote fetcher is implemented, the blocklist must be hard-coded
non-disablable. Meanwhile the HTTPS/git fetcher stays OUT of P30 scope
(deferred until git sources exist) — see D36.

**Evidence.**
- `docs/research/tdam/08-knowledge-panel-sdk.md:45` — TDAM `PRIVATE_ADDR_RE` blocklist (10./172.16-31./192.168./169.254./127./0./localhost/::1/fe80:) disablable via `KNOWLEDGE_SSRF_CHECK=off` (git-fetcher.ts l.32-37).
- Same doc `:109` — explicit warning: "SSRF desactivable por env — no propagar desactivado."
- P30 plan Task 3 (MEM-29): fetcher deferred, D30 applies when implemented.

**Consequences assumed.**
- No SSRF surface exists today (nothing fetches remote URLs) — the decision costs nothing now but binds future work.
- When implemented: no operator override even in air-gapped/debug scenarios; legitimate private-repo fetching will need a designed exception.

**Socratic questions.**
1. ¿Por qué el env-off de TDAM era peligroso de propagar acá aunque sea cómodo para desarrollo?
2. Nombrá el ataque concreto que bloquea la regex de direcciones privadas en un contexto de fetcher de fuentes git.
3. Cuando existan fuentes legítimas en una red privada, ¿cómo debería pedirse esa capacidad si el kill-switch no existe?

---

## D31 — Proxy config in TOML

**Trade-off context.** Alternatives: env vars only (12-factor style), JSON/YAML,
TOML (chosen). Proxy needs structured nested config (upstream URL/apiKey, port,
rate-limits, features) edited by hand by a developer.

**Evidence.**
- `vanta-proxy/src/config.rs:1` — header names decision D31; serde TOML load.
- `config.rs:9` forward timeout default 600s (TDAM parity `config.ts:10` = 600_000 ms); `config.rs:17` `ProxyConfig { server, ... }`.

**Consequences assumed.**
- Secrets (upstream apiKey) sit in a plaintext file next to the binary — deployment story must handle that.
- Config schema changes are semver-visible for anyone hand-editing config.toml.

**Socratic questions.**
1. ¿Por qué TOML y no env vars para este crate, siendo que el resto del workspace usa config programática?
2. ¿Dónde termina el archivo y dónde empieza el entorno: qué valor aceptarías sobreescribir por env y cuál jamás?

---

## D32 — Ingest progress: internal channel + polling `wiki_status(run_id)`, no HTTP

**Trade-off context.** Alternatives: S2S HTTP callbacks with auth (TDAM pattern
rejected for us), pure polling without run_id (stale packets corrupt state),
internal trait/callback channel + pollable status keyed by run_id (chosen).
The desktop later bridges the channel to Tauri events; the CLI just polls.

**Evidence.**
- `vanta-memory/src/ingest/callback.rs:6-7` — stale `run_id` packets discarded (one run_id persisted per build, MEM-28 store).
- `callback.rs:20-21` `PROGRESS_THROTTLE_MS = 500` (TDAM parity `manager.ts:110`); phases extracting|merging|indexing with `{total, completed, failed, skipped, percent}`; `wiki_status(run_id)` consultable from another handle.

**Consequences assumed.**
- Progress is pull-based: consumers see up to throttle-interval-stale data.
- Channel is in-process: cross-process consumers need the future Tauri/CLI bridge.

**Socratic questions.**
1. ¿Qué corrupción exacta evita descartar paquetes con run_id viejo? Descríbela como secuencia de eventos.
2. ¿Por qué canal interno y no HTTP callback con auth? ¿Qué ganó y qué perdió el desktop?
3. ¿Por qué 500ms de throttle — qué mide ese número contra la experiencia de UI?

---

## D33 — mem-command: exactly TDAM's 3 commands, disabled by default

**Trade-off context.** Alternatives: richer command DSL, no mem-command at all,
parity port (chosen): `mem:sync | mem:create-skill | mem:help`, disabled by
default, enabled via config. Strict-args rule preserved: help/sync take NO
arguments — anything else passes through as ordinary conversation.

**Evidence.**
- `vanta-proxy/src/mem_command.rs:7-22` — `KNOWN_COMMANDS = ["sync", "create-skill", "help"]`; strict-args rule (TDAM parser.ts:37-41): `mem:help what is rust` is passed through verbatim.
- Disabled-by-default parity with TDAM `index.ts:24`; enabled responds sync/help (P30 Task 9 contract test e).

**Consequences assumed.**
- Feature invisible unless explicitly enabled — discoverability cost accepted.
- Command detection runs on every message: false positives depend on the strict prefix+args rule.

**Socratic questions.**
1. ¿Por qué `mem:help what is rust` debe atravesarse verbatim en vez de responder ayuda? ¿Qué regla del parser lo garantiza?
2. ¿Qué abuso potencia tener comandos de memoria dentro del chat y por qué alcanza el disabled-by-default?

---

## D35 — Rate-limit default 60 req/min per spaceId×model

**Trade-off context.** This parameterizes D24. Alternatives: unlimited (trusts
the caller), much lower defaults (breaks coding-agent bursts). 60/min matches
the TDAM sliding window bucket and leaves headroom for agentic loops while
still protecting upstream quota.

**Evidence.**
- `vanta-proxy/src/rate_limit.rs:1` — D35 named alongside D24; default 60 req/min, configurable via `ProxyConfig` (D31 TOML).
- P30 Task 9 contract test (a): excess blocked with 429 + Retry-After.

**Consequences assumed.**
- Agentic workflows doing parallel subagent calls can hit 60/min legitimately → visible 429s rather than silent throttling.
- Per-spaceId×model granularity means one noisy space doesn't starve others, but one model quota is shared across its spaces' callers.

**Socratic questions.**
1. ¿60 req/min protege a quién exactamente — tu cuota upstream o el backend del proveedor? ¿Cambia el número según la respuesta?
2. Un coding agent legítimo dispara ráfagas: ¿qué espera el cliente ver cuando se pasa — 429+Retry-After, y qué hace bien hecho el retry?

---

## D36 — Wiki sources v1 = local .md paths only (network deferred)

**Trade-off context.** Alternatives: ship HTTPS/git fetcher in P30 (pulls SSRF
surface into scope immediately), local-paths-only first (chosen). Network
fetch only makes sense once git sources actually exist; D30 governs it when
it lands.

**Evidence.**
- `src/wiki/sources.rs:3-10` — recursive `.md` discovery under a root with path-traversal guard: canonicalize + `starts_with(canonical_root)`, symlink escapes rejected, non-.md skipped with trace log.
- Chunker defaults 12000 chars / 400 overlap (`src/wiki/chunker.rs:13`; tests `chunker.rs:169-180`), SOURCE_CHAR_BUDGET 28000 (TDAM `ingest-v2/index.ts:78`).

**Consequences assumed.**
- v1 cannot ingest remote repos — adoption limited to local docs until the fetcher lands.
- Path traversal guard is a trust boundary: the canonicalize-prefix logic is security-critical code.

**Socratic questions.**
1. ¿Por qué canonicalize + starts_with y no comparación lexicográfica de paths? ¿Qué escape específico mata cada uno?
2. ¿Qué ataque intenta un symlink apuntando fuera de la raíz y en qué línea del guard muere?
3. ¿Qué tendría que ser cierto en el mundo para que valga la pena construir el fetcher HTTPS — y qué decisión vieja se activaría?

---

## Cross-cutting frame — D37 (accepted risks, confirmed by user 2026-08-21)

Three risks were consciously accepted with documented upgrade paths; you do
not need to re-decide them, but your ADR text should acknowledge they exist:

| Risk | Origin | Upgrade path |
|---|---|---|
| Rate-limit loses state multi-instance | D24 | Redis/shared limiter if ever multi-instance |
| chars/3 underestimates CJK | D21 | per-script estimator / tiktoken opt-in |
| keyword-overlap recall without vectors | D38 (recall) | vector index wiring (done post-P31: D38/D39) |

---

## Author checklist (what YOU do next)

1. **Read this guide end-to-end.** For any question you cannot answer out loud,
   that topic is your study queue before signing — Regla 10 (explainability gate).
2. **Edit `ADR-029-vanta-memory-context-engine.md` in FIRST PERSON, your words.**
   Fill the `Decision` and `Consequences` sections for D21-D23; keep the
   technical evidence tables, rewrite the framing. Do not paste this guide's
   prose — paraphrasing is the point.
3. **Decide where D24-D37 live:** either extend ADR-029 or create a companion
   ADR (e.g., `ADR-0XX-proxy-knowledge.md`) covering the proxy/knowledge
   decisions (D24-D36) using the evidence above. The plan's contract says step
   3 (AI transcription into the new ADR) happens AFTER you articulate.
4. **Remove both draft banner blockquotes** from ADR-029 once articulated, set
   `status: accepted`, update `last_reviewed`.
5. **Sign it**: add your name/date under the title.
6. **Commit yourself** (your commit is the gate): e.g.
   `docs(adr): ADR-029 articulation + D24-D37 decisions (author-signed)`.
   Per plan Notas: the lead commits only after your articulation exists.
7. If time is not available now: leave everything as-is (draft banners intact);
   the honest state is "open" — do NOT mark accepted to close the loop.
