# MEM-68 — Gate opcional de aprobación de capturas

> **Plan:** `docs/plans/2026-09-09-backlog.md` (Task 3, Wave0)
> **Estado:** ⏳ IN PROGRESS
> **Branch:** develop
> **Tipo:** feature-add (Rust core, slice mínimo)
> **SDP:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design (v2 BUILD; `frontend-ui-engineering` descartada — score ruido, sin `web/` tocado)

## Contrato

`cargo test -p vanta-memory` 0 failed + test cola pendiente→approve/reject ✅ + `cargo clippy -p vanta-memory -- -D warnings` 0

## Spec

| # | Decisión | Opción elegida + evidencia |
|---|----------|---------------------------|
| 1 | Punto del gate | Cola sobre memorias L1 extraídas ANTES de `apply_dedup_batch` (`l1_writer.rs` NO se toca; approve llama a `apply_dedup_batch` con decisiones `Store` por defecto). Evidencia: `l1_writer.rs:241-278` ya defaultea decisiones ausentes a `Store` |
| 2 | Default | `enabled: false` — con gate off el path existente es byte-idéntico (test `default_off_passthrough` como guardia de regresión, pre-mortem #1) |
| 3 | Scope | Solo cola simple en memoria (`Mutex<Vec>` estilo `LocalStateBackend`); SIN threads/worker wiring, SIN persistencia (pre-mortem #2: scope a cola simple) |
| 4 | Superficie MCP | Thin handlers en `gateway/` (patrón `knowledge_handlers.rs:1-17` — validate → store call → typed envelope): `capture_list_pending` / `capture_approve` / `capture_reject` |
| 5 | IDs pendiente | `p_{now_ms}_{seq}` (mismo esquema que `t_{created_at}_{seq}` en `local_backend.rs:193` y `m_{now}_{idx}` en `l1_writer.rs:38`) |
| 6 | Stop condition | Appetite >1d → shippear config + DEFER tool (no alcanzado: slice ~2 archivos nuevos) |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `core/record/l1_writer.rs` (371L), `core/record/mod.rs` (36L), `core/record/l1_dedup.rs` (§1-120), `core/hooks/mod.rs` + `auto_capture.rs` (punto de captura), `services/conversation_hook.rs` (107L, bridge HTTP→L0+L1), `services/pipeline_worker.rs` (§1-120), `gateway/mod.rs` + `knowledge_handlers.rs` (§1-80), `utils/local_backend.rs` (queue/claim pattern), `core/conversation/l0_recorder.rs` (§1-120), `lib.rs`, `Cargo.toml`, `tests/scene.rs` (patrón `open_db` InMemory)
- **Referencias hacia dentro (lo nuevo usa):** `apply_dedup_batch` + `L1Error` (`l1_writer`), `ExtractedMemory`/`MemoryRecord` (`abstractions`), `read_session_records` (`l1_reader`, solo tests)
- **Referencias entrantes (quién usa lo nuevo):** nadie existente — aditivo puro; ningún caller actual cambia
- **Veredicto:** impacto CERO en paths existentes (2 archivos NUEVOS + 2 re-exports de 1 línea). `l1_writer.rs`, `l1_dedup.rs`, worker, hooks: NO tocados. Rollback = borrar 2 archivos + revertir 2 líneas

## Steps

- [x] S0 DISCOVERY — confirmar `l1_writer.rs` existe + punto de captura + patrón tests ✅
- [x] S1 RED — `tests/capture_approval.rs` (default-off passthrough + pendiente→approve + pendiente→reject + unknown-id) — falló con `E0432 approval inexistente` ✅
- [x] S2 GREEN — `core/record/approval.rs` + `gateway/approval_handlers.rs` + re-exports — mínimo código ✅
- [x] S3 VERIFY — `cargo test -p vanta-memory --jobs 2` 0 failed + clippy 0 + fmt ✅
- [x] S4 CLOSE — commit solo archivos propios + lesson + `campaign_update_task_state` completed

## Gate D

No disparado — owner aprobó DO en Gate P del plan 2026-09-09; blast radius 2 archivos nuevos aditivos, contrato mecánico no ambiguo.

## Iteraciones

| # | Acción | Resultado | Herramienta |
|---|--------|-----------|-------------|
| S0 | DISCOVERY: l1_writer + capture point + test pattern | ✅ punto de gate = pre-`apply_dedup_batch` | codegraph + read |
| S1 | RED: `tests/capture_approval.rs` 4 tests | ✅ falla E0432 (módulo inexistente) | cargo test |
| S2 | GREEN: `approval.rs` (queue+config) + `approval_handlers.rs` (3 tools) + 2 re-exports; fixes: serde en `PendingCapture`, import `EmbedFn`, match exhaustivo, `derive(Default)` | ✅ 4/4 pasan | cargo test |
| S3 | VERIFY full: suite 0 failed + clippy 0 + fmt | ✅ (jobs default → link.exe OOM ambiental; `--jobs 2` = perfil audit canónico) | cargo test/clippy/fmt |
