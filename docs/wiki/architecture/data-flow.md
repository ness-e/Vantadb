---
title: "Flujo de datos en VantaDB"
kind: concept
status: active
description: "Recorrido de una escritura y de una búsqueda a través de SDK, WAL, backend KV e índices, con las fuentes de cada etapa."
---

# Flujo de datos en VantaDB

Audiencia: lectores que quieren seguir una operación de punta a punta. Alcance: rutas de escritura y lectura del motor embebido; no cubre el pipeline cognitivo L0–L3 (ver [cognitive-memory](../modules/cognitive-memory.md)).

## Ruta de escritura (put)

`[HECHO]` La entrada pública es `Embedded::put(MemoryInput)` en `src/sdk/api/memory.rs:1073`; el handle se abre con `Embedded::open`/`open_with_config` (`src/sdk/builder.rs:120`, `:188`) o `connect` (`src/sdk/connect.rs:13`). `[INFERENCIA]` El orden de las etapas siguientes se deduce de la estructura de módulos (`src/storage/engine/`, `src/wal.rs`, `src/backends/`, `src/index/`); las líneas citadas marcan la entrada, no el pipeline completo.

Etapas con fuente:

1. `[HECHO]` Validación/límites en la capa SDK — constantes `MAX_K`, `MAX_VEC_DIM`, `MAX_BATCH_SIZE`, `MAX_F32_VEC_LEN` re-exportadas desde `vantadb::config` (`src/lib.rs:193`).
2. `[HECHO]` Persistencia WAL — módulo `wal` público con `WalWriter`/`WalReader`/`WalRecord` y versiones `WAL_FORMAT_VERSION`/`WAL_POSTCARD_VERSION` (`src/lib.rs:172`, `src/lib.rs:221-222`; implementación `src/wal.rs`, variante sharded interna `src/wal_sharded.rs`, `src/storage/wal.rs`).
3. `[HECHO]` Aplicación en el motor de almacenamiento — `src/storage/engine/{insert,delete,get,ops,txn,maintenance}.rs`; backends KV en `src/backend.rs` y `src/backends/{mod,fjall_backend,rocksdb_backend,in_memory}.rs` (`pub(crate)`, `src/lib.rs:73-74`); `BackendKind` se expone público (`src/lib.rs:218`).
4. `[HECHO]` Publicación en índices — vectorial en `src/index/` (HNSW en `src/index/graph/`, IVF `src/index/ivf.rs`, ScaNN `src/index/scann.rs`, DiskANN `src/index/diskann.rs`, flat `src/index/flat.rs`); textual en `src/text_index.rs` (`pub(crate)`); de aristas en `src/edge_index.rs` (`pub(crate)`).

`[HECHO]` Existe una ruta batch: `put_batch` y variantes `put_batch_raw` por binding ([matriz de superficies](../../api/BINDINGS_NAMESPACES.md)); el pipeline asíncrono `ingestion` existe tras la feature `async-ingestion` (`src/lib.rs:183-184`, `src/ingestion.rs`).

## Ruta de lectura/búsqueda (search)

`[HECHO]` La entrada es `Embedded::search(MemorySearchRequest) -> Vec<MemorySearchHit>` (`src/sdk/search/mod.rs:73`). La rama `src/sdk/search/` contiene las etapas: `vector.rs`, `lexical.rs`, `hybrid.rs`, `fusion.rs` (RRF), `mmr.rs`, `multi.rs`, `page.rs`, `explain.rs`, `snippet.rs`, `phrase.rs`, `sparse.rs` — ver [retrieval](../modules/retrieval.md).

`[HECHO]` La semántica score/distancia es contractual: `MemorySearchHit.score` (mayor = mejor) para búsqueda memoria/híbrida y `SearchHit.distance` (menor = mejor) para ANN crudo, homogéneo en los cinco transportes ([BINDINGS_NAMESPACES §Score vs Distance](../../api/BINDINGS_NAMESPACES.md); contrato oficial en [scores.md](../../api/scores.md)).

## Ruta IQL

`[HECHO]` El lenguaje IQL tiene parser propio (`src/parser/{mod,lexer,grammar}.rs`, versión `IQL_VERSION` re-exportada en `src/lib.rs:200`), planner (`src/planner.rs`), plan físico (`src/physical_plan/`) y ejecutor (`src/executor.rs`); la entrada es `Embedded::query` expuesta como IQL y como tool `query_iql` en MCP ([IQL.md](../../api/IQL.md) — referencia canónica).

## Recuperación al abrir

`[HECHO]` El WAL implementa CRC32C y auto-healing con truncado de cola corrupta (decisión ADR-0002). El replay al abrir el engine (`open` en [`src/storage/engine/init.rs:24`](../../../src/storage/engine/init.rs#L24)) distingue dos caminos: el lector sharded que itera `next_record` (`init.rs:618-651`) y el lector legacy `WalReader::replay_all` ([`src/wal.rs:2320`](../../../src/wal.rs#L2320)) — no todo pasa por `replay_all`. Detalles y límites en [durability](durability.md) y en [DURABILITY_GUARANTEES](../../user/operations/DURABILITY_GUARANTEES.md).

## Límites conocidos

`[HECHO]` La cuota por namespace (`max_records_per_namespace`) no se aplica a la ruta de bulk import `.vdbdump` en 0.9.0 — exención declarada en el propio campo (`src/config.rs:750-758`). Límite de esta página: no se afirma atomicidad multi-registro — el contrato exacto vive en `src/storage/engine/txn.rs`.

## Fuentes

`src/sdk/builder.rs` · `src/sdk/api/memory.rs` · `src/sdk/search/` · `src/wal.rs` · `src/storage/engine/` · `src/backends/` · `src/index/` · `src/parser/` · [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md) · [STORAGE_FORMATS](../../api/STORAGE_FORMATS.md) · [IQL](../../api/IQL.md)
