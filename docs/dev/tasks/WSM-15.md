---
title: "WSM-15: OPFS multi-pestaña — lock vía Web Locks API (hoy = corrupción silenciosa)"
kind: task
description: "OPFS no usa navigator.locks (IDB sí, idb.rs:62): pestañas/workers concurrentes sobre el mismo archivo pierden updates y clobbean el temp compartido. Fix: lock per-archivo vía Web Locks API en las mutaciones de OpfsStorage, fail-loud sin locks; repro multi-contexto browser (página+worker) + suite wasm verde."
---

# WSM-15: OPFS multi-pestaña: lock (hoy = corrupción silenciosa)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 9, F0)
- **Fuente:** validación externa v0.8.0 (2026-10-03) — plan Task 9 (Gate Justificación: pérdida/corrupción silenciosa en la superficie browser; el patrón de fix ya existe en el repo: IDB)
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Tipo:** WASM binding (Rust + tests browser wasm-bindgen)
- **Turns estimados:** 15-25
- **Creado:** 2026-10-04T03:24
- **last-synced:** 2026-10-04T05:05
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas (approach + disponibilidad de API resueltos en DISCOVERY con evidencia)
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-wasm/src/lib.rs` — `save()` (`:909`, `:921` write_file de `db_state.json`/`graph_state.json`), `load()` (`:1047`, `:1061` read_file, sin lock), `connect_worker` (`:648` worker_read); `vantadb-wasm/src/worker.rs` — `OpfsWorker::handle` (`:107,:123,:139,:155`) sobre el mismo `OpfsStorage`; `vantadb-wasm/tests/wasm_tests.rs` — 12 tests OPFS existentes + helper `try_opfs` |
| Callees | `navigator.locks.request` (Web Locks API — **nuevo**, vía `inline_js` espejo de `idb.rs:37-97`), OPFS API existente (`getDirectoryHandle`/`getFileHandle`/`createWritable`/`move`/`removeEntry`), `wasm_bindgen_futures::JsFuture`, `js_sys` |
| Implicaciones | Sin cambio de API pública: ningún `pub fn`/firma nueva (helpers privados + campo privado `name` en `OpfsStorage`; Gate D no dispara). Sin cambio de formato on-disk (mismo CRC-footer + temp+rename). Reads quedan lock-free (rename atómico → el lector ve archivo completo viejo o nuevo). Escrituras concurrentes al MISMO archivo se serializan; archivos distintos no contienden. `connect_worker` hereda el fix (el worker corre `OpfsStorage`). Entornos sin Web Locks (pre-Safari 15.4): las escrituras fallan loud con mensaje accionable (lecturas siguen). Bundle: +~1 KB (snippet inline, sin deps nuevas). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-wasm/src/opfs.rs` (498L), `vantadb-wasm/src/idb.rs` (244L), `vantadb-wasm/src/worker.rs` (400L), `vantadb-wasm/src/lib.rs` (secciones `:1-120`, `:380-1079`, `:1625-1674` — call sites OPFS + `OpfsStorage` usage), `vantadb-wasm/tests/wasm_tests.rs` (`:1-374` + grep de las 12 secciones OPFS/worker/CRC), `vantadb-wasm/Cargo.toml` (60L), `vantadb-wasm/README.md` (`:150-264`, §3 features/logging — destino del edit docs), `.opencode/rules/js-ecosystem.md`, `dev-tools/verify_changed.ps1`, `.opencode/references/clean-code-clean-architecture.md` (Apéndice V), `definition-of-done.md`
- **Archivos referenciados hacia dentro (imports/deps de los editados):** `opfs.rs` ← `js_sys` (Function/Promise/Reflect/Uint8Array), `wasm_bindgen`, `wasm_bindgen_futures::JsFuture`; `tests/wasm_tests.rs` ← `vantadb_wasm::{Client, IdbStorage, OpfsFile, OpfsStorage}` + `wasm_bindgen_test`; `Cargo.toml` ← workspace heredado (`version/edition/rust-version/lints`)
- **Archivos que referencian a los editados (referencias entrantes):** grep `OpfsStorage` = `lib.rs` (mod/pub use `:58-62`, campo `:404`, `connect_persistent :540`, tests internos `:2304`), `worker.rs` (`use :25`, campo `:73`, `open :85`), `tests/wasm_tests.rs` (12 tests). `OpfsFile` = `lib.rs` pub use + tests. CodeGraph: `write_file` (opfs) 2 callers (opfs.rs append + lib.rs save), sin covering tests pre-fix (los tests nuevos cierran ese gap)
- **Veredicto impacto:** **bajo-medio** — cambio localizado a la capa de storage OPFS (`opfs.rs`) + tests + nota README; sin cambio de firma pública, sin migración de datos, sin tocar el path de lectura de `lib.rs` (coordinación DX-01: no se edita `get`/memoria), sin tocar `opencode.jsonc`/plan file/`docs/pipeline-state.json`/WIP ajeno (FIND-233). El riesgo de regresión single-tab queda cubierto por los 12 tests OPFS existentes (misma suite).

## Contrato

"OPFS usa `navigator.locks` (lock **per-archivo**, espejo del patrón `idb.rs:62`) en todas las operaciones mutantes (`write_file`/`append_file`/`delete_file`) **o falla loud** con mensaje accionable si Web Locks no está disponible → repro multi-contexto (2 contextos: página + Worker dedicado, mismo origin) sin corrupción (el write espera el lock del otro contexto; appends concurrentes preservan TODOS los datos) → sin regresión single-tab (suite wasm completa verde, 12 tests OPFS existentes incluidos)."

Verificación exacta:
1. **RED→GREEN (browser, multi-contexto):** `cd vantadb-wasm && wasm-pack test --chrome --headless` — 3 tests nuevos fallan pre-fix (RED) y pasan post-fix (GREEN): `test_opfs_write_waits_for_worker_lock` (worker real sostiene el lock → write espera), `test_opfs_concurrent_appends_preserve_all_data` (2 contextos lógicos: 2 storages, lost update pre-fix), `test_opfs_write_fails_loud_without_web_locks` (shadow de `navigator.locks` → error accionable).
2. **Suite wasm completa verde:** mismo comando — 12 tests OPFS existentes + resto sin regresión.
3. **Compilación estática:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ + `cargo fmt --check` ✅ + `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` ✅.
4. **No-regresión feature `opfs`:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown --features opfs` ✅ (el worker usa el mismo storage).

## Spec (decisiones resueltas por evidencia — no feature-add: sin símbolos públicos nuevos)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Mecanismo de exclusión | A: Web Locks API (`navigator.locks`) vs B: BroadcastChannel mutex vs C: lock en memoria por instancia | A | ✅ **A** — patrón propio ya probado en el repo (`idb.rs:62`), origin-wide (cubre tabs + workers), soportado en Window y Worker (MDN, ver Investigation Notes). B requeriría protocolo propio (elección de líder, timeouts); C no cruza contextos |
| 2 | Granularidad | A: global (`vantadb-opfs-write`) vs B: per-archivo (`vantadb-opfs-write:{dir}:{path}`) | B | ✅ **B** — pre-mortem del plan (2): "lock global degrada concurrencia → lock por archivo/namespace (espejo del nombre en IDB)". Archivos distintos = tmp-files distintos = sin colisión; solo el mismo path contiende. Sin deadlock: nunca se anidan dos nombres |
| 3 | Fallback sin Web Locks | A: fail-loud (error accionable) vs B: warn + proceder vs C: fail en `connect_persistent` | A | ✅ **A** — contrato: "degradación explícita fail-loud". B reproduce la corrupción silenciosa que el task existe para eliminar; C rompería también lectura single-tab. Con A: mutaciones fallan loud, lecturas siguen; entorno sin locks = Safari 15.2-15.3 (OPFS sí, locks no — gap EOL), resto de browsers OPFS tienen locks (Chrome 69+/FF 96+/Safari 15.4+) |
| 4 | Ubicación del lock | A: `OpfsStorage::{write_file,append_file,delete_file}` (capa KV pública) vs B: `OpfsFile` (handle crudo) vs C: lib.rs `save()` | A | ✅ **A** — todos los caminos mutantes pasan por ahí (lib.rs `save`, worker.rs `handle`, tests); B no da atomicidad entre `write`+`move_to` de una misma operación lógica; C deja afuera `append_file`/`delete_file` y al worker |
| 5 | Implementación JS | A: `inline_js` en `opfs.rs` (espejo idb.rs) vs B: `opfs_bridge.js` externo | A | ✅ **A** — el worker (blob, sin imports) también ejecuta `OpfsStorage`: un bridge externo no está disponible ahí. idb.rs probó el patrón inline (BND-01: snippet DEBE exportar función) |
| 6 | ¿Lock también en reads? | A: no (rename atómico basta) vs B: sí | A | ✅ **A** — `write_file` publica vía temp+rename atómico: un lector concurrente ve el archivo completo (viejo o nuevo), nunca parcial. Espejo de IDB (lecturas sin lock). Menos contención |
| 7 | Gate D (question) | A: no dispara — sin símbolos públicos nuevos (`pub fn`/firma/binding) | A | ✅ **A** — helpers privados (`with_write_lock`, `acquire_web_lock`, `WebLockGuard`, `*_unlocked`) + campo privado `name`; `OpfsStorage` mantiene su superficie |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Formato on-disk intacto:** CRC-32 footer + temp-file + rename atómico + `QuotaExceededError` enriquecido — sin cambios (tests CRC existentes verdes).
  2. **Sin API pública nueva/rota:** mismas firmas `OpfsStorage::{open,write_file,read_file,delete_file,append_file,estimate_quota,check_quota_before_write,dir_handle,is_available}`; sin `pub fn` nuevo.
  3. **Reads lock-free** (rename atómico) — no agregar locks a `read_file`.
  4. **Sin deadlock:** jamás anidar dos locks distintos; `append_file`/`write_file`/`delete_file` adquieren UN nombre y llaman a los `*_unlocked` (nunca a la versión con lock).
  5. **Fail-loud:** cuando Web Locks no existe, la mutación DEBE devolver `Err` con mensaje accionable (nunca proceder en silencio).
  6. **No tocar:** path de lectura `get`/memoria de `lib.rs` (WIP DX-01), `opencode.jsonc`, plan file, `docs/pipeline-state.json`, `benchmarks/**` (WIP FIND-233).
- **Comandos de verificación:** `cd vantadb-wasm && wasm-pack test --chrome --headless` (suite browser completa verde) · `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` · `cargo fmt --check` · `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings`
- **Deuda pendiente:** ninguna nueva; el fix REDUCE deuda (P2-1 ya resuelto; este cierra el gap "no covering tests" de `write_file` reportado por CodeGraph).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (saldo ≤ 0). Reduce deuda: (a) elimina la clase de corrupción multi-contexto silenciosa; (b) agrega cobertura de tests al path `write_file`/`append_file` (CodeGraph: "no covering tests found"); (c) reusa patrón existente (`idb.rs` inline_js) sin dependencias nuevas.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ (lock per-archivo + fail-loud + 3 tests RED→GREEN + suite verde) + capa determinista (`cargo fmt`, `cargo check`/`clippy` wasm32) |
| **Commit** | Atómico, `fix(wasm): WSM-15 — ...`, verificación mecánica registrada (nunca auto-reporte), commit LOCAL (push diferido por owner) |
| **Release** | `fix:` → changelog patch vía release-plz en 0.9.0 (n/a en este commit); nota README del paquete |

## Herramientas necesarias
- Terminal: `wasm-pack 0.15.0` (local ✅), `cargo check/fmt/clippy` con `--target wasm32-unknown-unknown`, Chrome local ✅ (`C:\Program Files\Google\Chrome\Application\chrome.exe`)
- `codegraph_codegraph_explore` (blast radius ✅), `codebase-memory-mcp_check_index_coverage` (✅ no_recorded_issue; freshness metadata_changed → fuentes leídas directo), `campaign_verify_cmd` (contrato), `campaign_update_task_state` (recitation), `pwsh dev-tools/ocr-review.ps1` (cierre), `vanta-review` (P2-01 fork)
- Browser tests: `wasm-bindgen-test` (ya en dev-deps) + Worker blob (js_sys reflection) — sin deps nuevas

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` (base, auto) · `source-driven-development` (Web Locks validado contra MDN) · `test-driven-development` (pin: RED→GREEN) · `systematic-debugging` (pin: bug de corrupción) · `security-and-hardening` (pin: trust boundary/storage) · `incremental-implementation` (slices) · `context-engineering` · `rust-write-tests` (calidad de los tests browser) · `codebase-memory` (mantenimiento de grafos al usar CBM)

## Investigation Notes

- **Patrón de referencia (idb.rs:55-67):** `runWriteTx` envuelve la transacción en `navigator.locks.request("vantadb-write", () => new Promise(execute))`; si no hay locks → `execute` directo (degradación silenciosa — el bug que WSM-15 no repite). IDB usa un nombre GLOBAL; OPFS usará nombre per-archivo.
- **Web Locks API (validado 2026-10-04 contra MDN, ver fuentes):**
  - `navigator.locks.request(name, callback)` → callback corre al obtener el lock; **el lock se retiene hasta que la promesa retornada por el callback settle** (patrón "hold for arbitrary time": callback retorna una promise resuelta por el caller — MDN §Advanced use). Requests al mismo nombre (mismo contexto O otros tabs/workers) se encolan.
  - Disponible en **Window** (`Navigator.locks`) y **Web Workers** (`WorkerNavigator.locks`) — baseline "widely available". Requiere secure context.
  - Nombre no puede empezar con `-` (`NotSupportedError`).
  - Fuentes: <https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API> (última mod. 2025-04-03) · <https://developer.mozilla.org/en-US/docs/Web/API/LockManager/request> (última mod. 2025-11-03) — ambas verificadas por webfetch en esta sesión.
- **Entorno:** Node v26.8.1 local tiene `navigator.locks.request` (función) pero NO `navigator.storage` (sin OPFS) → los tests OPFS solo corren en browser (`wasm-pack test --chrome`). Chrome local presente.
- **Repro de corrupción (causa raíz, Phase 1):** dos contextos que escriben el mismo path interleavean: (a) ambos abren el MISMO temp `{path}.tmp` con `create:true` → el segundo `move_to` mueve un tmp clobbeado o falla; (b) `append_file` = read-modify-write sin exclusión → lost update (ambos leen base, el último write pisa al primero). `write_file` aislado es atómico (temp+rename), pero la colisión de tmp y el RMW de append no están protegidos. Sin lock, el CRC de `read_file` detecta el resultado corrupto *después* del daño — el dato ya se perdió.
- **Por qué no lock en `OpfsFile`:** `write` + `move_to` son DOS llamadas que deben ser atómicas juntas; el lock va en la operación lógica de `OpfsStorage`, no en el handle crudo.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — mecanismo (Web Locks), disponibilidad (Window+Worker, MDN), granularidad y fallback decididos con evidencia |
| Pendientes de ejecución (downhill) | 1 — Step 4 (review P2-01 + commit local) |
| % completado | 90% |

## Context Save Point

- **RED:** `$env:TEMP\wsm15-red.log` — `69 passed; 3 failed` (3 razones correctas: no espera / NoModificationAllowedError en temp compartido / no fail-loud).
- **GREEN:** `$env:TEMP\wsm15-green2.log` (72/72 default) + `$env:TEMP\wsm15-green-opfs.log` (77/77 con `--features opfs`).
- **Baseline:** `$env:TEMP\wsm15-baseline.log` (69/69 pre-cambio).
- **Gates:** `verify_changed.ps1` ALL 4 PASS; `check-docs` all clear; `check-links` exit 0; `markdownlint-cli2` 0 issues; `gen-index --check` exit 0 (regenerado).
- **Pendiente:** veredicto `vanta-review` + commit local `fix(wasm): WSM-15`.

## Fase 1 — Evidencia de Debugging (GATE — bug)

- **Repro:** multi-contexto en browser (Chrome). Método canónico: `wasm-pack test --chrome --headless` con 3 tests nuevos: (a) worker dedicado sostiene el lock `vantadb-opfs-write:{dir}:{path}` vía `navigator.locks.request` y la página llama `write_file` → pre-fix completa de inmediato (sin esperar), post-fix espera; (b) dos `append_file` concurrentes (2 storages mismo dir) → pre-fix lost update (un solo chunk sobrevive), post-fix ambos; (c) `navigator.locks` shadowed a `undefined` → pre-fix escribe igual (peligro), post-fix `Err` accionable. Escenario real: dos pestañas con `connect_persistent("mydb")` que guardan a la vez (mismo origin, mismo `db_state.json`).
- **Hipótesis:** `OpfsStorage` no adquiere ningún lock cross-context (a diferencia de `IdbStorage`); el temp-file compartido + RMW de append interleavan entre contextos → pérdida/corrupción silenciosa.
- **1 variable controlada:** la adquisición del Web Lock per-archivo alrededor de las mutaciones de `OpfsStorage`. Nada más: sin cambio de formato, sin cambio de API, sin tocar reads.
- **Test RED:** los 3 tests nuevos en `vantadb-wasm/tests/wasm_tests.rs` — se ejecutan ANTES del fix y FALLAN (a: write no espera; b: lost update; c: write no falla loud). Post-fix → GREEN. Evidencia capturada en el log de `wasm-pack test`.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY — aplica (pin SDP v3: storage/trust boundary):** previene **pérdida de datos del usuario** (integridad, STRIDE-T) por escrituras concurrentes interleaveadas; sin input nuevo de usuario, sin red, sin secretos; el bridge JS no usa `eval` ni construye strings ejecutables (nombre de lock = `format!` de dir+path del caller, pasado como string a una API nativa). Fail-loud en vez de degradación silenciosa (safe default). Sin deps nuevas (supply chain sin cambio). Hallazgo: sin Critical/High.
- [x] **PERFORMANCE — aplica acotado (no hot path de engine):** los locks solo tocan la capa de persistencia browser (I/O ya lento por definición); costo = 1 request de lock por mutación (~µs, mismo orden que el propio I/O OPFS). Archivos distintos no contienden (per-archivo, decisión #2). Sin benchmark de motor aplicable (no toca HNSW/search/serialización); medición indirecta: la suite browser completa debe seguir verde en tiempo razonable.

## Steps

### Step 1: RED — tests browser multi-contexto (contrato)
- **Archivos:** `vantadb-wasm/tests/wasm_tests.rs`
- **Acción:** agregar 3 tests + helpers: `test_opfs_write_waits_for_worker_lock` (Worker blob real sostiene el lock; mide que `write_file` espera), `test_opfs_concurrent_appends_preserve_all_data` (2 storages, `future_to_promise` + `Promise::all`), `test_opfs_write_fails_loud_without_web_locks` (defineProperty shadow + restore antes del assert). Ejecutar contra el código actual → los 3 FALLAN.
- **Verify:** `cd vantadb-wasm && wasm-pack test --chrome --headless` → 3 failed (RED) + 69 existentes verdes
- **Estado:** ✅ COMPLETED — RED verificado 2026-10-04T04:0x: `69 passed; 3 failed` con las 3 razones correctas: (a) write no espera (elapsed 13ms < 300ms), (b) `NoModificationAllowedError: Failed to move concurrent.bin.tmp. A FileSystemHandle cannot be moved while it is locked.` (colisión real del temp-file compartido), (c) write NO falla loud con locks shadowed (escribió igual). Evidencia: `$env:TEMP\wsm15-red.log`

### Step 2: GREEN — lock per-archivo en `OpfsStorage` (inline_js espejo idb.rs)
- **Archivos:** `vantadb-wasm/src/opfs.rs`
- **Acción:** snippet `inline_js` (`vantaOpfsAcquireLock` → `Promise<release|null>` con el patrón hold-promise de MDN); `WebLockGuard` (Drop llama release); `with_write_lock` (fail-loud si `None`); split `write_file`/`append_file`/`delete_file` → `*_unlocked`; campo privado `name` en `OpfsStorage`. Sin símbolos públicos nuevos.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` + `cargo fmt --check` + los 3 tests pasan (GREEN) + 69 existentes verdes
- **Estado:** ✅ COMPLETED — GREEN verificado 2026-10-04: `72 passed; 0 failed` (default) y `77 passed; 0 failed` (`--features opfs`, incluye 5 worker tests). `cargo check` nativo + wasm32 + `--features opfs` ✅; `cargo fmt --check` ✅. Evidencia: `$env:TEMP\wsm15-green2.log`, `$env:TEMP\wsm15-green-opfs.log`. Nota clippy: `-D warnings` en wasm32 falla SOLO por warning pre-existente del core (`src/index/serialize/file.rs:146`, `drop_non_drop`) — ya ruteado como **FIND-241** (creado por FIND-238); el crate wasm no emite warnings.

### Step 3: Docs — nota multi-tab en README del paquete
- **Archivos:** `vantadb-wasm/README.md` (§3, subsección nueva tras Console logging), `vantadb-wasm/demo/README.md` (requirements: Safari 15.4+ por Web Locks)
- **Acción:** documentar: lock per-archivo vía Web Locks en mutaciones; reads lock-free; fail-loud sin Web Locks (usar `connect_idb`); scope (mismo origin/dir/path).
- **Verify:** markdownlint + check-links/check-docs
- **Estado:** ✅ COMPLETED — `markdownlint-cli2` 0 issues (3 files); `check-docs` all clear; `check-links` within budget (exit 0); `gen-index --write` regenerado (docs/index.md + llms.txt, incluye task files siblings pendientes de indexar) → `--check` exit 0.

### Step 4: Verify full + cierre (OCR + review P2-01 + commit LOCAL)
- **Archivos:** task file + commit
- **Acción:** suite browser completa; `cargo check`/`clippy` wasm32 (default + `--features opfs`); `dev-tools/verify_changed.ps1`; OCR delegation; fork `vanta-review`; commit `fix(wasm): WSM-15 — ...` (LOCAL, sin push)
- **Verify:** todos los gates verdes + veredicto review registrado
- **Estado:** ✅ COMPLETED — verify_changed ALL 4 PASS; docs gates verdes (check-docs/check-links/gen-index/markdownlint); OCR delegation sin Critical/High; review P2-01 ✅ approve; commit local `265abe6a` (hook fmt+clippy+actionlint verde; 8 archivos).

## Dependencias
- F0 — sin dependencias. `nextTask: DUR-03` (plan). Coordinación: NO tocar path `get`/memoria de `lib.rs` (DX-01) ni `benchmarks/**` (FIND-233) ni `opencode.jsonc`/plan file (WIP ajeno).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` fork (sesión `ses_efa1f8a53ffeShYDTcWV7CEETT`, contexto fresco — confirmado reviewer_context ≠ author_context)
- **Enfoque:** Web Locks per-archivo con hold-promise + `WebLockGuard` RAII + fail-loud; alternativas evaluadas y descartadas con razón (lock global estilo IDB, BroadcastChannel mutex, lock en `OpfsFile`). Deadlock/leak del guard analizado (incl. cancelación del future y liberación al morir el agente).
- **Cómo se probó:** el revisor **re-ejecutó la suite por su cuenta** (`wasm-pack test --chrome --headless` → 72/72; `--features opfs` → 77/77) y **reprodujo el RED en un worktree aislado** contra HEAD (69/3 con las 3 causas exactas: elapsed 12ms<300ms, `NoModificationAllowedError` en el temp compartido, `expect_err` fallido). Verificó `cargo fmt/check` (default y `--features opfs`), el claim de clippy (1 warning pre-existente del core = FIND-241; **cero** en `vantadb-wasm`) y el mapeo completo de call sites (`lib.rs save`, `worker.rs handle`, tests). Logs propios del revisor: `%TEMP%\wsm15-review-{green,opfs,red,clippy}.log`.
- **Checklist anti-hábitos tóxicos:** ✅ sin salidas inventadas (logs propios + worktree aislado); ✅ sin saltos de clarificación; ✅ contrato verificado contra acceptance criteria; ✅ sin fallos ignorados; ✅ sin supuestos presentados como evidencia.
- **Hallazgos del revisor:** Critical: ninguno · Required: ninguno · Optional: (1) `delete_file_unlocked` sin await de `removeEntry` — **pre-existente**, ruteado a **FIND-242** (con repro determinista pendiente); (2) índice regenerado incluye task files siblings untracked (DX-01/DUR-03) — incluido en el commit con nota (convención de commits recientes + DoD docs); (3) cobertura worker-side indirecta — pre-existente. Nits (revokeObjectURL, wording README, colisión de nombres) — aplicados los accionables (wording + terminate-on-error), resto descartado.
- **Veredicto:** ✅ **approve** — contrato completo, verificación independiente real, sin cambios requeridos. Deltas post-review (wording README + higiene del helper de test, sin cambio de código de producción) re-verificados con suite 72/72.

## Notas
- Coordinación multi-worker: `git status` pre-tarea muestra WIP ajeno (`opencode.jsonc` M, plan file M, `vantadb-ts/repro-dx01.mjs` untracked) — NO se tocan; el commit de esta tarea agrega SOLO sus archivos.
- Stop condition del plan (>2d): no alcanzada — el fix es implementable con el patrón existente; no se requiere la degradación alternativa (FIND del lock real).
- Decisión de fallback documentada (Spec #3): fail-loud en mutaciones, reads siguen; entorno afectado = Safari 15.2-15.3 únicamente (gap EOL; OPFS+WebLocks co-extensivos en todo browser vigente).
- **FIND-242** (nuevo, del review P2-01): `delete_file_unlocked` no espera `removeEntry` (pre-existente) — no se arregla inline (necesita repro determinista propio; el reviewer lo marcó follow-up).
- **Índice generado:** `docs/index.md` + `llms.txt` regenerados; incluyen entradas de `DX-01.md`/`DUR-03.md` (task files siblings untracked de otros workers — no commiteados por esta tarea; el índice quedará consistente cuando ellos commiteen). Convención verificada: FIND-233/235/239 también incluyeron el índice en su commit.
- **NOTICED BUT NOT TOUCHING:** `vantadb-wasm/demo/README.md` actualizado solo en la línea de requirements (Safari 15.4+); el resto del demo sin tocar. `OpfsFile` (handle crudo) sigue sin lock por diseño (el lock cubre la operación lógica de `OpfsStorage`, no el handle).

## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4
PROXIMO_STEP: ninguno (nextTask del plan: DUR-03)
COMMIT_HASH: 265abe6a — fix(wasm): WSM-15 - OPFS lock per-archivo via Web Locks API + fail-loud sin locks (multi-tab sin corrupcion)
ARCHIVOS: vantadb-wasm/src/opfs.rs · vantadb-wasm/tests/wasm_tests.rs · vantadb-wasm/README.md · vantadb-wasm/demo/README.md · docs/dev/tasks/WSM-15.md · docs/dev/Backlog.md · docs/index.md · llms.txt
VERIFY_CONTRATO: pasa — `cd vantadb-wasm && wasm-pack test --chrome --headless` → 72/72 (default) y 77/77 (`--features opfs`); RED pre-fix 69/3 con causas correctas; re-ejecutado independientemente por vanta-review
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P/D/V/C no dispararon: sin ambigüedad de contrato, sin blast radius >10 archivos/hot path/API nueva (Spec #7), verify sin fallas repetidas, sin colaterales que requieran decisión del usuario (FIND-242 ruteado como ticket)
SKILLS_CARGADAS: campaign-executor · progreso (base, auto) · source-driven-development · test-driven-development · systematic-debugging · security-and-hardening · incremental-implementation · context-engineering · rust-write-tests · documentation-skill
```

