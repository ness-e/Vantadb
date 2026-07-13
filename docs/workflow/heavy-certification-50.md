# `heavy-certification-50.yml` — HEAVY: Certification — All Tests

## ¿Qué hace?

Suite completa de certificación que ejecuta todos los tests pesados de VantaDB en modo release. Prueba estrés del protocolo, validación HNSW, recuperación de índices, persistencia de storage, concurrencia, memoria, failpoints, y tests de integración con datasets reales.

## ¿Cómo lo hace?

9 jobs paralelos, cada uno ejecutando tests específicos:

| Job | Tests que ejecuta | Flags | Timeout |
|-----|------------------|-------|---------|
| `stress-protocol` | `stress_protocol` | release, 1 thread | 180m |
| `hnsw-validation` | `hnsw_validation` | release, 1 thread | 120m |
| `hnsw-recall` | `hnsw_recall_certification` | release, 1 thread | 40m |
| `sift-validation` | `sift_validation` (opcional) | release, 1 thread | 45m |
| `competitive-bench` | `competitive_bench` (opcional) | release, 1 thread | 45m |
| `failpoint-tests` | `chaos_integrity`, `wal_resilience`, `crash_injection` | failpoints, 1 thread | — |
| `storage-persistence` | 17 tests de storage: `backend_tests`, `storage`, `durability_recovery`, `derived_index_recovery`, `index_reconstruction`, `fjall_cold_copy_restore`, `property_durability`, `file_locking_stress`, `schema_evolution`, `gc`, `mmap_index`, `mutations`, `antilocality_layout`, `tombstone_ann_vstore`, `multi_process_lock`, `operational_metrics`, `prefetch_benchmark` | release + cli, 1 thread | 90m |
| `text-index` | `text_index_recovery` | release, 1 thread | 60m |
| `memory-concurrency` | `memory_brutality`, `memory_export_import`, `concurrency_parity`, `memory_api`, `memory_telemetry`, `edge_cases`, `fuzz_proptest` | release, 1 thread | 90m |
| `other-heavy` | Tests de integración varios: `benchmark_internal`, `benchmark_datasets`, `cli_tests`, `structured_api_v2`, `python_sdk_boundary`, `hybrid_ranking_metrics`, `hybrid_retrieval_quality`, `basic_node`, `hardware_profiles`, `vector_scale_check`, `integration`, `multilingual_tokenizer_integration`, `columnar`, más tests de `vantadb-server` y `vantadb-mcp` | release + cli,arrow, 1 thread | 60m |

## ¿Qué tests usa?

~35 test binaries específicos de certificación (todos en modo `--release` y `--test-threads=1`).

## ¿Qué verifica

- Integridad del protocolo bajo estrés
- Precisión y recall del índice HNSW (incluyendo recall mínimo certificado)
- Validación contra dataset SIFT-1M (opcional)
- Recuperación ante fallos (crash injection, WAL resilience)
- Persistencia de storage y recuperación ante cortes
- Recuperación de índices de texto
- Operaciones concurrentes y uso de memoria en escenarios extremos
- Integración con CLI, Python SDK, MCP server, datasets reales

## Funcionalidad final

Certificar que VantaDB cumple con los estándares de calidad, durabilidad y rendimiento antes de releases. Es la suite más exhaustiva del proyecto.

## ¿Cuándo se ejecuta?

- **Semanal** (domingo 03:00 UTC) vía `schedule`
- **Workflow dispatch** manual, con opciones para incluir SIFT validation y competitive benchmarks
