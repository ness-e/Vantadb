# FIND-57: Default `worker_count` 4→1 en AsyncIngestionPipeline (Gate D aprobado)

## Metadata
- **Plan file:** — (ejecución directa desde Backlog L220, orden del orquestador 2026-09-03; Gate D aprobado por el usuario al despachar)
- **Creado:** 2026-09-03T00:00
- **last-synced:** 2026-09-03T00:00
- **Estado:** ✅ COMPLETO (2026-09-03, vanta-worker — ejecución reanudada; Steps 1-5 DONE abajo)
- **SDP:** campaign-executor (pipeline) + ponytail full (activo); TDD/incremental/context per base del agente; `concurrency-async.md` leído (el cambio no altera patrones R1-R8 — solo constante de default). Sin candidatos extra justificados para un cambio de 1 constante.
- **Sub-agente:** vanta-worker
- **Área:** `src/ingestion.rs` (+tests) y `docs/operations/BENCHMARKS.md` §13. NADA más. Paralelo: FIND-56 (`vantadb-server/`) y GOV-TK1 (`cli.rs`/`backup.rs`) — no tocar sus archivos.

## Blast Radius (Discovery)

- Únicos callers de `AsyncIngestionPipeline::new`: `benches/ingestion_concurrent.rs:41` (`Some(workers)` explícito) y `src/ingestion.rs:124` (`Some(2)` en test de caracterización). **Ningún caller pasa `None`** → el cambio de default no altera comportamiento medido del bench ni de tests; solo el API público para consumidores externos del feature `async-ingestion`.
- Sin config/builder — el default vive en `src/ingestion.rs:40` (`worker_count.unwrap_or(4).max(1)`) y se documenta en el doc-comment L37.
- Evidencia: RES-03 (BENCHMARKS §13, 2026-09-03): p1 w1=113.5, w2=78.5 (−31%), w4=65.0 (−43%) ops/s; degradación monótona por convoy sobre `insert_lock` global + WAL fsync (no por el canal). FIND-59/FUT-12 son los que atacan el cuello real.
- TDD N/A con rationale: el default `None→1` no tiene superficie observable sin añadir API nueva (getter de worker_count = scope creep YAGNI); el test vecino usa `Some(2)` y no asume 4 → sin tests que actualizar. La prueba del cambio es el contrato mecánico (rg) + la re-medición §13.
- Regla 9: no se optimiza — se retira un default que la medición mostró perjudicial; re-medir post-cambio con el mismo bench.

## Impacto mapeado (Regla 0) — poblada en reanudación 2026-09-03 (cubre Step 1 pre-existente)

- **Archivos leídos completos:** `src/ingestion.rs` (147L, incl. mod tests), `benches/ingestion_concurrent.rs` (123L), `docs/operations/BENCHMARKS.md` §13, `docs/Backlog.md` L220.
- **Referencias hacia dentro (qué toca el cambio):** `worker_count` solo vive en `AsyncIngestionPipeline::new` (`src/ingestion.rs:37,41,45`); sin config/builder; feature `async-ingestion` no entra al build wasm (`vantadb-wasm` = `default-features=false,features=["wasm"]`).
- **Referencias entrantes (callers de `::new`):** solo 2 — `benches/ingestion_concurrent.rs:41` (`Some(workers)` explícito) y `src/ingestion.rs:129` test (`Some(2)`); **ningún caller pasa `None`** → el default solo afecta a consumidores externos del feature.
- **Veredicto:** blast radius = 1 constante + doc-comment; ningún test asume 4 (`rg "= 4" src/ingestion.rs` = 0); bench independiente del default (pasa `Some` explícito) → re-medición válida como re-validación. Fuera de scope declarado: `completions/*`, `.github/workflows/*`, `release-plz.toml` (trabajo paralelo de otros agentes en el worktree — NO tocar, NO stagear).

## Contrato

`rg -n "worker_count" src/ingestion.rs | rg -c "1"` ≥1 Y `rg "worker_count: 4|worker_count = 4|= 4" src/ingestion.rs` == 0 Y `cargo test -p vantadb --lib ingestion --features async-ingestion` 0 failed Y BENCHMARKS §13 con números post-default Y `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 Y `cargo fmt --all -- --check` 0.

Commit: `fix(ingestion): default worker_count 1 - w>4 medido -43% (FIND-57)` — `fix:` → release-plz patch; convención: CHANGELOG managed por release-plz (precedente ERR-TS-01: behavior BREAKING 0.x documentado bajo Fixed) → patch, no `feat!:`. Anotado en cierre.

## Herramientas

- bash (rg, cargo test/fmt/clippy/bench), edit, write

## Steps

### Step 1: Cambiar default a 1
- **Archivos:** `src/ingestion.rs`
- **Acción:** `unwrap_or(4)` → `unwrap_or(1)` (L40); doc-comment L37 "defaults to 4" → "defaults to 1" + nota de degradación apuntando a BENCHMARKS §13; comentario `ponytail:` con techo conocido y upgrade path (FUT-12/FIND-59).
- **Verify:** cláusulas rg del contrato
- **Estado:** ✅ DONE (2026-09-03 — verificado en reanudación: `rg -n worker_count` = L37/L41/L45 con `unwrap_or(1)`; `rg -c "= 4"` = 0)

### Step 2: Verificar tests vecinos sin asunción de 4
- **Archivos:** `src/ingestion.rs` (mod tests)
- **Acción:** rg worker_count en tests — único uso L124 `Some(2)` (test de entrega, no de throughput) → sin cambios, rationale anotado.
- **Verify:** `cargo test -p vantadb --lib ingestion --features async-ingestion` 0 failed
- **Estado:** ✅ DONE (2026-09-03: 1 passed, 0 failed; `Some(2)` sin asunción de 4 → sin cambios)

### Step 3: Re-medición post-cambio
- **Archivos:** — (solo ejecución)
- **Acción:** `cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion -- "p[14]/[14]"` ×2 corridas (celdas workers∈{1,4} × producers∈{1,4}; w=2 ya medido en §13, no se repite).
- **Verify:** números capturados
- **Estado:** ✅ DONE (2026-09-03: 8 invocaciones 1-filtro — criterion solo acepta UN [FILTER]; por corrida = mediana de 11 samples; A: p1/1=114 p1/4=65 p4/1=105 p4/4=62; B: p1/1=109 p1/4=61 p4/1=108 p4/4=62; final p1/1=**111.5** p1/4=63.0 p4/1=106.5 p4/4=62.0; logs en `Temp/opencode/find57-{A,B}-*.log`, no versionados)

### Step 4: BENCHMARKS §13 subsección post-FIND-57
- **Archivos:** `docs/operations/BENCHMARKS.md`
- **Acción:** nueva subsección "post-FIND-57: default=1" con tabla de decisión w1 vs w4 fresca + comando reproducible (Regla 11) + nota de que el bench pasa `Some(workers)` explícito.
- **Estado:** ✅ DONE (2026-09-03: `### Post-FIND-57 re-measurement` en §13 con tabla p{1,4}×w{1,4} + reproduce con 4 invocaciones 1-filtro + lectura: misma forma, default=1 confirmado, 111.5 ≥110 ✅)

### Step 5: Cierre
- **Archivos:** `docs/Backlog.md` (fila FIND-57 eliminada), `docs/avance/activo/core-engine.md` (entrada), memoria (`campaign_memory_write` decisions si aplica)
- **Verify:** clippy workspace all-features 0 + fmt 0; commit con archivos exactos (NO stagear completions/ ni .opencode/ ni stash)
- **Estado:** ✅ DONE con desviación declarada (2026-09-03: fmt 0 ✅; `clippy -p vantadb --lib --all-features -D warnings` 0 ✅; `clippy --workspace --all-targets --all-features` ROJO pre-existente en HEAD — `vanta-memory` 2696 errores + targets stale `tests/integration|sparse_vectors`, `benches/hybrid_queries` con rg ingestion=0, fuera de scope NADA-más → documentado en avance + decisions, queda para el lead; fila FIND-57 eliminada de Backlog — solo queda mención histórica en el contador L17; entrada en `docs/avance/activo/core-engine.md`; decisions.md; commit scoped 2 archivos con mensaje exacto del brief)

## Dependencias
- RES-03 (bench + §13) — landed (quality-gtm-wave 2026-09-03). FIND-59/FUT-12 son upgrade path, no bloqueantes.

## Notas
- El mensaje de commit del brief dice "w>4 medido -43%" — se usa literal por instrucción; la medición real es w=2 −31% / w=4 −43% vs w=1 (anotado, no corregir en secreto).

## Context Save Point
- **Fecha:** 2026-09-03
- **Branch:** (actual, sin cambiar)
- **CI pendiente:** no
- **Decisiones:** default=1 (no "auto") porque el convoy se mide contra el lock serial del motor y ninguna carga de trabajo actual se beneficia; `fix:` patch según convención release-plz del repo.
- **Problemas conocidos:** ninguno
- **Próxima tarea:** —
