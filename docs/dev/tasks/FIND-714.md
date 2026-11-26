---
title: "FIND-714: la lane TSan no mide nada — suppression del harness + aserción de corpus"
kind: task
status: in-progress
description: "Devolverle utilidad al gate TSan: suppression acotada al canal de libtest, aserción de tests ejecutados por partición y corrección del comentario de FIND-650 que afirmaba lo contrario"
---

# FIND-714: la lane TSan no mide nada — suppression del harness + aserción de corpus

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.b (residuales del run)
- **Fuente:** fila `FIND-714` de `docs/dev/Backlog.md`
- **Esfuerzo:** 🟡 4-8h (la más incierta del lote: el verde no se puede fabricar)
- **Prioridad:** 🟠 Alta (gate de la Regla 8 que hoy no mide: un motor con paths multi-índice sin gate de concurrencia efectivo)
- **Tipo:** CI/CD / DevOps — sin código de producto
- **Carril:** C
- **Dependencias:** `FIND-227` ✅ (la clase de ruido ya está documentada y cerrada)
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `ci-rust.yml` es un check del ruleset de `develop`/`main`. El job `sanitizer-tsan` está en `continue-on-error`, así que **retirar esa exención sí puede bloquear merges**: es el riesgo principal de esta tarea |
| Callees | `cargo +nightly nextest` (flags `--partition`, `--no-fail-fast`), TSan runtime (`TSAN_OPTIONS`: `suppressions`, `halt_on_error`), `llvm-symbolizer` |
| Implicaciones | Ninguna sobre código. Un suppression demasiado ancho apagaría races reales (pre-mortem (a)); una aserción mal contada crearía un falso "verde medido" (pre-mortem (c)) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rust.yml` (1024L), `docs/dev/operations/CI_POLICY.md` (717L), `.opencode/rules/release-ci.md` (42L), `.config/nextest.toml` (108L), `docs/dev/tasks/FIND-701.md`, filas `FIND-714` de `docs/dev/Backlog.md` y §6.b del plan.
- **Referencias hacia dentro:** `ci-rust.yml` es required en ambos rulesets; `docs-gates.yml` (`actionlint` + gate de inventario de workflows) cuenta **archivos** en `.github/workflows/`, no jobs → esta tarea no crea archivos ahí (el nuevo artefacto vive en `dev-tools/`).
- **Referencias hacia dentro del nuevo archivo:** solo el job `sanitizer-tsan` lo consume (`TSAN_OPTIONS.suppressions`); ningún script lo importa.
- **Veredicto de impacto:** alto en el gate de merge solo si se retira `continue-on-error`; **ninguno** en código de producto.

## Contrato

```
npx --no-install actionlint .github/workflows/ci-rust.yml     # exit 0
gh run list --workflow ci-rust.yml --limit 3 --json headSha,conclusion
```

Local, sin `npx` (no hay paquete local; el binario standalone 1.7.12 está en PATH y el job de
`docs-gates.yml` usa 1.7.7 pineado):

```
actionlint .github/workflows/ci-rust.yml                     # exit 0
pwsh dev-tools/ocr-review.ps1                                # advisory, sin API key
```

## Diagnóstico (heredado, no re-medido aquí)

Run `45486044760`, job `138775290328` (partición 1/4):

```
Summary [   0.801s] 4/821 tests run: 0 passed, 4 failed, 2322 skipped
```

Confirmado en el run más reciente (`45510840907`, sha `ec1bf9d58`, job `138887722230`,
2026-11-25T00:44): `Summary [0.777s] 4/821 tests run: 0 passed, 4 failed, 2323 skipped`, y en el
mismo sha `Tests (Linux)` = **success** — el corpus pasa sin TSan, así que el rojo no describe al
motor.

- 821 = tests que nextest selecciona para esa partición; 2322/2323 = resto del corpus
  (3143/3144 totales, según el commit) excluido por `--partition hash:1/4` — el `/` es el
  marcador de **run abortado** (fail-fast), no un formato cosmético.
- Los 4 procesos son del binario `--lib` (`2763 filtered out` en cada uno) y los otros
  59 binarios nunca arrancan.
- La firma es siempre la misma y **sin frames de `vantadb`**:
  `<std::sync::mpmc::list::Channel<test::event::CompletedTest>>::read` /
  `::write` / `::send` / `::recv` / `::start_send` vía
  `core::profiling::compiler_copy::<MaybeUninit<CompletedTest>, 304>`. Inventario del run
  `45510840907`: **5 reportes, 5 del canal de `libtest`, 0 frames de `vantadb`, 0 de rayon/flume**
  — la clase del run actual está enteramente cubierta por las dos entradas del suppression.
- El comentario de `FIND-650` (`ci-rust.yml:961-966`) afirmaba que nextest elimina esta
  clase porque "runs each test in its own process". nextest invoca el binario de libtest
  con `--exact`, pero libtest **sí** construye su `mpmc::Channel<CompletedTest>` en cada
  proceso. La premisa era falsa y es lo que impedía arreglarlo.

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | `dev-tools/tsan-suppressions.txt` con `race:std::sync::mpmc` + `race:CompletedTest` y alcance documentado | ✅ |
| 2 | Cablear `TSAN_OPTIONS=suppressions=<abs>:halt_on_error=1:...` + guard loud si el archivo falta | ✅ |
| 3 | `--no-fail-fast` + `tee` del log y aserción `executed == selected` (N medido por nextest, no hardcodeado) | ✅ |
| 4 | Corregir el comentario de `FIND-650` con la evidencia del run | ✅ |
| 5 | `CI_POLICY.md`: contrato de la lane, alcance del suppression, estado de `continue-on-error` | ✅ |
| 6 | Verify: actionlint + docs gates + ocr-review + review P2-01 | ✅ |
| 7 | **Pendiente del owner:** run real con `partition <n>/4: <N> tests run (selected == executed)`, `race reports: 0`, `suppression hits: > 0` para retirar `continue-on-error` | ⬜ |

## Investigación (digest, fuentes verificadas)

- **Sintaxis y semántica.** Una suppression por línea, `tipo:string`; el string se compara
  contra *nombres de función, archivo y módulo* de **todas** las frames (no solo la primera) y,
  en data races, también contra el stack del acceso previo; `*` se agrega por defecto en ambos
  extremos; `race_top` es la variante que mira solo la frame superior. Fuente:
  [ThreadSanitizerSuppressions](https://github.com/google/sanitizers/wiki/ThreadSanitizerSuppressions)
  (verificado 2026-11-25) y el parser real en
  [tsan_suppressions.cpp](https://github.com/llvm/llvm-project/blob/main/compiler-rt/lib/tsan/rtl/tsan_suppressions.cpp)
  (`IsSuppressed` recorre `SymbolizedStack` con `Match(info.function) || Match(info.file) || Match(info.module)`).
- **Que el archivo se cargue de verdad.** `InitializeSuppressions()` hace
  `suppression_ctx->ParseFromFile(flags()->suppressions)` → `TSAN_OPTIONS=suppressions=<ruta>` es
  el canal correcto; el mismo archivo añade `PrintMatchedSuppressions()` (log
  `ThreadSanitizer: Matched N suppressions`), que es la señal de que la entrada **matcheó** y no
  solo cargó. La aserción del step imprime ese conteo por partición.
- **Por qué el ruido existe.** El Unstable Book de Rust documenta que TSan "needs to be aware of
  all synchronization operations" y que usarlo sin instrumentar todo el código "can lead to false
  positive reports", y que **no soporta `atomic::fence`**
  ([sanitizer.html](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html)).
  Sin `-Zbuild-std`, `std` queda sin instrumentar y el canal de `libtest` se reporta racy.
- **nextest.** `--no-fail-fast` (alias `--nff`) es flag oficial y el fail-fast es el default del
  perfil (`[profile.default]` no lo desactiva; `[profile.audit]` sí).
  [nexte.st/docs/running](https://nexte.st/docs/running/). No se tocó `--test-threads`, corpus ni
  timeout.
- **Sin ejemplo público equivalente.** La búsqueda de código en GitHub por
  `race:std::sync::mpmc` (`gh api search/code`) devolvió `total_count: 0` — evidencia débil, porque
  el endpoint puede estar limitado; los motores web no devolvieron nada útil. Conclusión honesta:
  no se encontró un `.supp` público análogo, así que la sintaxis se fundó en el runtime y su parser
  (arriba), que es fuente más fuerte que copiar el archivo de otro proyecto.

## Alcance añadido durante la tarea

`dev-tools/tsan-suppressions.txt` entró en los `paths:` de `push` y `pull_request` de
`ci-rust.yml`: sin eso, editar el suppression no re-ejecutaba la lane. Un input del gate sin
trigger es la misma clase de defecto que esta tarea cierra.

Residual análogo **no** tocado: `.lsan_suppressions` (lane ASan) tampoco está en `paths:`. Es
pre-existente y queda para el orquestador como fila `FIND-*` (no se escribe `Backlog.md` desde
esta tarea).

## Lo que esta tarea NO hace

- **No** retira `continue-on-error` sin un run verde real: no hay push en esta sesión y el
  verde no se fabrica. Queda documentado el comando exacto para el owner y la salida
  honesta si sigue rojo (opción (b) del plan: retirar el job con decisión en
  `CI_POLICY.md` + `# CATEGORY`).
- **No** cambia `--test-threads`, el corpus ni el timeout para tapar el problema.
- **No** ensancha el suppression a `race:std` (ver §Veredicto).

## Veredicto sobre el alcance del suppression

`race:` se compara contra los nombres de funciones y de archivos del stack principal y del
stack del acceso previo (documentación oficial de TSan, §Suppressions), así que una entrada
ancha silencia cualquier race en cuyo stack aparezca ese string. Las dos entradas elegidas son
libtest-only:

- `std::sync::mpmc` — `rg -n "mpmc" src/ tests/` → 0 hits: el crate no usa ese canal.
- `CompletedTest` — tipo de evento de `libtest`, inexistente en `src/`.

Si en el próximo run aparece un frame de `vantadb` en un reporte, ese reporte **no** queda
suprimido: es señal real y es exactamente lo que la lane debe decir.

## Review (P2-01)

- **Reviewer:** `vanta-review` (contexto fresco, sin acceso a la sesión que implementó).
- **Veredicto inicial:** 🔴 `changes-required` — 0 Critical · 3 Required · 3 Optional · 3 Nits.
  Enfoque aprobado; bloquearon dos afirmaciones que la evidencia desmentía (misma clase de defecto
  que esta tarea existe para corregir) y un límite de commit roto.
- **Veredicto final: `approve`.** El reviewer lo dejó condicionado por escrito: *"Resueltos esos
  tres, el veredicto es `approve` sin condiciones adicionales."* R1, R2 y R3 están resueltos y
  verificados abajo. Se registra la condición y su cumplimiento, no una aprobación literal
  posterior (la tarea pedía una ronda).
- **R1 (corregido):** `grep … | tail -1` **no** devuelve 0 bajo `bash -eo pipefail` (el shell real
  de `run:` en GitHub): pipefail propaga el fallo de `grep` y `set -e` mataba el step en la línea
  del `SUMMARY`, dejando el `::error::` como código muerto. Ahora `if ! SUMMARY="$(…)"` es gate y
  diagnóstico a la vez. El comentario que afirmaba lo contrario se borró.
- **R2 (corregido):** `0 tests run` pasaba la aserción (selección vacía = verde que no midió nada) y
  el texto decía que el corpus "no se podía debilitar", cuando numerador y denominador encogen
  juntos. Ahora hay un `-eq 0` explícito y el alcance real está declarado: detecta **abortos** y
  **selección vacía**; el corpus lo acotan la `-E` y el `default-filter`.
- **R3 (corregido por el commit de FIND-725):** el índice mezclaba el blob de `FIND-725` (ronda 2,
  wasm) con el unstaged de esta tarea. `1b6e19024` commiteó lo suyo con blob propio; el índice de
  esta tarea se armó y verificó por pathspec antes de commitear.
- **Nits aplicados:** regex tolera `1 test run` singular (`tests? run`); "step 6" → "step 7"; fecha
  de la corrección alineada con la creación de la tarea; `N/N` reemplazado por la cadena real que
  imprime el step; nota de que el alcance efectivo del suppression es la **unión** de las dos
  entradas.
- **Verificación del fix:** los 7 shapes (abortado, completo-con-fallos, completo-limpio,
  sin-summary, sin-log, selección 0, singular) pasan ejecutando el bloque literal bajo
  `bash --noprofile --norc -eo pipefail`. Harness: `%TEMP%\opencode\find714-assert-pipefail.sh`
  (efímero, no versionado).
- **Evidencia que falta para cerrar del todo:** la del step 7 (run del owner).

## Cierre — commit bloqueado por el árbol compartido

El cambio está completo, staged y verificado, pero **el commit no se pudo crear**: el hook
`pre-commit` aborta con `Your pre-commit configuration is unstaged` porque
`.githooks/pre-commit-config.yaml` tiene un cambio **en vuelo de otra tarea** (`FIND-718`,
`default_language_version: node: lts`). Con un commit por pathspec el índice temporal parte de
`HEAD`, así que el guard ve esa config como "unstaged" siempre que el cambio exista y no esté
commiteado. Además, en dos intentos el índice compartido tuvo un path en conflicto
(`UU docs/dev/plans/archive/2026-09-03-quality-gtm-wave.md`) y un `index.lock` de otro proceso.

No se usó `--no-verify` (prohibido) ni se tocó el estado de las otras tareas.

**Comando exacto para el owner, una vez que `FIND-718` commitee su config:**

```
git commit -m "ci(tsan): suppress the libtest harness race and make the lane measure (FIND-714)" \
  -- .github/workflows/ci-rust.yml dev-tools/tsan-suppressions.txt \
     docs/dev/operations/CI_POLICY.md docs/dev/tasks/FIND-714.md
```

El `actionlint` del hook ya pasa sobre este changeset (se corrió también sobre el blob staged:
exit 0).
