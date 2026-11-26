---
title: "Arquitectura de VantaDB — visión general"
kind: concept
status: active
description: "Qué es VantaDB, cómo se divide el workspace y qué problema resuelve cada crate, con enlaces a las fuentes reales."
---

# Arquitectura de VantaDB — visión general

Audiencia: cualquiera que necesite el mapa mental del sistema antes de leer código o usar una API.

## Qué es

`[HECHO]` VantaDB es un motor embebido de memoria persistente y recuperación híbrida para agentes de IA: un crate Rust (`vantadb`) que combina almacenamiento durable con WAL, índice vectorial HNSW, índice lexical BM25, grafo de propiedades y un planificador de consultas DSL (IQL), todo en proceso — sin servicio externo obligatorio (docstring del crate, `src/lib.rs:9-52`).

`[HECHO]` El crate raíz declara como tipos centrales `Embedded` (handle del motor), `UnifiedNode`, `MemoryRecord` y `Error` (`src/lib.rs:15-22`, implementaciones en `src/sdk/builder.rs:15`, `src/node/unified.rs:15`, `src/sdk/types/record.rs:324`, `src/error.rs`).

## División del workspace

`[HECHO]` El workspace declara ocho miembros ([`Cargo.toml`](../../../Cargo.toml), sección `members`): el crate raíz `vantadb`, `vantadb-python`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`, `vantadb-ffi-core`, `vanta-memory` y `vanta-proxy`. `fuzz/` queda excluido explícitamente y `providers/*` no son miembros (`Cargo.toml:793-796`). `vantadb-ts` (npm `vantadb`) es un wrapper TypeScript sobre `vantadb-wasm`/`vantadb-node`, no un crate (`vantadb-ts/package.json:63-66`).

| Capa | Componentes | Responsabilidad |
|---|---|---|
| Motor | `vantadb` (src/) | `[HECHO]` storage + WAL + índices + SDK embebido + grafo + IQL (`src/lib.rs:66-187`) |
| Capa cognitiva | `vanta-memory` | `[HECHO]` pipeline L0–L3 host-neutral sobre el store del motor; no re-exportada por bindings (`vanta-memory/src/lib.rs:4-34`) |
| Transporte FFI | `vantadb-python`, `vantadb-node`, `vantadb-wasm`, `vantadb-ts` | `[HECHO]` superficies PyO3, napi-rs, wasm-bindgen y wrapper TS ([BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md)) |
| Servicios | `vantadb-server`, `vantadb-mcp`, `vanta-proxy` | `[HECHO]` binario HTTP+MCP (`vantadb-server/src/lib.rs:1-12`), librería de herramientas MCP, proxy LLM congelado (`vanta-proxy/src/lib.rs:4-35`) |
| Soporte | `vantadb-ffi-core` | `[HECHO]` hoja std-only con `OpGate`/`OpGuard` y clamp `top_k` compartidos por los tres transports (`vantadb-ffi-core/src/lib.rs:1-58`) |

## Feature flags del core

`[HECHO]` Features por defecto: `cli`, `fjall`, `memmap2`, `fs2`, `roaring` (`Cargo.toml:156`). Opcionales que condicionan módulos públicos: `server`, `tls`, `prometheus`, `encryption`, `wal-shipping`, `async-ingestion`, `async-io`, `arrow`, `rocksdb`, `advanced-tokenizer` (tantivy), `embed-local`, `remote-inference`, `tui`, `failpoints`, `opentelemetry`, `rayon`, `sysinfo`, `hot-reload`, `wasm`, `bayesian_decay` (`Cargo.toml:155-198`; gating por módulo en `src/lib.rs:63-187`).

## Decisiones estructurales registradas

`[HECHO]` Las decisiones arquitectónicas viven como ADRs bajo `docs/dev/architecture/adr/` (ADR-0000 a ADR-0059 + `DECISIONS-NOT-TAKEN.md`), formato MADR e inmutables una vez aceptados. Entradas frecuentes: backend por defecto Fjall (ADR-0020), parámetros HNSW (ADR-0005), RRF k=60 (ADR-0006), granularidad del lock de inserción (ADR-0037), JWT offline (ADR-0039), napi-rs para Node (ADR-0050), cifrado en reposo (ADR-0057).

## Límites del modelo

`[HECHO]` La escritura es de un solo escritor por base de datos: el motor usa un lock de fichero exclusivo (`.vanta.lock`) y el MCP multi-instancia requiere el fallback HTTP documentado en ADR-0036. `[INFERENCIA]` En consecuencia, la concurrencia de escritura entre procesos no es un objetivo del diseño actual; la concurrencia real se da dentro del proceso (ver [concurrencia](concurrency.md)).

## Fuentes

`src/lib.rs` · `Cargo.toml` · `vanta-memory/src/lib.rs` · `vanta-proxy/src/lib.rs` · `vantadb-ffi-core/src/lib.rs` · [ARCHITECTURE](../../dev/architecture/ARCHITECTURE.md) · [BOUNDARIES](../../dev/architecture/BOUNDARIES.md) · [SURFACE_MANIFEST](../../api/SURFACE_MANIFEST.md) · [índice de la wiki](../index.md)
