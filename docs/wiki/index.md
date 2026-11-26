---
title: "VantaDB Wiki"
kind: index
status: active
description: "Wiki documental de VantaDB: navegación por audiencia y mapa de cobertura del motor, bindings, servidor, MCP y ecosistema, basada en el código presente."
---

# VantaDB Wiki

Esta wiki es una capa de navegación editorial sobre el código y la documentación canónica del repositorio: explica y enlaza, no sustituye a los documentos normativos (`docs/api/`, ADRs, `docs/user/`). Es distinta de `src/wiki/` y `vantadb-mcp/src/wiki.rs`, que son una función del producto (wiki de conocimiento del agente), no documentación.

Convención de evidencia: cada afirmación sustantiva se etiqueta `[HECHO]` (verificable en fuente/doc citado), `[INFERENCIA]` (deducción razonada) o `[ESPECULACIÓN]` (hipótesis). Los ejemplos de código son ilustrativos y **no se han ejecutado** en esta sesión; las firmas citadas se contrastaron contra la fuente o los snapshots vigentes.

## Navegación por audiencia

- **Quiero usar VantaDB**: [quickstart](user/quickstart.md) · [selección de runtime](user/runtime-selection.md) · [búsqueda híbrida](user/hybrid-search.md) · [persistencia](user/persistence.md) · [operación](user/operations.md) · [troubleshooting](user/troubleshooting.md) · [FAQ](faq.md)
- **Quiero entender el diseño**: [overview](architecture/overview.md) · [flujo de datos](architecture/data-flow.md) · [concurrencia](architecture/concurrency.md) · [durabilidad](architecture/durability.md) · páginas por módulo en [modules/](modules/core.md)
- **Quiero la API**: [Rust](api/rust.md) · [Python](api/python.md) · [Node](api/node.md) · [TypeScript](api/typescript.md) · [WASM](api/wasm.md) · [HTTP](api/http.md) · [MCP](api/mcp.md) · [paridad](api/parity.md)
- **Quiero desarrollar el repo**: [mapa del código](developer/code-map.md) · [workflow local](developer/local-workflow.md) · [contratos](developer/contracts.md) · [verificación](developer/verification.md) · [contribuir](contributor/getting-started.md) · [checklist de cambio](contributor/change-checklist.md)

## Mapa de cobertura

Asignación de crates/paquetes y módulos públicos del core (`src/lib.rs`) a páginas. Los módulos `pub(crate)` se listan en cursiva: son internos y solo se agrupan aquí para trazabilidad. `[HECHO]` la lista de módulos deriva de [`src/lib.rs`](../../src/lib.rs) (declaraciones en [líneas 54-187](../../src/lib.rs#L54-L187); re-exportaciones en [líneas 190-222](../../src/lib.rs#L190-L222)); los nombres de canal y estado de cada superficie derivan de `docs/api/SURFACE_MANIFEST.md` (generado, workspace 0.9.0).

### Crates del workspace (`Cargo.toml` líneas 779-795 — ocho miembros)

| Superficie | Crate/paquete | Estado declarado en SURFACE_MANIFEST | Página de la wiki |
|---|---|---|---|
| `.` | `vantadb` | [HECHO] PUBLIC — 0.9.0 en crates.io ([metadata](https://crates.io/api/v1/crates/vantadb) verificada 2026-11-06) | [modules/core](modules/core.md) y resto de `modules/` |
| `vantadb-python` | `vantadb_py` → PyPI `vantadb-py` | [HECHO] PUBLIC — 0.9.0 en PyPI ([metadata](https://pypi.org/pypi/vantadb-py/json) verificada 2026-11-06) | [api/python](api/python.md) |
| `vantadb-wasm` | npm `vantadb-wasm` | [HECHO] PUBLIC — 0.9.0 en npm ([registry](https://registry.npmjs.org/vantadb-wasm) verificado 2026-11-06) | [api/wasm](api/wasm.md) |
| `vantadb-ffi-core` | `vantadb-ffi-core` | [HECHO] INTERNAL — hoja std compartida | [api/parity](api/parity.md) · [modules/ecosystem](modules/ecosystem.md) |
| `vantadb-server` | `vantadb-server` | [HECHO] PUBLIC — binarios GitHub Release | [api/http](api/http.md) · [api/mcp](api/mcp.md) |
| `vantadb-mcp` | `vantadb-mcp` | [HECHO] INTERNAL — embebido en el server | [api/mcp](api/mcp.md) |
| `vanta-memory` | `vanta-memory` | [HECHO] EXPERIMENTAL — publicable, release retenido | [modules/cognitive-memory](modules/cognitive-memory.md) |
| `vanta-proxy` | `vanta-proxy` | [HECHO] FROZEN — solo build desde fuente | [modules/proxy](modules/proxy.md) |

`[INFERENCIA]` La clasificación (PUBLIC/EXPERIMENTAL/…) proviene del manifiesto generado; la existencia del artefacto 0.9.0 está además comprobada contra la metadata de crates.io, npm (`vantadb`, `vantadb-wasm`, `vantadb-node`) y PyPI (`vantadb-py`) el 2026-11-06. Comprobar metadata no equivale a haber descargado ni ejecutado los artefactos; para los binarios GitHub el estado sigue siendo el declarado por el manifiesto.

### Paquetes fuera del workspace

| Ruta | Naturaleza | Página |
|---|---|---|
| `vantadb-node` | [HECHO] crate napi-rs + paquete npm `vantadb-node`; NO es miembro del workspace (ausente de `Cargo.toml:780-792`); publicado 0.9.0 en npm (verificado 2026-11-06 contra registry.npmjs.org); clasificación EXPERIMENTAL declarada en docs — el manifiesto generado está desactualizado en este punto | [api/node](api/node.md) |
| `vantadb-ts` | [HECHO] paquete npm `vantadb` (TypeScript puro, no crate cargo); envuelve `vantadb-wasm` y `vantadb-node` como dependencias `file:` (`vantadb-ts/package.json`); publicado 0.9.0 en npm (verificado 2026-11-06) | [api/typescript](api/typescript.md) |
| `providers/openai`, `providers/ollama`, `providers/litellm` | [HECHO] crates Rust + shim Python; NO son miembros del workspace (`Cargo.toml:796`); estado EXPERIMENTAL, canal PyPI preparado no publicado | [modules/ecosystem](modules/ecosystem.md) |
| `integrations/*` (langchain, llamaindex, crewai, dspy, haystack, letta, mem0, ollama, openai) | [HECHO] adaptadores Python con `pyproject.toml` propio | [modules/ecosystem](modules/ecosystem.md) |
| `fuzz/` | [HECHO] harness cargo-fuzz fuera del workspace (`Cargo.toml:793-795`) | [modules/evals](modules/evals.md) |
| `evals/` | [HECHO] harnesses Python de evaluación de memoria (locomo, longmemeval, head-to-head) | [modules/evals](modules/evals.md) |
| `vantadb-wasm/threads-kernel/` | [HECHO] crate kernel WASM experimental | [api/wasm](api/wasm.md) |

### Módulos públicos del core (`src/lib.rs`)

| Página | Módulos asignados (`pub mod`; *cursiva* = `pub(crate)`) |
|---|---|
| [modules/core](modules/core.md) | `sdk`, `node`, `config`, `error`, `serialization`, `metadata`, `schema` (versión/compartida con storage), `migration`, `entity`, `agentic`, `wiki`, `skills`, `crypto`, `audit`, `attestation`, `sync_ext`, `telemetry`, `metrics`, `hardware`, `embedding_health`, `utils`, `accumulator`, `api`, `llm`, `integrations`, `cli`, `cli_complete`, `cli_handlers`, `cli_server`, `console`, `tui`, `transcript`, `python` (puente in-crate), `testing` |
| [modules/indexes](modules/indexes.md) | `index`, `index_port`, `vector`, *`text_index`* (spec pública re-exportada), `tokenizer`, *`scalar_index`* |
| [modules/storage](modules/storage.md) | `storage`, `wal`, *`wal_sharded`*, `wal_shipping`, `binary_header`, `shred`, `columnar`, `ingestion`, *`backend`*, *`backends`*, *`lsm`*, *`cache_warmer`* |
| [modules/retrieval](modules/retrieval.md) | `search_profile`, y la rama `sdk::search` (fusión RRF, MMR, snippets, multi-namespace, explain) |
| [modules/query](modules/query.md) | `query`, `parser`, `planner`, `physical_plan`, `executor`, `operator_registry`, *`cost_estimator`* |
| [modules/memory-budget](modules/memory-budget.md) | `governor`, *`memory_governor`*, `eviction`, `gc`, `vector::governor` |
| [modules/graph](modules/graph.md) | `graph`, `graphrag`, `gds`, *`edge_index`* |
| [modules/cognitive-memory](modules/cognitive-memory.md) | crate `vanta-memory` (pipeline L0–L3) + hooks Python `memory_capture`/`memory_recall` |
| [modules/proxy](modules/proxy.md) | crate `vanta-proxy` |
| [modules/ecosystem](modules/ecosystem.md) | `server`, `circuit_breaker`, `connection_pool`, *`rbac`*; crates `vantadb-server`, `vantadb-mcp`, `vantadb-ffi-core`, bindings, `providers/*`, `integrations/*` |
| [modules/evals](modules/evals.md) | `evals/`, `benches/`, `benchmarks/`, `fuzz/`, `tests/` |
| [modules/tooling](modules/tooling.md) | `scripts/`, `dev-tools/`, `.github/workflows/`, `Justfile`, toolchain de docs |

`[INFERENCIA]` La agrupación es editorial: un módulo puede tocar varias páginas (p. ej. `schema` cubre versionado de almacenamiento pero se documenta en core); la tabla asigna una página primaria sin afirmar exhaustividad funcional.

## Documentos canónicos que esta wiki enlaza (no duplica)

`[HECHO]` Autoridad por dominio según `docs/api/SURFACE_MANIFEST.md` §Authority map: API Rust → `tests/api/public-api.txt`; HTTP → `docs/api/openapi.yaml`; superficie por binding → `docs/api/BINDINGS_NAMESPACES.md`; versiones → `Cargo.toml` y `package.json`/`pyproject.toml`; decisiones → `docs/dev/architecture/adr/`; benchmarks → `docs/user/operations/BENCHMARKS.md`.
