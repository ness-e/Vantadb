# Task API-STD-02 — INDIVIDUAL (1/11) Rust core SDK

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual Rust core: funcionamiento + uso + código + veredicto por fallo.

## 2. Funcionamiento

Dueño del estado (`Embedded`/`InMemoryEngine`, `src/lib.rs:19`): open/close, CRUD memoria (`MemoryInput`/`MemoryRecord`), search híbrido (HNSW+BM25+RRF), grafo (nodos/aristas/traversals), IQL (`src/parser/`), índices (rebuild/repair/audit), WAL/durabilidad, import/export, capabilities/metrics. Todo in-process; bindings son vistas sin lifecycle propio.

## 3. Uso (ejemplo mínimo, de `src/lib.rs:40-53`)

```rust
use vantadb::sdk::{Embedded, MemoryInput};
use vantadb::config::Config;
let engine = Embedded::open_with_config(Config::default()).unwrap();
engine.put(MemoryInput::new("docs", "example", "Hello, VantaDB!")).unwrap();
let record = engine.get("docs", "example").unwrap();
engine.close().unwrap();
```

## 4. Código (verificado 2026-09-24 por lectura directa)

- Tipos estables: `MemoryInput` (`record.rs:35-54`, todo documentado, `vector/sparse_vector/ttl_ms: Option`), `MemoryRecord` (`:79-114`, `node_id` con `u128_serde` `:95-96`, `superseded_by/at_ms` `:105-113`).
- Re-exports ergonómicos `src/lib.rs:167-196`.
- **Fallo R1 CONFIRMADO — stutter residual:** `pub struct VantaHeader` (`binary_header.rs:20`, re-export `lib.rs:167`); `json_to_vanta_value` (`cli_handlers/crud.rs:445`). ADR-041 lo excluye por compat on-disk (`adr/041:43,56`), sigue `proposed` sin firma (`:64`).
- **Fallo R2 CONFIRMADO — catch-all errores:** `Generic(ChainedError)` (`error.rs:275-276`), `ResourceLimit(String)` (`:184`), `InvalidInput(String)` (`:283`), `Schema(String)` (`:287`).
- **Fallo R3 CONFIRMADO — docs por variante:** `FilterOp::{Eq,Neq,Gt,Lt,Gte,Lte}` sin `///` por variante (`record.rs:13-20`, solo doc de enum `:11`); contrasta con `MemoryInput/Record` documentados campo por campo.
- **Fallo R4 CONFIRMADO — wire >2^53:** `QueryResult::Write.node_id: Option<u128>` SIN `u128_serde` (`graph.rs:18-24`) mientras `StaleContext.node_id` SÍ (`:27-31`) y `MemoryRecord.node_id` SÍ (`record.rs:95-96`). JSON pierde IDs >2^53 solo en writes.
- **R5 breaking históricos CONFIRMADOS:** aliases `Vanta*Hit` eliminados 0.6.0 + flat `get_memory/list_memory/delete_memory` removidos AST-012 (`BINDINGS_NAMESPACES.md:37-39,240-244`); `SnapshotRecord→MemoryRecord` pierde `superseded_by/at_ms` (`version_history.rs:144-145` None).

## 5. Veredicto + implicaciones

5/5 fallos confirmados en código. Propuestas para API-STD-15: (a) `u128_serde` en `Write.node_id` (breaking wire, `feat!:`); (b) tipar `Generic/ResourceLimit/InvalidInput` o documentar catch-all como diseño; (c) docs por variante `FilterOp`; (d) cerrar ADR-041 (firmar o fijar `VantaHeader` permanente). Blast radius: (a) afecta 4 bindings + HTTP + MCP (todos leen `QueryResult`).

## 6. DoD

- [x] Contrato ✅ (lecturas + citas `file:line`)
- [x] Task file sync · Recitation: Rust ficha completa, 5/5 fallos confirmados

## Context Save Point

API-STD-02 DONE. Next: API-STD-03 (Python). Deuda: ninguna. WIP: ninguno.
