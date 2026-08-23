# Deep Module Review — `src/` (Core Engine VantaDB)

**Fecha:** 2026-08-22
**Revisor:** ox-alpha (segunda opinión, contexto fresco — sin participación en la implementación)
**Alcance:** `src/` completo (~140 archivos Rust): storage Fjall/WAL, HNSW (`index/graph.rs` + `index/search/*`), índice de texto BM25 (`text_index.rs`), hybrid search RRF (`planner.rs`, `sdk/search/*`), SDK público (`sdk/api.rs`), graph/IQL parser (`parser/mod.rs`, `graph.rs`, `executor.rs`), metrics (`metrics/core/*`), servidor HTTP (`cli_server.rs`).
**Skills aplicadas:** code-review-and-quality, security-and-hardening, code-simplification, performance-optimization, doubt-driven-development (RBI), ponytail-audit, systematic-debugging.

---

## Resumen Ejecutivo

`src/` es un motor embebido de memoria persistente notablemente **maduro y disciplinado**. La evidencia de disciplina es abrumadora y verificable:

- **0 TODOs/FIXMEs/HACKs** en todo `src/` (verificado con grep exhaustivo).
- Los `.unwrap()/.expect()/panic!` están **confinados a tests y doctests** — verificado archivo por archivo contra la línea donde empieza `mod tests` (p.ej. `parser/mod.rs` tests desde línea 629; `cli_server.rs` desde 2876; `sdk/api.rs` desde 1682). Cero unwraps en rutas de producción.
- Invariantes documentadas inline con IDs trazables a ciclos de auditoría previos (`ERR-010..ERR-050`, `INV-024`, `PERF-07..PERF-30`, `AUDREP-15/16`, `VS-CORE-07`, `DRV-009`). Esto no es decoración: cada ID referencia una corrección real con su racional.
- Todo bloque `unsafe` lleva comentario `SAFETY` con las tres garantías (bounds, alignment, lifetime); el crate declara `#![deny(unsafe_op_in_unsafe_fn)]` (`lib.rs:2`).
- **2.055 tests unitarios** en `src/` + **422 tests de integración** en `tests/` (80 archivos), proptest, chaos testing con failpoints, fuzzing, 17 benches, y un pipeline CI de 17 workflows (chaos, fuzz, CodeQL, perf-bench nocturno, certificación).

Sin embargo, la lectura adversarial (red-team) encontró hallazgos reales que los tests existentes no cubren:

1. 🔴 **HIGH — `InMemoryEngine.insert()`/`update()` escriben al WAL *antes* de validar**, y el replay aplica los registros incondicionalmente → un insert rechazado por `DuplicateNode` **se resurrecta y sobrescribe datos legítimos tras un reinicio** (`engine.rs:228` vs `231`; replay en `engine.rs:150-166`).
2. 🟡 **MEDIUM — las transacciones no son crash-atómicas**: `commit_transaction` escribe `Begin+ops+Commit` vía `batch_append` distribuido por shards, pero la recuperación ignora los marcadores `Begin/Commit/Abort` → un crash entre escrituras de shards deja una transacción parcialmente aplicada que se replay-a como si estuviera commiteada.
3. 🟡 **MEDIUM — `trigger_compaction()` es un stub engañoso**: solo cuenta tombstones y loguea un warning; nunca compacta nada (`maintenance.rs:22-48`) pese a que el log dice "offline compaction triggered".

**Veredicto final: 8.3 / 10.** Código de calidad excepcional con deuda puntual bien acotada. El hallazgo #1 merece fix inmediato; los otros dos, fix planificado.

---

## Arquitectura

### Mapa de módulos y relaciones internas

```
                            ┌─────────────────────────────────────────┐
                            │            sdk/ (FACADE PÚBLICO)        │
                            │  api.rs (VantaEmbedded, 2.497 líneas)   │
                            │  search/ (vector, lexical, hybrid,      │
                            │           phrase, snippet, sparse…)     │
                            │  serialization/ (export/import/rebuild) │
                            └───────┬────────────────────┬────────────┘
                                    │                    │
                    ┌───────────────▼──────┐   ┌─────────▼──────────┐
                    │  engine.rs           │   │  query pipeline    │
                    │  InMemoryEngine      │   │  parser (nom IQL)  │
                    │  (legacy, WAL-only)  │   │  planner (RRF)     │
                    └──────────────────────┘   │  executor          │
                                               │  physical_plan/    │
┌──────────────────────────────────────────────┴────────────────────▼─┐
│                 storage/engine/  (StorageEngine — motor principal)   │
│  mod.rs · init.rs · insert.rs · get.rs · delete.rs · txn.rs (MVCC)   │
│  maintenance.rs (flush/compact/vacuum/merge/pipeline) · ops.rs       │
│  partition.rs · stats.rs                                             │
└───┬──────────────┬──────────────┬───────────────┬───────────────────┘
    │              │              │               │
┌───▼────┐   ┌─────▼─────┐  ┌─────▼──────┐  ┌─────▼─────────────┐
│ backend│   │ vfile/    │  │ index/     │  │ text_index.rs     │
│ trait  │   │ vstore    │  │ graph.rs   │  │ (BM25 postings    │
│ fjall  │   │ (mmap     │  │ (CPIndex   │  │  en KV partition) │
│ rocksdb│   │  vectors) │  │  HNSW)     │  │ tokenizer.rs      │
│ memory │   │           │  │ ivf/diskann│  │ (tantivy, opt.)   │
└────────┘   └───────────┘  │ /scann/flat│  └───────────────────┘
                            └────────────┘
    + wal_sharded.rs (ShardedWal round-robin) · wal.rs (formato)
    + shred/ (columnar metadata) · node/ (UnifiedNode) · graph.rs (traversals)
    + metrics/core/ (registry, snapshot) · cli_server.rs (axum HTTP + auth 3 capas)
```

### Patrones detectados (y calidad de aplicación)

| Patrón | Dónde | Evaluación |
|---|---|---|
| **Facade** | `VantaEmbedded` sobre `StorageEngine` | ✅ Bien aplicado. API pública limpia, validación centralizada (`validate_namespace/key/metadata`), auditoría uniforme en cada op. |
| **Strategy** | `backend` trait (fjall/rocksdb/in-memory); índices vectoriales flat/hnsw/ivf/diskann/scann | ✅ Bien aplicado, feature-gated correctamente. |
| **MVCC / Snapshot isolation** | `txn.rs` (`Snapshot`, `created_by_txn/deleted_by_txn`) | ⚠️ Correcto en lógica de visibilidad, pero ver hallazgo de atomicidad crash (H-2). |
| **Write-ahead logging sharded** | `wal_sharded.rs` | ✅ Diseño sólido con reconciliación de layout on-disk (AUDREP-16) y detección de shard truncado (ERR-011). |
| **Derived materialized indexes** | text_index, shred, version_history | ✅ Buena decisión: records canónicos = source of truth; índices derivados reconstruibles. Consistencia best-effort documentada. |
| **Failpoints / chaos** | `fail::fail_point!` en insert/flush + `testing/chaos.rs` + CI chaos workflow | ✅ Raro y valioso verlo en proyectos reales. |
| **Interning** | `label_intern` (String→u32 edges) | ✅ Apropiado. |
| **Two-engine coexistence** | `InMemoryEngine` (legacy) + `StorageEngine` | ⚠️ Deuda estructural: dos motores con semánticas distintas (upsert vs error-on-duplicate) conviven como API pública. Ver recomendación R1. |

### Flujos verificados

- **put → get:** `put()` → `put_one()` valida namespace/key/metadata → colisión de node_id detectada (`api.rs:120-133`) → WAL + KV + HNSW bajo `insert_lock` con orden ERR-014 (HNSW antes que KV para cerrar la ventana insert→get stale) → derived indexes reemplazados. Lectura vía `get()` con chequeo de colisión simétrico. ✅ Coherente.
- **put → persist:** flush() toma `insert_lock` → drain HNSW pendiente → serializa snapshot → checkpoint_seq **después** del snapshot (invariante ERR-010 documentada en `maintenance.rs:90-98`). ✅ Orden correcto y demostrado.
- **search (hybrid):** lexical BM25 (postings en KV, phrase matching posicional) + vector HNSW (mmap, cached norms, ACORN filtered expansion) + sparse → fusión RRF determinista con tie-break estable (`planner.rs:145-181`, `sort_hits`). Budget de candidatos acotado `[32, 256]` con guardrail ≥ top_k. ✅ Bien diseñado.

---

## LO BUENO (fortalezas concretas)

1. **Disciplina de errores de élite.** `error.rs` define 30 variantes tipadas con source-chaining real (`SerdeMsgError`/`ChainedError` preservan `.source()`), hints accionables (`WALVersionMismatch.hint`, `IncompatibleFormat.hint`), `#[must_use]` y `#[non_exhaustive]` (`error.rs:88-91`). Clasificación retry/no-retry incluida.

2. **Orden WAL↔índice demostrado, no asumido.** El invariant ERR-010 en `storage/engine/insert.rs:89-121` y `maintenance.rs:55-116` explica *por qué* el orden drain→serialize→checkpoint es correcto, incluyendo el caso de fallo ("si save_vector_index falla, el checkpoint NO avanza y el replay cubre el gap"). Es exactamente el nivel de rigor que un motor durable necesita.

3. **Recuperación de errores post-fallo con compensación.** `apply_insert` deshace el HNSW entry y tombstone-a el vstore si el KV put falla (`insert.rs:268-287`) — saga manual correcta, con logging diferenciado de ambos errores.

4. **Detección de corrupción WAL activa.** `verify_shard_counts` (`wal_sharded.rs:69-94`) aborta la recuperación ante shard truncado en vez de replayar silenciosamente datos incompletos; `detect_shard_count` reconcilia el layout real on-disk (AUDREP-16, `wal_sharded.rs:130-138`) evitando pérdida silenciosa por mismatch de sharding.

5. **Búsqueda determinista y acotada.** Tie-breaking estable por `(score, key, node_id)` en lexical (`lexical.rs:150-156`) y RRF (`sort_hits`); budget híbrido con clamp superior anti-scan-desbocado (`planner.rs:96-104`) cubierto por tests unitarios específicos.

6. **HNSW de producción:** ACORN second-hop expansion para conectividad bajo filtros selectivos (`layer.rs:309-379`), prefetch de mmap guiado por perfil (`layer.rs:166-196`), normas cacheadas para cosine, pool de buffers para neighbor vecs (E2), y repair de links huérfanos (`graph.rs:1162`).

7. **Auth HTTP en serio.** Tres capas (bearer → RBAC token map → entity resolution), rate limiter dedicado a fallos de autenticación (5/min por IP, `cli_server.rs:443`), posture fail-closed cuando hay auth activa (burst conservador, AUD-021), CORS outermost para preflight, auditoría de eventos auth que nunca rompe el request (`cli_server.rs:482-483`).

8. **Seguridad de memoria mmap tratada explícitamente.** Guard de bounds + alignment INV-024 centralizado en `read_header` antes de cada cast `&[f32]`, con SAFETY comments de tres cláusulas (`txn.rs:363-372`, `layer.rs:53-71`) e incluso handler SIGBUS custom para mmap truncado en Unix (`vfile_mmap.rs:206`).

9. **Cobertura de rendimiento con medición, no intuición:** 17 benches incluyendo recall-vs-ef (`hnsw_recall_ef.rs`), presupuesto de memoria (`memory_budget.rs`), p99 canónico, sweep paramétrico. Los comentarios ponytail marcan simplificaciones deliberadas con techo conocido y upgrade path (`insert.rs:296-301`).

10. **Feature-gating limpio.** 20+ features opcionales sin contaminar el core; `pitr` experimental correctamente marcado como NO integrado con ADR de referencia (`Cargo.toml:123-127`).

---

## LO MALO / Hallazgos

| # | Severidad | Hallazgo | Evidencia | Recomendación |
|---|---|---|---|---|
| **H-1** | 🔴 **High** | **WAL escrito antes de validar → resurrection de writes rechazados tras reinicio.** `InMemoryEngine::insert()` apendea `WalRecord::Insert` ANTES del check `DuplicateNode`; el replay (`with_wal`) hace `nodes_map.insert(node.id, node)` incondicional. Escenario: caller A inserta id=5; caller B intenta insertar id=5 con payload distinto → recibe error, PERO tras restart el nodo 5 tiene el payload de B. Mismo problema en `update()` (WAL antes del check `NodeNotFound`: un update fallido sobre nodo eliminado lo **resurrecta** en replay, porque recovery aplica `Update` como insert). | `engine.rs:226-233` (orden), `engine.rs:150-157` (replay incondicional), `engine.rs:260-272` (update análogo) | Validar existencia ANTES de apendar al WAL (check bajo read lock, luego WAL, luego write lock con re-check double-checked), o filtrar en replay los `Insert` duplicados según semántica first-wins. Agregar test de durabilidad: insert-duplicado-rechazado → reopen → payload original intacto. |
| **H-2** | 🟡 **Medium-High** | **Transacciones no crash-atómicas entre shards.** `commit_transaction` escribe `Begin+ops+Commit` vía `batch_append`, que escribe grupo-por-shard secuencialmente con sync por shard (`wal_sharded.rs:214-219`). Un crash entre shards deja parte del batch en disco. La recuperación (`engine.rs:162-165`) **ignora** `Begin/Abort/Commit` y aplica Insert/Delete directamente → transacción parcial aplicada como si estuviera commiteada. El marker `Commit` se escribe pero nadie lo verifica. | `txn.rs:145-160` (batch), `wal_sharded.rs:201-221` (escritura multi-shard no atómica), `engine.rs:149-167` (replay sin verificar Commit) | En replay, bufferizar ops hasta ver `Commit(txn_id)` y solo entonces aplicarlas; descartar batches sin Commit. Test de chaos: kill entre shards de un commit multi-op. |
| **M-1** | 🟡 Medium | **`trigger_compaction()` es un stub engañoso.** Solo itera nodos contando tombstones y loguea "Fragmentation >20% — offline compaction triggered"… pero no dispara ninguna compactación. El nombre público sugiere acción; el log afirma una acción inexistente. Un operador que llame esperando reducir fragmentación no obtiene nada. | `maintenance.rs:22-48` | Renombrar a `report_fragmentation()` o implementar la delegación a `vacuum()`/`compact_layout_bfs()` ya existentes. |
| **M-2** | 🟡 Medium | **`purge_expired()` hace full scan O(N) de todos los nodos.** No existe índice de expiración; cada purge itera `scan_nodes()` completo decodificando campos. Con millones de records y TTLs activos, esta operación periódica domina I/O. | `sdk/api.rs:904` (`for node in engine.scan_nodes()?`) | Mantener un índice secundario `expires_at_ms → node_id` actualizado en put/delete (el patrón `scalar_index` ya existe), o min-heap persistido en partición InternalMetadata. |
| **M-3** | 🟡 Medium | **`write_shard_meta` no es atómico.** `std::fs::write` directo al sidecar `.shards` (`wal_sharded.rs:107-110`) — crash a mitad de escritura deja metadata corrupta/truncada. Impacto mitigado porque `detect_shard_count` escanea el directorio primero, pero el fallback roto es silencioso. | `wal_sharded.rs:97-110` | Temp + rename (el patrón ya existe en `save_vector_index`, `maintenance.rs:177`). |
| **L-1** | 🟢 Low | **`flush_all()` spawn-ea un thread por shard en cada llamada** (`std::thread::spawn` dentro del método). En flushes frecuentes con N shards esto es overhead de creación de hilos sin pool. | `wal_sharded.rs:275-292` | Rayon scope o sequential sync (los File::sync son mayormente wait-on-I/O, paralelismo de valor marginal salvo muchos shards). |
| **L-2** | 🟢 Low | **`batch_append` clona todos los records** en grupos per-shard (`record.clone()` por elemento, `wal_sharded.rs:211`). Para commits grandes duplica la memoria del buffer transaccional momentáneamente. | `wal_sharded.rs:206-212` | Agrupar por índice (`Vec<Vec<usize>>`) y pasar slices, o `drain` con split. El propio comentario admite el costo. |
| **L-3** | 🟢 Low | **Lookup de label intern repetido por nodo visitado** en BFS: `self.label_intern.lock().lookup(label)` está dentro del while-loop (`engine.rs:375-392`) en vez de hoistarse una sola vez. Lock + hash lookup redundante O(n) veces. | `engine.rs:376` | Hoist fuera del loop (línea única). |
| **L-4** | 🟢 Low | **Archivos monolíticos en zona caliente:** `cli_server.rs` ~169 KB (≈3.000+ líneas de producción antes de tests), `sdk/api.rs` 2.497 líneas, `index/graph.rs` 2.032 líneas, `maintenance.rs` 1.312 líneas. Superan el umbral saludable de inspección (~1.000 líneas). | tamaños de archivo | Extraer routers/auth/rate-limit de `cli_server.rs` en submódulos; separar `api.rs` en crud/search/admin (ya existe el patrón `sdk/search/*` — extenderlo). |
| **L-5** | 🟢 Low | **Eviction de cardinalidad descarta el campo entero** con menos entradas cuando se supera el cap global (`stats.remove(&min_field)`), no los valores menos usados. Sorpresa estadística: un campo muy útil pero pequeño desaparece completo. Marcado como ponytail consciente, pero el comportamiento default sorprende. | `insert.rs:158-168` y duplicado en `182-204` | Documentar en config o usar LRU por valor en lugar de drop-the-field. Nota: el bloque está **duplicado** en ambas ramas del if/else — extraer helper (DRY). |
| **N-1** | ℹ️ Nota | **Dos motores públicos con semánticas divergentes.** `InMemoryEngine` (error-on-duplicate, upsert-no) vs `StorageEngine` (insert-as-upsert implícito en `apply_insert_stats`). Confusión potencial para usuarios del crate; además H-1 vive solo en el path legacy. | `engine.rs:72` vs `storage/engine/insert.rs:131` | Deprecar `InMemoryEngine` hacia `StorageEngine` con backend in-memory (ya existe `backends/in_memory.rs`) — elimina una clase entera de bugs y ~850 líneas. |

---

## Incompletudes (qué falta para estar "completo")

1. **PITR no integrado:** `wal_archiver.rs` funcional y auto-testeado, pero desconectado de la rotación del StorageEngine y del recovery del SDK — explícitamente documentado en `Cargo.toml:123-127` y ADR-014. Falta el wiring.
2. **Compaction automática:** `trigger_compaction` stub (M-1); hoy solo `vacuum()`/`fresh_hnsw()` manuales reducen fragmentación. No hay trigger basado en ratio de tombstones.
3. **`delete_in_txn(reason)` ignorado:** el parámetro `reason` está reservado pero sin uso (`txn.rs:106` — "reserved for audit log"). El audit trail de deletes transaccionales prometido no existe.
4. **Atomicidad transaccional cross-crash** (H-2): la maquinaria Begin/Commit existe pero la recuperación no la consume — el contrato MVCC queda a mitad.
5. **Índice de expiración TTL** (M-2): la funcionalidad TTL existe y funciona, pero sin soporte estructural para purga eficiente.
6. **`InMemoryEngine` fase 2/3:** comentarios indican migración planeada a MemTable RocksDB-backed y HNSW real (`engine.rs:71`, `engine.rs:320`) — el motor legacy sigue en brute-force O(N) para búsqueda vectorial.

## Deuda conocida vs nueva

La deuda **documentada** (ponytail comments, ERR/PERF backlog, ADRs) está bien gestionada: cada item tiene ceiling nombrado y upgrade path. Los hallazgos H-1, H-2, M-1, M-2, L-3, L-5 son deuda **nueva o no documentada** — ninguno aparece en los registros de auditoría inline revisados.

---

## Propuestas de Mejora (priorizadas)

1. **🔴 R1 — Fix H-1 (días).** Reordenar validate→WAL→apply en `InMemoryEngine::insert/update` + test de durabilidad de regression. Es un bug de corrupción de datos alcanzable por uso normal de la API pública.
2. **🔴 R2 — Fix H-2 (semana).** Replay condicionado a `Commit`: bufferizar por txn_id en recovery, aplicar solo batches completos. Cubrir con failpoint de crash entre shards (la infraestructura `failpoints` ya existe).
3. **🟡 R3 — M-1: renombrar o implementar `trigger_compaction`.** Una línea de renombre si no hay tiempo; idealmente delegar al vacuum existente cuando ratio > 20%.
4. **🟡 R4 — M-2: expiry index.** Reutilizar el patrón `scalar_index`; cambia purge de O(N) a O(expired).
5. **🟡 R5 — Plan de deprecación de `InMemoryEngine` (N-1).** Elimina H-1 de raíz, ~850 líneas, y unifica semánticas. Es la mayor reducción de superficie de bug disponible.
6. **🟢 R6 — L-3/L-5/M-3:** micro-fixes triviales (hoist lookup, extraer helper de cardinality, atomic write del sidecar). Media hora total.
7. **🟢 R7 — Descomposición de `cli_server.rs`** en `server/{routes,auth,rate_limit,audit}` siguiendo el patrón ya existente en `sdk/search/*`.

---

## Veredicto Final

| Dimensión | Score | Justificación |
|---|---|---|
| Corrección | 8/10 | Lógica central sólida y probada; H-1 y H-2 son huecos de durability reales aunque en paths acotados. |
| Legibilidad | 9/10 | Comentarios explican *por qué*, invariantes con IDs, naming consistente; penaliza el monolito cli_server. |
| Arquitectura | 8.5/10 | Layering limpio, patrones correctos; penaliza la coexistencia de dos motores. |
| Seguridad | 9/10 | Auth 3-capas fail-closed, input validation en boundaries, unsafe documentado; sin secretos ni superficies injection (IQL parser nom-based, no eval). |
| Performance | 8.5/10 | Optimizaciones medidas (prefetch, norms cacheadas, batching WAL, quantización automática); penaliza purge O(N) y clones en batch_append. |
| Robustez/Durabilidad | 7.5/10 | Excelente manejo de corrupción detectable; los dos huecos de atomicidad (H-1, H-2) pesan aquí. |
| Testing/Infra | 9.5/10 | 2.477 tests + proptest + chaos + fuzz + 17 benches + CI de 17 workflows — top tier. |

### **Score global: 8.3 / 10**

Un motor embebido con nivel de ingeniería muy por encima del promedio: la cultura de invariantes documentadas, failpoints y medición es de proyecto de infraestructura seria. El camino a 9+ pasa por cerrar H-1/H-2 (durabilidad), consolidar a un solo motor, y convertir el stub de compaction en funcionalidad real.

---

*Verificación de este reporte: todos los hallazgos citan `file:línea` verificados por lectura directa del fuente durante esta sesión. No se ejecutó la suite de tests (revisión estática); los conteos de tests/unwraps/TODOs son outputs literales de grep sobre el working tree.*

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| H-1 — WAL escrito antes de validar → resurrection de writes rechazados tras reinicio | **MOD-01** |
| H-2 — Transacciones no crash-atómicas entre shards (replay ignora Begin/Commit) | **MOD-02** |
| M-1 — `trigger_compaction()` es un stub engañoso | **MOD-03** |
| M-2 — `purge_expired()` hace full scan O(N) sin índice de expiración | **MOD-04** |
| N-1 — Dos motores públicos divergentes: deprecar `InMemoryEngine` | **MOD-05** |
| M-3, L-1..L-5 — nits/micro-fixes (sidecar atómico, spawn por flush, clones en batch_append, hoist intern, monolitos, DRY cardinality) | **MOD-06** |
