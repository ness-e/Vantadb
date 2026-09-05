# MEM-33 - MCP tools wiki_* query-only (4 tools) — ✅ COMPLETED

Plan: `docs/plans/2026-08-21-vanta-proxy-knowledge.md` Task 7 · Ruta: vanta-worker
Contrato: `cargo check -p vantadb-mcp` pasa; tests D19: (a) wiki_search rankea; (b) wiki_read respeta `locked:true` como metadata visible; (c) wiki_list; (d) wiki_graph BFS multi-hop cap 200 nodos; (e) wiki pending (no ready) → falla clara con estado actual; (f) read-only estricto
Stop condition: si el índice por-wiki exige infra nueva en core → index en la partición del store con rebuild transaccional (manager.ts:394-412 adaptado). **NO aplica**: no se necesita infra nueva.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/wiki/store.rs` (466L, MEM-28 commit 0c3a9dcf) — `WikiStore::get/get_page/list_pages`, estados (`WikiState{Pending,Processing,Ready,Failed}` + Display), `WikiPage{namespace,wiki_slug,path,page_type,title,locked,content,updated_at_ms}`. Pages SIEMPRE `locked:true`. Validación de scope/path interna en cada método.
- `src/wiki/state.rs` (57L) — máquina `pending→processing→ready|failed`; `is_busy()`.
- `vantadb-mcp/src/code.rs` (327L, MEM-32 commit 70048abf) — patrón a copiar: `*_tool_definitions()` + `handle_*_tool(name,args,storage,config)`; errores de dominio como `Ok(error_content(...))` nunca `Err` propagado.
- `vantadb-mcp/src/handlers/tools.rs` (897L) — wiring aditivo: list extend + match arm delegando al módulo.
- `vantadb-mcp/src/lib.rs` (38L) — `mod code;` declaraciones privadas, sin re-exports del módulo.
- `vantadb-mcp/tests/code_tests.rs` (353L) — setup tempdir+StorageEngine+Executor, helper `call()`/`msg()`.
- MEM-30 (vanta-memory ingest): páginas se cross-referencian con `[[wikilinks]]` (prompts.rs:25,56).

**Elección documentada — índice de búsqueda:** las páginas viven como records serde en la partición `InternalMetadata` (keys `wiki:{ns}::{slug}:page:{path}`), NO como memory records — el text_index BM25 del core (`bm25`) solo cubre payloads de memory records y NO los ve. MEM-30 no construyó índice separado. Por lo tanto `wiki_search` usa **scan+rank propio estilo BM25** sobre `list_pages()` (k1=1.5, b=0.75, IDF suavizado; términos del título ponderados ×5, TDAM manager.ts:381-391). Sin deps nuevas, sin tocar core. Upgrade path: si el volumen exige, índice en la partición del store con rebuild transaccional (stop condition).

**wiki_graph — aristas:** los edges del grafo wiki son los `[[wikilinks]]` del contenido (única forma de linking que produce MEM-30). BFS multi-hop desde un `root_path` resolviendo links→página por título (case-insensitive) o stem del path canónico. Cap duro 200 nodos visitados (TDAM graph-search.ts:38 DEFAULT_MAX_NODES=200), flag `"truncated"` cuando se alcanza.

**Referencias entrantes:** handlers/tools.rs único punto de dispatch MCP. Cambios 100% aditivos. Blast radius = crate vantadb-mcp solamente. Core `vantadb` intocado (solo consumo SDK).

## Steps

### Step 1 - wiki.rs: 4 tools + wiring + definiciones ✅ DONE
- `vantadb-mcp/src/wiki.rs`: `wiki_tool_definitions()` + `handle_wiki_tool(name,args,storage,config)`; guard `require_ready()` en las 4 tools (error "wiki not ready" con estado actual); helpers `bm25_rank` + `wikilinks` + `bfs_graph`.
- Wiring: `mod wiki;` en lib.rs; list extend + match arm en handlers/tools.rs.

### Step 2 - Tests D19 ✅ DONE
- `vantadb-mcp/tests/wiki_tests.rs`: 6 tests sobre WikiStore seedeado (create→begin_processing→put_page→complete): (a) search rankea título×5 sobre body + query sin match → set vacío; (b) read expone `locked:true` como metadata visible + missing page error; (c) list ordenado por path canónico con flag locked; (d) graph BFS hub-topology (Root→Hub→300 leaves): visited=200 exacto, truncated=true, hops respetados, unknown root error; (e) pending → "wiki not ready" con estado actual en las 4 tools + wiki inexistente; (f) read-only estricto: fingerprint de list_pages estable tras las 4 tools + `version` del lifecycle record intacto.

### Step 3 - Verify mecánico completo ✅ DONE
- `cargo check -p vantadb-mcp` → Finished exit 0
- `cargo nextest run -p vantadb-mcp` → 29/29 PASS (6 wiki_* nuevos + 23 pre-existentes)
- `cargo fmt --check` → exit 0
- `cargo clippy -p vantadb-mcp --all-targets --no-deps -- -D warnings` → exit 0

## Recitation final

- **Resultado:** OK — contrato D19 cumplido completo ((a)-(f) todos verdes).
- **Elección índice:** scan+rank BM25 propio en MCP layer (k1=1.5, b=0.75, IDF ln_1p suavizado, título×5 TDAM manager.ts:381-391). Justificación: páginas son records serde en partición `InternalMetadata`, invisibles al text_index del core (que solo indexa memory records); MEM-30 no construyó índice separado; sin deps nuevas, core intocado. Upgrade path documentado: índice en la partición del store con rebuild transaccional si el volumen lo exige.
- **wiki_graph edges:** aristas = `[[wikilinks]]` extraídas del contenido (única forma de linking que produce MEM-30, prompts.rs:25/56); resolución link→página por título case-insensitive o stem del path; BFS cap duro 200 nodos (TDAM graph-search.ts:38) con flag `truncated`.
- **Fix durante verify (root-cause):** errores de dominio (`with_store`) salían por el slot `Err` → shape `{content:[...]}` perdido, test (e) fallaba con mensaje vacío. Aplicado aprendizaje MEM-32: `domain_err()` devuelve `Ok(error_content(...))`.
- **Deuda:** ninguna nueva; sin deps nuevas; blast radius = solo crate vantadb-mcp (`wiki.rs` nuevo, wiring aditivo en `lib.rs` + `handlers/tools.rs`). Core `vantadb` intocado.
- **Próxima tarea:** Task 8 (MEM-31 progreso ingest run_id).
