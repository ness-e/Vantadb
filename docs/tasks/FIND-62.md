# FIND-62: commit_transaction bajo insert_lock + test de interleaving (vanta-worker)

## Metadata
- **Backlog:** `docs/Backlog.md` (fila FIND-62, Alta, 1-2d, correctness/durabilidad)
- **Plan:** `docs/plans/2026-09-04-durability-release-readiness.md` Task 1, Wave 1
- **Creado:** 2026-09-05
- **Estado:** ⏳ IN PROGRESS (DISCOVERY completo, pre-mortem verde, entra a ACT)
- **SDP:** `source-driven-development` + `doubt-driven-development` + `ponytail` (base auto) + `campaign-executor` + `progreso` + `incremental-implementation` + `test-driven-development` + `context-engineering` (lifecycle BUILD) + `systematic-debugging` (bug-fix). Cargadas vía `skill`. Keywords: `insert_lock`, `commit_transaction`, `ERR-010`, `flush`, `checkpoint_seq`, `deadlock`, `WAL batch`.
- **Sub-agente / Ruta:** vanta-worker (bug-fix)
- **Área:** `src/storage/engine/txn.rs:119-213`, `src/storage/engine/tests/ops.rs` (nuevo test). NO tocar `wal.rs`, `vector/`, `storage/` backends (out-of-scope worker).
- **Paralelo declarado:** Wave 1 (FIND-62 + GOV-TK7 + STABLE-06, archivos disjuntos). NO stagear `.opencode` (submodule con contenido de otras sesiones) ni ningún archivo ajeno al blast radius.

## Gate D (question-gates.md) — NO dispara
- Blast radius: 1 archivo productivo (`txn.rs`, +~10/-3 líneas) + 1 archivo de tests (`tests/ops.rs`, +~70 líneas). <10 archivos, 0 API pública nueva, 0 `pub fn` nuevos, contrato no ambiguo (test named + suite + clippy + fmt). Bug-fix con fix acotado ya diseñado en FIND-61 Step 3 → sin `question` al usuario. Procede a EJECUCIÓN.

## Blast Radius (Discovery — lectura completa)
- `src/storage/engine/txn.rs:119-213` `commit_transaction()`: WAL `batch_append([Begin+ops+Prepare])` (L159-161) + apply loop (`apply_insert_with_txn`/`apply_delete`, L167-195) + `Commit` marker (L208-210) — **SIN `acquire_insert_lock` en ningún punto** (0 refs a insert_lock en txn.rs, verificado). Contraste: insert (insert.rs:100), batch_insert (insert.rs:750), delete (delete.rs:55), delete_batch (delete.rs:234), flush (maintenance.rs:47-48) SÍ lo toman. Hueco ERR-010 confirmado por lectura (origen: FIND-61 Step 3, BENCHMARKS §13.1).
- `src/storage/engine/maintenance.rs:36-93` `flush()`: guard ERR-010 `[drain → serialize → count checkpoint_seq → write]` (L47-48); checkpoint DESPUÉS de serializar (L67-93). Interleaving violador: commit-WAL-durable → flush-drain-vacío → serialize → count-incluye-commit → checkpoint → commit-pushea-tarde = invisible record en recovery.
- `src/storage/engine/ops.rs:124-204`: `flush_pending_hnsw` (toma lock) / `drain_hnsw_batch_locked` (requiere lock held, NUNCA lo adquiere) / `try_push_pending_hnsw` (oportunista vía `try_lock`, nunca bloquea — seguro bajo guard held: el op queda encolado).
- `src/storage/engine/delete.rs:122-157`: `apply_delete(id)` → `apply_delete_inner(id, acquire=true)` → **RE-ADQUIERE insert_lock (L148)**. Trampa mortal del fix: bajo guard held → timeout 5000ms (no-reentrante). El fix DEBE llamar `apply_delete_inner(id, false)`.
- `src/storage/engine/insert.rs:210-246` `apply_insert`: NO adquiere insert_lock (asume caller); llama `drain_hnsw_batch_locked()` (L246, requiere guard). `apply_insert_with_txn` (txn.rs:254-306, usado por commit): solo `try_push_pending_hnsw`, SIN drain — bajo el fix hay que drenar explícitamente tras el loop.
- `src/storage/engine/mod.rs:318` `insert_lock: FairMutex<()>` no-reentrante; `acquire_insert_lock` (L581-594): `try_lock_for(insert_lock_timeout_ms=5000, config.rs:179)`; wasm `try_lock`.
- `.opencode/rules/concurrency-async.md` R-8: orden global `cardinality_stats → insert_lock → {wal, pending_hnsw_batch, hnsw, vstore, volatile_cache, backend}`. `apply_insert_stats`/`apply_delete_stats` toman `cardinality_stats.write` + `self.get()` (get NUNCA toma insert_lock — lectores RCU). Ningún path retiene el guard de stats mientras adquiere insert_lock (insert/delete hacen stats y lo sueltan ANTES del guard — secuencial, no anidado) → anidar `insert_lock → stats` dentro del commit es orden consistente, sin inversión. Sin deadlock.
- Callers de `commit_transaction` (80 matches, solo 1 productivo + tests): `vantadb-mcp/src/handlers/tools.rs:2131` (collection_delete: begin 2088 → deletes → commit 2131; NINGÚN guard de insert_lock en el call path — el handler no toca `insert_lock` en absoluto) + 15 calls en `src/storage/engine/tests/ops.rs` (ninguno bajo guard held: los únicos guards en tests son 1264/1468 para locked-evict, que no llaman commit). commit NO llama a flush/rebuild/compact (verificado en call graph). **PRE-MORTEM VERDE: ningún caller mantiene el guard → el fix no introduce deadlock.**
- Reglas cargadas: `durability.md` (scope storage/engine — sin Must violado: no se cambia backend ni WAL format), `core-engine.md` R-3 (propagar `?`, 0 unwrap nuevo) + R-4 (0 unsafe nuevo), `concurrency-async.md` R-8 (orden de locks + variante `_locked`/`acquire=false`).

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** txn.rs (514L), maintenance.rs flush (36-125), delete.rs (318L), insert.rs (60-326 + 740-848), ops.rs (100-229), mod.rs (300-400), tests/ops.rs (1-250 + 1000-1370), tests/mod.rs, tests/maintenance.rs (400-520 + 660-707), tests/init.rs (349-530), config.rs (insert_lock_timeout_ms), tools.rs (2080-2140), rules (durability, core-engine, concurrency-async), ADR-037, BENCHMARKS §13.1, Backlog FIND-62, plan Task 1.
- **Referencias hacia dentro:** el fix toca `commit_transaction` + `apply_delete_inner(id,false)` (variante existente, 0 código nuevo fuera de txn.rs) + `drain_hnsw_batch_locked()` (existente). WAL format intacto (mismo batch + Commit marker).
- **Referencias entrantes:** 1 caller productivo (MCP collection_delete) + tests. MVCC visibility intacta (stamps sin cambio). Recovery intacto (Commit marker igual).
- **Veredicto:** BAJO y contenido. 1 guard + 1 drain + 1 flag `false`. Reversible (un commit atómico). Gate Regla 0 ✅ — se puede entrar a ACT.

## Contrato
Test `commit_flush_interleaving` (flush concurrente durante commit → todos los records visibles post-recovery) verde + suite `storage` verde + `cargo clippy -D warnings` + `cargo fmt --check` limpios. Commit: `fix(storage): commit_transaction bajo insert_lock + test interleaving (FIND-62)` (SOLO `src/storage/engine/txn.rs` + `src/storage/engine/tests/ops.rs`).

## Steps
### Step 1: RED — test commit_flush_interleaving
- **Acción:** en `src/storage/engine/tests/ops.rs` (junto a tests txn), test `#[cfg(any(feature="fjall",feature="rocksdb"))]`: engine en disco con WAL (patrón `open_disk_engine_with_wal` de init.rs) + Barrier commit-vs-flush ×N rondas (insert_in_txn buffer + commit concurrente con flush en otro thread, watchdog 30s anti-hang estilo FND-02) → flush final → drop → reopen → assert todos visibles vía `get()`. Vectores distintos por nodo (anti-patología HNSW, patrón ERR-014).
- **Verify:** `cargo nextest run -p vantadb --lib storage::engine::tests::ops::commit_flush_interleaving` — RED esperado: en pre-fix el test PUEDE pasar (ventana de race estrecha) → se documenta como regression-gate de interleaving (el root cause está probado por lectura FIND-61, no por flake). Lo que SÍ debe verificar en RED: compila + corre + no-hang.
- **Estado:** ✅ DONE (2026-09-05 — PASS pre-fix en 1.244s: compila, corre, no-hang; confirma harness válido como regression-gate; el root cause VIOLA sigue probado por lectura FIND-61 Step 3, no por flake)

### Step 2: GREEN — commit_transaction bajo insert_lock
- **Acción:** en `txn.rs`, tras el drain del buffer y antes del `batch_append` (paso 4): `let _guard = self.acquire_insert_lock("acquire insert_lock in commit_transaction (ERR-010)")?;` retenido hasta el `Commit` marker (paso 6). Cambiar `self.apply_delete(*id)` → `self.apply_delete_inner(*id, false)` (caller-holds-lock, NO re-adquiere). Tras el apply loop: `self.drain_hnsw_batch_locked()?;` (patrón insert()). Early-returns (txn inactiva / buffer vacío, solo marker Commit sin mutaciones HNSW) quedan SIN lock — mínimo diff, 0 cambio de comportamiento en esos paths. 0 unwrap/expect nuevos, 0 unsafe.
- **Verify:** test Step 1 verde + suite storage verde + clippy/fmt limpios.
- **Estado:** ✅ DONE (2026-09-05 — guard ERR-010 + `apply_delete_inner(id,false)` + `drain_hnsw_batch_locked()`; hallazgo: `apply_delete` quedó dead-code → eliminado wrapper, docs preservadas en `apply_delete_inner`; clippy -D warnings limpio)

### Step 3: Cierre (verify + commit scoped + plan sync sin stagear)
- **Acción:** `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo nextest run --profile audit` (o al menos `-p vantadb --lib storage::` + full lib si tiempo) + `git add` SOLO los 2 archivos + commit mensaje exacto del plan. Actualizar plan file Task 1 → COMPLETO + recitation SIN stagear (precedente GOV-TK3). `campaign_memory_write(file="lessons")` 1-2 entradas. NUNCA `git add .opencode` ni archivos ajenos.
- **Verify:** commit hash + `git status` muestra solo plan/task files sin stagear fuera del commit.
- **Estado:** ✅ DONE (2026-09-05 — commit `19a9651c` 3 files +119/-12, hooks pre-commit ok; plan Task 1 COMPLETO + recitation sin stagear; ajenos intactos)

## Dependencias
- FIND-61 Step 3 (root cause VIOLA documentado) — landed. ADR-037 (ERR-010 invariante) — landed. Regla 8 (variantes locked) — landed.

## Notas
- Si Step 2 mostrara contención medible (commit serializado con inserts) → es el tradeoff declarado de ERR-010 (correctness > throughput); NO optimizar sin bench (Regla 9). Registrar como nota, no como bloqueo.
- Si algún verify falla 2× mismo-error → Gate V: STOP + RESULTADO 🟡 con evidencia, no FAILED silencioso.
