# MOD-06: Nits agrupados WAL (flush thread-per-shard, clones batch_append, lookup loop, cardinality dup, write_shard_meta, PITR)

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 5)
- **Fuente:** backlog core.md P32 — nits agrupados de WAL
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟢
- **Tipo:** Rust
- **Turns estimados:** 12
- **Creado:** 2026-08-25T14:30
- **last-synced:** 2026-08-25T14:30
- **Estado:** ✅ COMPLETED (implementación verificada; commit pendiente del lead)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/storage/engine/{insert,delete,txn}.rs` (batch_append), `src/engine.rs:456` (flush_all), `src/storage/engine/init.rs:430-431` (read_shard_meta/detect_shard_count), `src/storage/wal.rs` (init_wal) |
| Callees | `src/wal.rs` (WalWriter::batch_append/sync), `crate::node::LabelIntern` |
| Implicaciones | Sin cambio de semántica del WAL (round-robin por shard preservado, batch-append por shard preservado). `ShardedWal` es `pub(crate)` → cambiar firma de `batch_append` a `Vec<WalRecord>` NO es cambio de API pública. `WalWriter` (público) NO se toca. engine.rs lookup hoist sin cambio de comportamiento. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `src/wal_sharded.rs` (848L, completo)
  - `src/storage/engine/insert.rs` (905L: 1-140, 140-219, 555-684, 720-789)
  - `src/storage/engine/txn.rs` (130-169)
  - `src/storage/engine/delete.rs` (230-274)
  - `src/engine.rs` (195-244, 370-414)
  - `src/wal.rs` (WalWriter 182-459 — solo lectura, NO se edita)
  - `src/node/label.rs` (LabelIntern — solo lectura)
- **Archivos referenciados hacia dentro (dependencias):** `crate::wal::{WalWriter, WalRecord}`, `crate::config::SyncMode`, `parking_lot::Mutex`, `crate::node::LabelIntern`
- **Archivos que referencian a los editados (referencias entrantes):**
  - `src/wal_sharded.rs` ← `src/engine.rs:19`, `src/storage/wal.rs:11`, `src/storage/engine/mod.rs:346`, `src/storage/engine/init.rs:430`
  - `src/storage/engine/insert.rs` ← `mod.rs`, `txn.rs` (llaman `apply_insert_stats` / `batch_insert`)
  - `src/engine.rs` ← lib.rs, cli_handlers, sdk
- **Veredicto impacto:** BAJO — refactor de higiene sin cambio de comportamiento público. Riesgo principal: cambio de firma `ShardedWal::batch_append(&[WalRecord])` → `Vec<WalRecord>` rompe los 3 callers si no se actualizan juntos (mismo PR).

## Contrato
"`cargo nextest run -p vantadb -E 'test(wal)|test(txn)'` pasa + `cargo check -p vantadb` + fmt + clippy de archivos tocados; sin cambio de comportamiento público; suite `durability_recovery` verde."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Semántica round-robin del WAL intacta (no revertir DRV-014 batch-append por shard).
  2. `WalWriter` (público, re-exportado en lib.rs:186) NO cambia su API.
  3. `ShardedWal` es `pub(crate)` → firma interna editable.
  4. Orden de registros por shard en `recover()` idéntico (los moves preservan el orden round-robin).
  5. `write_shard_meta` escribe el MISMO contenido, solo con publicación atómica.
- **Comandos de verificación:**
  - `cargo nextest run -p vantadb -E 'test(wal)|test(txn)'` → 0 fail
  - `cargo check -p vantadb` → 0 warnings
  - `cargo fmt --check` → limpio
  - `cargo clippy -p vantadb --all-targets -- -D warnings` → limpio
  - `cargo nextest run -p vantadb durability_recovery` → 0 fail
- **Deuda pendiente:** PITR sin wiring — requiere decisión humana (ADR). NO implementado. Se documenta en Notas y en Backlog como fila FIND.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | MOD-06: nits agrupados WAL |
| `lastAction` | DISCOVERY completo: codegraph + lectura de 6 archivos; nits confirmados en código |
| `result` | PARTIAL (en curso) |
| `nextAction` | Step 1: editar `src/wal_sharded.rs` (flush_all secuencial + write_shard_meta atómico + batch_append moves) |
| `contract` | ver Contrato + Invariantes; evidencia en steps |
| `nextTask` | MOD-11 (Task 6 del plan) |

## Deuda técnica (Regla 6 — MUST)

**Sin deuda nueva.** Elimina: N clones de `WalRecord` en batch_append (deuda perf), spawn de N threads por flush, double lookup en BFS loop, ~40 líneas duplicadas de cardinality. Saldo neto negativo (pago de deuda).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file + tests wal/txn + durability_recovery + fmt/clippy |
| **Commit** | El lead commitea (sub-agente NO commitea). Commit atómico convencional `refactor(wal): ...`. |
| **Release** | N/A (no toca release; el lead decide). |

## Herramientas necesarias
- cargo, cargo-nextest, codegraph_explore

## Investigation Notes
- No se requirió web research: todos los patrones (flush, batch_append, LabelIntern) están en el repo, sin APIs externas ambiguas.
- `std::fs::rename` en Rust reemplaza destino en Windows (MoveFileExW + MOVEFILE_REPLACE_EXISTING) → temp+rename es seguro para atomicidad.
- `write_shard_meta` corrupto → `read_shard_meta` → `None` → fallback `detect_shard_count` (fuente primaria). Riesgo real bajo pero el fix es 3 líneas.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (5/5 steps ✅) |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)
N/A — no es bug, es refactor de higiene.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica: no toca trust boundaries, input de usuario, auth, ni dependencias. Solo refactor interno de WAL.
- [x] **PERFORMANCE** — aplica PARCIALMENTE: elimina clones y threads en paths de escritura, pero es limpieza, NO optimización medible. **Regla 9:** no se requiere bench because no se reclama mejora de performance — solo reducción de overhead obvio (clone/thread-spawn). Los cambios preservan semántica; no se toca el hot path de serialización (wal.rs intacto).

## Steps

### Step 1: wal_sharded.rs — flush_all secuencial + write_shard_meta atómico
- **Archivos:** `src/wal_sharded.rs`
- **Acción:** (a) `flush_all`: iterar shards secuencialmente (elimina spawn de N threads por llamada; mismo resultado de durabilidad); (b) `write_shard_meta`: escribir a `*.shards.tmp` + `std::fs::rename` atómico.
- **Verify:** `cargo check -p vantadb` + `cargo nextest run -p vantadb wal_sharded` — ✅ check Finished, tests wal_sharded 17/17 pass
- **Estado:** ✅ COMPLETED

### Step 2: wal_sharded.rs — batch_append mueve en vez de clonar
- **Archivos:** `src/wal_sharded.rs`
- **Acción:** cambiar firma `batch_append(&self, records: Vec<WalRecord>)` y `groups[idx].push(record)` (move, sin clone). Comentario de doc actualizado.
- **Verify:** `cargo check -p vantadb` — ✅ Finished
- **Estado:** ✅ COMPLETED

### Step 3: callers batch_append — pasar Vec owned
- **Archivos:** `src/storage/engine/txn.rs:159`, `src/storage/engine/insert.rs:781`, `src/storage/engine/delete.rs:263`
- **Acción:** `sharded.batch_append(&wal_records)` → `sharded.batch_append(wal_records)` (los 3 callers no reusan `wal_records` después).
- **Verify:** `cargo check -p vantadb` — ✅ Finished
- **Estado:** ✅ COMPLETED

### Step 4: engine.rs — hoist lookup fuera del loop BFS
- **Archivos:** `src/engine.rs:383-406`
- **Acción:** mover `let label_id = self.label_intern.lock().lookup(label);` antes del `while` (label invariante en el loop).
- **Verify:** `cargo check -p vantadb` + tests engine traverse — ✅ check Finished; test_traverse_returns_neighbors pass
- **Estado:** ✅ COMPLETED

### Step 5: insert.rs — extraer bump_cardinality (dedup)
- **Archivos:** `src/storage/engine/insert.rs`
- **Acción:** extraer helper `fn bump_cardinality(stats: &mut CardStats, node: &UnifiedNode)` que unifica el bloque duplicado (increment + cap eviction) de las ramas overwrite e insert-only en `apply_insert_stats` (líneas 148-168 y 183-204). Type alias local `CardStats`.
- **Verify:** `cargo nextest run -p vantadb -E 'test(wal)|test(txn)'` + `cargo nextest run -p vantadb cardinality` — ✅ 76/76 wal+txn; 28/28 cardinality+batch_append+traverse
- **Estado:** ✅ COMPLETED

### Step 6: Verify full + cierre
- **Archivos:** — (sin edición)
- **Acción:** suite completa del contrato: `cargo nextest run -p vantadb -E 'test(wal)|test(txn)'`, `cargo check -p vantadb`, `cargo fmt --check`, `cargo clippy -p vantadb --all-targets -- -D warnings`, `cargo nextest run -p vantadb durability_recovery`. Actualizar task file, memory_write(lessons), memoria decisions para PITR.
- **Verify:** exit 0 en todos
- **Estado:** ✅ COMPLETED — 76/76 wal+txn, 14/14 recovery/durab, 28/28 cardinality/batch/traverse, clippy ✅, fmt ✅ (5 archivos; api.rs diff es de FIND-31, no mío), cargo check ✅

## Dependencias
- Ninguna (Wave 1; FIND-31 y MCP-34a corren en paralelo, no comparten archivos).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (delegar antes de cerrar; el lead orquesta).
- **Enfoque:** ¿el approach es el correcto? ¿alternativas mejores?
- **Cómo se probó:** evidencia de verificación real, no auto-reporte
- **Veredicto:** pendiente

## Notas
- **PITR sin wiring (decisión pendiente):** `src/wal_archiver.rs` existe pero NO está conectado a ningún flujo (init_wal no lo usa). Implementarlo requiere decisión de diseño (dónde se engancha: recovery, CLI, API) → Regla 5: ADR humano. Se documenta aquí y se registra en Backlog como fila FIND para que el lead decida.
- **flush_all secuencial vs paralelo:** para 2-8 shards el spawn de threads por llamada es overhead puro; fsyncs al mismo disco se serializan de todos modos. Comportamiento de durabilidad idéntico (todos los shards sincronizados antes de retornar).
- **batch_append con Vec:** los 3 callers construyen `wal_records` owned y no lo reusan → el move es gratis y elimina N clones de `UnifiedNode` (payload + vector + fields).
- `std::thread::scope` existe como alternativa pero sigue spawnando threads; secuencial es más simple y correcto para el caso.
