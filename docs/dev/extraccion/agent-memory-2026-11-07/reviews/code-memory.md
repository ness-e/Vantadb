---
title: "Revisión adversarial independiente de code-memory"
kind: research
status: proposed
description: "Contraste documental independiente de cuatro productores de grafos de código, sus flujos y deltas frente a VantaDB"
tags: [research, memory, engineering]
---

# Revisión adversarial independiente de code-memory

## Alcance y criterio

Revisión del 2026-11-07, independiente de la autoría de los cuatro informes.
Lectura estática: no ejecución de proyectos, tests, builds, benchmarks, índices,
instalaciones ni servicios; no cambios de runtime, configuración, evaluación o git.
Único archivo autorizado: este documento. No constituye aceptación final.

Se aplican [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md),
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md) y [REVIEW-COVERAGE](../REVIEW-COVERAGE.md).
Vanta HEAD comprobado: `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.
Se contrastaron [manifiesto de clones](../clone-manifest.json) y HEADs locales;
sin fetch. Una fuente leída o un test escrito no acredita resultado ejecutado.
Severidad alta: puede inducir una decisión de integración incorrecta; media:
laguna material del contrato; baja: precisión documental. No hay score agregado.

## codebase-memory-mcp

**Informe:** [codebase-memory-mcp.md](../codebase-memory-mcp.md), leído íntegro,
líneas 1–498. **Veredicto propuesto: cambios requeridos.**
SHA contrastado: `612eee2518ef84cc573e491fdee5fa5c90264cb5`;
clon `C:\Users\Eros\codebase-memory-mcp`.

### Ledger 0 + 10

| Eje | Estado | Sección y evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/flujos | parcial | §1, §11.1–11.2: C/SQLite/FTS5, resolución y closure contrastados; faltan consulta concreta y borrado de proyecto (CM-1) |
| 1. Recomendaciones | sustentado | §1, §12: adapter opcional, sin sustituir retrieval/grafo Vanta ya existentes |
| 2. Fichas Rust/delta | parcial | A–E y §12: alternativas, ownership, rollback, errores y aceptación explícitos; E no identifica caller/job Vanta concreto (CM-2) |
| 3. Incidencias upstream | sustentado | §11.3: #867/#516 y comentarios releídos por API pública; versiones, cierre y límites de PR correctamente diferenciados |
| 4. Prerrequisitos Vanta | parcial | §4, §12: `MemoryInput`/`add_edge` reales, publicación no presumida; falta seam de refresco E |
| 5. No trasladar | sustentado | §5: no daemon/parsers/LSP, ni selección experimental como gate; MIT no absorbe licencias vendorizadas |
| 6. Método/advertencias | sustentado | §6, §11: AST no garantiza dispatch, ID SQLite no es identidad, no ejecución ni ausencia global |
| 7. Benchmarks | sustentado | §7, §11.4: ambos scripts releídos; wall-clock vs monotónico, `?`/0, efectos y denominadores separados de calidad |
| 8. Matriz | sustentado | §8: `code_index`, scanners y grafo propios reconocidos; equivalencia AST/LSP no acreditada, no negada |
| 9. Negativos Vanta | sustentado | §9, §12.4: hash-skip/referencias heurísticas observables; impacto interarchivo rotulado hipótesis, no reproducción |
| 10. UI/escritorio | parcial | §10, §12.4: web/Lens ≠ Tauri; App.tsx:1–132 es principalmente comentario e imports, no flujo React trazado |

### Hallazgos

1. **CM-1 — Media; informe:193, 269, 289–339, 492–495. Consulta y eliminación
   quedan sustituidas por inventario/anotaciones.** El dossier sí explica
   reconstrucción y bajas de archivos, pero no sigue una query ni
   `delete_project`. La fuente concreta
   [mcp.c:5488–5580](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/mcp.c#L5488-L5580)
   resuelve store/proyecto, valida indexado y límites, rechaza BM25+semantic
   simultáneos y puede caer de BM25 al camino regex.
   [handle_delete_project:8054–8150](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/mcp.c#L8054-L8150)
   bloquea mutación/indexado, cierra store, elimina DB y sidecar ADR, distingue
   `delete_failed`/`not_found` y desregistra watcher incluso tras fallo de unlink.
   **Consecuencia:** no queda definido qué éxito/fallo debe preservar el adapter
   ni la diferencia entre retirar proyección, borrar proyecto y borrar ADR humano.
   **Corrección verificable:** documentar esos caminos y sus errores/ownership;
   separar delete upstream de tombstones D y de memorias manuales Vanta.

2. **CM-2 — Media; informe:449–463. Seam de ficha E insuficientemente identificado.**
   `code_index.rs:193–202` prueba comparación de hash, no encolado, ACK o
   recuperación de jobs; “scheduler/jobs y checkpoint existentes” no nombra
   ruta/símbolo/caller ni contrato de persistencia compatible. El código upstream
   citado `watcher.c:64–108` define estado y comentarios; la aplicación efectiva
   del ACK está más abajo (`commit_pending_changes:1632–1638`, llamada de indexado
   en torno a 1733). **Consecuencia:** una tarea posterior podría inventar un
   segundo coordinador o asumir atomicidad fuente/job.
   **Corrección verificable:** seguir caller y frontera de publicación upstream,
   y localizar el puerto Vanta real o marcar explícitamente integración nueva no
   acreditada; conservar la buena FMEA/aceptación, sin implementar nada.

### Fuentes contrastadas y límites

- Local upstream: `LICENSE`; `pass_calls.c:837–891`; `store.c:363–410`;
  `pipeline_incremental.c:4039–4218`; `pipeline.c:3191–3221,4900–4984`;
  `watcher.c:1–108` y localización de asignaciones/llamada posteriores;
  `mcp.c:5488–5645,8054–8150`; `store_impact.h:1–105`;
  `compact_out.c:454–480`; ambos scripts completos de benchmarks.
- `graph-ui/src/App.tsx:1–132` y Studio
  `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:1–90`
  en `0ad38c542b39a589035b0e17fdedb4c09dca24b9`: web e inspector existente,
  sin afirmar navegación/runtime verificados.
- Vanta: CodeGraph devolvió fuente de `code_index::index_repo` y scanners;
  `git show` del snapshot para `src/sdk/api/graph.rs:113–151` y
  `src/sdk/types/record.rs:184–224`. `add_edge` escribe extremos secuencialmente
  y no recibe metadata arbitraria. C12 contrastado en `CONSOLIDATED.md:144–151`.
- Red: skill `coordinated-web-search` cargada y MCP listado; lectura keyless de
  [#867](https://api.github.com/repos/DeusData/codebase-memory-mcp/issues/867),
  [comentarios](https://api.github.com/repos/DeusData/codebase-memory-mcp/issues/867/comments),
  [#516](https://api.github.com/repos/DeusData/codebase-memory-mcp/issues/516) y
  [comentarios](https://api.github.com/repos/DeusData/codebase-memory-mcp/issues/516/comments).
  Acceso exitoso; cierre no se usa como prueba de corrección completa.
- No se contrastaron todos los LSP/parsers, planner closure, publicación completa,
  migraciones, seguridad de transports ni calidad del paper. La disciplina de
  generación propuesta no acredita que C12 o su atomicidad estén implementados.
