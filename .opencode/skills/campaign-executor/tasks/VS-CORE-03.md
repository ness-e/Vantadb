# VS-CORE-03 — Exponer `explain` en el bridge desktop

> **Plan:** `docs/plans/2026-08-18-vanta-studio-fase1.md` Task 1 · **Wave 0** (gaps bridge/core)
> **Estado:** ✅ COMPLETO (commit 2a1f3012 - fase 1)
> **Rol:** vanta-worker (implementación bridge; NO core)
> **OJO paralelismo:** VS-12 edita `native.rs` en paralelo — este cambio es SOLO aditivo (nunca reescribir el archivo completo ni la lógica de open/audit).

## Objetivo

Exponer `explain` en el bridge desktop (re-scopeado: **consumir, no crear**). El core YA produce
`VantaSearchExplanation`/`VantaSearchExplanationHit` (`src/sdk/types.rs:434-463`) y — clave —
**`VantaMemorySearchRequest.explain` + `VantaMemorySearchHit.explanation` YA existen y funcionan**
(`src/sdk/serialization/vector_types.rs:30,66`; `src/sdk/search/mod.rs:122` rellena `explanation`
dentro del `search()` normal cuando `request.explain == true`). El bridge es el único que no lo expone.

## Contrato (aditivo, backward-compat)

- `SearchQuery`: + `explain: bool = false` (serde default)
- `SearchResult`: + `explanation: Option<ExplanationHit>` (serde default)
- `ExplanationHit` + `Bm25Term`: espejo 1:1 de `VantaSearchExplanationHit`/`VantaBm25TermContribution`
  (shape: `{identity, score, snippet?, matched_tokens, matched_phrases, bm25_terms: [{token, tf, df, doc_len, contribution}], rrf_text_rank?, rrf_vector_rank?}`)
- `native.rs`: cuando `explain=true`, `search_request()` setea `explain: q.explain` y `hit_to_result()`
  mapea `h.explanation` → `Option<ExplanationHit>` (el core ya rellena el campo; `db.search()` con
  request explicado ES el camino, sin llamar `explain_memory_search` por separado)
- `data.rs` `vanta_search`: **sin cambio** — `query: SearchQuery` viaja completo por IPC/manager
  (el flag fluye solo; verificado contra manager.rs:177 y data.rs:57-62)
- `vanta.ts`: `SearchQuery.explain?` + `SearchResult.explanation?` + interfaces `ExplanationHit`/`Bm25Term`
- **NO tocar core** (`src/sdk/*`, `src/backends/*`)

## Impacto mapeado (Regla 0) — archivos leídos completos

| Archivo | Referencias entrantes | Referencias salientes | Veredicto |
|---|---|---|---|
| `desktop/src-tauri/src/connections/types.rs` (307L, leído completo) | re-export en `mod.rs:26-29`; usado por trait.rs, manager.rs, native.rs, server.rs, commands/data.rs | serde DTOs | ✅ editar: +explica +ExplanationHit/Bm25Term +tests |
| `desktop/src-tauri/src/connections/native.rs` (502L, leído completo) | `commands/connection.rs` (open); test manager.rs | `VantaEmbedded` (search/get/put/...), `types::*` | ✅ editar aditivo: search_request + hit_to_result + mapper + test |
| `desktop/src-tauri/src/connections/server.rs:201-219` (leído 180-259) | `mod.rs` re-export | `ServerClient.search` | ✅ editar mínima: literal `SearchResult` + `explanation: None` |
| `desktop/src-tauri/src/connections/manager.rs:177,339` (leído 315-374) | `lib.rs` re-export | trait `VantaConnection` | ✅ editar mínima: test literal + `explain: false` |
| `desktop/src-tauri/src/commands/data.rs:57-62` (leído 1-95) | `lib.rs` invoke_handler | manager.search | ✅ SIN cambio (flag viaja en el DTO) |
| `desktop/src/vanta.ts` (251L, leído completo) | componentes: SearchBar.tsx, ResultsList.tsx, WorkspaceShell.tsx | `invoke("vanta_search")` | ✅ editar: tipos TS aditivos |
| `src/sdk/types.rs:420-478` (leído 380-499) | core público | — | 🔒 NO tocar (lectura de referencia) |
| `src/sdk/search/mod.rs:69-256` (leído 60-269) | `sdk/search` | explain_hit | 🔒 NO tocar (referencia: `explain` ya funciona en `search()`) |
| `src/sdk/search/debug.rs:45-84` (leído vía codegraph) | `sdk/search` | — | 🔒 NO tocar (referencia shape) |
| `src/sdk/serialization/vector_types.rs:9-67` (leído 1-90) | `sdk` | — | 🔒 NO tocar (referencia: `VantaMemorySearchRequest.explain` + `VantaMemorySearchHit.explanation` YA existen) |

**Constructores de `SearchQuery`** (se rompen con el campo nuevo): `types.rs:237` (test), `native.rs:411` (test), `manager.rs:339` (test).
**Constructores de `SearchResult`** (se rompen con el campo nuevo): `types.rs:249` (test), `native.rs:178` (`hit_to_result`), `server.rs:211`.

## Steps

### Step 1 — DTOs del wire shape en `types.rs` (RED: tests primero)
- [x] Discovery (shape core verificado: `VantaSearchExplanationHit`/`VantaBm25TermContribution` en `src/sdk/types.rs:445-478`)
- [x] `SearchQuery`: + `#[serde(default)] pub explain: bool`
- [x] `SearchResult`: + `#[serde(default)] pub explanation: Option<ExplanationHit>`
- [x] Nuevos structs `ExplanationHit` + `Bm25Term` (espejo 1:1 del core)
- [x] Tests: `search_query_roundtrip` con `explain: true`; `search_result_roundtrip` con explanation; nuevo test wire-shape (JSON contiene `identity`/`bm25_terms`/`token`/`tf`/`df`/`doc_len`/`contribution`/`rrf_text_rank`/`rrf_vector_rank`); test backward-compat (JSON sin `explain` deserializa a `false`)

### Step 2 — `native.rs` consume explain (GREEN)
- [x] `search_request()`: `explain: q.explain` (ya hay `..Default::default()` cubriendo el resto)
- [x] `hit_to_result()`: mapear `h.explanation` → `Option<ExplanationHit>` con fn `explanation_to_dto`
- [x] Imports: `VantaSearchExplanationHit`, `VantaBm25TermContribution`
- [x] Test literal `SearchQuery` en native.rs:411 + `explain: false`
- [x] Nuevo test `search_explain_fills_breakdown` (tokio::test): ingest 2 records, search `explain: true`, assert `explanation` Some con `bm25_terms` no vacío y ranks

### Step 3 — literales restantes + vanta.ts
- [x] `server.rs:211`: `explanation: None` (server no soporta explain; flag ignorado, backward-compat)
- [x] `manager.rs:339`: test literal + `explain: false`
- [x] `vanta.ts`: `SearchQuery.explain?: boolean`; `SearchResult.explanation?: ExplanationHit | null`; interfaces `ExplanationHit`/`Bm25Term` (mirror snake_case del wire)

### Step 4 — Verificación mecánica (obligatoria)
- [x] `cargo check` en `desktop/src-tauri` ✅ (solo warning pre-existente `mobile` cfg, ajeno)
- [x] `cargo test -j 1` en `desktop/src-tauri` ✅ — **41/41 lib + 15 integración (56 total, 0 failed)**; incluye mis 3 tests nuevos + los 7 de VS-12 (slice paralelo)
- [x] `npm run build` en `desktop/` ✅ (tsc + vite; solo warning chunk size pre-existente)

## Handoff al lead / VS-12 (paralelismo)

El worktree contiene AMBOS slices (VS-CORE-03 + VS-12). Merge = lead. Notas:
- **Fixes mínimos aplicados al slice WIP de VS-12** (solo `.clone()`/`let mut`, sin cambiar su lógica de open/audit) para que `cargo check`/`test` compilen:
  - `native.rs:55` `open()`: `path.clone()` antes de `path.join("audit.jsonl")` (borrow-after-move)
  - `native.rs:71` `open_with_audit()`: `audit_log_path.clone()` en `VantaConfig` (move duplicado)
  - `native.rs:641` test `put_and_delete_write_audit_events`: `let mut conn`
- **`mod.rs` re-export:** mi edit añadió `Bm25Term`/`ExplanationHit` a la lista `pub use types::{...}`; si VS-12 añade `AuditEvent`/`AuditPage` ahí, el merge combina ambos.
- **`types.rs`/`native.rs`/`manager.rs`/`vanta.ts` son archivos compartidos** — los diffs de ambos slices coexisten y pasan los tests juntos; el merge debe conservar ambas partes.
- **`data.rs` NO se tocó:** el flag `explain` viaja en el DTO `SearchQuery` completo por IPC → manager → trait (verificado manager.rs:177, data.rs:57-62).

## Notas de diseño

- **Camino del explain:** el plan dice "llamar `search_with_method`/`search` con el request explicado" —
  exactamente eso: `db.search(request)` con `request.explain=true` (el core ya rellena
  `VantaMemorySearchHit.explanation` en `src/sdk/search/mod.rs:240-255`). No hace falta
  `explain_memory_search` por separado (ese API explica sin hits planos; el search con flag devuelve
  AMBOS: record + explanation — lo que el SearchResult necesita).
- **1:1 mirror DTO** (no reusar el tipo core): convención del crate (MemoryRecord/ListPage son mirrors)
  + el wire shape del frontend queda desacoplado del core.
- **data.rs sin cambio:** el flag viaja en `SearchQuery` (DTO completo a través de IPC → manager → trait).
- **Backward-compat:** `explain` ausente → `false`; `explanation` ausente/null → `None`. Server backend
  ignora el flag (explica solo el native).