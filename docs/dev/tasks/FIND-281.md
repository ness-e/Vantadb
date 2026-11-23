---
id: FIND-281
title: "Preservar confidence del motor al re-persistir registros L1 contradichos en l1_writer.rs"
state: in-progress
route: vanta-worker
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 22)
---

# FIND-281 — Preservar `confidence` al re-persistir registros L1 contradichos

## Objetivo
Un registro L1 contradicho no debe "resucitar" con `default_confidence()` (1.0) al
re-persistirse: la ruta de contradicción conserva exactamente la `confidence` y la
`confidence_class` que el motor tiene almacenadas para esa clave.

## Spec

| Decisión | Valor | Justificación |
|----------|-------|---------------|
| Dónde vive la confianza | En el registro del motor (`sdk::MemoryRecord.confidence`), NO en el `MemoryRecord` L1 (`types.rs:71`) | El payload L1 no tiene campo de confianza |
| Mecanismo | `record_input` recibe `confidence_override: Option<(ConfidenceClass, f32)>`; `None` = comportamiento actual | Diseño decidido por owner; mínimo dif |
| Firma pública | `put_record` intacta; helper interno `put_record_with_confidence` para la ruta de contradicción | No romper callers (dream, rollback, cascade_forget, tests) |
| Lectura | `db.get(ns, &sanitize_key(target_id))` antes de re-persistir; si `Err`/`None` → `tracing::debug!` + comportamiento actual | Carrera benigna, fail-open documentado |
| Rango `[0,1]` | Garantizado por el motor en write boundary (`src/sdk/serialization/mod.rs:764`, `api/memory.rs:345`) | No añadir clamp redundante |
| Sin penalización nueva | Solo preservar; ningún decay por contradicción | Fuera de alcance explícito |

## Causa raíz (verificada @ 69662a0bc)
`record_input` (`l1_writer.rs:598-599`) fija siempre `ConfidenceClass::Asserted` +
`default_confidence()`. `mark_contradicted_targets` (`:392`) re-persiste el registro
viejo por esa misma vía → la confianza almacenada se sobrescribe con 1.0.

## Impacto mapeado (Regla 0)
- Leídos completos: `vanta-memory/src/core/record/l1_writer.rs` (1031L), `src/sdk/types/record.rs` (campos 212-217, 372-376), `tests/l1_contradiction.rs`.
- Referencias entrantes a `put_record`: `persist_planned` (mismo fichero), `dream/mod.rs:970`, `rollback.rs:120`, `cascade_forget.rs:162`, `tests/gdpr_cascade_forget.rs:70` → firma pública intacta.
- `record_input` solo llamado por `put_record_with_confidence`/`apply_dedup_batch` (`persist_planned` entra vía `put_record`) → todos pasan `None`.
- Veredicto: blast radius = 1 fichero de producción + 1 fichero de tests. Sin API pública nueva.

## Contrato
`cargo nextest run --profile audit -p vanta-memory -E "test(contradiction) or test(confidence) or test(l1_writer)" --locked --build-jobs 2`

## Steps
- [x] DISCOVERY: task file + spec + blast radius
- [x] RED: `contradiction_repersist_preserves_stored_confidence` (Fjall+reopen) FALLÓ contra HEAD: `left: 1.0, right: 0.3` (razón correcta)
- [x] RED: `new_record_still_gets_default_confidence` PASS contra HEAD (línea base positiva)
- [x] GREEN: `confidence_override` en `record_input` + `put_record_with_confidence` + lectura `db.get` en `mark_contradicted_targets`
- [x] VERIFY: contrato 20/20 PASS; clippy 0 warnings; fmt OK contrato verde + `cargo clippy -p vanta-memory --all-targets --locked -- -D warnings` + `cargo fmt --check -p vanta-memory`
- [x] Review P2-01 APPROVE (vanta-review, fresh context) (vanta-review, fresh context)
- [ ] Commit local `fix(memory): preserve stored confidence when re-persisting contradicted L1 records (FIND-281)`

## Notas
- SDP: `systematic-debugging`, `test-driven-development` (registrado en plan Task 22).
- El test de reopen usa Fjall+tempdir (dev-dep `vantadb` con `fjall` habilitada — precedente `tests/scheduler_crash.rs`).
