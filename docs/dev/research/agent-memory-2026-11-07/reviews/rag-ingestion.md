---
title: "Revisión adversarial independiente de retrieval e ingesta RAG"
kind: research
status: proposed
description: "Contraste documental independiente de LanceDB, LlamaIndex, GraphRAG, RAGFlow y Quivr, sin ejecución ni aceptación final"
tags: [research, memory, engineering]
---

# Revisión adversarial independiente de retrieval e ingesta RAG

## Alcance y criterio

Lectura estática independiente del 2026-11-07. Encargo: brief local
`C:\Users\Eros\AppData\Local\Temp\vantadb-research-review-brief.md`,
[contrato](../EXPANSION-CONTRACT.md),
[referencia Vanta](../VANTA-REFERENCE.md), [método](../METHOD.md),
[notas de benchmarks](../BENCHMARK-NOTES.md) y
[cobertura](../REVIEW-COVERAGE.md). Se revisan informes congelados; no se
modifican sus fuentes, índices, contratos, JSON, backlog, código ni Git.
Los números de línea son del informe leído, no de una versión futura.

Se cargaron el catálogo de skills, `documentation-skill`,
`coordinated-web-search`, `doubt-driven-development` y `codebase-memory`.
Se listaron los servidores MCP y los tools de `websearch` y `codegraph`.
Las consultas públicas siguientes usan lectura de fuentes oficiales; no son
reproducciones. No se ejecutaron aplicaciones, tests, builds, benchmarks,
instalaciones, validadores ni servicios; tampoco mantenimiento de grafos.
Las lecturas CodeGraph muestran código en disco, no certifican por sí mismas
identidad byte a byte con el snapshot Vanta de referencia
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.

Estados por eje: **sustentado**, **parcial**, **no acreditado**,
**no aplicable con justificación**. No hay puntuación agregada.
Los veredictos son propuestas al responsable principal, nunca aceptación.

## LanceDB

Informe leído íntegro: [lancedb.md](../lancedb.md), líneas 1–684.
Clon de contraste: `C:\Users\Eros\lancedb`, SHA del manifiesto
`bff93ff314eee34f625d05bd92b73d95867fe787`; incorporación expresa separada
del filtro original, correctamente indicada en §0.1.
**Veredicto propuesto: sin hallazgos bloqueantes en este alcance.**

### Ledger 0+10

| Eje | Estado | Sustento o laguna delimitada |
|---|---|---|
| 0. Identidad, arquitectura y flujos | sustentado | §§0–1 distinguen wrapper/Lance/Cloud, Arrow, row ID/clave/revisión, guards de mutación, materialización híbrida y commit de valores separado del stamp; contraste indicado abajo. No acredita internals Lance ni servidor. |
| 1. Recomendaciones | sustentado | §1 prioriza receipts, frescura, coordenada, proyección y explain; F/G se condicionan y no fuerzan reemplazar motor ni scheduler. |
| 2. Fichas Rust | sustentado | §2 A–G incluyen interfaces expresamente propuestas, delta, seams existentes, alternativas, migración/rollback, FMEA y aceptación adversa. La disponibilidad real de revisión/CAS queda como dependencia, no capacidad inventada. |
| 3. Incidencias | sustentado | §§3.1–3.3: cuerpos oficiales #4844/#4397/#4402 y PR #4428 releídos; Open/Open/Open y Merged respectivamente en esta consulta. El fix cliente no se confunde con validación servidor. |
| 4. Prerrequisitos Vanta | sustentado | §4 separa delete-before-put, RMW y lectura diagnóstica; `persist_planned` confirma delete→put y `mutate` read→write, sin demostrar fallo concurrente. |
| 5. Rechazos | sustentado | §5 rechaza doble almacén, row ID físico como negocio, RRF duplicado y timeout como rollback. |
| 6. Advertencias | sustentado | §6 y §11 no extrapolan wheel, Cloud, LSM/RTE ni lectura de tests a ejecución. |
| 7. Benchmarks | sustentado | §7, ambos runners completos contrastados: hit textual sobre pools y throughput de filas son unidades distintas; denominadores, caché histórica, defaults, abortos y efectos documentados. Procedencia/split del archivo externo sigue desconocida y declarada. |
| 8. Matriz | sustentado | §8 no niega híbrido, sparse, explain, chunks, jobs ni Studio; coordina A–G con capacidades existentes y equivalencias no acreditadas. |
| 9. Negativos propios | sustentado | §9 etiqueta hecho estático L1 y riesgo RMW por separado, sin convertirlos en defectos WAL ni ausencia global. |
| 10. UI/escritorio | sustentado | §10 limita ausencia a muestra upstream y diferencia Cloud de desktop; bridge `vanta.ts:30-90` y `MemoryLens.tsx:1-83` contrastados, sin certificar runtime. |

### Hallazgos

- **Baja — referencia interna inexistente.** Informe `lancedb.md:355`
  remite a «§0.7», pero los flujos terminan en §0.6 y la traza refresh está
  en §1, líneas 159–182. La separación de commits sí está sustentada por
  `table/refresh.rs:411-467`. Consecuencia: navegación defectuosa al justificar
  el fallo del stamp. Corrección verificable: remitir a esa traza de §1
  y conservar enlace SHA al mecanismo. No invalida B/G.
- No se encontró contradicción bloqueante en los mecanismos contrastados.
  La página actual de #4844 muestra además desarrollo enlazado #4851;
  **no se leyó ese PR**, no se infiere merge ni corrección del SHA congelado,
  y «sin PR acreditada» no se transforma en «no existe PR».

### Evidencia contrastada y límites

- Upstream: `table.rs:3821-3928,3997-4010`; `table/merge.rs:337-444`,
  `update.rs:87-130`, `delete.rs:12-61`; `dataset.rs:14-145,198-270`;
  `query.rs:1517-1695`, `table/query.rs:334-425`,
  `rerankers/rrf.rs:19-149`. Confirman guards asimétricos, publicación tras
  `await?`, fusión materializada y RRF por row ID, no QA.
- Derivados e índices: `freshness.rs:4-110,1633-1724`,
  `refresh.rs:287-467`, `index/waiter.rs:16-89`,
  `remote/table.rs:772-812,3676-3743`,
  `python/python/lancedb/remote/table.py:1057-1105`. `verified_version=None`
  omite el fence; waiter por nombre no es watermark causal; estadísticas
  remotas son placeholder y el docstring sync discrepa de la espera Rust.
- Runners completos: `python/benchmarks/bench_typesafe.py:1-245` y
  `bench_streaming_dataloader.py:1-135`. Gold se compara con texto exacto;
  no answerer/juez QA. Cacheados reutilizan latencias previas; excepciones
  abortan antes del agregado completo. Nada se ejecutó ni recalculó.
- Vanta mediante CodeGraph: `src/wiki/ingestors.rs::ingest`,
  `l1_writer.rs::persist_planned`, `checkpoint.rs::mutate`,
  `scheduler.rs::run_pass`, `src/sdk/search/explain.rs::explain_memory_search`.
  Este último efectúa búsquedas internas: no basta invocarlo aparte para
  acreditar traza de la primera búsqueda, riesgo que E ya reconoce.
- Lecturas públicas positivas con `websearch.fetchWebContent`:
  [#4844](https://github.com/lancedb/lancedb/issues/4844),
  [#4397](https://github.com/lancedb/lancedb/issues/4397),
  [#4402](https://github.com/lancedb/lancedb/issues/4402),
  [PR #4428](https://github.com/lancedb/lancedb/pull/4428).
  Sin error de acceso en esas cuatro lecturas; no inspección de todos los
  comentarios/PR relacionados ni de Cloud. Licencias/componentes se contrastan
  como declaraciones, no resolución transitiva ni autorización para portar.
- Residual: internals de commit/fsync Lance, RTE/LSM, auth remota, cancelación
  real, GC/purga, completitud del inventario de licencias y performance no
  certificados. Esos límites impiden una aceptación de runtime, no obligan a
  inventar un hallazgo contra el alcance estático delimitado.
