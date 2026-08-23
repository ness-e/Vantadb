# MOD-17 — Deadlock potencial `OpGate::drain()` espera condvar sosteniendo GIL en `close()`

## Objetivo
`close()` concurrente no puede colgar el intérprete: el wait del condvar del
gate corre FUERA del GIL (`py.detach`). Test de estrés (N threads operando +
close() simultáneo) termina sin hang con timeout duro; `pytest -q` verde.

## Impacto mapeado (Regla 0)
**Archivos leídos completos (regiones relevantes, verbatim vía codegraph):**
- `vantadb-python/src/lib.rs:88-162` — `OpGate` (`Arc<(Mutex<OpState>, Condvar)>` std), `try_enter`, `drain` (:132-139), `OpGuard` + Drop, `enter`.
- `vantadb-python/src/lib.rs:1708-1718` — `close(&self, py)`: llama `self.op_gate.drain()` **con GIL vivo**, luego `py.detach(engine.close())`.
- `vantadb-python/src/lib.rs:853-882` — patrón `put`: `enter(gate)` → prepara input con GIL → `py.detach(engine.put)` → re-adquiere GIL al retornar → drop guard.
- `vantadb-python/Cargo.toml` — pyo3 **0.29** (`Python::detach` vigente), crate-type cdylib.
- `vantadb-python/tests/conftest.py` (88L, MOD-16) — fixture autouse cierra DBs; close() idempotente.

**Causa raíz (confirmada contra review H2 `docs/reviews/modulos/vantadb-python.md:60`):**
Thread A entra a `close()`, toma GIL y se bloquea en `cvar.wait` sosteniéndolo
esperando `count == 0`. Thread B está dentro de su propio `py.detach`
(`engine.put`) con OpGuard activo; para salir necesita RE-ADQUIRIR el GIL →
nunca lo suelta A → `count` nunca llega a 0 → deadlock mutuo del intérprete.
Escenario realista con `AsyncVantaDB` (`asyncio.to_thread`).

**Referencias hacia dentro:** `drain` tiene UN solo caller (`close`, codegraph).
WASM/node tienen su propia copia del gate SIN GIL (single-threaded JS / napi async)
→ fuera de alcance de este fix.
**Referencias entrantes:** tests existentes llaman `db.close()` mono-hilo —
semántica observable intacta (closing=true rechaza ops nuevas; espera in-flight;
luego engine.close() idempotente).
**Veredicto:** fix acotado a call-site en `close()` (+ derive Clone en OpGate,
2 líneas). No cambia lock-ordering ni agrega locks nuevos (Regla 8: std::sync only,
ni dashmap/parking_lot/tokio → auditoría chaos no disparada por letra de la regla).

## Discovery (evidencia)
- Fuente oficial PyO3 0.29 (pyo3.rs/v0.29.0/parallelism): *"You should always call
  `detach` … especially so in cases where worker threads need to acquire the GIL,
  to prevent deadlocks."* — caso exacto de este bug.
- Pre-mortem resuelto: NO existe `impl Drop for VantaDB` que llame drain (grep:
  solo `impl Drop for OpGuard`, Rust puro sin GIL); `close()` solo es dispatcheable
  con intérprete vivo → mover el wait a `py.detach` no introduce detach-en-shutdown.
  Se mantiene `drain()` como método Rust puro (usable sin token Python).
- Entorno build: `.venv\Scripts\maturin.exe develop` reconstruye `vantadb_native.pyd`
  in-place; pytest usa ese .pyd → ciclo RED (binario buggy) → GREEN (binario fixed) viable.

## Spec
1. `#[derive(Clone)]` en `OpGate` (Arc clone; permite mover una copia owned al closure Send de `detach`).
2. `close()`: `let gate = self.op_gate.clone(); py.detach(move || gate.drain());`
   antes del `py.detach(engine.close())`. Doc-comments actualizados (drain exige
   llamarse sin GIL cuando hay threads Python in-flight).
3. Test nuevo `tests/test_close_concurrency.py`: 4 workers spinning put/get +
   closer thread ejecuta `db.close()`; aserción `closed.wait(30)` (timeout duro =
   RED si hay hang; threads daemon → pytest sale limpio); errores ≠ "database is
   closing" fallan el test. Sin marker slow (~1s).

## Steps
- ⬜ Step 1: RED — escribir test estrés, rebuild binario buggy (`maturin develop`), correr test → debe FALLAR por timeout (hang reproducido).
- ⬜ Step 2: GREEN — aplicar fix en lib.rs (Clone + detach en close), rebuild, test pasa.
- ⬜ Step 3: VERIFY full — pytest -q suite completa + fmt/clippy/nextest workspace + validate-docs-coverage; commit conventional; cierre.

## Contrato
Test de estrés concurrente (N threads operando la DB mientras otro llama
close()) termina SIN hang (timeout duro 30s); `pytest -q` exit 0 en
vantadb-python (fixture autouse MOD-16 ya commiteada en deefc919).

## Pre-mortem / riesgos
- ~~py.detach durante interpreter shutdown~~ → descartado (sin Drop impl que drene; ver Discovery).
- Reorder de fields del struct → NO requerido con esta aproximación (drain ya es Rust puro).
- Falso-negativo del test (workers terminan antes de close): workers usan stop-event loop infinito hasta cierre → siempre hay ≥1 op in-flight cuando close llega.

## Context Save Point
- Estado: DISCOVERY completo, task file creado, implementación no iniciada.
- Repro RED: `& .venv\Scripts\maturin.exe develop` + `.venv\Scripts\python.exe -m pytest tests/test_close_concurrency.py -q` desde `vantadb-python/`.
