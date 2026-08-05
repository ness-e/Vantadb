# Public API & Contract — Reglas

> **Scope:** `src/sdk/` (`builder.rs` VantaEmbedded, `api.rs`, `search/`, `serialization/`, `types.rs`, `connect.rs`, `graph.rs`, `gds.rs`), `node.rs`, `error.rs` (`VantaError`), `config.rs`, versionado semver, sync de bindings
> **No tocar aquí:** release/CI (`release-ci.md`), bindings específicos (`python-bindings.md`, `js-ecosystem.md`, `server-mcp.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** VFY-002, INV-011, auditoría docs/api 2026-08-04

## Reglas

### R-1: Todo claim de API en docs debe apuntar a un símbolo real

- **Must:** antes de documentar una función/clase/método/endpoint/tool en `docs/api/`, verificar que existe en el código y bindings reales (`codegraph_explore`/grep).
- **Must not:** documentar APIs especulativas o planeadas como existentes (ej. `vantadb_python.Client`/`client.graphrag_search(...)` — 0 hits en Python/TS/WASM; `GraphRagPipeline` existe en Rust pero ningún binding lo expone. GRAPH_RAG.md lo documentaba así hasta AUD-002, 2026-08-05, cuando se marcó Python como no-implementado).
- **Por qué:** un usuario que sigue el ejemplo de docs falla en runtime; GRAPH_RAG.md documentaba una API inexistente — el patrón sigue prohibido.

### R-2: Versión única de referencia en docs de API

- **Must:** `docs/api/openapi.yaml` y cualquier header de versión en docs usar `[workspace.package] version` (hoy 0.5.0) como única fuente.
- **Must not:** hardcodear versiones divergentes (hoy: openapi.yaml `0.4.0`, MCP.md `0.1.5`, HTTP_API.md `0.0.4` — tres versiones distintas en el mismo docset).
- **Por qué:** el drift de versiones hace imposible saber qué versión describe el doc; release-plz bumpa el workspace y los docs quedan atrás.

### R-3: No exponer en bindings/MCP APIs que el core rechaza por defecto

- **Must:** un tool MCP o método binding solo puede invocar código del core que funcione con las features default.
- **Must not:** exponer `query_lisp` (MCP) mientras LISP requiere `experimental-lisp` y no existe `src/parser/lisp.rs` — la tool existe pero siempre devuelve error.
- **Por qué:** una API "viva" que falla siempre erosiona confianza y genera issues falsos (INV-011 detectó el drift `query` vs `query_lisp`).

### R-4: Referencias de archivo exactas en docs

- **Must:** citar rutas reales (`src/sdk/api.rs`, `src/sdk/builder.rs`).
- **Must not:** citar `src/sdk.rs` cuando el módulo es `src/sdk/` (EMBEDDED_SDK.md lo cita; no existe), ni usar el prefijo falso `vantadb/src/...` para la crate raíz.
- **Por qué:** navegación fallida para lectores y agentes; la auditoría encontró 2 convenciones de ruta falsas.

### R-5: Paridad tools MCP / endpoints HTTP entre código y docs en el mismo PR

- **Must:** al añadir/renombrar/eliminar un tool en `handle_tools_list()` (`vantadb-mcp/src/lib.rs:808-964`) o una ruta HTTP, actualizar `docs/api/MCP.md`/`HTTP_API.md` en el mismo cambio.
- **Must not:** mergear tools nuevos sin doc (hoy 8/15 tools MCP sin documentar: `query_lisp`, `get_node_neighbors`, `inject_context`, `read_axioms`, `collection_stats`, `collection_list`, `collection_delete`, `rehydrate`).
- **Por qué:** el doc de MCP documenta solo 7 de 15 tools reales; sin regla el gap crece.

### R-6: Enums públicos en crecimiento: `#[non_exhaustive]` obligatorio

- **Must:** todo enum público que pueda ganar variantes (errores, modos, backends) declara `#[non_exhaustive]`.
- **Must not:** hacer match exhaustivo sin wildcard `_` sobre un enum público de otra crate; añadir variantes de error como `String` crudo cuando existe el patrón `ChainedError`.
- **Por qué:** `VantaError` ya es `#[non_exhaustive]`, pero 4 variantes (`InvalidInput`, `SchemaError`, `DatabaseBusy`, `NoVectorForKey`) son `String` crudo — pierden chaining `.source()`. Regla 4 de AGENTS.md (P2-6).

### R-7: No gatear campos `pub` de structs con `cfg(feature)`

- **Must not:** declarar un campo `pub` de un struct público bajo `#[cfg(feature = "...")]` — el literal de struct se rompe entre feature sets (API no-commutativa).
- **Must:** si un campo solo existe con feature, usar `Option<T>` interno, `#[doc(hidden)]`, o gatear el struct completo.
- **Por qué:** `src/config.rs:617` (`advanced_tokenizer_config`) desaparece sin `advanced-tokenizer`; consumidores con otra combinación de features fallan en compilación sin relación con su código.

<!-- Referencias cruzadas: → ver release-ci.md, python-bindings.md, js-ecosystem.md, server-mcp.md -->
