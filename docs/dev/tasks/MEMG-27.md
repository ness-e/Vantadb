# MEMG-27 — L1Writer flush write-first (eliminar ventana de pérdida)

- **Estado:** IN PROGRESS
- **Plan:** docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Wave 2)
- **Origen:** plan Wave 2; depende de FIND-221 (put_batch group-commit, MEMG-11)
- **Prioridad:** 🔴 · **Esfuerzo:** 🟡 2h · **Appetite:** 4h · **Cynefin:** 🟨 complicado
- **SDP:** systematic-debugging, test-driven-development, doubt-driven-development

## Problema

`apply_dedup_batch` (vanta-memory/src/core/record/l1_writer.rs) borra las claves
update/merge ANTES de escribir el estado consolidado (Phase 2 `db.delete` →
Phase 3 `put_batch`). Un fallo entre fases destruye memorias L1 previas sin
dejar reemplazo — ventana de pérdida en el sustrato cognitivo.

## Solución (write-first)

1. Phase 2 → `put_batch(new_records)` PRIMERO. Si devuelve `Err`, cero deletes
   ejecutados: los registros previos sobreviven intactos.
2. Phase 3 → borrar solo `stale = old_keys \ new_batch_keys`: una clave de
   target que coincide con una clave del lote ya fue sobrescrita
   atómicamente por `put_batch`; borrarla destruiría el registro recién
   escrito (la trampa de la diferencia de conjuntos).

### Garantía de put_batch (verificada en src/sdk/api/memory.rs:1290-1349)

- Validación upfront all-or-nothing (shape + dimensión FIND-411 + cuota MEMG-04).
- Commits por chunk (`batch_size`, default 1000): un flush L1 típico es un solo
  chunk = un WAL `batch_append` + KV `write_batch`. Con >1000 inputs un fallo
  mid-chunk deja chunks previos commiteados PERO cero deletes (Err) → viejo y
  nuevo coexisten, nunca pérdida.

## Impacto mapeado (Regla 0)

- Leído completo: `vanta-memory/src/core/record/l1_writer.rs` (669 líneas)
- Entrantes (`apply_dedup_batch`): `services/pipeline_worker.rs:561,590`, `core/record/l1_dedup.rs:236` — firma inalterada, sin cambios para callers.
- `persist_planned` (path secuencial `write_memory`) conserva delete-then-put: ventana análoga pero fuera de scope (el flush es el batch path). NOTICED, no tocado.
- Veredicto: cambio interno en 1 función + tests en el mismo archivo.

## Contrato

```
cargo nextest run --profile audit -p vanta-memory -E "test(l1_writer) or test(record)" --locked --build-jobs 2
```

## Tests (TDD)

- [RED] `failed_flush_preserves_preexisting_records` — put_batch forzado a
  fallar (dims de vector mezcladas → validación upfront all-or-nothing) →
  registro target previo sobrevive. Hoy FALLA (deletes ya aplicados).
- `coincident_key_survives_write_first_prune` — update cuyo record_id coincide
  con la clave target → el registro nuevo sobrevive (guarda la trampa old\new).
- `merged_orphan_keys_are_pruned` — merge A+B→C: A y B se purgan, C persiste.

## Stop conditions

>4h, >3 fallos en vanta-memory, premisa invalidada.

## Resultado (ejecución 2026-11-08)

- RED: `failed_flush_preserves_preexisting_records` FAIL contra código viejo
  (`got: []` — old1 borrada antes del put_batch fallido). junit: target/nextest/audit/junit.xml
- GREEN: `put_batch` primero + prune `old_keys \ new_batch_keys` (unión global).
- Review P2-01 (vanta-review, contexto fresco): REQUEST CHANGES → resuelto:
  R-1 documentada (upsert vs delete+put diverge en storage version/historial —
  deliberado, más conservador; payload version idéntica). O-1 cubierta con
  `cross_plan_key_coincidence_survives_prune`.
- Final: `cargo nextest run --profile audit -p vanta-memory -E "test(l1_writer) or test(record)" --locked --build-jobs 2` → **83/83 PASS**; clippy -p vanta-memory --all-targets -D warnings ✅; fmt ✅.
- Commits: f993f3a13 (fix principal — NOTA: absorbió ficheros FIND-585 que el
  agente paralelo había stageado previamente; incidente de índice compartido,
  sin amend/reset por política) + 71b1184d1 (doc R-1 + test O-1, solo l1_writer.rs).
- Deuda: homogeneizar `persist_planned` (path secuencial write_memory) a
  write-first; atomicidad real multi-chunk de put_batch (FIND-221 ya abierto).
- Incidente proceso: `git commit` sin pathspec barrió el índice del otro
  agente → usar `git commit -m "..." -- <paths>` cuando hay staging ajeno.
