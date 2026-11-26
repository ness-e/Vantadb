---
title: "Módulo: cognitive-memory (`vanta-memory`)"
kind: concept
status: active
description: "Crate de capa cognitiva L0–L3: captura, extracción/dedup, escenas, persona, context engine, offload e ingest — alcance por binding y fuentes."
---

# Módulo: cognitive-memory (`vanta-memory`)

Responsabilidad: pipeline de memoria agéntica sobre el store del motor. Referencia canónica: [VANTA_MEMORY](../../api/VANTA_MEMORY.md).

## Qué es

`[HECHO]` Crate `vanta-memory`: pipeline L0 captura → L1 extracción/dedup → L2 escenas → L3 persona, más extracción de skills, context engine (estimador de tokens, compresión, spill) y offload con cursor (`vanta-memory/src/lib.rs:4-69`). Host-neutral: define la abstracción `LlmRunner` y sin `llm-driver` degrada a equivalentes LLM-free (`vanta-memory/src/lib.rs:15-18`). `[HECHO]` Toda la persistencia vive en el store VantaDB — nodos, particiones `InternalMetadata`, text index, HNSW, grafo (`vanta-memory/src/lib.rs:10-13`).

`[HECHO]` Submódulos ([`vanta-memory/src/lib.rs:38-64`](../../../vanta-memory/src/lib.rs#L38-L64)): `core` (conversation/record/scene/persona/skill/ontology/prompts/dream/hooks/state/reflection), `utils` (timers, checkpoint, backup, erasure), `services`, `adapters` (standalone vs openclaw LLM runners), `offload`, `context_engine`, `gateway` (handlers MCP scene/knowledge), `seed`, `ingest`.

## Alcance por binding

`[HECHO]` Decisión normativa (Gate P D42/D43 + DIST-03): **core-only para TS/Node/WASM** — bloqueantes wasm32 verificados (`SystemTime::now` panic, `std::fs` error, `reqwest::blocking` ausente; FIND-255) — y **superficie mínima aterrizada en Python**: `memory_capture`/`memory_recall` (DIST-02). El invariante mecánico exige `rg "vanta[_-]memory" vantadb-ts vantadb-node vantadb-wasm` → 0 matches (BINDINGS_NAMESPACES §Cognitive layer).

`[HECHO]` Estado de publicación: EXPERIMENTAL, publishable pero `release = false` en release-plz hasta bootstrap de Trusted Publishing (DIST-01; SURFACE_MANIFEST).

## Consumidores reales

`[HECHO]` `vantadb-mcp` (tools de escenas/sueños/wiki: `scenes.rs`, `dreams.rs`, `wiki.rs`, `gateway`), `vanta-proxy` y el scheduler de `vantadb-server` (`vantadb-server/src/scheduler.rs`, ADR-0054).

## Fuentes

`vanta-memory/` · [VANTA_MEMORY](../../api/VANTA_MEMORY.md) · ADR-0029/0040 · BINDINGS_NAMESPACES §Cognitive layer scope
