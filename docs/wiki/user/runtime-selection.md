---
title: "Selección de runtime"
kind: howto
status: active
description: "Qué superficie elegir: Rust embebido, Python, TypeScript/WASM, Node nativo, HTTP o MCP, con estado de publicación real de cada una."
---

# Selección de runtime

## Matriz de decisión

`[HECHO]` Estado por superficie en 0.9.0 ([SURFACE_MANIFEST](../../api/SURFACE_MANIFEST.md)):

| Runtime | Paquete | Estado | Cuándo elegirlo |
|---|---|---|---|
| Rust | `vantadb` (crates.io) | [HECHO] PUBLIC — 0.9.0 ([metadata](https://crates.io/api/v1/crates/vantadb) verificada 2026-11-06) | Acceso directo al motor y SDK embebido completo |
| Python | `vantadb-py` (PyPI) | [HECHO] PUBLIC — 0.9.0 ([metadata](https://pypi.org/pypi/vantadb-py/json) verificada 2026-11-06) | `[INFERENCIA]` opción por defecto para agentes/scripts: la superficie binding más amplia, incl. capa cognitiva mínima |
| TypeScript | `vantadb` (npm) | [HECHO] PUBLIC — 0.9.0 ([registry](https://registry.npmjs.org/vantadb) verificado 2026-11-06) | JS/TS multi-runtime: navega sobre WASM; misma API en Node/Bun/Deno/browser |
| Node nativo | `vantadb-node` | [HECHO] Publicado 0.9.0 en npm ([registry](https://registry.npmjs.org/vantadb-node) verificado 2026-11-06); clasificación EXPERIMENTAL declarada | Node con motor nativo, sin límites wasm32 |
| HTTP | binario `vantadb-server` | [HECHO] PUBLIC — binarios GitHub Release | Multi-proceso/multi-lenguaje; única superficie con página `SearchPageV2` y señal de abstinencia |
| MCP | `vanta-cli server --mcp` | [HECHO] INTERNAL embebido | Integración con IDEs/agentes MCP (Claude Code, Cursor, OpenCode) |
| Rust WASM crudo | `vantadb-wasm` (npm) | [HECHO] PUBLIC — 0.9.0 ([registry](https://registry.npmjs.org/vantadb-wasm) verificado 2026-11-06) | Solo si el wrapper TS no cubre un caso |

`[INFERENCIA]` Comprobar metadata de registry certifica existencia y versión, no instalación probada ni compatibilidad por plataforma.

## Reglas prácticas

`[HECHO]` Single-writer: una base de datos = un proceso escritor; multi-instancia exige HTTP/MCP con fallback (ADR-0036). `[HECHO]` Browser real solo vía `vantadb`/`vantadb-wasm` (OPFS/IDB); persistencia por runtime en [WASM_PERSISTENCE](../../api/WASM_PERSISTENCE.md). `[INFERENCIA]` Si necesitas IQL desde Node nativo, hoy no existe método (FIND-268) — usa Python/WASM/HTTP.

## Fuentes

[SURFACE_MANIFEST](../../api/SURFACE_MANIFEST.md) · [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md) · páginas [api/](../api/rust.md) individuales
