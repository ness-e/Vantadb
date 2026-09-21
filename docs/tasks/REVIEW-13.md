# REVIEW-13: supersede() TOCTOU concurrente — serializar read-modify-write

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md
- **Fuente:** plan file Task 1 (Wave 0)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Rust (core SDK, bug de concurrencia)
- **Turns estimados:** 6
- **Creado:** 2026-08-25T15:00
- **last-synced:** 2026-08-25T16:00
- **Estado:** ✅ COMPLETED (implementación worker; commit + review del lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-python/src/lib.rs:1784` (PyO3 `supersede`), `vantadb-wasm/src/lib.rs` (via inner), bindings Node/TS, tests `src/sdk/api.rs` (4 tests supersede), serialization roundtrips |
| Callees | `VantaEmbedded::get` → `StorageEngine::get` (cache+backend read), `engine.insert` → `insert_lock` (FairMutex, no reentrante), `version_history::write_snapshot`, `memory_record_to_node_owned` |
| Implicaciones | Contrato público `supersede(namespace, old_key, new_key) -> Result<()>` NO cambia (firma ni semántica). Solo serialización interna. Bindings existentes intactos. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/builder.rs` (385L, completo), `src/sdk/api.rs` supersede 840-894 + tests 1768-1817, 2540-2699 + imports; `src/storage/engine/mod.rs` struct StorageEngine (vía codegraph, verbatim); `src/storage/engine/insert.rs` (vía codegraph, verbatim); `src/engine.rs` patrón MOD-01 (vía codegraph)
- **Archivos referenciados hacia dentro:** `src/sdk/builder.rs` (VantaEmbedded) es consumido por todo `src/sdk/*` (api.rs, search/*, serialization/*, graph.rs, gds.rs) + bindings externos (vantadb-python/node/wasm/ts) + MCP + server. `src/sdk/api.rs` es el API público del core.
- **Archivos que referencian a los editados:** 92 callers de `engine_handle`; bindings Python (`supersede` en lib.rs:1784) y WASM llaman `VantaEmbedded::supersede`; tests existentes de supersede: `test_supersede_marks_old_and_leaves_new_intact`, `test_supersede_errors_on_missing_keys`, `test_supersede_errors_when_old_equals_new`, `test_supersede_idempotency_second_call_errors`.
- **Veredicto impacto:** **bajo** — cambio aditivo interno: un campo `Arc<Mutex<()>>` en `VantaEmbedded` (3 constructores) + guard en `supersede()`. No cambia API pública, no rompe callers, no requiere migración.

## Contrato
`cargo nextest run -p vantadb supersede` pasa; test concurrente nuevo (2 threads supersede mismo key → exactamente 1 gana, el otro recibe "already superseded", estado final consistente); `cargo check -p vantadb` + `cargo fmt` + `cargo clippy -p vantadb --all-targets -- -D warnings` pasan.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) idempotency guard: segundo `supersede` sobre el mismo old_key devuelve error "already superseded" — debe seguir funcionando; (2) semántica: old se marca (`superseded_by`/`superseded_at_ms`/version+1), new queda intacto; (3) `VantaEmbedded` sigue `Clone` y clones comparten el mismo engine.
- **Comandos de verificación:** `cargo nextest run -p vantadb supersede` ✅ · `cargo check -p vantadb` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅
- **Deuda pendiente:** ninguna (si se confirma el fix). Auditoría Regla 8 (data race/deadlock) la delega el lead a `vanta-chaos` + `vanta-review` antes de cerrar la PR.

## Recitation (canónico — estructura única)

- `activeGoal`: REVIEW-13 — eliminar TOCTOU concurrente en supersede() serializando el read-modify-write.
- `lastAction`: DISCOVERY completo — root cause confirmado vía codegraph; task file creado con Regla 0 mapeada.
- `result`: `PARTIAL` (en ejecución)
- `nextAction`: Step 1 — agregar `supersede_lock` en `src/sdk/builder.rs` + guard en `src/sdk/api.rs::supersede`.
- `contract`:
  - `verificacion`: `cargo nextest run -p vantadb supersede` + `cargo check -p vantadb` + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings`
  - `evidencia`:
    - claim: supersede() tiene TOCTOU (read `:851`, guard `:857`, write `:886` sin atomicidad); el `insert_lock` del engine solo serializa el write individual
      evidencia: src/sdk/api.rs:840-894 + src/storage/engine/insert.rs:33,96-99
      confianza: alta
  - `artefactos`: `.opencode/skills/campaign-executor/tasks/REVIEW-13.md`
  - `invariantes`: idempotency guard + semántica de supersede intactas; VantaEmbedded sigue Clone
  - `deuda`: ninguna
  - `queda_pendiente`: auditoría de concurrencia Regla 8 delegada al lead (vanta-chaos + vanta-review)
- `nextTask`: MOD-04 (Task 2 del plan) o la que el lead asigne.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — lock global `Arc<Mutex<()>>` marcado con `ponytail:` (supersede es op administrativa rara; striping por-namespace solo si hubiera contención). No introduce unsafe, clones ni deuda nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| Task | Contrato verifica: test concurrente + tests idempotencia existentes pasan |
| Commit | Lo ejecuta el lead (NO COMMIT del worker). Cambio ~30 líneas, conventional `fix:` |
| Release | No aplica (worker no release; verify del lead) |

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius)

## Investigation Notes
- El engine tiene `insert_lock: FairMutex<()>` (src/storage/engine/mod.rs:317) que serializa WAL+HNSW del insert individual — NO cubre el read+validate del SDK. No se puede usar directamente: `FairMutex` no es reentrante → llamar `engine.insert()` sosteniéndolo es deadlock. Por eso el lock vive en el handle compartido `VantaEmbedded` (Arc compartido entre clones).
- Patrón de referencia: `InMemoryEngine::insert` (src/engine.rs:221-253) ya documenta el anti-patrón TOCTOU (MOD-01) con write-lock sobre validate→WAL→apply.
- Lock ordering: `supersede_lock` → `engine.read()` (RwLock) → `insert_lock`. Ningún path adquiere `supersede_lock` desde adentro → deadlock-free.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY — NO aplica:** no toca trust boundaries, input de usuario (validations existentes intactas), auth, ni dependencias. Es serialización interna de una op de escritura. No se añaden/quitan deps.
- [x] **PERFORMANCE — NO aplica:** `supersede` es op administrativa rara, no hot path (no es search/ingestión/serialización). El lock añade un acquire por llamada — despreciable. No requiere bench (Regla 9 no aplica: no es optimización).

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** 2 threads concurrentes llaman `supersede(ns, "old", "new_a")` y `supersede(ns, "old", "new_b")`. Ambos leen `old.superseded_by == None` → ambos pasan el guard → ambos `engine.insert()` exitosos → 2 "ganadores" (estado divergente: `superseded_by` apunta a uno solo pero el loser no recibió error, o doble-mark). Con barrera de sincronización el race es determinista post-fix: exactamente 1 gana.
- **Hipótesis:** el read-modify-write de supersede (get → check idempotencia → insert) no es atómico; el `insert_lock` del engine cubre solo el write. Serializar la secuencia completa con un mutex compartido del handle elimina el TOCTOU sin cambiar semántica.
- **1 variable controlada:** solo se agrega el lock (campo + guard) y el test concurrente. No se toca lógica de validación, ni el write path del engine, ni el snapshot.
- **Test RED:** `test_supersede_concurrent_race_exactly_one_wins` — **RED CONFIRMADO** (guard deshabilitado temporalmente): 7/7 runs fallan con `assertion failed: exactly one supersede must win; got r1=Ok(()), r2=Ok(())` (doble-mark). Con el guard restaurado: GREEN 10/10.

## Context Save Point (2026-08-25T16:00)

**Estado de implementación:** COMPLETO. Todos los checks verdes. NO COMMIT (worker — lo ejecuta el lead).

**Verificación mecánica obtenida:**
- `cargo check -p vantadb` ✅
- `cargo nextest run -p vantadb supersede` ✅ 10/10 (incluye test concurrente nuevo + 4 tests idempotencia/estado existentes + roundtrips serialización)
- `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ (sin warnings)
- `cargo fmt --check` ✅

**RED proof:** guard deshabilitado → 7/7 iteraciones del test concurrente fallan con `r1=Ok(()), r2=Ok(())` (2 ganadores = TOCTOU doble-mark confirmado). Guard restaurado → GREEN.

**Archivos tocados (solo estos, 2):** `src/sdk/builder.rs` (+12), `src/sdk/api.rs` (+62, incl. test). Otros archivos en `git status` (layer.rs, e2e.rs, etc.) son de workers paralelos de Wave 0 — NO los toca este worker.

**Próximo paso (lead):** `git add src/sdk/builder.rs src/sdk/api.rs` + commit `fix: REVIEW-13 — serialize supersede() read-modify-write (TOCTOU)`. Auditoría Regla 8 (deadlock/data race): delegar a `vanta-chaos` (stress 10k w/s + 1k r/s) + `vanta-review` (P2-01) antes de merge.

## Steps

### Step 1: Agregar supersede_lock a VantaEmbedded + guard en supersede
- **Archivos:** `src/sdk/builder.rs`, `src/sdk/api.rs`
- **Acción:** campo `supersede_lock: Arc<parking_lot::Mutex<()>>` (pub(crate)) + init en `from_engine`/`open_with_config`/`test_empty`; `let _guard = self.supersede_lock.lock();` antes del primer `get` (cubre read+check+write). Lock ordering supersede_lock→engine.read→insert_lock; deadlock-free.
- **Verify:** `cargo check -p vantadb` ✅
- **Estado:** ✅ COMPLETED

### Step 2: Test concurrente RED→GREEN
- **Archivos:** `src/sdk/api.rs` (mod tests)
- **Acción:** test `test_supersede_concurrent_race_exactly_one_wins` (Barrier(3), new_a/new_b, asserts exact-1-gana + loser "already superseded" + estado final consistente).
- **Verify:** `cargo nextest run -p vantadb supersede` ✅ 10/10 (RED confirmado 7/7 sin guard)
- **Estado:** ✅ COMPLETED

### Step 3: Verify completo (fmt + clippy + check)
- **Archivos:** — (verify)
- **Acción:** `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + `cargo check -p vantadb`
- **Verify:** exit 0 los tres ✅
- **Estado:** ✅ COMPLETED

### Step 4: Cierre — actualizar task file + recitation
- **Archivos:** `.opencode/skills/campaign-executor/tasks/REVIEW-13.md`
- **Acción:** steps ✅, Context Save Point, bloque RESULTADO con evidencia. NO commit (lo hace el lead).
- **Verify:** bloque RESULTADO entregado en el mensaje final
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 0, independiente).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador — delegado por el lead al cierre (NO COMMIT del worker). Checklist anti-hábitos pendiente de verificación por vanta-review.

- **Revisor:** vanta-review (designado por el lead; gate Regla 8: vanta-chaos para stress 10k w/s + 1k r/s)
- **Enfoque:** ¿serializar con mutex global es el fix correcto vs re-check en sección crítica del engine? Alternativa evaluada: lock del engine NO viable (FairMutex no reentrante → deadlock con engine.insert).
- **Cómo se probó:** evidencia mecánica en Steps 1-3 (cargo check/nextest/fmt/clippy).
- **Veredicto:** pendiente

## Notas
- El plan file (Task 1) ya pre-mortemó 3 fallos: (1) lock que no cubre el read → cubierto con lock desde antes del primer get; (2) test flaky sin barrera → Barrier(3); (3) romper idempotency guard → tests existentes de idempotencia (test_supersede_idempotency_second_call_errors) se mantienen y pasan.
