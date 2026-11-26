---
title: "API de Rust (crate `vantadb`)"
kind: reference
status: active
description: "Entrada embebida al motor: Embedded, Config, tipos centrales y re-exportaciones, con el snapshot canónico de superficie pública."
---

# API de Rust (crate `vantadb`)

Audiencia: consumidores del crate y contribuidores. La superficie pública completa — exhaustiva y mecánica — es el snapshot `tests/api/public-api.txt` (golden de `tests/api/public_api.rs`, public-api 0.52.x con features por defecto); esta página es el mapa, no la lista.

## Entrada

`[HECHO]` Handle del motor: `Embedded` ([`src/sdk/builder.rs:15`](../../../src/sdk/builder.rs#L15)). Apertura: `Embedded::open(path)` ([`:120`](../../../src/sdk/builder.rs#L120)), `Embedded::open_with_config(Config)` ([`:188`](../../../src/sdk/builder.rs#L188)), `vantadb::connect(path)` ([`src/sdk/connect.rs:13`](../../../src/sdk/connect.rs#L13)). Cierre: `Embedded::close()` ([`builder.rs:395`](../../../src/sdk/builder.rs#L395)).

```rust
use vantadb::sdk::{Embedded, MemoryInput};
use vantadb::config::Config;

let engine = Embedded::open_with_config(Config::default()).unwrap();
engine.put(MemoryInput::new("docs", "example", "Hello, VantaDB!")).unwrap();
let record = engine.get("docs", "example").unwrap();
engine.close().unwrap();
```

`[HECHO]` El ejemplo anterior es el propio doctest del crate (`src/lib.rs:39-52`); no se ha ejecutado en esta sesión.

## Tipos centrales

`[HECHO]` Re-exportados desde la raíz (`src/lib.rs:190-222`):

| Grupo | Símbolos |
|---|---|
| Configuración | `Config` (`src/config.rs:719`), `BackendKind`, `PipelineMode`, `StorageTier`, constantes `MAX_K`, `MAX_VEC_DIM`, `MAX_BATCH_SIZE`, `MAX_F32_VEC_LEN` |
| Registros | `MemoryInput` (`src/sdk/types/record.rs:184`), `MemoryRecord` (`:324`), `MemorySearchHit`, `MemorySearchRequest`, `MemoryListPage`, `MemoryMetadata` |
| Grafo | `UnifiedNode` (`src/node/unified.rs:15`), `Edge`, `NodeInput`, `NodeRecord`, `RelFields`, `SparseVector`, `DistanceMetric`, `VectorRepresentations` |
| Búsqueda | `SearchHit`, `SearchExplanation*`, `MmrConfig`, `FilterOp`, `MemoryFilter*`, `RangeFilter`, `GroupByConfig`, `Bm25TermContribution`, `HybridFusionReport` |
| Ciclo de vida | `Capabilities`, `OperationalMetrics`, `RuntimeProfile`, `IndexRebuildReport`, `VacuumReport`, `MergeReport`, `PipelineReport` |
| Olvido/merge | `ForgetMode`, `ForgetTarget`, `ForgetReport`, `MergeOutcome`, `MergeResult`, `DedupPolicy` |
| Temporal/confianza | `ValidWindow`, `ConfidenceClass`, `AbstentionReason`, `ReinforceOutcome`, `DERIVATION_DISCOUNT`, `MAX_DERIVATION_DEPTH` |
| Skills | `SkillRecord`, `Skill*Input`, `SkillListPage`, `SkillWriteResult` |
| Errores | `Error`, `Result` (`src/error.rs`) |

## Operaciones principales

`[HECHO]` Memoria: `put` ([`src/sdk/api/memory.rs:1073`](../../../src/sdk/api/memory.rs#L1073)), `get` ([`:1459`](../../../src/sdk/api/memory.rs#L1459)), `delete` ([`:1534`](../../../src/sdk/api/memory.rs#L1534)), `forget` ([`:1671`](../../../src/sdk/api/memory.rs#L1671) — modos Soft/Hard/Certified). Búsqueda híbrida: `search(MemorySearchRequest)` ([`src/sdk/search/mod.rs:73`](../../../src/sdk/search/mod.rs#L73)). Administración: `flush` ([`src/sdk/api/admin.rs:124`](../../../src/sdk/api/admin.rs#L124)). IQL: `query` (ver [IQL](../../api/IQL.md)).

## Características por feature

`[HECHO]` Módulos que solo existen con feature: `crypto` (`encryption`), `server`/`circuit_breaker`/`connection_pool`/`cli_server` (`server`), `cli*` (`cli`), `columnar` (`arrow`), `python` (`python_sdk`), `llm` (`remote-inference`|`embed-local`), `tokenizer` (`advanced-tokenizer`), `tui` (`tui`), `wal_shipping` (`wal-shipping`), `ingestion` (`async-ingestion`), `transcript` (`async-io`), `testing`+`cfg_failpoint` (`failpoints`) — `src/lib.rs:63-241`.

## Convenciones

`[HECHO]` `clippy::unwrap_used`/`expect_used` están en `deny` fuera de tests (`Cargo.toml:771-772`; permiso solo en `#[cfg(test)]`, `src/lib.rs:7`). `[HECHO]` El enum `Error` es la única vía de error pública del motor; la referencia de manejo es [ERROR_HANDLING](../../api/ERROR_HANDLING.md).

## Fuentes

`src/lib.rs` · `src/sdk/` · `src/config.rs` · `tests/api/public-api.txt` · [EMBEDDED_SDK](../../api/EMBEDDED_SDK.md) · [COMPATIBILITY](../../api/COMPATIBILITY.md) · [VERSIONING](../../api/VERSIONING.md) · [DEPRECATIONS](../../api/DEPRECATIONS.md)
