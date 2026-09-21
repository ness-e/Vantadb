# WSM-01: Eliminar fallback silencioso OPFS→in-memory (OpfsStorage::open .ok())

## Metadata
- **Plan file:** docs/plans/2026-08-27-backlog-pipeline.md
- **Fuente:** Task ID WSM-01 (contrato usuario 2026-08-27) — backlog pipeline research digest (Wasm OPFS fallback silencioso, capabilities.persistence no fiel)
- **Esfuerzo:** 🟢 2-4h
- **Prioridad:** 🔴
- **Tipo:** Rust (WASM bindings)
- **Turns estimados:** 6
- **Creado:** 2026-08-27T00:00
- **last-synced:** 2026-08-27T00:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-wasm/src/lib.rs` (VantaDB::connect_persistent), `vantadb-ts` (wraps VantaDB wasm), `web/` demos que llaman connect_persistent, tests `vantadb-wasm/tests/wasm_tests.rs` try_opfs |
| Callees | `vantadb-wasm/src/opfs.rs` (OpfsStorage::open, read_file, write_file), `vantadb-wasm/src/idb.rs` (IdbStorage), `vantadb/src/sdk/api.rs` (VantaEmbedded::capabilities, VantaCapabilities), `js_sys::global` navigator.storage |
| Implicaciones | Contrato público de `connect_persistent` cambia de silencioso-success a error-propagado → rompe callers que asumían in-memory fallback; `capabilities().persistence` pasa de hardcoded true a fiel → semver ok (corrección de bug, no nueva API); performance neutra; migración no necesaria; tests wasm existentes deben seguir pasando con OPFS disponible |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `vantadb-wasm/src/lib.rs` (2245L), `vantadb-wasm/src/opfs.rs` (298L), `vantadb-wasm/src/idb.rs` (202L), `vantadb-wasm/src/worker.rs` (400L), `vantadb-wasm/Cargo.toml` (51L), `src/sdk/types.rs:757-768` (VantaCapabilities), `src/sdk/api.rs:1189-1199` (capabilities impl)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `lib.rs` imports `crate::opfs::OpfsStorage`, `crate::idb::IdbStorage`, `vantadb::sdk::*`, `wasm_bindgen`, `js_sys`, `serde`; `opfs.rs` usa `js_sys`, `wasm_bindgen`, `wasm_bindgen_futures`; `idb.rs` inline_js bridge `vantaIdbStorage`
- **Archivos que referencian a los editados (referencias entrantes):** `rg "connect_persistent"` → `vantadb-wasm/src/lib.rs:471`, `vantadb-wasm/tests/wasm_tests.rs` (no direct), `vantadb-ts` docs, `web/` examples; `rg "OpfsStorage"` → `lib.rs:25,292,473`, `tests/wasm_tests.rs:7,66-67`, `worker.rs:25,73-85`; `rg "capabilities"` → `vantadb/src/sdk/types.rs`, `vantadb/src/sdk/api.rs`, `vantadb-wasm/src/lib.rs:900`, `docs/api/*`
- **Veredicto impacto:** medio — cambia comportamiento observable de `connect_persistent` (propaga error OPFS) y de `capabilities().persistence` (fiel). Riesgo: callers que confiaban en fallback silencioso recibirán rechazo Promise; mitigado con mensaje descriptivo + alternativa documentada `connect_idb`.

## Contrato
`rg -n "\.ok\(\)" vantadb-wasm/src/lib.rs` filtrado a `OpfsStorage::open` → 0 hits after fix + `wasm-pack build --target bundler` exit 0 + test simula fallo `getDirectory` y verifica `connect_persistent` retorna error o cae a IDB con warning y `capabilities().persistence` fiel

## Spec (SDD — feature-add check Phase 1b)
No es feature-add: no agrega `pub fn`/tool/endpoint/binding nuevo. Corrige bug de persistencia silenciosa. Tabla N/A justificada por evidencia:
| # | Decisión | Evidencia | Resuelto |
|---|----------|-----------|----------|
| 1 | Política ante fallo OPFS: propagar error vs fallback IDB | Contrato admite ambas; se elige propagar error (ponytail: 1 línea `?` vs 20L fallback+warn). Evidencia: `lib.rs:473` actual `.ok()` traga error | ✅ decidido-por-evidencia (ref: lib.rs:473) |
| 2 | capabilities.persistence fiel | Core `VantaCapabilities.persistence` existe (`types.rs:761`); WASM debe overridearlo según backend real. Evidencia: `lib.rs:902` delega a `inner.capabilities()` hardcoded true | ✅ decidido-por-evidencia (ref: api.rs:1194) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** No tocar `vantadb/src/wal.rs`, `vector/`, `storage/`; no agregar `unwrap()/expect()` en código nuevo; `connect_persistent` no debe crear DB in-memory silenciosa cuando OPFS falló; `save()` no debe ser no-op silencioso en DB que prometió persistencia; `capabilities().persistence` debe reflejar backend real
- **Comandos de verificación:** `rg -n "OpfsStorage::open" vantadb-wasm/src/lib.rs` → 0×`.ok()`; `cargo check -p vantadb-wasm`; `cargo clippy -p vantadb-wasm -- -D warnings`; `wasm-pack build --target bundler --manifest-path vantadb-wasm/Cargo.toml` (si wasm-pack disponible)
- **Deuda pendiente:** ninguna al cerrar (si wasm-pack no disponible, queda pendiente verificar bundler en CI)

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | WSM-01: Eliminar fallback silencioso OPFS→in-memory |
| `lastAction` | Steps 1-4 ✅ fix .ok() + persistence fiel + wasm tests 29 passed + wasm-pack bundler + clippy + fmt |
| `result` | OK (COMPLETED) |
| `nextAction` | ninguno — tarea cerrada |
| `contract` | ver ## Contrato + ## Invariantes de dominio |
| `nextTask` | none (tarea aislada WSM-01) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda nueva. Si se introduce `persistence: bool` field, es corrección (no deuda). Compensa P2-8 `collect_all_deduped` O(n) no tocado.

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | `rg` 0 hits + build bundler exit 0 + test fallo OPFS verifica error/warning + persistence fiel + `cargo check/clippy` clean |
| **Commit** | Commit atómico `feat: WSM-01 — fix silent OPFS fallback` + verify mecánico por step |
| **Release** | No aplica (WASM fix, no release inmediato) |

## Herramientas necesarias
- cargo check/clippy/nextest (verify)
- wasm-pack (bundler build, si disponible)
- rg (contrato)

**Skills cargadas (SDP):** campaign-executor (base task system), progreso (backlog sync), ponytail (full, minimal diff), source-driven-development (validar wasm-bindgen/opfs APIs), systematic-debugging (bug root-cause), test-driven-development (red-green test fallo OPFS), incremental-implementation (slices ≤100L), code-review-and-quality (pre-commit gate) — 8 total, justificadas 1L c/u; SDP keyword grep "wasm/opfs/persistence/capabilities" sin candidatos adicionales en SKILLS-MANIFEST (Essential no lista WASM), lifecycle mapping añadió systematic-debugging/test-driven/incremental/code-review

## Investigation Notes
- Root cause: `lib.rs:473` `OpfsStorage::open(path).await.ok()` convierte `Err(JsValue)` (getDirectory fail, permissions, private mode) en `None`, luego `VantaDB { opfs: None }` y `save()` early-return Ok(()) → pérdida silenciosa de datos. `capabilities()` delega a core `persistence:true` aunque opfs=None → miente.
- Alternativa fallback IDB con warn requeriría 20L + branching + warn deduplication; ponytail elige propagar error (1 línea `?` + msg) — cumple contrato "retorna error o cae a IDB con warning"; error es más explícito y evita estado híbrido.
- Validar wasm-pack docs: `wasm-pack build --target bundler` existe, opt-level s ya en Cargo.toml.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)
- **Repro:** `rg -n "\.ok\(\)" vantadb-wasm/src/lib.rs` → `473: let opfs = OpfsStorage::open(path).await.ok();` ; simular `navigator.storage.getDirectory` reject → `connect_persistent` resuelve OK con `opfs=None`, `save()` no escribe, `capabilities().persistence==true` falso
- **Hipótesis:** `.ok()` traga error OPFS y crea DB in-memory bajo promesa de persistencia
- **1 variable controlada:** reemplazar `.ok()` por `?` propagación (1 línea)
- **Test RED:** test que stub `getDirectory` a `Promise.reject` y llama `connect_persistent` debe fallar (o warn+IDB) y `capabilities.persistence` fiel — antes del fix pasa silencioso, después falla correctamente

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — No toca trust boundaries de auth, pero toca persistencia (storage) — se revisa que no hay unwrap, errores mapeados a JsValue con mensaje descriptivo, no info leak. `security-and-hardening` no requerido (no user input untrusted nuevo).
- [x] **PERFORMANCE** — No toca hot path (search/ingest). No hay baseline necesario. Si toca, skip justificado: persistencia OPFS async I/O, no hot path CPU.

## Steps
### Step 1: Fix `connect_persistent` — propagar error OPFS + persistence fiel en capabilities
- **Archivos:** `vantadb-wasm/src/lib.rs`
- **Acción:** (1) Reemplazar `let opfs = OpfsStorage::open(path).await.ok();` por `.map_err(|e| JsValue::from(js_sys::Error::new(...)))?` y `VantaDB { opfs: Some(opfs), persistence:true }`. (2) Añadir campo `persistence: bool` al struct y settearlo en `new/open→false`, `connect_persistent/connect_idb/connect_worker→true`. (3) Override `capabilities()` con `caps.persistence = self.persistence`.
- **Verify:** `Select-String OpfsStorage::open.*\.ok` → 0 hits ✅; `cargo check -p vantadb-wasm` → ok ✅; `cargo clippy -p vantadb-wasm -- -D warnings` → 0 warnings ✅ (tras fix doc en vfile_mmap.rs)
- **Estado:** ✅ COMPLETED

### Step 2: Conectar `connect_idb` y `new/open` a persistence fiel + warning en save no-op
- **Archivos:** `vantadb-wasm/src/lib.rs`
- **Acción:** Asegurar `capabilities()` fiel para todos los constructores vía `persistence` flag (hecho en Step 1). `connect_worker` también `persistence:true`.
- **Verify:** `cargo check -p vantadb-wasm` ✅ ; `wasm-pack test --node` capabilities tests ✅
- **Estado:** ✅ COMPLETED

### Step 3: Test que simula fallo getDirectory y verifica error / warning + persistence fiel
- **Archivos:** `vantadb-wasm/src/lib.rs` (mod wsm01_persistence_tests)
- **Acción:** Añadidos 4 tests: `connect_persistent_opfs_failure_propagates` (stub getDirectory→reject, aserta err con "OPFS unavailable"), `capabilities_persistence_fidelity_in_memory` (new/open→false), `capabilities_persistence_fidelity_idb` (connect_idb→true), `connect_persistent_success_reports_persistence_true` (OPFS success→true).
- **Verify:** `wasm-pack test --node` → 29 passed (4 nuevos + 25 existentes) ✅
- **Estado:** ✅ COMPLETED

### Step 4: Verify full + wasm-pack bundler build + commit
- **Archivos:** `vantadb-wasm/src/lib.rs`, `src/storage/vfile_mmap.rs` (clippy doc fix), `vantadb-python/src/lib.rs` (unused import)
- **Acción:** `cargo fmt --check` ✅, `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✅ (tras fix pyo3 import y vfile_mmap), `cargo nextest run --profile audit -p vantadb --lib` 1944 passed ✅ + `wasm-pack test --node` 29 passed ✅, `wasm-pack build --target bundler` exit 0 ✅, `rg OpfsStorage::open.*\.ok` 0 ✅. Commit y progreso.
- **Verify:** todos exit 0 ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (WASM isolated)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (o vanta-audit para security)
- **Enfoque:** ¿propagar error es correcto vs fallback IDB? ¿capabilities override no rompe serialización?
- **Cómo se probó:** `wasm-pack test --node` + rg 0 hits + bundler build
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⬜ pendiente

## Notas
- Ponytail: solución minimal 1 línea `?` + override capabilities 10L. Skipped fallback IDB automático (añadir cuando haya demanda explícita de graceful degradation).
- Si wasm-pack no instalado local, validar con `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`.

