---
title: "TASK FIND-227: TSan — supresión targeted o decisión documentada"
kind: task
description: "Decisión status quo (best-effort) documentada: 565/583 reports 100% en el binario lib de tests; familias std/libtest + rayon/fjall/flume (std sin instrumentar + fences que TSan no modela); supresión targeted no converge (17/565) y la convergente sobre-suprime; sin tsan.supp"
---

# TASK FIND-227: TSan — supresión targeted o decisión documentada

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 10, Wave 1)
- **Fuente:** `docs/dev/Backlog.md` (FIND-227)
- **Esfuerzo:** 🟢 3-4h (+1 ciclo CI) | **Appetite:** 1d
- **Prioridad:** 🟢 Baja
- **Tipo:** CI/CD-DevOps (job best-effort + decisión documentada) — blast radius: 1 job + 1 task file; cero código de producción
- **Turns estimados:** 20-40
- **Creado:** 2026-10-03 | **last-synced:** 2026-10-03
- **Estado:** ⏳ IN PROGRESS → ✅ COMPLETED al cierre
- **Campaign ID:** post-release-0.8.0-20261002
- **Incógnitas (uphill):** 1 → 0 (¿la race es supresible con patrones acotados? resuelto: no converge sin sobre-suprimir — evidencia §Convergencia)
- **Pendientes (downhill):** 1 (verificación post-push opcional, orquestador/owner); Steps 1-6 ✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/ci-rust.yml` job `sanitizer-tsan` (best-effort, `continue-on-error`) — único consumidor del job. El comentario nuevo es inerte (sin cambio de comportamiento). |
| Callees | `cargo +nightly test --target x86_64-unknown-linux-gnu --package vantadb` con `-Zsanitizer=thread`; `TSAN_OPTIONS` (sin `suppressions=`); binario `-p vantadb --lib` (unit tests). No se crea `tsan.supp` (decisión §Decisión 2). |
| Implicaciones | Cero cambios de comportamiento: el job queda idéntico (mismos flags, mismo `TSAN_OPTIONS`, sin archivo de supresiones). El cambio es un bloque de comentarios que documenta las familias de ruido conocidas + la decisión, para que el rojo crónico sea auto-explicativo. Ninguna firma pública, wire format, performance ni datos se tocan. |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/ci-rust.yml` (733L — jobs `sanitizer-asan` L630-683 y `sanitizer-tsan` L685-733; el primero NO se toca), `.lsan_suppressions` (9L — referencia de estilo: comentarios + `leak:rocksdb`), `docs/dev/operations/CI_POLICY.md` (§1 Fast Gate — `sanitizer-tsan` listado como best-effort; taxonomía CATEGORY), `.opencode/rules/release-ci.md` (42L — Regla 5 CATEGORY), `docs/dev/workflow/RULES.md` §6 (adjacency del `CATEGORY:` a `continue-on-error`), `src/index/distance/kernels.rs` (L1-60, L225-248 — `OnceLock`/`select_kernels`, PERF-38), `src/node/bitset.rs` (L60 vía log — `LazyLock<FilterBitset>`).
- **Referencias entrantes (grep):** `sanitizer-tsan` = 1 job en `ci-rust.yml` + fila en `CI_POLICY.md` §1; `TSAN_OPTIONS` = 1 ocurrencia (L733); `tsan.supp`/`suppressions=tsan` = **0 ocurrencias** en el repo (solo el plan lo menciona como "posible") — no hay links que romper.
- **Referencias salientes de los editados:** `ci-rust.yml` → actions pinneadas por SHA (`actions/checkout@3d3c42e5…`, `dtolnay/rust-toolchain@02cb101e…`), apt packages, `rust-setup`; task file → plan, logs GH, docs oficiales citadas.
- **Veredicto impacto:** **BAJO / aislado.** Cambio comment-only en 1 job + 1 task file. Reversible con `git revert` (1 línea de diff efectivo). No altera gates, no toca `sanitizer-asan` (FIND-226) ni ningún otro job/workflow.

## Evidencia dura (análisis de runs TSan)

### Runs analizados (logs completos descargados vía `gh run view --job <id> --log`)

| Run | Job | Fecha | Resultado | Reports |
|-----|-----|-------|-----------|---------|
| 37090029650 (PR release-plz) | 111110955601 | 2026-10-03T02:46Z | failure | **565 warnings** |
| 37082038794 (push main) | 111086294694 | 2026-10-03T00:34Z | failure | **583 warnings** |

- Cierre del job: `ThreadSanitizer: reported 565 warnings` → `error: test failed, to rerun pass -p vantadb --lib` → `exit code 66`. **Todo el log es de UN binario** (pid único 10821/…): el target `--lib` de unit tests. Los binarios de integración nunca corren (cargo se detiene en el primer target que falla).
- Crónico: muestreo de runs 2026-09-25 → 10-03 (`gh run view <run> --json jobs`): **todos los runs completados con job TSan = failure** (36180022072 09-25, 37064937902 10-02, 37090029650 10-03; los "skipped" corresponden a runs cancelados).
- Config actual (L685-733): **sin `-Zbuild-std`**; `TSAN_OPTIONS: "halt_on_error=0"` **sin `suppressions=`**; `--target x86_64-unknown-linux-gnu` (rustflags aplican a deps del registry, no a std precompilado).

### Desglose por familias (565 reports, clasificador propio sobre los frames de la sección de race)

| Familia | Reports | Firma (SUMMARY / top frames) |
|---------|---------|------------------------------|
| Allocator-op | 450 | `free` (247) vs `atomic_sub` (227) — teardown de `Arc`/`Weak` (flume Hook de fjall worker pool, `test::run_test`, thread lifecycle); `__tsan_memcpy` (133+108) vs memcpy/memcpy |
| `std::thread::lifecycle::Packet<Result<(), fjall::Error>>::drop` | 34 | drop del packet de join vs memcpy |
| Rayon (sleep/latch/stack-job) | ~55 | `Sleep::wake_specific_thread` (23), `Sleep::sleep` (15), `LockLatch` (14), `StackJob::execute` vs caller read — varios con `Location is stack of thread X` |
| Rayon (collect interno) | 7 | `ListReducer`/`fast_collect` (`rayon-1.12.0/src/iter/extend.rs`) — mismo mecanismo fence/stack-job (detectado en review ronda 1) |
| fjall/flume/crossbeam | ~56 | `fjall::worker_pool`, `flume::Shared::recv/send`, `crossbeam_epoch` collector init vs Arc clone |
| std-mpmc/libtest | 17 | `std::sync::mpmc::list::Channel<test::event::CompletedTest>` (4), `test::run_test` (1), `core::profiling::compiler_copy/move` (17 combinados) |

- **Corrección de la premisa del plan:** el plan describía la race como "100% mpmc + test harness, cero frames de VantaDB". El análisis del log completo muestra que la familia mpmc/libtest/compiler_* es **17/565 (~3%)**; el resto son allocator-op + rayon + fjall/flume/crossbeam + std internals. Y **sí aparecen frames de VantaDB** como top frame en ~40-50 reports — pero nunca como un par no sincronizado de producto (ver clasificación abajo). La premisa era parcial; la decisión se basa en el log completo.
- **Clasificación de los reports con frame VantaDB en el tope (spot-check R97/R99/R133/R140 + muestreo):**
  1. **OnceLock/LazyLock init FP** (R97: `kernels.rs:247` lee el fn pointer cacheado por `OnceLock<DistanceKernels>`; R99: `bitset.rs:60` lee `LazyLock<FilterBitset>`): el write ocurre en `OnceLock::initialize`/`LazyLock::force` y el reader obtiene la referencia vía `get_or_init`/`deref` — la sincronización release/acquire vive en `std::sys::sync::once::futex::Once` (**std precompilado, sin instrumentar** → TSan no la ve).
  2. **Rayon StackJob** (R128/R130/R133/R140: `Location is stack of thread X`; el worker escribe con `__tsan_memcpy` en `StackJob::execute` mientras el caller lee su propio stack): sincronización por latch/fences que TSan no modela (rayon-rs/rayon#812).
  3. **Test harness** (R0-R3: `mpmc::list::Channel` + `test::run_test` + `compiler_copy/move`): ruido de libtest (rust-lang/rust#39608).
  4. **Teardown de deps** (R10 y familia `free`/`atomic_sub`): drop de `DatabaseInner`/worker pool de fjall (flume Hook) vs worker thread — sincronización de cierre vía fences/park de std sin instrumentar.
- **Ningún report muestra una race entre dos accesos de producto VantaDB sin una edge de sincronización que TSan simplemente no puede modelar.** Los frames VantaDB son puntos de acceso al valor (lectura de cache OnceLock, valores movidos por rayon/flume) o frames del harness — no el lado no sincronizado de una race real de producto.

### Convergencia de supresiones (la prueba de sobre-supresión)

Simulación mecánica sobre los 565 reports (patrón `race:` matchea CUALQUIER frame del stack — LLVM docs §Suppressions):

- **Intento 1 — supresión targeted std/libtest** (`std::sync::mpmc`, `test::run_test`, `test::event::CompletedTest`, `compiler_copy`, `compiler_move`): **suprime 17/565** → quedan 548 → el job sigue rojo. **No converge.**
- **Intento 2 — convergencia**: requeriría patrones blanket — `free` (247), `__tsan_memcpy` (255), `rayon` (274), `fjall` (284), `flume` (152), `std::thread::lifecycle` (556), `LockLatch` (236)… Es decir: supresión global de allocator/memcpy + de TODA la capa de storage (fjall) y del runtime paralelo (rayon). Eso es exactamente la **sobre-supresión del pre-mortem #1** (`race:memcpy` global prohibido): ocultaría races reales justo donde más importan (backend de storage, rebuild HNSW con rayon). **Descartado por diseño.**
- Stop condition aplicada: 2 intentos que no convergen → decisión "status quo" documentada (este archivo).

### Causa raíz (clase documentada — validada contra fuentes oficiales)

1. **std sin instrumentar.** El job no usa `-Zbuild-std`; Rust Unstable Book §ThreadSanitizer: *"Using it without instrumenting all the program code can lead to false positive reports"* + *"It is strongly recommended to combine sanitizers with recompiled and instrumented standard library, for example using cargo `-Zbuild-std`"*. LLVM §Limitations: *"ThreadSanitizer generally requires all code to be compiled with `-fsanitize=thread` … may report false positives."* → explica las familias OnceLock/LazyLock, mpmc, thread lifecycle, Arc refcount.
2. **Fences no soportados.** Rust Unstable Book: *"ThreadSanitizer does not support atomic fences"*; rust-lang/rust#65097 (fences no entendidos → *"numerous false positives"*; arreglado para `Arc` en 2020) → explica rayon work-stealing (#812, abierto), crossbeam_deque (#589, cerrado), parking_lot (#257).
3. **Test runner.** rust-lang/rust#39608: *"ThreadSanitizer detects a data race in the test runner (`rustc --test`)"* — la misma clase del patrón `free` + `test::run_test` (rayon-rs/rayon#1211 lo reproduce idéntico con `-Zsanitizer=thread` sin build-std, **cerrado as not planned**).

## Decisión

1. **Status quo (best-effort) documentado — SIN `tsan.supp`.** El contrato del plan ofrece dos ramas: (a) verde con supresión targeted, o (b) decisión "status quo" con evidencia si la supresión sobre-suprime. Aplica **(b)**: la supresión targeted no converge (17/565) y la convergente sobre-suprime (allocator + storage + runtime paralelo). No se crea `tsan.supp`: un archivo que no puede poner el job verde sería lastre + falsa expectativa, y cablear `suppressions=` sin convergencia deja el mismo rojo con más superficie.
2. **El job se vuelve señal accionable por documentación, no por supresión:** bloque de comentarios en `sanitizer-tsan` (comment-only, cero cambio de comportamiento) que resume familias + causa raíz + decisión + puntero a este archivo. Quien vea el rojo encuentra la explicación en el propio workflow.
3. **Camino de fix propio (tarea futura, fuera de scope):** instrumentar std con `-Zbuild-std` en el job TSan (elimina la clase 1 de FPs). Costo: rebuild de std + deps por run (~10-25 min extra en ubuntu-latest) — decisión de costo a evaluar por el owner; los falsos positivos de fences (clase 2) persistirían en rayon/fjall. No se ejecuta acá: cambia el costo/tiempo del job, excede el contrato de esta tarea.
4. **No se toca `sanitizer-asan`** (FIND-226, cerrado en `7ea9ab3e`) ni ningún otro job/workflow.

## Contrato (del plan)

| Rama | Estado |
|------|--------|
| (a) TSan verde con `suppressions=tsan.supp` en 1 run de CI | **No aplica** — demostrado no convergente sin sobre-supresión (§Convergencia, intentos 1-2) |
| (b) Decisión "status quo (best-effort)" registrada en FIND-227 con la evidencia del análisis | **✅ Cumplida** — este archivo (§Evidencia dura + §Decisión) + comentario in-place en el job |

- **Verificación local del tramo (b):** `actionlint .github/workflows/ci-rust.yml` exit 0 + `node scripts/docs/check-links.mjs` exit 0 + `node scripts/docs/check-docs.mjs` exit 0 + grep del bloque FIND-227 en el job + ausencia deliberada de `tsan.supp`.
- **Tramo post-push (opcional, orquestador/owner):** el cambio es comment-only y `ci-rust.yml` no dispara con push a develop (triggers: push main / PR main / dispatch). Verificación cuando ocurra el próximo run de ci-rust (PR a main o `gh workflow run ci-rust.yml --ref <rama>`): el job debe comportarse idéntico (mismo conteo de familias; el comentario no altera nada). Comandos: `gh run list --workflow=ci-rust.yml --limit 1` → `gh run view --job <tsan-job-id> --log | Select-String "reported .* warnings"` → esperado: `failure` best-effort con la misma clase de reports (status quo confirmado).

## Steps

- [x] **Step 1 — Evidencia dura de runs:** descarga completa de los logs de los jobs 111110955601 (565 warnings) y 111086294694 (583); cierre `-p vantadb --lib` + exit 66; muestreo de chronicidad 09-25→10-03; config actual del job (sin build-std, sin suppressions). ✅
- [x] **Step 2 — Clasificación por familias:** clasificador propio (565 reports): allocator-op 450, thread-lifecycle 34, rayon ~55, fjall/flume/crossbeam ~56, std-mpmc/libtest 17; spot-check de 8 reports completos (R2/R4/R10/R97/R99/R128/R133/R140) → 4 clases de FP documentadas; corrección de la premisa del plan (mpmc/libtest = 3%, no 100%; frames VantaDB sí aparecen, pero nunca como par no sincronizado de producto). ✅
- [x] **Step 3 — Análisis de convergencia de supresión (intentos 1-2):** targeted = 17/565 → no converge; convergente = blanket allocator+fjall+rayon+flume → sobre-supresión (pre-mortem #1) → decisión status quo. ✅
- [x] **Step 4 — Comentario in-place en el job:** bloque FIND-227 en `sanitizer-tsan` (después de `continue-on-error: true` para preservar la adyacencia CATEGORY de RULES.md §6; cero cambio de comportamiento). **Verify:** `actionlint .github/workflows/ci-rust.yml` → exit 0 ✅
- [x] **Step 5 — Task file + docs:** este archivo (evidencia + decisión + contrato + DoD). **Verify:** `node scripts/docs/check-links.mjs` ✅ · `node scripts/docs/check-docs.mjs` ✅
- [x] **Step 6 — Cierre local:** OCR delegation + review P2-01 (agente fresco) + commit LOCAL conventional (`ci: FIND-227 — …`). ✅ (ver §Review)

## Review (P2-01)

- **Revisor:** `vanta-review` (sub-agente fresco; sesión `ses_efffffac8ffeahYtpk4o4HM5Br`; contexto ≠ autor).
- **Paths del diff:** `.github/workflows/ci-rust.yml`, `docs/dev/tasks/FIND-227.md` → **Tier Fast** (CI + docs; sin paths adversariales).
- **Gate requerido:** verify fast mecánico (actionlint + docs checks) + veredicto registrado.
- **Ronda 1 — 🔴 changes-required** (1 High procedural, 1 Medium, 1 Low, 1 nit):
  - High: Step 6/DoD pre-claimaban el commit antes de existir → resuelto con el commit local de este mismo cambio.
  - Medium: atomicidad — `opencode.jsonc` (WIP ajeno) queda fuera del commit → respetado (`git show --stat`).
  - Low: la tabla de familias omitía el collect de rayon (7 reports `ListReducer`/`fast_collect`) → añadido a §Desglose.
  - Nit: la cita LLVM §Suppressions estaba agrupada bajo causa-raíz (documenta matching de supresiones, no fences) → reubicada en el comentario.
  - **Re-derivación adversarial independiente (evidencia del revisor):** 17/565 exacto re-derivado desde el log crudo; **0 reports con dos frames `#0 vantadb::`** (falsación del claim "producto-vs-producto" fallida — el claim se sostiene); citas #39608/#65097/#812/#1211/Unstable Book reales; diff comment-only; RULES.md §6 intacto; `sanitizer-asan` intacto.
- **Ronda 2 — ✅ approve** (sin findings remanentes): commit `396b4019` verificado (`ci:` + task ID, local); `opencode.jsonc` fuera del commit; familia rayon-collect añadida; nit de cita corregido; `actionlint` exit 0. Los 4 hallazgos de ronda 1 resueltos y los conteos núcleo (565/583, 17/565, familias) re-derivados por el revisor.
- **Veredicto final:** **✅ approve** — `vanta-review`, sesión `ses_efffffac8ffeahYtpk4o4HM5Br` (contexto fresco ≠ autor; P2-01 satisfecho).

## DoD (3 niveles)

- **task:** contrato rama (b) cumplido — decisión status quo registrada con evidencia dura verificable (logs GH, conteos reproducibles, fuentes oficiales). El contrato NO exige "verde" si la supresión sobre-suprime (rama explícita del plan).
- **commit:** conventional (`ci:`), atómico, verify mecánico local (comment-only + docs checks); commit LOCAL, sin push.
- **release:** n/a (job best-effort; no gatea releases).

## Deuda técnica (Regla 6)

- **Introducida:** ninguna (0 dependencias; comentarios + task file).
- **Pagada:** elimina la ambigüedad del rojo crónico del job TSan (mystery → documented noise con causa raíz y camino de fix). Reduce el costo de triage futuro: la próxima persona/agente que vea el job encuentra la clasificación completa sin re-analizar 565 reports.

## Herramientas necesarias

- `gh` CLI 2.91 (logs de runs), `actionlint` 1.7.12, `node scripts/docs/*.mjs` (docs), OCR delegation (`dev-tools/ocr-review.ps1`), PowerShell (clasificador de logs en `%TEMP%\opencode\find227\`).

**Skills cargadas (SDP):** `campaign-executor` + `progreso` (base, auto), `ci-cd-and-automation` (pinned CI), `git-workflow-and-versioning` (pinned CI), `doubt-driven-development` (base type), `incremental-implementation` + `test-driven-development` + `context-engineering` (lifecycle BUILD), `documentation-skill` (obligatoria por editar `.md` bajo `docs/`), `coordinated-web-search` (router de investigación web — usado para validar TSan suppressions y la causa raíz contra docs oficiales).
