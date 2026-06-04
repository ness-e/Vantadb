# FASE-05: Concurrent HNSW — Fine-Grained Locking

**Objetivo:** Eliminar la contención global del `RwLock<CPIndex>` para permitir búsquedas simultáneas desde múltiples hilos sin bloqueo por inserts. Inserts permanecen serializados (Opción A — Search-first).

> [!NOTE]
> Este plan incorpora correcciones de dos revisiones técnicas independientes. El título original "Lock-Free" ha sido corregido: la arquitectura propuesta es **Fine-Grained Locking**, no lock-free. DashMap usa locks por shard; los atómicos coordinan metadata global. La distinción es técnicamente importante.

---

## Decisiones Resueltas

| Pregunta | Decisión | Justificación |
|:--|:--|:--|
| **DashMap vs sharded-slab** | DashMap aprobado | `sharded-slab` requiere IDs `usize` internos. VantaDB usa `u64` externo en todo el stack. La capa de traducción destruiría rendimiento. |
| **Search-first vs Full concurrent** | Opción A (Search-first) | Ratio read:write ~100:1. El 90%+ del beneficio viene de desbloquear lecturas. Inserts concurrentes requieren rediseño del enlace bidireccional HNSW con riesgo desproporcionado. |
| **Serialización de inserts** | `Mutex` en StorageEngine | Un `insert_lock: parking_lot::Mutex<()>` serializa inserts. No cola/canal: complejidad innecesaria para la carga actual. |
| **Inner RwLock por nodo** | No en esta fase | Con inserts serializados, no hay dos writers mutando nodos simultáneamente. La contención DashMap `get_mut()` vs `get()` en el mismo shard es de nanosegundos (push a Vec). Reservado para Opción B futura. |

---

## Arquitectura Propuesta

### Estado Actual (Antes)

```
StorageEngine
  └─ hnsw: RwLock<CPIndex>          ← Lock global exclusivo
       └─ nodes: HashMap<u64, HnswNode>  ← Sin concurrencia interna
       └─ max_layer: usize               ← Requiere &mut self
       └─ entry_point: Option<u64>        ← Requiere &mut self
       └─ rng: StdRng                     ← Requiere &mut self

Insert:  hnsw.write()  → bloquea TODAS las búsquedas
Search:  hnsw.read()   → bloqueado por cualquier insert en curso
```

### Estado Propuesto (Después)

```
StorageEngine
  ├─ hnsw: RwLock<CPIndex>                ← Solo write() para rebuild/sync (raro)
  └─ insert_lock: Mutex<()>              ← Serializa inserts (1 a la vez)
       │
       └─ CPIndex
            ├─ nodes: DashMap<u64, HnswNode>  ← Concurrencia por shard (~64 shards)
            ├─ max_layer: AtomicUsize          ← Monotónicamente creciente
            ├─ entry_point: AtomicU64          ← Sentinel: u64::MAX = None
            ├─ config: HnswConfig              ← Inmutable post-init
            └─ backend: IndexBackend           ← Solo mutado en rebuild

Insert:  hnsw.read() + insert_lock.lock()  → NO bloquea búsquedas
Search:  hnsw.read()                       → Concurrente con todo excepto rebuild
Rebuild: hnsw.write()                      → Exclusivo (operación administrativa rara)
```

### Flujo de Concurrencia

```
Hilo 1 (Search):   hnsw.read() ─── DashMap.get() ─── DashMap.get() ─── ✓
Hilo 2 (Search):   hnsw.read() ─── DashMap.get() ─── DashMap.get() ─── ✓   ← Ambos concurrentes
Hilo 3 (Insert):   hnsw.read() + insert_lock ─── DashMap.insert() ─── DashMap.get_mut() ─── ✓
                    ↑ NO bloquea hilos 1 y 2 (excepto nanosegundos si coinciden en shard)
Hilo 4 (Insert):   espera insert_lock ─── ⏳ (serializado con Hilo 3)

Hilo 5 (Rebuild):  hnsw.write() ─── bloquea TODO (operación rara, administrativa)
```

---

## Proposed Changes

### Fase 0: Benchmark de Baseline (Obligatorio antes de cambios)

Crear un benchmark que mida el throughput actual del HNSW con 1, 4, 8 y 16 hilos de búsqueda. Este dato es imprescindible para cuantificar la mejora real de la Fase 1.

#### [NEW] [bench_concurrent.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/benches/bench_concurrent.rs)

Test de throughput multi-hilo con `criterion` o manual con `std::time::Instant`:
- Insertar N nodos (1K, 10K) single-threaded
- Medir queries/sec con 1, 4, 8, 16 hilos concurrentes
- Registrar p50, p99 de latencia por query
- Este será el baseline contra el que medimos Fase 1

---

### Fase 1: Migración Interna

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)

```diff
+dashmap = "6"
```

---

#### [MODIFY] [index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)

**C1: Imports y constantes**
```diff
+use dashmap::DashMap;
+use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
+
+/// Sentinel value for "no entry point". Using u64::MAX avoids
+/// collision with user-defined ID 0 which is a valid node ID.
+const ENTRY_POINT_NONE: u64 = u64::MAX;
```

**C2: Struct CPIndex**
```diff
 pub struct CPIndex {
-    pub nodes: HashMap<u64, HnswNode, BuildHasherDefault<XxHash64>>,
-    pub max_layer: usize,
-    pub entry_point: Option<u64>,
+    pub nodes: DashMap<u64, HnswNode, BuildHasherDefault<XxHash64>>,
+    pub max_layer: AtomicUsize,
+    pub entry_point: AtomicU64,
     pub backend: IndexBackend,
     pub config: HnswConfig,
-    rng: rand::rngs::StdRng,
+    // rng eliminado: random_layer() usa thread_rng() (thread-local, zero-lock)
 }
```

**C3: Constructores**
```diff
 impl CPIndex {
     pub fn new() -> Self {
         Self {
             nodes: Default::default(),
-            max_layer: 0,
-            entry_point: None,
+            max_layer: AtomicUsize::new(0),
+            entry_point: AtomicU64::new(ENTRY_POINT_NONE),
             backend: IndexBackend::InMemory,
             config: HnswConfig::default(),
-            rng: rand::rngs::StdRng::seed_from_u64(42),
         }
     }
     // Análogo para new_with_config() y with_backend()
 }
```

**C4: `random_layer()` — Eliminar &mut self**
```diff
-    fn random_layer(&mut self) -> usize {
-        let r: f64 = self.rng.gen_range(0.0001..1.0);
+    fn random_layer(&self) -> usize {
+        let r: f64 = rand::thread_rng().gen_range(0.0001..1.0);
         (-r.ln() * self.config.ml).floor() as usize
     }
```

**C5: Helpers de acceso para entry_point**
```rust
/// Thread-safe accessor for the current entry point.
#[inline]
fn get_entry_point(&self) -> Option<u64> {
    let ep = self.entry_point.load(Ordering::Acquire);
    if ep == ENTRY_POINT_NONE { None } else { Some(ep) }
}

/// Thread-safe setter. Uses Release ordering to ensure the node
/// is fully visible in DashMap before other threads follow this pointer.
#[inline]
fn set_entry_point(&self, id: u64) {
    self.entry_point.store(id, Ordering::Release);
}
```

**C6: `search_layer()` — Adaptación de DashMap guards**

Los `.get()` de DashMap retornan `dashmap::mapref::one::Ref<>`. El patrón cambia de:
```rust
if let Some(node) = self.nodes.get(&id) { /* usa node directamente */ }
```
a:
```rust
if let Some(node_ref) = self.nodes.get(&id) { /* usa *node_ref o node_ref.value() */ }
```

Los scopes de los guards deben ser mínimos para no retener shard locks innecesariamente. En particular, el prefetch loop debe **copiar** los datos necesarios del guard antes de llamar a funciones externas.

**C7: `add()` — Firma y coordinación**
```diff
-    pub fn add(&mut self, id: u64, bitset: u128, vec_data: VectorRepresentations, storage_offset: u64) {
+    /// SAFETY CONTRACT: Caller MUST hold StorageEngine::insert_lock.
+    /// This ensures only one insert executes at a time, avoiding
+    /// bidirectional neighbor update races.
+    pub fn add(&self, id: u64, bitset: u128, vec_data: VectorRepresentations, storage_offset: u64) {
```

Cambios internos:
- `self.nodes.get_mut(&id)` → DashMap `RefMut` guard (shard-level lock, nanosegundos)
- `self.nodes.insert(id, node)` → DashMap concurrent insert
- `self.entry_point = Some(id)` → `self.set_entry_point(id)`
- `self.max_layer = level` → `self.max_layer.fetch_max(level, Ordering::Release)`
- `self.rng` → eliminado, `random_layer()` usa `thread_rng()`

**C8: `select_neighbors()` — Sin cambio de firma**

Ya acepta `&self`. Los `self.nodes.get()` se adaptan a DashMap guards.

**C9: Serialización**
- `serialize_to_bytes()`: `self.nodes.iter()` de DashMap retorna `RefMulti`. Adaptar.
- `serialization_order()`: `self.nodes.iter().map(|r| *r.key())` en lugar de `self.nodes.keys()`.
- `deserialize_from_bytes()`: construir DashMap con `DashMap::with_hasher()` y luego insertar nodos.
- `self.max_layer` y `self.entry_point` se leen/escriben vía Atomic load/store.

**C10: `stats()` y `validate_index()`**
- Adaptar iteradores a DashMap. Sin cambio semántico.
- `self.nodes.len()` funciona igual en DashMap.

**C11: `sync_to_mmap()` — Mantiene &mut self**

Este método hace swap atómico de todo el índice. Se invoca bajo `hnsw.write()` exclusivo. Mantener `&mut self`.

---

#### [MODIFY] [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)

**C12: Añadir insert_lock**
```diff
 pub struct StorageEngine {
     // ...
     pub hnsw: RwLock<CPIndex>,
+    /// Serializes insert/refresh operations to avoid bidirectional
+    /// neighbor update races. Searches acquire hnsw.read() freely.
+    insert_lock: parking_lot::Mutex<()>,
     pub volatile_cache: RwLock<std::collections::HashMap<u64, UnifiedNode>>,
     // ...
 }
```

Inicializar en constructor:
```diff
+            insert_lock: parking_lot::Mutex::new(()),
```

**C13: Insert path — Downgrade write() a read()**
```diff
 // storage.rs L996-L1004
 {
+    let _guard = self.insert_lock.lock(); // Serializa inserts
-    let mut hnsw = self.hnsw.write();
+    let hnsw = self.hnsw.read();
     hnsw.add(
         active_node.id,
         active_node.bitset,
         active_node.vector.clone(),
         storage_offset,
     );
 }
```

**C14: refresh_index path — Análogo**
```diff
 // storage.rs L1024-L1039
 pub fn refresh_index(&self, node: &UnifiedNode, storage_offset: u64) {
     // ...
     if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
         if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
+            let _guard = self.insert_lock.lock();
-            let mut index = self.hnsw.write();
+            let index = self.hnsw.read();
             index.add(/* ... */);
         }
     }
 }
```

**C15: rebuild_vector_index() — Mantiene write()**
```rust
// Este es el ÚNICO path que necesita hnsw.write():
let mut hnsw = self.hnsw.write(); // Bloquea búsquedas temporalmente
*hnsw = rebuilt;                   // Swap atómico
```

**C16: sync_to_mmap() en CPIndex — Mantiene &mut self**
Invocado bajo `hnsw.write()` exclusivo. Sin cambios.

**C17: Accesos a `hnsw.nodes` desde storage.rs**

Los 10 sites que hacen `hnsw.nodes.get()`, `hnsw.nodes.len()`, `hnsw.nodes.is_empty()` funcionan con DashMap sin cambios semánticos. Solo adaptar el tipo de retorno (DashMap Ref vs HashMap ref).

---

#### [MODIFY] [executor.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/executor.rs)

Sin cambios. Los 2 sites (L224, L300) ya usan `hnsw.read()`.

---

#### [MODIFY] [physical_plan.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/physical_plan.rs)

Sin cambios. El site en L223 ya usa `hnsw.read()`.

---

## Análisis de Riesgos (FMEA Base)

| # | Riesgo | Sev. | Prob. | Mitigación |
|:--|:--|:--|:--|:--|
| 1 | **Contención cross-shard en `add()` durante update bidireccional** — `get_mut(nodo_A)` bloquea shard X, luego `get_mut(vecino_B)` necesita shard Y. Si otro hilo tiene Y y pide X: deadlock. | Alta | **Nula con Opción A** | `insert_lock` serializa inserts. Solo un insert a la vez. No hay dos hilos ejecutando `add()` simultáneamente. Deadlock cross-shard es imposible. |
| 2 | **Auto-deadlock en mismo shard** — `get_mut(A)` retiene guard de shard, luego `get_mut(B)` que está en el mismo shard. | Alta | **Nula con Opción A** | Mismo razonamiento: un solo insert a la vez. Además, el código de `add()` nunca retiene dos `get_mut()` guards simultáneamente (la estructura actual libera el guard del nodo actual antes de iterar vecinos). |
| 3 | **Contención búsqueda vs insert en mismo shard** — Search hace `get()` (shared shard lock), insert hace `get_mut()` (exclusive shard lock). | Media | Baja (~1/64) | Ventana de contención: nanosegundos (push a Vec). Worst case: búsqueda espera ~100ns. Aceptable. |
| 4 | **Race en `entry_point`** — Thread lee entry_point que apunta a nodo antiguo en capa inferior, mientras insert actualiza a capa superior. | Baja | Baja | Corrección algorítmica no afectada: search con entry point en capa inferior simplemente hace más trabajo. Recall no degradado, solo latencia marginalmente mayor durante la transición (microsegundos). Memory ordering: `Release`/`Acquire` garantiza visibilidad. |
| 5 | **Inconsistencia `max_layer` vs `entry_point`** — Uno actualizado antes que el otro. | Media | Baja | `fetch_max` es monotónico. Si `max_layer` se ve actualizado pero `entry_point` aún es viejo, search busca desde un entry point con menos capas — equivalente a un search con `ef=1` en capas superiores, que es correcto. Documentar como trade-off deliberado. |
| 6 | **Regresión de cache locality** — DashMap introduce indirección extra (shards, locks, guards) vs HashMap directo. | Media | Media | **Fase 0 benchmark obligatorio.** Comparar throughput single-thread antes/después. Si regresión >5%, evaluar número de shards (DashMap permite configurar). |
| 7 | **Regresión en determinismo de tests** — `thread_rng()` no es seeded. La topología HNSW será diferente entre runs. | Baja | Alta | Recall@10 ≥ 0.95 es invariante estadístico, no depende de topología exacta. Tests de serialización round-trip siguen siendo deterministas (operan sobre índices ya construidos). |

---

## Trade-offs Documentados

| Trade-off | Aceptado | Justificación |
|:--|:--|:--|
| **Consistencia eventual durante insert** | Sí | Una búsqueda concurrente puede no encontrar un nodo recién insertado cuyas conexiones bidireccionales aún no están completas. Esto es inherente a la Opción A y es aceptable: el nodo será visible en la siguiente búsqueda. |
| **Single-thread overhead** | Evaluable | DashMap tiene ~15-30ns de overhead por operación vs HashMap directo. En un search_layer que hace ~200 `get()` calls, esto es ~6µs extra. Contra latencias de cómputo de distancia (~50µs × 200 = 10ms), es ~0.06%. Despreciable, pero confirmaremos con Fase 0. |
| **Pérdida de determinismo en RNG** | Sí | `thread_rng()` elimina la necesidad de `&mut self` por RNG. La calidad estadística de la distribución de capas es idéntica. Los tests de recall validan la invariante correcta (estadística, no determinista). |

---

## Verification Plan

### Fase 0: Baseline

Ejecutar benchmark con 1, 4, 8, 16 hilos antes de cualquier cambio. Registrar:
- Queries/segundo (throughput)
- p50, p99 latencia por query
- Insert bulk 10K nodos (latencia total)

### Fase 1: Post-migración

1. **`cargo clippy --all-targets -- -D warnings`** — Zero warnings.
2. **`cargo deny check`** — Verificar que DashMap no tiene advisories.
3. **`cargo test --workspace --release`** — 0 failures.
4. **`cargo test --test stress_protocol --release`** — Recall@10 ≥ 0.95.
5. **Test existente:** `serialization_order_preserves_search_results` — Round-trip OK.
6. **Test nuevo: `concurrent_search_during_insert`**
   ```
   - 2 hilos insertando 5K nodos cada uno (secuencialmente via insert_lock)
   - 4 hilos buscando continuamente
   - Validar: zero panics, zero deadlocks (timeout 60s)
   - Post-test: validate_index() retorna Ok(())
   ```
7. **Test nuevo: `concurrent_insert_preserves_hnsw_invariants`**
   ```
   - Insertar 10K nodos con 4 hilos insertando vía StorageEngine::insert()
   - Post-insert:
     1. Todos los nodos alcanzables desde entry_point (BFS)
     2. validate_index() sin violations
     3. Recall@10 >= 0.95
   ```
8. **Benchmark comparativo** — Repetir Fase 0 benchmark. Criterio de éxito:
   - Multi-thread throughput: ≥ 2x mejora con 4 hilos vs 1 hilo
   - Single-thread throughput: regresión < 5%

### Verificación Manual

Ejecutar `cargo test --workspace --release` y confirmar. Comparar latencia de insert bulk antes/después.

---

## Evolución Futura (NO en esta fase)

> [!NOTE]
> **Opción B (Inserts concurrentes)** queda documentada como evolución posible SOLO si benchmarks post-Fase 1 demuestran que el `insert_lock` es bottleneck real en workloads de producción. Requeriría:
> - `DashMap<u64, parking_lot::RwLock<HnswNode>>` con locks por nodo
> - Lock ordering global por `min(id_a, id_b)` para evitar deadlocks en updates bidireccionales
> - Test de stress con inserts y deletes concurrentes
> - Rediseño del entry_point update con CAS loop
>
> Esta complejidad no está justificada actualmente dado el ratio read:write ~100:1.
