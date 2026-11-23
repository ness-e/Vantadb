---
title: "S5 — Deep Audit: MCP Server + HTTP Server + User-Facing Skills"
kind: review
status: active
description: "Informe crudo del subagente S5: servidor MCP, servidor HTTP y las skills de usuario (`skills/vantadb-mcp/**`, `skills/vantadb/**`). Base de los hallazgos `S5-*` del ledger (conteos de tools, default de perfil, smoke test)."
tags: [audit, reviews, mcp, server, skills, subagent]
---

# Informe del subagente — Deep Audit: MCP Server + HTTP Server + User-Facing Skills

- **Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
- **Mode:** READ-ONLY (no file in the repo was created/edited/deleted; no git write commands; no persistent servers)
- **Date:** 2026-10-20
- **Target version under audit:** workspace `0.9.0` (`Cargo.toml:805`)
- **Scope:** `vantadb-mcp/`, `vantadb-server/`, `src/server/`, `server.json`, `docs/api/MCP.md`, `docs/api/HTTP_API.md`, `docs/api/openapi.yaml`, `.opencode/skills/vantadb/`, `.opencode/skills/vantadb-mcp/`

## Ground truth established (read from code, not from docs)

**Actual MCP tool surface — 88 defined / 82 listed.**

| Source | Count | Tools |
|---|---|---|
| `vantadb-mcp/src/handlers/tools.rs:83-1053` (`base_tools`) | 49 | `memory_put`, `memory_put_batch`, `memory_get`, `memory_delete`, `memory_verify_certificate`, `memory_delete_by_filter`, `memory_list`, `memory_list_namespaces`, `memory_versions`, `memory_supersede`, `memory_reinforce`, `memory_search`, `search_semantic`, `search_with_method`, `search_multi`, `memory_recall`, `embed_texts`, `query_iql`, `get_node_neighbors`, `graph_page_rank`, `graph_degree_centrality`, `graph_traverse`, `graph_topological_sort`, `graph_is_dag`, `remove_edge`, `inject_context`, `read_axioms`, `write_axiom`, `delete_axiom`, `collection_stats`, `collection_delete`, `flush`, `compact_wal`, `compact_layout`, `rebuild_index`, `audit_text_index`, `repair_text_index`, `snapshot_create`, `snapshot_restore`, `list_snapshots`, `purge_expired`, `rehydrate`, `vacuum`, `capabilities`, `generate_snippet`, `export`, `import`, `bulk_import_file`, `bulk_import_stream` |
| `skills.rs` | 7 | `skill_list`, `skill_view`, `skill_create`, `skill_update`, `skill_patch`, `skill_files_write`, `skill_extract` |
| `code.rs` | 8 | `code_search`, `code_explore`, `code_callers`, `code_callees`, `code_impact`, `code_node`, `code_status`, `code_files` |
| `code_index.rs` | 1 | `code_index` |
| `wiki.rs` | 6 | `wiki_search`, `wiki_read`, `wiki_list`, `wiki_graph`, `wiki_ingest`, `wiki_ingest_status` |
| `context.rs` | 1 | `context_assemble` |
| `dreams.rs` | 5 | `dream_list`, `dream_load`, `dream_discard`, `dream_consolidate`, `dream_promote` |
| `scenes.rs` | 5 | `scene_read`, `scene_list`, `scene_query`, `scene_write`, `scene_edit` |
| `threads.rs` | 6 | `thread_create`, `thread_send`, `thread_get`, `thread_list`, `thread_delete`, `thread_purge_expired` |

6 of the 8 `code_*` names are **absorbed dispatch-only** (never in `tools/list`): `code_callers`, `code_callees`, `code_impact`, `code_node`, `code_status`, `code_files` → canonical `code_explore` / `capabilities` / `code_search` (`handlers/tools.rs:1315-1331`). Plus 2 API-04 aliases: `search_memory` → `memory_search`, `collection_list` → `memory_list_namespaces`.

**Profiles** (`vantadb-mcp/src/config.rs:10-57`, `handlers/tools.rs:1097-1308`): `agent` = 39 (**default**), `full` = 82, `dev` = 38, `memory` = 22. Asserted by `vantadb-mcp/tests/mcp_tests.rs:4961-4995`.

**Actual HTTP route surface — 44 path+method pairs** (`src/server/router.rs:173-285` + `mount_dashboard:346-383`): 1 public (`GET /health`), 38 protected, 3 long-running, 2 dashboard. `node scripts/check_openapi_parity.mjs` → **Parity OK** (37 paths / 44 operations, exit 0).

---

## Findings

### A1 | P2 | docs/api/MCP.md — self-contradictory extended-family counts
- **Location:** `docs/api/MCP.md:408` vs `docs/api/MCP.md:410` vs `docs/api/MCP.md:200-209`
- **Claim/Break:** The heading reads `## Extended Tool Families (38 defined; 32 listed)`. The body one line below reads `defined outside handlers/tools.rs (9+7+6+5+6+1+5 = 39 defined; WIRE-02 absorbs 6 code_* → 33 listed)`. The §Tool Families table (lines 200-209) sums to 49 + 3 + 7 + 6 + 1 + 5 + 6 + 5 = **82 listed** and 49 + 9 + 7 + 6 + 1 + 5 + 6 + 5 = **88 defined**, i.e. extended = 39 defined / 33 listed. The heading is wrong by 1 on both numbers and contradicts its own body.
- **Why it matters:** MCP.md is the canonical repo-side reference for the MCP surface. A reader (or a downstream agent) computing "core 49 + extended 38 = 87" gets a count that matches no profile and no code path.
- **Trigger:** Any reader reconciling the family table against the extended-family section; any generator that parses the heading.
- **Fix:** Change the heading to `## Extended Tool Families (39 defined; 33 listed)`.
- **Verify:** `Select-String -Path docs/api/MCP.md -Pattern 'Extended Tool Families'` → heading matches the body sum.

### A2 | P2 | docs/api/MCP.md — the 6 `thread_*` tools are counted but never documented
- **Location:** `docs/api/MCP.md:208` (table row `| thread_* | 6 | threads.rs |`); no corresponding section between `docs/api/MCP.md:469` (`### Dreams API`) and `docs/api/MCP.md:481` (`## Prompts`)
- **Claim/Break:** `docs/api/MCP.md` has sections for Core, Code Intelligence, Skills, Wiki, Context Engine, Scenes and Dreams — but **no Threads section**. The 6 tools `thread_create`, `thread_send`, `thread_get`, `thread_list`, `thread_delete`, `thread_purge_expired` are counted in the family table and in the `agent`/`full` profile descriptions (lines 237-238) but are never named, no params, no return shapes, no error codes anywhere in the file. Confirmed: `Select-String -Path docs/api/MCP.md -Pattern 'thread'` returns only the family-table row and prose mentions.
- **Why it matters:** `thread_send` is the *only* documented capture path in the recall policy ("Capture durable turns with `thread_send`", `vantadb-mcp/src/handlers/initialize.rs:21`) and `thread_*` is 6 of the 39 tools in the **default** `agent` profile. A user following only MCP.md cannot call them or know their params.
- **Trigger:** An agent reads MCP.md (the repo canonical) and tries to capture a turn → no `thread_send` contract.
- **Fix:** Add `### Threads API — threads.rs (6)` between the Dreams and Prompts sections, mirroring `skills/vantadb-mcp/references/api-reference.md:153-165` (which does document them).
- **Verify:** `Select-String -Path docs/api/MCP.md -Pattern 'thread_send'` → hits the new table.

### A3 | P2 | docs/api/MCP.md + server.json — registry version claim is stale
- **Location:** `docs/api/MCP.md:676` — `| version | tracks [workspace.package].version (0.5.0 as of this doc) |`
- **Claim/Break:** `server.json:6` declares `"version": "0.9.0"` and `Cargo.toml:805` declares `version = "0.9.0"`. MCP.md itself declares `Current MCP implementation version: 0.9.0` at line 11, and `docs/api/SURFACE_MANIFEST.md` shows `0.9.0` for every surface. The registry table is 4 minor versions behind.
- **Why it matters:** The registry manifest table is what a publisher copies when submitting `server.json`; a `0.5.0` claim there is a release-compliance defect and contradicts the same page's line 11.
- **Trigger:** MCP registry submission / any release checklist reading §Registry manifest.
- **Fix:** Replace `0.5.0` with `0.9.0` (or make it non-numeric: "tracks `[workspace.package].version`").
- **Verify:** `Select-String -Path docs/api/MCP.md -Pattern '0\.5\.0'` → no hits.

### A4 | P3 | docs/api/MCP.md — CLI version floor is below the contract it describes
- **Location:** `docs/api/MCP.md:21` — ``The VantaDB CLI (`vanta-cli` ≥ **0.5.0**) installed and on PATH``
- **Claim/Break:** The page then documents an 82-tool surface, the `agent` default profile (line 237) and the WIRE-02 absorbed names (lines 216-227) — none of which exist in 0.5.0 (0.5.0 predates API-04 and WIRE-02; its `tools/list` returned 85 names including `search_memory`/`collection_list` and a `full` default).
- **Why it matters:** A user satisfying the stated floor (0.5.0) gets a materially different tool surface than the page describes, and the WIRE-02 "not in tools/list" guidance becomes wrong.
- **Trigger:** Installing the minimum stated version and configuring the MCP client against this page.
- **Fix:** Raise the floor to `0.9.0` (or `0.8.0`, the version where WIRE-02 landed) and note the profile default change.
- **Verify:** Compare the floor against `vantadb-mcp/src/config.rs:10` (profile counts) — the floor must be ≥ the version that introduced the `agent` default.

### A5 | P2 | docs/api/MCP.md — the "every error carries `data.code`" claim is false for the 5 standard JSON-RPC factories
- **Location:** `docs/api/MCP.md:105-108` — *"Every error response carries both a **JSON-RPC transport code** (the `error.code` field) and a **Vanta canonical code** (in `error.data.code`) from the 10-element contract"*
- **Claim/Break:** `vantadb-mcp/src/error.rs:21-27` — `parse_error`, `invalid_request`, `method_not_found`, `invalid_params`, `internal_error` all use `plain()`, which sets `data: Value::Null`. `to_json()` (`error.rs:123-129`) then emits `{"code":…,"message":…}` with **no `data` key at all**. Only `validation()`, `not_found()`, `resource_limit()` and `from_domain()` attach `data`. In practice a malformed JSON line produces `{"code":-32700,"message":"Parse error: …"}` with no `data.code` and no `data.retriable`.
- **Why it matters:** The whole point of the ERR-MCP-01 envelope is that an LLM client can branch on a stable `data.code`/`data.retriable` instead of parsing message text. A client written against the doc will KeyError on `error.data.code` for exactly the transport-level errors the doc says always carry it.
- **Trigger:** Any client that does `err["error"]["data"]["code"]` on a parse error / method-not-found / invalid-params response.
- **Fix:** Either (a) amend the doc to say "the 5 standard factories omit `data`; only the `-320xx` domain codes and `validation`/`not_found`/`resource_limit` carry it", or (b) make the 5 factories emit `data: {code, retriable:false}`. (a) is the smaller change and matches the code.
- **Verify:** `node -e` over the 5 factories in `error.rs`, or a unit test asserting `to_json()` has no `data` for `invalid_params`.

### A6 | P2 | docs/api/openapi.yaml — 9 response schemas do not match the wire shape
- **Location:** `docs/api/openapi.yaml:283` (POST `/api/v2/records`), `:320` (DELETE `/api/v2/records`), `:355` (POST `/api/v2/records/batch`), `:385` (GET `/api/v2/records/{ns}/{key}`), `:413` (DELETE `/api/v2/records/{ns}/{key}`), `:670` (POST `/api/v2/export`), `:703` (POST `/api/v2/import`), `:1304` (DELETE `/api/v2/threads/{id}`), `:1708` (POST `/api/v2/snapshots/{name}`)
- **Claim/Break:** 8 of these declare `components/schemas/AckResponse` = `{success: boolean, error?: string|null, data?: any}` (openapi.yaml:2044-2057) and one declares `RecordEnvelope` = `{success, error?, data}` (openapi.yaml:2059-2072). The actual handlers return **raw** objects with no `success` wrapper:
  - `src/server/handlers.rs:254` → `Json(record)` (the record itself)
  - `src/server/handlers.rs:265` → `Json(records)` (a JSON **array**)
  - `src/server/handlers.rs:277` → `Json(record)`
  - `src/server/handlers.rs:326` → `{"deleted": true}`
  - `src/server/handlers.rs:361` → `{"deleted": N}`
  - `src/server/handlers.rs:806` → `Json(report)` (export report)
  - `src/server/handlers.rs:841` → `Json(report)` (import report)
  - `src/server/handlers.rs:1379` → `{"deleted": true}`
  - `src/server/handlers.rs:1655-1662` → `{"name": …, "path": …}`
  `docs/api/HTTP_API.md:141` documents the real shape (`{"namespace":"agent/main","key":"note-1",…,"version":1,"node_id":"…"}`), so HTTP_API.md is correct and openapi.yaml is wrong.
- **Why it matters:** `openapi.yaml` is declared the authority for the HTTP surface (`docs/api/SURFACE_MANIFEST.md` §Authority map: *"HTTP / OpenAPI surface → `docs/api/openapi.yaml`"*). Any code generator, client SDK, contract test or `curl`-copy-paste derived from it will read `resp.success` / `resp.data` and get `undefined`. `scripts/check_openapi_parity.mjs` only checks **path+method**, never response schemas, so CI is green.
- **Trigger:** Generating a client from `openapi.yaml`; asserting `body["success"] == true` on `POST /api/v2/records`.
- **Fix:** Replace the 8 `AckResponse` refs and the 1 `RecordEnvelope` ref with the real shapes (add a `MemoryRecord` schema and use `type: array, items: $ref: MemoryRecord` for the batch). Or change the handlers to emit an ack envelope — but that is a breaking HTTP change and HTTP_API.md would have to change too.
- **Verify:** `node scripts/check_openapi_parity.mjs` (paths only) **plus** a new schema assertion; or replay the HTTP_API.md quickstart transcript (lines 133-168) and check the body against the openapi schema.

### A7 | P2 | docs/api/openapi.yaml + docs/api/HTTP_API.md — `/dashboard` is documented as authenticated but is mounted outside the auth middleware
- **Location:** `docs/api/openapi.yaml:107-108` and `:128-129` (`security: [bearerAuth: []]` on `/dashboard` and `/dashboard/{path}`); `docs/api/HTTP_API.md:893-894` (`| GET | /dashboard | Bearer (if configured) |`)
- **Claim/Break:** `src/server/router.rs:173` creates `public = Router::new().route("/health", …)` and the auth middleware is layered **only** on the `protected` sub-router (`router.rs:265`, `:281`). `mount_dashboard` (`router.rs:346-383`) then `nest_service`s `/dashboard` on the **outer** router. The code comment at `router.rs:343-345` says so explicitly: *"it stays **outside** the auth middleware, so `/dashboard` is public on loopback (D12) even when `require_auth` guards `/api/v2/*`."* The openapi's `bearerAuth` scheme description (openapi.yaml:1769-1772) also says *"all requests to `/api/*` endpoints must include this header"* — `/dashboard` is not under `/api/`, so even by the doc's own wording it is not covered.
- **Why it matters:** Documented auth posture ≠ enforced auth posture, on a route that serves the Vanta Studio console (HTML + static assets). An operator reading the openapi to decide "do I need a token for the dashboard?" gets the wrong answer.
- **Trigger:** Operator deploys with `VANTADB_API_KEY` set and assumes `/dashboard` is token-gated; a browser hit (no Authorization header) reaches the console.
- **Fix:** Either move `mount_dashboard` inside the protected router, or change the openapi (`security: []`) + HTTP_API.md Auth column to "No (public on loopback)" and state the D12 rationale.
- **Verify:** `curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:8080/dashboard` with `VANTADB_API_KEY` set → 404 (dashboard disabled) or 200, and **not** 401.

### A8 | P2 | vantadb-mcp — the `/api/v2/mcp/proxy` HTTP route is undocumented and outside the parity gate
- **Location:** `vantadb-mcp/src/server.rs:246` — `.route("/api/v2/mcp/proxy", axum::routing::post(mcp_proxy_handler))`; handler at `vantadb-mcp/src/server.rs:113-168`
- **Claim/Break:** This is a real, live HTTP endpoint on the MCP-35 writer's ephemeral loopback listener (it forwards any MCP method to the writer's storage; `proxy.rs:90-130` is the client). It appears in **none** of: `docs/api/openapi.yaml`, `docs/api/HTTP_API.md` (Route Summary, lines 850-895), or the `opencode.jsonc`/skill docs. `scripts/check_openapi_parity.mjs:16` hardcodes `RS_FILE = src/server/router.rs`, so the gate only ever scans that one file and can never see this route.
- **Why it matters:** An endpoint that accepts `Authorization: Bearer` and executes arbitrary MCP methods against the live database, reachable on loopback, is invisible to the API contract, to the parity gate, and to the multi-tenant/RBAC analysis in `docs/api/HTTP_API.md` §Multi-tenant. It is the one place where the HTTP surface can execute a *write* tool without the `/api/v2/*` RBAC path.
- **Trigger:** An operator enumerating the attack surface from the openapi misses it; a future refactor moves it without a doc change.
- **Fix:** Document it in `docs/api/HTTP_API.md` (loopback-only, Bearer if `VANTADB_API_KEY`, not RBAC-scoped) and either add it to `openapi.yaml` with `x-internal: true` or extend the parity script to also scan `vantadb-mcp/src/server.rs`.
- **Verify:** `Select-String -Path docs/api/HTTP_API.md,docs/api/openapi.yaml -Pattern 'mcp/proxy'` → 1 hit each.

### A9 | P3 | docs/api/openapi.yaml — the `x-vanta-user-key` header is not declared
- **Location:** `docs/api/openapi.yaml:1764-1772` (`securitySchemes` declares only `bearerAuth`); `src/server/middleware.rs:166-176` (L3 identity resolution requires the header and 401s with `"hint": "Provide a valid x-vanta-user-key header."`)
- **Claim/Break:** The middleware resolves a *third* identity tier (L3 service/user) via the `x-vanta-user-key` header and returns a 401 whose remediation hint names that header — but no security scheme, no header parameter, and no mention in `docs/api/HTTP_API.md` §Authentication (lines 28-36) or §Multi-tenant (lines 38-91) documents it.
- **Why it matters:** A client that receives the 401 hint cannot discover the header name from the contract; the multi-tenant section describes only Bearer token→role mapping and never the L2/L3 identity path.
- **Trigger:** A service-to-service client hitting the 401 hint and grepping the openapi for `x-vanta-user-key` → no hit.
- **Fix:** Add an `apiKeyHeader` security scheme (`in: header, name: x-vanta-user-key`) and document the L1/L2/L3 identity tiers in HTTP_API.md §Authentication.
- **Verify:** `Select-String -Path docs/api/openapi.yaml -Pattern 'x-vanta-user-key'` → 1 hit.

### A10 | P3 | docs/api/MCP.md — the "mechanically enforced" coverage claim is narrower than stated
- **Location:** `docs/api/MCP.md:665` — *"Tool coverage on this page is enforced mechanically by `scripts/validate-docs-coverage.ps1` against `handle_tools_list()` in `vantadb-mcp/src/handlers/tools.rs`."*
- **Claim/Break:** `scripts/validate-docs-coverage.ps1:176-181` slices the `handle_tools_list` function body out of `handlers/tools.rs` and extracts `"name": "X"` from **that block only** — i.e. the 49 base tools. The 39 extended-family tools are contributed by `crate::skills`, `crate::code`, `crate::code_index`, `crate::wiki`, `crate::context`, `crate::dreams`, `crate::scenes`, `crate::threads` (`handlers/tools.rs:1072-1079`) and are **never** checked. Running the script prints `vantadb-mcp (tools) - 49 items ok en MCP.md`.
- **Why it matters:** The gap is exactly where A2 lives: MCP.md can drop the entire `thread_*` family and the gate stays green. The doc's parity claim is stronger than the mechanism.
- **Trigger:** A future tool is added to `skills.rs`/`wiki.rs`/… — nothing fails, and the doc silently drifts.
- **Fix:** Extend the script to also parse the `*_tool_definitions()` functions in the eight extended modules (or add a Rust test that asserts every defined tool name appears in `docs/api/MCP.md`).
- **Verify:** `pwsh -File scripts/validate-docs-coverage.ps1` should report `vantadb-mcp (tools) - 88 items ok` (not 49).

### A11 | P3 | server.json — schema-valid but not installable, and pinned to a moving branch
- **Location:** `server.json:1-22`
- **Claim/Break:** The file is schema-valid against `https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json` (required: `name`, `description`, `version` — all present; `packages`/`remotes` are optional), `version: "0.9.0"` matches `Cargo.toml:805`, and `name: "io.github.ness-e/vantadb"` matches the reverse-DNS pattern. But it declares **no `packages[]` and no `remotes[]`**, so a registry client has no `registryType`/`identifier`/`transport`/`packageArguments`/`environmentVariables` to install from — there is no `command`/`env` to check against the built artifact, and `vantadb-server` (the only MCP binary, `vantadb-server/Cargo.toml:9-11`) is `publish = false` and not on crates.io. Separately, `websiteUrl` (server.json:7) points at `blob/develop/docs/api/MCP.md` — a moving branch that 404s for anyone following a tagged release.
- **Why it matters:** The descriptor is a "discoverable descriptor, not an installable one" (MCP.md:684-686 acknowledges this) — but the file itself gives a consumer no signal, and the branch-pinned URL rots.
- **Trigger:** A user finds the registry entry, tries to install, gets no `packages[]`.
- **Fix:** Either add a `packages[]` entry with `registryType: "cargo"`-style install guidance (or the GitHub release binary) plus `environmentVariables` for `VANTADB_STORAGE_PATH`/`VANTADB_MCP_PROFILE`, or add a `_meta` note saying "descriptor only — install via `cargo install --git` / release binaries". Point `websiteUrl` at `blob/main`.
- **Verify:** Validate `server.json` against the published schema; assert `name`/`version` against `Cargo.toml:805`.

### A12 | P3 | opencode.jsonc — the repo's own MCP entry claims a non-existent tool count and profile
- **Location:** `opencode.jsonc:69` — `"description": "VantaDB MCP vía launcher (EMB-12: …) Memoria persistente + vector retrieval (85 tools perfil full; API-04: nombres canónicos memory_search/memory_list_namespaces)."`
- **Claim/Break:** Two errors. (1) `85` is not a real count — `full` lists 82, and 85 was the pre-WIRE-02 figure. (2) "perfil full" is false: the launcher (`vanta-mcp-local.ps1:109-129`) never sets `VANTADB_MCP_PROFILE`, and the server default is `agent` (39 tools) since WIRE-02 (`vantadb-mcp/src/config.rs:39-45`, `handlers/tools.rs:1130-1133`).
- **Why it matters:** This is the config the project itself runs; the description is what an operator reads to reason about the exposed surface.
- **Trigger:** Reading `opencode.jsonc` to determine the live tool surface, or to diagnose "why doesn't the agent see `wiki_ingest`".
- **Fix:** Replace with "39 tools (perfil agent por defecto) — `VANTADB_MCP_PROFILE=full` expone 82".
- **Verify:** `Select-String -Path opencode.jsonc -Pattern '85 tools'` → no hit.

### A13 | P3 | vantadb-mcp — a panic outside `spawn_blocking` silently drops the response (no server kill, but a client hang)
- **Location:** `vantadb-mcp/src/server.rs:499-526` (the `inflight.spawn` task) and `vantadb-mcp/src/server.rs:571-635` (`dispatch_request`)
- **Claim/Break:** `tools/call` and `resources/read` run inside `tokio::task::spawn_blocking`, so a handler panic is caught and mapped to `internal_error("Task panicked: …")` (`server.rs:603`, `:627`) — **the stdio server does not die** (answer to the audit question). But `initialize`, `tools/list`, `resources/list`, `prompts/list`, `prompts/get` (`server.rs:572-573`, `:605`, `:629-630`) run **inline inside the spawned task**. A panic there aborts the task, the response is never written, and the client waits forever for that JSON-RPC id — a silent hang rather than an error.
- **Why it matters:** The failure mode is invisible: no error, no log-to-client, the client just stalls until its own timeout. Today the risk is low (these handlers are pure JSON construction), but the invariant is not enforced and the next non-trivial `tools/list` body (e.g. dynamic schema generation) would inherit it.
- **Trigger:** Any future panic in `handle_tools_list` / `handle_prompts_list` / `handle_resources_list`.
- **Fix:** Wrap the whole `dispatch_request` body in `spawn_blocking` (or add `std::panic::catch_unwind` around the inline branches and map to `internal_error`), so every dispatch path has the same panic→error contract.
- **Verify:** A test that forces a panic in `handle_tools_list` and asserts the client receives a JSON-RPC error rather than a hang.

---

## Part B — User-facing skills

> Scope note: `.opencode/skills/` holds 197 dirs. Only two are *end-user* skills for the VantaDB **product**: `vantadb` (Python/Rust SDK guide) and `vantadb-mcp` (MCP server guide). The other matches for `vanta-cli`/`memory_put` (`progreso`, `review-deep`, `source-driven-development`, `test-driven-development`) are internal dev-workflow skills, not shipped to VantaDB users. The git-tracked canonical copies live in `skills/vantadb/` and `skills/vantadb-mcp/` (byte-identical `SKILL.md` hashes); findings below are quoted from the `.opencode/` copies, which are the ones the agent actually loads.

### B1 | P1 | .opencode/skills/vantadb-mcp — the headline tool count is wrong (85 vs 82 listed / 88 defined)
- **Location:** `.opencode/skills/vantadb-mcp/SKILL.md:8` — *"The MCP server exposes **85 tools** (47 core + 7 `skill_*` + 8 `code_*` + 6 `wiki_*` + 1 `context_assemble` + 5 `scene_*` + 6 `thread_*` + 5 `dream_*`)"*; `.opencode/skills/vantadb-mcp/SKILL.md:124` — `## Available MCP Tools (85)`; `.opencode/skills/vantadb-mcp/SKILL.md:127` — *"The sections below document the 47 core tools in detail; the other 38 are summarized here."*
- **Claim/Break:** Real: **82 listed** / **88 defined**, of which **49 are core** (`handlers/tools.rs:83-1053` = 49 base; the 39 extended come from the 8 modules). The `code_*` family is **9 defined** (8 in `code.rs` + `code_index` in `code_index.rs`), not 8. Three separate numbers are wrong (85/47/38) and the `code_*` count is wrong by one.
- **Why it matters:** This is the *first* sentence of the skill and the number is repeated in the section heading the agent is told is the "single source of truth" (`SKILL.md:126-127` → `references/api-reference.md`). An agent budgeting tool-schema tokens, or a user checking "did the surface change", is working from a figure that matches no code path and no profile.
- **Trigger:** The agent reads the skill to plan which tools exist; or a user compares `tools/list` output to the documented count and concludes the server is broken.
- **Fix:** `85` → `82 listed (88 defined)`, `47 core` → `49 core`, `8 code_*` → `9 code_* (3 listed: code_search/code_explore/code_index)`, `other 38` → `other 33 listed / 39 defined`.
- **Verify:** `Select-String -Path .opencode/skills/vantadb-mcp/SKILL.md -Pattern '85 tools|47 core'` → no hits; cross-check against `vantadb-mcp/tests/mcp_tests.rs:4961` (`assert_eq!(full_tools.len(), 82)`).

### B2 | P1 | .opencode/skills/vantadb-mcp — presents absorbed dispatch-only tools as available and omits a listed one
- **Location:** `.opencode/skills/vantadb-mcp/SKILL.md:133` — `| Code Intelligence (code_*) | 8 | code_search, code_explore, code_callers, code_callees, code_impact, code_node, code_status, code_files* |`; `.opencode/skills/vantadb-mcp/references/api-reference.md:77-91` (the same 8, as a "Code Intelligence" table)
- **Claim/Break:** Since WIRE-02, `code_callers`, `code_callees`, `code_impact`, `code_node`, `code_status` and `code_files` are **absorbed dispatch-only** — they are never returned by `tools/list` in any profile (`vantadb-mcp/src/handlers/tools.rs:1315-1331`, `:1190-1239`); they resolve to `code_explore`/`capabilities`/`code_search`. The skill lists them as peers of `code_search` with no dispatch-only marking, and **omits `code_index`** — a real, LISTED tool in the `full` profile (`code_index.rs`, wired at `handlers/tools.rs:1074`).
- **Why it matters:** An agent that reads the skill tries `code_impact`/`code_node` from `tools/list` and finds nothing, then wastes turns on names that will never appear; and it never learns `code_index` exists, so it never indexes a codebase into the graph even though the tool is available.
- **Trigger:** Agent enumerates `tools/list`, diffs against the skill table, and concludes 6 tools are missing/broken.
- **Fix:** Split the row into "3 listed (`code_search`, `code_explore`, `code_index`)" and "6 dispatch-only absorbed (`code_callers`…`code_files`) — call via their canonical tool"; add `code_index` to the listed set; mirror `docs/api/MCP.md:216-227` (which documents the absorbed set correctly).
- **Verify:** `Select-String -Path .opencode/skills/vantadb-mcp/SKILL.md -Pattern 'code_index'` → 1 hit; `code_callers` appears only inside a "dispatch-only" note.

### B3 | P1 | .opencode/skills/vantadb-mcp — the shipped MCP smoke test can never pass
- **Location:** `.opencode/skills/vantadb-mcp/scripts/test-mcp.py:58-62` — `EXPECTED_TOOLS = {"full": (85, 85), "dev": (36, 36), "memory": (20, 20)}`; docstring `.opencode/skills/vantadb-mcp/scripts/test-mcp.py:21-26`; default profile selection at `.opencode/skills/vantadb-mcp/scripts/test-mcp.py:165`
- **Claim/Break:** Three compounding defects:
  1. The expected counts are stale: real is `full` 82, `dev` 38, `memory` 22, `agent` 39 (`vantadb-mcp/src/config.rs:10-34`; asserted at `vantadb-mcp/tests/mcp_tests.rs:4961/4976/4993`). The script cites that very test as its source of truth (line 25, 51-52) — the test says 82/39/38/22, the script says 85/36/20.
  2. `profile = os.environ.get("VANTADB_MCP_PROFILE", "full")` (line 165) decides the *expected* bucket, but the script never exports `VANTADB_MCP_PROFILE` to the child process (line 112-117: `env=None` for `vanta-cli`, `dict(os.environ, VANTADB_STORAGE_PATH=…)` for `vantadb-server`). The server therefore uses its **default `agent` profile → 39 tools**, and the script compares 39 against the `full` bucket `[85, 85]` → drift → `continue` → `passed` never reaches 5 → `return 1` (line 286).
  3. Even with `VANTADB_MCP_PROFILE=full` exported by the user, the server returns **82** and the script still expects `[85, 85]` → drift → exit 1. The `agent` profile (the default) is not even a key in `EXPECTED_TOOLS`, so `VANTADB_MCP_PROFILE=agent` makes the script bail at line 166-171.
  Net: `python scripts/test-mcp.py` — the skill's documented "Test the MCP server" step (`SKILL.md:83-87`) — **always exits 1** against a current build.
- **Why it matters:** This is the only executable verification the skill gives a user. It fails on a healthy server, so the user's only diagnostic step reports a false "stale binary? rebuild" (the script's own message, line 227-229) and sends them to rebuild in a loop. It also cannot detect the *real* drift the gate exists for.
- **Trigger:** Any user running the skill's testing step.
- **Fix:** Replace the hardcoded map with `{"agent": (39, 39), "full": (82, 82), "dev": (38, 38), "memory": (22, 22)}`; export `VANTADB_MCP_PROFILE=profile` into the child env (both branches); default the expected bucket to `"agent"` to match the server default. Better: import the counts from the Rust test rather than duplicating them.
- **Verify:** `VANTADB_MCP_BIN=target/debug/vantadb-server.exe python .opencode/skills/vantadb-mcp/scripts/test-mcp.py` → exit 0, "5/5 passed".
- **Note:** `scripts/validate-docs-coverage.ps1:194` explicitly excludes `test-mcp.py` from the skills-mirror hash check ("owned by FIND-82"), which is why the drift went unnoticed.

### B4 | P1 | .opencode/skills/vantadb-mcp — configuration.md documents the wrong default profile and the wrong tool counts
- **Location:** `.opencode/skills/vantadb-mcp/references/configuration.md:125-131` — *"### VANTADB_MCP_PROFILE … Accepted values: `full` (default, 85 tools), `dev` (36: memory + graph + collections + key maintenance + axioms), `memory` (20: core CRUD + search + IQL + collections + capabilities)."*
- **Claim/Break:** Real (`vantadb-mcp/src/config.rs:10`, `:22`, `:27`, `:30`, `:33`): default is **`agent` (39 tools)**, `full` = 82, `dev` = 38, `memory` = 22. The `agent` profile — the one every user gets unless they opt in — is **not mentioned at all** in the skill. The skill also never mentions that `tools/call` for a known out-of-profile tool returns `-32601` "Tool not found: `<name>` (not in profile `<profile>`)" (`handlers/tools.rs:1368-1374`) — the single most confusing error a new user hits.
- **Why it matters:** A user sets `VANTADB_MCP_PROFILE=dev` per the skill and still sees 38 tools, not 36; a user reads "default = full" and expects `wiki_ingest`/`skill_*`/`code_index` to be present, but the default profile excludes them → "tool not found" on tools the skill says exist. The skill never explains the profile gate at all.
- **Trigger:** Any user who sets a profile, or who wonders why `wiki_ingest` is missing.
- **Fix:** Rewrite the section with the real table (agent 39 default / full 82 / dev 38 / memory 22), add the `agent` row and its membership, and document the `-32601 … (not in profile …)` behaviour.
- **Verify:** Cross-check every number against `vantadb-mcp/src/config.rs:10-34` and `handlers/tools.rs:1097-1308`.

### B5 | P2 | .opencode/skills/vantadb-mcp — configuration.md names a non-existent env var for the CLI `--db` flag
- **Location:** `.opencode/skills/vantadb-mcp/references/configuration.md:17` — *"The CLI's `--db` flag (env `VANTA_DB`, default `./db`) is separate and only affects `vanta-cli`."*; repeated at `.opencode/skills/vantadb-mcp/references/configuration.md:195` — *"The `vanta-cli` binary accepts the global `--db` flag (env `VANTA_DB`, default `./db`)"*
- **Claim/Break:** `src/cli.rs:15-22` declares `#[arg(short, long, env = "VANTADB_STORAGE_PATH", default_value = "./db", global = true)] pub db: String`. The env var for `--db` is **`VANTADB_STORAGE_PATH`**, not `VANTA_DB`. `VANTA_DB` is read by nothing in the repo. The same section correctly documents `VANTADB_STORAGE_PATH` at line 7-15, so the page contradicts itself within 3 lines.
- **Why it matters:** A user who exports `VANTA_DB=/data/vanta` expecting `--db` to pick it up gets the default `./db` — a database created in the CWD instead of the intended directory. The skill even warns at line 19 that `~` is not expanded, so the user is clearly path-sensitive.
- **Trigger:** `VANTA_DB=/data/vanta vanta-cli server --mcp` → server opens `./db`.
- **Fix:** Replace `VANTA_DB` with `VANTADB_STORAGE_PATH` at lines 17 and 195 (and note that `--db` and the env var are the same knob, not separate).
- **Verify:** `Select-String -Path .opencode/skills/vantadb-mcp/references/configuration.md -Pattern 'VANTA_DB'` → no hits.

### B6 | P1 | .opencode/skills/vantadb-mcp — the shipped client-config templates use `~`, which the same skill says does not work
- **Location:** `.opencode/skills/vantadb-mcp/assets/claude-desktop-config.json:4` — `"args": ["server", "--mcp", "--db", "~/.vantadb"]`; `.opencode/skills/vantadb-mcp/assets/opencode-config.json:5` — `"command": ["vanta-cli", "server", "--mcp", "--db", "~/.vantadb"]`
- **Claim/Break:** The skill's own rule, stated 40 lines earlier — `SKILL.md:55-58`: *"`~` does NOT expand in client configs: MCP clients spawn the command directly, without a shell … Always use an **absolute path** for `--db`."* — and `docs/api/MCP.md:37` says the same, adding that `~/.vantadb` "arrives literally and the open fails (`Path::new` receives the verbatim string)". The two templates the skill advertises at `SKILL.md:60-63` ("**Pre-configured templates available in assets/**") violate it. The server receives the literal 11-character string `~/.vantadb` and creates/opens a directory named `~` relative to the client's CWD.
- **Why it matters:** These are the copy-paste artifacts a new user reaches for first. A user who uses them gets a silent wrong-database (data written to a `~` directory, invisible to their real `~/.vantadb`), and then "the skill told me to use an absolute path but its own template doesn't".
- **Trigger:** Copy `assets/claude-desktop-config.json` into `claude_desktop_config.json` → the server opens `./~` (or a `~` folder), not the user's home DB.
- **Fix:** Replace `~/.vantadb` with a placeholder that cannot be silently wrong, e.g. `"--db", "C:/Users/<you>/.vantadb"` (as `docs/api/MCP.md:46` does) plus a comment line in the JSON, or ship a `*_config.example.json` plus a README that says "replace the path".
- **Verify:** `Select-String -Path .opencode/skills/vantadb-mcp/assets/*.json -Pattern '~/'` → no hits.

### B7 | P2 | .opencode/skills/vantadb-mcp — api-reference.md, the declared "single source of truth", omits 11 real tools
- **Location:** `.opencode/skills/vantadb-mcp/references/api-reference.md:5` (`## MCP Tools (85)`), `:8-13` (the "exactly 85 tools = 47 core …" derivation), `:15` (`### Core — … (47)`), and the Core table rows at `:19-58`
- **Claim/Break:** The Core table has **41 rows**; the real base surface is **49**. Missing from the table: `memory_recall`, `memory_reinforce`, `memory_verify_certificate`, `search_with_method`, `search_multi`, `embed_texts`, `write_axiom`, `delete_axiom`, `snapshot_create`, `snapshot_restore`. Missing from the extended families: `code_index`. The file opens with *"This is the single source of truth for the VantaDB MCP contract"* (`:7`) and *"Verified against `vantadb-mcp/src/`: exactly 85 tools"* (`:8`) — both false.
- **Why it matters:** `memory_recall` is the tool the server's own `initialize.instructions` tells the agent to call before every answer (`vantadb-mcp/src/handlers/initialize.rs:19`), and `SKILL.md:459` states the recall policy — but the skill's own "single source of truth" table never documents its params or return shape. `memory_verify_certificate` (DIST-16) and `memory_reinforce` (MEMG-02) are both recent (2026-10-04 / 2026-10-05) and were never back-filled.
- **Trigger:** Agent reads api-reference.md to learn `memory_recall`'s arguments → finds nothing, falls back to guessing.
- **Fix:** Add the 10 missing base tools to the Core table and `code_index` to the Code Intelligence table; restate the header as "88 defined / 82 listed (49 core …)".
- **Verify:** Mechanical diff — every name in `handlers/tools.rs` + the 8 extended modules must appear in `api-reference.md` (extend `scripts/validate-docs-coverage.ps1` to cover the extended modules, per A10).

### B8 | P2 | .opencode/skills/vantadb-mcp — 9 relative links are broken in the copy the agent actually loads
- **Location:** `.opencode/skills/vantadb-mcp/SKILL.md:584-590` (`../../examples/python/{crewai_memory,mem0_integration,autogen_memory,haystack_documentstore,langgraph_checkpoint,semantic_kernel_memory,dspy_retriever}.py`) and `:594` + `:668` (`../../docs/api/MCP.md`)
- **Claim/Break:** From `.opencode/skills/vantadb-mcp/`, `../../docs/…` resolves to `.opencode/docs/…` and `../../examples/…` to `.opencode/examples/…` — **neither directory exists** (`Test-Path .opencode/docs` → False; `Test-Path .opencode/examples` → False). The same links resolve correctly from the git-tracked `skills/vantadb-mcp/SKILL.md` (repo root), which is why the defect is invisible in review but fatal for the `.opencode/` copy — which is the copy OpenCode loads and the copy a user who copies the skill out of `skills/` inherits.
- **Why it matters:** The AI-framework integration list (7 links) and the two "for the full contract see MCP.md" pointers are the skill's only pointers to runnable examples and to the canonical repo reference. Every one is a 404 in the loaded copy.
- **Trigger:** Agent follows `[examples/python/crewai_memory.py](../../../examples/python/crewai_memory.py)` → file not found → it either invents an example path or skips the integration.
- **Fix:** Use repo-root-relative paths in the `.opencode/` copy (or ship an absolute/`${CLAUDE_PLUGIN_ROOT}`-style anchor), and/or add a sync step in `scripts/validate-docs-coverage.ps1` that rewrites the `../../` prefix for the `.opencode/` mirror.
- **Verify:** For each markdown link in `.opencode/skills/vantadb-mcp/SKILL.md`, resolve relative to its own directory and assert existence.

### B9 | P2 | .opencode/skills/vantadb-mcp — mcp-protocol.md declares the wrong current protocol version
- **Location:** `.opencode/skills/vantadb-mcp/references/mcp-protocol.md:11` — `Current version: 2024-11-05`
- **Claim/Break:** `vantadb-mcp/src/handlers/initialize.rs:7` — `pub const LATEST_PROTOCOL_VERSION: &str = "2025-06-18";` and `:10` — `SUPPORTED_PROTOCOL_VERSIONS = ["2025-06-18", "2024-11-05"]`. The server advertises **2025-06-18** and only echoes 2024-11-05 when the client asks for it (`initialize.rs:35-38`); an unknown version also falls forward to 2025-06-18. The reference's `initialize` example (line 39) and response example (line 55) use 2024-11-05, so a reader concludes the server is on the 2024-11-05 spec — which is the version **without** tool `annotations`/`structuredContent`, the two features the rest of the skill depends on.
- **Why it matters:** The skill's whole tool-annotation contract (readOnlyHint/destructiveHint/idempotentHint/openWorldHint) and the `structuredContent` envelope only exist in 2025-06-18. A client written against this reference would not expect them.
- **Trigger:** A user writes a client that negotiates `2024-11-05` and then cannot read `structuredContent`.
- **Fix:** Set line 11 to `Current version: 2025-06-18 (negotiates down to 2024-11-05 only if the client requests it)` and update the two examples.
- **Verify:** `Select-String -Path .opencode/skills/vantadb-mcp/references/mcp-protocol.md -Pattern '2025-06-18'` → hit.

### B10 | P2 | .opencode/skills/vantadb-mcp — setup script pins a stale version and resolves the wrong repo root from the .opencode copy
- **Location:** `.opencode/skills/vantadb-mcp/scripts/setup-vantadb.sh:8` — `VANTADB_VERSION="0.5.0"`; `.opencode/skills/vantadb-mcp/scripts/setup-vantadb.sh:20-26` — `VANTADB_REPO="$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")"` then `cargo install --manifest-path "${VANTADB_REPO}/Cargo.toml" --bin vanta-cli`
- **Claim/Break:** (1) `0.5.0` vs the repo's `0.9.0` (`Cargo.toml:805`); the script prints it in the banner (line 11). (2) From `.opencode/skills/vantadb-mcp/scripts/`, three `dirname`s land on `.opencode/` (scripts → vantadb-mcp → skills → .opencode) — there is no `Cargo.toml` there, so `cargo install --manifest-path .opencode/Cargo.toml` fails with a confusing "could not find `Cargo.toml`". From the git-tracked `skills/vantadb-mcp/scripts/` the same arithmetic correctly lands on the repo root, so the script only works in one of its two homes.
- **Why it matters:** `SKILL.md:14-18` tells the user to run `bash scripts/setup-vantadb.sh` as the **Installation** step. From the installed skill location it fails; and even where it succeeds it announces the wrong version and installs whatever the working tree is, not a released 0.9.0.
- **Trigger:** User follows SKILL.md §Installation from the loaded `.opencode` skill → install error.
- **Fix:** Derive the repo root from an explicit argument or a marker file (`while [ ! -f "$d/Cargo.toml" ]; do d=$(dirname "$d"); done`), and read the version from the manifest instead of hardcoding it.
- **Verify:** `bash .opencode/skills/vantadb-mcp/scripts/setup-vantadb.sh --help` (dry-run) resolves a directory containing `Cargo.toml`.

### B11 | P2 | .opencode/skills/vantadb — the install script's post-install version probe always fails
- **Location:** `.opencode/skills/vantadb/scripts/install-vantadb.sh:41-42` — `"$PY" -c "import vantadb; print('vantadb', vantadb.__version__())" || die "installation succeeded but import check failed"`
- **Claim/Break:** `vantadb.__version__` is a **string**, not a callable. `vantadb-python/vantadb/__init__.py:16` does `from vantadb_py import __version__`; the native module registers it as a string — `vantadb-python/src/lib.rs:2884`: `m.add("__version__", metadata::reported_version().into_owned())?` where `reported_version()` returns `Cow<'static, str>` (`src/metadata.rs:28-40`). Calling it raises `TypeError: 'str' object is not callable`, so the `||` branch fires and the script exits with "installation succeeded but import check failed" **on a fully successful install**.
- **Why it matters:** The script is the `vantadb` skill's install entry point. It reports failure after succeeding, so the user re-runs it, and every run ends the same way — the check can never pass. A user who trusts the exit code concludes the install failed when it did not.
- **Trigger:** `./install-vantadb.sh` on any machine with network access to PyPI.
- **Fix:** `print('vantadb', vantadb.__version__)` (no call), or `str(vantadb.__version__)`.
- **Verify:** `python -c "import vantadb; print(vantadb.__version__)"` → prints the version without raising.

### B12 | P2 | .opencode/skills/vantadb — stale version claims throughout
- **Location:** `.opencode/skills/vantadb/SKILL.md:48` — `vantadb = "0.5.0"`; `.opencode/skills/vantadb/SKILL.md:729` — `### Current Boundaries (v0.5.x)`; `.opencode/skills/vantadb/SKILL.md:776` — `- **Current Version:** 0.5.0`; `.opencode/skills/vantadb/scripts/install-vantadb.sh:49` — `vantadb = "0.5.0"` inside the printed Cargo.toml snippet
- **Claim/Break:** The workspace and every published surface are `0.9.0` (`Cargo.toml:805`; `docs/api/SURFACE_MANIFEST.md` shows `0.9.0` for `vantadb`, `vantadb-py`, `vantadb-server`, `vantadb-wasm`).
- **Why it matters:** The `[dependencies]` block at line 46-49 is the copy-paste target for a user's `Cargo.toml`; pinning `0.5.0` pulls a release that predates the current memory/graph/embedding surfaces. "Current Version: 0.5.0" in a skill that is published to end users is a 4-minor-version understatement of the product's capability (it also implies features that landed later are absent).
- **Trigger:** A user copies line 48 into their `Cargo.toml` and pins an old crate.
- **Fix:** Bump all four occurrences to `0.9.0` (or make the `[dependencies]` block `vantadb = "0.9"`).
- **Verify:** `Select-String -Path .opencode/skills/vantadb/SKILL.md,.opencode/skills/vantadb/scripts/install-vantadb.sh -Pattern '0\.5\.0'` → no hits.

### B13 | P2 | .opencode/skills/vantadb-mcp — configuration.md omits 4 real MCP env vars, including the injection ACL and the audit sink
- **Location:** `.opencode/skills/vantadb-mcp/references/configuration.md:121-139` (the whole "MCP Server (tool surface + budgeting)" section)
- **Claim/Break:** The section documents only `VANTADB_MCP_PROFILE` and `VANTADB_MCP_BYTE_BUDGET`. The server reads four more (`vantadb-mcp/src/config.rs:187-211`): `VANTADB_MCP_INJECT_NAMESPACES` (namespace-prefix ACL restricting what the recall surfaces may read), `VANTADB_MCP_TAINTED_NAMESPACES` (trust classes), `VANTADB_MCP_INCLUDE_TAINTED` (opt-in re-enable), `VANTADB_MCP_AUDIT_LOG` (append-only JSONL injection audit, 10 MiB × 5 rotation). `docs/api/MCP.md:555-620` documents all four in §Injection governance — the skill does not.
- **Why it matters:** These are the VER-04 / MGR-04 security controls. An operator hardening a shared MCP server has no way to discover them from the skill, which is the artifact the MCP client is configured from. The skill's §Security section (SKILL.md:611-616) offers only "namespace isolation" and "editor-level access control" as mitigations — it never mentions that the server itself supports a namespace ACL, tainted-namespace blocking, or an injection audit trail.
- **Trigger:** Operator wants to restrict what an agent can recall; greets the skill's configuration reference → the knob does not exist there.
- **Fix:** Add the four vars to configuration.md §MCP Server, with the boundary-aware prefix semantics and the audit JSONL field list, cross-referencing `docs/api/MCP.md` §Injection governance.
- **Verify:** `Select-String -Path .opencode/skills/vantadb-mcp/references/configuration.md -Pattern 'VANTADB_MCP_INJECT_NAMESPACES'` → hit.

### B14 | P3 | .opencode/skills/vantadb-mcp — the skill calls `docs/api/MCP.md` a "stub" in both directions
- **Location:** `.opencode/skills/vantadb-mcp/SKILL.md:594` — *"see [docs/api/MCP.md](../../api/MCP.md) (a stub)"*; `.opencode/skills/vantadb-mcp/SKILL.md:668` — *"see [docs/api/MCP.md](../../api/MCP.md) — a stub pointing back to this skill as the source of truth."*
- **Claim/Break:** `docs/api/MCP.md` is a 688-line self-contained reference with the tool families, profiles, error-code tables, output budgeting, injection governance, abstention, quarantine and the registry manifest. It does not point back to the skill; it *is* the repo-side authority. The skill's own `SKILL.md:126-127` and `references/api-reference.md:7` simultaneously claim the skill is the "single source of truth" — two documents in the same product each claim authority over the other.
- **Why it matters:** A user told "MCP.md is a stub" will not check the one repo document that carries the 82-tool family table, the profile table and the `-320xx` code table. (Both are broken links anyway — see B8.)
- **Trigger:** User reads the skill's "Editor Integration" section and skips MCP.md entirely.
- **Fix:** Replace "(a stub)" with a one-line description of what MCP.md covers, and state the division of labour explicitly (skill = agent-facing contract; MCP.md = repo reference + registry manifest).

### B15 | P3 | .opencode/skills/vantadb-mcp — mcp-protocol.md repeats the stale tool count
- **Location:** `.opencode/skills/vantadb-mcp/references/mcp-protocol.md:7` — *"The concrete tool list (85 tools: 47 core + 7 skill_* + 8 code_* + 6 wiki_* + 1 context_assemble + 5 scene_* + 6 thread_* + 5 dream_*)"*
- **Claim/Break:** Same defect as B1 (real: 88 defined / 82 listed, 49 core, 9 `code_*` defined). The sentence also says the list "is documented in `references/api-reference.md` § "MCP Tools" — the single source of truth", which is itself incomplete (B7).
- **Why it matters:** A third file repeating a wrong number dilutes the signal — a reader who cross-checks two files that both say 85 concludes the number is right.
- **Trigger:** Reader opens mcp-protocol.md to confirm the surface.
- **Fix:** Same numbers as B1; ideally derive all three files' counts from one place (`vantadb-mcp/tests/mcp_tests.rs` is already the asserted truth) rather than hand-editing three prose files.
- **Verify:** The count string appears in exactly one place after the fix, or is generated.

### B16 | P3 | .opencode/skills/vantadb-mcp — the `.opencode` mirror is missing two asset directories
- **Location:** `.opencode/skills/vantadb-mcp/` (contents) vs `skills/vantadb-mcp/` (contents)
- **Claim/Break:** `skills/vantadb-mcp/` ships `assets/hooks/` (12 files: `README.md`, `TOKEN-BUDGET.md`, `VERSION.md`, `claude/settings.example.json`, `codex/hooks.json`, `cursor/hooks.json`, `opencode/vantadb-memory.js`, `tests/test-hooks.ps1`) and `assets/install/` (7 files: `agent-rule.md`, `claude-desktop.json`, `codex-manual.md`, `cursor.json`, `opencode.json`, `proxy.toml.example`). The `.opencode/skills/vantadb-mcp/` copy has **neither** — it only has `assets/{claude-desktop,cursor,opencode}-config.json`. The mirror gate (`scripts/validate-docs-coverage.ps1:198-205`) hash-checks exactly 10 files, none of which live in those directories, so the omission is invisible to CI. `SKILL.md:463-464` also states *"per-client hook templates are FIND-106 (not here)"*, so the skill text does not promise them — but the canonical `skills/` copy ships them and the two copies disagree.
- **Why it matters:** The loaded copy and the published copy have different capabilities. A user installing the skill from `skills/` gets hook templates; a user (or agent) reading `.opencode/skills/vantadb-mcp/` never learns they exist.
- **Trigger:** Diffing the two trees; a user asking "where are the hook templates?".
- **Fix:** Sync the two trees (or make the mirror gate cover the full file list, e.g. compare recursive file hashes and report extra/missing).
- **Verify:** `Compare-Object (Get-ChildItem skills/vantadb-mcp -Recurse -File -Name) (Get-ChildItem .opencode/skills/vantadb-mcp -Recurse -File -Name)` → empty.

### B17 | P3 | .opencode/skills/vantadb-mcp — "2 resources" vs "2 static + 2 dynamic" inside the same file
- **Location:** `.opencode/skills/vantadb-mcp/SKILL.md:8` — *"exposes **85 tools** …, 2 resources, and 4 prompt templates"*; `.opencode/skills/vantadb-mcp/SKILL.md:437` — *"The server lists 2 static resources in `resources/list`; 2 additional dynamic URIs are servable via `resources/read` (not listed)"*; `docs/api/MCP.md:494` — `## Resources (2 fixed URIs + 2 dynamic schemes)`
- **Claim/Break:** The code serves 4 URIs total: `metrics://` and `schema://` (listed, `vantadb-mcp/src/handlers/resources.rs:13-30`) plus `memory://{ns}/{key}` and `namespace://{ns}` (dynamic, `resources.rs:55-129`). The line-8 summary undercounts the servable surface by 2.
- **Why it matters:** Minor, but the summary line is the one an agent skims; undercounting dynamic resources means an agent that only trusts the summary will not try `memory://` / `namespace://` reads, which are the cheapest read path for a single record.
- **Trigger:** Agent reads line 8, believes only 2 resources exist, and calls `memory_get` (a tool call, costs a schema slot) where a `resources/read` would do.
- **Fix:** Line 8 → "4 resources (2 listed + 2 dynamic), and 4 prompt templates".

---

## Coverage

**Verified / measured (mechanically, not by inspection):**
- Actual MCP tool inventory: extracted from `handlers/tools.rs` `handle_tools_list` block + the 8 `*_tool_definitions()` modules → **88 defined, 82 listed, 6 absorbed**; profile counts recomputed from `profile_allowed_tools` → agent 39 / full 82 / dev 38 / memory 22.
- `docs/api/MCP.md` tool-name coverage: diffed 88 real names against every backticked identifier in the page → **6 `thread_*` tools never named** (A2).
- `docs/api/openapi.yaml` vs `src/server/router.rs`: **44 vs 44 path+method pairs, exact match**; `node scripts/check_openapi_parity.mjs` → "Parity OK", exit 0. Zero paths in openapi.yaml with no route; zero router routes missing from openapi.yaml. Response *schemas* were compared per handler → 9 mismatches (A6).
- `docs/api/HTTP_API.md` Route Summary: 44 rows, same 44 path+method pairs; auth column compared against `middleware.rs` → `/dashboard` mismatch (A7).
- `server.json` validated against the live `https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json` (fetched) → schema-valid (required `name`/`description`/`version` present; `packages`/`remotes` optional).
- `scripts/validate-docs-coverage.ps1` executed → exit 0, "0 gaps", but reports only `vantadb-mcp (tools) - 49 items` (A10).
- `scripts/check_openapi_parity.mjs` executed → exit 0.
- Every MCP `tools/call` JSON example in the skill parsed and was checked against the live `inputSchema` (`memory_list_namespaces` → `required: []`, `additionalProperties:false` enforced at `handlers/tools.rs:1060-1066`; `memory_search` → `namespace` required, `query_vector`/`text_query`/`top_k` accepted) → **all valid**.
- All 19 repo paths referenced by the two skills and by `docs/api/MCP.md` were `Test-Path`-checked → **all exist** (the broken ones in B8 are `.opencode`-relative, not repo-relative).
- All `python`/`json` fenced blocks in the skills + `docs/api/MCP.md` + `docs/api/HTTP_API.md` were parsed → the only failures are intentional `...` placeholders (`mcp-protocol.md:17/84/109/242` use `{ ... }`; `MCP.md:540` / `HTTP_API.md:404` use a bare `...` inside a JSON array) — **not contract defects**, noted for completeness only.
- Panic analysis of the stdio server: every `tools/call` / `resources/read` dispatch is inside `spawn_blocking` → panic becomes `internal_error` → **the server does not die** (A13).
- Error-channel analysis: `McpError::plain` (5 factories) omits `data` entirely; `error_content_mcp` / `error_content_vanta` (`validation.rs:496-517`) render the canonical envelope as a JSON **string** inside `content[0].text` on the `isError` channel — matches `docs/api/MCP.md:157-160` and `SKILL.md:433`.
- Version claims: workspace `0.9.0` (`Cargo.toml:805`); `server.json:6` `0.9.0` ✓; `docs/api/MCP.md:11` `0.9.0` ✓; `docs/api/openapi.yaml:4` `0.9.0` ✓; `docs/api/SURFACE_MANIFEST.md` `0.9.0` ✓; **stale**: `MCP.md:21`, `MCP.md:676`, `SKILL.md:8/124/127`, `api-reference.md:5/8-13/15`, `mcp-protocol.md:7/11`, `configuration.md:127`, `test-mcp.py:21-26/58-62`, `setup-vantadb.sh:8`, `install-vantadb.sh:49`, `vantadb/SKILL.md:48/729/776`, `opencode.jsonc:69`.

**Not covered (out of scope / not verifiable read-only):**
- Runtime behaviour of the HTTP server (no server was started): the A6/A7 findings are derived from handler source + router layering, not from live responses. A live `curl` transcript would be the confirming evidence.
- `vantadb-python` / `vantadb-ts` / `vantadb-node` / `vantadb-wasm` binding surfaces (out of the stated scope). The `vantadb` skill's Python API claims were checked against `vantadb-python/src/lib.rs` signatures only for the 16 methods it names — all exist with matching parameter names (`put`/`search`/`list`/`get`/`delete`/`put_batch`/`put_batch_raw`/`export_namespace`/`export_all`/`import_file`/`purge_expired`/`compact_wal`/`query`/`operational_metrics`/`rebuild_index`/`search_batch`/`hardware_profile`/`flush`/`close`); argument *semantics* (e.g. `db.memory.get` returning a record object rather than a plain dict) were not re-verified at runtime.
- `skills/vantadb-mcp/assets/hooks/**` and `assets/install/**` (present only in the git-tracked `skills/` copy) — their internal content was not audited, only their absence from the `.opencode/` mirror.
- The MCP Registry submission state (external), the official MCP client behaviour regarding `~` expansion (taken from the repo's own documented contract, which is consistent across `docs/api/MCP.md:37`, `SKILL.md:55-58` and `configuration.md:19`).
- No file in the repository was created, modified or deleted; no git write command was run; no server process was started.
