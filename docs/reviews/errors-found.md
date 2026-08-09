# Errores Encontrados — Revisión Multi-Agente

> **Fecha:** 2026-08-08
> **Rama:** develop (7a19a9f5)
> **Método:** Revisión por partes/funciones/features con sub-agentes en paralelo (vanta-audit, vanta-arch, vanta-engine, vanta-worker, vanta-docs, vanta-tuner)
> **Estado:** ⚠️ EN CURSO — los errores se documentan incrementalmente según se descubren

## Formato de cada hallazgo

| Campo | Descripción |
|---|---|
| ID | Número correlativo (ERR-001, ERR-002, ...) |
| Severidad | 🔴 Crítico / 🟠 Alto / 🟡 Medio / 🔵 Bajo / ⚪ Info |
| Área | Módulo o feature afectada |
| Tipo | Bug, Panic, Unsafety, Race, Memory, API break, Doc drift, Perf, Seguridad, Lint, Test |
| Ubicación | `archivo:línea` |
| Descripción | Qué falla y por qué |
| Suggestion | Fix propuesto |
| Reportado por | Sub-agente que lo encontró |

---

## Hallazgos

## ERRORES ENCONTRADOS (documentados incrementalmente)

### Auditoría de seguridad (vanta-audit) — 8 hallazgos

| ID | Sev | Tipo | Ubicación | Descripción | Suggestion |
|----|-----|------|-----------|-------------|------------|
| ERR-001 | 🟠 | Unsafety/UB | `src/storage/engine/ops.rs:518-521` (+1242-1266, 1429-1451, 1831-1851), `src/index/search.rs:541` | `view_start + vector_len*4` en `usize` con wrap en targets 32-bit (wasm32 es miembro del workspace). El guard `view_start > vstore.size -> None` puede pasar con `view_end` truncado y `slice::from_raw_parts` crea slice OOB -> UB | `checked_mul`/`checked_add`; validar `vector_len` contra capacidad real del parametro |
| ERR-002 | 🟠 | Panic/Hang | `src/storage/vfile.rs:211-223` | Handler SIGBUS devuelve sin resolver el fault -> se re-ejecuta la instruccion faultante -> **ciclo infinito SIGBUS** (hang) en vez de error. Flags `SIGBUS_OCCURRED`/`SIGBUS_FAULT_ADDR` nunca consumidas | `siglongjmp` a recovery point o `abort()`; leer flag tras cada acceso a mmap |
| ERR-003 | 🟠 | Panic | `src/storage/engine/ops.rs:507, 1311, 1397, 1820` | Indexacion directa `vector_store[seg_id as usize]` con `seg_id` extraido del byte alto de `storage_offset` persistido; header corrupto -> **panic** (mata el proceso CLI/server). `apply_delete` ya usa `.get()` | Replicar patron `.get()` en los 4 puntos |
| ERR-004 | 🟠 | Supply chain | `deny.toml` / Cargo.lock | `lru 0.12.5` (RUSTSEC-2026-0002, `IterMut` viola Stacked Borrows) via `ratatui 0.28.1` (feature `tui` opcional). cargo audit = warning; deny pasa porque default no incluye ratatui | Bump ratatui a version con `lru >= 0.14` o override directa |
| ERR-005 | 🟡 | Test regression | `src/storage/ops.rs` | Test AUDREP-45 (rechazo de length prefix oversized en postcard) eliminado en diff — regresion de cobertura para guard de input | Restaurar test |
| ERR-006 | ⚪ | Supply chain | `deny.toml` | Ignore de RUSTSEC-2024-0436 reporta "advisory-not-detected" — entrada stale; si el crate reaparece el ignore sigue vivo sin issue tracker | Limpiar ignore o crear issue de tracking |
| ERR-007 | ⚪ | Perf/Bloat | Cargo.lock | `multiple-versions = warn`: hashbrown x3, rand 0.9/0.10, syn x2, thiserror x2, windows-sys x2 | Consolidar versiones |
| ERR-008 | ⚪ | Unsafety | `src/storage/vfile.rs` (`copy_unsafe`) | `ptr.add` fuera de bounds solo chequeado en debug asserts | Guard explicito (defense-in-depth) |
| ERR-009 | ⚪ | Verification | CI | Recomendacion: correr `MIRIFLAGS=-Zmiri-tree-borrows cargo miri test` sobre vfile.rs/ops.rs antes de merge | CI gate |

### Arquitectura / concurrencia / persistencia (vanta-arch) — 6 hallazgos

| ID | Sev | Tipo | Ubicacion | Descripcion | Suggestion |
|----|-----|------|-----------|-------------|------------|
| ERR-010 | 🔴 | Persistencia / Race | `src/storage/engine/maintenance.rs:56-86` | **Raza checkpoint-snapshot:** `checkpoint_seq` se escribe en el backend ANTES de serializar el indice (`save_vector_index()`), sin `insert_lock` en ninguna de las dos. Insert concurrente -> en reopen el record se reaplica (duplicacion); o si entra entre `flush_pending_hnsw()` y checkpoint -> nodo comprometido invisible para siempre | Lock unico sobre todo el bloque checkpoint->serialize, o fijar `checkpoint_seq` DESPUES de `save_vector_index` con la seq del snapshot; failpoint + test de interleave |
| ERR-011 | 🟠 | Persistencia | `src/wal_sharded.rs` + `src/storage/engine/init.rs:454-480` | `recover_state` asume round-robin exacto + `local_pos` como seq global: shard truncado por crash -> record en `local_pos` bajo se marca como ya-checkpointed sin serlo -> **perdida silenciosa en recuperacion** | No confiar en `local_pos`; usar seq global del header del record; detectar truncation por falta de EndRecord y parar replay |
| ERR-012 | 🟠 | Consistencia | `src/index/neighbor_index.rs` + `src/index/graph.rs` `shrink_neighbors` | `apply_delete`/`flush_pending_hnsw` remueven de `hnsw.nodes` sin decrementar contadores `inbound` ni limpiar neighbor lists de supervivientes; `shrink_neighbors` decide eviccions por `inbound_count <= last` con conteo stale -> evicta/preserva candidatos equivocados y filtra union progresivamente | Decrementar inbounds de vecinos en path de delete o marcar tombstones y purgar en repair; test con 100 deletes |
| ERR-013 | 🟠 | Atomicidad | `src/storage/engine/ops.rs` (`insert` paths) | Stats/meta (`cardinality`, `edge`, `scalar`) se actualizan ANTES del `txn.buffer(Abort)` -> txn abortada deja inventario inflado y edges a records inexistentes | Mover stats al commit o re-aplicarias en path de abort |
| ERR-014 | 🟡 | TOCTOU / staleness | `src/storage/ops.rs` (`get()`) | Insert escribe vstore+backend+WAL antes de volcar `pending_hnsw`; `get()` concurrente responde `Ok(None)` para nodo ya committed — staleness en patron insert->get inmediato | Documentar en ADR (eventual consistency) o fallback de get a vstore |
| ERR-015 | 🔵 | Shutdown | `desktop/src-tauri/src/connections/child_process.rs:170-189` | `request_shutdown` documenta "send stop signal + wait grace" pero SIEMPRE `kill()` (SIGKILL/TerminateProcess); el grace shutdown via `ctrl_c` del sidecar MCP nunca se ejercita; en Windows mata sin flush de metadata (WAL cubre, pero pierde stats) | `cfg(unix)` SIGINT + esperar grace; windows `GenerateConsoleCtrlEvent` o documentar exploplicito |

### Motor vectorial / indices (vanta-engine) — 5 hallazgos

| ID | Sev | Tipo | Ubicacion | Descripcion | Suggestion |
|----|-----|------|-----------|-------------|------------|
| ERR-016 | 🔴 | Logic/Data loss | `src/parser/mod.rs:174-175` + `src/index/executor.rs:160` | Parser consume `WHERE`/`RANK` como alias -> filtro se **descarta silenciosamente** en queries | Corregir parseo de clausulas WHERE/RANK |
| ERR-017 | 🟠 | Correctness/Recall | `flat.rs:49-51` vs `src/index/search.rs:650-652` | Score euclidiano inconsistente: flat usa `-dist^2` vs HNSW usa `-dist` -> resultados y recall difieren entre modos | Unificar metrica |
| ERR-018 | 🟠 | Algorithm | `src/index/graph.rs:441-444` | `random_layer` capado en level 2 con `ml` default -> grafos sin profundidad (L<5) degradan recall en alta dimensionalidad | Teto superior por `ml` real |
| ERR-019 | 🟠 | Bench invalido | `benches/hnsw_pure.rs` | `flat_threshold=Some(10000)` con count=10000 -> bench mide brute-force, no HNSW | Corregir threshold del bench |
| ERR-020 | 🟠 | Bug | ACORN second-hop + `repair_orphan_links` | second-hop usa `take_inline_neighbors` stale despues de repair -> arcos muertos/omitidos | Re-sync de neighbor lists tras repair |

### Logica core y bindings (vanta-worker) — 14 hallazgos

| # | Sev | Area | Tipo | Ubicacion | Descripcion | Fix |
|---|-----|------|------|-----------|-------------|-----|
| ERR-021 | 🔴 | MCP | OOM regression | `vantadb-mcp/src/lib.rs:333-365`, `:1401`, `:1430`, `:1499` | `collection_stats/list/delete` materializan namespace completo via `collect_all_records`; streaming `collect_stats` (AUDREP-21) eliminado; test `mcp_tests.rs:725` ya no verifica boundness -> >100k vectores = OOM por llamada | Restaurar streaming con limite + paginacion `take(n)`; delete por ID/batch |
| ERR-022 | 🔴 | Core+bindings | Crash | `vantadb-mcp/src/lib.rs:1301`; python `lib.rs:858,1246`; wasm `:736-738`; alloc en `src/index/search.rs:522-601` | `top_k`/`k` sin tope: `Hashset::with_capacity(ef_search.max(top_k)*3)` con k=10^9 -> intento de alloc gigante -> abort del proceso | `k.min(MAX_K)` en bindings (reusar `max_top_k` del core en MCP) |
| ERR-023 | 🟠 | Python | API break/trunc | `vantadb-python/src/lib.rs:858-878`, `:1234` | Node IDs u64: `search_vector` devuelve `u64` truncado; `delete(id:u64)` no puede borrar IDs >= 2^64 (PyO3 OverflowError) | Pasar `u128` o exponer como `str` |
| ERR-024 | 🟠 | WASM | API break | `vantadb-wasm/src/lib.rs:1011,1039,1047` | `insert_node/get_node/delete_node` toman u64 mientras `u128` core; search devuelve id string que `get_node(u64)` no acepta -> nodos >2^64 inaccesibles | Aceptar `JsValue` (string/number) + `u128::from_str` |
| ERR-025 | 🟠 | MCP | API break | `vantadb-mcp/src/lib.rs:1330-1340` | `get_node_neighbors` lee id como `as_u64` + JSON numerico pierde precision >=2^53 -> nodos u128 grandes inaccesibles | Tomar id como string JSON + `u128::from_str` |
| ERR-026 | 🟡 | MCP | Silently wrong | `vantadb-mcp/src/lib.rs` (parse_metadata) | Filtros no-escalares (arrays, objetos, null) descartados en silencio -> filtro ignorado, resultados superconjunto erroneo | Conservar como JSON repr o error explicito |
| ERR-027 | 🟡 | CLI HTTP | API contract | `src/cli_server.rs:607-627` | `execute_query` devuelve HTTP 200 con `success:false` para errores IQL -> proxies/monitorizado no distinguen fallos | 400/422 para user errors, 500 internos |
| ERR-028 | 🟡 | Core | Silently empty | `src/index/search.rs:509-528` | Query con vector norma 0 devuelve `[]` con solo `tracing::warn` -> bindings muestran "sin resultados" falso | Propagar `VantaError::InvalidInput` |
| ERR-029 | 🟡 | Storage | Corrupcion | `src/storage/ops.rs:85` | `edge_count = edges.len() as u16` -> nodo con >65.535 aristas corrompe al persistir (wrap a 0) | Crecer campo a u32 o retornar error |
| ERR-030 | 🟡 | python | Cross-namespace | `vantadb-python/src/lib.rs:311-350` | `put_batch` path legacy permite namespace por entrada -> datos mezclados contra el path keyword | Forzar coherencia de namespace |
| ERR-031 | 🔵 | Core | Silent loss -> Result | `src/index/search.rs:661-698` (trait `VecIndex::add`) | Trait traiga rechazos con solo `warn!`, sin Result; hoy el engine usa metodos inherent con `?`, pero adapters `Arc<dyn>` perderían inserts | `fn add -> Result` o `#[doc(hidden)]` |
| ERR-032 | 🔵 | Storage | Coverage | `src/storage/ops.rs` | Test de `deserialize_node_payload` (guard `MAX_PERSISTED_NODE_BYTES`) removido sin mover; regresion futura de max payload | Reubicar test en tests/ |
| ERR-033 | 🔵 | MCP | Edge case | `vantadb-mcp/src/lib.rs:1139-1142` vs `src/sdk/api.rs:502` | `memory_list(limit=0)` -> `max(1)` -> devuelve 1 cuando pidio 0 | Normalizar en binding |
| ERR-034 | 🔵 | CLI | Verificado OK | `src/cli_server.rs:143-179` | `/metrics` protegido, `/health` publico, body limit 1MB | Sin hallazgo |

### Performance (vanta-tuner) — 15 hallazgos

| # | Sev | Ubic. | Patron | Explosion | Fix |
|---|-----|-------|--------|-----------|-----|
| ERR-035 | 🔴 | `src/physical_plan.rs:289-290` (+ `ops.rs:507`, `ops.rs:1235`) | `vector_store[0].read()` retenido durante TODO `search_nearest(..., Some(&vs))` — HNSW completo corre bajo read-lock del `RwLock<VantaFile>` | Contension global: `insert`/`batch_insert` necesitan `write()` (ops.rs:997) y quedan congelados por queries; inserts serializados tras any query | Snapshot `(ptr,len)` mmap sin sostener guard (ArcSwap); lock solo para cursor/header |
| ERR-036 | 🔴 | `src/storage/ops.rs:619-627` (`get()`) | `let mut cache = self.volatile_cache.write()` en read path solo para `hits += 1`/`last_accessed` | Todos los reads calientes toman WRITE lock -> lectores serializados; con `top_k`+`get_many` cada nodo paga write | Contadores atomicos side-table; read-lock + hits eventual |
| ERR-037 | 🟠 | `src/storage/ops.rs:1027+` (`batch_insert`) | Por cada nodo llama `self.get(id)` completo: write-lock cache + backend KV + hnsw.get + read vstore + `to_vec()` del vector | 10k batch = 10k read-paths completos en serie, clona vector que descarta | Snapshot de existencia por lote o `skip_existing_check` |
| ERR-038 | 🟠 | `src/index/search.rs:245-263` | `should_prefetch()` default **true** -> syscall de prefetch (madvise/PrefetchVirtualMemory) POR vecino por pop, duplica lookups `nodes.get`+`read_header` | ~100 pops x 32 vecinos = ~3.200 syscalls/query (Windows +1-3ms) | Prefetch en misma pasada del loop; default Disabled en NVMe |
| ERR-039 | 🟠 | `src/index/ivf.rs:279-286` | `VectorRepresentations::Full(entry.vector.clone())` por cada entrada de las nprobe listas | ~1.000 allocs `Vec<f32>`/query solo para pasar el vector a `calculate_similarity` | Overload `similarity_f32(query, other: &[f32])` — cero clones |
| ERR-040 | 🟠 | `src/index/ivf.rs:132-139` (k-means) | Clona el centroide completo por par (n x nlist x hasta 20 iter) | ~632M allocs de dim d durante build | Pasar `&centroid[..]` hacia el helper f32-slice; `new_centroids` flat Vec |
| ERR-041 | 🟠 | `src/storage/ops.rs:1035-1095` (batch_insert) | `node.clone()` + `relacional.clone()` + `key.to_vec()` + `WalRecord::Insert(n.clone())` (re-clona todo el batch) + `vector.clone()` en level_entries | En modo Incremental (batch<1000), vector 1536-d se clona ~4x -> ~24MB allocs/batch | Reusar `active_node` clonado para WAL (mover); `bitset`/`vector` mover en vez de clone |
| ERR-042 | 🟡 | `src/index/search.rs:275-280` + `:347-353` | `vs.read_header(neighbor.offset)` 2x por candidato (vector + eligibility), `vfile_reads += 2` contado en el propio profile | 2x lecturas mmap por candidato en hot loop | Leer header una vez + pasar flags (header es Copy) |
| ERR-043 | 🟡 | `src/index/graph.rs:920-926` (`shrink_neighbors`) | `vec_data.as_f32_slice().map(to_vec)` clona el vector completo del nodo solo para usarlo como query | d=1536 + M=32 -> ~6KB por evento overflow | Evitar to_vec cuando inline; mantener guard DashMap |
| ERR-044 | 🟡 | `src/tokenizer.rs:44-106` | `TextAnalyzer` (SimpleTokenizer+Stemmer+StopWordFilter) se reconstruye en CADA `tokenize_advanced` | Batch N docs paga N setups; el propio bench mide el setup por llamada | `OnceLock<TextAnalyzer>` por config + clonable |
| ERR-045 | 🟡 | `src/index/neighbor_index.rs:66-68` | `get_neighbors` -> `map(van v.clone())` clona lista completa cada vez | `traverse_graph` (archive.rs:166) clona por nodo del BFS -> O(NxM) allocs por compactacion | API borrow: `with_neighbors(f: impl FnOnce(&NeighborVec))` |
| ERR-046 | 🟡 | `src/index/graph.rs:606-609`, `:757-760` | `vec_data.to_f32()` clona el vector (`node.rs:523`) solo para usarlo como query de consucción | 1 alloc de d x4 por insert | `match vec_data { Full(v) => v... }` o `to_f32_borrowed()` |
| ERR-047 | 🔵 | `src/index/search.rs:225-238` + `:380-392` | `take_neighbors()` + `extend_from_slice(e)` copia lista inline en cada pop del hot loop | ~500B copy/pop (mitigado por thread-local pool E2) | Devolver `&NeighborVec` con guard DashMap en scope |
| ERR-048 | 🔵 | `src/index/search.rs:268-269` | `if !visited.contains(&id) { visited.insert(id) }` — 2 hash lookups | 2x coste de bookkeeping de visited por vecino | `if visited.insert(id)` (insert devuelve bool) |
| ERR-049 | ⚪ | Midiendo tencias | — | No hay bench dedicado a ivf.rs ni batch_insert | Añadir micro-benches para cuantificar ERR-037/39-41 |

### Documentacion / API sync (vanta-docs + verificacion del lead) — 2 hallazgos

| ID | Sev | Tipo | Ubicacion | Descripcion | Suggestion |
|----|-----|------|-----------|-------------|------------|
| ERR-050 | 🟡 | Changelog drift | `docs/CHANGELOG.md` | Ultima entrada es [0.5.0] 2026-07-31. Hay 25+ commits posteriores sin documentar: serie ADMIN-01..07, DESKTOP-20, AUDREP-21/23/26/41/42/43/46/47/48/50/52/54/56/57/58/59/61. No existe seccion [Unreleased] | Regenerar changelog con `git cliff` o añadir [Unreleased] antes del proximo release |
| ERR-051 | 🔵 | Doc verificado OK | `src/cli.rs` vs `src/bin/vanta-cli.rs` | Subcomandos clap (Namespace, Migrate, Snapshot, Wal) existen y tienen handlers; no hay drift visible | — |

> **Nota de verificacion del lead:** ERR-017 (score euclidiano flat vs HNSW) fue DESCARTADO — la evidencia en `src/index/distance.rs` (`calculate_similarity` paths en lineas 495, 516, 536) y `flat.rs:43` (usa `calculate_similarity` compartido) y `search.rs:176/327` muestra **metrica unica** `-euclidean_distance_squared_f32` en todos los modos. Hallazgo descartado por falta de evidencia.

---

## RESUMEN EJECUTIVO

| Gravedad | Cantidad | IDs |
|----------|----------|-----|
| 🔴 Critico | 5 | ERR-010, ERR-016, ERR-021, ERR-022, ERR-035 |
| 🟠 Alto | 17 | ERR-001,002,003,004,011,012,013,017_,018,019,020,023,024,025,037,038,039,040,041 |
| 🟡 Medio | 13 | ERR-005,014,026,027,028,029,030,042,043,044,045,046,050 |
| 🔵 Bajo | 9 | ERR-015,031,032,033,034,047,048,051 |
| ⚪ Info | 6 | ERR-006,007,008,009,049 |

**Prioridad de accion sugerida:**
1. **ERR-010** (raza checkpoint/snapshot — corrupcion/duplicacion de datos) → vanta-chaos/arch para test de interleave
2. **ERR-021** (OOM en MCP) y **ERR-022** (alloc gigante por top_k sin clamp) — crash de proceso
3. **ERR-016** (WHERE silenciosa en parser) — perdida de datos de query
4. **ERR-035/036** (locks de lectura bloqueando writers) — contención global
5. Resto: triage semanal

_Pendiente opcional:_ correr `cargo miri test` (ERR-009) antes de tocar ops.rs/vfile.rs.