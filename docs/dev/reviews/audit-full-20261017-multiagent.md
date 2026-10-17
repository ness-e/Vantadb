---
title: "Auditoría Integral Multi-Agente VantaDB v0.8.0 (5 Dominios + Gates)"
kind: review
status: active
description: "Auditoría completa de código, módulos, paquetes, bindings FFI, servicios y documentación orquestada en 5 dominios con verificación empírica de gates"
tags: [vantadb, audit, security, storage, ffi, packaging, documentation]
---

# Auditoría Integral Multi-Agente VantaDB v0.8.0 (5 Dominios + Gates)

> **Fecha:** 2026-10-17
> **Modo:** `full` (5 sub-agentes especializados + validación empírica de gates mecánicos y unit tests)
> **Cobertura:** Dominio 1 (Core/Storage/Indexes/Concurrency), Dominio 2 (Services/Server/MCP/Proxy/Memory), Dominio 3 (FFI/Python/WASM/Node/TS), Dominio 4 (Packages/Manifests/Providers/Integrations), Dominio 5 (Docs/API Contracts/Governance)
> **Trazabilidad:** Todos los 34 hallazgos han sido registrados bajo el esquema canónico de 10 columnas en [`docs/dev/Backlog.md`](../Backlog.md) (`FIND-345` a `FIND-378`) y ejecutados en la campaña de remediación inmediata.

---

## 1. Resumen Ejecutivo y Estado de Gates

| Métrica / Gate | Estado Inicial (Pre-Fix) | Hallazgos Asociados |
|---|---|---|
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ PASS (0 warnings, 8 crates) | — |
| `cargo nextest run --profile audit -p vantadb --lib` | ✅ PASS (2,450/2,450 tests; 1 contención CPU aislada) | — |
| `cargo nextest run --profile audit -p vanta-memory -p vantadb-server -p vantadb-mcp -p vantadb-ffi-core --lib` | ✅ PASS (451/451 tests) | — |
| `node scripts/docs/check-links.mjs` | ✅ PASS (0 broken links) | — |
| `cargo deny check` | ❌ FAIL (4 advisories `rustls`, 1 `unsafe-libyaml`) | `FIND-351` |
| `node scripts/docs/check-docs.mjs` | ❌ FAIL (`missingFrontmatter=1` en `borrada-2026-10-10.md`) | `FIND-364` |

### Distribución por Severidad (34 Hallazgos)

- **🔴 CRITICAL (6):** `FIND-345`..`FIND-350`
- **🟠 HIGH (11):** `FIND-351`..`FIND-361`
- **🟡 MEDIUM (11):** `FIND-362`..`FIND-372`
- **🟢 LOW (6):** `FIND-373`..`FIND-378`

---

## 2. Bloqueantes Críticos (`CRITICAL` — 6 Hallazgos)

### `FIND-345` (VDB-D1-01) — Inversión de orden de locks (Deadlock) en `commit_transaction`
- **Ubicación:** `src/storage/engine/txn.rs:316-333` vs `src/storage/engine/insert.rs:183,208` y `src/storage/engine/delete.rs:36,42`
- **FMEA:** En el flujo estándar (`insert.rs`, `delete.rs`) y acorde a `.opencode/rules/concurrency-async.md` (Regla 8), el motor actualiza estadísticas (`cardinality_stats.write()`) fuera de `insert_lock` o antes de adquirirlo. En `commit_transaction` (`txn.rs:316`), se adquiere primero `_guard = self.acquire_insert_lock()` y, manteniéndolo vivo, se invoca `apply_insert_stats` / `apply_delete_stats` en `:333`, que adquiere `cardinality_stats.write()`. Esto genera un ciclo AB-BA clásico bajo contención concurrente entre transacciones y escrituras directas.
- **Remediación:** Liberar explícitamente `_guard` (`drop(_guard)`) inmediatamente después de confirmar la escritura en backend/vstore y antes de aplicar `apply_insert_stats` / `apply_delete_stats`, o unificar el orden de adquisición.

### `FIND-346` (D2-FIND-01) — Bypass total de autenticación cuando se configura exclusivamente JWT (`jwt_secret` sin `api_key`)
- **Ubicación:** `src/server/middleware.rs:72-79, 126-130` · `src/server/bootstrap.rs:47-58`
- **FMEA:** En `auth_middleware`, la guarda `let Some(expected) = &state.config.auth.api_key else { return next.run(req).await; }` cortocircuita la ejecución asumiendo "modo desarrollo" sin verificar `state.config.auth.jwt_secret`. Toda la lógica de validación Bearer JWT (`middleware.rs:88-115`) queda muerta si no se define simultáneamente una `api_key` estática. Además, `validate_auth_config` en `bootstrap.rs:47` rechaza iniciar con `require_auth = true` si solo se provee `jwt_secret`.
- **Remediación:** Evaluar conjuntamente `api_key.is_none() && jwt_secret.is_none()` para el modo sin autenticación, permitir validación JWT pura cuando `jwt_secret.is_some()`, y actualizar `validate_auth_config` en `bootstrap.rs`.

### `FIND-347` (D2-FIND-02) — Escritura y lectura arbitraria de archivos vía HTTP `/api/v2/export` y `/api/v2/import`
- **Ubicación:** `src/server/handlers.rs:773-822` · `src/sdk/serialization/impl_export.rs:97-105` · `src/storage/ops.rs:228-241`
- **FMEA:** Cuando `config.export_base_dir` es `None` (valor por defecto), `resolve_export_path` delega en `prevent_path_traversal(raw_path)`. Esta función rechaza componentes `..`, pero permite rutas absolutas (`Path::is_absolute()`), posibilitando que un cliente HTTP escriba un dump JSONL o lea archivos en cualquier ruta del sistema de archivos donde el proceso tenga permisos.
- **Remediación:** En `resolve_export_path` (o en los handlers HTTP `/api/v2/export` e `/api/v2/import` cuando no hay `export_base_dir` configurado), rechazar rutas absolutas o confinar por defecto al subdirectorio `exports` dentro de `storage_path`.

### `FIND-348` (FFI-01) — Re-adquisición del GIL dentro de `py.detach` y `par_iter()` de Rayon en `vantadb-python`
- **Ubicación:** `vantadb-python/src/convert.rs:874-881` · `vantadb-python/src/lib.rs:354,389,460,798,841,1092,1349,1982-2002`
- **FMEA:** `map_vanta_error` invoca `attach_err_meta`, que ejecuta `Python::attach(|py| ...)` para adquirir el GIL. Sin embargo, múltiples métodos en `vantadb-python/src/lib.rs` llaman `.map_err(map_vanta_error)` *dentro* del closure de `py.detach(|| ...)` e incluso dentro de `requests.par_iter().map(...)` de Rayon (`search_batch_requests`, `:1982`). Si el hilo principal de Python intenta adquirir un lock de Rust o sincronizar con el pool de Rayon mientras un worker de Rayon espera el GIL en `Python::attach`, se produce un deadlock determinista.
- **Remediación:** Extraer todas las llamadas a `.map_err(map_vanta_error)` fuera de los bloques `py.detach(...)` y recolectar `Result<Vec<_>, vantadb::error::Error>` en Rayon antes de mapear a `PyErr` con el GIL ya adquirido.

### `FIND-349` (PKG-01) — Rutas relativas rotas en `feature-freez/desktop/src-tauri/Cargo.toml`
- **Ubicación:** `feature-freez/desktop/src-tauri/Cargo.toml:42,46`
- **FMEA:** Tras mover `desktop/` a `feature-freez/desktop/`, las dependencias por path `vantadb = { path = "../.." }` y `vanta-memory = { path = "../../vanta-memory" }` apuntan a `feature-freez/` en lugar de la raíz del repositorio, rompiendo cualquier compilación del crate Tauri.
- **Remediación:** Actualizar a `path = "../../.."` y `path = "../../../vanta-memory"`.

### `FIND-350` (PKG-02) — Dependencia local `"vantadb-node": "file:../vantadb-node"` en `vantadb-ts/package.json`
- **Ubicación:** `vantadb-ts/package.json:64` · `.github/workflows/release-npm.yml:87-96`
- **FMEA:** `vantadb-ts/package.json` declara `"vantadb-node": "file:../vantadb-node"` en `dependencies` de producción. El workflow de publicación `release-npm.yml` solo reescribe la versión de `vantadb-wasm`, dejando el protocolo `file:../vantadb-node` intacto en el tarball publicado en npm o fallando en clientes externos dado que `vantadb-node` aún no está publicado en el registry (`FIND-331`).
- **Remediación:** Mover `vantadb-node` a `devDependencies` (o `optionalDependencies` gestionado en el script de empaquetado/release) dado que `vantadb-ts/src/native.ts` lo carga dinámicamente vía `createRequire` en runtime.

---

## 3. Hallazgos de Severidad Alta (`HIGH` — 11 Hallazgos)

### `FIND-351` (SEC-DENY) — Gate `cargo deny check` en rojo por 4 advisories en `rustls` y 1 en `unsafe-libyaml`
- **Ubicación:** `Cargo.lock` · `Cargo.toml:765` · `deny.toml:40-52`
- **FMEA:** `rustls v0.23.45` presenta 4 vulnerabilidades (`RUSTSEC-2026-0346`, `0347`, `0348`, `0349`, parcheadas en `>=0.23.46`). Adicionalmente, `serde_yaml` en `[dev-dependencies]` arrastra `unsafe-libyaml v0.2.11` (`RUSTSEC-2024-0449`, unmaintained).
- **Remediación:** Actualizar `rustls` a `>=0.23.46` en `Cargo.lock` y reemplazar/gestionar `serde_yaml` / `RUSTSEC-2024-0449`.

### `FIND-352` (VDB-D1-02) — Corrupción silenciosa del grafo HNSW para nodos cuantizados/mmap en `apply_index_entry_unlocked`
- **Ubicación:** `src/storage/engine/maintenance.rs:169-192` · `src/index/graph/core.rs:254`
- **FMEA:** `apply_index_entry_unlocked` solo preserva el vector cuando `node.vectors` coincide con `VectorRepresentations::Full(vec)`. Para nodos con `SQ8`, `Binary`, `Turbo` o `MmapFull`, construye un nodo sombra con `VectorRepresentations::None` y llama a `vector_index.add(&index_node)`. En `HnswIndex::add` (`core.rs:254`), recibir `VectorRepresentations::None` dispara la purga del nodo de todas las listas de vecinos HNSW, desconectándolo silenciosamente del índice vectorial.
- **Remediación:** Clonar o reconstruir la representación vectorial real (`node.vectors.clone()`) en `apply_index_entry_unlocked` (o materializar `MmapFull` cuando aplique) en lugar de degradar a `VectorRepresentations::None`.

### `FIND-353` (VDB-D1-03) — Truncamiento de IDs `u128` a `u64` tras recuperación WAL en `InMemoryEngine`
- **Ubicación:** `src/engine.rs:209,218-220`
- **FMEA:** `InMemoryEngine::with_wal` calcula `max_id: u128` sobre todos los nodos recuperados (que pueden incluir UUIDs deterministas de 128 bits generados por `derive_node_id`), y luego inicializa `next_id: AtomicU64::new((max_id + 1) as u64)`. El cast trunca los 64 bits superiores y puede reinicializar `next_id` en un valor bajo que colisiona con IDs secuenciales existentes.
- **Remediación:** Filtrar `max_id` considerando únicamente IDs dentro del rango secuencial `<= u64::MAX` (`nodes.keys().copied().filter(|&id| id <= u64::MAX as u128).max()`), usando `saturating_add(1)`.

### `FIND-354` (VDB-D1-04) — Replay fuera de orden en `ShardedWal::recover`
- **Ubicación:** `src/wal_sharded.rs:481-524`
- **FMEA:** `ShardedWal::recover` itera secuencialmente por shard (`0..num_shards`) e invoca el callback con todos los registros del Shard 0 antes que los del Shard 1, ignorando `global_seq`. Operaciones causales entre distintos shards (ej. `DeleteNode` o `Clear` frente a `InsertNode`) se reproducen fuera de orden cronológico global.
- **Remediación:** Recolectar los registros con secuencia de todos los shards, ordenarlos por `global_seq` ascendente y luego invocar el callback de aplicación.

### `FIND-355` (D2-FIND-03) — Doble offset de cursor en paginación multi-namespace (`list_records`)
- **Ubicación:** `src/server/list_records.rs:107-130`
- **FMEA:** Cuando `cmd.namespace` es `None`, el bucle aplica `cursor: cmd.cursor.clone()` a cada llamada individual `engine.list_in(Some(ns), ...)` (saltando `start` registros en *cada* namespace) y posteriormente vuelve a aplicar `records[start..end]` sobre el vector fusionado. Esto omite registros válidos y devuelve páginas vacías prematuramente.
- **Remediación:** Al listar todos los namespaces, invocar `list_in` con `cursor: None` y `limit: start.saturating_add(cmd.limit)` por namespace, fusionar y aplicar el corte global `[start..end]` una única vez.

### `FIND-356` (D2-FIND-04) — Operaciones síncronas de almacenamiento bloqueando workers Tokio en `vanta-proxy`
- **Ubicación:** `vanta-proxy/src/capture.rs:158` · `vanta-proxy/src/memory_tools.rs:175,220` · `vanta-proxy/src/server.rs:448`
- **FMEA:** Los handlers HTTP asíncronos de `vanta-proxy` invocan directamente métodos síncronos de `MemoryEngine` (`append_turn`, `execute`, `perform_auto_recall`) que realizan I/O de disco (Fjall/WAL) e inferencia local sin `tokio::task::spawn_blocking`, pudiendo agotar los hilos del runtime Tokio bajo concurrencia.
- **Remediación:** Envolver las llamadas síncronas pesadas a `MemoryEngine` en `tokio::task::spawn_blocking`.

### `FIND-357` (FFI-02) — Extracción de vectores sin validar NaN/Inf, dimensión máxima ni buffers 1D en `vantadb-python`
- **Ubicación:** `vantadb-python/src/convert.rs:201-244`
- **FMEA:** `extract_vector` acepta tensores/arrays multidimensionales aplanándolos silenciosamente vía `PyBuffer<f32>` sin verificar `buf.dimensions() == 1`, no aplica cota `MAX_VEC_DIM` antes de reservar memoria, y no valida `val.is_finite()`, permitiendo que `NaN`/`Inf` entren al grafo HNSW por caminos que no pasan por `Embedded::put`.
- **Remediación:** Validar `buf.dimensions() == 1`, longitud `<= vantadb_ffi_core::MAX_VEC_DIM` y que todos los componentes sean finitos (`is_finite()`), lanzando `VantaValueError` en caso contrario.

### `FIND-358` (FFI-03) — `vantadb-wasm` trunca `top_k` silenciosamente sin usar `vantadb_ffi_core::clamp_top_k`
- **Ubicación:** `vantadb-wasm/src/lib.rs:1274,1346,1380,1487`
- **FMEA:** Mientras Python y Node utilizan `vantadb_ffi_core::clamp_top_k` (que emite warning diagnóstico al truncar `top_k > 10_000`), `vantadb-wasm` aplica `.min(vantadb_ffi_core::MAX_K as usize)` de forma silenciosa en 4 puntos.
- **Remediación:** Utilizar `vantadb_ffi_core::clamp_top_k` en `vantadb-wasm` y emitir `web_sys::console::warn_1` cuando `clamped == true`.

### `FIND-359` (FFI-04) — Brecha de paridad entre `vantadb-ts/src/native.ts` y `vantadb-node`
- **Ubicación:** `vantadb-ts/src/native.ts` · `vantadb-node/src/lib.rs`
- **FMEA:** `NativeVantaDB` en `vantadb-ts/src/native.ts` no expone varios métodos ya implementados en el addon nativo `vantadb-node` (`listRecordVersions`, `getRecordAtVersion`, `addEdge`, `getEdges`, `removeEdge`, `findNodesByLabel`, `graphBfs`, `graphShortestPath`, `metricsPrometheus`), provocando que aplicaciones TS que cambian de WASM a backend nativo pierdan métodos del contrato.
- **Remediación:** Exponer los métodos delegados faltantes en `NativeVantaDB` (`vantadb-ts/src/native.ts`) e interfaz asociada.

### `FIND-360` (DOC-H01) — `db.search` en Python exige `query_vector` posicional obligatorio, contradiciendo `PYTHON_SDK.md`
- **Ubicación:** `vantadb-python/src/lib.rs:1295` · `vantadb-python/vantadb_py/__init__.pyi:50` · `docs/api/PYTHON_SDK.md:455`
- **FMEA:** La documentación oficial (`PYTHON_SDK.md:455`) muestra búsquedas léxicas puras `db.search("docs", text="rust")`, pero la firma PyO3 `#[pyo3(signature = (namespace, query_vector, top_k=10, ...))]` carece de `=None` en `query_vector`, lanzando `TypeError: VantaDB.search() missing 1 required positional argument: 'query_vector'`.
- **Remediación:** Añadir `query_vector=None` por defecto en `#[pyo3(signature = ...)]` de `search` en `vantadb-python/src/lib.rs` y sincronizar los stubs `.pyi`.

### `FIND-361` (DOC-H02) — Tabla Markdown corrupta en `docs/dev/reports/INDEX.md`
- **Ubicación:** `docs/dev/reports/INDEX.md:55-68`
- **FMEA:** En la línea 55, el pie de página `_Generated by vanta-lead..._` quedó concatenado en la misma línea que `| 2026-08-22 | integración final | ...`, rompiendo el renderizado de las 14 filas posteriores de reportes (agosto–septiembre 2026) y mezclando esquemas de 6 y 7 columnas.
- **Remediación:** Reintegrar las filas posteriores dentro de la tabla principal normalizada a 7 columnas y colocar el pie de página al final del documento.

---

## 4. Hallazgos de Severidad Media (`MEDIUM` — 11 Hallazgos)

### `FIND-362` (VDB-D1-05) — Bloques `unsafe` sin comentario `// SAFETY:` ni validación de alineación en `txn.rs`
- **Ubicación:** `src/storage/engine/txn.rs:548-553, 571-576`
- **FMEA:** `get_with_snapshot` convierte `&[u8]` a `&[f32]` mediante `std::slice::from_raw_parts(slice.as_ptr() as *const f32, ...)` sin comentario `// SAFETY:` (violando `.opencode/rules/core-engine.md` Regla 4) y sin garantizar alineación de 4 bytes (UB si el buffer subyacente no está alineado).
- **Remediación:** Reemplazar ambos bloques `unsafe` por decodificación segura por chunks (`chunks_exact(4)` + `f32::from_le_bytes`) o el helper `decode_full_bytes` existente en `get.rs`.

### `FIND-363` (VDB-D1-06) — Mutación prematura de índices secundarios antes de validar escritura y WAL en `insert.rs` y `delete.rs`
- **Ubicación:** `src/storage/engine/insert.rs:183-208` · `src/storage/engine/delete.rs:36-46`
- **FMEA:** `insert` y `delete` ejecutan `apply_insert_stats` / `apply_delete_stats` (que mutan `edge_index` y `scalar_index`) *antes* de verificar `ensure_writable()`, adquirir `insert_lock` y escribir en el WAL. Si el motor está en modo read-only o falla la escritura WAL, quedan entradas fantasma en los índices secundarios.
- **Remediación:** Mover `apply_insert_stats` y `apply_delete_stats` después de confirmar la escritura en el WAL/backend (y fuera de `insert_lock` para respetar la jerarquía de locks de `FIND-345`).

### `FIND-364` (DOC-M01) — Gate `check-docs.mjs` falla por falta de frontmatter en `borrada-2026-10-10.md` y claves legacy en docs
- **Ubicación:** `docs/dev/historial/borrada-2026-10-10.md:1` · `docs/api/PYTHON_SDK.md:1-14` · `docs/api/EMBEDDED_SDK.md:1-16` · `docs/user/operations/BENCHMARKS.md:1-15`
- **FMEA:** `docs/dev/historial/borrada-2026-10-10.md` carece de frontmatter YAML (`title`, `kind`), causando exit code 1 en `node scripts/docs/check-docs.mjs`. Además, `PYTHON_SDK.md`, `EMBEDDED_SDK.md` y `BENCHMARKS.md` conservan claves retiradas (`type`, `last_reviewed`, `related`).
- **Remediación:** Añadir frontmatter válido (`kind: research`) a `borrada-2026-10-10.md` y limpiar las claves retiradas en los tres documentos.

### `FIND-365` (D2-FIND-05) — Crecimiento de memoria no acotado en `vanta-proxy` (`writeback.rs`) y `vantadb-mcp` (`wiki.rs`)
- **Ubicación:** `vanta-proxy/src/writeback.rs:72-74` · `vantadb-mcp/src/wiki.rs:400,448`
- **FMEA:** En `vanta-proxy`, la cola `inner.pending: Mutex<Vec<PendingEntry>>` crece sin límite si los clientes nunca llaman `/v1/memory/discard` o `/commit`. En `vantadb-mcp`, el mapa estático `RUNS` acumula el historial de ejecuciones de `wiki_ingest` indefinidamente.
- **Remediación:** Imponer una cota máxima (`MAX_PENDING_ENTRIES = 10_000` y `MAX_INGEST_RUNS = 1_000`) con desalojo de las entradas más antiguas al alcanzar el límite.

### `FIND-366` (D2-FIND-06) — Falta de validación `validate_safe_path` en `code_index` y `wiki_ingest` de `vantadb-mcp`
- **Ubicación:** `vantadb-mcp/src/code_index.rs:104-124` · `vantadb-mcp/src/wiki.rs:269-275`
- **FMEA:** A diferencia de `bulk_import_file`, las herramientas MCP `code_index` y `wiki_ingest` construyen `Path::new(root)` directamente sin invocar `validate_safe_path`, permitiendo rutas con segmentos `..` (path traversal).
- **Remediación:** Invocar `crate::handlers::validate_safe_path` sobre el argumento de ruta antes de acceder al sistema de archivos en `code_index.rs` y `wiki.rs`.

### `FIND-367` (FFI-05) — Firma duplicada `search_vector` en `.pyi` y métodos delegados faltantes en `AsyncClient` (Python)
- **Ubicación:** `vantadb-python/vantadb_py/__init__.pyi:171,232` · `vantadb-python/vantadb/__init__.py` · `vantadb-python/vantadb/__init__.pyi`
- **FMEA:** `AsyncClient` en `vantadb_py/__init__.pyi` define `async def search_vector` dos veces (`:171` y `:232`), generando errores en mypy/pyright, y no expone `graphrag_search` ni `graph_bfs_filtered` en `AsyncClient`.
- **Remediación:** Eliminar la definición duplicada de `search_vector` en `vantadb_py/__init__.pyi` y añadir los delegados `graphrag_search` y `graph_bfs_filtered` en `AsyncClient`.

### `FIND-368` (PKG-04/05) — Desfase de versiones y cotas `<0.7.0` en `providers/*` e `integrations/*`
- **Ubicación:** `providers/{openai,ollama,litellm}/Cargo.toml:3` · `providers/{openai,ollama,litellm}/pyproject.toml:7` · `integrations/{openai,ollama}/pyproject.toml:26`
- **FMEA:** Los crates `providers/*` mantienen versión `0.5.0` y los adaptadores `integrations/openai` e `integrations/ollama` fijan `vantadb-py>=0.5.0,<0.7.0`, impidiendo su instalación conjunta con `vantadb-py >= 0.7.0` / `0.8.0`.
- **Remediación:** Ampliar el rango de compatibilidad en `integrations/{openai,ollama}/pyproject.toml` a `vantadb-py>=0.5.0,<0.9.0` y sincronizar `vantadb-node/Cargo.toml` con `vantadb-node/package.json`.

### `FIND-369` (PKG-06) — `tests/version_coherence.rs` verifica ruta inexistente `packages/langchain-vantadb`
- **Ubicación:** `tests/version_coherence.rs:149,169`
- **FMEA:** El test `python_wrappers_declare_compatible_vantadb_py_dependency` busca `packages/langchain-vantadb/pyproject.toml`. Como ese directorio fue movido a `integrations/langchain`, el test hace `continue` silenciosamente (`if !full_path.exists() { continue; }`) sin validar ninguna integración Python real.
- **Remediación:** Actualizar la lista de wrappers en `tests/version_coherence.rs` para apuntar a `integrations/{langchain,llamaindex,crewai,autogen,openai,ollama,mem0,letta}/pyproject.toml`.

### `FIND-370` (PKG-07/08/09) — Metadatos `license`, `[lints]` y features `tls`/`opentelemetry` incompletos en manifiestos Cargo
- **Ubicación:** `Cargo.toml:168-174,188,802` · `vantadb-server/Cargo.toml` · `vantadb-mcp/Cargo.toml` · `vanta-proxy/Cargo.toml` · `fuzz/Cargo.toml` · `vantadb-node/Cargo.toml` · `providers/*/Cargo.toml`
- **FMEA:** (1) En el `Cargo.toml` raíz, las features `tls` y `opentelemetry` no activan `"server"`, por lo que `cargo check --no-default-features --features tls` omite el módulo `server` (`#[cfg(feature = "server")]`). (2) `vantadb-server`, `vantadb-mcp`, `vanta-proxy` y `fuzz` carecen del campo `license`. (3) Los crates standalone (`vantadb-node`, `providers/*`, `feature-freez/desktop/src-tauri`) carecen de `[lints]` (`unwrap_used = "deny"`, `expect_used = "deny"`).
- **Remediación:** Añadir `"server"` a las dependencias de las features `tls` y `opentelemetry`, declarar `license = "Apache-2.0"` en los crates del workspace y añadir `[lints.clippy]` en los crates standalone.

### `FIND-371` (DOC-M02) — Secciones en español dentro de documentación técnica en inglés (`VANTA_MEMORY.md` y `CONSTRAINTS.md`)
- **Ubicación:** `docs/api/VANTA_MEMORY.md:5,11-19` · `docs/specs/CONSTRAINTS.md:11-14,21-23,101-103`
- **FMEA:** Viola la Regla 3 de `.opencode/AGENTS.md` (*Doc Language Split*: documentación técnica y de usuario en inglés).
- **Remediación:** Traducir al inglés los bloques en español de `docs/api/VANTA_MEMORY.md` y `docs/specs/CONSTRAINTS.md`.

### `FIND-372` (DOC-M03) — Nombres de workflows CI obsoletos en `CONTRIBUTING.md` y `QUICKSTART.md`
- **Ubicación:** `CONTRIBUTING.md:210,221-223` · `docs/user/QUICKSTART.md:146`
- **FMEA:** `CONTRIBUTING.md` referencia `fuzz-40.yml` y `heavy-certification-50.yml` (renombrados a `nightly.yml` y `certification.yml` / `heavy-certification.yml`), y `QUICKSTART.md` referencia `release-wheels-60.yml` (renombrado a `release-wheels.yml`).
- **Remediación:** Actualizar las referencias a los nombres vigentes en `.github/workflows/`.

---

## 5. Hallazgos de Severidad Baja (`LOW` — 6 Hallazgos)

### `FIND-373` (HW-BANNER) — Duplicación de sufijo `GBGB` y desalineación de borde en banner de hardware (`src/hardware/mod.rs`)
- **Ubicación:** `src/hardware/mod.rs:313,334-338`
- **FMEA:** El formato `RAM {}GB (cache {}GB)` recibe argumentos ya formateados con `"GB"` (`format!("{}GB", ram_gb)`), imprimiendo `RAM 31GBGB (cache 7GBGB)`. Además, `hw_row_plain` omite `[ ` y ` ]` alrededor de `instr_label`, desalineando el borde derecho de la caja ASCII.
- **Remediación:** Pasar `ram_gb` y `recommended_gb` sin sufijo duplicado e incluir `[ {} ]` en `hw_row_plain`.

### `FIND-374` (VDB-D1-07) — `FlatIndex::add` no deduplica nodos con el mismo `id`
- **Ubicación:** `src/index/flat.rs:170`
- **FMEA:** `FlatIndex::add` hace `self.nodes.push(...)` incondicionalmente sin reemplazar entradas previas con el mismo `id`, pudiendo retornar IDs duplicados en `search` tras actualizaciones.
- **Remediación:** Si ya existe una entrada con el mismo `node.id` en `self.nodes`, actualizarla in-place en lugar de añadir un duplicado.

### `FIND-375` (D2-FIND-07) — `unreachable!()` alcanzable con input externo en `LayerTelemetry::emit`
- **Ubicación:** `vanta-memory/src/services/pipeline_worker.rs:120`
- **FMEA:** El método público `LayerTelemetry::emit(&self, session_id: &str, kind: &str)` ejecuta `other => unreachable!("unknown layer telemetry kind: {other}")` ante cualquier string fuera de `"l1"|"l2"|"l3"|"dream"`, provocando pánico en lugar de degradación segura.
- **Remediación:** Reemplazar `unreachable!()` por `tracing::warn!` y retorno temprano.

### `FIND-376` (D2-FIND-08) — Feature `reqwest/blocking` innecesaria en `vanta-proxy/Cargo.toml`
- **Ubicación:** `vanta-proxy/Cargo.toml:12` · `vanta-proxy/src/langfuse.rs`
- **FMEA:** `vanta-proxy/Cargo.toml` habilita `features = ["json", "stream", "blocking"]` en `reqwest`. `langfuse.rs` usa `reqwest::blocking::Client` dentro de `std::thread::spawn` por cada evento (`flush_event`), creando un runtime Tokio efímero + pool HTTP nuevo en cada llamada.
- **Remediación:** Reemplazar por `tokio::spawn` con `reqwest::Client` asíncrono compartido o documentar el aislamiento del thread.

### `FIND-377` (PKG-10/11/12) — `html_root_url` en `0.3.0`, exclusión estrecha `feature-freez/desktop/` y comentario `SKIP` obsoleto en `justfile`
- **Ubicación:** `src/lib.rs:1` · `Cargo.toml:25` · `justfile:158-160`
- **FMEA:** (1) `src/lib.rs:1` declara `html_root_url = "https://docs.rs/vantadb/0.3.0/vantadb/"` (versión actual `0.8.0`). (2) `Cargo.toml:25` excluye `"feature-freez/desktop/"` en lugar de `"feature-freez/"`. (3) `justfile:158-160` (`surface-check`) imprime un mensaje `SKIP` obsoleto en lugar de invocar `node scripts/docs/check-npm-versions.mjs`.
- **Remediación:** Actualizar `html_root_url` a `0.8.0`, cambiar exclusión a `"feature-freez/"` y ejecutar `node scripts/docs/check-npm-versions.mjs` en `justfile`.

### `FIND-378` (DOC-L01) — Sección CLI de `README.md` omite comandos de verificabilidad (`verify` y `certificate verify`)
- **Ubicación:** `README.md:323-329`
- **FMEA:** La sección CLI del `README.md` principal no muestra los subcomandos `vanta-cli verify` ni `vanta-cli certificate verify`, que son pilares diferenciales de la propuesta de memoria verificable (VER-01/VER-02).
- **Remediación:** Añadir `vanta-cli verify` y `vanta-cli certificate verify` al bloque de ejemplos CLI en `README.md`.
