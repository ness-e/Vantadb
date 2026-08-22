# MEM-44 — E2e ingest→tools wiki_* roundtrip (P31 Task 2)

**Estado:** ✅ COMPLETED · **Plan:** docs/plans/2026-08-22-vanta-final-cierre.md (Task 2) · **Ruta:** vanta-worker

## Objetivo
Un test cross-crate que encadene: fixture .md temporales → `vanta_memory::ingest::worker::run` (runner fake) → `wiki_search` / `wiki_read` / `wiki_graph` vía handlers MCP encuentran y leen lo ingestado.

## DISCOVERY

### Dirección de dependencias (`cargo tree`, 2026-08-22)
```
vantadb-mcp v0.5.0 └── vantadb v0.5.0
vanta-memory v0.5.0 └── vantadb v0.5.0
```
Ninguno depende del otro hoy → **sin ciclo posible en ninguna dirección**. Decisión: test único en `vantadb-mcp/tests/` con `vanta-memory` como **dev-dependency** de `vantadb-mcp` (solo tests, cero impacto productivo). El stop condition "2 tests hermanos" NO se activa.

### Impacto mapeado (Regla 0)
**Archivos leídos completos:**
- `docs/plans/2026-08-22-vanta-final-cierre.md` — bloque Task 2 completo
- `vantadb-mcp/src/wiki.rs` (425L) — handlers wiki_*: `with_store` exige state `ready`; `handle_wiki_tool` es la superficie; BM25 local; BFS wikilinks
- `vanta-memory/src/ingest/worker.rs` (312L) — `worker::run(store, ns, slug, root, runner, config)`: request_ingest→begin_processing→scan/chunk/extract(LLM)→merge→put_page→complete
- `vanta-memory/tests/ingest.rs` (596L) — patrón ScriptedRunner FIFO + `file_block()` + in-memory engine
- `vantadb-mcp/tests/wiki_tests.rs` (379L) — patrón `handle_tools_call(name, args, executor, storage, config)` + `seed_wiki`
- `vantadb-mcp/Cargo.toml` — dev-dependencies actual `[tempfile = "3"]`

**Referencias hacia dentro:** ninguna (test nuevo, no toca código productivo salvo dev-dep en Cargo.toml).
**Referencias entrantes:** `cargo nextest -p vantadb-mcp` correrá el nuevo archivo; clippy `-p vantadb-mcp --all-targets` lo lintea.
**Veredicto de impacto:** mínimo — 1 línea Cargo.toml + 1 test file nuevo. Sin cambios en src/.

## Steps

### Step 1 ✅ — dev-dependency + test e2e roundtrip
- [x] `vantadb-mcp/Cargo.toml`: `vanta-memory = { path = "../vanta-memory" }` en `[dev-dependencies]`
- [x] `vantadb-mcp/tests/wiki_roundtrip_e2e.rs`: UN test (`e2e_ingest_then_wiki_tools_roundtrip`) que encadena fixture .md → worker::run (ScriptedRunner, 2 páginas enlazadas [[Redis]]) → wiki_search ("memory" matchea ambas) → wiki_read (contenido mergeado + locked:true) → wiki_graph (edge persistence→redis). **PASS**

### Step 2 ✅ — Verify mecánico completo (todos exit 0)
- [x] `cargo check -p vanta-memory` — exit 0
- [x] `cargo nextest run -p vanta-memory` — 455/455 pass
- [x] `cargo check -p vantadb-mcp` — exit 0
- [x] `cargo nextest run -p vantadb-mcp` — 30/30 pass (29 + 1 nuevo)
- [x] `cargo fmt --check` — exit 0
- [x] `cargo clippy -p vantadb-mcp --all-targets --no-deps -- -D warnings` — exit 0

### Step 3 ✅ — Cierre
- [x] Task file actualizado
- [x] `campaign_update_task_state` taskId=2 completed + recitation §3
- [x] SIN commit ni edición de plan file (orden explícito del orquestador) — commit pendiente del lead

## Notas de ejecución
- Primer nextest falló con E0425/E0463 masivo → reintento compiló limpio (transitorio file-lock/AV Windows, patrón MEM-43 ya documentado).
- Query inicial "persistence" solo matcheaba 1 página → corregida a "memory" (término presente en el contenido de ambas páginas ingestadoas).
- Filtro posicional de nextest no matcheó el binario nuevo; usar `-E 'test(...)'`.

## Decisiones
- **D-test-1:** runner fake scripted (no fallback P4): sin runner el ingest no extrae nada (sources_skipped), imposible probar search/read/graph. P4 queda cubierto por tests MEM-30 existentes.
- **D-test-2:** 1 solo archivo .md fuente → exactamente 1 chunk → 1 llamada LLM determinística (el orden de scan multi-archivo no está garantizado).
- **D-test-3:** dirección dev-dep mcp→memory elegida porque los handlers MCP se consumen mejor desde su propio crate (`handle_tools_call` público) y mantiene vanta-memory libre de deps de test ajenas.
