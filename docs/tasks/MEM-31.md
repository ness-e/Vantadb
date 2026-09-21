# Task: MEM-31 — Progreso de ingest: canal interno + polling con run_id

**Plan:** docs/plans/2026-08-21-vanta-proxy-knowledge.md · **Task 8** · **Ruta:** vanta-worker
**Estado:** ✅ COMPLETADA (código verificado; commit pendiente de orquestador) · **Iteración:** 1

## Contrato
`cargo check -p vanta-memory` pasa; tests D19: (a) run_id viejo descartado;
(b) throttle 500ms; (c) summary truncado a límites TDAM (≤100 chars, ≤20 páginas);
(d) el canal nunca bloquea el ingest; (e) fases extracting|merging|indexing con
{total,completed,failed,skipped,percent}; (f) wiki_status(run_id) consultable desde otro handle.
**Resultado:** TODOS los gates exit 0 (ver Verify abajo).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vanta-memory/src/ingest/worker.rs` (194L) — punto de integración; `run()` orquesta begin→body→complete/fail; run_id viene de `begin_processing`.
- `vanta-memory/src/ingest/mod.rs` (234L) — IngestConfig/IngestError/STRUCTURAL_FILES; módulos declarados acá.
- `vanta-memory/tests/ingest.rs` (492L) — fixtures reutilizables (in_memory_engine, ScriptedRunner, source_dir).
- `src/wiki/store.rs` (MEM-28, vía codegraph) — run_id ya persistido por build (`begin_processing`, guard `expect_processing`).

**Referencias hacia afuera (del código nuevo):** solo std (`Arc`, `Mutex`, `Cell`) + serde ya en deps. Sin deps nuevas.

**Referencias entrantes:** `worker::run` llamado por `tests/ingest.rs` — firma preservada; nueva API `run_with_progress` aditiva. `callback.rs` NUEVO sin dependientes previos.

**TDAM refs verificadas (clone @97f9465):**
- `manager.ts:110-121`: IngestProgress {phase, total, completed, failed, skipped, percent}; PROGRESS_THROTTLE_MS=500; phase change siempre emite; misma fase exige percent>prev Y ≥500ms salvo extracting≥90%.
- `wiki-service.ts:1026`: ingestRunId compartido con el worker.
- `callback.ts:128-170`: summary ≤100 chars pedidos al LLM; pageList `.slice(0, 20)`.

**Veredicto de impacto:** BAJO — todo aditivo. Core `vantadb` NO tocado (late-packet guard ya existe en store.rs:219-239); el tracker replica el guard en memoria.

## Steps

1. ✅ `vanta-memory/src/ingest/callback.rs`: IngestPhase/IngestProgress (serde snake_case), ProgressTracker (try_lock → nunca bloquea), `update_progress[_at]` throttle 500ms + filtro run_id activo, `wiki_status(run_id)`, `truncate_summary`(≤100 chars char-safe) + `cap_summary_pages`(≤20). Unit tests D19 a/b/c/d/f inline (8 tests).
2. ✅ Worker: `run_with_progress(..., Option<&ProgressTracker>)`; `run` delega con None. Emite Extracting (inicio + per-source), Merging (inicio + per-page write ok/fail vía Cell counters), Indexing, Done/Failed (terminal). Integration tests (e) y (d/f).
3. ✅ Verify mecánico completo + cierre.

## Verify (todos exit 0)

- `cargo check -p vanta-memory` → exit 0
- `cargo nextest run -p vanta-memory` → 453/453 passed (443 previos + 10 nuevos)
- `cargo fmt --check` → exit 0 (tras `cargo fmt`)
- `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` → exit 0 (fix: checked_mul/checked_div en percent)

## Context Save Point
(nada pendiente — tarea completa)

## Notas de diseño
- Throttle replica TDAM manager.ts:117-121 fielmente: phase-change siempre emite; misma fase exige percent monótono creciente + intervalo 500ms salvo extracting ≥90% (near-end bypass). Terminal Done/Failed siempre visibles (phase change).
- No-bloqueo garantizado por construcción: TODO acceso al estado del tracker usa `try_lock`; bajo contención la actualización se descarta (P4 best-effort). Test unitario con timing bound <100ms.
- Percent derivado en `IngestProgress::new` con checked arithmetic (total=0 → 100 si Done, 0 si no).
- Commit NO realizado (instrucción del orquestador): `git status` contiene callback.rs (nuevo), mod.rs, worker.rs, tests/ingest.rs, task file.
