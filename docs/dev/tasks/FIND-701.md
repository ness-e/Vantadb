---
title: "FIND-701 + FIND-702: el toolchain y el gate de composite actions valen lo que dice CI"
kind: task
status: completed
description: "Cerrar FIND-701 (el input toolchain era decorativo) y FIND-702 (ningun gate lintea composite actions) con la pasada real de CI sobre 8d355e2df, y dar veredicto al rojo cronico de TSan con evidencia de log"
---

# FIND-701 + FIND-702: el toolchain y el gate de composite actions valen lo que dice CI

## Metadata
- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — F0.1 (§4, Fase 0)
- **Fuente:** filas `FIND-701` y `FIND-702` de `docs/dev/Backlog.md` (origen: gates pre-release 2026-11-23)
- **Esfuerzo:** 🟢 2h (diagnóstico pasivo; sin código de producto)
- **Prioridad:** 🟠 Alta (ambas bloquean el release por ser gates falsos, no por romper nada)
- **Tipo:** CI/CD / DevOps — diagnóstico + cierre administrativo
- **Turns estimados:** 8
- **Creado:** 2026-11-24T15:00
- **last-synced:** 2026-11-24T15:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 (`Tests (Windows)` cerró ✅ success)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/ci-rust.yml` es **required** en `develop`: sus jobs son el gate de merge de todo el repo. `./.github/actions/rust-setup` tiene ~70 call sites en 11 workflows (FIND-702) |
| Callees | `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `sccache-action`, `taiki-e/install-action`; `cargo nextest`, `actionlint v1.7.7` |
| Implicaciones | **Ninguna sobre código.** El entregable es veredicto + cierre de dos filas de Backlog. No se toca `src/**`, ni `Cargo.toml`, ni el workflow |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/actions/rust-setup/action.yml` (136L), `.github/workflows/ci-rust.yml` (jobs `msrv` 467-482, `release-combo` 633-665, `sanitizer-tsan` 932-1024), `.github/workflows/docs-gates.yml` (job `actionlint` 611-655), `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`, `docs/dev/operations/CI_POLICY.md` (§taxonomía `continue-on-error`, líneas 56-92), `docs/dev/Backlog.md` (filas 704-706)
- **Referencias hacia dentro:** los 11 workflows que consumen `rust-setup`; `docs-gates.yml:611` es el único job que corre `actionlint`
- **Referencias entrantes a los editados:** ninguna — no se edita ningún workflow (ver §Notas: el fix de TSan queda como opción medida, no aplicada)
- **Veredicto impacto:** nulo en código; **alto** en el gate de merge si se hubiera tocado `ci-rust.yml`. Por eso no se toca

## Contrato

```
gh run view 45456649424 --json jobs    # MSRV Check (1.95.0) = success
gh run view 45461424443 --json jobs    # actionlint (workflows lint + inventory gate) = success
```

Cumplido, más las dos verificaciones que impiden cerrar por apariencia:

```
# FIND-701: que el step que exporta RUSTUP_TOOLCHAIN haya corrido de verdad
gh api repos/ness-e/VantaDB/actions/jobs/138667791016/logs   # -> rustc 1.95.0 (59807616e) + Finished
# FIND-702: que el run del job actionlint sea DESCENDIENTE del commit que añadió el guard
git merge-base --is-ancestor a78e36ed2 ee41ec51b             # -> exit 0
```

## Veredicto

### (1) `CI: Rust` sobre `8d355e2df` — run `45456649424`

**Run concluido `success`.** 27 jobs: **19 `success`**, **4 `failure`** (TSan, no bloqueantes), **4 `skipped`**. Los 19 en verde incluyen **todos los que gatean menos uno — ver el aviso de ruleset abajo**: `MSRV Check (1.95.0)`, `Clippy Lints`, `Format Check`, `Security Audit`, `OSV-Scanner`, `Unused Deps`, `Dependency Policy Check`, `Secret Scan`, `Minimal Versions`, `Tests (Linux)`, `Tests (macOS)`, **`Tests (Windows)`**, `ASan`, `Miri`, `Code Coverage`, `Tests (embeddings features)` ×2, `Experimental Crates`, `Release Combo`. Skipped por sus propias condiciones: `ADR Gate` (`event_name == pull_request`), `Public API Snapshot` y `Semver Checks` (`if ref == main`), `WASM` (`paths:` de wasm).

**Rojo:** las 4 particiones de `TSan` — **no bloqueante por diseño** (`continue-on-error: true # CATEGORY: NON-CRITICAL`, `ci-rust.yml:952`; catalogado como tal en `CI_POLICY.md:65`). No entra en required checks: el run concluyó `success` con esos 4 jobs en rojo.

> ⚠️ **Aviso que la pasada de CI no puede ver y que sí bloquea el release:** los rulesets `develop` (`23692587`) y `main` (`16844590`) siguen exigiendo el context **`MSRV Check (1.94.1)`**, nombre que ningún job reporta desde `5a669a186`. El rename de FIND-705 tocó el workflow pero no el ruleset. Por eso **este run es verde y el gate requerido sigue insatisfacible**: registrado como `FIND-715`. No se corrige acá (escribir rulesets está fuera del alcance de la tarea y es acción del owner).


### (2) Causa raíz de TSan — infraestructura del harness, no una race de VantaDB

Los 4 logs (jobs `138675656963` / `138675657055` / `138675657053` / `138675657050`) son **la misma firma**, sin una sola diferencia material:

```
WARNING: ThreadSanitizer: data race (pid=18274)
  Read of size 8 ... by main thread:
    #0 memcpy
    #1 core::profiling::compiler_copy::<MaybeUninit<test::event::CompletedTest>, 304>
    #4 <std::sync::mpmc::list::Channel<test::event::CompletedTest>>::read   library/std/src/sync/mpmc/list.rs:381
  Previous write of size 8 ... by thread T1:
    #4 <std::sync::mpmc::list::Channel<test::event::CompletedTest>>::write  library/std/src/sync/mpmc/list.rs:274
SUMMARY: ThreadSanitizer: data race (…) in memcpy
```

Cuatro hechos que cierran el diagnóstico:

1. **Cero frames de `vantadb`.** Los dos accesos en conflicto son a memoria del harness: `std::sync::mpmc::list::Channel<test::event::CompletedTest>` (`read` en `list.rs:381`, `write` en `list.rs:274`) vía `core::profiling::compiler_copy`. La memoria en disputa es un `Block<CompletedTest>` del canal interno de `libtest`, no un campo del motor. El hilo T1 es el worker que ejecuta un test del crate (`api::scores::*`), pero lo que se pisa es la estructura del harness que transporta el resultado.
2. **El test no falló; el proceso sí.** Cada proceso imprime `test result: ok. 1 passed; 0 failed` y a continuación muere con **exit code 66** (el `exitcode` por defecto de TSan cuando hay un reporte). El reporte se cuelga del canal de *eventos de test completado* del harness, no de una aserción: **no hay fallo de test que triagear**. (Precisión: el orden `test result: ok` → `WARNING` es lo que muestra el log; el interleaving de stdout/stderr en CI no es un timestamp, así que "ocurre en el teardown" sería una inferencia, no un hecho medido — y no la necesitamos para el veredicto.)
3. **No son 4 races distintas: son 4 tests concurrentes del mismo binario.** `Starting 821 tests across 60 binaries` y luego `4/821 tests run: 0 passed, 4 failed, 2322 skipped` en **0,60–0,79 s**. Los 4 procesos que arrancaron en paralelo imprimen todos `running 1 test` + `2763 filtered out`, es decir: los 4 son del binario `--lib` (2763 tests), no de binarios distintos. nextest es fail-fast, así que los 4 fallan a la vez y el run aborta sin llegar a los otros 59 binarios. La lane no correría ni el 0,5 % de la suite.
4. **No es regresión de `8d355e2df`, y no es el nightly.** **Misma clase y los mismos cuatro tests** en:
   - run `45109855351` (sha `1650ec5b1`, nightly **`137d1d494`** del 11-22)
   - run `45456649424` (sha `8d355e2df`, nightly **`20fe935d5`** del 11-23)
   - ya rojo en run `44853213235` (sha `d3ff572e5`, 11-21) — el run figuraba `success` solo porque el job es `continue-on-error`.

   Los cuatro tests son los mismos (`api::scores::tests::cosine_distance_similarity_roundtrip`, `accumulator::tests::test_accumulator_basic`, `accumulator::tests::test_accumulator_send_sync`, `api::scores::tests::rrf_contribution_none_zero`) y el `SUMMARY … in memcpy` es el mismo. **Los conteos NO son idénticos** y no hace falta que lo sean: `59 vs 60` binarios, `2319 vs 2322` skipped y `2761 vs 2763` filtered-out reflejan dos commits distintos. El nightly flotante **cambió** entre los dos runs y la clase no cambió ⇒ la hipótesis "el nightly rompió TSan" queda refutada con evidencia, no por intuición.

**Causa raíz: la clase documentada, no una medición propia.** El mecanismo (race sobre un `Block<CompletedTest>` publicado por atómicos/fences que TSan no modela, con `std` sin instrumentar al no haber `-Zbuild-std`) es la clase que FIND-227 ya cerró y que `ci-rust.yml:953-960` documenta con `rust-lang/rust#39608`, `#65097` y `rayon-rs/rayon#812`. **Esta tarea no lo midió**: verificó que *todo* el stack del reporte es del harness y que la clase es estable entre commits y nightlies. Se cita como clase heredada, no como hallazgo propio — y explícitamente **sin** apoyarse en el comentario vecino de FIND-650, que esta misma tarea refuta.

### (3) Hallazgo propio (no arreglado, por scope): la lane es ciega

El ruido ya está diagnosticado en FIND-227; lo que **no** cubre es esto otro. La premisa de FIND-650 — *"nextest runs each test in its own process, which removes the dominant libtest-harness class"* (`ci-rust.yml:961`) — **es falsa para `--lib --tests --bins`**. nextest ejecuta el binario de `libtest` en modo `--exact`, y `libtest` sigue construyendo su `mpmc::Channel<CompletedTest>` para mover el evento al hilo principal. La clase sobrevive en **cada** proceso, así que con fail-fast el run muere en <1 s habiendo probado **4 de 821 tests**: el rojo no significa "hay una race en el motor", y un eventual verde tampoco significaría "el motor está limpio".

Resultado neto de la lane hoy: **cero señal y rojo permanente**, sobre 821 tests y 60 binarios que nunca se ejercen. Es el mismo género de defecto que FIND-701/702 (un gate que parece medir y no mide), y en un motor con paths multi-índice es deuda de la **Regla 8** por sí sola. No se toca acá porque ninguna de las salidas es verificable sin round-trip de CI (y el push no está autorizado):

- **Aislar el ruido del harness y exigir que la lane mida** — `tsan.supp` acotado a `compiler_copy`/`mpmc` (rechazado en FIND-227; reabrirlo exige re-medir la tasa de falsos negativos) **más** un contador de *tests ejecutados* por partición, no solo el exit code. `--no-fail-fast` por sí solo **no** sirve como opción: con el ruido en todos los procesos solo convierte "4 rojos" en "~205 rojos lentos", no crea señal.
- **`-Zbuild-std`** ataca la raíz real (instrumentar `std`) pero reconstruye `std` completo bajo TSan dentro del mismo budget de 50 min.
- **Retirar el job** con decisión explícita en `CI_POLICY.md` + `# CATEGORY:` — nunca con `continue-on-error` para destrabar (Regla 2).

Detalle y contrato: fila `FIND-714`.


Ninguna de las tres es verificable en la ventana sin push (Regla 1 y Regla 2: no se cambia un gate sin pasada real). Queda registrado como `FIND-714` con repro exacto y contrato.

### (4) `Tests (Windows)` — job `138675656820`

**✅ `success`**, 56 min de 90 de `timeout-minutes` (arrancó 18:22:47Z). Con eso el run `45456649424` cerró **`completed / success`**: 19 `success`, 4 `skipped`, 4 `failure` (TSan, no bloqueantes). Es un required check en ambos rulesets, así que su veredicto **sí** condicionaba el de (1) mientras estaba corriendo; ya no.

## Evidencia de cierre por fila

| Fila | Gate exigido | Evidencia real |
|---|---|---|
| `FIND-701` | `MSRV Check (1.95.0)` verde **y** el toolchain declarado siendo el que compila | `MSRV Check (1.95.0)` = **success** (job `138667791016`, run `45456649424` @ `8d355e2df`). Log del job: `rustc 1.95.0 (59807616e 2026-04-14)`, el step del composite ejecutó `echo "RUSTUP_TOOLCHAIN=1.95.0" >> "$GITHUB_ENV"`, todos los steps siguientes llevan `RUSTUP_TOOLCHAIN: 1.95.0`, y `cargo check --locked` → `Finished dev profile in 1m 16s`. **El job mide lo que declara.** Aclaración de alcance: el contrato pedía "los jobs de release"; lo verificado en *este run* es `Release Combo (server + allocator, -D warnings)` = **success** (job `138675656932`), que es un job de CI (`ci-rust.yml:646`) que usa el composite **sin** `toolchain:` → default `1.95.0` (`action.yml:7`). Los workflows de release no corren en este push (van por tag/dispatch), así que de ellos solo se verificó el estado estático: `release.yml` (`1.95.0` ×2), `release-adapters.yml` (`1.95.0` + `RUSTUP_TOOLCHAIN`), `release-npm-node.yml` (`1.95.0`). Fix `ea3200cda` es ancestro de `8d355e2df` (`git merge-base --is-ancestor`). **Caveat fuera del job:** el context *requerido* en los rulesets sigue llamándose `MSRV Check (1.94.1)` → `FIND-715`. |
| `FIND-702` | job `actionlint` de `GATES: Docs` verde, y ese run descendiente del commit del guard | `actionlint (workflows lint + inventory gate)` = **success** en run `45461424443` @ `ee41ec51b`, con el step del guard también en `success`. `git merge-base --is-ancestor a78e36ed2 ee41ec51b` → exit 0, así que el run incluye el guard nuevo (`docs-gates.yml:634-648`: compara conteos `run:` vs `shell:` por `.github/actions/*/action.yml` y falla con `::error`). **Alcance honesto del gate:** es una comparación de conteos (el propio código lo marca `# ponytail: … upgrade if a false negative ever appears`), así que un fichero con un `shell:` de más y un `run:` sin `shell:` queda balanceado; y el glob cubre solo `action.yml`, no `action.yaml` ni subdirectorios anidados. Ese residual está registrado en `FIND-704` y **se queda ahí**: no se pierde con el cierre. Lo cerrado es el bug reportado (el gate no existía) más su pasada real de CI — no "toda la convención de composite actions queda validada". |

**Lo que NO se aceptó como evidencia** (auto-engaño descartado):
- Cerrar `FIND-701` mirando solo el nombre `MSRV Check (1.95.0)` en verde. El bug era justamente que el nombre decía una cosa y el compilador hacía otra → hizo falta el `rustc --version` + `RUSTUP_TOOLCHAIN` del log.
- Cerrar `FIND-702` por "el job existe y pasó en algún run". Un job verde en un run anterior al guard no prueba nada → hizo falta el `merge-base`.
- Marcar TSan como "flaky" sin leer los stacks. Pocas cosas más caras que descartar una race real por costumbre: por eso se leyó el stack completo de las 4 particiones y se comparó contra un nightly distinto.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Veredicto de `CI: Rust` escrito con evidencia por job · causa raíz de TSan con log citado · veredicto de `Tests (Windows)` (✅ success) · review P2-01 con veredicto registrado |
| **Commit** | atómico: task file + 2 filas retiradas de `Backlog.md` + altas de `FIND-714` y `FIND-715` + entrada en `avance/activo/ci-cd.md`; conventional `docs(ci):` |
| **Release** | `CI: Rust` no es el bloqueo (run verde). TSan tampoco (non-blocking por política, pero ciego → `FIND-714`). **Sí es bloqueo el ruleset** con el context `MSRV Check (1.94.1)` → `FIND-715` |

## Herramientas necesarias
- `gh` (lectura: `gh run view` / `gh api …/jobs/<id>/logs`)
- `git merge-base --is-ancestor`

**Skills cargadas (SDP v3, phase=BUILD, `ci-cd-and-automation` + `git-workflow-and-versioning` pinned):** `ci-cd-and-automation` (gates, feedback de fallos de CI, taxonomía de no-bloqueantes) · `systematic-debugging` (Iron Law: causa raíz antes de cualquier fix; fases 1-3 ejecutadas completas) · `doubt-driven-development` (verificación adversarial del diagnóstico) · `git-workflow-and-versioning` (conventional commit `docs(ci):`) · base auto-cargada: `campaign-executor`, `progreso`, `ponytail`

## Fase 1 — Evidencia de Debugging (TSan)

- **Repro:** el fallo es 100 % determinista y no necesita repro local: ocurre en las 4 particiones de cada run del workflow. Comando de lectura: `gh api repos/ness-e/VantaDB/actions/jobs/<id>/logs --allow-escape-sequences | Select-String "SUMMARY: ThreadSanitizer|tests run:"`.
- **Hipótesis (a) — nightly flotante cambió:** **refutada.** nightly `137d1d494` (run `45109855351`) y `20fe935d5` (run `45456649424`) producen la **misma clase** (`SUMMARY … in memcpy`, frames `mpmc::Channel<CompletedTest>` + `compiler_copy`) sobre los **mismos cuatro tests**. Los conteos difieren (`59 vs 60` binarios, `2761 vs 2763` filtered-out) porque son commits distintos; la clase no cambió.
- **Hipótesis (b) — el job pide features que TSan no soporta:** **no aplica.** Las features del job son `cli,arrow,tls,opentelemetry`; no hay SIMD/atómicas exóticas involucradas, y el stack ni siquiera desciende al código del crate.
- **Hipótesis (c) — race real en `src/`:** **refutada por inspección del stack.** 0 frames `vantadb::`; el reporte es interno a `libtest` (`mpmc::Channel<CompletedTest>`), un patrón que ya es ruido conocido desde FIND-227.
- **Hipótesis (d) — OOM/timeout:** **refutada.** Las 4 particiones terminan en **0,60–0,79 s** de un budget de 50 min; exit 66 (TSan), no 137 (OOM) ni 124 (timeout).
- **1 variable controlada:** el nightly es la única variable que efectivamente cambió entre los dos runs comparados, y la salida no cambió con ella.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% (todo lo accionable, cerrado; `FIND-714`/`FIND-715` nacen como filas, no como deuda de esta tarea) |

## Steps

### Step 1: Veredicto de `CI: Rust` por job
- **Archivos:** — (lectura)
- **Acción:** `gh run view 45456649424 --json jobs`; clasificar success / failure / in_progress / skipped
- **Verify:** 18 jobs mandatory en `success`; 4 TSan en `failure`; 1 en `in_progress`
- **Estado:** ✅

### Step 2: Causa raíz de las 4 particiones de TSan
- **Archivos:** `.github/workflows/ci-rust.yml:932-1024`
- **Acción:** bajar el log de cada job vía `gh api …/logs` y leer el stack completo + el resumen de nextest
- **Verify:** 4/4 logs con `SUMMARY: … in memcpy`, frames `std::sync::mpmc` + `core::profiling::compiler_copy`, `test result: ok` seguido de exit 66
- **Estado:** ✅

### Step 3: Diferencial contra runs anteriores (¿regresión o crónico?)
- **Archivos:** — (lectura)
- **Acción:** comparar `gh run list --workflow ci-rust.yml` + los jobs TSan de `45109855351`, `44853213235`
- **Verify:** misma firma en `<nightly 137d1d494>` y `<nightly 20fe935d5>`; ya rojo antes de `8d355e2df`
- **Estado:** ✅

### Step 4: Gate de `FIND-701` contra el log real (no el nombre del job)
- **Archivos:** `.github/actions/rust-setup/action.yml:73-92`
- **Acción:** leer el log del job `138667791016` buscando `rustc`, `RUSTUP_TOOLCHAIN`, y el `cargo check`
- **Verify:** `rustc 1.95.0 (59807616e 2026-04-14)` + `RUSTUP_TOOLCHAIN: 1.95.0` en cada step + `Finished` en 1m16s
- **Estado:** ✅

### Step 5: Gate de `FIND-702` contra el run descendiente
- **Archivos:** `.github/workflows/docs-gates.yml:611-655`
- **Acción:** confirmar el job `actionlint` verde y que su run contiene el commit del guard
- **Verify:** `actionlint` = success en `45461424443`; `git merge-base --is-ancestor a78e36ed2 ee41ec51b` exit 0
- **Estado:** ✅

### Step 6: Cierre administrativo (2 filas + avance)
- **Archivos:** `docs/dev/Backlog.md`, `docs/dev/avance/activo/ci-cd.md`
- **Acción:** retirar las filas `FIND-701`/`FIND-702` y registrar el cierre con evidencia y commit
- **Verify:** `rg "FIND-701|FIND-702" docs/dev/Backlog.md` = 0; entrada presente en el avance
- **Estado:** ✅

### Step 7: Review P2-01 + commit local
- **Archivos:** `docs/dev/tasks/FIND-701.md` (§Review)
- **Acción:** review por agente distinto + commit local (sin push)
- **Verify:** veredicto en §Review; `git status` sin archivos ajenos en el commit
- **Estado:** ✅

## Dependencias
- Ninguna. Wave 0.2. `F0.3` (`vanta-docs`) corrió en paralelo sobre `docs/`, `vantadb-node/package.json` y `.github/SECURITY.md` — no se tocó ninguno.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (sesión `ses_df1048a77ffeFsbR5csTj7ipCC`, contexto distinto al autor, solo lectura)
- **Tier:** **Fast** (`docs/dev/**` únicamente — ningún path Adversarial tocado) → verify mecánico + spot-check adversarial del revisor
- **Enfoque:** ¿el diagnóstico de TSan es *infra* porque el stack lo dice y no porque convenga que lo sea? ¿el cierre de cada fila se apoya en el log y no en el nombre del job? ¿el documento afirma más de lo que el comando demuestra?
- **Dictamen inicial: ❌ cambios requeridos — 2 criticals reales, 4 high, 5 medium, 3 low.** El revisor re-corrió todo (`gh run view`, `gh api …/logs`, `gh api …/rulesets`, `git merge-base`, `git ls-files`) en vez de leer el informe. Clasificación y resolución:
  1. **CRITICAL — el required check no existe (✅ aceptado, verificado por mí también).** Los rulesets `develop` (`23692587`) y `main` (`16844590`), ambos `enforcement: active`, exigen `MSRV Check (1.94.1)`; el job se llama `MSRV Check (1.95.0)` desde `5a669a186`. Mi claim (1) tomaba el nombre del job como definición del gate: circular, y del mismo género que el bug que FIND-701 cierra. → **Nueva fila `FIND-715`** (🔴 Crítica) + aviso en §Veredicto (1). No se corrige acá: escribir rulesets es acción del owner, fuera del alcance de la tarea.
  2. **CRITICAL — §Review vacío (✅ aceptado).** Era cierto en el momento de la lectura: el review se estaba ejecutando. Se cierra con este bloque.
  3. **CRITICAL — no existía commit (✅ aceptado, por diseño).** El commit es el último paso; el documento se escribió con `_(este commit)_` por adelantado. Se resuelve al commitear.
  4. **HIGH — "son 4 binarios" (⚠️ ya corregido antes del review).** El revisor leyó una versión previa del fichero; la línea 81 ya decía "4 tests concurrentes del mismo binario" con el log de `821 tests across 60 binaries`. **Noise / snapshot stale** — pero su evidencia (`FAIL (1/821) … api::scores::…`) se incorporó: los cuatro test names son ahora parte del registro.
  5. **HIGH — "verde en todo lo que gatea" con `Tests (Windows)` corriendo (✅ aceptado).** `Tests (Windows)` **es** required check, así que su estado condicionaba el claim. El run ya cerró `success`; claim (1) reescrito con los conteos finales (19/4/4) y el veredicto de Windows.
  6. **HIGH — FIND-702 cerrado sin mencionar el residual del guard (✅ aceptado como trade-off).** El guard es una comparación de conteos y cubre solo `action.yml`. El cierre mecánico es válido (el bug era que el gate no existía), pero ahora la fila de evidencia dice explícitamente qué **no** prueba y remite el residual a `FIND-704`, que sigue abierta.
  7. **MEDIUM — "firma y conteos idénticos" es falso (✅ aceptado).** `59 vs 60` binarios, `2761 vs 2763` filtered-out, offsets y BuildId distintos. Corregido a "misma clase + los mismos cuatro tests" en los **tres** sitios (task file, `FIND-714`, `ci-cd.md`) y con la diferencia numérica a la vista.
  8. **MEDIUM — "teardown" era inferencia, no medición (✅ aceptado).** Los frames son el canal de *eventos completados* del harness, no un hook de salida; el orden `test result: ok` → `WARNING` es interleaving de stdout/stderr. Reescrito: el test no falla, el proceso muere con 66, y se declara que "teardown" no está medido ni hace falta.
  9. **MEDIUM — causa raíz heredada de un comentario que el propio doc desautoriza (✅ aceptado).** Ahora se separa explícitamente: *lo medido acá* (todo el stack es del harness, clase estable entre commits/nightlies) vs *la clase heredada de FIND-227* (`std` sin instrumentar, TSan ciego a fences), marcada como no medida en esta tarea.
  10. **MEDIUM — "22 jobs" vs 18 nombres (✅ aceptado).** Reescrito con los conteos medidos: 27 totales, 19 success, 4 skipped, 4 failure.
  11. **MEDIUM — el total del encabezado del Backlog no se recalculó (❌ noise, contexto del owner).** No había inconsistencia: el `40 🟡` del encabezado son 36 `🟡 Parcial` + 4 `🟡 Pendiente`, y el owner ya corrigió la etiqueta en la línea 18 de ese fichero (edición incluida en este commit). El revisor no tenía ese contexto.
  12. **MEDIUM — `--no-fail-fast` no puede hacer lo que le atribuí (✅ aceptado).** Con ruido en todos los procesos solo convierte "4 rojos" en "~205 rojos lentos". Reescrito: `--no-fail-fast` por sí solo **no** es opción; la opción (a) es aislar el ruido **más** contador de tests ejecutados.
  13. **LOW — timestamps en el futuro y minutos de Windows divergentes (✅ aceptado, parcialmente stale).** Los minutos ya estaban corregidos a 52 en el task file (el revisor leyó el "62+" previo); ahora son 56 con el veredicto real. Los timestamps del frontmatter **sí** estaban mal (usé el reloj UTC de GitHub en campos locales) → corregidos.
  14. **LOW — una sola causa para 4 jobs skipped distintos (✅ aceptado).** Ahora cada uno con su condición real (`event_name`, `ref == main`, `paths:`).
  15. **LOW trade-off — el contrato pedía "jobs de release" y verifiqué `Release Combo` (✅ aceptado).** La fila aclara que `Release Combo` es un job de CI y que de los workflows de release solo hay verificación estática (no corren en un push).
- **Veredicto:** ✅ **approve** tras cerrar los 15 ítems (2 con corrección de fondo: `FIND-715` y los conteos; 1 rechazado como noise con justificación; el resto precisiones incorporadas). El diagnóstico de TSan sobrevivió intacto y la verificación independiente del orquestador (segunda lectura del mismo log) coincide: cero frames de `vantadb`, race en `mpmc::Channel<CompletedTest>`.

## Notas

- **NOTICED BUT TOUCHING (hallazgo nuevo, registrado como `FIND-714`):** la lane TSan es **ciega** además de ruidosa. `nextest` es fail-fast y el ruido de `libtest` está en *todos* los procesos, así que el run muere en <1 s tras probar 4 de 821 tests. Repro exacto: `Starting 821 tests across 60 binaries` → `tests run: 4/821 · 0 passed · 4 failed · 2322 skipped · 0.60–0.79 s` en las 4 particiones, y los 4 procesos son del binario `--lib` (todos `2763 filtered out`). Opciones con contrato en la fila: (a) **aislar el ruido y exigir tests ejecutados** (`tsan.supp` acotado a `compiler_copy`/`mpmc` + umbral por partición) o (b) **retirar el job** con decisión en `CI_POLICY.md` + `# CATEGORY:`. `--no-fail-fast` por sí solo no es opción (convierte 4 rojos en ~205 rojos lentos). **No se toca el workflow acá** porque un gate no se cambia sin pasada real de CI.
- **NOTICED BUT TOUCHING (hallazgo nuevo, registrado como `FIND-715`):** los rulesets exigen el context `MSRV Check (1.94.1)`, que ningún job reporta desde `5a669a186`. El run es verde y el required check sigue insatisfacible: **el PR `release-plz-*` → `main` no puede cumplirlo**. Es la cara que la pasada de CI no puede ver porque el nombre del job y el del ruleset viven en sitios distintos. Acción del owner (o `gh api -X PUT` con token admin), no de esta tarea.
- **NOTICED BUT NOT TOUCHING:** el `continue-on-error` de la lane lleva `CATEGORY: NON-CRITICAL` y está catalogado en `CI_POLICY.md:65`, así que es legítimo según la Regla 2. **No se añadió ninguna exención nueva** ni se retiró la existente: retirarla hoy pondría el run verde en rojo por un defecto de la herramienta, no del repo.
- **NOTICED BUT NOT TOUCHING:** el comentario de `FIND-650` (`ci-rust.yml:961-966`) afirma que nextest eliminó la clase del harness `libtest`; los 4-5 reportes por partición prueban que no. Es parte de `FIND-714` (b): un comentario que justifica el diseño actual con una premisa falsa es lo que impide arreglarlo.
- **Alcance respetado:** no se editó `.github/workflows/ci-rust.yml` (el fix de TSan no es verificable sin push), ni `src/**`, ni `Cargo.toml`/`Cargo.lock`, ni `release-manifest.json`, ni `Formula/**`, ni los rulesets.
