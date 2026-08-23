# Review profunda — `vantadb-node` (addon napi-rs nativo para Node)

**Fecha:** 2026-08-22 · **Alcance:** lectura completa de `src/lib.rs` (489 líneas), `index.d.ts`, `index.js`/`index.cjs` (glue napi generado), `package.json`, `Cargo.toml`, y `tests/persistence.test.ts`. Verificación cruzada contra la superficie del core y los otros dos transports.

---

## 1. Resumen

Addon napi-rs que expone `VantaEmbedded` a Node con persistencia real de filesystem (fjall/WAL/fsync) — su razón de existir frente al WASM. Ejecución correcta en lo poco que hace: todo async vía `spawn_blocking`, OpGate de durabilidad, parsing manual defensivo en la frontera FFI. El problema es el alcance: **expone ~9 métodos de memoria; no hay grafo, ni IQL, ni búsqueda explicada, ni export/import, ni métricas** — es con diferencia el transporte más limitado, y su estrategia de distribución multiplataforma está incompleta.

## 2. Arquitectura

```
JS (native.ts / directo) → VantaDb (napi class) → tokio spawn_blocking → VantaEmbedded (core)
I/O: serde_json::Value in/out (parseo manual entrada, Serialize salida)
OpGate: Mutex+Condvar durability barrier (idéntico a wasm/python)
Errores: VantaError → napi::Error::from_reason(Display) — strings planos
```

Decisiones correctas: el `MutexGuard` nunca cruza un `.await` (documentado en `drain()`); `":memory:"`/vacío mapea explícitamente a `BackendKind::InMemory`; validación de tipos campo por campo (`get_str/get_opt_u64/get_f32_vec`) con mensajes accionables.

## 3. Fortalezas

- **Nunca bloquea el event loop**: cada op corre en threadpool blocking con handle clonado — patrón correcto para napi.
- **OpGate verbatim-consistente** con wasm/python: misma barrera de durabilidad, mismos comentarios; la carrera fire-and-forget put→close está cerrada y testeada conceptualmente.
- **Parsing defensivo real** (trust boundary FFI): rechaza tipos incorrectos con mensajes por campo; valida finitud y entereza de números antes de castear a f32/u64.
- Test de persistencia diferencial bien pensado: close→reconnect→dato sobrevive (la ventaja clave vs WASM, afirmada y probada).
- `capabilities()` estable con labels legibles (Enterprise/Performance/LowResource).

## 4. Hallazgos

### Crítico / Required

1. **Superficie API ~5× más chica que WASM.** Expuesto: connect/close/flush/capabilities/put/put_batch/get/delete/list/list_namespaces/search. **Ausente por completo:** grafo (insert_node, get_node, add_edge, remove_edge, todos los traversals), IQL (`query`), explain_search, search_vector, generate_snippet, purge_expired, delete_by_filter, export/import/bulk_import, operational_metrics, compact_wal/compact_layout, rebuild/reindex, audits de text index, count/namespace_stats/similar_to_key, versionado/supersede, threads, snapshots. Para un backend cuya promesa es "todo lo que hace WASM + persistencia real", hoy es un CRUD de records. Ver §API coverage.

2. **Distribución multiplataforma rota/incompleta.** `package.json.files` incluye `"*.node"` y hay **un binario commiteado en el repo** (`vantadb_native.win32-x64-msvc.node`). No hay `optionalDependencies` por plataforma (patrón estándar napi-rs: `@scope/pkg-win32-x64`, etc.) pese a declarar 5 targets. Resultado: quien instala desde npm recibe solo el binario que hubiera en el tarball (win-x64) y en Linux/macOS fallará en runtime con "is the native binding built?". Requiere el esquema de paquetes por-plataforma o prebuilds descargados.

3. **Semántica de score contradictoria con TS/WASM.** `search` devuelve `{record, score}` y el test documenta/asserta que score es **similitud** (mayor = más cercano, orden descendente). El SDK TS encima mapea ese mismo campo a `.distance` documentado como distancia (menor = más cercano). Al menos uno de los tres transportes documenta al revés lo que emite el core. Decidir en core y propagar (bloquea #3 del reporte TS).

### Required

4. **`index.d.ts` todo `any`:** `put(record: any): Promise<any>` etc. Sin tipos de MemoryRecord/SearchRequest — el consumidor directo del addon pierde toda seguridad de tipos (mitigado parcialmente por `NativeVantaDB`, que además está incompleto). Los shapes están en el propio lib.rs; generarlos o escribirlos a mano es barato.
5. **Límites divergentes del resto:** MAX_VEC_DIM=10_000 (vs 10_000_000 en WASM) y top_k cap=10_000 (vs 1_000 en WASM). Misma operación, distinto contrato según transporte — mover las constantes al core y reutilizarlas.
6. **`distance_metric` acepta `"Euclidean"` y `"euclidean"`, WASM solo `"Euclidean"`** exacto (cualquier otra cosa → Cosine silencioso). Menor, pero otra divergencia gratuita.
7. **Tests insuficientes para la frontera FFI:** 3 tests happy-path. Nada de: filtros en list, paginación cursor, ttl_ms, metadata inválida, read_only mode, memory_limit, error paths del OpGate ("database is closing"), ni concurrencia put/close.

### Nit / Optional

8. `README.md` listado en `files` pero **no existe** en el directorio — npm publish advertirá/fallará la inclusión.
9. Binario `.node` commiteado en git (además del problema #2): artefacto de build en el repo, bloat y riesgo de desincronización con src.
10. `memory_limit: 0` se silencia a `None` (`(value > 0).then_some(value)`) — aceptar y ignorar un valor sin avisar es una trampa menor.
11. `capabilities()` es sync y no pasa por OpGate — inconsistente con el resto (inofensivo: es lectura pura).
12. `list` solo soporta el path legacy `filters` (`filter_ops: None` hardcoded) — igual que WASM, pero aquí ni siquiera está tipada la opción avanzada.
13. Clase `VantaDb` con alias `VantaDB` — elegir uno (el alias ya existe, ok, pero index.d.ts documenta `VantaDb`).

## 5. API coverage vs core (verificado leyendo `vantadb/src/sdk/**`)

**Expuesto (11):** connect, close, flush, capabilities · put, put_batch, get, delete, list, list_namespaces, search.

**Faltante vs core (~45 ops relevantes):**
- Grafo completo: insert_node, get_node, delete_node, add_edge, **remove_edge**, graph_bfs/dfs/topological_sort/is_dag/bfs_filtered/dfs_filtered, degree_centrality, page_rank, acumuladores.
- IQL: `query`.
- Búsqueda: search_vector, explain_memory_search, search_multi/all/with_method, similar_to_key, generate_snippet.
- Memoria avanzada: delete_by_filter, count, namespace_stats, purge_expired, supersede, get_version, versions.
- Mantenimiento: flush ✅ (único), compact_wal, compact_layout, rebuild_index, reindex_hnsw_from_text, repair_text_index, audit_text_index(+deep), vacuum, snapshots.
- Portabilidad: export_all/export_namespace(+filtered), import_records/file, bulk_import_file/stream.
- Métricas: operational_metrics. Hilos/wiki: create_thread…purge_expired_threads, recover_archived_nodes, graphrag_search.
- Parámetros perdidos: filter_ops, exclude_superseded, sparse_vector, search_profile.

## 6. Incompletudes

- Sin README (referenciado pero ausente), sin changelog.
- Sin CI visible para los 5 targets declarados (los binarios no existen en el repo salvo win-x64).
- `NativeVantaDB` en vantadb-ts replica este subset y hereda sus límites; ampliar node amplía TS gratis.
- No hay tests de integración contra `NativeVantaDB` dentro de este paquete (los de TS cubren otro camino).

## 7. Propuestas (ordenadas)

1. Cerrar la brecha #1 por olas: (a) query/IQL + search_vector + explain (paridad de lectura), (b) grafo completo con IDs string-u128 como wasm/python, (c) mantenimiento/portabilidad. Cada ola con test de roundtrip.
2. Implementar distribución napi-rs estándar: optionalDependencies por plataforma + CI que publique los 5 targets; sacar el binario del repo.
3. Unificar score/distance y límites (vector dim, top_k) con los otros transports — decisión única en core.
4. Tipar `index.d.ts` (MemoryRecord, SearchRequest, ListOptions) — copiar de `vantadb-ts/src/types.ts` y ajustar.
5. Ampliar tests: error paths, OpGate, paginación, filtros.

## 8. Consistencia con otros SDKs

| Aspecto | node | WASM | TS wrapper | Python |
|---|---|---|---|---|
| Métodos | camelCase (napi) | snake_case crudo | camelCase | snake_case |
| Cobertura | ~11 ops | ~45 ops | ~35 ops (sobre wasm) | paridad amplia |
| IDs grafo | n/a | strings u128 | bigint edges | strings u128 |
| Errores | string plano | string plano | codes propios | PyErr |
| Async | siempre Promise | sync (+async persist) | sync wasm / async native | sync |
| Límites | vec≤10k, top_k≤10k | vec≤10M, top_k≤1k | hereda | — |

Duplicación de lógica cliente: el retry existe solo en wasm-worker; el cursor/parsing viven en core (✅ no reimplementado); OpGate triplicado verbatim entre bindings (wasm/node/python) — extraer a crate compartida cuando toque el cuarto duplicado. La divergencia principal de consistencia es de **cobertura**, no de nombres: los nombres que existen son isomorfos como promete el docstring.

## 9. Score

**4.5 / 10** — Lo implementado está bien hecho (async correcto, durabilidad, parsing defensivo), pero el módulo promete ser "el backend nativo completo" y entrega un CRUD mínimo, con distribución multiplataforma no funcional fuera de Windows y tipado ausente. Alto potencial, ejecución incompleta.

---
*Anterior:* [`vantadb-wasm.md`](./vantadb-wasm.md) · [`vantadb-ts.md`](./vantadb-ts.md)

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| #1 — Superficie API mínima (~11 ops expuestas vs ~45 relevantes del core) | **MOD-29** |
| #2 — Distribución multiplataforma rota (binario único win-x64 commiteado, sin optionalDependencies por plataforma) | **MOD-30** |
| #3–#7, #8–#13 — nits (`index.d.ts` todo `any`, límites divergentes, tests insuficientes FFI, README ausente, `.node` en git) | **MOD-31** |
