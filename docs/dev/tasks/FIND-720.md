---
title: "FIND-720: el comparador del bench canónico nunca tiene baseline (stable_count 0) y main mide el corpus superseded"
kind: task
status: in-progress
description: "El job bench-informational compara labels canonical_p99/* contra un fichero de baseline que solo tiene las familias de analyze (0 de 9 coincidencias) y que nadie refresca: el job pasa a persistir su propio baseline (cache branch-scoped) y lo siembra en el primer run, así has_regression deja de ser inalcanzable"
id: FIND-720
---

# FIND-720: el comparador del bench canónico nunca tiene baseline

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.b
- **Fuente:** fila `FIND-720` de `docs/dev/Backlog.md`
- **Esfuerzo:** 🟠 3-4 h (medición de artefactos + diseño de persistencia + prueba de disparo)
- **Prioridad:** 🟠 Media-alta (es el gate de regresión de la Regla 9; hoy es estructuralmente incapaz de disparar)
- **Tipo:** CI/CD / Observabilidad — sin código de producto
- **Carril:** C
- **Dependencias:** `FIND-317` ✅ (`9ffdd97b9`, movió el job a `schedule`/`dispatch`). En paralelo: `FIND-717` (mismo directorio, fichero distinto: `certification.yml`). Carril R lleno.
- **Creado:** 2026-11-24

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Nadie: `nightly.yml` job `bench-informational` es informativo (`continue-on-error`), no es check requerido de ningún ruleset. Un error aquí no bloquea merges; el riesgo es de **señal** (un gate que parece vivo y no lo está) |
| Callees | `scripts/bench_regression.py` (`extract` / `compare` / `update-baseline`), `actions/cache` (nuevo en el job), `actions/upload-artifact` (`canonical-p99-results`), `benches/canonical_p99.rs` (solo lectura: define los labels) |
| Implicaciones | Ninguna sobre código ni sobre números publicados. `benchmarks/criterion_baseline.json` **no** se edita: sigue siendo el baseline que escribe `certification.yml` job `analyze` (familias `hnsw_pure`/`hybrid_queries`/…). El baseline de `canonical_p99` pasa a vivir en el cache del job (branch-scoped) y se publica como artefacto para auditoría |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la extensión necesaria):** `.github/workflows/nightly.yml` (734 L, completo — el job `bench-informational` ocupa L458-610), `scripts/bench_regression.py` (304 L, completo: `BASELINE_PATH` L23, `load_baseline` L105-108, `compare_command` L111-230, `update_baseline_command` L233-253), `benchmarks/criterion_baseline.json` (65 L, completo), `.github/workflows/certification.yml` L404-710 (`light-benchmarks` corre también `canonical_p99` y sube `target/criterion/`; `analyze` L520-690 hace `update-baseline` + `git add`/`git commit`/`git push` con `contents: write`), `docs/user/operations/BENCHMARKS.md` §11 (L436-457) y §20 (L1507-1545), `docs/dev/operations/CI_POLICY.md` §FIND-317 (L643-675), `.opencode/rules/release-ci.md` (42 L), `docs/dev/tasks/FIND-716.md` (formato canónico), `docs/dev/tasks/FIND-317.md` (§Datos publicados), `benches/canonical_p99.rs` (lectura: `benchmark_group("canonical_p99")` L85, `bench_function` L89 y L156).
- **Referencias hacia dentro de `nightly.yml`:** `ci-gate.yml` (reusable, no lo toca), `notify-failure` (no lista jobs informativos en `needs`, por diseño L47-48), `.github/actions/rust-setup`, `scripts/bench_regression.py`. `docs-gates.yml` cuenta el **número** de workflows (20) contra `CI_POLICY.md`: no cambia (no se crea ni se borra ningún workflow).
- **Referencias hacia fuera:** `actions/cache@55cc8345…` ya usado en `certification.yml:311,827,890`, `ci-rust.yml:316,529`, `release-binaries.yml:41` — mismo pin, no se introduce ninguna acción nueva.
- **Referencias entrantes que quedan stale (fuera de mi alcance de escritura):** `docs/dev/operations/CI_POLICY.md:646` describe el job sin mencionar el baseline; `docs/dev/Backlog.md` fila `FIND-665` (mismo fichero de baseline, `PLACEHOLDER` desde 2026-08-30) queda **parcialmente** cubierta por esta tarea para el lado `canonical_p99`.
- **Veredicto de impacto:** **bajo en corrección, alto en señal.** Se añaden 3 steps y se mueve 1 (el de summary) dentro del job informativo; no se toca `if:`, `concurrency:`, `timeout-minutes:`, `permissions:`, ni el bench. El fichero versionado de baseline y `certification.yml` no se tocan.

## Diagnóstico

### El síntoma, medido en los artefactos (no heredado)

Artefactos reales de `nightly.yml`, bajados **dentro del repo** (trampa #6 del post-mortem 0.9.0: nunca desde un directorio fuera del repo):

```console
$ gh run download 44228771621 -R ness-e/Vantadb -n canonical-p99-results -D target/tmp/find720/44228771621
$ gh run download 44228761011 -R ness-e/Vantadb -n canonical-p99-results -D target/tmp/find720/44228761011
```

Ambos runs son de `nightly.yml`, `event=pull_request`, rama `release-plz-2026-11-18T03-12-19Z` (o sea basados en `main`), `conclusion=success`, 2026-11-18. El job midió 145m02s y 159m39s respectivamente (`gh api .../jobs`).

Contrato mecánico — `jq '.stable_count, .new_benchmarks, .has_regression'` sobre el `bench_comparison_*.json` de cada run (en esta máquina Windows no hay `jq` en el PATH; se ejecutó la **misma expresión** con `python -c`, y el paso equivalente corre en el runner con `jq`, que sí está en `ubuntu-latest`):

```
run 44228771621:
.stable_count   = 0
.new_benchmarks = [{"label": "canonical_p99/insert_100k_1536d", "current_ms": 698162.643},
                   {"label": "canonical_p99/search_1000_queries_1536d", "current_ms": 3958.101}]
.has_regression = false

run 44228761011:
.stable_count   = 0
.new_benchmarks = [{"label": "canonical_p99/insert_100k_1536d", "current_ms": 766984.434},
                   {"label": "canonical_p99/search_1000_queries_1536d", "current_ms": 4260.701}]
.has_regression = false
```

Y el campo que decide qué baseline se usó, en el mismo JSON:

```json
"baseline_commit": "TBH-02-initial-seed"
```

### Las dos hipótesis del ticket, separadas

El ticket pedía confirmar por separado (i) "el baseline no se persiste" y (ii) "el corpus 1536d→100d rompe el matching por label". **Las dos son parcialmente falsas y la causa real es una tercera.**

**(i) "el comparador no recibe baseline / se compara contra sí mismo" → REFUTADA como causa mecánica.**
`baseline_commit: "TBH-02-initial-seed"` solo puede salir de `benchmarks/criterion_baseline.json` (`metadata.commit` L4). Si el fichero no existiera, `load_baseline()` (L105-108) devuelve `{"metadata": {"description": "empty"}, ...}` y el campo saldría `"unknown"` (L118). El comparador **sí** carga el fichero, y no hay ningún camino en `compare_command` que compare contra el propio run. `stable_count: 0` no es "compara contra sí mismo": es que **ninguna de las 2 etiquetas del run existe en el baseline**, así que las 2 caen en el `else` → `new_entries` (L146-150).

Lo que **sí** es cierto de (i): el fichero nunca se refresca en esta lane. `update-baseline` (L233) tiene **un solo caller** en todo el repo: `certification.yml:679` (job `analyze`), que además hace `git add`/`git commit`/`git push` (L682-687). Ese job es uno de los 4 que desde la consolidación CI-08 (`a4c290914`) morían con cero steps — exactamente lo que arregla `FIND-716` — y su escritura quedó congelada. Testigo: `benchmarks/criterion_baseline.json` conserva `created: 2026-08-30` y un `description` que dice literalmente *"Values are PLACEHOLDER … they MUST be refreshed by the first scheduled nightly run"*, y esa refrescada nunca ocurrió.

**(ii) "el cambio 1536d→100d rompe el matching" → REFUTADA como causa del cero de hoy, CONFIRMADA como defecto latente.**
Los dos runs medidos son **1536d** y aun así tuvieron 0 coincidencias: el baseline no contiene **ninguna** etiqueta `canonical_p99/*`, mida lo que mida. Las 9 del fichero son:

```
hnsw_pure/insert_10k · hnsw_pure/search_10k · hybrid_queries/memory {text-only,vector-only,hybrid-rrf} filtered
The Memory Abyss/Point Lookup {Valido,Spurious (Bloom Filter Reject)} · high_density_search/knn_search_768d
logarithmic_spam_friction/50k_spam_mutations
```

Cruzar 2 etiquetas del run contra 9 del baseline da **0/2**. Es disjunción total de etiquetas, no un cambio de dimensionalidad.

Latente y real: la etiqueta **lleva el corpus dentro** (`insert_100k_1536d` en `main`, `insert_100k_glove100d` en `develop` desde `2b821184a`, 2026-10-29). Eso significa que un baseline de un corpus **no puede** compararse contra el otro — y está bien que no pueda (comparar 1536d contra 100d daría una "mejora" falsa de ~4x). El problema no es la etiqueta: es que **hay dos corpus vivos y un solo hueco de baseline**, y el schedule corre solo en la rama por defecto. Hoy: `origin/main:benches/canonical_p99.rs` sigue en 1536d sintético (superseded según `BENCHMARKS.md` §11); `develop` mide GloVe-100d real.

### Causa raíz (la tercera, que explica las dos anteriores)

**Un solo fichero de baseline, dos productores con etiquetas disjuntas, y el productor del hueco que usa esta lane está muerto.**

| Productor | Report que promueve | Etiquetas | Estado |
|---|---|---|---|
| `certification.yml` → `analyze` (L679) | `metricas/benchmarks/benchmark_report_criterion.json` (de `light-benchmarks` + `high-density`) | `hnsw_pure/*`, `hybrid_queries/*`, `high_density_search/*`, y `canonical_p99/*` **cuando corra** | muerto desde CI-08 (`FIND-716`) |
| `nightly.yml` → `bench-informational` | `metricas/benchmarks/benchmark_report_canonical_p99.json` | solo `canonical_p99/*` | **no promueve nada**: `update-baseline` nunca se invoca aquí |

`BASELINE_PATH` está hardcodeado en el script (L23, sin flag `--baseline`), así que las dos lanes comparten un único hueco. El resultado es el observado: la lane de `canonical_p99` compara contra un fichero que, o no tiene sus etiquetas (hoy), o las tendrá con el corpus de `main` y las habrá escrito **otro job en otro runner** (cuando `analyze` revida). En ninguno de los dos casos hay una comparación runner-contra-runner de la misma medición.

### Prueba de que el comparador sí puede disparar (sin tocar código de producto)

El script se copió byte a byte a `target/tmp/find720/scripts/` para que su `BASELINE_PATH` resuelva al directorio de scratch (el fichero versionado del repo **no** se modificó). Cuatro corridas:

**1. Estado actual reproducido localmente** (baseline = el fichero versionado real):

```
$ python scripts/bench_regression.py compare 44228771621/benchmark_report_canonical_p99.json --format json --output cold.json
"baseline_commit": "TBH-02-initial-seed"
.stable_count = 0 · .new_benchmarks = [2] · .has_regression = false
```

Idéntico al artefacto del run: la reproducción es fiel, y confirma que el fichero **sí** se lee.

**2. Baseline sembrado con el run `…1621` (medido) vs report del run `…1011` (medido) — dispara con datos reales:**

```
$ python scripts/bench_regression.py update-baseline 44228771621/benchmark_report_canonical_p99.json
Baseline updated -> …/benchmarks/criterion_baseline.json (2 benchmarks)
$ python scripts/bench_regression.py compare 44228761011/benchmark_report_canonical_p99.json --format json --output cmp_1011.json
.stable_count   = 0
.new_benchmarks = []
.has_regression = true        <-- el gate puede disparar, con números medidos, no sintéticos
```

**3. La dirección inversa, para leer el ruido entre corridas:**

```
insert_100k_1536d        766984.434 -> 698162.643  = -8.97%
search_1000_queries_1536d  4260.701 ->   3958.101  =  -7.10%
```

Dato duro para el diseño: **el spread entre dos corridas reales del mismo bench es 7-9%**, por encima del `REGRESSION_THRESHOLD = 5.0` del script. Consecuencia honesta: tras sembrar el baseline, lo que cambia es la **comparabilidad** (`new_benchmarks: []`), no que `stable_count` pase a ser > 0 — con ±5% y ruido de 7-9%, un run puede caer en `regressions` o `improvements` y dejar `stable_count` en 0 igual. El criterio mecánico correcto es `new_benchmarks` vacío (comparable), no `stable_count > 0`. **No se toca el umbral** (prohibido por el contrato: subir umbrales para tapar): queda como decisión del owner, documentada en §Deuda.

**4. Camino de `develop` (bootstrap con las etiquetas reales del bench + delta grande sintético):**

```
$ python scripts/bench_regression.py update-baseline synth_seed_glove100d.json
$ python scripts/bench_regression.py compare synth_current_glove100d.json --format json --output synth_cmp.json --fail-on-regression
::warning::Detected 2 benchmark regression(s) exceeding 5.0%!
exit=1
canonical_p99/insert_100k_glove100d        190000.0 -> 285000.0  +50.0%  critical
canonical_p99/search_1000_queries_glove100d    900.0 ->   1350.0  +50.0%  critical
"new_benchmarks": [] · "has_regression": true
```

`synth_*.json` es sintético y así queda declarado (magnitudes inventadas sobre las etiquetas reales de `benches/canonical_p99.rs:89,156`); su único propósito es probar el mecanismo, no publicar cifras.

## Fix: el job persiste su propio baseline y lo siembra en el primer run

En `nightly.yml` job `bench-informational` (tres steps nuevos, uno movido):

1. **Restore** — `actions/cache` (mismo pin `55cc8345…` que ya usa el repo) sobre `metricas/benchmarks/baseline`, `key: bench-p99-baseline-${{ github.run_id }}` + `restore-keys: bench-p99-baseline-`. Es un *rolling cache*: la key es única por run (un cache no se puede reescribir) y el prefijo restaura el más reciente. El scope de caches es **por rama** (`main` puede leer las de `main`; `develop` no puede leer las de `main` ni al revés), así que la línea 1536d de `main` y la línea GloVe-100d de `develop` **no pueden contaminarse** aunque compartan nombre.
2. **Adopt** — el script tiene `BASELINE_PATH` fijo y sin flag, así que el baseline restaurado se copia a esa ruta **solo para el step de compare**. Nada se commitea (el job tiene `contents: read`).
3. **Seed** — después del compare: si `total > 0` y `comparable == 0` (todas las etiquetas fueron nuevas), se promueve el report de este run con `update-baseline` y se guarda en la ruta cacheada. Es el único caso en que el baseline se reescribe: un run que **sí** comparó nunca lo pisa (adoptar cada corrida haría invisible una regresión lenta por definición). Regla heredada de AUD-030 (*"skipped when a regression was detected so a bad run is never baked in"*), estrechada.
4. **Summary** — se mueve después del seed y publica `comparable / total / stable / regressions / new` + el corpus detectado en las etiquetas + el baseline usado; con `::warning::` si `comparable == 0`.

Por qué `actions/cache` y no un artefacto (como hace `critcmp-regression`): el patrón de artefacto exige buscar el último `canonical-p99-baseline` con `gh api .../artifacts?per_page=100` y quedarse con el primero, y en este repo eso **no alcanza** — la página más nueva de 100 artefactos no contiene ninguno `canonical-p99-*` (medido: hay que paginar). El cache resuelve el "el más reciente" por prefijo, sin paginación y con scope de rama incorporado. Los artefactos se conservan **para auditoría**: el baseline efectivamente usado se sube dentro de `canonical-p99-results` (30 días), así que la Regla 11 sigue siendo verificable por un tercero.

Y por qué **no** se siembra el fichero versionado con números de `develop`: no existe ninguna medición de `canonical_p99` con GloVe-100d en runner Linux (las dos únicas corridas medidas son 1536d). Rellenarlo con los números de la máquina de desarrollo (176.3 s / p50 0.8036 ms de `BENCHMARKS.md` §11) sería comparar runner contra dev-box, que es el error que el propio §20 documenta con **1.7x-2.4x de spread cross-VM**. El bootstrap del propio runner es la única semilla honesta, y es exactamente la acción que `FIND-665` pide ("capturar un baseline real medido en runner Linux limpio"), automatizada.

## Contrato

```
# 1. El job sigue produciendo el report y la comparación
python scripts/bench_regression.py compare <report> --format json --output <cmp>   # inalterado

# 2. Señal viva: comparable > 0 (new_benchmarks vacío) y has_regression capaz de true
python -c "import json;d=json.load(open('<cmp>'));print(len(d['new_benchmarks'])==0, d['has_regression'])"

# 3. El job no puede perder el baseline silenciosamente
grep -c "bench-p99-baseline-" .github/workflows/nightly.yml    # 2 (key + restore-keys)
```

Estado esperado tras el fix, por corrida:

| Corrida | `new_benchmarks` | `has_regression` | Por qué |
|---|---|---|---|
| 1ª tras el merge (cache frío) | 2 | false | siembra; publica `::notice::Baseline seeded from this run` |
| 2ª en adelante | **0** | **capaz de `true`** | compara runner-contra-runner del mismo corpus |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Bajar los artefactos reales de dos runs de `nightly.yml` y confirmar por separado las dos hipótesis del ticket | ✅ |
| 2 | Probar que el comparador puede dar `has_regression: true` (datos reales cruzados + delta sintético) sin tocar código de producto | ✅ |
| 3 | `nightly.yml`: restore (cache branch-scoped) + adopt + seed + summary con comparabilidad y corpus | ✅ |
| 4 | `BENCHMARKS.md` §11: qué corpus mide cada rama, qué baseline usa cada uno, y el puntero stale a `bench-canonical-p99-informational.yml` | ✅ |
| 5 | Verify (actionlint + docs gates + ocr-review) + review P2-01 + commit local | ✅ |

## Verificación

| Gate | Comando | Resultado |
|---|---|---|
| Workflow | `actionlint .github/workflows/nightly.yml` | ✅ exit 0 |
| Hook pre-commit (staged-area) | `dev-tools/verify_staged.ps1` vía `.githooks/pre-commit` | ✅ `ALL 0 PASS` |
| Markdownlint (hook) | `markdownlint-cli2 --config .github/markdownlint-cli2.yaml` | ✅ `Passed` |
| Docs links | `node scripts/docs/check-links.mjs` | ✅ exit 0 (0 broken; 1 wikilink presupuestado en un histórico) |
| Docs schema | `node scripts/docs/check-docs.mjs` | ✅ exit 0 (FIND-720.md aparece como ORPHAN, igual que FIND-716/717/721 ya aceptados) |
| OCR delegation | `pwsh dev-tools/ocr-review.ps1 -Format json` | ✅ `nightly.yml` cae en el grupo `.github/**`; revisado contra sus reglas (abajo) |
| Funcional end-to-end | `python target/tmp/find720/harness_steps.py` | ✅ `frio=True caliente=True vacio=True` |

**El harness funcional** extrae los `run:` del YAML (no los copia a mano, así no hay
divergencia) y ejecuta la cadena `adopt → compare → seed → summary` con `bash` +
`python3` sobre un sandbox con fixtures. Tres escenarios:

```
FRIO     (cache vacío, baseline versionado de hoy)
  ::notice::No persisted canonical_p99 baseline yet (cold cache) - this run seeds it.
  comparable=0 of total=2
  ::notice::Baseline seeded from this run; comparability starts on the next one.
  ::warning::Nothing was comparable this run (2 labels landed in new_benchmarks) ...
  -> comparable=0 new=2 has_regression=False · baseline persisted: True

CALIENTE (baseline persistido de la corrida anterior)
  persisted baseline: commit=ff53a12c6a9e0c407be88e2a90b5b0cdbea74548 run=44228771621 labels=2
  ::warning::Detected 2 benchmark regression(s) exceeding 5.0%!
  (non-zero exit tolerated: continue-on-error is set)
  comparable=2 of total=2
  baseline kept: nothing to adopt (total=2 comparable=2).
  -> comparable=2 new=0 has_regression=True · baseline persisted: True

VACIO    (el extract no produjo labels)
  comparable=0 of total=0
  baseline kept: nothing to adopt (total=0 comparable=0).
  -> comparable=0 new=0 has_regression=False · baseline persisted: True
```

El segundo caso es el contrato cumplido: **con baseline persistido, 2 de 2 etiquetas se
comparan y `has_regression` dispara** con números medidos en CI (mismo corpus, misma rama).
El tercero es el pre-mortem (a) cerrado por construcción: un report vacío no puede borrar
un baseline bueno.

**Review OCR contra el Rule Group `.github/**`** (autoaplicado, sin LLM — es texto de
reglas, no un veredicto independiente): `pull_request_target` N/A · ningún secret impreso ·
`permissions:` del job intacto (`contents: read`, `issues: write`), no se añade ninguno ·
`actions/cache` pinneado al **mismo SHA completo** que ya usan `certification.yml`/`ci-rust.yml`
(`55cc8345…`, v6.1.0) · ninguna expresión `${{ }}` interpolada dentro de un bloque `run:`
(`github.run_id` solo aparece como input `key:` del cache, `github.sha` va por `env:`) ·
`timeout-minutes: 240` y `concurrency` del job intactos · ningún `continue-on-error` nuevo
sin `# CATEGORY: INFORMATIONAL` · sin sintaxis deprecada (`::set-output` no se usa).

## Deuda declarada

- **Umbral 5% vs ruido 7-9%.** Medido en este ticket (dos corridas reales del mismo bench). Con ese margen, `stable_count` puede quedar en 0 teniendo comparación válida, y el "rojo" puede ser ruido de runner. **No se sube el umbral en esta tarea** (el contrato lo prohíbe explícitamente y es una decisión del owner, no del agente). Opciones para el owner: subir a 15-25% (`--threshold`), o adoptar el patrón A/B same-job que §20 ya documenta para el harness Python.
- **El corpus de `main` es superseded hasta el próximo merge `develop → main`.** No se puede alinear desde acá: `benches/**` está fuera de alcance y el merge es del owner. Mientras tanto el job mide lo que la rama tenga y lo compara **contra sí mismo** (misma rama, mismo corpus, etiqueta que lo garantiza) — degradación honesta, no falsa señal.
- **`CI_POLICY.md:646`** describe el job sin el baseline (fuera del scope de escritura de esta tarea): el texto correcto vive ahora en `BENCHMARKS.md` §11.
- **`FIND-665`** (baseline versionado `PLACEHOLDER`) queda cubierta **solo para `canonical_p99`**; el fichero versionado sigue pendiente para las familias de `analyze` hasta que ese job revida (depende de `FIND-716`, ya cerrado).

## Review (P2-01)

- **Tier (mecánico, `prompts/task.md` Fase 5):** **Fast**. Los paths del diff son
  `.github/workflows/nightly.yml`, `docs/user/operations/BENCHMARKS.md` y
  `docs/dev/tasks/FIND-720.md`; ningún glob adversarial (`docs/api/**`, `src/**`, `src/wal*`,
  serialización, etc.). Diff = CI + docs, 0 líneas de Rust.
- **Gate Fast ejecutado:** `.githooks/pre-commit` → `pre-commit` gates (`markdownlint-cli2`
  Passed) + `dev-tools/verify_staged.ps1` → `ALL 0 PASS` + `actionlint ok`; más
  `check-links`/`check-docs` en exit 0 (§Verificación).
- **Review independiente: NO ejecutado (degradado, declarado).** Esta tarea corrió en un
  sub-agente sin tool `task`, así que **no pudo invocar `vanta-review`** (P2-01 exige
  `reviewer_context ≠ author_context`). No se simula un veredicto: queda para la sesión
  orquestadora o el owner, con la lista corta de lo que hay que mirar:
  1. ¿El *rolling cache* (`key: bench-p99-baseline-<run_id>` + `restore-keys:` prefijo) es
     aceptable como persistencia de un baseline, sabiendo que es invisible en la UI? El
     artefacto de auditoría está en `canonical-p99-results` (30 días).
  2. ¿La regla "sembrar solo cuando `comparable == 0` y `total > 0`" cubre el caso de
     regresión lenta (no re-siembra) sin dejar un baseline viejo para siempre? Hoy no hay
     caducidad del baseline: si el bench mejora 40% de golpe, el baseline se queda atrás.
  3. ¿`BENCHMARKS.md` §11 mezcla idiomas de forma aceptable (prosa nueva en inglés dentro de
     una sección en español)? Es la regla de `AGENTS.md` §Doc Language Split para
     `docs/user/operations/`, pero el resto de la sección no la cumple.
- **Waiver registrado:** `owner: Eros (project owner)` · `ref: FIND-720 — subagente sin tool
  task; review fresh pendiente, gate Fast mecánico completo`.

## Lo que esta tarea NO hace

- **No** toca `benches/canonical_p99.rs` ni `src/**`: el bench y el motor no se modifican.
- **No** edita `benchmarks/criterion_baseline.json`: sigue siendo el baseline de `analyze`.
- **No** toca `.github/workflows/certification.yml` (`FIND-717` en paralelo) ni `ci-rust.yml`.
- **No** sube el umbral de regresión ni cambia la política Q5 (evaluar sin bloquear).
- **No** corre el bench (2h30) ni publica cifras nuevas: los números del diagnóstico son los **ya medidos** en los artefactos de 2026-11-18.
- **No** escribe `Backlog.md`, `plans/**` ni `avance/**`.
- **No** hace push (política de git: el push es acto explícito del owner).
