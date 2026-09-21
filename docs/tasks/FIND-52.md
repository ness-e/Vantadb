# FIND-52: Regresión runtime wasm32 (panics std::time/thread::sleep) + deuda ICE release + pkg stale

## Metadata
- **Plan file:** — (ejecución directa desde Backlog, orden del orquestador 2026-09-02; digest de investigación aportado por vanta-research)
- **Creado:** 2026-09-02T18:30
- **last-synced:** 2026-09-02T18:05
- **Estado:** ✅ COMPLETED
- **SDP:** base del agente (incremental-implementation + test-driven-development + context-engineering — el pipeline aporta el workflow RED→GREEN explícito) + `ponytail full` + reglas de área leídas: `.opencode/rules/js-ecosystem.md`, `.opencode/rules/durability.md` (nada obsta la migración web_time). Sin candidatos extra justificados: root-cause ya investigado por vanta-research — no reinvestigar.
- **Commits:** `a137bdc7` (fix) + `docs(progreso)` closure
- **Sub-agente:** vanta-worker
- **Área:** `src/**` (rutas compiladas bajo feature `wasm`) + `vantadb-ts/package.json`. NO tocar `src/server/errors.rs` (recién cambiado por FIND-55) ni `vantadb-mcp/`. NO tocar toolchain (`rust-toolchain.toml`).

## DISCOVERY (digest vanta-research — insumo, verificado en código)

### (a) 30 panics de vitest — root cause
- `std::thread::sleep` en `src/storage/engine/init.rs:234` (retry file-lock) → panic `condvar::no_threads` en wasm32-unknown (viene del parker interno de `thread::sleep`; no hay `Condvar` propio en src/).
- `std::time::Instant::now()` / `SystemTime::now()` → panic `sys::time::unsupported` en estos sitios (verificados en código, 2026-09-02):
  - `src/storage/engine/init.rs:205` (retry file-lock — ya importa `web_time::Instant` línea 7, el call usa path crudo `std::time::`)
  - `src/storage/engine/mod.rs:158` (tipo campo `FsSnapshot::created_at` — hallazgo extra propio, necesario para que 641/679 tipen), `:641`, `:679` (constructores FsSnapshot), `:784` (`SystemTime::now()` en restore staging)
  - `src/index/search/profile.rs:11-12,23-24,29` (`SearchProfile` bajo `#[cfg(debug_assertions)]` — `wasm-pack --dev` lo compila y corre en todo search)
  - `src/index/graph.rs:1162` (`repair_orphan_links` — hallazgo propio: cae en el rg del contrato `src/index`, 1 línea, drop-in)
- `web-time = "1.1.0"` ya es dependencia raíz y se usa en ~30 archivos (patrón: `use web_time::Instant;` / `use web_time::{SystemTime, UNIX_EPOCH};` — re-exporta std fuera de wasm32 → comportamiento nativo idéntico).
- `Duration` crudo (`std::time::Duration`) es inofensivo en wasm (tipo de dato puro, no sys call) → NO migrar.

### (a′) Verificados NO-impacto (documentado, sin cambio)
- `src/sdk/api.rs:235,268` → ambos dentro de `mod tests` (`#[cfg(test)]`) — no entran al build wasm de wasm-pack.
- `src/index/core.rs:132,137` → stress test con `thread::spawn` (cfg(test)).
- `src/wal_shipping.rs`, `src/ingestion.rs:82` → módulos `#[cfg(feature = "wal-shipping"/"async-ingestion")]`; el build wasm usa `default-features = false, features = ["wasm"]` → NO se compilan.
- `src/lsm.rs:11` → importa `std::time::Instant` pero jamás llama `now()` (campo `Option<Instant>` asignado solo a `None`); el tipo compila OK en wasm32 → no es panic site. Sin cambio (YAGNI); migrar si algún día se inicializa.
- `src/server/*`, `src/tui/*`, `src/cli_handlers/*`, `src/bin/*` → featuregated fuera del build wasm.

### (b) ICE/`0xc0000409` de `wasm-pack --release` NO bloquea release
- `release-npm-61.yml` corre ubuntu-latest → CI unaffected. Es `__fastfail` mal-reportado (rust#141757/#120955, host Windows; fixes LLVM wasm en 1.97+, estable hoy 1.98). `rust-toolchain.toml` pinea 1.95.0 → bump es tarea SEPARADA (deuda a documentar, no ejecutar acá).

### (c) pkg prebuilt sin `vantadb_new`
- `vantadb-wasm/pkg/` es gitignored → era artefacto LOCAL stale (glue/.wasm desync de 29/8). Fix: rebuild local con `--dev` + `engines` `">=22.12"`→`">=22.19"` en `vantadb-ts/package.json` (ESM instantiating wasm sin flag desde Node 22.19/24.5).

## Blast Radius (verificación propia sobre el digest)
- `FsSnapshot.created_at`: único consumidor es `create_snapshot` en `sdk/builder.rs:253` (retorna el struct); `handlers.rs:1550` documenta que NO es serializable (por eso el wire lleva name+path). Cambio de tipo `std::time::Instant`→`web_time::Instant`: en nativo es idéntico (re-export), en wasm habilita la construcción de snapshots. Sin consumidores rotos.
- `SearchProfile`: `pub(crate)`, usado solo por `index/search` — API interna, tipos swap-in.
- `repair_orphan_links`: `pub` en CPIndex, usado por `FreshHnswReport` (mismo file). `Instant::now()` local → solo cambio de import/path.

## Contrato

vitest 278/278 (o documentado residual con evidencia backtrace) AND rg confirmatorio `rg -n "std::time::(Instant|SystemTime)::now|std::thread::sleep" src/storage src/index src/lsm.rs src/ingestion.rs` 0 hits (o solo cfg(test)/cfg-not-wasm comentado) AND node check del glue `function` AND `cargo check --workspace --all-targets` 0 AND `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 AND `cargo test -p vantadb --lib` 0 failed (init.rs retry logic verde en nativo) AND `cargo fmt --all -- --check` 0.

Commit: `fix(wasm): web_time + cfg-out thread::sleep — 30 panics vitest (FIND-52)`

## Steps

### Step 1: RED — reproducir 30 panics
- **Acción:** `wasm-pack build --dev --target bundler` en `vantadb-wasm/` + `npx vitest run` en `vantadb-ts/`.
- **Verify:** ✅ RED confirmado: **29 failed / 227 passed / 22 skipped (278)** — panics `unsupported.rs:13` (`Instant::now`) y `no_threads.rs:20` (`Condvar::wait`). **Hallazgo de backtraces (2 contribuyentes extra sobre el digest):** (B) `Condvar::wait` ← `vantadb_wasm::VantaDB::close`→`OpGate::drain` (NO init.rs:234); (C) `<std::time::Instant>::now` ← `parking_lot::util::to_deadline` en TODO put (12 sitios `try_lock_for`).
- **Estado:** ✅ DONE

### Step 2: GREEN (a) — migrar a web_time + cfg-out sleep
- **Archivos:** `src/storage/engine/init.rs`, `src/storage/engine/mod.rs`, `src/index/search/profile.rs`, `src/index/graph.rs`
- **Acción:** los sitios de (a) a `web_time::{Instant, SystemTime}` + clases B/C descubiertas en RED: cfg-out del `while count>0` condvar en `OpGate::drain` (wasm) y helper `StorageEngine::acquire_insert_lock` (nativo `try_lock_for` idéntico / wasm `try_lock` → mismo `VantaError::Timeout`) en los 12 sitios.
- **Estado:** ✅ DONE

### Step 3: VERIFICAR GREEN
- **Acción:** rg confirmatorio + rebuild `--dev` + vitest → 278/278 (si quedan fails: backtraces del panic_hook → 2º contributor → fix o FIND nueva; NO declarar verde si no es verde).
- **Resultado:** 1er ciclo: **2 failed / 276 passed** — los 2 residual NO eran panics sino grupo previously-unexecuted (el archivo abortaba antes): (i) `count records in namespace` aserta `toBe(1n)` filtrando por pseudo-campo `"key"` que el core nunca implementó (`matches_advanced_filters` solo lee metadata — verificado nativo y node) → test-stale en scope declarado de FIND-52, actualizado a semántica metadata; (ii) `put accepts sparse_vector` → `invalid type: string "1", expected u32`: objeto JS llave-string no pasa serde-wasm-bindgen (a diferencia de serde_json) → adapter `deserialize_sparse_vector` en la frontera wasm. 2º ciclo: **278/278 ✅**.
- **Estado:** ✅ DONE

### Step 4: (c) — node load check + engines bump
- **Acción:** `node --input-type=module -e "import('./vantadb-wasm/pkg/vantadb_wasm.js').then(m=>console.log(typeof m.VantaDB))"` → `function` (documentar nombre exportado real). `vantadb-ts/package.json`: `engines` `>=22.12` → `>=22.19`.
- **Resultado:** `typeof m.VantaDB === 'function'` ✅ (nombre exportado real = clase `VantaDB`; `vantadb_new` es interno de wasm-bindgen → undefined en el glue, documentado). Engines bump verificado contra changelog Node (unflag `--experimental-wasm-modules`: 24.5.0 + backport 22.19).
- **Estado:** ✅ DONE

### Step 5: (b) — documentar deuda toolchain (sin tocar)
- **Acción:** nota en `docs/operations/OBSERVABILITY.md`... verificar archivo natural; si no, registrar en este task file + `campaign_memory_write(lessons)`.
- **Resultado:** nota de deuda colocada en comentario de `rust-toolchain.toml` (archivo natural — ya documentaba targets/rationale): ICE Windows-only, fixes 1.97, estable 1.98, bump = tarea separada, CI ubuntu no afectado. Toolchain sin tocar.
- **Estado:** ✅ DONE

### Step 6: gates nativos
- **Verify:** `cargo check --workspace --all-targets` 0 ✅, `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 ✅, `cargo test -p vantadb --lib` 1983/0 failed ✅ (retry/Timeout nativo intacto), `cargo test -p vantadb-wasm` ok ✅, `cargo check --target wasm32` 0 ✅, `cargo fmt --all -- --check` 0 ✅, rg contrato: 0 hits de producción (solo cfg(test) + cfg-not-wasm comentado + prosa docs) ✅.
- **Estado:** ✅ DONE

### Step 7: Cierre
- Fila FIND-52 fuera de `docs/Backlog.md`, avance `docs/avance/activo/bindings.md` + `core-engine.md`, task file ✅, memory lesson.
- **Estado:** ✅ DONE (Backlog fila removida; registros `bindings.md` §FIND-52 + `core-engine.md` §FIND-52 (core); lesson vía campaign_memory_write)

## Dependencias
- FIND-55 (errors.rs) — ya cerrada `fefdbc93`; NO tocar ese file.
- ERR-TS-01 (registro de los 29 fails como FIND-52) — origen.

## Notas
- NO stagear `completions/*`, `.opencode` (submodule), `vantadb-wasm/pkg` (gitignored). NO tocar `stash@{0}`.
- `web_time::Duration` no existe como re-require: `std::time::Duration` se deja (inofensivo en wasm).
- Opción elegida para sleep: (i) cfg-not sobre el `std::thread::sleep` (una pasada en wasm) — el loop NO requiere múltiples pasadas en wasm porque no hay concurrencia de procesos que libere el lock asincrónicamente; helper dedicado (opción ii) sería scaffolding para un escenario inexistente.

## Context Save Point
- **Fecha:** 2026-09-02
- **Branch:** develop (sin cambiar)
- **CI pendiente:** no
- **Decisiones:** sleep cfg-out opción (i); lsm.rs/ingestion.rs sin cambio (verificado no-panic-site); toolchain intocado (deuda en comentario `rust-toolchain.toml`); OpGate::drain cfg-out del wait (imposible en single-thread, la barrera conserva cierre); sparse adapter en frontera wasm (no en core); count test-stale actualizado a semántica documentada (metadata).
- **Problemas conocidos:** ICE release local Windows (rust#141757) — no bloquea CI (ubuntu); bump toolchain = tarea separada.
- **Próxima tarea:** —
