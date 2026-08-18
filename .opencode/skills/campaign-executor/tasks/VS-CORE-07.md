# VS-CORE-07 — Retención de versiones históricas en `VantaMemoryRecord` (D2)

- **Plan:** `docs/plans/2026-08-18-vanta-studio-fase1.md` (Task 3, Wave 0)
- **Estado:** ⏳ ANÁLISIS COMPLETO — EN ESPERA DE APROBACIÓN HUMANA (D2 checkpoint). **NO implementar sin aprobación.**
- **Tipo:** rust · análisis/propuesta (Fase 1 de la tarea) · sin código
- **Archivos clave:** `src/sdk/types.rs:175` (`VantaMemoryRecord`), `src/sdk/api.rs:112-177` (`put_one`), `src/sdk/api.rs:243-374` (`put_batch_inner`), `src/backends/fjall_backend.rs` (keyspaces), `src/backend.rs:30-68` (`BackendPartition`), `src/storage/engine/insert.rs:33-122` (`insert`, WAL+KV), `src/wal.rs:41-73` (`WalRecord`), `src/sdk/serialization/mod.rs:394-448` (`memory_record_to_node_owned`)
- **Cláusula de doble consumidor:** P26 Studio (Historial+Diff, VS-14) + P27 memory (offload/skills versionadas, F4/F5). Diseñar UNA vez.

## Contexto verificado (código real, 2026-08-18)

| Hallazgo | Evidencia |
|---|---|
| `VantaMemoryRecord.version: u64` existe; **no hay snapshots** | `src/sdk/types.rs:189` |
| `put_one` lee el record existente, hace `version = existing.version.saturating_add(1)` (o 1) y sobrescribe el nodo **in-place** — el estado anterior se pierde | `src/sdk/api.rs:120-162` |
| `put_batch_inner` replica el mismo bump; lleva `seen_versions` para dedup intra-batch; cada chunk va por `engine.batch_insert_with_opts` | `src/sdk/api.rs:259-337` |
| `put_record_exact` (import) escribe versiones exactas sin bump | `src/sdk/api.rs:477-511` |
| Write path: `engine.insert` = WAL `WalRecord::Insert(node)` (commit point) + vstore (mmap) + `backend.put(Default, node_id, NodeMetadata)` + HNSW, dentro de `insert_lock` | `src/storage/engine/insert.rs:33-122, 226-287` |
| El KV `Default` guarda solo `NodeMetadata { relational, edges, created_by_txn, deleted_by_txn }` — **no** el payload/vector (viven en vstore) | `src/storage/engine/insert.rs:234-240` |
| Particiones por backend: `BackendPartition` (9 variantes) → Fjall keyspaces / RocksDB column families / InMemory map | `src/backend.rs:30-49`, `src/backends/fjall_backend.rs:35-46, 111-124` |
| `scan_prefix_iter` existe en el trait para listar por prefijo (clave para `versions(ns,key)`) | `src/backend.rs:172-176` |
| `delete(ns,key)` SDK lee el record existente (tiene ns/key) antes de borrar → punto natural para purgar snapshots | `src/sdk/api.rs:450-474` |
| Expiración: lectura filtra TTL lazy (`memory_record_from_node`); `purge_expired()` (api.rs:770-848) recorre records expirados y llama `engine.delete(node_id)` | `src/sdk/serialization/mod.rs:326-329`, `src/sdk/api.rs:770-848` |
| Precedente de derivado best-effort post-commit: `ShreddedRowStore::put` con `let _ =` — si falla, el record funciona igual | `src/sdk/api.rs:166-172` |
| WAL: `WalRecord { Insert/Update/Delete/Checkpoint/Begin/Commit/Abort }`; agregar variante = cambio de formato + replay + tests de recovery | `src/wal.rs:41-73` |

---

## PROPUESTA

### 1. Retención: historial completo (v1..vN) con CAP configurable (default 32)

| Opción | Coste por put | Almacenamiento | Qué permite | Veredicto |
|---|---|---|---|---|
| **Solo n-1 (anterior)** | 1 write | 1 snapshot/key | Diff vs anterior únicamente | ❌ Historial de 1 paso no sirve para P26 ni P27 |
| **n-k (últimos k)** | 1 write + 1 delete amortizado al llegar al cap | ≤ k × record/key | Historial acotado | 🟡 Válido, pero el cap con FIFO agrega un delete por put en steady-state |
| **Historial completo + cap (recomendado)** | **1 write** (insert; delete solo al evictar el más viejo al llegar al cap) | ≤ cap × record/key | Historial completo hasta el cap; diff entre cualquiera; offload versionado | ✅ **Elegido** |
| Todo sin cap | 1 write | Ilimitado (crece con cada update) | Ilimitado | ❌ Hot keys (ej. checkpoints LangGraph, `examples/python/langgraph_checkpoint.py`) crecen sin límite |

**Decisión:** retener **todas las versiones v1..vN con cap FIFO `VantaConfig.version_history_limit: Option<usize>`** (default `Some(32)`, `None` = sin límite). Al llegar al cap, cada put nuevo evicta la versión más vieja (1 delete + 1 insert). El cap hace el almacenamiento **acotado por clave** y le da techo predecible a la compacción.

- **Snapshot = el record NUEVO** (no el anterior): tras cada put se persiste el estado recién escrito bajo su versión. Invariante: *la partición de versiones es la historia completa v1..vN y el record vivo es espejo de vN*. `get_version(ns,key,vN)` funciona; `versions()` incluye el último. Coste idéntico (1 write) a snapshot-earlier, pero con API consistente (nunca "falta" la versión viva). Alternativa (snapshot del anterior) descartada porque `get_version(vN)` devolvería `None` o exigiría un fallback al record vivo.
- **Valor del snapshot:** `postcard::to_allocvec(&VantaMemoryRecord)` — reusa la serialización existente, autocontenido (payload + metadata + vector + sparse + timestamps + version). **Sin campos nuevos en `VantaMemoryRecord`** (R-7 api-contract: no gatear campos; y cero breaking en structs públicos).
- **Evicción TTL en snapshots:** el snapshot conserva su `expires_at_ms` como dato histórico (para Diff). La purga física es responsabilidad de delete/expiración (punto 4).

### 2. Coste: 1 write extra por put + impactos

- **put_one:** 1 `backend.put(Versions, key, postcard(record))` extra por put. Mismo orden de coste que el `ShreddedRowStore::put` existente.
- **put_batch_inner:** los snapshots del chunk se agrupan en **un solo `backend.write_batch`** por chunk (atómico entre sí). `seen_versions` ya da la versión correcta por clave incluso con duplicados intra-batch. Coste total ≈ 1 write extra por registro del batch.
- **Import (`put_record_exact`):** **NO genera snapshots** por defecto. El import es bulk y su fuente de verdad es el archivo; duplicar cada registro duplicaría la escritura sin valor de historial (el historial vivo arranca con los puts posteriores). Opción `import_version_history: bool` en el import si P27 lo necesita. **Decisión a confirmar por el humano.**
- **Expiración:** `purge_expired()` (api.rs:770) ya tiene el record completo (ns/key) antes de `engine.delete` → ahí se purgan los snapshots del key (scan_prefix + delete por versión). Sin purga, los snapshots de keys expirados quedan huérfanos (misma clase de basura que el record expirado no purgado — hoy la expiración es lazy).
- **Compacción:** LSM (Fjall auto; RocksDB background) maneja los tombstones de evicción/delete. Sin cambios de config backend (ADR-023: no tocar opciones sin bench — no hace falta tocar nada).
- **Storage rough:** record 1536d ≈ 6.3 KB (postcard) + payload + metadata. Con cap 32: ≤ ~200 KB por hot key; para el uso típico (records < 1 KB) es despreciable.

### 3. Clave en Fjall y partición nueva

- **Nueva partición `BackendPartition::Versions`** (aditiva): keyspace Fjall `"versions"`, column family RocksDB `"versions"`, `BTreeMap<Vec<u8>, Vec<u8>>` en InMemory.
- **Clave binaria length-prefixed, versión al FINAL en big-endian:**
  ```
  ns_len(u32 LE) ‖ ns ‖ key_len(u32 LE) ‖ key ‖ version(u64 BE)
  ```
  - `version` BE al final ⇒ `scan_prefix(ns_len‖ns‖key_len‖key)` devuelve versiones **en orden ascendente** (v1 < v2 < v10 — el `\0`-join propuesto en Fase 0 Task 19 ordenaría v10 < v2 como string y rompería si ns/key contienen `\0`).
  - `get_version` = `get(Versions, key_exacta(ver))` — 1 point-read.
- **Apertura de DBs existentes:** crear keyspace/CF es aditivo en Fjall y RocksDB → **sin migración ni backfill**. DBs pre-feature tienen la partición vacía; el historial arranca en la fecha de ship. `versions()` devuelve vacío. Backward-compat total.

### 4. API propuesta (core, aditiva — `impl VantaEmbedded`)

```rust
/// Devuelve el record tal como estaba en la versión dada.
/// `None` si esa versión nunca fue persistida (key desconocida o versión purgada por cap/delete).
pub fn get_version(&self, namespace: &str, key: &str, version: u64) -> Result<Option<VantaMemoryRecord>>;

/// Devuelve todas las versiones retenidas del key, ascendente (v1..vN).
/// Vacío si el key no existe o no tiene historial. Las versiones expiradas se incluyen
/// como dato histórico hasta su purga.
pub fn versions(&self, namespace: &str, key: &str) -> Result<Vec<VantaMemoryRecord>>;
```

- **Config:** `VantaConfig.version_history_limit: Option<usize>` (default `Some(32)`; `None` = sin límite). No es campo feature-gated (R-7 OK).
- **Integración con `VantaMemoryRecord`: NINGUNA.** El snapshot ES un `VantaMemoryRecord` (autodescriptivo). No hay campos nuevos → sin breaking en struct público, sin cambios en bindings existentes (el getter `version` de Python ya existe, `vantadb-python/src/types.rs:110`). La exposición en bridge/bindings es tarea separada (VS-14 la consume vía bridge, no directo).
- **Durabilidad (clase de garantía):** el snapshot se escribe **después del commit point (WAL)**, igual que `ShreddedRowStore` y que los derived indexes. Falla/crash entre WAL y snapshot ⇒ `versions()` muestra un gap (versión sin snapshot) — degradación honesta, nunca panic ni corrupción. **No se agrega variante `WalRecord::VersionSnapshot` en v1** (cambio de formato WAL + replay + tests de recovery + chaos = mucho más coste para un gap de ms en un crash). **Hardening diferido documentado:** si P27 exige historial crash-exacto, la variante WAL es el mecanismo (deuda P27, no bloqueante).
- **Concurrencia (nota, no se arregla aquí):** dos `put` concurrentes del mismo key ya pueden colisionar en el número de versión hoy (read-compute-write fuera de `insert_lock`). El snapshot keyed por versión hereda la colisión (last-wins). Pre-existente, fuera de scope; arreglarlo requiere lock por key (cambio de concurrencia mayor, ver concurrency-async.md).

### 5. Puntos de integración (write path)

| Punto | Cambio |
|---|---|
| `put_one` (api.rs:112-177) | Tras `engine.insert`, escribir `Versions` con el record nuevo keyed por su versión + evicción FIFO si `limit` alcanzado |
| `put_batch_inner` (api.rs:243-374) | Recolectar snapshots del chunk (usando `seen_versions`) y emitir 1 `backend.write_batch` por chunk |
| `put_record_exact` / import | Sin snapshots (decisión a confirmar; flag opt-in si P27 lo pide) |
| `delete(ns,key)` (api.rs:450) | Purga `scan_prefix` del key + delete por versión |
| `purge_expired` (api.rs:770-848) | Purga de snapshots de cada record expirado antes de `engine.delete` |
| `BackendPartition` (backend.rs:30) | Variante `Versions` + arm en `cf_name` (RocksDB) + keyspace Fjall + map InMemory. **Auditar matches exhaustivos** de `BackendPartition` en tests (compile-time, low risk) |

### 6. Tests afectados + plan de test

**Existentes — ninguno debería romperse** (aditivo; sin cambio de WAL, sin campos nuevos). Riesgo bajo: matches exhaustivos de `BackendPartition` en `src/backend.rs` tests (test_write_batch_all_partitions etc.) y el `cf_name` match — actualización compile-time. `proptest_wal_roundtrip`/`durability_recovery` intactos (no se toca WAL).

**Nuevos (plan — se escriben en la fase de implementación):**
1. `put` ×3 → `versions()` = [v1,v2,v3], live.version = 3, `get_version(2)` = payload del 2º put
2. Primer `put` crea snapshot v1; `get_version(key desconocido)` y `get_version(999)` → `None`
3. `put_batch` con keys duplicados intra-batch → snapshots reflejan la secuencia de bump
4. Cap: `version_history_limit=2`, 3 puts → `versions()` = [v2,v3] (FIFO)
5. `delete` purga → `versions()` vacío tras delete
6. `purge_expired` elimina snapshots de records expirados
7. Import (`put_record_exact`) no genera snapshots (o flag opt-in según decisión)
8. Roundtrip backend: InMemory + Fjall + RocksDB (si feature) — write/read/scan_prefix de `Versions`
9. Backward-compat: abrir DB pre-feature → `versions()` vacío, sin error de migración
10. Serialización: `postcard(VantaMemoryRecord)` roundtrip incl. vector 1536d + sparse + metadata

### 7. Recomendación

**Retener historial completo v1..vN con cap FIFO `version_history_limit` (default 32); snapshot = el record nuevo en cada put; partición `Versions` con clave length-prefixed + versión BE al final; API `get_version(ns,key,ver)` + `versions(ns,key)`; snapshots best-effort post-commit (sin cambio de WAL en v1); purga en `delete` y `purge_expired`; import sin snapshots por defecto.** Aditivo, backward-compat total, sin migración. El único trade-off asumido: gap de snapshot en ventana de crash (ms) — documentado, aceptable para P26; P27 decide si exige variante WAL (hardening diferido).

**Checkpoint humano D2:** aprobar esta propuesta (o ajustar: cap default, snapshot-new vs snapshot-previous, import con/sin snapshots) antes de cualquier implementación.

---

## Impacto mapeado (Regla 0 — fase implementación; análisis previo)

- **Archivos leídos completos:** `src/sdk/api.rs` (put_one/put_batch_inner/delete/purge_expired), `src/sdk/types.rs` (VantaMemoryRecord), `src/backend.rs` (trait+particiones), `src/backends/fjall_backend.rs`, `src/storage/engine/insert.rs` (apply_insert/insert), `src/wal.rs` (WalRecord/append), `src/sdk/serialization/mod.rs` (record↔node), `vantadb-python/src/types.rs` (getter version).
- **Referencias entrantes (a tocar al implementar):** matches de `BackendPartition` en `rocksdb_backend.rs`/`in_memory.rs`/tests (`cf_name`, keyspace resolve, exhaustivos); callers de `engine.insert` no se tocan (cambio en SDK layer).
- **Referencias salientes:** nueva API pública `VantaEmbedded::{get_version, versions}` → docs/api/ (Doc-Driven) + bindings/bridge en tareas separadas (VS-14).
- **Veredicto:** cambio aditivo, sin breaking; requiere ADR si se confirma el diseño (Regla 5 AGENTS.md) — la decisión de retención/cap es trade-off que el humano articula.

## RESULTADO

**Resumen:** Propuesta D2 completa y verificada contra código real. Retención = historial completo v1..vN con cap FIFO (default 32) vía `VantaConfig.version_history_limit`; snapshot del record nuevo en cada put (1 write extra) serializado con postcard; nueva partición `Versions` con clave `ns_len‖ns‖key_len‖key‖ver(u64 BE)` (versión BE al final para scan ordenado); API aditiva `get_version(ns,key,ver)` + `versions(ns,key)`; sin campos nuevos en `VantaMemoryRecord` (backward-compat total, sin migración); purga en `delete`/`purge_expired`; import sin snapshots (decisión a confirmar); durabilidad best-effort post-commit (misma clase que ShreddedRowStore), variante WAL diferida para P27. Sin código de implementación (checkpoint humano obligatorio). 3 puntos abiertos para el humano: cap default, snapshot-new vs snapshot-previous, import con/sin snapshots.

**Recomendación:** aprobar el diseño como está (default 32, snapshot-new, import sin snapshots) para destrabar VS-14 y el diseño compartido P26/P27.

<!--
RESULTADO (contrato pipeline):
RESULTADO: ✅ COMPLETO (fase análisis D2 — sin commit, regla explícita)
STEPS_OK: 1/1 (análisis + propuesta)
PROXIMO_STEP: aprobación humana del checkpoint D2 → implementación VS-CORE-07 (fase 2)
COMMIT_HASH: ninguno (NO commitees — regla)
ARCHIVOS: .opencode/skills/campaign-executor/tasks/VS-CORE-07.md
VERIFY_CONTRATO: propuesta completa (trade-offs + API + tests afectados) documentada; sin código
BLOQUEO: checkpoint humano obligatorio antes de implementar
-->