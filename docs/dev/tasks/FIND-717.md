---
title: "FIND-717: la cobertura de mutación no existe — el job nunca corrió un test"
kind: task
status: in-progress
description: "Job mutation-test: selección acotada de mutantes (superficie fijada por área), tests reales por mutante, presupuesto de 90 min declarado y umbral de 0 sobrevivientes; el inventario de 12.676 mutantes se publica como contexto, no como cobertura"
id: FIND-717
---

# FIND-717: la cobertura de mutación no existe — el job nunca corrió un test

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.b
- **Fuente:** fila `FIND-717` de `docs/dev/Backlog.md` (derivada del triage `FIND-328`, commit `3f7fdd3fa`)
- **Esfuerzo:** 🟡 2-4 h (medición local incluida)
- **Prioridad:** 🟡 Media (no gatea PRs; hoy no hay ninguna señal de calidad de tests por mutación)
- **Tipo:** CI/CD / DevOps — sin código de producto
- **Carril:** C
- **Dependencias:** `FIND-328` ✅ (`3f7fdd3fa`). En paralelo: `FIND-720` (toca `nightly.yml`, no este fichero). Carril R lleno (`723`, `724`, `721`)
- **Creado:** 2026-11-26

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Nadie: `certification.yml` es el tier pesado (schedule 02:00/dom 03:00/lun 06:00 + `workflow_dispatch`). **No es check requerido** de ningún ruleset → un rojo aquí no bloquea merges |
| Callees | El runner (`ubuntu-latest`), `cargo-mutants` (instalado con `taiki-e/install-action`), `cargo test --lib -p vantadb`, y los artifacts `mutation-inventory` / `mutants.out` |
| Implicaciones | Ninguna sobre código. El riesgo es **de señal**: un job llamado "mutation coverage" que no mide entrena a confiar en un verde falso (peor que no tenerlo) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la extensión necesaria):** `.github/workflows/certification.yml` (L1-70 header/triggers/crons, L379-421 el job `mutation-test` completo, L422-431 el job de bench siguiente donde se ancla el `concurrency` de `FIND-716`), `docs/dev/operations/CI_POLICY.md` (L40-66 tabla del fast gate, L381-456 tabla de jobs de Heavy + §"FIND-328 triage", L480-506 subset nightly), `docs/dev/architecture/adr/ADR-0015-coverage-policy.md` (L110-137: la política de cobertura que nombra mutation testing como "next quality lever", sin umbral definido), `.opencode/rules/release-ci.md` (42 L, Regla 5 = `continue-on-error` con `CATEGORY`), `docs/dev/tasks/FIND-716.md` (formato canónico de task file), `docs/dev/tasks/closed/TBH-07.md` (de dónde salió el job original y con qué comando roto).
- **Referencias hacia dentro:** `CI_POLICY.md` L404 (fila de la tabla de jobs) y L432-436 (§category rule del job) describen el estado que esta tarea cambia → se actualizan en el mismo cambio.
- **Referencias hacia fuera:** `taiki-e/install-action@43aecc8d…` (pin v2.83.2, ya usado para nextest/llvm-cov), `actions/upload-artifact@cf430e03…` (pin v7.0.2), `./.github/actions/rust-setup`.
- **Veredicto de impacto:** **bajo en corrección, alto en señal**. El cambio vive en un solo job (7 steps → 6) y en dos secciones de `CI_POLICY.md`. No toca triggers, `needs:`, `concurrency:`, ni ningún otro job.

## Diagnóstico — qué mide hoy el job (nada)

Tres defectos encadenados, los dos primeros ya arreglados por `FIND-328` (`3f7fdd3fa`), el tercero abierto:

1. **Pin inexistente** de `actions/upload-artifact@4e7e4d1b…`: el run moría en "Set up job" (0 steps) durante 9 días.
2. **`--check` no mide**: el manual de cargo-mutants lo define como *"Cargo check generated mutants, but don't run tests"* — compila el mutante y no corre un solo test. `--output-json` tampoco existe. El `|| true` final habría convertido el fallo en verde sin artefacto.
3. **Estado actual (`3f7fdd3fa` → hoy):** el job produce un **inventario** (`cargo mutants --list --json -p vantadb`) y nada más. Es honesto, pero **no hay ninguna señal de calidad de tests por mutación**: cero mutantes ejecutados, cero tests corridos por mutante, cero umbral.

## Investigación — por qué cargo-mutants es caro, con los números de este repo

Medido en el worktree de `HEAD` (`3d55a5840`), `cargo-mutants 27.1.0`, Windows 12 cores / 32 GB:

| Medición | Comando | Resultado |
|---|---|---|
| Mutantes del crate raíz | `cargo mutants --list -p vantadb` | **12.676** mutantes, **231** ficheros con mutantes (257 ficheros escaneados), **3,8 s** (parsea, no compila) |
| Selección incremental vs. último tag | `git diff v0.9.0..HEAD -- 'src/**'` → `--list --in-diff` | **3.741** mutantes (167 ficheros, +37.520/−4.341), `--list` **11,9 s** |
| Reparto por área | `--list` agrupado por prefijo | `src/sdk` 2.465 · `src/storage` 2.047 · `src/index` 1.556 · `src/sdk/search` 646 · `src/wal` 639 · `src/vector` 296 · `src/physical_plan` 255 · `src/text_index` 136 · `src/query` 12 |
| Coste por mutante | `cargo mutants --file src/physical_plan/dedup.rs --in-place --cargo-test-arg=--lib` | ver §Medición |

**El mecanismo del coste** (libro oficial, [how-it-works](https://mutants.rs/how-it-works.html) y [shards](https://mutants.rs/shards.html)): por cada mutante cargo-mutants **recompila el crate mutado y corre la suite de tests completa** de ese paquete. El modelo del propio libro es

```
SHARD_STARTUP + (CLEAN_BUILD + TEST) + (N_TOTAL_MUTANTS / N_SHARDS) * (INCREMENTAL_BUILD + TEST)
```

Con 12.676 mutantes y ~1-3 min de suite por mutante, el crate completo son **semanas de CPU**, no horas. Por eso "cobertura de mutación del crate" no es un objetivo alcanzable en CI con este tamaño de suite: **el presupuesto tiene que fijar el número de mutantes, no el porcentaje del crate.**

**Trampa medida en este repo (`--in-diff`):** `--in-diff` exige que el lado nuevo del diff **coincida carácter a carácter con el árbol**. Con el working tree sucio (otros carriles editando `src/`), cargo-mutants aborta:

```
ERROR Diff content doesn't match source file: src/parser/mod.rs line 1707
diff has:   "        // A trailing statement terminator is tolerated (lenient, same as"
source has: "                assert_eq!(sel.limit, Some(5));"
EXIT=5
```

Código de salida **5** = "the new side of the `--in-diff` diff doesn't match the text in the tree" ([exit-codes](https://mutants.rs/exit-codes.html)). Un job que use `--in-diff` sobre un checkout limpio funciona; sobre un árbol con ediciones locales falla por la razón equivocada. Documentado para que nadie lo reintroduzca a ciegas.

## Spec — qué mide el job a partir de ahora

**Decisión (con los números de arriba):** el crate completo y el diff-desde-el-tag **no caben en CI** (12.676 y 3.741 mutantes, respectivamente). La salida válida es **superficie fijada por área** (`--file`, uno de los mecanismos que documenta el libro para acotar), con umbral duro y presupuesto declarado. No es cobertura del crate y el job lo dice en su propio summary.

| Eje | Contrato |
|---|---|
| **Selección** | `--file` con una lista fija de ficheros (superficie), mantenida en `env.MUTANTS_FILES` del job. Determinista: los mismos mutantes en cada run, así el umbral no parpadea |
| **Qué corre** | `cargo-mutants` **sin `--check`**: por cada mutante recompila y corre `cargo test --lib -p vantadb` (passthrough `-- --lib`). `--lib` porque los mutantes viven en `src/` y sus tests unitarios viven en el mismo módulo; la suite de integración completa por mutante multiplicaría el coste sin cambiar la señal |
| **Umbral** | **0 sobrevivientes** en la superficie (exit `2` de cargo-mutants → job rojo). Un sobreviviente es una brecha de test, no ruido. Timeouts (exit `3`, default 5× el baseline) y baseline roto (exit `4`) también fallan, con mensaje propio. La superficie medida tiene 9 mutantes y **2 sobrevivientes reales**: se excluyen por nombre (`MUTANTS_KNOWN_SURVIVORS`) para que el umbral signifique "ningún sobreviviente **nuevo**"; ver §Medición y §Deuda |
| **Presupuesto** | `timeout-minutes: 105` en el job, **90** en el step de mutación, `--build-timeout 900`, `--in-place` (sin copiar el árbol) y **secuencial** (`--jobs` es incompatible con `--in-place`). Sin `--timeout` fijo: ver §Review R2 |
| **Anti-verde-falso** | Tres guards de `::error::` + exit 1 antes de mutar: superficie `0` (un filtro que no matchea nada no es un gate), superficie `< MUTANTS_MIN_SURFACE` (6: la superficie no se recorta a un fichero) y superficie `> MUTANTS_MAX_SURFACE` (12: no se agranda sin medir) |
| **Contexto** | El inventario del crate (12.676) se sigue publicando y el summary imprime `surface` junto a `crate`, para que nadie lea el verde como cobertura del crate |

## Medición (evidencia de que el job puede fallar y de que puede pasar)

Dos corridas en un **worktree limpio** (`git worktree add --detach $TEMP/m717-wt HEAD`, `HEAD` = `3d55a5840`),
`cargo-mutants 27.1.0`, Windows 12 cores / 32 GB, sin sccache. El comando es el del job:

```
cargo mutants -p vantadb --file <superficie> --in-place --cap-lints=true --build-timeout 900 -- --lib
```

(La corrida de calibración se lanzó con `--cargo-test-arg=--lib` y `--timeout 90` antes de corregir R1/R2
del review; son el mismo passthrough, y ese `--timeout` fijo es justamente el defecto que R2 documenta.)

### 1. Calibración de coste — un fichero, 4 mutantes

```
Found 4 mutants to test
ok       Unmutated baseline in 1157s build + 63s test
caught   src/physical_plan/dedup.rs:47:9: replace <impl PhysicalOperator for PhysicalDedup<'_>>::open -> Result<()> with Ok(()) in 229s build + 41s test
caught   src/physical_plan/dedup.rs:53:9: replace <impl PhysicalOperator for PhysicalDedup<'_>>::next -> Result<Option<UnifiedNode>> with Ok(None) in 234s build + 64s test
caught   src/physical_plan/dedup.rs:53:9: replace <impl PhysicalOperator for PhysicalDedup<'_>>::next -> Result<Option<UnifiedNode>> with Ok(Some(Default::default())) in 192s build + 42s test
caught   src/physical_plan/dedup.rs:63:9: replace <impl PhysicalOperator for PhysicalDedup<'_>>::close -> Result<()> with Ok(()) in 256s build + 72s test
4 mutants tested in 39m: 4 caught
```

Lectura: 4 mutantes **detectados**, 4 **muertos** (`caught`), 0 sobrevivientes → exit 0. El coste es el
**build** (1157 s de baseline en frío + 192–256 s por mutante), no el test (41–72 s). De ahí que el
presupuesto se exprese en mutantes y no en porcentaje del crate.

> **Corrección posterior (importante):** ese `caught` de `dedup.rs:63` era **falso**. La corrida de la
> superficie (abajo) lo midió como `MISSED` y el código lo confirma: `MockScan::close` es un no-op, así que
> mutar `PhysicalDedup::close` a `Ok(())` es invisible para los 3 tests. Un test flaky de otro módulo
> fallaba por carga de CPU y cargo-mutants lo contó como "matado". Moraleja registrada abajo: **bajo carga,
> un test flaky puede fabricar un falso `caught`** — el baseline verde no protege contra eso.

### 2. La superficie exacta que fija el job — 9 mutantes, 2 sobrevivientes reales

Tres corridas de la misma superficie (`dedup.rs` + `index/search/pool.rs`), mismo árbol, mismo binario:

**(a) Con `--timeout 90`** (el valor que traía el primer borrador):

```
Found 9 mutants to test
ok       Unmutated baseline in 150s build + 43s test
caught   src/physical_plan/dedup.rs:47:9: ... ::open -> Result<()> with Ok(()) in 220s build + 69s test
TIMEOUT  src/physical_plan/dedup.rs:53:9: ... ::next -> Result<Option<UnifiedNode>> with Ok(None) in 382s build + 90s test
TIMEOUT  src/physical_plan/dedup.rs:53:9: ... ::next -> Result<Option<UnifiedNode>> with Ok(Some(Default::default())) in 443s build + 90s test
```

Los dos `TIMEOUT` son **falsos**: en la corrida de calibración esos mismos dos mutantes murieron
(`caught ... 234s build + 64s test` y `192s build + 42s test`). `--timeout` acota solo la fase de test, y
90 s sobre un baseline de 43–63 s es 1.4–2×: con la máquina cargada (tres agentes compilando en paralelo)
el test cruzó el umbral y cargo-mutants lo clasificó como `TIMEOUT` (exit 3). **De aquí sale el fix**:
el flag se quitó y el default (5× el baseline) hace el trabajo.

**(b) Ya sin `--timeout`, primera corrida:**

```
test result: FAILED. 2633 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 47.47s
failures:
    cli_handlers::mcp_init::tests::test_brain_profile_degraded_warning_on_probe_timeout
*** result: Failure(101)
ERROR cargo test failed in an unmutated tree, so no mutants were tested
EXIT=4
```

Exit **4** = `baseline` rojo. El job **se niega a mutar** y lo dice con el mensaje correcto. El fallo es un
test sensible a timeouts (`..._probe_timeout`) que se cae bajo carga de CPU sostenida — preexistente y ajeno
a este cambio. Esto es exactamente el guard que impide el falso verde: si el baseline está roto, con
`--baseline=skip` **todos** los mutantes se verían "caught" y el job entregaría un 100 % muerto falso. Por
eso `--baseline` se queda en `run`.

**(c) La corrida que decide la superficie — exit 2:**

```
Found 9 mutants to test
ok       Unmutated baseline in 164s build + 74s test
 INFO Auto-set test timeout to 372s
MISSED   src/physical_plan/dedup.rs:63:9: replace <impl PhysicalOperator for PhysicalDedup<'_>>::close -> Result<()> with Ok(()) in 273s build + 36s test
MISSED   src/index/search/pool.rs:16:5: replace take_nl -> NeighborVec with Default::default() in 183s build + 32s test
9 mutants tested in 43m: 2 missed, 7 caught
EXIT=2
```

**Exit 2 = el job se pone rojo.** 9 mutantes detectados, **7 muertos**, **2 sobrevivientes**. Los dos son
brechas reales, confirmadas leyendo el código, no ruido:

| Mutante | Por qué sobrevive | Test que lo mata |
|---|---|---|
| `dedup.rs:63` `PhysicalDedup::close -> Ok(())` | `MockScan::close` es un no-op y los 3 tests sólo llaman `dedup.close().unwrap()` al final: nada observa que el `close` se propague al hijo | un flag `Rc<Cell<bool>>` en el mock que `close()` ponga a `true`, y `assert!` después de `dedup.close()` |
| `pool.rs:16` `take_nl -> Default::default()` | el test sólo comprueba `v.is_empty()`, que un vec nuevo vacío también cumple: nunca verifica que el buffer venga **reciclado** del pool | después de `give_nl(v)`, `assert_eq!(NL_POOL.with(\|p\| p.borrow().len()), 0)` tras un `take_nl()`, o comprobar `v.capacity() >= 40` (el buffer reciclado) |

**Decisión:** el job arranca con esos dos mutantes **excluidos por nombre** (`--exclude-re`, en
`MUTANTS_KNOWN_SURVIVORS`) y el umbral sigue siendo **0 sobrevivientes sobre los 7 restantes**. No es un
`|| true`: la exclusión es un regex de dos nombres exactos, está impresa en el summary del job y en
`CI_POLICY.md`, y el comentario del workflow dice qué test la borra. Sin ella el gate nace rojo y el tier
semanal se vuelve ruido (el patrón que produjo el rojo invisible de 9 días que arregló `FIND-328`). La
alternativa —dejarlo rojo hasta que existan los dos tests— queda declarada abajo: la decisión es del owner.

**(d) La corrida final — el comando exacto del job, exit 0:**

```
Found 7 mutants to test
 INFO Auto-set test timeout to 168s
ok       Unmutated baseline in 230s build + 33s test
7 mutants tested in 35m: 7 caught
WALL_S=2124.7 EXIT=0
```

7 mutantes detectados, **7 muertos**, 0 sobrevivientes → **exit 0**. 35 min de step (baseline 230 s de build
+ 33 s de test en caliente, ~4.4 min por mutante) — dentro de los 90 min declarados con margen. El timeout
auto (168 s = 5× 33 s) muestra por qué el flag fijo sobraba.

**Resumen de los cuatro códigos de salida, medidos, no supuestos:** `0` (7/7 muertos, 35 min) · `2` (9
probados, 2 sobrevivientes reales) · `3` (dos falsos `TIMEOUT` con el `--timeout 90` del primer borrador) ·
`4` (baseline rojo por un test flaky ajeno). El job puede pasar y puede fallar por cada una de las razones
que declara.

### 3. Qué prueba esto y qué no

- **Prueba que el job corre tests y puede fallar:** exit 2 se alcanzó de verdad (9 mutantes, 2 sobrevivientes
  reales), exit 3 y 4 también quedaron demostrados (timeout mal configurado → falsos `TIMEOUT`; baseline con
  un test flaky → `no mutants were tested` + exit 4). El modo "verde sin medir" no puede volver: `--check`
  fuera, `|| true` fuera, superficie vacía o fuera de presupuesto = `::error::` + exit 1 en segundos.
- **Prueba que el job puede pasar:** 7 de 9 mutantes mueren hoy y, con los dos sobrevivientes excluidos y
  nombrados, la corrida final del comando exacto del job dio **7 caught en 35 min, exit 0** (§2d).
- **No prueba el wall-time en `ubuntu-latest`:** la medición es Windows 12c sin sccache. El budget usa el
  número lento con headroom (35–43 min medidos vs 90 declarados) y `MUTANTS_MIN/MAX_SURFACE` corta en
  segundos si alguien toca la superficie sin volver a medir.
- **Lección que costó una corrida:** un test flaky bajo carga de CPU puede convertir un `MISSED` en
  `caught` (el `dedup.rs:63` de la corrida de calibración). El `--baseline=run` protege el baseline, no los
  mutantes: la señal solo es confiable con la suite determinista.

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Reescribir el job `mutation-test`: superficie fijada, tests reales, umbral 0 sobrevivientes, guards de presupuesto, budget declarado | ✅ |
| 2 | `CI_POLICY.md`: fila de la tabla de jobs + §"FIND-328 triage" (category rule) + §"FIND-717" con el contrato, los números y las trampas | ✅ |
| 3 | Verify mecánico: `actionlint` exit 0 + medición local de la superficie (9 mutantes, 7 muertos, 2 sobrevivientes reales) pegada arriba | ✅ |
| 4 | Review P2-01 (tier Fast) + commit local | ✅ |

## Deuda — lo que queda fuera y por qué

- **Los dos sobrevivientes excluidos son deuda de tests, no de CI.** El gate no los mide (están en
  `MUTANTS_KNOWN_SURVIVORS`), y cada uno necesita ~10 líneas de test en el `mod tests` del propio módulo
  (`src/**` está fuera del alcance de escritura de esta tarea):
  - `src/physical_plan/dedup.rs` — hacer que el mock registre el `close` (`Rc<Cell<bool>>`) y aseverar que
    `PhysicalDedup::close()` lo propaga.
  - `src/index/search/pool.rs` — aseverar que `take_nl()` devuelve un buffer **reciclado** (capacidad ≥ 40
    tras `give_nl`) y no uno nuevo, además de que el pool baja en 1.
  Al aterrizar cada test, se borra su entrada del regex en el mismo PR. Ese día el umbral cubre 9/9.
- **La alternativa que NO se tomó, y por qué.** Dejar el gate rojo hasta que existan esos dos tests reporta
  la verdad cruda, pero un tier semanal rojo desde el primer día se ignora (es el modo de fallo del rojo de
  9 días de `FIND-328`). Si el owner prefiere la señal cruda: vaciar `MUTANTS_KNOWN_SURVIVORS` en el
  workflow y el job pasa a `exit 2`/rojo con los dos mutantes nombrados en el artefacto.
- **La superficie es pequeña a propósito.** Cubre lo que cabe en 90 min; crecerla es una decisión con medición nueva (`--list` de la superficie propuesta + coste por mutante), no una edición de YAML. Candidatos ya contados: `src/sdk/search` (646), `src/vector` (296), `src/physical_plan` (255).
- **Sharding** (`--shard k/n` + matrix, [mutants.rs/shards](https://mutants.rs/shards.html)) es el mecanismo oficial para crecer la superficie sin crecer el wall-time; no se usa hoy porque cada shard paga un build en frío y la superficie actual no lo necesita.
- **`--iterate`** ("skip mutants that were caught in previous runs") convertiría el job en incremental de verdad, pero requiere persistir `mutants.out` entre runs (cache/artifact + download). Sin eso, cada run repite la misma superficie.
- **`--in-diff`** (la variante incremental que pide el plan) queda documentado como alternativa válida con su trampa: exit 5 si el árbol no coincide con el diff, y 3.741 mutantes medidos en el diff del último minor — no cabe.
- **Un test flaky puede fabricar un falso `caught`** (medido: `test_brain_profile_degraded_warning_on_probe_timeout` cae bajo carga y `dedup.rs:63` apareció como `caught` cuando en realidad sobrevive). El `--baseline=run` protege el baseline, no los mutantes; la señal solo es confiable con la suite determinista. Ese test debería ser fila `FIND-*` propia (fuera del alcance de escritura de esta tarea: `Backlog.md` está prohibido).
- **Cobertura del crate:** nadie debe leer este job como "cobertura de mutación de VantaDB". Es una superficie de 2 ficheros (9 mutantes, 7 bajo umbral) con umbral duro. El summary lo imprime.

## Lo que esta tarea NO hace

- **No** toca los jobs de bench (`light-benchmarks`, `high-density`, `analyze`, `critcmp-regression`): `FIND-716` ya cerró.
- **No** toca `nightly.yml` (`FIND-720`), ni triggers, ni `needs:`, ni `concurrency:`.
- **No** añade `continue-on-error` (Regla 2) ni reintroduce `|| true`.
- **No** edita `src/**`, `tests/**`, `dev-tools/**`, `Backlog.md`, `plans/**` ni `avance/**`.
- **No** define un umbral porcentual inventado: el umbral es 0 sobrevivientes sobre un conjunto medido.
- **No** hace push (política de git: el push es acto explícito del owner).

## Review (P2-01)

- **Reviewer:** `vanta-review` (contexto fresco, sesión `ses_def677334ffeIiEy0jqIKVtHOR`; sin acceso a la sesión que implementó).
- **Tier:** Fast (paths `.github/workflows/**` + `docs/dev/operations/**` + `docs/dev/tasks/**`; ningún glob adversarial de `prompts/task.md` Fase 5). Aun así se pidió review de agente distinto, no solo el verify mecánico.
- **Veredicto inicial:** 🔴 `changes-required` — 1 Critical · 5 Required · 3 Optional · 2 Nits. Lo que el reviewer validó del enfoque: las tres prohibiciones se respetan de verdad (sin `--check`, sin `|| true`, sin `continue-on-error` nuevos), `actionlint` exit 0 sobre el blob revisado, `surface=9` / `crate=12.676` verificados con `--list`, mapeo de exit codes contra la página oficial, `--in-place` según la guía de CI, y bash correcto (`set -euo pipefail`, `files+=(...)`, `set +e`/`rc=$?`/`set -e`, `exit "$rc"`).

### Lo corregido (una ronda)

- **C1 — el commit no estaba en `develop`.** Cierto en el momento del review: este carril convive con otros agentes que commitean en el mismo árbol, y un `git reset` de otro carril se llevó mi commit (contenido intacto en el working tree). Re-creado y verificado con `git merge-base --is-ancestor <sha> HEAD` después del último cambio, no antes.
- **R2 — `--timeout 90` era una fábrica de falsos negativos (y se materializó).** El flag acota **solo la fase de test**; el baseline medido es 63 s, así que 90 s es 1.43×. En la corrida de 9 mutantes, **dos mutantes que en la corrida anterior murieron en 41–64 s aparecieron como `TIMEOUT` a los 90 s** con la máquina cargada:
  ```
  TIMEOUT  src/physical_plan/dedup.rs:53:9: replace ... ::next ... with Ok(None) in 382s build + 90s test
  TIMEOUT  src/physical_plan/dedup.rs:53:9: replace ... ::next ... with Ok(Some(Default::default())) in 443s build + 90s test
  ```
  Esos mismos dos mutantes: `caught ... 234s build + 64s test` y `caught ... 192s build + 42s test` en la corrida previa. Fix: **se quitó `--timeout`**; el default (5× el baseline) se adapta al runner. El mensaje de exit 3 ahora dice "default 5x baseline test time" en vez de afirmar un hang.
- **R3 — sin timeout de build en la combinación más riesgosa.** Con `--in-place` no hay build timeout, y `--cap-lints=true` puede colgar rustc. Fix: `--build-timeout 900` (≈4× el build medido por mutante, 192–443 s).
- **R4 — §Medición vacía en el momento del review.** Ahora contiene tres corridas con comando y salida literales (calibración de 4 mutantes, la corrida que destapó R2, y la corrida final de la superficie con el comando exacto del job).
- **R5 — números contradictorios entre documentos.** Unificado en 2 ficheros / 9 mutantes (7 bajo umbral) / 90 min step / 105 min job / `MUTANTS_MIN_SURFACE=6` / `MUTANTS_MAX_SURFACE=12`; borrada la referencia a un `MUTANTS_SURFACE_BUDGET_MIN` que nunca existió; la calibración declara que usó `--cargo-test-arg=--lib` (equivalente al `-- --lib` del job).
- **R6 — `CI_POLICY.md` se contradecía.** La fila histórica de `FIND-328` ahora dice explícitamente que `FIND-717` la supersede, y el bullet del subset nightly ya no dice "non-gating by design".
- **O7 — piso de superficie.** Añadido `MUTANTS_MIN_SURFACE` (6 con la superficie medida de 7): sin él, recortar `MUTANTS_FILES` a un fichero dejaba el job en verde midiendo casi nada.
- **O8 — `MUTANTS_MAX_SURFACE=9` daba rojo espurio.** Una rama nueva en `dedup.rs` o `pool.rs` sube el conteo legítimamente. Margen a **12** y el mensaje de error ahora nombra las dos causas (código nuevo en los ficheros vs. edición del presupuesto).
- **O9 — 90 min de job con 85 comprometidos.** Job a **105 min** para que checkout + setup + install + inventory quepan sin comerse el `if: always()` del upload.
- **N10 — la cifra "~60 MB" del JSON no tenía fuente.** Eliminada; queda la razón que sí verifica ("nothing consumes it") y la decisión (texto).
- **N11 — nombre del artefacto.** `mutation-inventory` → **`mutation-results`** (ahora publica `mutants.out/` además del inventario; ningún consumidor lo referenciaba).

### Lo que la medición agregó después del review

La corrida de la superficie (43 min) encontró **2 sobrevivientes reales** en los dos ficheros fijados, y el
contrato se ajustó a eso, no al revés: los dos se excluyen por nombre (`MUTANTS_KNOWN_SURVIVORS`), el umbral
sigue en 0 sobrevivientes sobre los 7 restantes, el piso baja de 8 a 6 y el summary imprime la exclusión.
Los dos son deuda de tests con la aserción exacta escrita en §Deuda, no un `|| true` con otra forma. También
quedó demostrado que un test flaky bajo carga puede fabricar un falso `caught` (calibración vs. superficie).

### Lo que el reviewer dejó abierto a propósito

- No se añadió sharding ni `--iterate` (deuda declarada arriba), ni se agrandó la superficie: la decisión es el contrato de esta tarea.
- El wall-time real en `ubuntu-latest` sigue sin medir: la primera corrida programada es la medición que falta, y el step tiene 90 min con el número lento de referencia.
