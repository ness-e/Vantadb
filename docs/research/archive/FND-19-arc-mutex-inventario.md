# FND-19 — Inventario `Arc<Mutex<>>` en el core (Fase 0 pre-launch)

> **Tipo:** Auditoría (NO fixes) · **Prioridad:** 🔴 · **Esfuerzo:** 🟢
> **Backlog:** P20d · **Complementa:** FND-02 (paths multi-índice) · **Auditor:** vanta-audit
> **Fecha:** 2026-08-16

## Resumen

- **Instancias `Arc<Mutex<` en `src/`:** **2** (obligatorias por contrato).
- **Clasificación:** 1 NECESARIA (wal_sharded) · 1 SOSPECHOSA (ingestion) · 0 ANIDADAS.
- **Reemplazos propuestos:** 1 (ingestion → canal multi-consumidor share-nothing).
- **Conteo total documentado:** 2 `Arc<Mutex<` literales. (Se listan además 23 `Mutex<` no-Arc como contexto de concurrencia para FND-02/Regla 8, fuera del contrato literal.)

> **Método:** `grep -rn "Arc<Mutex<" src/` + variantes (`Arc<std::sync::Mutex<`, `Arc<parking_lot::Mutex<`). Verificado que no hay instancias partidas en líneas separadas (`Arc<` al final de línea: 0). Análisis de callers vía lectura directa de los módulos.

---

## 1. Inventario `Arc<Mutex<` (contrato)

| # | Archivo:línea | Tipo | Clasificación | Justificación | Recomendación | Prioridad |
|---|---|---|---|---|---|---|
| 1 | `src/ingestion.rs:72` | `Arc<tokio::sync::Mutex<mpsc::Receiver<(IngestionTask, oneshot::Sender<Result<u128>>)>>>` | **(b) SOSPECHOSA** | Anti-patrón "shared receiver": N workers comparten un único `mpsc::Receiver` envuelto en un Mutex para repartirse tareas. El lock se mantiene a través del `.await` de `recv()`, serializando la espera de recepción y añadiendo contención innecesaria (solo 1 worker puede estar bloqueado en `recv` a la vez). `mpsc::Receiver` no es clonable, lo que fuerza el `Arc<Mutex>`. Un canal multi-consumidor resuelve esto sin lock. | **Reemplazar por canal multi-consumidor share-nothing** (`async-channel` o `flume`): cada worker posee su propio `Receiver` clonado → sin Mutex, sin contención. Alternativa mínima: `tokio::sync::mpsc` + un distribuidor único (single consumer) si el fan-out no es requisito. | 🔴 Alta |
| 2 | `src/wal_sharded.rs:10` | `Vec<Arc<parking_lot::Mutex<WalWriter>>>` | **(a) NECESARIA** | Sharding de WAL: cada shard es un `WalWriter` protegido por **su propio** lock independiente dentro de un `Vec`. `append`/`batch_append` lockean exactamente un shard (round-robin vía `AtomicUsize`), por lo que escrituras a shards distintos corren en paralelo sin contención. El `Arc` externo permite que `flush_all` clone y envíe shards a threads. Es locking fino correcto, no share-nothing global. El `Mutex` es necesario porque `WalWriter` no es `Sync` (File handle + buffer). | **Mantener.** No aplicar `DashMap` (los shards son índice fijo/ordenado, no clave hash) ni canal (es I/O síncrona, no productor/consumidor). Único matiz: el `Arc<Mutex<WalWriter>>` está a su vez dentro de `Arc<ShardedWal>` (mod.rs:346) — pero es `Arc` de ownership, NO lock-dentro-de-lock; cada shard lockea su propio `Mutex`. | 🟢 Baja |

**Totales contrato:** NECESARIA = 1 · SOSPECHOSA = 1 · ANIDADA (red flag) = 0.

---

## 2. Contexto de concurrencia no-Arc (referencia FND-02 / Regla 8)

No son `Arc<Mutex<` (fuera del contrato literal), pero son `Mutex` en paths multi-índice / almacenamiento y entran en la auditoría de concurrencia (Regla 8, FND-02). Se listan para que FND-02 las cubra si aplica — **ninguna requiere acción en FND-19**.

| Archivo:línea | Tipo | Nota |
|---|---|---|
| `src/audit.rs:54` | `Mutex<BufWriter<File>>` | Serializa escritura del log de auditoría. Necesario. |
| `src/cli_server.rs:223` | `Mutex<LruCache<...>>` | Rate-limit de fallos de auth. Lock corto. OK. |
| `src/engine.rs:86` | `parking_lot::Mutex<LabelIntern>` | Interner de labels. Lock corto. OK. |
| `src/sync_ext.rs:21` | trait `MutexExt` | Helper de extensión, no una instancia de dato. |
| `src/index/diskann.rs:48-51` | `Mutex<HashMap<...>>` ×4 | Estado del índice DiskANN (graph/vectors/bitsets/medoid). Path multi-índice — revisar en FND-02 (posible `RwLock` si lecturas dominan). |
| `src/index/graph.rs:353-363` | `parking_lot::Mutex` ×3 | rng + ivf_index + scann_index opcionales. Lock corto. OK. |
| `src/index/flat.rs:64` | `Mutex<Vec<FlatEntry>>` | Índice flat. Path multi-índice — revisar `RwLock` en FND-02. |
| `src/index/scann.rs:51-59` | `Mutex<...>` ×5 | Entries/bounds/dim. Path multi-índice — revisar en FND-02. |
| `src/vector/governor.rs:65` | `Mutex<HashMap<u128, AccessEntry>>` | Governor de acceso. Lock corto. OK. |
| `src/storage/engine/mod.rs:317` | `FairMutex<()>` | `insert_lock` serializa insert/refresh HNSW. Intencional (fairness anti-starvation). OK. |
| `src/storage/engine/mod.rs:320` | `Mutex<Vec<PendingHnswOp>>` | Micro-batch pendiente. Lock corto. OK. |
| `src/storage/engine/mod.rs:330` | `Mutex<HashSet<u64>>` | txns activas. Lock corto. OK. |
| `src/storage/engine/mod.rs:333` | `Mutex<HashMap<u64, Vec<BufferedWrite>>>` | Buffers por txn. Lock corto. OK. |
| `src/storage/engine/mod.rs:377` | `parking_lot::Mutex<LabelIntern>` | Interner. OK. |

> Nota: `engine.rs` y `storage/engine/mod.rs` usan `RwLock` para caches de lectura-dominante (volatile_cache, vector_store, text_stats_cache) — patrón ya correcto. Los `Mutex` de índice (diskann/flat/scann) son candidatos a `RwLock` solo si FND-02 demuestra que las lecturas dominan sobre escrituras.

---

## 3. Conclusiones

1. **Solo 1 instancia requiere acción real** (`ingestion.rs:72`): reemplazar el patrón `Arc<Mutex<Receiver>>` por un canal multi-consumidor (`async-channel`/`flume`) share-nothing. Elimina el Mutex y la contención de recepción. Es candidata a fix en una tarea futura (NO en esta auditoría).
2. **`wal_sharded.rs:10` es diseño correcto** (sharding de locks finos). Mantener.
3. **No hay `Arc<Mutex<>>` anidado** (lock dentro de otro lock) en el core. El único doble `Arc` (ShardedWal) es ownership, no locking.
4. Los `Mutex` no-Arc de índices (diskann/flat/scann) quedan delegados a **FND-02** para evaluar `RwLock` por ratio lectura/escritura.

**Recomendación global (proactiva):** tras el fix de ingestion, correr `cargo miri` / revisión de locks en el path de ingesta (FND-02 complementa) y considerar `loom` para el patrón de sharding del WAL (Regla 8, vanta-chaos).
