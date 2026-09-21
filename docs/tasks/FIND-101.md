# FIND-101 — `vanta-cli query` doc-vs-fix (INSERT/UPDATE/DELETE por CLI)

- **Objetivo:** `vanta-cli query` abre read-only (`src/cli_handlers/data.rs:212`) → todo IQL mutante falla. Decidir en el mismo slice (doc límite vs abrir read-write) + implementar + `--help` coherente.
- **Contrato:** doc-vs-fix decidido con evidencia + implementado + `--help` coherente + test/smoke del comportamiento decidido.
- **AC:** (1) decisión escrita con evidencia (no gusto); (2) `--help` describe el comportamiento real; (3) test/smoke verde del comportamiento decidido; (4) sin rediseño del CLI IQL (scope-creep prohibido).
- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja · **Wave:** Wave1 tercera (FIND-112 ✅, FIND-113 ✅) · **Next:** FIND-102.
- **Branch:** develop · **Commit:** `fix: FIND-101 — ...` (o `docs:` si ganaba doc).
- **SDP:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design + documentation-and-adrs, systematic-debugging (sugeridas plan). `frontend-ui-engineering` (lifecycle, score 1.00) descartada: sin superficie `web/` en el slice. SKILLS_CARGADAS en RESULTADO.

## Decisión doc-vs-fix: FIX (parse-then-open)

**Gana FIX. Evidencia (no gusto):**

1. `cmd_query` abre **siempre** read-only: `open_database(db_path, true)` (`src/cli_handlers/data.rs:212`).
2. Todo IQL mutante falla ahí: `insert` (`storage/engine/insert.rs:185`), `delete` (`storage/engine/delete.rs:40`) y mantenimiento exigen `ensure_writable()` (`storage/engine/stats.rs:14-27` → `"StorageEngine is read-only; write operation rejected"`).
3. **No hay invariante CLI que exija read-only en `query`.** La convención del CLI es por-comando: `cmd_put` abre read-write (`crud.rs:27`), `cmd_get`/`cmd_list` read-only (`crud.rs:167,269`); `export` read-only (`data.rs:23`), `import` read-write (`data.rs:160`). `query` es mixto (lectura+mutación), así que el modo correcto depende del statement.
4. IQL documentado "for CRUD operations" con 6 statement types (`docs/api/IQL.md:12-25`); HTTP (`POST /api/v2/query`) y MCP (`query_iql` "inserting/mutating") aceptan mutantes → doc-only rompería paridad.
5. `cmd_query` (`data.rs:288-297`) y el REPL del TUI (`tui/repl.rs:129-133`) ya manejan `ExecutionResult::Write` — hoy **brazo muerto** en ambos (TUI también abre read-only, `bin/vanta-cli.rs:270`; NOTICED BUT NOT TOUCHING: TUI fuera de scope, fila separada si se quiere).
6. Siempre-read-write (opción A) regresa: pierde shared-lock, pierde skip de WAL replay (FIND-109: read-only abre DB con WAL truncado donde read-write aborta por ERR-011), crea DB ante typo de path (`init.rs:161`), toma lock exclusivo para un `SELECT`.
7. Por tanto: **parse-then-open** — lecturas (`SELECT`, `FROM`/`MATCH`→`Query`) siguen read-only; mutantes (`INSERT`, `UPDATE`, `DELETE`, `RELATE`, `INSERT MESSAGE`) abren read-write. Mismo parser que el executor (`crate::parser::parse_statement`), misma entrada con `trim_start` → clasificación coherente por construcción. Parseo fallido o LISP `(` → read-only (el executor devuelve el error sin necesitar escritura).

## Spec (decisiones del slice)

| # | Decisión | Opción elegida | Alternativa rechazada + por qué |
|---|----------|----------------|---------------------------------|
| 1 | doc vs fix | FIX parse-then-open | doc-only: contradice IQL.md CRUD + paridad HTTP/MCP + brazos Write muertos |
| 2 | siempre read-write vs parse-then-open | parse-then-open | siempre-rw: regresa lock compartido, WAL-replay-skip (FIND-109), crea-DB-en-typo |
| 3 | dónde clasificar | helper privado `query_is_mutating` en `data.rs` (cero símbolos públicos nuevos) | flag CLI `--write`: fricción UX + Hyrum (el modo se vuelve contrato) |
| 4 | help | `cli.rs` `Query`: 3 líneas doc (reads ro / mutantes rw) | help largo/tutorial: scope-creep |
| 5 | TUI REPL mismo bug | NO tocar (fuera de scope; anotar) | arreglar de paso: viola Scope Discipline |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/cli_handlers/data.rs` (304L: `cmd_query:207-304`, `cmd_export:19-130` ro, `cmd_import:134-203` rw), `src/cli.rs:127-134` (help `Query`), `src/bin/vanta-cli.rs:83-85` (dispatch `Query`), `src/cli_handlers/db.rs:1-25` (`open_database`/`open_embedded`), `src/executor.rs:20-34` (`ExecutionResult`), `:155-190` (`execute_hybrid`/`execute_statement`), `:245-400` (insert/update/delete/relate → `storage.insert/delete` + `Write`), `src/parser/grammar.rs:432-442` (`parse_statement` alt order), `src/parser/mod.rs:1-9` (re-exports), `src/storage/engine/stats.rs:14-27` (`ensure_writable`), `src/tui/repl.rs:98-143` (mismo patrón Write-muerto), `docs/api/IQL.md:12-25` (IQL=CRUD), `tests/cli_tests.rs:466-478` (test vecino `test_cmd_query_empty_db`).
- **Referencias hacia dentro (el slice usa):** `open_database`, `crate::parser::parse_statement`, `crate::query::Statement::{Select,Query}`, `Executor::execute_hybrid`, `ExecutionResult::{Read,Write,StaleContext}`.
- **Referencias entrantes (dependen del cambio):** `bin/vanta-cli.rs:83-85` (solo dispatch, sin cambio de firma); `test_cmd_query_empty_db` + `cli_tests.rs:1452` (reads → siguen ro, deben seguir verdes); completions generadas (PROHIBIDO regenerar a mano).
- **Veredicto:** impacto mínimo (1 helper privado + 1 línea de apertura + help). Sin símbolos públicos nuevos. Sin hot path (`vector/`, `engine.rs` search loop no tocados). Sin trust boundary nuevo (mismo input IQL que ya aceptaba; el modo de apertura no añade validación). Rollback: `git revert` limpio.
- **Prohibidos (intocables):** rediseño IQL, resto del CLI, `src/wal*.rs`, approval/skill/scheduler, spec ingesta FIND-112, `reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/Backlog.md`, plan file (solo recitation), `C:/Users/Eros/.vantadb*`.

## Steps

- [x] **Step 1 — DISCOVERY:** cero-code planning + decisión FIX + task file. Verify: task file existe con Spec + Regla 0.
- [x] **Step 2 — RED:** test `test_find101_query_insert_mutates` falla pre-fix con `Validation { field: "read_only", reason: "StorageEngine is read-only; write operation rejected" }` (razón correcta, bug confirmado).
- [x] **Step 3 — GREEN:** `query_is_mutating` (mismo parser que el executor) + apertura condicional + flush post-`Write` (root cause #2: WAL-buffered invisible a reopen read-only, ERR-050b; `cmd_put` ya flushea en `crud.rs:131`) + help `cli.rs` + test extendido INSERT/UPDATE/DELETE/SELECT verde.
- [x] **Step 4 — VERIFY+CLOSE:** fmt ✅ + clippy `-D warnings` 0 ✅ + `cli_tests` 85/85 ✅ + `--help` smoke ✅ + binario E2E en Temp (INSERT→SELECT→UPDATE→DELETE→SELECT-vacío) ✅ + OCR (ver abajo) + commit + recitation + RESULTADO.

## Verificación (evidencia)

- `cargo fmt --check` — ✅ (limpio tras `cargo fmt`; solo tocó `data.rs` + `cli_tests.rs`)
- `cargo clippy -p vantadb --all-targets -j 2 -- -D warnings` — ✅ 0 warnings
- `cargo test -p vantadb --test cli_tests -j 2` — ✅ 85/85 (84 pre-existentes + 1 nuevo)
- `vanta-cli query --help` — ✅ muestra read-only vs read-write por statement
- Smoke binario Temp (`$TEMP/find101-smoke`, borrado tras el smoke): INSERT ✓ → SELECT 1 record ✓ → UPDATE ✓ → DELETE ✓ → SELECT vacío ✓
- OCR delegation: ver output en cierre (advisory).
- DoD 3 niveles: Correctness (AC 1-4 ✅, runtime verificado binario real, test falla-sin/pasa-con, 85/85 sin regresiones, error paths: parse-fail/LISP → ro + error del executor) · Quality (naming `query_is_mutating`, sin duplicación — reusa `parse_statement`, sin dead code, scope estricto 3 archivos + task file, fmt+clippy ✅) · Integration (firma `cmd_query` intacta, `bin/vanta-cli.rs` sin cambios, reads con mismo modo que antes) · Documentation (help coherente; IQL.md ya documentaba CRUD — sin cambio necesario) · Ship-readiness (sin trust boundary nuevo, sin deuda nueva, rollback `git revert` limpio; Regla 6: deuda neta 0).
- Rollback: `git revert <hash>` (commit aditivo, sin migración).

## Colateral (Gate C → propuesta: fila FIND nueva, no arreglar acá)

- TUI REPL (`src/tui/repl.rs:129-133` + `bin/vanta-cli.rs:270` abre read-only) tiene el mismo bug latente: maneja `Write` pero el engine es read-only → todo mutante falla igual. Fuera de scope (Scope Discipline). Propuesta: fila `FIND-*` nueva (reusar `query_is_mutating` — requeriría hacerlo `pub(crate)`).

## P2-01 (vanta-review, fresh-context)

- **Veredicto: ✅ approve, 0 bloqueantes.** Divergencia parser/executor: no (misma entrada, mismo parser; futura variante `Statement` cae en mutante→rw, fail-safe). Flush gated en `Write`: correcto (ro nunca flushea). Scope: slice = 3 archivos + task file; dirt ajeno (`completions/*`, `Backlog.md` FIND-112, `.opencode`) pre-existente, no commitear. Ponytail ✅.
- Sugerencias aplicadas: help cubre `INSERT MESSAGE` (`cli.rs`). No aplicada (follow-up): test RELATE/INSERT MESSAGE — misma rama negada los cubre por construcción.

## Gates

- P: heredado (owner aprobó set de 9 + alcance en sesión) — no disparado.
- D: evaluado pre-task-file — **no disparado** (3 archivos, sin símbolos públicos nuevos, sin hot path/API pública, contrato no ambiguo, tiny).
- V: si 2 fallas mismo-error en VERIFY → question; sin respuesta → STOP.
- C: al cierre — colaterales (TUI REPL mismo bug) → question arreglar-ahora vs fila FIND nueva (propuesta: fila, fuera de scope).
