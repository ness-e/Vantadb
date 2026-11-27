---
title: "FIND-794: ningún `-E` de nextest puede ejecutarse sin una aserción de población"
kind: task
status: completed
description: "Un filtro `-E` que no selecciona nada no cambia el color del gate: el contrato de FIND-782 declaraba `test(find782)` y medía 0 tests. Se fija `--no-tests fail` en las 4 invocaciones de dev-tools (CI ya lo lleva en las suyas) y un check estático que impide añadir un `-E` sin él, más la existencia de cada `test(<nombre>)` positivo."
---

# FIND-794: la selección vacía no cambia el color

## Metadata
- **Fuente:** `docs/dev/Backlog.md` §`Alta 2026-11-26` (fila `FIND-794`)
- **Origen:** cierre de `FIND-782` (review P2-01, R2) · **Ver:** `FIND-785`, `FIND-768`, `FIND-766`, `FIND-788`
- **Esfuerzo:** 1-2h · **Prioridad:** 🔴 Alta · **Tipo:** tooling/CI (sin Rust)
- **Creado:** 2026-11-27 · **Estado:** ✅ COMPLETED

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `dev-tools/check-harness-contract.mjs` (check nuevo + fixtures) · `dev-tools/verify.ps1` · `dev-tools/coverage-budget.ps1` · `dev-tools/e2e-memory-cycle.ps1` · `dev-tools/scripts/run-compat-v090-rollback.ps1` |
| Inmutable | `.github/workflows/certification.yml` y `snapshot-certification.yml` (PROHIBIDOS; ya llevan `--no-tests fail`) · `dev-tools/git-commit-isolated.ps1` (recién cerrado) |
| Fuera de alcance | `src/**`, `vanta-memory/src/**` (`WIRE-20` compilando), `vantadb-mcp/**` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `dev-tools/check-harness-contract.mjs` (1444 L: helpers, `CHECKS`, `runSelfTest`) · `dev-tools/verify.ps1` (156 L) · `dev-tools/gate-common.ps1` (34 L) · `dev-tools/coverage-budget.ps1` (§70-83) · `dev-tools/e2e-memory-cycle.ps1` (§30-44) · `dev-tools/scripts/run-compat-v090-rollback.ps1` (23 L) · `.github/workflows/certification.yml` (§380-570) · `.github/workflows/snapshot-certification.yml` (§108-227) · `docs/dev/tasks/FIND-782.md` (§Contrato) · fila `FIND-794` del Backlog.
- **Referencias hacia dentro del check:** `dev-tools/check-harness-contract.mjs` se invoca desde `/harness` y desde `verify_staged.ps1`; añadir un `CHECKS` no cambia la CLI (`--self-test`, `--rows`, `--plan` intactos).
- **Referencias hacia dentro de `--no-tests fail`:** ninguna. Es un flag de nextest, no un símbolo del repo; los 4 sitios lo pasan a `cargo nextest run` (y a `cargo llvm-cov nextest`, cuyo contrato documentado es `[NEXTEST_OPTIONS]` → nextest run).
- **Referencias entrantes de los 4 scripts:** `verify.ps1` ← hook pre-push, `verify_changed.ps1`, `audit-all.ps1`; `coverage-budget.ps1` ← `nightly.yml`; `e2e-memory-cycle.ps1` y `run-compat-v090-rollback.ps1` ← manuales (runbook).
- **Veredicto impacto:** bajo y confinado a `dev-tools/**`. El riesgo real no es el diff: es que un flag mal puesto apague un gate. Por eso cada sitio se ejercita con el mismo comando de su contrato (ver §Verificación).

## Premisa (medida, no re-derivada)

La fila afirma: «un `-E 'test(<nombre>)'` que no matchea ningún test **sale exit 0**». Medido en esta máquina (nextest 0.9.133, perfil `default` y `audit`):

```
$ cargo nextest run -p vantadb --test derived_index_prefix_scan -E 'test(no_existe_jamas)'
     Summary [   0.000s] 0 tests run: 0 passed, 1 skipped
error: no tests to run
RAW_EXIT=4                      ← NO es 0
```

⇒ **El «exit 0» del `run` no reproduce en 0.9.133** (`--no-tests=auto` ya devuelve 4 en una ejecución filtrada). Lo que **sí** reproduce es el verde silencioso por las otras tres puertas, todas medidas:

| Puerta | Comando | Resultado medido |
|---|---|---|
| `--no-tests pass` (nada lo prohíbe) | `… -E 'test(no_existe_jamas)' --no-tests pass` | **exit 0**, resumen idéntico (`0 tests run: 0 passed`) |
| `list` (la medida que usan `FIND-768/785/788`) | `cargo nextest list … -E 'test(no_existe_jamas)'` | **exit 0**, 0 entradas |
| lectura del pipeline | `… \| Select-Object -Last 4` + `$?` | verde (`$?` es del último cmdlet, no del binario) |

Y el propio registro de `FIND-782` (`docs/dev/tasks/FIND-782.md:41,113`) no cita ningún exit code: dice «devuelve **0 tests**» y «el gate local quedaba verde». La parte del hallazgo que se sostiene es la que importa —**la selección vacía no cambia el color**—, y la palanca correcta es la misma sea cual sea el código de salida: **una aserción de población, no la confianza en el exit code.**

## Inventario de `-E` (medido, no supuesto)

Contado en dos unidades distintas, porque no son la misma y confundirlas fue el primer defecto que el review P2-01 cazó:

- **8 declaraciones de selector** (fases `run_phase`): 1 en `certification.yml` (`:539`) + 7 en `snapshot-certification.yml` (`:236,238,241,243,245,247,249`).
- **4 invocaciones de nextest con `-E` en CI**: en cada lane el selector vive en un array (`run_args+=(-E …)` / `expect+=(-E …)`) y lo consumen **dos** comandos — un `list` (que mide) y un `run` (que gatea). `certification.yml:466,468` → `:472` y `:494`; `snapshot-certification.yml:168,170` → `:175` y `:201`.
- **4 invocaciones con `-E` en `dev-tools`**, ninguna con aserción (hoy).
- **Total: 8 invocaciones con `-E`** (4 CI + 4 `dev-tools`).

| Sitio (líneas de `HEAD`) | Filtro | Población | `--no-tests fail` |
|---|---|---|---|
| `.github/workflows/certification.yml:539` (fase 03, `scoped`) | `test(index::core::tests::concurrent_insert_preserves_hnsw_invariants)` | 1 (matchea `fn concurrent_insert_preserves_hnsw_invariants`) | ✅ `:497` |
| `.github/workflows/snapshot-certification.yml:236,238` | `test(snapshot) or test(pending_restore)` | >0 (substring de `test_snapshot_create_and_list`, `test_pending_restore_symlink_marker_fails_closed`) | ✅ `:204` |
| `.github/workflows/snapshot-certification.yml:241,243` | `binary(snapshot_certification)` (sin `test()`) | >0 | ✅ `:204` |
| `.github/workflows/snapshot-certification.yml:245` | `test(test_snapshot_rocksdb_native_checkpoint_roundtrip)` | 1 (existe la `fn`) | ✅ `:204` |
| `.github/workflows/snapshot-certification.yml:247,249` | `test(snapshot)` | >0 | ✅ `:204` |
| `dev-tools/verify.ps1:115` | `$fastGateFilter` = 3 × `not test(/…/)` | >0 (todo menos 3) | ❌ → ✅ `:118` |
| `dev-tools/coverage-budget.ps1:77` | mismo `$fastGateFilter` | >0 | ❌ → ✅ |
| `dev-tools/e2e-memory-cycle.ps1:41` | `test(e2e_l0)` | >0 (`fn e2e_l0_to_l3_cycle_in_the_shipped_binary`) | ❌ → ✅ `:43` |
| `dev-tools/scripts/run-compat-v090-rollback.ps1:21` | `test(compat_v090_rollback)` | >0 (`fn compat_v090_rollback_fixture_reopened_by_v090_binary`) | ❌ → ✅ |

Átomos `test()` **positivos: 10**, los 10 resuelven contra un `fn` del workspace; los 3 negados (`gate-common.ps1:33`) quedan fuera a propósito. Ninguna de las 4 de `dev-tools` podía distinguir «0 seleccionados» de «verde».

## Decisión: `--no-tests fail` + check estático (opción c)

Evaluadas las tres del contrato:

| Opción | Por qué no |
|---|---|
| (a) wrapper `dev-tools/nextest-run.ps1` (`list` → afirmar población → `run`) | **Crea dos poblaciones** (pre-mortem c): el wrapper arma el `list` y el `run` por separado, y el día que uno de los dos reciba un arg distinto hay dos verdades. Además nextest **ya** tiene la aserción: `--no-tests fail` la hace una sola población —la del run— sin código propio. Y `certification.yml` ya la usa desde `FIND-785`: la convención existe |
| (b) sólo check estático | No ve los filtros por variable (`$fastGateFilter`) ni prueba comportamiento |
| **(c) elegida: `--no-tests fail` en los 4 sitios + check estático que impide añadir un `-E` sin él + existencia de cada `test(<nombre>)` positivo** | Una sola población (la del run), cero código de ejecución nuevo, y el contrato queda **mecánico**: un `-E` nuevo sin aserción no puede entrar, y un `test(<nombre>)` que no nombra ninguna función se caza **antes de compilar** |

**Por qué no rompe los `-E` legítimos (pre-mortem a):** el check sólo exige el flag y sólo mira átomos **positivos**; `not test(/…/)` (el `$fastGateFilter`) y `not (… )` de grupo se ignoran a propósito — una negación muerta deja la población como **superconjunto**, no como gate apagado. Los tres únicos `test()` del repo que son negaciones son los de `gate-common.ps1:33`, y quedan fuera.

**Techo declarado (corregido tras el review P2-01):**

- un selector armado en una variable no expone **átomos** — de ahí que R1 siga `X+=(-E …)` hasta el `run` que expande `"${X[@]}"`; sin esa mitad las dos lanes de CI eran invisibles (ver §Review C2);
- un `test(/regex/)` no se verifica: solo se cuenta;
- un átomo `test(a::b::c)` se comprueba por su **último** segmento: un prefijo de módulo equivocado pasa;
- el corpus de `fn` es de todo el workspace, **sin `-p`**: un átomo que exista en otro crate satisface R2;
- `sourcesForNextest` escanea `.github/workflows/*.yml`, `dev-tools/**` y el `Justfile` de la raíz. **No** escanea `scripts/**` ni `.github/actions/**`;
- el propio auditor se excluye **entero** (no por rango): un `nextest run` real añadido ahí sería invisible;
- el flag cuenta solo dentro del comando: uno que viva únicamente en un comentario de cola **no** satisface R1.

Lo que sí garantiza: el flag no puede faltar en una invocación de `run` que reciba un selector (inline o por array conocido), y un `test(<nombre>)` positivo literal no puede nombrar algo inexistente.

## Verificación

1. **Demostración (pre-fix):** `-E 'test(no_existe_jamas)'` → salida y exit code pegados en §Premisa. ✅ medido.
2. **Fixtures en las dos direcciones** (`node dev-tools/check-harness-contract.mjs --self-test`): cada regla va en pares — una mitad roja que **muerde** (la forma violatoria) y una verde que **no se puede aflojar** (la misma forma, correcta). Las verdes (`list` que no gatea, comentario de prosa, negación, grupo `not`) pasan en ambos lados por diseño: son guardas anti-falso-positivo, no pruebas de detección.
3. **Check contra el árbol:** `node dev-tools/check-harness-contract.mjs` → rojo sin el fix (lista los 4 sitios sin flag) y verde tras el fix.
4. **Comportamiento:** el comando de cada uno de los 4 sitios, con un filtro que no matchea, sale ≠ 0.

## Resultados (ejecutados)

**1. Check contra el árbol, las dos direcciones.** Con el check nuevo y los 4 scripts **sin tocar** (`HEAD`):

```
FAIL nextest-select (FIND-794)
  - dev-tools/coverage-budget.ps1:77: nextest run with -E but no `--no-tests fail`
  - dev-tools/e2e-memory-cycle.ps1:40: nextest run with -E but no `--no-tests fail`
  - dev-tools/scripts/run-compat-v090-rollback.ps1:21: nextest run with -E but no `--no-tests fail`
  - dev-tools/verify.ps1:115: nextest run with -E but no `--no-tests fail`
4 harness-contract violation(s)
RAW_EXIT=1
```

Tras añadir el flag (y aceptar la forma argv-array `"--no-tests", "fail"`):

```
ok   nextest-select (FIND-794)
harness contract holds (8 checks)
RAW_EXIT=0
```

**2. Self-test: 62/62 → 81/81** (19 fixtures nuevas; 0 regresiones, `RAW_EXIT_SELFTEST=0`).

| Fixture | Dirección roja (muerde) | Dirección verde (no se puede aflojar) |
|---|---|---|
| `-E` sin / con `--no-tests fail` | 1 violación | 0 |
| `cargo llvm-cov nextest -E` | 1 violación | 0 con flag |
| **selector por array + `"${X[@]}"` (forma de las dos lanes de CI)** | 1 violación | 0 con flag |
| `nextest run` que **no** recibe el array del selector | — | 0 (no se marca) |
| `nextest list -E` (mide, no gatea) | — | 0 |
| continuación PowerShell (`backtick`) | 1 violación (una sola orden) | — |
| `--no-tests fail` **sólo en un comentario de cola** | 1 violación | 0 con el flag real + comentario |
| comentario de prosa sobre el flag | — | 0 |
| `test(find782)` / `test(proxy)` | 1 violación | 0 |
| `test(=test_prox)` (exacto) | 1 violación | 0 con el nombre completo |
| `not test(/x/)` y `not (test(a) and test(b))` | — | 0 (sin falso positivo) |
| **átomo positivo FUERA de un grupo `not (…)`** | 1 violación | 0 si está dentro del `not` |
| `1 test(s)` en un `echo` | — | 0 (no es un filtro) |

**2b. Cobertura de las dos lanes de CI, probada por mutación** (antes del P2-01 el check era **ciego** a ellas: el `-E` vive en un array, no en el comando). Con el fichero real en memoria:

```
.github/workflows/certification.yml:          as-is=0  flag borrado → 1 violación (:494)
.github/workflows/snapshot-certification.yml: as-is=0  flag borrado → 1 violación (:201)
```

**3. Comportamiento en runtime (sin compilar Rust; binario ya construido).**

| Comando | Salida | Exit |
|---|---|---|
| `-E 'test(no_existe_jamas)'` | `0 tests run: 0 passed, 1 skipped` + `error: no tests to run` | **4** |
| `-E 'test(no_existe_jamas)' --no-tests pass` | idéntico, sin `error:` | **0** |
| `cargo nextest list -E 'test(no_existe_jamas)'` | 0 líneas | **0** |
| `-E 'test(derived)'` (nombre de **binario**) `--no-tests fail` | `error: no tests to run` | **4** |
| `-E '$fastGateFilter' --no-tests fail` (forma real) | `1 test run: 1 passed` | **0** |

**4. No verificado, declarado:** `coverage-budget.ps1` (build instrumentado completo), `e2e-memory-cycle.ps1` (binario de `vantadb-mcp` en HEAD) y `run-compat-v090-rollback.ps1` (fixture v0.9.0 en `%TEMP%`) no se ejecutaron — disco a 14 GB. El cambio en los tres es **aditivo** (un flag). El review comprobó los prerrequisitos que podrían volverlos rojos: `compat_v090_rollback_fixture_reopened_by_v090_binary` es `#[test]` **sin** `#[ignore]` (`tests/compat_v090.rs:427`) y el runner exporta `VANTADB_V090_CLI` → 1 test corre; `e2e_l0_to_l3_cycle_in_the_shipped_binary` es `#[ignore]` (`vantadb-mcp/tests/l0_l3_cycle_e2e.rs:324`) y el script pasa `--run-ignored ignored-only` → 1 test corre.

## Review

**P2-01, `vanta-review`, contexto fresco, una ronda → `changes-required`: 2 Critical + 3 Required, todos cerrados.**

- **C1 (crítico real, y no era del check): la fila del Backlog.** `git diff --numstat` daba `0 1` (el número del contrato) pero la línea retirada era **`WIRE-20`**, no `FIND-794`: otro carril reescribió `Backlog.md` entre mi edición y el cierre (su fila de vuelta, la mía restaurada). Exactamente el patrón `FIND-753`/`FIND-792` — un gate numérico en verde mientras la operación real no se hizo. **Cerrado:** el fichero se reconstruye **desde `HEAD`** retirando sólo `FIND-794` (restaura `WIRE-20`, no arrastra el retiro ajeno), y el commit va con `-ExpectAbsent` de mi fila + `-ExpectBlob`.
- **C2: R1 era ciego a las dos únicas lanes de CI que decía guardar.** El `-E` vive en `run_args+=(-E "$selector")` y el `run` sólo expande `"${run_args[@]}"`; una regla que exige `-E` en el mismo comando lógico nunca las miraba. Medido por mutación: borrar `--no-tests fail` de `certification.yml:497` o de `snapshot-certification.yml:204` dejaba el check **verde**. **Cerrado:** `nextestSelectorVars` recoge los nombres `X+=(-E …)` y R1 exige el flag a todo `run` que expanda `X`/`"${X[@]}"` (y sólo a ésos: un `nextest run --test security_audit` en el mismo fichero sigue sin marcarse). Las dos lanes pasan de "no miradas" a "vigiladas", con mutación en las dos direcciones.
- **R3: un `--no-tests fail` en un comentario de cola satisfacía R1** (la misma forma que el check existe para impedir). **Cerrado:** `stripTrailingComment` trunca en el primer `#`/`//` fuera de comillas, con fixture pareja.
- **R4: el `return` de `not (` se llevaba por delante átomos positivos de la misma línea** (`not (…) and test(NONEXISTENT)` pasaba). **Cerrado:** rangos de negación por paréntesis (`negationRanges`) en vez de salto por línea, con fixture en las dos direcciones.
- **R5: cifras mal etiquetadas.** «12 invocaciones / 8 en CI» contaba **declaraciones de selector** como si fueran invocaciones. **Corregido:** 8 declaraciones (1 + 7) servidas por **4 invocaciones con `-E` en CI** (2 `list` + 2 `run`) + 4 en `dev-tools` = **8 invocaciones**. Los otros números re-verificados por el revisor: 10 átomos positivos, 3 negados, 62/62 → 74/74 (hoy 81/81 con las fixtures de las correcciones), exit 4/0/0.
- **Opcionales aplicados:** el flag de `coverage-budget.ps1` pasa al **final** del array (no entre los args de nextest y los de llvm-cov); `sourcesForNextest` incluye el `Justfile` de la raíz (corre nextest y era invisible).
- **Opcionales declarados en §Techo:** átomo por último segmento (prefijo de módulo no verificado), corpus sin scope de paquete, `scripts/**` y `.github/actions/**` fuera del escaneo, auto-exclusión del auditor por fichero.
- **NOTICED BUT NOT TOUCHING (review, 8):** `.opencode/commands/harness.md:13` sigue diciendo «7 checks» y enumera siete; ahora son ocho. Fuera de alcance (`.opencode/**`) — lo registra el carril del harness.
- **Resistió sin cambios (re-ejecutado por el revisor):** la §Premisa (honesta y completa, sin sobre-claim), la elección frente al wrapper (una sola población, sin punto donde la aserción use un filtro distinto del que corre), los 4 fixes reales (`1→0` fichero a fichero), y el alcance (ni `src/**`, ni `vantadb-mcp/**`, ni las lanes prohibidas, ni `git-commit-isolated.ps1`).
