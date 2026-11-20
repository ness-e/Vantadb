---
title: "Referencia VantaDB para la ampliacion comparativa"
kind: research
status: active
description: "Snapshot local y limites de las capacidades que los informes ampliados deben contrastar antes de proponer extracciones"
tags: [research, memory, engineering]
---

# Referencia VantaDB para la ampliacion comparativa

## Snapshot, no certificacion de runtime

La referencia observada limpia al comenzar la ampliacion es
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612` en `develop`.
El informe OpenViking y las primeras fichas usan snapshots anteriores; no se
pueden trasladar sus conclusiones como hechos actuales.

La lectura usa CodeGraph y busqueda local de literales no cubiertos por el
grafo. No se ejecutaron pruebas, aplicaciones ni benchmarks. Un test existente
acredita que el caso esta escrito, no que haya pasado en esta sesion.

## Capacidades y limites discriminantes

| Superficie | Evidencia local observada | Consecuencia para las recomendaciones |
|---|---|---|
| Context assembly | `vanta-memory/src/context_engine/engine.rs::assemble`, `assemble_inner`, `assemble_with_recall` | Ya hay coordinador y compactacion: extenderlo, no crear otro |
| Presupuesto protegido | `engine.rs:197-218` conserva el prefijo protegido incluso si excede el presupuesto | No afirmar garantia absoluta de caber siempre; explicitar esa excepcion |
| Reporte contextual | `IntegratedContext`, `CompactionReport` | Ya hay diagnostico; compararlo con el contrato concreto requerido, no llamar inexistente a toda observabilidad |
| Inyeccion gobernada | `core/hooks/auto_recall.rs::InjectionPolicy`, `perform_auto_recall_governed` | Hay ACL por prefijos y reporte de denegaciones; no equivale a demostrar todos los transports o toda la autorizacion |
| Prefijos | `auto_recall.rs:133-142` declara limites por segmento y contenido tainted | No proponer corregir ciegamente colision sess-1/sess-12 como si estuviera demostrada |
| Recall configurable | `RecallConfig` en `auto_recall.rs:302-335` | Hay modo, scope, limites por resultado y presupuesto por caracteres |
| Ruta de busqueda | `RecallConfig::core_search` tiene default false en la muestra | La existencia de BM25/HNSW/RRF en el motor no prueba que toda llamada de memoria use ese camino |
| Presupuesto de recall | `apply_recall_budget` en `auto_recall.rs:1159-1189` | No afirmar ausencia de presupuesto; caracteres y tokens son unidades diferentes |
| Tenancy L1 | `l1_writer.rs::default_tenancy`, `plan_write` | Ya existen sellos agent/team y tratamiento de legado; verificar el caso concreto antes de proponer un modelo nuevo |
| Dedup y supersession | `core/record/l1_dedup.rs`, `l1_writer.rs`, `lifecycle.rs` | Ya existe politica de decisiones y marcado de contradicciones; merge declarativo seria un delta, no dedup desde cero |
| Escritura L1 | `persist_planned` y `apply_dedup_batch` en `l1_writer.rs` | Separar plan, persistencia, borrados y efectos secundarios; no inferir atomicidad del batch completo por su nombre |
| Checkpoint global | `utils/checkpoint.rs::mutate`, lineas 112-117 | Lectura-modificacion-escritura separadas en ese helper; concurrencia real requiere examinar el caller y reproduccion discriminante |
| Checkpoint de tarea | `utils/task_checkpoint.rs::TaskCheckpointManager` | Ya existen begin/load/advance/complete/fail; no crear otro scheduler por desconocer esta superficie |
| Log de generacion | `core/memory_generation_log/` | Existe rastro de generacion, distinto de un ledger de entrega al modelo |
| Cuotas por namespace | `src/config.rs:760-762`, builders `with_max_namespace_records`, `with_max_namespace_bytes` | Hay configuracion de cuotas; no llamar ausente al concepto por comparar con presupuestos de contexto |
| Retrieval e indices | `src/sdk/search/`, `src/index/`, `src/vector/quantization.rs` | Ya hay hibrido, filtros, HNSW, Flat/IVF y cuantizacion; el valor transferible debe ser concreto |
| Auditoria | `src/cli_handlers/index.rs` | Auditoria textual existente no demuestra el eje completo fuente-vector-derivado, pero tampoco puede negarse |
| Codigo como memoria | `vantadb-mcp/src/code_index.rs`, `code.rs` | Hay indexacion y grafo propios; herramientas externas del desarrollador no implican integracion automatica |
| Integraciones | `integrations/langchain/`, bindings oficiales y MCP | Revisar adapters existentes antes de especificar uno desde cero |
| Studio | Worktree `C:\Users\Eros\VantaDB Proyect\develop-desktop`, HEAD observado `0ad38c542` | Extender el producto existente; su runtime y componentes nuevos no quedan acreditados por este inventario |
| LoCoMo | `evals/eval_locomo.py::main`, `run_locomo_evaluation`, `format_markdown_table` | Revalidado en este snapshot: sintetico por defecto, fallback a fake fuera de self-test y tabla no equivalente; ver [notas de benchmarks](BENCHMARK-NOTES.md#6-revalidacion-estatica-de-locomo-durante-la-ampliacion) |

Los simbolos concretos observados para recall son `RecallScope`,
`RecallConfig` e `InjectionPolicy`. La busqueda literal de `MemoryScope` en
`src`, `vanta-memory` y `vantadb-mcp` no devolvio coincidencias en esta pasada.
No presentar ese nombre de propuestas previas como un tipo implementado;
esto tampoco acredita ausencia en todo el repositorio o en otros worktrees.

## Reglas para cada informe

### Identidad de la evidencia Studio

La referencia inmutable del worktree separado es
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`. En ese commit las rutas son
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx`,
`feature-freez/desktop/src/components/memory/MemoryLens.tsx` y
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx`.
Se contrastaron cuerpos mediante `git show`, sin modificar el worktree.
Las rutas de disco `desktop/src/...` no pertenecen automaticamente a ese
snapshot: hay movimientos y modificaciones locales. Cada informe debe citar
la ruta y los rangos del commit si utiliza esa evidencia inmutable, o declarar
por separado una lectura del worktree modificado, sin atribuir sus bytes ni
sus rangos a HEAD. Localizar un componente no acredita haber leido su flujo.
Esta precision no cambia el snapshot del motor ni certifica el runtime de UI.

1. Una capacidad de VantaDB debe citar una ruta real y la revision observada.
2. Ausencia en una muestra se expresa como no acreditada, no como inexistencia.
3. Un riesgo local necesita mecanismo y discriminacion, no parecido nominal
   con un issue upstream.
4. No unir L1, RMW de checkpoint, recovery WAL y atomicidad transaccional como
   si fueran un solo defecto.
5. No prometer tokenizacion exacta cuando se usa estimador ni confundir cuotas
   de almacenamiento, presupuestos de recall y presupuesto del modelo.
6. LLM y proveedores externos siguen siendo opcionales en las primitivas del
   motor. Describir sus costes sin inventar mediciones.
7. Durabilidad del primario y disponibilidad de proyecciones son dimensiones
   distintas; un indice reconstruible puede ir retrasado.
8. No se autoriza implementar, modificar backlog activo o ejecutar los casos
   de aceptacion propuestos durante esta investigacion.

## Limites de cobertura

Este archivo registra los puntos discriminantes leidos por el responsable
principal para evitar falsos positivos repetidos. No sustituye la lectura de
las rutas de integracion de cada candidato. Las cuotas se observaron como
configuracion/builders: su enforcement completo debe comprobarse aparte.
Tampoco certifica recuperacion tras crash, seguridad multitenant, igualdad
entre bindings, soporte visual ni calidad de retrieval.

Los enlaces externos de los informes deben fijar el SHA cuando describan
codigo. Las rutas relativas son navegacion local y pueden avanzar despues del
snapshot. Ver [contrato de ampliacion](EXPANSION-CONTRACT.md).
