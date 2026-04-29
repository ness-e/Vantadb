# Repo Alignment Checklist

Este checklist define el corte inmediato del repositorio después del release técnico inicial. Su objetivo ya no es “empujar distribución”, sino alinear narrativa, telemetría y surface area con el estado real del core.

## 1. Claims y documentación

- [x] README reposicionado como memoria persistente embebida + vector retrieval.
- [x] Claims de multimodelo universal rebajados o eliminados.
- [x] Claims de “hybrid search” acotados a vector + filtros estructurados mientras BM25/RRF siga pendiente.
- [x] SIFT1M etiquetado como benchmark no comparable para competitividad mientras el motor siga en cosine-only.
- [x] Documentación de arquitectura reescrita para reflejar el boundary actual del producto.

## 2. Naming y consistencia técnica

- [x] Restos principales de naming legado eliminados en tests y descripciones públicas.
- [x] El boundary estable del SDK se documenta como `src/sdk.rs`.
- [x] El paquete Python sigue siendo source-install only y no promete PyPI.

## 3. Observabilidad y métricas

- [x] Contrato de telemetría de memoria documentado.
- [x] Métricas de proceso separadas de métricas lógicas del índice.
- [x] El repo deja explícito qué métricas son confiables y cuáles siguen siendo experimentales.
- [x] Harness controlado de memoria añadido para cold start, ingestión, replay y reinicio.

## 4. Gate de confiabilidad

- [x] `durability_recovery`
- [x] `index_reconstruction`
- [x] `backend_tests`
- [x] `memory_telemetry`
- [x] `python_sdk_boundary`
- [x] smoke del SDK Python
- [x] `pytest vantadb-python/tests/test_sdk.py -v`

## 5. Trabajo diferido de forma explícita

- [x] PyPI, wheels y signing quedan fuera de este ciclo.
- [x] BM25, RRF y planner real quedan fuera de este ciclo.
- [x] Namespaces first-class y modelo canónico pasan al siguiente bloque del MVP.

## 6. Siguiente bloque activo

- [x] Modelo canónico de memoria separado de `UnifiedNode`.
- [x] Namespaces first-class con `namespace + key`.
- [x] API mínima `put/get/delete/list/search`.
- [x] Flujo Python SDK para memoria persistente.
- [x] CLI embebida mínima `put/get/list`.

## 7. Bloque operativo memory-mvp-core

- [x] Rebuild ANN manual expuesto en Rust SDK, Python SDK y CLI.
- [x] Export/import JSONL por namespace y base completa.
- [x] Índices derivados persistidos para namespace y filtros escalares de metadata.
- [x] Rebuild de índices derivados desde registros canónicos.
- [x] Suite de brutalidad con recovery, pérdida de índice, export/import y smoke de 10K records.

## 8. Límites que siguen abiertos

- [ ] Optimizar los índices derivados con iteradores/prefix scans reales en el backend.
- [ ] Añadir telemetría estructurada de `startup_ms`, `wal_replay_ms`, `wal_records_replayed`, `rebuild_ms`, `records_exported` y `records_imported`.
- [ ] Diseñar índice textual antes de implementar BM25/RRF.
- [ ] Mantener PyPI/wheels/signing fuera del ciclo hasta estabilizar API y release engineering.
