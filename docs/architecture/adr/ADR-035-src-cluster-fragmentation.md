# ADR-035: src cluster fragmentation (cohesion 0.59-0.71)

## Status
accepted-pending-owner-review

## Context
**Owner articulation required (Regla 5):** el owner humano debe articular el trade-off entre diversidad deliberada vs consolidación si quiere revertir esta decisión.

**Origen:** FIND-41 (P38, codegraph-20260827-143245 Fase 1, cluster Leiden 6 src cohesion 0.52-0.71 vs 0.82-0.97 en otras areas).

**Evidencia empírica (snapshot 2026-08-30):**
- 6 clusters src cohesion 0.5237-0.7107 (Leiden IDs 15, 33, 49, 74, 58, 17)
- 0 ciclos, 0 god objects, 0 dependencies circulares
- vs 0.82-0.97 en otras areas (skills/desktop)

**Análisis cluster por cluster:**

1. **Cluster 15 (cohesion 0.71):** multi-binding reuso (python/node/wasm/desktop/server) — share primitives
2. **Cluster 33 (cohesion 0.68):** stdlib-first, API canónica por handle (VantaEmbedded builder)
3. **Cluster 49 (cohesion 0.65):** in-memory backend (sqlite::memory pattern) — decisión deliberada
4. **Cluster 74 (cohesion 0.61):** builder sin abstracción, primitives compartidas
5. **Cluster 58 (cohesion 0.59):** boundary bindings core<->desktop (handlers gRPC, IPC, observability)
6. **Cluster 17 (cohesion 0.52):** vector quantization (Binary, Turbo, SQ8) — dominio experimental

## Decision
**Documentar fronteras, NO consolidar.** La fragmentación es **LEGÍTIMA diversidad de dominios compartidos** (multi-binding reuso, primitives cross-crate, builder pattern sin abstracción deliberada), NO un anti-pattern arquitectónico.

## Alternatives considered

1. **Consolidar 6 clusters** — RECHAZADO: riesgo regresión alto, cohesión 0.52-0.71 refleja diversidad intencional. Ponytail rung 1: YAGNI.
2. **Documentar fronteras (este ADR)** — ELEGIDO: ADR + análisis cluster por cluster sin cambiar código. Regla 6 saldo = 0.
3. **Refactor futuro** — DEFER (Q4 si apetito). Mantener como debt sin schedule.

## Consequences

### Positivas
- 0 código nuevo introducido (Regla 6 saldo = 0)
- Documentación archivable del estado actual
- Decisión trazable en el tiempo (Regla 5 cumplido: ADR existe con status)
- cluster IDs Leiden son snapshot — pueden cambiar entre regeneraciones del grafo (riesgo de obsolescencia, documentado)

### Negativas
- Cohesion 0.52-0.71 puede degradar si nuevos features se agregan sin criterio
- Aceptar el estado actual puede enmascarar regresiones futuras (false negative en codegraph alerts)
- Documento de 328 líneas — sobre-mantenimiento si clusters cambian

### Riesgos
- Leiden IDs cambian entre regeneraciones del grafo (documentado)
- "Documentar sin consolidar" puede leerse como YAGNI-excuse (deuda P3 implícita: refactor Q4 si apetito)

## References
- codegraph-20260827-143245 (Fase 1)
- docs/research/FND-06-core-bindings-boundaries
- ADR-034 (sibling: src→skills boundary, falso positivo)
- plan §W25-2
- Backlog.md: FIND-41 (migrado 2026-08-30)
