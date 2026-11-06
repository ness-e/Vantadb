---
title: "FAQ de VantaDB"
kind: concept
status: active
description: "Preguntas frecuentes con respuesta verificable en el código o la documentación canónica."
---

# FAQ de VantaDB

El FAQ canónico para usuarios es [docs/user/FAQ.md](../user/FAQ.md). Aquí solo preguntas con respuesta verificable en fuente.

**¿VantaDB es un servidor o una librería?**
`[HECHO]` Primariamente un crate embebido ([`Embedded` en `src/sdk/builder.rs:15`](../../src/sdk/builder.rs#L15)); el servidor HTTP+MCP (`vantadb-server`) es una superficie opcional distribuida como binario.

**¿Necesito un servicio de embeddings?**
`[HECHO]` No por defecto: VantaDB es BYO-vector ([EMBEDDINGS](../api/EMBEDDINGS.md)); hay ruta opt-in local (`embed-local`, ONNX) y remota (`remote-inference`).

**¿Pueden varios procesos escribir en la misma base de datos?**
`[HECHO]` No — single-writer por `.vanta.lock`. Multi-instancia: HTTP o MCP con fallback (ADR-0036). Multi-escritor LWW está decidido como diseño (ADR-0055), no como disponibilidad general verificada aquí.

**¿Qué diferencia hay entre `vantadb` (npm), `vantadb-node` y `vantadb-wasm`?**
`[HECHO]` `vantadb` es el wrapper TS que corre sobre el artefacto WASM; `vantadb-wasm` es el binding crudo; `vantadb-node` es el binding nativo napi-rs. Los tres están publicados en npm a 0.9.0 — comprobado contra el registry el 2026-11-06 (el SURFACE_MANIFEST quedó desactualizado en `vantadb-node`, que conserva la clasificación EXPERIMENTAL declarada de madurez) ([runtime-selection](user/runtime-selection.md)).

**¿La capa cognitiva está en todos los SDKs?**
`[HECHO]` No: solo Python expone `memory_capture`/`memory_recall`; TS/Node/WASM son core-only por decisión documentada (BINDINGS_NAMESPACES §Cognitive layer).

**¿`score` o `distance`?**
`[HECHO]` `score` (híbrida, mayor=mejor) vs `distance` (ANN crudo, menor=mejor) en todos los transportes — ver [scores](../api/scores.md).

## Fuentes

[FAQ canónico](../user/FAQ.md) · [index](index.md) · BINDINGS_NAMESPACES · SURFACE_MANIFEST
