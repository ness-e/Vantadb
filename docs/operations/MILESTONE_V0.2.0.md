# Milestone v0.2.0: Repo Alignment & Reliability Gate

## Objetivo
Cerrar el gap entre el núcleo técnico que ya existe y la forma en que el repositorio lo explica, lo mide y lo expone a consumidores embebidos.

## Temas principales

### 1. Repo truth
- [x] Reposicionar el proyecto como embedded persistent memory + vector retrieval + structured fields.
- [x] Eliminar claims prematuros sobre multimodelo universal y hybrid textual real.
- [x] Convertir SIFT1M en benchmark de stress/recovery, no de competitividad.

### 2. Memory telemetry contract
- [x] Corregir unidades de proceso en certificación.
- [x] Explicitar que la telemetría de proceso no equivale a footprint lógico del índice.
- [x] Añadir harness controlado para cold start, ingestión, replay y reinicio.

### 3. Embedded SDK stabilization
- [x] Mantener `src/sdk.rs` como boundary estable interno.
- [x] Mantener el binding Python detrás de ese boundary.
- [x] Diferir PyPI, wheels, signing e instaladores.

### 4. Reliability gate
- [x] Durability recovery green
- [x] Index reconstruction green
- [x] Backend parity green
- [x] Memory telemetry harness green
- [x] Python SDK smoke green
- [x] Python SDK pytest green

## Criterios de éxito
- El README describe exactamente lo que el core hace hoy.
- Las métricas de memoria tienen tipo, unidad y nivel de confianza explícitos.
- El SDK Python funciona localmente vía source-install sin tocar internals del engine.
- El siguiente ciclo quedó despejado y ahora avanza sobre `modelo canónico`, `namespaces` y `put/get/delete/list + WAL/recovery`.

## Extensión memory-mvp-core

El bloque posterior ya quedó implementado en repo:

- Modelo canónico de memoria persistente en el SDK, separado de `UnifiedNode`.
- Namespaces first-class con identidad lógica `namespace + key`.
- API mínima `put/get/delete/list/search` con BM25 para `text_query` texto-only y rechazo explícito de hybrid hasta RRF/planner.
- Python SDK con flujo de memoria y compatibilidad con APIs legacy.
- CLI embebida para `put/get/list`.
- Rebuild ANN manual desde VantaFile/storage canónico.
- Export/import JSONL estable para memoria persistente.
- Índices derivados `namespace_index` y `payload_index` reconstruibles desde registros canónicos.

## Evidencia adicional

- `memory_api`
- `memory_export_import`
- `derived_indexes`
- `memory_brutality`

`memory_brutality` cubre recovery sin flush explícito, pérdida de `vector_index.bin`, rebuild manual, export/import round-trip y smoke de 10K records con namespaces y filtros.
