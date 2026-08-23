# Review profunda — `vantadb-wasm` (binding wasm-bindgen, OPFS/IndexedDB)

**Fecha:** 2026-08-22 · **Alcance:** lectura completa de `src/lib.rs` (1937 líneas), `src/opfs.rs`, `src/idb.rs`, `src/worker.rs`, `src/opfs_bridge.js`, `Cargo.toml`, glue `pkg/` (.d.ts), tests (`tests/wasm_tests.rs`, inline, `e2e/e2e-persistence.mjs`). Contexto: bug conocido **CORE-02** (consola IQL vía WASM lee graph-store vacío).

---

## 1. Resumen

Binding wasm-bindgen sobre `VantaEmbedded` con tres capas de persistencia browser (OPFS directo, IndexedDB fallback explícito, Web Worker opcional) y un cache de persistencia diferencial (PERF-08). Es el transporte JS más completo y el más maduro en testing. Sus debilidades estructurales: **la persistencia solo round-tripea memory records (el grafo se pierde al recargar)**, un **fallback silencioso a in-memory** cuando OPFS falla, y errores como strings planos sin códigos.

## 2. Arquitectura

```
lib.rs   — VantaDB #[wasm_bindgen]: CRUD memoria + búsqueda + grafo + IQL + mantenimiento
           OpGate (durability barrier close/in-flight) · PersistCache (differential persist PERF-08)
opfs.rs  — OpfsStorage/OpfsFile: KV sobre archivos OPFS, write tmp+rename, footer CRC-32
idb.rs   — IdbStorage: bridge IndexedDB inline-JS (wasm_bindgen inline_js), BroadcastChannel + Web Locks
worker.rs— OpfsWorkerProxy: MessageChannel req/resp con timeout 5s y retry ×2 backoff
opfs_bridge.js — spawnOpfsWorker (blob worker) + helpers JS duplicados de opfs.rs
```

Coerciones de tipos: u64/u128 → String en records e IDs de nodo (política correcta y consistente con MCP/Python); vectors → Float32Array zero-copy con sanitización NaN/Inf→0.0 (PERF-08/P2-7); `next_cursor` → f64.

## 3. Fortalezas

- **Persistencia diferencial (PERF-08)** bien diseñada: dirty/deleted/cache_invalid, skip-write cuando no hay cambios, recuperación a rebuild completo si la escritura falla después de drenar dirty.
- **OpGate**: barrera de durabilidad idéntica a node/python — cierra la carrera write-after-close. Correcta y documentada.
- **Guardas de límite en la frontera FFI**: MAX_F32_VEC_LEN, MAX_BATCH_SIZE, MAX_K (ERR-022) antes de tocar el engine.
- **Testing excepcional para WASM**: ~40 wasm_bindgen_tests + suite OPFS/IDB real (`wasm_tests.rs`, 1146 líneas) + e2e con reload real de browser y verificación del fallback no-modules.
- Detalles finos correctos: NotFoundError→Ok(None) en OPFS read; CRC-32 footer; dedup por node_id u128 (AUD-043); roundtrip u128 >2^64 testeado (ERR-024).
- `-Oz` en release para tamaño binario; features gateadas (`opfs`, `tracing-wasm`).

## 4. Hallazgos

### Crítico / Required

1. **El grafo NO se persiste — hipótesis fuerte para CORE-02.**
   `save()`/`save_idb()` serializan **exclusivamente `Vec<VantaMemoryRecord>`** a `db_state.json` (`persist_payload`, `lib.rs:661-720`). `insert_node`, `add_edge`, `delete_node` ni tocan el `PersistCache`. Al recargar, `load()` reconstruye solo records vía `import_records`: **nodos y edges desaparecen**. En modo standalone, cualquier flujo que escriba edges y relea tras persist/reload ve un graph-store vacío — exactamente el síntoma de CORE-02 ("IQL lee graph-store vacío aunque haya edges insertados"). Aunque CORE-02 describa el fallo in-session, esta brecha garantiza el fallo cross-session y debe entrar en el análisis root-cause junto al init del backend (`build_config` fuerza `BackendKind::InMemory`, `lib.rs:72`). Fix mínimo: serializar también nodos/edges (o delegar persistencia en snapshots del core).

2. **Fallback silencioso a in-memory si OPFS falla al abrir** (`connect_persistent`, `lib.rs:425`):
   ```rust
   let opfs = OpfsStorage::open(path).await.ok();
   ```
   Si `getDirectory` falla (permiso denegado, Safari privado), la DB abre igual, `save()` se vuelve no-op silencioso (`opfs: None ⇒ Ok(())`) y el usuario cree que persiste. No hay fallback automático a IndexedDB ni error: hay que *saber* llamar `connect_idb` aparte. Debería: propagar error o caer a IDB con warning explícito vía `capabilities().persistence`.

3. **Errores sin estructura:** `to_js_err` aplana todo `VantaError` a string de `js_sys::Error` (`lib.rs:1518`); validaciones locales usan `JsValue::from_str` crudo (p.ej. dirección inválida). El consumidor no puede distinguir `InvalidInput`/`NotFound`/`Closing` sin parsear texto. El SDK TS encima intenta disimularlo con códigos propios (`WASM_ERROR`). Necesita un shape `{code, message}` consistente.

4. **`OpfsFile::append` sobreescribe en lugar de agregar** (`opfs.rs:85-98`): usa `createWritable({keepExistingData:true})` y escribe sin `position` → el write arranca en offset 0 y pisa el comienzo del archivo. La versión JS del bridge sí calcula posición (`opfs_bridge.js:53-57`) — las dos implementaciones divergen. No afecta el flujo principal (append no se usa en save/load), pero es API pública rota.

### Required

5. **`flush()` engañoso en WASM:** el docstring dice "Flush all pending writes to disk", pero el backend es InMemory y la durabilidad real depende de que el usuario llame `save()`/`save_idb()` manualmente. `flush()` da falsa sensación de persistencia. Renombrar/documentar o auto-delegar a save.
6. **metadata descartada silenciosamente:** en `memory_record_to_js`, `if let Ok(meta) = serde_wasm_bindgen::to_value(&rec.metadata)` ignora el error — un record podría devolverse sin metadata sin señal alguna (`lib.rs:1582`).
7. **Detección de corrupción débil:** `read_file` devuelve los datos crudos si el CRC no matchea ("legacy fallback") — un archivo corrupto pasa directo a `serde_json` y produce un error de parseo confuso en vez de "storage corrupto". Mejor: error explícito con opt-out legacy flagueado.
8. **Cuotas sin manejar:** ni OPFS ni IDB consultan `navigator.storage.estimate()`, piden `navigator.storage.persist()` (evitar eviction de IDB best-effort), ni traducen `QuotaExceededError` a un error accionable. Datasets grandes mueren con un DOMException crudo desde `write_file`.

### Nit / Optional

9. `next_cursor` viaja como f64 (`lib.rs:943`, `(cursor as f64)`) — inconsistente con la política string-u64 del resto; pierde precisión >2^53 (teórico pero rompe la propia regla del proyecto).
10. Retry del worker proxy matchea strings ("timeout"/"abort"/"try again") — frágil; y reintentar `Write` no-idempotente puede duplicar efectos (aquí writes son replace, riesgo bajo).
11. MessagePorts del MessageChannel nunca se `.close()`an tras cada request — leak menor por request.
12. Sanitización NaN→0.0 en vectors/scores altera datos silenciosamente (documentado, pero convendría un flag o contador de sanitizaciones).
13. `connect_worker` exige inyectar `globalThis.spawnOpfsWorker` a mano — DX incómodo (documentado, pero sería trivial exponer el import desde el glue pkg).
14. Duplicación `opfs.rs` ↔ `opfs_bridge.js`: mismas operaciones implementadas dos veces (Rust puro y JS exportable). Una sola fuente.
15. Typo en doc: "persiated" (`lib.rs:261`).

## 5. API coverage vs core (verificado)

**Expuesto (~45 ops):** put, put_batch, get, delete, delete_by_filter, list, list_namespaces, search, search_vector, explain_memory_search, generate_snippet, purge_expired · insert_node, get_node, delete_node, add_edge, graph_bfs/dfs/topological_sort/is_dag/filtered_traversal/degree · query (IQL), flush, compact_wal, compact_layout, rebuild_index, reindex_hnsw_from_text, repair_text_index, audit_text_index(+deep), operational_metrics, capabilities, close · export_all/export_namespace(+filtered), import_records/import_file, bulk_import(+bytes) · save/save_idb/load/load_idb/delete_idb, connect_persistent/connect_idb/connect_worker, worker_read/write/delete.

**Faltante vs core:** `remove_edge`, `graph_dfs_filtered`, `count`, `namespace_stats`, `similar_to_key`, `search_multi/search_all/search_with_method`, `get_version/versions/supersede`, threads (6 métodos), `recover_archived_nodes`, `graphrag_search`, snapshots, `vacuum/pipeline/optimizer_config`, debug ops. Parámetros perdidos: `filter_ops`, `exclude_superseded`, `sparse_vector`, `search_profile`.

## 6. Incompletudes

- Persistencia parcial (solo memory records) — ver hallazgo #1; sin snapshot binario del estado completo.
- Glue `pkg/*.d.ts`: casi todo `any` (limitación wasm-bindgen) — mitigado por vantadb-ts, pero el paquete standalone queda sin tipos útiles.
- Sin auto-save ni hook de `beforeunload`/`visibilitychange`; si la pestaña muere sin `save()`, se pierde todo lo escrito desde el último save. Documentado implícitamente, no resuelto.

## 7. Propuestas

1. Resolver persistencia de grafo (bloqueante para CORE-02 y para confiar en OPFS): extender `db_state.json` a `{records, nodes, edges}` o usar snapshots nativos del core.
2. `connect_persistent`: propagar fallo de OPFS (o fallback a IDB logueado); exponer `capabilities().persistence` honesto post-connect.
3. Shape de error `{code, message}` único (mapear `VantaError` discriminado) — habilita que TS/node/Python compartan taxonomía.
4. Arreglar `OpfsFile::append` (usar posición como el bridge JS) o eliminarlo hasta que se necesite.
5. Manejo de cuota: chequear `estimate()`, intentar `navigator.storage.persist()`, y mapear QuotaExceeded a error descriptivo.

## 8. Consistencia con otros SDKs

- **Nombres:** snake_case crudo vs camelCase del wrapper TS — coherente por diseño (thin wrapper), pero duplica superficie documental.
- **IDs:** strings u128 aquí y en Python/MCP (ERR-023/ERR-025) — ✅ consistente; node no tiene grafo con qué comparar.
- **Score/distance:** emite campo `score` que TS documenta como distance — ver reporte TS #3; el test de node lo describe como similitud. Divergencia activa entre transports.
- **Límites:** MAX_F32_VEC_LEN=10_000_000 vs node MAX_VEC_DIM=10_000 — misma operación acepta 1000× menos dimensiones según transporte. MAX_K=1_000 vs node top_k≤10_000. Unificar constantes (moverlas al core).
- **Retry:** existe solo aquí (worker proxy); node/TS no reintentan nada. Aceptable (FFI sync local no necesita retry), pero documentarlo como decisión.
- **Duplicación de lógica cliente:** parsing/cursor viven en core ✅; OpGate está triplicado verbatim (wasm/node/python) — candidato a extraerse a crate compartida `vantadb-gate` o similar.

## 9. Score

**7 / 10** — El binding más completo y mejor probado, con ingeniería seria (PERF-08, OpGate, e2e real). Penaliza: persistencia de grafo ausente (#1, núcleo de CORE-02), fallback silencioso (#2), errores sin códigos (#3) y append roto (#4).

---
*Anterior:* [`vantadb-ts.md`](./vantadb-ts.md) · *Siguiente:* [`vantadb-node.md`](./vantadb-node.md)

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| #2 — Fallback silencioso a in-memory si OPFS falla al abrir | **MOD-25** |
| #4 — `OpfsFile::append` escribe en offset 0 y sobreescribe el archivo | **MOD-26** |
| #8 — Cuotas de storage sin manejar (sin `estimate()`, `persist()`, ni mapeo de `QuotaExceededError`) | **MOD-27** |
| #5–#7, #9–#15 — nits (`flush()` engañoso, metadata descartada, CRC débil, cursor f64, retry frágil, duplicación opfs↔bridge) | **MOD-28** |

Los hallazgos ya trackeados previamente que este reporte menciona (**CORE-02** — hallazgo #1: grafo no persistido, hipótesis fuerte para ese bug) → referenciados en su fila existente en `docs/Backlog.md`, no duplicados aquí.
