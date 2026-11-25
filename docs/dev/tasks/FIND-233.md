---
title: "FIND-233 — cross-VM delta: decide, atribuir o declarar no-atribuible"
kind: task
status: active
description: "El instrumento de perf-bench ya no está en develop y el A/B nunca se invocó; el ruido intra-job (13-42%) es mayor que el delta a atribuir (+10-19%), así que la decisión es aceptar el ruido con coste cero y escalar a un dispatch costeado."
---

# FIND-233 — cross-VM delta: decide, atribuir o declarar no-atribuible

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §6.c (y §7 fila 3.2)
> **Ruta:** `vanta-tuner` · **Carril:** C · **Appetite:** 4-6h
> **Dependencias:** `FIND-232` ✅ (bandas calibradas a colapsos) · `FIND-720` ✅ (`aea7b1221`, baseline canónico — no se toca) · `FIND-317` ✅ (canónico movido a schedule)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `docs/user/operations/BENCHMARKS.md` (§20 + §11 + §13 + §2/§3), `.github/workflows/perf-bench.yml` @ `main` (vía `git show`, no existe en `develop`), `benchmarks/compare_baseline.py` @ `main` (referencia), `benchmarks/python_baseline.json` @ `main`, `docs/dev/workflow/perf-bench-40.md`, `benchmarks/README.md`, `docs/dev/tasks/FIND-233.md` @ `92a17144f` (intento previo, 2026-10-04), `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §6.c, `.opencode/tasks…` n/a.
- **Referencias entrantes (rg `compare_baseline|perf-bench`):** `benchmarks/README.md:71` (retiro), `docs/specs/SPEC.md:140` (**stale** — describe el gate como operativo, ver §Colaterales), `docs/dev/workflow/TRIGGERS.md:64`, `docs/dev/workflow/RULES.md:50,52`, `docs/dev/workflow/FAQ.md:23`, `docs/dev/workflow/perf-bench-40.md` (runbook vivo), `CHANGELOG.md`, `docs/dev/Backlog.md:215`, `docs/dev/plans/backlog-release-v0.10.0.md:56`.
- **Referencias salientes:** ninguna nueva. El cambio es docs-only: no toca `benches/`, `src/`, `vantadb-ts/`, `.github/workflows/`, `.githooks/`.
- **Veredicto de impacto:** **LOCALIZADO / docs-only.** Sin efecto en el motor, en el baseline canónico (§11, `FIND-720`) ni en ningún gate (el workflow no existe en `develop`).

## Evidencia — auditoría del instrumento (2026-11-25)

Todas las cifras de abajo salen de `gh` contra `ness-e/Vantadb` el 2026-11-25, no de estimaciones.

| Pregunta | Respuesta | Comando |
|---|---|---|
| ¿Existe el workflow? | **Solo en `main`.** Ausente en `develop` desde `255db8208` (de-bloat 2026-10-25); `compare_baseline.py` y `python_baseline.json` se fueron con él | `git ls-tree develop --name-only .github/workflows/` → ninguna entrada `perf`; `git show main:benchmarks/python_baseline.json` → sigue con la metadata de re-baseline del 2026-10-03 |
| ¿Se ha usado el A/B? | **Nunca.** 1 run en total, `event: push`, 0 `workflow_dispatch` | `gh api repos/ness-e/Vantadb/actions/workflows/364848308/runs --jq .total_count` → `1` |
| ¿Cuál run? | `40592702083` · `2026-10-27T22:10:15Z` · `success` · `main` @ `3e1e21b3` · job **6m12s** | `gh run list --workflow perf-bench.yml --limit 10 --json conclusion,createdAt,headSha` |
| ¿Se puede re-derivar la evidencia de `FIND-232`? | **No.** Los 4 runs citados devuelven `HTTP 404`: artefactos y logs borrados | `gh run view 37088714140 \| 37089421873 \| 37089581213 \| 36093538630 --json databaseId` → `404 Not Found` |

Contrato mecánico (salida literal, 1 fila):

```json
[{"conclusion":"success","createdAt":"2026-10-27T22:10:15Z","headSha":"3e1e21b33639cc6aeb99304e065e22d34a2c704b"}]
```

## Noise floor medido intra-job (run 40592702083)

Tres sub-runs del mismo perfil (`10000 × 128d`, `1000` queries) dentro de **un** job, **una** VM, **un** commit. Es el suelo de ruido más generoso que el instrumento puede dar: entre columnas solo cambia el scheduler.

| Métrica | sub-run 1 | sub-run 2 | sub-run 3 | spread (max/min) |
|---|---:|---:|---:|---:|
| ingest throughput (rec/s) | 230.19 | 243.59 | 243.37 | **5.8%** |
| `query_vector.p50` (ms) | 2.2630 | 2.2308 | 1.9384 | **16.7%** |
| `query_vector.p95` (ms) | 2.6754 | 2.5325 | 2.3638 | **13.2%** |
| `query_vector.p99` (ms) | 2.8245 | 3.0010 | 2.9553 | **6.2%** |
| `query_hybrid.p50` (ms) | 5.4610 | 6.1058 | 4.5494 | **34.2%** |
| `query_hybrid.p95` (ms) | 6.9599 | 7.3392 | 5.1622 | **42.2%** |
| `query_hybrid.p99` (ms) | 11.7452 | 12.2807 | 10.5246 | **16.7%** |
| `query_text.p50` (ms) | 0.0047 | 0.0047 | 0.0047 | 0% |

Reproducir: `gh run view 40592702083 -R ness-e/Vantadb --log` (tres líneas `Vector HNSW` / `Hybrid Fusion`, una por sub-run). Entorno: GitHub-hosted `ubuntu-latest` (Azure `eastus2`), comparador mediana-de-3.

**Lectura:** un mediana-de-3 hereda 13-42% de spread en vector/hybrid **con la máquina constante**. La banda que gatea la familia *quarantined* (warn >15%) está **por debajo** de ese suelo — por eso ese run emitió 2 warnings (`query_text.p95 +62.1%`, `p99 +17.5%`) sobre números de <10 µs donde el jitter absoluto (~3 µs) es toda la señal.

## Atribución — la aritmética

Delta a atribuir (Backlog, `36093538630` 2026-09-25 ref `114f55f0` vs runs del 2026-10-03): `query_vector` p50 **+15.7%** / p95 **+19.3%**, `query_hybrid` **+10.7%** / **+8.5%**, `insert` **−2%**, `rebuild` **−1.3%**, `text` **+8-13%**.

| Delta | Valor | Suelo intra-job | Suelo cross-VM | Veredicto |
|---|---:|---:|---:|---|
| `vector.p50` | +15.7% | 16.7% | 70-140% | dentro del ruido |
| `vector.p95` | +19.3% | 13.2% | 70-140% | 1.46× el intra-job; 3.6× por debajo del cross-VM → dentro |
| `hybrid.p50` | +10.7% | 34.2% | 70-140% | dentro del ruido |
| `hybrid.p95` | +8.5% | 42.2% | 70-140% | dentro del ruido |
| `insert` | −2% | 5.8% (throughput) | 70-140% | dentro del ruido |
| `text` | +8-13% | µs-scale | 70-140% | dentro del ruido |

La comparación original cruza **nueve días y runners distintos** → el suelo aplicable es el cross-VM (`1.7x-2.4x` = +70%..+140%). Todos los deltas quedan 4-9× por debajo.

**Veredicto: NO atribuible.** Tampoco "es la VM": lo honesto es que el instrumento no separa código de máquina a esa amplitud, y el A/B que sí podría (`ab_ref=114f55f0`) **nunca se despachó**.

## Decisión (2026-11-25)

| Opción | Veredicto | Coste |
|---|---|---|
| **(a) Calibración de CPU antes del bench** | **Rechazada** | **+~1 min por corrida, recurrente** (est.: paquete de 111 kB + un pase `sysbench cpu --time=10` por brazo ×3) = +15-22% de un job de 4.5-6.2 min. Normaliza **solo CPU**: `insert.p99` es fsync-bound (16.8× cross-VM) y todo número cruza PyO3 → un factor no modela 3 regímenes. Y no es validable hoy: los runs multi-VM contra los que se calibraría están borrados. |
| **(b) A/B same-job como camino único** | **Método correcto, no adoptado como camino** | **11-15 min por invocación** en un runner como el de hoy (base 6m12s + 5-9 min de A/B), opt-in. Adoptarlo como camino único obliga a resucitar un workflow que `FIND-317` retiró y el de-bloat borró: re-litigar una decisión cerrada sin evidencia nueva. |
| **(c) Aceptar el ruido; bandas como están** | **ELEGIDA** | **0 min recurrentes, 0 líneas de código.** Las bandas quedan como las calibró `FIND-232`; en `develop` ya no hay gate que bandear y no hay datos nuevos para re-derivarlas. |

**Escalada, si la atribución hay que responderla** (sin cambio de código, nada recurrente):

```bash
gh workflow run "PERF: Benchmarks — Python Integration" -R ness-e/Vantadb \
  --ref main -f ab_ref=114f55f0 -f size=10000 -f queries=1000 -f dim=128
gh run watch <id> --exit-status
```

Presupuestar **11-15 min** y calibrar las bandas A/B en la misma campaña con un dispatch same-SHA (`-f ab_ref=$(git rev-parse main)`), que mide el ratio same-job directamente.

**Si la señal fina pasa a ser load-bearing**, el instrumento a construir no es un reloj calibrado: la medición determinista por conteo de instrucciones (Valgrind, vía [`iai-callgrind`](https://docs.rs/iai-callgrind/latest/iai_callgrind/), que documenta "consistent measurements even in virtualized CI environments") esquiva el factor máquina en vez de normalizarlo. Decisión del dueño de §11 — no se hace acá.

## Steps

### Step 1 — Auditoría del instrumento con `gh` — ✅ COMPLETED
- Ejecutado el contrato mecánico (`gh run list --workflow perf-bench.yml`), el conteo por API, la existencia del archivo en `main` vs `develop` y el `404` de los 4 runs citados. Números en §Evidencia.

### Step 2 — Noise floor intra-job desde el log del único run vivo — ✅ COMPLETED
- `gh run view 40592702083 --log` → 3 sub-runs extraídos, spread calculado. Números en §Noise floor.

### Step 3 — Aritmética de atribución + decisión con coste — ✅ COMPLETED
- §Atribución y §Decisión. Sin estimaciones presentadas como mediciones: los costes de (a) y (b) van marcados como est. y anclados a la duración **medida** del job (6m12s).

### Step 4 — Instrumentación de la decisión en BENCHMARKS.md §20 — ✅ COMPLETED
- Subsección nueva `### Operator state and the attribution verdict (2026-11-25, FIND-233)` + corrección del banner de retiro (decía "the workflow no longer exists" — cierto en `develop`, falso en `main`). Sin tocar ninguna cifra medida (Regla 11).

## Review (P2-01)

- **Tier:** **Fast** (paths: `docs/user/operations/BENCHMARKS.md`, `docs/dev/tasks/FIND-233.md` — sin paths adversariales).
- **Enfoque:** ¿la decisión está tomada con números verificables? ¿se distingue medición de estimación? ¿se respetó Regla 11 (no tocar cifras medidas) y el baseline de `FIND-720`? ¿el coste declarado basta para que el owner decida?
- **Evidencia mecánica (real, no auto-reporte):**
  1. `gh run list --workflow perf-bench.yml --limit 10 --json conclusion,createdAt,headSha` → 1 fila (`success` / `2026-10-27T22:10:15Z` / `3e1e21b3…`).
  2. `gh api repos/ness-e/Vantadb/actions/workflows/364848308/runs --jq .total_count` → `1`; el único run con `event: push`.
  3. `gh run view 40592702083 --json jobs` → job `benchmark` `2026-10-27T22:10:18Z`→`22:16:30Z` (**6m12s**).
  4. `gh run view 40592702083 --log` → 3 sub-runs extraídos; spread calculado a mano (tabla §Noise floor).
  5. `gh run view {37088714140,37089421873,37089581213,36093538630}` → `HTTP 404` en los cuatro.
  6. `git ls-tree develop --name-only .github/workflows/` → sin entrada `perf`; `git show main:benchmarks/python_baseline.json` → metadata de re-baseline 2026-10-03 presente.
  7. `node scripts/docs/check-links.mjs` → exit 0 (0 broken); `node scripts/docs/check-docs.mjs` → exit 0; `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml` → 0 issues; `dev-tools/verify_changed.ps1` → ver §RESULTADO.
- **Regla 11:** no se modificó ninguna cifra medida existente; el bloque nuevo es 100% datos con comando + entorno. §11 (baseline canónico, `FIND-720`) intacto.
- **Veredicto:** approve — la decisión (c) es la única que no añade coste recurrente y la que los propios datos soportan; la atribución se declara no-atribuible con la aritmética explicitada, no con una opinión.

## Colaterales para el orquestador (no arreglados — fuera de scope)

| # | Colateral | Evidencia | Por qué no acá |
|---|---|---|---|
| 1 | `docs/specs/SPEC.md:140` describe el gate como operativo (bandas + `ab_ref`) | `rg perfect-bench docs/specs/SPEC.md` | `SPEC.md` no está en los archivos permitidos de esta tarea → fila `FIND-*` para el owner |
| 2 | La banda warn de la familia *quarantined* (15%) está bajo el suelo intra-job (34-42%) y no tiene piso absoluto en el lado warn (sí en el lado block) | run 40592702083: 2 warnings sobre métricas de <10 µs | solo aplica si el gate se revive en `main`; tocar `compare_baseline.py` exigiría resucitar el workflow (decisión (b), rechazada) |
| 3 | `node scripts/docs/gen-index.mjs --check` **falla en `develop` desde antes de esta tarea** | probado: `--check` devuelve `CHECK FAILED` con `FIND-233.md` movido fuera del árbol (drift commiteado + `FIND-706`/`FIND-722` sin commitear de otros workers) | regenerar acá hornearía WIP ajeno en el índice y podría crear links a ficheros que aún no existen → lo hace el orquestador al cerrar la campaña |


## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4
PROXIMO_STEP: ninguno (escalada opcional: 1 dispatch A/B en main, 11-15 min, comando en §Decisión)
COMMIT_HASH: <se rellena al commitear>
ARCHIVOS: docs/user/operations/BENCHMARKS.md · docs/dev/tasks/FIND-233.md
VERIFY_CONTRATO: pasa (tabla de auditoría + noise floor + aritmética + decisión con coste; docs gates verdes)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:disparado | P: contrato explícito; D: sin símbolos públicos nuevos; V: verify sin fallas; C: colateral SPEC.md:140 stale
SKILLS_CARGADAS: ci-cd-and-automation · git-workflow-and-versioning · performance-optimization (pinned SDP) · doubt-driven-development · documentation-skill (+ campaign-executor/progreso base)
```
