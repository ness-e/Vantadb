# AUDIT-02: Sparse hot-path micro-opt (gate de medición)

## Metadata
- **Plan file:** — (tarea desde backlog, no plan dedicado)
- **Fuente:** `docs/Backlog.md:73`
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Tipo:** Rust / Performance
- **Turns estimados:** 15-30
- **Creado:** 2026-08-06T00:00
- **last-synced:** 2026-08-06T00:00
- **Estado:** ✅ COMPLETED (2026-08-06, WONTFIX con reporte de medición)

## Contexto (reformulada 2026-08-06)
La premisa original ("sparse_memory_search full-scan") es **FALSA** desde NUEVO-22:
el SparseIndex invertido con posting lists ya está implementado. NO re-hacer el
full-scan. Quedan **2 micro-beneficios candidatos**:

1. **Serialización JSON del sparse** — `src/sdk/serialization/mod.rs:268-282` y
   `338-342`: el payload sparse se serializa como `String` (JSON relacional).
   Candidato: zero-copy / representación numérica directa. (Regla 4: "serialización
   completa donde zero-copy es viable").
2. **`sort_hits` sort completo + truncate** — `src/sdk/search/mod.rs:775-776`:
   ordena TODOS los hits y luego trunca. Candidato: heap parcial o `select_nth`
   para top-k chico.

## Contrato (deliverable = medición, no código)
"Entregar un reporte de medición documentado: bench/flamegraph que atribuye
impacto a serialización-JSON y a sort completo vs sort parcial en el hot-path
sparse con dataset realista. **Decisión clara y verificable**: impacto ≥1% →
propuesta de fix + implementación; impacto <1% → WONTFIX con nota. Si se toca
código, `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb` pasan."

## Herramientas necesarias
- cargo-mcp (check, test, bench, clippy)
- codegraph_explore (blast radius de sort_hits / serialization)
- rust-analyzer-mcp (si hace falta goto-def)

## Investigation Notes
- Regla del gate aplicada idéntica a AUDIT-06/07 fusionadas: **medir primero,
  atribuir ≥1% antes de tocar código**.
- Referencia deuda P2-7 (`src/sdk/serialization/mod.rs:227-294`) se solapa con
  candidato 1 — no duplicar; si aplica, pagar P2-7.

## Steps
1. **Medir (obligatorio)** — bench sparse representativo (criterion `benches/`) +
   flamegraph si está disponible local. Atribuir tiempo en serialización-J vs sort vs
   el resto del hot path.
   - Verify: números reproducibles en `benches/` output
2. **Decidir por gate** — si impacto ≥1% en cualquiera de los 2 candidatos → pasar a
   fix; si <1% → marcar WONTFIX y CERRAR la tarea con reporte.
   - Verify: decisión documentada en el reporte
3. **Fix (solo si gate pasa)** — candidato ganador, diff mínimo (ponytail). sin
   reordenar el full pipeline.
   - Verify: `cargo check -p vantadb` + `cargo nextest run -p vantadb --build-jobs 2`
   - Verify: re-bench confirma la mejora medida
4. **Reporte + commit** — reporte en `docs/Investigaciones/AUDIT-02-*`; commit
   conventional con task ID.

## Dependencias
- NUEVO-22 (ya completado) — resolvió el full-scan; es pre-requisito histórico.
- Ninguna tarea pendiente bloquea.

## Notas
- El deliverable PRINCIPAL es un reporte de medición, NO código. Si el gate decide
  WONTFIX, el task se cierra igual con el reporte como evidencia.