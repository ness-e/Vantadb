# STABLE-05: Validar vantadb-wasm (gates 1-8)

## Metadata
- **Plan file:** docs/plans/2026-09-07-cleanup-gates.md (Task 1 — NO tocar)
- **Fuente:** docs/Backlog.md P47 fila STABLE-05 (:609)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** WASM binding (validación, 0 código esperado)
- **Turns estimados:** 8
- **Creado:** 2026-09-08
- **last-synced:** 2026-09-08
- **Estado:** ✅ COMPLETED
- **Cierre lead (SARL STRATEGY inline, 2026-09-08):** el sub-agente ejecutor dejó
  G1-G8 ✅ con evidencia y murió antes de devolver RESULTADO (2 aborts infra
  "model not available" + 1 "tool execution aborted"). Re-verificación lead:
  `cargo check -p vantadb-wasm --all-targets` exit 0, `cargo fmt` exit 0,
  `pkg/vantadb_wasm_bg.wasm` 1658051 bytes presente, `git status vantadb-wasm/`
  limpio, `cargo deny check` ok (warning no-fatal advisory-not-detected ya
  documentado en G8). Se acepta evidencia G5/G6 del file (outputs exactos +
  tiempos + log path; Chrome 152 headless 67 passed).
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 8 gates

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-ts` (wraps pkg/), `desktop/src/vanta-wasm-map.ts`, `desktop/src/vanta-wasm-map.test.ts`, `vantadb-python` migrate scripts |
| Callees | `vantadb` core (feature `wasm`), `wasm-bindgen`, `serde-wasm-bindgen`, `getrandom/wasm_js` |
| Implicaciones | validación read-only: no rompe contratos, no cambia comportamiento público, no afecta perf/mem/serialización, no requiere migración, no afecta tests existentes |

Cobertura índice: `vantadb-wasm/src/` indexado OK; `pkg/` + `e2e/pkg-nomodules/` excluidos por diseño (gitignore) — coverage `known_gaps` esperado.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-wasm/Cargo.toml` (54L), `docs/api/WASM_PERSISTENCE.md` (184L), `.opencode/rules/js-ecosystem.md` (34L), `vantadb-wasm/tests/wasm_tests.rs` (header 40L + macro config)
- **Archivos referenciados hacia dentro:** `vantadb-wasm/src/{lib,idb,opfs,worker}.rs`, `opfs_bridge.js`, `vantadb_wasm.d.ts` (vía codegraph_explore); `../` core con feature `wasm`
- **Archivos que referencian a los editados:** N/A — tarea de validación, 0 ediciones planificadas (doc touch solo si gate 7 lo exige)
- **Veredicto impacto:** bajo — comandos read-only + 1 build (artefacto gitignored, nunca commiteado)

## Contrato

`rustup target add wasm32-unknown-unknown` ok + `cargo check -p vantadb-wasm` 0 warnings + `wasm-pack build --target bundler` exit 0 + `cargo fmt --check` 0 + docs `WASM_PERSISTENCE.md` al día + `cargo deny check` 0

Gates operativos 1-8: 1=toolchain · 2=check · 3=clippy · 4=fmt · 5=build bundler · 6=test --node (scoping pre-mortem F1) · 7=docs al día · 8=deny.

## Spec (SDD — Phase 1b)

No es feature-add: el plan de solución no agrega ningún símbolo público (`pub fn`, tool, endpoint, método de binding). Validación mecánica de checklist. Sin decisiones técnicas abiertas → sin tabla de decisiones; justificación por evidencia: contrato mecánico del plan + fila Backlog P47 (:609).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `pkg/` gitignored nunca commiteado (R-2 js-ecosystem); no tocar plan file, ni archivos GOV-TK2/DOC-SYNC-01; branch `develop`; no versionar artefactos regenerados
- **Comandos de verificación:** gates 1-8 (ver Steps); cierre `cargo clippy -p vantadb-wasm --all-targets -- -D warnings`
- **Deuda pendiente:** ninguna prevista (si gate 6 se scopea → nota, no deuda)

## Recitation (canónico)

| Campo | Valor |
|-------|-------|
| activeGoal | STABLE-05 — Validar vantadb-wasm (gates 1-8) |
| lastAction | DISCOVERY completo + task file creado |
| result | PARTIAL |
| nextAction | Step Gate 1: `rustup target add wasm32-unknown-unknown` |
| contract | ver ## Contrato; verificacion: pendiente; evidencia: abajo; artefactos: docs/tasks/STABLE-05.md; invariantes/deuda: ver ## Invariantes |
| nextTask | ninguna (Wave0 cierra con STABLE-09 fuera del plan) |

## Deuda técnica (Regla 6 — MUST)

Saldo neto 0 — 0 código, 0 deuda nueva. Sin deuda.

## Definition of Done

- **Task:** 8 gates con evidencia de comando real (nunca auto-reporte)
- **Commit:** N/A por orden explícita (NO commitear) — gate 5 regenera `pkg/` gitignored, se deja en disco sin stage
- **Release:** N/A (validación, sin cambio publicable)

## Herramientas necesarias

- bash (rustup, cargo, wasm-pack, deny), codegraph_explore, campaign_verify_cmd

**Skills cargadas (SDP):** systematic-debugging (lifecycle VERIFY — builds/tests rotos) · source-driven-development (base WASM — verificar toolchain contra docs oficiales si hay duda) · ponytail full (siempre activo). Evaluadas-no-cargadas: campaign-executor/progreso (auto-vía-MCP, no manual) · browser-testing-with-devtools (solo si gate 6 llega a browser real) · performance-optimization/shipping-and-launch (keyword mapping, sin cambio de código que optimizar/lanzar).
`SDP: systematic-debugging, source-driven-development, ponytail-full (base-only+MCP: campaign-executor, progreso)`

## Investigation Notes

- Toolchain pre-verificado: `wasm32-unknown-unknown` instalado + `wasm-pack 0.15.0` + `cargo-deny 0.19.9` + `node v26.8.1`
- `git ls-files vantadb-wasm/pkg` vacío + `pkg/.gitignore: *` → rebuild seguro, nada que commitear
- `tests/wasm_tests.rs:13` `wasm_bindgen_test_configure!(run_in_browser)` + header "require a browser environment ... Use `wasm-pack test --chrome`" → gate 6 (`--node`) probablemente no-ejecutable; scoping con evidencia en su step
- Doc `WASM_PERSISTENCE.md` (rev 2026-08-19, nota CDN 2026-08-27): entry points `connect_persistent/connect_idb/connect_worker` + `save/load/save_idb/load_idb` coinciden con `src/vantadb_wasm.d.ts`; src persiste `db_state.json` + `graph_state.json` (CORE-02) — verificar mención en gate 7
- Working tree pre-existente ajeno: `M docs/Backlog.md`, `M opencode.jsonc`, `M .opencode` — NO tocar

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes de ejecución | 8 gates |
| % completado | 10% (discovery) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — no aplica: 0 cambios de código/dependencias; FFI/WASM sin modificar. Justificado.
- [x] **PERFORMANCE** — no aplica: 0 cambios en hot paths. Justificado.

## Steps

### Step G1: Toolchain (gate 1)
- **Archivos:** ninguno (comando)
- **Acción:** `rustup target add wasm32-unknown-unknown` + `wasm-pack --version`
- **Verify:** exit 0, target listado como installed
- **Resultado:** ✅ `info: component rust-std for target wasm32-unknown-unknown is up to date`, EXIT 0
- **Estado:** ✅ COMPLETED

### Step G2: cargo check (gate 2)
- **Archivos:** ninguno (comando)
- **Acción:** `cargo check -p vantadb-wasm --all-targets` (0 warnings)
- **Verify:** exit 0 + stderr sin warnings
- **Resultado:** ✅ `Finished dev profile in 22.82s`; re-chequeo: 0 líneas `warning`
- **Estado:** ✅ COMPLETED

### Step G3: clippy (gate 3)
- **Archivos:** ninguno (comando)
- **Acción:** `cargo clippy -p vantadb-wasm --all-targets -- -D warnings`
- **Verify:** exit 0
- **Resultado:** ✅ `Finished dev profile in 12.45s`, sin warnings (`-D warnings`)
- **Estado:** ✅ COMPLETED

### Step G4: fmt (gate 4)
- **Archivos:** ninguno (comando)
- **Acción:** `cargo fmt -p vantadb-wasm -- --check`
- **Verify:** exit 0
- **Resultado:** ✅ sin output, exit 0
- **Estado:** ✅ COMPLETED

### Step G5: wasm-pack build bundler (gate 5)
- **Archivos:** `vantadb-wasm/pkg/` (regenerado, gitignored)
- **Acción:** `wasm-pack build vantadb-wasm --target bundler` (timeout generoso, 1 corrida)
- **Verify:** exit 0 + `pkg/vantadb_wasm_bg.wasm` existe; `git status --short vantadb-wasm/` limpio de trackeados
- **Resultado:** ✅ `Done in 1m 06s`, `pkg/` regenerado (`vantadb_wasm_bg.wasm` 1.6MB, `.js`+`.d.ts` presentes); `git status --short vantadb-wasm/` vacío; `git ls-files vantadb-wasm/pkg` = 0 (nada que commitear)
- **Estado:** ✅ COMPLETED

### Step G6: test --node scoping (gate 6)
- **Archivos:** ninguno
- **Acción:** verificar ejecutabilidad: `wasm_tests.rs` es `run_in_browser` (evidencia :13) → chequear chrome disponible; si no hay browser/driver → scope-out con nota (pre-mortem F1, no inventar test)
- **Verify:** decisión documentada con evidencia (pass real o nota de scoping)
- **Resultado:** ✅ PASS REAL (mejor que scoping): `wasm-pack test --chrome --headless vantadb-wasm` EXIT 0 — `test result: ok. 67 passed; 0 failed` en 1.04s (Chrome 152 headless, chromedriver auto). Nota: primer intento con orden `wasm-pack test <path> --flags` falló por parseo de args (`Must specify at least one of --node/--chrome...`); orden correcto `wasm-pack test --chrome --headless <path>`. Log: `C:\Users\Eros\AppData\Local\Temp\opencode\stable05-g6.log`
- **Estado:** ✅ COMPLETED

### Step G7: docs al día (gate 7)
- **Archivos:** `docs/api/WASM_PERSISTENCE.md` (solo si hay drift)
- **Acción:** contrastar claims (backends, entry points, `db_state.json`/`graph_state.json`, E2E) contra `src/`; touch mínimo solo si drift real
- **Verify:** `git diff --stat` (0 archivos o solo el doc) + regla R-1 cumple
- **Resultado:** ✅ SIN DRIFT, 0 toques: 11/11 entry points del doc existen en `src/lib.rs` (`connect_persistent:585`, `connect_idb:618`, `connect_worker:662`, `save:944`, `save_idb:973`, `load_idb:1054`, `delete_idb:1075`, `load:1081`, `worker_read/write/delete:716/728/740`); `db_state.json`+`graph_state.json` confirmados en `lib.rs`; `WORKER_TIMEOUT_MS=5000` en `worker.rs:27`; `e2e/persist.html`+`e2e-persistence.mjs`+`opfs_bridge.js` existen. NOTICED BUT NOT TOUCHING: doc no menciona `graph_state.json` explícito (menor, no stale claim) + `WASM_STORAGE_REVIEW.md` stale ya declarado en el propio doc §Known gaps
- **Estado:** ✅ COMPLETED

### Step G8: cargo deny (gate 8)
- **Archivos:** ninguno (comando)
- **Acción:** `cargo deny check`
- **Verify:** exit 0
- **Resultado:** ✅ EXIT 0 (`advisories ok, bans ok, licenses ok, sources ok`); 1 warning no-fatal `advisory-not-detected` (RUSTSEC-2026-0253 ya no matchea — lru actualizado vía tantivy; remover entrada de `deny.toml:27` = tarea aparte, fuera de scope)
- **Estado:** ✅ COMPLETED

## Dependencias

- Ninguna — Wave0 independiente; no tocar archivos GOV-TK2/DOC-SYNC-01 ni plan file

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead (post-check inline — segundo contexto distinto del ejecutor, P2-01)
- **Enfoque:** ¿gates bien mapeados? ¿scoping gate 6 honesto? ¿sin salidas inventadas?
- **Cómo se probó:** re-corridos lead: check + fmt + artefacto pkg + deny; evidencia G5/G6 del file con outputs exactos
- **Checklist anti-hábitos tóxicos:** sin `continue-on-error`, sin tests ignorados, sin claims sin comando, sin artefactos commiteados
- **Veredicto:** ✅ APPROVE — G6 PASS REAL documentado (no scoping), G8 warning no-fatal declarado fuera de scope, 0 código tocado

## Notas

- Gate Justificación (plan): último paquete sin validar; cierra P47, desbloquea STABLE-09
- Stop condition: toolchain roto sin fix en 1h → BLOQUEADO con evidencia
- Cierre: verify full → NO commitear (orden explícita); RESULTADO siempre

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 8/8 gates (G1 toolchain, G2 check, G3 clippy, G4 fmt, G5 build 1m06s, G6 chrome 67/67, G7 docs sin drift, G8 deny ok)
PROXIMO_STEP: ninguno
COMMIT_HASH: (lead)
ARCHIVOS: docs/tasks/STABLE-05.md (único; pkg/ gitignored no se commitea)
VERIFY_CONTRATO: pasa (re-verificado lead: check/fmt/pkg/deny + evidencia G5/G6 del file)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no (colateral G8 advisory-not-detected anotado, fuera de scope)
SKILLS_CARGADAS: systematic-debugging, source-driven-development, ponytail-full (+ campaign-executor/progreso base)
```
