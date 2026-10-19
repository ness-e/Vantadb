---
title: "TASK MEMG-08: Formatos e ingestores (trait `Ingestor` — MGR-25)"
kind: task
description: "Trait `Ingestor` en el core (`vantadb::wiki`) que convierte formatos a chunks `MemoryInput` con proveniencia obligatoria (`source`/`page`/`chunk`) + pipeline de scan con `SOURCE_CHAR_BUDGET` (28k) y chunking con overlap; mínimo viable txt/json/csv end-to-end con tests (html/pdf/docx → FIND-298, sin deps nuevas)"
---

# TASK MEMG-08: Formatos e ingestores (trait `Ingestor` — MGR-25)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 52, bloque F3 L1492-1518)
- **Fuente:** plan Task 52 + Backlog `MEMG-08` (fila L140) + Backlog `MGR-25` (fila L835 — spec completa en Notion, no en repo)
- **Esfuerzo:** 🟠 1-2sem | **Appetite:** max 2sem | **Stop (plan L1503):** 2sem sin los 5 formatos → entregar trait + txt/json/csv end-to-end + FIND de html/pdf/docx → **este run entrega el stop (trait + 3 formatos + FIND-298)**
- **Prioridad:** 🟠
- **Tipo:** Rust — core (`src/wiki/ingestors.rs` nuevo, refactor interno de `src/wiki/sources.rs`, exports en `src/wiki/mod.rs`) + test de integración nuevo (`tests/wiki_ingestors.rs`)
- **Turns estimados:** 7-10 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `52` en el campaign server)
- **Incógnitas (uphill):** 1 — forma exacta del trait + de `metadata.source={file,page,chunk}` (spec en Notion, no accesible) → **RESUELTA en DISCOVERY con evidencia del repo** (ver §Spec #2/#3)
- **Pendientes (downhill):** 6 steps (0-5)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `52`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `scan_local_sources` (7 callers: `src/wiki/mod.rs`, `vanta-memory/src/ingest/worker.rs:177`, `vanta-memory/src/ingest/auto_sync.rs:164`, tests inline) — **comportamiento idéntico, firma intacta** (el refactor extrae el walk genérico `collect_text_files`). Símbolos nuevos (`Ingestor`, `default_ingestors`, `scan_ingestable_sources`) = 0 callers productivos iniciales (superficie core-only; el wiring a un host es tarea futura, igual que MEM-28/29). |
| Callees | `src/wiki/chunker.rs` (`chunk_text`, `DEFAULT_TARGET_CHARS`=12k, `DEFAULT_OVERLAP_CHARS`=400), `src/sdk/types` (`MemoryInput`, `MemoryMetadata`, `Value`), `src/error.rs` (`Error::InvalidInput`), `tracing`. |
| Implicaciones | 0 cambios de wire/serialización; 0 migración; 0 deps nuevas; 0 unsafe. `scan_local_sources` conserva semántica exacta (guard traversal + budget + skip) — cubierto por sus 7 tests actuales. Nuevo camino: lectura de archivos ya soportada (UTF-8) → `Vec<MemoryInput>` (chunks con proveniencia) — el consumidor decide el `put`. Riesgo de concurrencia: ninguno (funciones puras de scan single-thread; sin locks nuevos). |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `559cf8ad`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/wiki/sources.rs` (:18 `SOURCE_CHAR_BUDGET`=28_000; :22-27 `SourceFile{rel_path,content}`; :32-49 `scan_local_sources`; :51-55 `ensure_within_root` (trust boundary); :57-121 `walk` — filtro `.md` :82, guard :87-96, truncación budget :109-116; tests :131-266).
  - `src/wiki/mod.rs` (:19-27 exports del módulo).
  - `src/wiki/chunker.rs` (:10 `DEFAULT_TARGET_CHARS`=12_000; :13 `DEFAULT_OVERLAP_CHARS`=400; :19-63 `chunk_text` — overlap = cola de 400 chars que abre el chunk siguiente).
  - `src/sdk/types/record.rs` (:178-222 `MemoryInput` — `payload: String` + `metadata: MemoryMetadata`; :224-248 `MemoryInput::new`).
  - `src/sdk/types.rs` (:105-128 `Value` — **sin variante objeto**; :146 `Fields`; :149 `MemoryMetadata`).
  - `src/sdk/serialization/mod.rs` (:12 `RESERVED_PREFIX`=`__vanta_`; :108-131 `validate_namespace`; :133-153 `validate_key` — no-vacío, ≤512 bytes, sin NUL → `#` y `/` permitidos; :155-168 `validate_metadata`).
  - `vanta-memory/src/ingest/worker.rs` (:177 `scan_local_sources`; :188-192 `chunk_text` — consumidor actual del scanner).
  - `vanta-memory/src/ingest/auto_sync.rs` (:161-164 — segundo consumidor).
  - `vanta-memory/src/seed/md_import.rs` (:320-365 `import_md_dir` — round-trip `.md`; :393-407 walk `.md` — formato ya cubierto por otro path, NO entra acá).
  - `vantadb-mcp/src/wiki.rs` (:182-309 `handle_wiki_tool`; :267-297 `wiki_ingest`; :420-481 `start_ingest` — el path wiki sigue siendo `.md` + LLM; el trait MGR-25 es del path memoria, no del wiki-page).
  - `vantadb-mcp/src/validation.rs` (:288-296 `parse_metadata`; :742-814 tests — **objetos anidados en metadata RECHAZADOS** → evidencia para la forma plana de proveniencia).
  - `examples/rag_pdf_chat/rag_pdf_demo.py` (:186 — precedente repo de proveniencia MGR-25: `{"source": file, "page", "chunk"}` planos).
  - `docs/dev/research/mgr-12-confianza.md` (:71 — "procedencia externa va en `metadata.source`").
  - `tests/memory_api.rs` (:1-70 patrón de test de integración: `tempfile` + `Embedded::open`; :165-197 patrón `db.search(MemorySearchRequest{...})`).
  - `src/sdk/importers/mem0.rs` (:83 — precedente `source` como `Value::String`), `src/sdk/api/memory.rs` (:761 `put`, :783 `put_batch(Vec<MemoryInput>)`, :1067 `get`).
- **Archivos referenciados hacia dentro (imports/deps):** `sources.rs` importa `crate::error::{Error, Result}` + `std::path`; `ingestors.rs` importará `crate::error`, `crate::sdk::types::{MemoryInput, Value}` (vía `crate::sdk`), `crate::wiki::chunker::{chunk_text, DEFAULT_*}` + `super::sources::collect_text_files` (nuevo `pub(crate)`); el test nuevo importará `vantadb::{Embedded, MemorySearchRequest}` + `vantadb::wiki::{scan_ingestable_sources, default_ingestors, ...}` + `tempfile`.
- **Referencias entrantes (grep HEAD):** `scan_local_sources` = 2 callers productivos (worker.rs:177, auto_sync.rs:164) + tests inline (sources.rs) + `src/wiki/mod.rs:25`; `collect_text_files`/`Ingestor`/`scan_ingestable_sources`/`default_ingestors` = **0 hits** (nuevos); `wiki_ingestors` = **0 hits** (test nuevo).
- **Archivos a crear/tocar (este run):** `src/wiki/ingestors.rs` (nuevo), `tests/wiki_ingestors.rs` (nuevo), `src/wiki/sources.rs` (refactor interno), `src/wiki/mod.rs` (exports), `docs/dev/tasks/MEMG-08.md`, `docs/dev/Backlog.md`.
- **Veredicto impacto:** **MEDIO (API pública aditiva en core + refactor interno de un scanner con guard de seguridad)** — sin cambios de firma en lo existente, sin wire, sin deps. Pre-mortems del plan mitigados: (1) parsers pesados → **sin deps nuevas**; html/pdf/docx → FIND-298 (stop L1503); (2) proveniencia → forma plana `source`/`page`/`chunk` decidida-por-evidencia (§Spec #3); (3) budget 28k → cuenta payload post-chunk con truncación declarada + test exacto (§Spec #4).

## Contrato

"Trait `Ingestor` (formato → chunks `MemoryInput` con `metadata.source={file,page,chunk}` obligatoria) + mínimo viable ≥2 formatos end-to-end (txt/json/csv) con tests; respeta `SOURCE_CHAR_BUDGET` (28k) + chunking con overlap; orden declarado txt/json/csv → html → pdf → docx (por costo/beneficio); código (rs/py/ts) NO entra (MGR-22); sin deps nuevas (stop: 5 formatos → trait + txt/json/csv e2e + FIND de html/pdf/docx)."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): NO disparado** — el contrato del plan F0 (Gate Result ✅ DO, L1500-1503, aprobado por el owner) **manda literalmente** el símbolo público nuevo ("trait `Ingestor` (formato → chunks `MemoryInput`...)") + su forma + stop conditions; precedente idéntico: WIRE-18/MEMG-03 ("Gate D pre-respondido por el plan F0"). Blast radius <10 archivos, sin hot path. La forma exacta del trait se fija acá con evidencia del repo (la spec Notion no es accesible a workers — `pipeline-full.md` 0c-context; el plan ya la resume en L1497/L1501).

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Ubicación del trait | A) **`src/wiki/ingestors.rs` (core `vantadb::wiki`)** (pro: `src/wiki/` ya es "LLM-free store in core" — `mod.rs:1`; el scanner (`sources.rs`) y el chunker viven ahí; el output `MemoryInput` es tipo core; los archivos clave del plan son core) / B) `vanta-memory/src/seed/` (contra: invierte la dirección de dependencia — vanta-memory ya consume `vantadb::wiki`; duplicaría scan/chunker) | ✅ **A** — decidido-por-evidencia: `worker.rs:177` ya llama `vantadb::wiki::scan_local_sources`; el trait compone con esa maquinaria |
| 2 | Forma del trait | A) **`extensions() -> &'static [&'static str]` + `ingest(namespace, file, content) -> Result<Vec<MemoryInput>>` con método provisto** (chunk_text + proveniencia; override para formatos futuros) (pro: contrato literal "formato → chunks MemoryInput"; entrada `&str` = lo que ya produce el scanner y lo que consumen los formatos texto; pdf/docx overridean `ingest` en su momento) / B) entrada `&[u8]` (pro: binarios pdf/docx directos; contra: YAGNI sin consumidor, ensucia los 3 formatos texto) / C) sin método provisto (cada formato duplica chunk+provenance; contra: duplicación) | ✅ **A** — decidido-por-evidencia: `SourceFile.content: String` (`sources.rs:26`); `chunk_text` existente; `MemoryInput.payload: String` (`record.rs:184`). `Send + Sync` como supertraits (los ingest futuros corren en threads — `start_ingest` spawnea; gratis para unit structs) |
| 3 | Forma de `metadata.source={file,page,chunk}` | A) **Plana: `source`=String(rel_path) + `page`=Int + `chunk`=Int** (pro: `Value` NO tiene variante objeto (`types.rs:105-128`) y el parser MCP RECHAZA objetos anidados (test `parse_metadata_delegates_lists_and_null_rejects_objects`, `validation.rs:750`); tipado/queryable por filters; precedente exacto del repo) / B) `source`=String(JSON `{"file":...,"page":...,"chunk":...}`) (contra: string opaco, sin precedente) / C) claves dotted `source.file` (contra: sin precedente, mismo problema) | ✅ **A** — decidido-por-evidencia: `rag_pdf_demo.py:186` usa `{"source": file, "page", "chunk"}` como "proveniencia estilo MGR-25"; importers usan `source` String (`mem0.rs:83`); MGR-12: procedencia externa → `metadata.source` (`mgr-12-confianza.md:71`). **`page`=0 en formatos sin paginación (declarado); `chunk`=índice 0-based** |
| 4 | Semántica del budget 28k | A) **Cuenta payload total emitido (post-chunk, overlap incluido); el chunk que cruza se trunca y el scan para** (pro: es lo que se persiste; misma semántica declarada del scanner actual `sources.rs:109-116`) / B) contar chars de archivo pre-chunk (contra: no refleja lo emitido; overlap duplicaría fuera de cuenta) | ✅ **A** — decidido-por-evidencia: contrato "respeta SOURCE_CHAR_BUDGET + chunking con overlap" + pre-mortem 3 "truncación declarada"; test de truncación exacta (mismo estilo de `total_content_respects_char_budget`) |
| 5 | Keys de los chunks | A) **`{rel_path}#{chunk_index}`** (pro: determinístico → re-scan idempotente (re-put actualiza, mismo patrón `content_key`/md_import); right-split unívoco porque el índice es el último `#` + dígitos) / B) hash de contenido (contra: re-put duplica al editar; `content_key` es para importers sin id) | ✅ **A** — decidido-por-evidencia: `validate_key` solo exige no-vacío/≤512/sin NUL (`serialization/mod.rs:133-153`) |
| 6 | Registro y dispatch | A) **`default_ingestors()` (txt/json/csv) + `scan_ingestable_sources(root, namespace, ingestors)`** (pro: el trait es el punto de extensión declarado; el orden txt/json/csv → html → pdf → docx queda en el registry; test de dispatch con registro explícito) / B) scan con lista hardcodeada (contra: agregar html/pdf/docx tocaría el pipeline) | ✅ **A** — decidido-por-evidencia: contrato "orden declarado" (L1501) |
| 7 | Reuso del walk (guard traversal) | A) **Refactor: extraer `collect_text_files(root, accept, visit)` `pub(crate)` en `sources.rs`; `scan_local_sources` pasa a usarlo (comportamiento idéntico) y el nuevo scan también** (pro: UN solo guard `ensure_within_root` + canonicalización; cero duplicación de trust boundary) / B) walk nuevo en `ingestors.rs` (contra: duplica el guard de seguridad — deuda prohibida) | ✅ **A** — decidido-por-evidencia: `ensure_within_root` (`sources.rs:51-55`) es trust-boundary; los 7 tests de `sources.rs` verifican el refactor |
| 8 | html/pdf/docx en este run | A) **FIND-298** (pro: stop L1503 lo permite explícitamente; pre-mortem 1: "deps solo si la spec las sanciona" — spec Notion no accesible → no hay sanción verificable → **cero deps nuevas**) / B) implementarlos (contra: `html2text`/`pdf-extract`/docx = deps pesadas sin sanción; budget de sesión) | ✅ **A** — decidido-por-evidencia: stop L1503 + pre-mortem 1 (L1502) |
| 9 | Documentación | A) **Rustdoc canónico del módulo (module + items) + task file + avance** (pro: `vantadb::wiki` no tiene página en `docs/api/` — precedente MEM-28/29 sin docs/api; `validate-docs-coverage.ps1` no escanea `src/wiki/`) / B) sección nueva en `docs/api/EMBEDDED_SDK.md` (contra: ese doc es de la facade `Embedded`; wiki no vive ahí) / C) `docs/api/WIKI.md` nueva (contra: scope creep documental de MEM-28/29 ajenos) | ✅ **A** — decidido-por-evidencia: `rg scan_local_sources docs/` → 0 páginas de API del módulo wiki; R-1/R-4 cumplidos por rustdoc con rutas reales. NOTICED: gap docs/api del módulo wiki (pre-existente, no es de este cambio) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **`scan_local_sources` comportamiento idéntico** (firma + semántica: filtro `.md`, guard traversal, budget con truncación) — sus 7 tests actuales son el contrato del refactor.
  2. **Proveniencia obligatoria:** cada chunk emitido por el pipeline lleva `source` + `page` + `chunk` (claves `__vanta_` están prohibidas para metadata — `RESERVED_PREFIX`; las nuestras no lo son).
  3. **Budget:** suma de `payload.chars().count()` de todos los inputs ≤ `SOURCE_CHAR_BUDGET` (28_000).
  4. **Cero deps nuevas / cero wire / cero migración / cero unsafe**; sin `unwrap`/`expect` en producción (tests con `#![allow(clippy::expect_used, clippy::unwrap_used)]`, convención repo `memory_api.rs:1`).
  5. **No tocar** `src/wal.rs`, `src/vector/`, `src/storage/` (dominio Arch/Engine); el cambio vive en `src/wiki/` + test.
  6. **WIP ajeno** (MEMG-03 en vuelo: `src/sdk/api/{graph,memory}.rs`, `tests/memory_graph_lineage.rs`, `docs/dev/tasks/MEMG-03.md`, `docs/api/EMBEDDED_SDK.md`, `opencode.jsonc`) NO se toca ni se stagea; PROHIBIDO tocar master plan / `docs/pipeline-state.json`; commit con **pathspec**.
  7. **Determinismo:** orden de scan = paths ordenados (ya lo hace el walk); keys estables entre re-scans.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin deps nuevas, sin unwrap. **Elimina** la deuda "`put` recibe `payload: String` sin extractor para documentos reales" (MEMG-08) en txt/json/csv y **no duplica** el guard traversal (refactor compartido). `ponytail:` notas: (a) scan single-thread secuencial (volúmenes grandes → paralelizar si aparece consumidor); (b) keys stale de chunks de un archivo que encoge no se limpian (mismo comportamiento que md_import; sin contrato de borrado). `NOTICED BUT NOT TOUCHING`: `.md` no ingestable por este path (lo cubre el wiki path); gap docs/api del módulo `vantadb::wiki` (pre-existente); wiring a un host (MCP/worker) no pedido por el contrato → FIND-298 lo menciona como continuación.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: trait `Ingestor` + `default_ingestors()` + `scan_ingestable_sources()`; txt/json/csv e2e (scan → `put_batch` → get/search con proveniencia); budget 28k exacto con truncación; chunking con overlap (400) verificado; `scan_local_sources` sin regresión (tests existentes); fmt/clippy; rustdoc; FIND-298; review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(wiki): MEMG-08 — ...` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | changelog (minor) — `feat:` → release-plz bump minor |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius: `scan_local_sources`/`MemoryInput`/`chunk_text`) + `codebase-memory-mcp_check_index_coverage` (paths clave)
- `cargo nextest` scoped por crate (`-p vantadb --test wiki_ingestors`, `-p vantadb wiki::` suite) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs`) — task file + Backlog editados

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `source-driven-development` · `security-and-hardening` (SDP base) · `incremental-implementation` · `test-driven-development` · `context-engineering` · `doubt-driven-development` (SDP lifecycle BUILD) + rol: `rust-write-tests` (tests por formato/edge), `api-and-interface-design` (forma del trait). `documentation-and-adrs` evaluada (decisión: rustdoc canónico — §Spec #9).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: trust boundary tocado = **lectura de archivos de un root provisto** (ya existente, `scan_local_sources`). El refactor PRESERVA el guard `ensure_within_root` + canonicalización + skip de no-UTF8 (tests existentes + nuevos cubren). Superficie nueva: contenido de archivos → `payload` (texto plano, sin parsing ejecutable; `Value::String` metadata con claves fijas `source`/`page`/`chunk` — sin inyección de claves). Sin red, sin secrets, sin FFI, sin deps. → Sin hallazgos que requieran `security-and-hardening` más allá del checklist (no aplica carga completa: no hay input de usuario nuevo — el root ya es input existente del scanner).
- [ ] **PERFORMANCE** — no dispara (Regla 9): sin hot path (scan de disco single-thread, no search/ingestión core); sin claims de performance. Nota: el nuevo scan no altera `scan_local_sources` (mismos pasos).

## Steps

### Step 0 — DISCOVERY + task file + FIND-298 + in-progress

- **Archivos:** `docs/dev/tasks/MEMG-08.md` (nuevo), `docs/dev/Backlog.md` (fila FIND-298)
- **Acción:** este archivo + fila FIND-298 (html/pdf/docx restantes — findings.md: "nace como fila EN EL MOMENTO del discovery"; ID 298 porque 296/297 están reservados por MEMG-03 in-flight, ver §Notas) + `campaign_update_task_state(in-progress, taskId 52)`.
- **Verify:** task file existe; fila en Backlog; recitation actualizada.
- **Evidencia:** [en progreso]
- **Estado:** ⬜ PENDING

### Step 1 — RED: test de integración e2e (3 formatos + proveniencia + budget + idempotencia)

- **Archivos:** `tests/wiki_ingestors.rs` (nuevo)
- **Acción:** test `scan_ingestable_sources` e2e con `tempdir` + `Embedded::open`: (1) root con `notes/a.txt`, `data/b.json`, `tables/c.csv` (+ `skip.md` y binario que NO deben entrar) → scan → chunks con `source`/`page`/`chunk`; (2) `put_batch` → `get` devuelve payload + metadata; (3) `search` text_query encuentra el chunk y su proveniencia; (4) re-scan → mismas keys (idempotencia); (5) budget: archivos grandes → total payload == 28_000 exacto + truncación del último. Compile-error = RED (módulo inexistente).
- **Verify:** `cargo nextest run --profile audit -p vantadb --test wiki_ingestors --build-jobs 2` → **RED esperado** (no compila: `vantadb::wiki::scan_ingestable_sources` no existe).
- **Evidencia:** ✅ **RED EJECUTADO** — `error[E0432]: unresolved imports vantadb::wiki::default_ingestors, vantadb::wiki::scan_ingestable_sources — no `scan_ingestable_sources` in `wiki`` (falla por la razón correcta: API no existe). Nota: el bin `vanta-cli.exe` está lockeado por el server MCP de la sesión (PID 35608) → los comandos `-p vantadb` usan `--no-default-features --features "arrow,fjall,roaring,advanced-tokenizer,memmap2,fs2,sysinfo,rayon"` (= default menos `cli`, sin construir el bin; fidelidad total — mejor que el recorte minimal de MEMG-03).
- **Estado:** ✅ COMPLETED

### Step 2 — GREEN: refactor `sources.rs` + `ingestors.rs` + exports

- **Archivos:** `src/wiki/sources.rs` (extraer `collect_text_files`; `scan_local_sources` re-escrito sobre él, comportamiento idéntico), `src/wiki/ingestors.rs` (nuevo: trait + 3 structs + registry + scan), `src/wiki/mod.rs` (exports)
- **Acción:** mínimo código para el Step 1. `collect_text_files(root, accept, visit) -> Result<()>` `pub(crate)` con el walk actual (sort determinístico, guard, skip); `scan_local_sources` = accept `.md` + budget/truncación actual; `scan_ingestable_sources` = accept de extensions del registry + `ingest` por archivo + budget post-chunk. `ponytail:` notas.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test wiki_ingestors --build-jobs 2` → GREEN + `cargo nextest run --profile audit -p vantadb wiki:: --build-jobs 2` → tests de `sources.rs`/`chunker.rs` verdes (sin regresión) + `cargo check -p vantadb`.
- **Evidencia:** ✅ **GREEN** — `wiki_ingestors`: **4/4 passed** (proveniencia 3 formatos + round-trip put/get/search + budget exacto 28_000 + root error). `--lib -E "test(wiki)"`: **32/32 passed** (sources/chunker/ingestors/store, 2272 skipped) → refactor sin regresión. Incidente resuelto: el e2e con `Embedded::open` falla bajo feature-set recortado (backend Fjall gated) → el test usa `BackendKind::InMemory` (patrón `md_import.rs`), independiente de features. Nota: con `--no-default-features` pelado el text-search del fallback tokenizer no encuentra hits (comportamiento pre-existente del feature-set, NO del código — con el feature-set default-menos-cli el test pasa); registrado como NOTICED en §Notas.
- **Estado:** ✅ COMPLETED

### Step 3 — RED→GREEN: unit tests de edge cases (overlap, dispatch, skips, subdirs)

- **Archivos:** `src/wiki/ingestors.rs` (tests inline, convención del repo)
- **Acción:** tests: (a) `chunks_carry_mandatory_provenance` (3 formatos, page=0, chunk=0..n); (b) `large_text_chunks_with_overlap` (chunk[i+1] empieza con la cola de 400 de chunk[i]; cada chunk ≤ 12k); (c) `unknown_extension_is_skipped`; (d) `empty_content_produces_no_chunks`; (e) `dispatch_picks_ingestor_by_extension` (registro explícito con fake ingestor); (f) `subdirectory_paths_stay_forward_slash`. RED primero donde aplique (los tests de comportamiento nuevo fallan si el código no lo hace).
- **Verify:** `cargo nextest run --profile audit -p vantadb wiki:: --build-jobs 2` → GREEN.
- **Evidencia:** ✅ **8 unit tests verdes** en `ingestors.rs` (proveniencia por formato ×2, overlap+cobridad de párrafos, skip de extensión/vacío, dispatch por registro fake, subdir forward-slash, determinismo, root error) — incluidos en el run `--lib -E "test(wiki)"` **32/32**. Los asserts están acoplados al comportamiento (budget total==28_000 exacto; overlap `starts_with(tail)`; keys estables), no a detalles internos.
- **Estado:** ✅ COMPLETED

### Step 4 — REFACTOR + rustdoc + task file sync

- **Archivos:** `src/wiki/ingestors.rs`, `src/wiki/sources.rs`, `docs/dev/tasks/MEMG-08.md`
- **Acción:** rustdoc completo (módulo: contrato + proveniencia + budget + orden de formatos; items públicos); revisar naming/stuttering (Apéndice V.3); sync del task file (evidencias).
- **Verify:** `cargo doc -p vantadb --no-deps` (sin warnings nuevos del módulo) + tests verdes.
- **Evidencia:** ✅ `cargo fmt -p vantadb -- --check` exit 0 (rustfmt aplicado solo a `ingestors.rs` — 3 diffs; los archivos de MEMG-03 intactos). ✅ `cargo clippy -p vantadb --all-targets -- -D warnings` **Finished sin warnings** (default features — check no linkea, el lock del bin no aplica). ✅ `cargo doc -p vantadb --no-deps --lib`: 1 warning propio detectado y corregido (link intra-doc a item privado en `sources.rs:13` → texto plano); re-run: **9 warnings, todos pre-existentes ajenos a `src/wiki`** (cli_handlers/config/error/wal/storage). ✅ Naming sin stuttering (Apéndice V.3): `Ingestor::extensions`, `scan_ingestable_sources`, `chunk_key` (privado). **Docs (decisión §Spec #9 confirmada):** Rust public API → **rustdoc** (tabla de ownership de `documentation-skill` §6: "Rust public API | rustdoc | cargo doc en ci-rustdoc.yml"); el cambio es core-only (sin binding — DIST-03 declara wiki core-only), `docs/api/` no tiene página de `vantadb::wiki` (gap pre-existente MEM-28/29, NOTICED) y `validate-docs-coverage.ps1` no escanea `src/wiki/`.
- **Estado:** ✅ COMPLETED

### Step 5 — Verificación de cierre + review P2-01 + commit + campaign

- **Archivos:** todos los tocados
- **Acción:** `campaign_verify_cmd`: fmt + clippy + nextest scoped (`-p vantadb` suite wiki + test nuevo) + OCR delegation + review P2-01 (`vanta-review` fork o degradado) + commit LOCAL `feat(wiki):` con pathspec + `campaign_update_task_state(completed, taskId "52")` + `skill progreso`.
- **Verify:** contrato completo verde; RESULTADO §7.
- **Evidencia:** ✅ Verify full: fmt exit 0 · clippy `-D warnings` sin warnings · `wiki_ingestors` **4/4** (×3 feature sets: default-menos-cli, fjall-only [post-hardening], default en hook pre-commit) · wiki lib **32/32** · `vanta-memory` ingest **15/15** · rustdoc 0 warnings propios · gates docs ✅ · OCR delegation 0 Critical/High · **review P2-01 APPROVE (ronda 1)** con fold M1/L1/L2/L3/N1 re-verificado (4/4 ambos feature sets + public-api compare 1/1). ✅ Commit feat **`97039053`** (LOCAL, pathspec 5 archivos; hook pre-commit fmt+clippy+actionlint ok). ⏳ Commit docs (este task file + hunk propio de Backlog + index/llms) + campaign ACCEPT (taskId 52) + `skill progreso`.
- **Estado:** ✅ COMPLETED

## Verificación

- `cargo fmt -p vantadb -- --check` — ✅ exit 0 (rustfmt aplicado solo a `ingestors.rs`; archivos de otros sin tocar)
- `cargo clippy -p vantadb --all-targets -- -D warnings` — ✅ **Finished sin warnings** (default features; check no linkea → inmune al lock del bin)
- `cargo nextest run --profile audit -p vantadb --test wiki_ingestors --build-jobs 2` — ✅ **4/4 passed** (feature-set default-menos-cli; ver §Notas)
- `cargo nextest run --profile audit -p vantadb --lib -E "test(wiki)" --build-jobs 2` — ✅ **32/32 passed** (sources/chunker/ingestors/store — sin regresión del refactor)
- `cargo nextest run --profile audit -p vanta-memory --test ingest --build-jobs 2` — ✅ **15/15 passed** (consumidor de `scan_local_sources`: worker + auto_sync sin regresión)
- `cargo doc -p vantadb --no-deps --lib` — ✅ 0 warnings de `src/wiki` (1 propio corregido; 9 pre-existentes ajenos)
- Gates docs — ✅ check-links exit 0 · check-docs exit 0 · gen-index `--write` aplicado (`docs/index.md` +1 fila MEMG-08, `llms.txt` +1 línea)
- OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) — ✅ Rule Group extraído (group 1 `**/*.rs` → los 4 archivos); self-review: **0 Critical/High** (sin unsafe/deps/unwrap en prod; guard traversal reusado y testeado; logs de skip/budget presentes; `as i64` de `chunk_index` acotado por tamaño de contenido)
- Review P2-01 por agente distinto — ✅ **APPROVE** (ronda 1, `vanta-review`, sesión fresca — ver §Review P2-01; M1/L1/L2/L3/N1 foldados en la misma ronda y re-verificados)
- `tests/api/public-api.txt` (snapshot HARD-01) — ✅ refrescado en este run (+101/−0: módulo `ingestors` + trait + consts + fns; cero cambios ajenos); compare re-ejecutado 1/1 ✅

## Review P2-01 (ronda 1 — `vanta-review`, agente distinto, contexto fresco) — respuesta del autor

**Revisor:** subagente `vanta-review` (leaf, sesión nueva sin participación en la implementación). **Enfoque:** RBI aplicado inline (red-team → brainstorm → iterate) + re-ejecución de TODOS los comandos del contrato (4/4 `wiki_ingestors`, 32/32 wiki lib, 15/15 `vanta-memory` ingest, clippy `-D warnings` limpio, fmt exit 0, gates docs exit 0, rustdoc 8 warnings ninguno en `src/wiki`) y lectura del refactor contra `git show HEAD:src/wiki/sources.rs` (equivalencia línea a línea). **Veredicto ronda 1: `approve`** con 1 Medium + 5 Low + 1 NIT — respuestas:

**M1 (Medium) — snapshot de API pública (`tests/api/public-api.txt`) no refrescado (HARD-01). FIX APLICADO.** El changeset agrega superficie pública (`vantadb::wiki::ingestors`) y el job `public-api-snapshot` (PRs a `main`) falla hasta refrescar el golden file. Refresh ejecutado con el nightly local (warm, 38s): `$env:VANTADB_PUBLIC_API_UPDATE=1; cargo nextest run -p vantadb --no-default-features --features "arrow,fjall,roaring,advanced-tokenizer,memmap2,fs2,sysinfo,rayon" --test public_api --run-ignored ignored-only` → **+101/−0 líneas** (módulo ingestors + 3 structs + trait + 3 consts + `default_ingestors`/`scan_ingestable_sources`); compare mode re-ejecutado sin la env: ✅ 1/1 (9.7s). Nota PR (queda_pendiente lead): el `adr-gate` (`.github/workflows/ci-rust.yml:164+`) exige ADR o `[no-adr]` para PRs que tocan `src/` — decisión del lead en el PR develop→main.

**L1 (Low) — caracterización imprecisa del NOTICED de feature-set + assert de search sensible a features. FIX APLICADO.** El reviewer aisló el discriminador real: no es "text-search sin hits" en general sino el **token compuesto** (`needle-topic`); queries de palabra simple funcionan en ambos tokenizers (`memory_api_filters` pasa con `--features fjall`). Hardening: el assert de search usa `"needle"` (token simple, comentario en el test) → `wiki_ingestors` ahora **4/4 con `--features fjall` pelado** (antes 3/4 en ese set) y 4/4 con el set canónico. Nota de §Notas corregida.

**L2 (Low) — keys >512 bytes sin guard ni mención. FIX (doc).** Cláusula de rustdoc en `Ingestor::ingest`: el boundary de escritura rechaza keys >512 bytes y `put_batch` valida el batch entero → un rel_path que empuje la key sobre el límite hace fallar el write. Guard en scan no agregado (decisión declarada: el consumidor es dueño del `put`; re-evaluable con consumidor real — NOTICED).

**L3 (Low) — extensiones mayúsculas ignoradas en silencio. FIX (doc).** `extensions()` documenta matching **exact-case** (`NOTES.TXT` no matchea `txt`) — paridad declarada con el scanner `.md` pre-existente.

**L4 (Low) — ciclos de symlinks de directorio recursan sin límite. NO INTRODUCIDO (pre-existente).** `walk_files` idéntico a HEAD (`meta.is_dir()` sin ciclo-guard); el refactor lo preserva. Fuera del scope de MEMG-08; requiere árbol hostil → NOTICED en §Notas (sin FIND: pre-existente y sin impacto conocido).

**N1 (NIT) — el log de truncación también disparaba en el fit exacto (`len == budget`). FIX APLICADO.** `truncated = len > budget` capturado antes de agotar el budget; el log distingue y `take` solo corre cuando hay corte real (comportamiento de emisión idéntico — probado por el test de budget exacto).

**P1 (Low, proceso) — `docs/dev/Backlog.md` con WIP de MEMG-09 mezclado (FIND-299/300 + edición de FIND-196). MITIGADO (por exclusión).** La fila FIND-298 **permanece en el working tree sin commitear** — mismo patrón que MEMG-03/06 (la fila nace en disco al discovery; el fold lo hace quien cierre `Backlog.md`). Se intentó staging parcial del blob (`git update-index --cacheinfo`) pero la consola corrompía el encoding del blob (mojibake) → intento **abortado** con `git restore --staged` (working tree verificado intacto). Decisión: NO stagear `Backlog.md` (WIP ajeno adyacente en el mismo hunk).

**Sin hallazgos abiertos bloqueantes.** L4 aceptado (pre-existente); L2/L3 resueltos como doc; M1/L1/N1 verificados por re-ejecución.

## Notas de entorno (2026-10-05)

- **Árbol compartido** — MEMG-03 cerró durante el run (commits `e41fef7d` → `e6efbc69`; sus archivos ya commiteados, sin solape). MEMG-09 apareció después en la misma tree (`docs/dev/tasks/MEMG-09.md`, `vantadb-mcp/src/{code_index.rs,lib.rs,handlers/tools.rs}`, research `mgr-22/23-24`, `vantadb-mcp/tests/code_index_tests.rs` + filas FIND-299/300 y edición de FIND-196 en `Backlog.md`) → TODO su WIP queda sin stagear; `Backlog.md` **no se stagea** (fila FIND-298 en disco sin commitear — patrón MEMG-03/06). `opencode.jsonc` y el master plan (recitations del campaign tool) siguen con WIP ajeno → NO se stagean.
- **Bin lock:** `target/debug/vanta-cli.exe` lockeado por el server MCP de la sesión (PID 35608). Workaround superior al de MEMG-03: `--no-default-features --features "arrow,fjall,roaring,advanced-tokenizer,memmap2,fs2,sysinfo,rayon"` = **default menos `cli`** (el bin tiene `required-features=["cli"]` → no se construye; fidelidad total del feature-set). clippy/fmt/public_api no linkean el bin → corren con default features.
- **Feature-set y text-search (caracterización corregida por review L1):** con `--features fjall` pelado el discriminador NO es "search roto" sino el **token compuesto** (`needle-topic`); con token simple el fallback matchea. El test usa `"needle"` → **4/4 en ambos feature sets** (default-menos-cli y fjall-only). CI usa default+extras (no afectado).
- **IDs FIND:** Backlog al 295; MEMG-03 cerró sin plegar sus propuestas 296/297; MEMG-09 usó 299/300 en su WIP → este run usó **FIND-298** (sin colisión con ninguno de los dos).
- **Regla `-p`:** iteración siempre `-p vantadb`/`-p vanta-memory`; `--workspace` solo cierre/certificación (diferido al lead — árbol compartido).

## FINDs

- **FIND-298 (creada en este DISCOVERY, fila en Backlog):** MGR-25 restante — ingestores html/pdf/docx fuera del corte (txt/json/csv entregados e2e). Origen: MEMG-08 · stop L1503.

## Resultado §7

- **activeGoal:** MEMG-08 (Task 52) — MGR-25: trait `Ingestor` + formatos e ingestores; stop L1503 → trait + txt/json/csv e2e + FIND de html/pdf/docx.
- **result:** OK.
- **lastAction:** Steps 0-5 COMPLETED: task file + FIND-298; RED del e2e; GREEN (refactor `collect_text_files` + `ingestors.rs` + exports); 8 unit tests de edge cases; fmt/clippy/rustdoc/docs-gates/OCR verdes; **review P2-01 APPROVE (ronda 1, `vanta-review` fresco) con fold M1/L1/L2/L3/N1**; snapshot `public-api.txt` refrescado (+101/−0); commit LOCAL `97039053` con pathspec.
- **contract:**
  - `verificacion`: `cargo nextest run --profile audit -p vantadb --test wiki_ingestors` ✅ 4/4 · `-p vantadb --lib -E "test(wiki)"` ✅ 32/32 · `-p vanta-memory --test ingest` ✅ 15/15 · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ · `cargo fmt --check` ✅.
  - `evidencia` (por claim):
    - claim: "trait `Ingestor` formato → chunks `MemoryInput` con proveniencia obligatoria `source`/`page`/`chunk`" · evidencia: `src/wiki/ingestors.rs` (trait + método provisto + 3 structs + registry), test `scans_supported_formats_into_chunks_with_mandatory_provenance` (3 formatos, page/chunk) · confianza: alta
    - claim: "txt/json/csv end-to-end (scan → put_batch → get/search)" · evidencia: `tests/wiki_ingestors.rs::ingested_chunks_round_trip_through_the_engine_and_are_searchable` ✅ · confianza: alta
    - claim: "respeta `SOURCE_CHAR_BUDGET` (28k) con truncación declarada + chunking overlap (400)" · evidencia: `scan_respects_source_char_budget_and_declares_truncation` (total == 28_000 exacto), `large_text_chunks_with_overlap_and_covers_all_content` (cola 400 + cobertura) · confianza: alta
    - claim: "`scan_local_sources` sin regresión (refactor `collect_text_files`)" · evidencia: 32/32 wiki lib (incluye los 7 tests de sources.rs) + 15/15 vanta-memory ingest · confianza: alta
    - claim: "cero deps nuevas / cero wire / cero unsafe" · evidencia: `git diff` de la tarea (sin Cargo.toml, sin unsafe) · confianza: alta
    - claim: "html/pdf/docx → FIND-298 (stop L1503)" · evidencia: `docs/dev/Backlog.md` (fila FIND-298) · confianza: alta
  - `artefactos`: `src/wiki/ingestors.rs` (nuevo), `tests/wiki_ingestors.rs` (nuevo), `src/wiki/sources.rs` (refactor), `src/wiki/mod.rs` (exports), `tests/api/public-api.txt` (snapshot +101/−0), `docs/dev/tasks/MEMG-08.md`, `docs/dev/Backlog.md` (FIND-298), `docs/index.md` + `llms.txt` (generados)
  - `invariantes`: `scan_local_sources` firma+semántica idénticas; proveniencia obligatoria por chunk; budget ≤ 28k; sin tocar wal/vector/storage; sin deps/unsafe; WIP ajeno intacto; commit con pathspec.
  - `deuda`: `ponytail:` scan single-thread secuencial (upgrade: paralelizar si aparece consumidor); keys stale tras shrink de archivo (sin contrato de borrado); NOTICED: gap docs/api del módulo wiki (pre-existente), text-search en feature-set pelado.
  - `queda_pendiente`: ACCEPT campaign (taskId 52 — el WIP=3 bloqueó la transición `in-progress` con recitation al inicio del run, la tarea ya estaba ⏳; el cierre intenta `completed` directo) + `skill progreso` (avance + fila Backlog) + fold del plan (orquestador) + **`adr-gate` del PR develop→main** (ADR o `[no-adr]` para `src/` — decisión del lead; el snapshot public-api YA está refrescado) + pliegue de FIND-298 (queda en el working tree sin commitear — WIP de MEMG-09 adyacente en `Backlog.md`; mismo patrón MEMG-03/06).
- **nextTask:** MEMG-09 (Task 53) — o el próximo que decida el orquestador.
