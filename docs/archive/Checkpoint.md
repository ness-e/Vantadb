---
tags: [mpts/vault, mpts/checkpoint]
mpts_vault_id: a1b2c3d4-e5f6-7890-abcd-ef1234567890
mpts_version: 1
---

# Resumen de Vault – VantaDB

**Última actualización:** 2026-06-22
**Cobertura:** docs/ frente a código fuente

## Arquitectura y Estado

VantaDB es una base de datos vectorial **embebida** (sin servidor independiente) escrita en Rust, diseñada para servir como capa de memoria persistente para agentes de IA. Opera **in-process** como librería enlazada, con bindings a Python (via PyO3/maturin), al browser (via WASM/wasm-pack) y al ecosistema TypeScript (npm).

El proyecto completó la **FASE 3 (pre-lanzamiento, ~95%)** y está transitando a **FASE 4 (Launch: deuda técnica estructural)**. El core crate `vantadb` v0.1.4 se publicó en crates.io. El crate `vantadb-py` se publica directamente a PyPI via CI. Un SDK TypeScript (`vantadb-ts`) está empaquetado para npm.

La versión actual es **0.1.5** en desarrollo.

## Estructura del Vault MPTS

| Archivo | Dominio | Estado |
|---------|---------|--------|
| `Master Index.md` | Navegación general + glosario (50 términos) | Estable |
| `Arquitectura Técnica y Core Engine.md` | Técnico | Estable |
| `Especificaciones Funcionales y SDK API.md` | Técnico | Estable |
| `Estrategia de Ecosistema y GTM.md` | Producto | Estable |
| `Operaciones, Calidad y Riesgos.md` | Operaciones | Estable |
| `Roadmap e Hitos de Ingeniería.md` | Producto/Técnico | Estable |
| `Visión y Posicionamiento Estratégico.md` | Estratégico | Estable |
| `Glosario.md` | Referencia transversal | Estable |
| `REPORT_AUDITORIA_RECTIFICACION_2026-06-14.md` | Calidad | Histórico |
| `REPORT_REVISION_WIKILINKS_GLOSARIO_2026-06-14.md` | Calidad | Histórico |

## Correcciones Aplicadas en Esta Sesión (2026-06-22)

### docs/api/HTTP_API.md (NUEVO)
- **Problema:** El archivo `docs/api/HTTP_API.md` no existía a pesar de estar listado como "Done" en el Master Index. Los 3 endpoints del servidor HTTP (`GET /health`, `GET /metrics`, `POST /api/v2/query`) no tenían documentación.
- **Corrección:** Documento creado con especificación completa: base URL, auth, rate limiting, TLS, payload schemas, ejemplos curl y tabla de rutas.

### docs/api/PYTHON_SDK.md — 27 métodos Rust-native agregados
- **Problema:** 27 métodos expuestos vía `#[pymethods]` en el binding Rust no estaban documentados: `insert()`, `get()` (node), `delete()` (node), `search()` (vector), `search_batch()`, `query()`, `flush()`, `compact_wal()`, `purge_expired()`, `rebuild_index()`, `compact_layout()`, `export_namespace()`, `export_all()`, `import_file()`, `audit_text_index()`, `repair_text_index()`, `operational_metrics()`, `capabilities()`, `hardware_profile()`, `add_edge()`, `graph_bfs()`, `graph_dfs()`, `graph_topological_sort()`, `graph_is_dag()`, `generate_snippet()`, `explain_memory_search()`, `close()`.
- **Corrección:** Agregada la sección "Node / Graph API" y "Maintenance & Diagnostics" con firmas, parámetros, tipos de retorno y descripciones. Tabla de return types expandida de 26 a 52 filas. Changelog reorganizado con versión correcta 0.1.2 que agrupa todos los métodos Rust-native.

### docs/operations/CONFIGURATION.md — CLI commands y Cargo features
- **Problema:** Faltaban 9 comandos CLI (`audit-index`, `repair-text-index`, `query`, `status`, `search`, `delete`, `completions`, `namespace`, `server`) y 3 Cargo features (`remote-inference`, `python_sdk`, `opentelemetry`) no documentados.
- **Corrección:** Sección 4 reescrita con tabla completa de 16 comandos CLI + global flags + ejemplos. Nueva sección 7 con tabla de 14 Cargo features, sus dependencias y descripciones.

### vantadb-ts/README.md — 9 métodos TS faltantes
- **Problema:** 9 métodos en `vantadb.ts` no estaban en el README: `exportNamespace`, `exportAll`, `importRecords`, `importFile`, `auditTextIndex`, `auditTextIndexDeep`, `repairTextIndex`, `query`, `generateSnippet`.
- **Corrección:** Tablas agregadas para Export/Import, Text Index, y Utilities.

### docs/VantaDB-MPTS/Master Index.md
- **Problema:** `docs/api/EMBEDDED_SDK.md` listado como "Done" pero no existe. HTTP_API.md igual (ahora creado).
- **Corrección:** EMBEDDED_SDK.md marcado como "❌ Missing". HTTP_API.md corregido a "Done" con descripción real. Enlaces TOC y glosario ya corregidos en sesión anterior.

### docs/api/EMBEDDED_SDK.md (NUEVO)
- **Problema:** El archivo no existía a pesar de estar listado como "Done" en el Master Index. La struct `VantaEmbedded` (~45 métodos públicos) no tenía documentación formal.
- **Corrección:** Documento creado con referencia completa: construcción, Memory API, Node/Graph API, Maintenance, Export/Import, Text Index Diagnostics, Observability, Lifecycle, ~15 tipos de datos (`VantaMemoryInput`, `VantaMemoryRecord`, `VantaMemorySearchRequest`, `VantaNodeInput`, `VantaNodeRecord`, `VantaCapabilities`, `VantaOperationalMetrics`, etc.), y 5 report types (`VantaIndexRebuildReport`, `VantaExportReport`, `VantaImportReport`, `VantaTextIndexAuditReport`, `VantaTextIndexRepairReport`).

### docs/architecture/ADVANCED_TOKENIZER.md
- **Problema:** La guía usaba el tipo `VantaDB` (inexistente en Rust) en ejemplos de código. Métodos `put_memory()` y `search_memory()` no existen (la API real es `put()` y `search()`). La sección "Future Enhancements" mencionaba "Runtime configuration via VantaConfig" como algo futuro, pero ya está implementado.
- **Corrección:** `VantaDB` → `VantaEmbedded` en todos los ejemplos Rust. `put_memory()` → `put()`. `search_memory()` → `search()`. Rutas de import corregidas (`vantadb::tokenizer::{...}` en vez de `vantadb::text_index::{...}` — `text_index` es `pub(crate)`). Se eliminó la sección "Future Enhancements" redundante y se reemplazó por runtime configuration real.

### docs/operations/CONFIGURATION.md
- **Problema:** La tabla de configuración estaba incompleta (solo ~15 de 26 campos). Variables de entorno incorrectas: `VANTADB_THREADS` (real: `VANTADB_MAX_BLOCKING_THREADS`), `HOST`/`PORT` descritos como primarios cuando son fallbacks de `VANTADB_HOST`/`VANTADB_PORT`. `RUST_LOG` no es un knob de VantaConfig.
- **Corrección:** Tabla completa con los 26 campos, tipos, valores por defecto, env vars reales y descripciones. Se agregaron secciones para los enums (`LogFormat`, `SyncMode`, `PrefetchMode`, `BackendKind`), la CLI y notas operativas.

### docs/api/PYTHON_SDK.md
- **Problema:** Versión indicada como 0.1.1 cuando la real es 0.1.5. Faltaban ~20 métodos que ya existen en el binding Python: `put_batch`, `list_namespaces`, `get_namespace_info`, `from_documents`, `from_file`, `from_url`, `split_text`, `update_payload`, `update_metadata`, `update_importance`, `rename_key`, `consolidate`, `get_namespace_insights`, `knowledge`, `knowledge_search`, `ask`, `chat`, `query_ollama`, `monitor_reset_window`.
- **Corrección:** Tabla completa con los 23 métodos del SDK Python, parámetros, tipos de retorno y changelog actualizado.

### docs/VantaDB-MPTS/Master Index.md
- **Problema:** 4 enlaces TOC rotos por caracteres Unicode corruptos en `#glosario` → `#glossary-glosario`. Wikilink `[progress](../progreso/README.md)` roto (debe ser ruta relativa). Glosario tenía 47 entradas listadas pero 50 definidas. Faltaban 3 entradas (Stemming, Stopwords, Advanced Tokenizer).
- **Corrección:** TOC anchors unificados al inglés. `[progress](../progreso/README.md)` → `docs/progreso/README.md`. Glosario expandido de 47→50 términos, añadiendo los 3 faltantes. Se agregaron enlaces a todos los archivos documentados.

## Estado Actual del Código

| Componente | Archivos | Estado |
|------------|----------|--------|
| Core engine (HNSW, BM25, WAL, mmap, Fjall) | `src/` ~50+ módulos | ✅ Estable |
| Python bindings (PyO3) | `vantadb-python/` | ✅ Publicado en PyPI |
| WASM bindings | `vantadb-wasm/` → `vantadb-ts/` | ✅ npm-ready |
| MCP server | `vantadb-mcp/` | ✅ Estable |
| CLI | `src/bin/vanta-cli.rs` + `src/cli.rs` + `src/cli_handlers.rs` | ✅ 46 tests pasan |
| HTTP server (axum) | `src/http.rs`, `src/cli_server.rs` | ✅ Estable |
| Prometheus metrics | `src/metrics.rs` + middleware | ✅ Implementado |
| Grafana dashboard | `docs/operations/grafana-dashboard.json` | ✅ Creado |
| Backups | CLI backup/restore/doctor WAL-based | ✅ Implementado |
| Filter operators | `MemoryFilter` con Eq/Neq/Gt/Gte/Lt/Lte/In/Exists | ✅ Implementado |
| Multi-namespace search | `namespaces: Vec<String>` en search request | ✅ Implementado |

## Backlog Activo (FASE 4)

| TSK | Título | Prioridad |
|-----|--------|-----------|
| TSK-123 | Promover advanced-tokenizer como feature default | 🔴 Alta |
| TSK-124 | Documentar `generate_snippet` y `highlighting` en PYTHON_SDK.md | 🔴 Alta |
| TSK-125 | Alinear documentación SLSA con implementación real | 🔴 Alta |
| TSK-126 | Agregar `impl Drop for StorageEngine` (liberación explícita de lock) | 🔴 Alta |
| TSK-127 | Formalizar estado de IQL (estable o experimental) | 🔴 Alta |
| TSK-128 | Hacer configurable timeout de `insert_lock` | 🟡 Media |
| TSK-129 | Hacer configurable timeout de `.vanta.lock` | 🟡 Media |
| TSK-130 | Instrumentación de heap memory drift | 🟡 Media |
| TSK-131 | Implementar PITR vía WAL archival | 🔵 Baja |
| TSK-132 | Investigar checkpoint API en Fjall upstream | 🔵 Baja |
| TSK-133 | Agregar backup incremental | 🔵 Baja |

## Issues de Infraestructura

| Issue | Estado |
|-------|--------|
| Windows pagefile — `os error 1455` en mmap_hnsw y proptest | 🔴 Entorno, no código |
| `install-action` — `cargo-llvm-cov` y `@nextest` fallan intermitentemente | 🔴 Infra GitHub |

## Próxima Acción Recomendada

Completar TSK-123 y TSK-124 (los dos bloqueantes de documentación más urgentes), luego ejecutar la auditoría de links/variables de entorno en docs/operations/ para asegurar consistencia antes del release v0.2.0.
