---
title: "Mapa del código"
kind: concept
status: active
description: "Cómo navegar el repo: layout de crates, dónde vive cada responsabilidad y qué leer primero."
---

# Mapa del código

## Layout

`[HECHO]` Workspace de ocho crates (`Cargo.toml:779-795`): motor `vantadb` en `src/` (raíz), bindings `vantadb-{python,wasm}`, servicios `vantadb-{server,mcp}`, capa cognitiva `vanta-memory`, proxy `vanta-proxy`, hoja FFI `vantadb-ffi-core`. Fuera del workspace: `vantadb-node` (napi-rs), `vantadb-ts/` (npm), `fuzz/`, `providers/*`, `integrations/*`. El repo mantiene además un índice CodeGraph pre-construido para navegación de símbolos.

## Dónde vive cada cosa

`[HECHO]` Entrada pública: [`src/lib.rs`](../../../src/lib.rs) (declara los `pub mod` y re-exportaciones). SDK: `src/sdk/` (builder/api/search/types/serialization/importers). Escritura: `src/storage/engine/` + `src/wal.rs` + `src/backends/`. Índices: `src/index/` + `src/text_index.rs` + `src/vector/` + `src/scalar_index.rs`. Consulta: `src/parser/` + `src/planner.rs` + `src/physical_plan/` + `src/executor.rs`. Grafo: `src/graph.rs` + `src/graphrag/` + `src/gds.rs`. Servidor: `src/server/` (feature `server`) + `vantadb-server/`. MCP: `vantadb-mcp/src/` (tools en `handlers/tools.rs`). Cognitivo: `vanta-memory/src/`.

## Reglas de lectura

`[INFERENCIA]` Empezar por `src/lib.rs` (qué es público y tras qué feature) → `src/sdk/builder.rs` (ciclo de vida) → rama de interés. La autoridad de superficie pública es `tests/api/public-api.txt`, no la intuición. Decisiones: ADRs en `docs/dev/architecture/adr/`.

## Fuentes

`src/lib.rs` · `Cargo.toml` · [modules/](../modules/core.md) · [ARCHITECTURE](../../dev/architecture/ARCHITECTURE.md)
