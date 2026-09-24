# AUDREP-01: Storage-Panic — compact_layout panic sobre datos truncados

## Metadata
- **Plan file:** (P13 backlog, verificado 2026-08-05)
- **Creado:** 2026-08-05
- **last-synced:** 2026-08-05
- **Estado:** ✅ COMPLETED
- **Severidad:** 🔴 CRÍTICO

## Objetivo
Prevenir panic fatal del proceso en `compact_layout` cuando el vstore tiene un
header cuyo `vector_len` reclama más bytes de los reales (vstore truncado por
crash mid-write). Devolver `VantaError` en vez de panic.

## Blast Radius
- **Archivo:** `src/storage/archive.rs:111-116` (compact_layout)
- **Callers:** hay que confirmar quién llama `compact_layout` — búsqueda con
  `codegraph_explore "compact_layout"` antes de editar.
- **Riesgo:** panic hoy tira TODO el proceso (FATAL). Fix = guard + error.
- Sin cambios de API pública.

## Contrato
`cargo check -p vantadb` + un test de regresión que trunca un vstore y
verifica que `compact_layout` devuelve `Err`: `n` en vez de panic.

## Herramientas
- codegraph (callers), cargo-mcp (check/test), rust-analyzer-mcp

## Steps
### Step 1: discovery + fix
- **Archivos:** `src/storage/archive.rs`
- **Acción:** identificados callers vía `codegraph_explore "compact_layout"`:
  `benches/vfile_search.rs:125`, `src/storage/engine/maintenance.rs:470`
  (`compact_layout_bfs`), tests en `src/storage/archive.rs` mod tests. Ningún
  caller espera panic; todos ya propagan `Result`. Antes de la copia se valida
  `src_end > old_data.len()` → `Err(VantaError::IoError(UnexpectedEof,
  "vstore truncated: ..."))`. Comportamiento normal intacto.
- **Verify:** `cargo check -p vantadb` ✅
- **Estado:** ✅ COMPLETED

### Step 2: test de regresión
- **Archivos:** `src/storage/archive.rs` (mod tests)
- **Acción:** `test_compact_layout_truncated_vstore_errors_not_panic` — escribe
  header con `vector_len = 100_000` (reclama ~400KB) en un vstore de 4096 bytes
  y assert que `compact_layout` devuelve `Err` con mensaje "truncated" (sin
  panic).
- **Verify:** `cargo nextest run -p vantadb --lib --profile audit compact --build-jobs 2`
  → 30 passed ✅
- **Estado:** ✅ COMPLETED

### Step 3: colaterales + dump
- clippy -D warnings ✅, fmt --check (archive.rs limpio; drift pre-existente en
  archivos no tocados). Commit atomic conventional
  `fix(AUDREP-01): validate truncated vstore before compact copy`.
- **Estado:** ✅ COMPLETED — ver commit hash en Context Save Point.

## Dependencias
- Ejecutado en el MISMO agente que AUDREP-04 (mismo archivo, cambios adyacentes —
  ahorra doble build y evita conflicto de archivo compartido).

## Notas
- Recomendación del audit report: `source.verify(h)::sync path to state`
- No tocar API pública.

## Context Save Point
- **Fecha:** 2026-08-05
- **Branch:** develop (working tree sucio en src/ — archivo objetivo `archive.rs`
  NO listado en `git status`, así que tocar coincide con un-archive limpio)
- **CI pendiente:** sí (post commit)
- **Decisiones:** agrupar AUDREP-01 + AUDREP-04 en un solo vanta-worker.
- **Commit:** `fix(AUDREP-01): validate truncated vstore before compact copy` — hash en git log
- **Próxima tarea:** AUDREP-04 en el mismo agente.