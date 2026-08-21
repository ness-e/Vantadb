# Task MEM-30 — Ingest merge serial + límite concurrencia LLM global (Task 5, P30)

## Estado: ✅ COMPLETADO (verify mecánico 4/4 exit 0)

## Steps

1. ✅ `ingest/mod.rs` — config (global_llm_concurrency default 5 clamp 1-20), STRUCTURAL_FILES, errores #[non_exhaustive], frontmatter helpers + ensure_sources
2. ✅ `ingest/prompts.rs` — prompts inglés (extracción FILE protocol + merge rewrite/append)
3. ✅ `ingest/merge.rs` — parse FILE blocks, normalize_wiki_path, merge_page (P4 fallback), commit serial por página no-bloqueante
4. ✅ `ingest/worker.rs` — orquestación begin_processing → scan/chunk → extract → commit → put_page → complete/fail
5. ✅ Tests D19 a-f (`tests/ingest.rs`, 13 tests) + wiring lib.rs
6. ✅ Verify mecánico completo: `cargo check -p vanta-memory` ✅ · `cargo nextest run -p vanta-memory` 443/443 ✅ · `cargo fmt --check` exit 0 ✅ · `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` exit 0 ✅

## Impacto mapeado (Regla 0)

**Archivos leídos completos / verificados vía codegraph (verbatim):**
- `vanta-memory/src/core/abstractions/llm_runner.rs` — trait `LlmRunner { run(&LlmRunParams) -> Result<String, LlmError> }`, sync (D1); `LlmError::NotConfigured` = LLM-free mode.
- `src/wiki/store.rs` — `WikiStore::new(&StorageEngine)`, `create/request_ingest/begin_processing/complete/fail/get/get_page/put_page/list_pages`; `put_page(namespace, slug, page_type, title, content)` fuerza canonical path type+title y `locked:true`.
- `src/wiki/state.rs` — `WikiState{Pending,Processing,Ready,Failed}`, `is_busy()`.
- `src/wiki/sources.rs` — `scan_local_sources(root) -> Vec<SourceFile{rel_path,content}>`, budget 28000.
- `src/wiki/chunker.rs` — `chunk_text(text, target, overlap) -> Vec<String>`, defaults 12000/400.
- `src/wiki/mod.rs` / `src/lib.rs:141` — todo público: `vantadb::wiki::{WikiStore, scan_local_sources, chunk_text}` consumible desde vanta-memory.
- TDAM clone @ 97f9465: `ingest-v2/index.ts` (STRUCTURAL_FILES :69-75 = wiki/{index,schema,purpose,log,overview}.md; commitCandidates serial :211-283; ensureSources :368-375), `module.ts:35` globalLlmLimit pLimit(getGlobalLlmConcurrency()), `config.ts:104-107` clamp 1-20, `merge.ts` (MergeDecision write|skip, locked→skip, redundant→union-sources write sin LLM, append vs rewrite), `file-protocol.ts` (`<<<FILE path="...">>> ... <<<END>>>`, normalizeWikiPath: no `..`, no drive letters, must start `wiki/`).

**Referencias hacia dentro:** ninguna — `ingest/` es módulo nuevo; nadie importa algo inexistente.

**Referencias entrantes a crear:** `vanta-memory/src/lib.rs` agrega `pub mod ingest;`.

**Veredicto de impacto:** aditivo puro en crate vanta-memory. Blast radius = solo lib.rs (1 línea). Core `vantadb` NO se toca. Sin deps nuevas (serde/serde_json/tracing/thiserror ya presentes).

## Decisiones documentadas

- **Concurrencia (b):** crate sync (D1). El worker hace merge **serial por página** — TDAM commitCandidates también itera páginas en serie; el pLimit(5) JS solo existe porque múltiples *sources* extraen concurrentes. Con un único hilo de merge el límite configurable (default 5, clamp 1-20) es cota superior honrada trivialmente; semaphore+threadpool solo pagaría con extracción concurrente (diferido — ponytail). Config expone el valor clamped.
- **LLM opcional (f):** sin runner o `LlmError::NotConfigured`: extracción no produce candidatos (skip documentado); en commit, página nueva → contenido candidato verbatim (no requiere LLM, igual que el short-circuit de mergePage TDAM); página existente que exige merge real → skip registrado "LLM unavailable". Nunca bloquea, nunca pierde datos.
- **Persistencia:** candidates con relPath `wiki/<dir>/<file>.md` → put_page deriva page_type=dir, title=stem (frontmatter type/title si existen). Canonical path lo decide WikiStore (dedup core).
