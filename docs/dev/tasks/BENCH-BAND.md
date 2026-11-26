---
title: "BENCH-BAND: subir la banda de regresión del bench por encima del ruido medido (5% -> 10%)"
kind: task
status: completed
description: "El comparador del bench canónico declara 5% de tolerancia mientras el ruido medido entre corridas del mismo commit es 7-9%: la banda sube a 10%, con la fuente del ruido documentada y el techo de la banda única declarado"
id: BENCH-BAND
---

# BENCH-BAND: la banda de regresión por encima del ruido medido

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §9 fila 5 (decisión del owner) y §9.b
- **Fuente:** decisión 5 de §9 del plan ("5% declarado vs 7-9% de ruido medido → subir por encima del ruido (~10%)"). **No** es fila de `docs/dev/Backlog.md`: es decisión del plan.
- **Esfuerzo:** 🟢 1 h
- **Tipo:** CI / instrumentación de medición — sin código de producto
- **Carril:** C
- **Dependencias:** `FIND-720` ✅ (`aea7b1221`, baseline real del job) · `FIND-233` ✅ (`5eaf978c8`, ruido medido). En paralelo: `FIND-732` (`dev-tools`/tests), `MEM-72` y `MCP-48` (Rust). Sin solape de ficheros.
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (2) | `nightly.yml` job `bench-informational` → `compare --fail-on-regression` (informativo, `continue-on-error`, abre issue). · `certification.yml` job `analyze` → `compare --fail-on-regression` (informativo) **y** `update-baseline` gateado por `steps.check_regression.outputs.has_regression != 'True'` — ese es el único punto donde la banda tiene consecuencia además del aviso |
| Callees | Ninguno: el cambio es una constante. `compare_command` no importa nada del repo |
| Implicaciones | **Sensibilidad**, no corrección. Con 10%: un delta real del 8% deja de avisar (y deja de bloquear la promoción del baseline en `analyze`, que lo hornearía). Ningún número medido publicado cambia (Regla 11 intacta) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/bench_regression.py` (304 L, completo: `REGRESSION_THRESHOLD` L22, `compare_command` L111-230 con `change_pct > threshold` L139 y `severity` L140, `--threshold` L275-276), `.github/workflows/nightly.yml` L420-709 (job `bench-informational` completo: compare L563-567 **sin** `--threshold`), `.github/workflows/certification.yml` L655-770 (compare L670-675 sin `--threshold`; promoción L761-768 gateada por `has_regression`), `docs/user/operations/BENCHMARKS.md` §11 L436-505 y §20 L1555-1707, `docs/dev/tasks/FIND-720.md` L1-143 (mismo instrumento, medición del spread), `.opencode/rules/release-ci.md` (42 L), `.opencode/AGENTS.md` Regla 9 / Regla 11.
- **Referencias hacia dentro:** ninguna importa la constante — ni scripts, ni tests, ni `dev-tools`, ni `.githooks` (`rg -n 'REGRESSION_THRESHOLD|bench_regression' .githooks dev-tools scripts` → solo el propio fichero). Los workflow **no** pasan `--threshold`, así que el default es el único sitio donde la banda vive.
- **Referencias salientes:** la constante solo alimenta `--threshold` default y el texto de `--help`. `severity: critical` usa `threshold * 2` (20% con la banda nueva) — coherente, no se toca.
- **Referencias entrantes que quedan stale (fuera de write-scope, no las toco):** `docs/dev/tasks/FIND-720.md:138,251` y `:143-144` citan `REGRESSION_THRESHOLD = 5.0` como estado de *ese* momento (registro histórico, no declaración de banda vigente); `benchmarks/README.md:69-77` describe la banda del gate Python retirado, no esta; `docs/dev/operations/CI_POLICY.md:699` menciona el job sin citar umbral.
- **Veredicto de impacto:** **bajo en corrección (una constante), medio en señal.** Una sola fuente de verdad (la constante), dos consumidores informativos, un efecto no-informativo (la guarda de promoción de baseline en `analyze`) que se declara como techo.

## Diagnóstico

### Dónde vive la banda (un solo sitio, verificado)

```
$ rg -n --glob '!.git' --glob '!target' -e '--threshold' -e 'REGRESSION_THRESHOLD' -e 'bench_regression' .
scripts\bench_regression.py:22:REGRESSION_THRESHOLD = 5.0  # percent
scripts\bench_regression.py:275:    cmp.add_argument("--threshold", type=float, default=REGRESSION_THRESHOLD,
scripts\bench_regression.py:276:                    help=f"Regression threshold %% (default: {REGRESSION_THRESHOLD})")
```

Ningún workflow pasa `--threshold` (`nightly.yml:563-567`, `certification.yml:670-675`): la banda **es** la constante. Cambiarla en un sitio y dejarla vieja en otro (pre-mortem a) no puede pasar mientras no se añada el flag al workflow — y no se añade, precisamente para no crear la segunda copia.

### El drift que ya existía

El workflow y la doc ya **declaraban** `>10%` en prosa mientras el comparador cortaba en 5%:

- `nightly.yml:437` — "Regression policy (>10% p99 vs baseline needs ADR-or-revert)"
- `nightly.yml:619,659` — el mismo `>10%` en el summary y en el issue
- `BENCHMARKS.md` §11 L453-454 — "Regresión p99 >10% vs baseline requiere ADR o revert"

Con la constante en 10% las tres afirmaciones pasan a ser ciertas. Antes eran la política declarada con un instrumento que avisaba a la mitad de ese umbral.

### De dónde sale el 10% (y qué familias cubre)

| Fuente | Instrumento | Ruido medido | ¿Bandea esto? |
|---|---|---|---|
| `FIND-720` (dos corridas reales, misma rama, 5h20m aparte): `insert` 766984.4 → 698162.6 ms, `search` 4260.7 → 3958.1 ms | **el mismo** (`canonical_p99` + `bench_regression.py`) | **−9.0% / −7.1%** vs la primera corrida; ratio max/min **9.9% / 7.7%** | **Sí — es la fuente primaria** |
| §20 (run `40592702083`, `perf-bench.yml`, **3 sub-runs, una VM, un commit**): `query_hybrid.p50` **34.2%** / `p95` **42.2%**, `query_vector.p50` **16.7%** | otro (Python, Retirado 2026-10-29) | 16.7-42.2% | **No** — corrobora que sub-10% está por debajo del ruido ambiente de un runner de GitHub, pero es otro instrumento |

**Por qué 10 y no 15-25** (`FIND-720` proponía 15-25%): 10% es la banda que el owner eligió, y cubre la fuente primaria (7-9% en base delta-vs-primera-corrida) con 1-3 pp de holgura sin apagar la señal por completo. **Al borde:** el mismo par medido, en base max/min, da 9.9% de spread — la banda queda pegada a ese borde. Una corrida con la varianza del peor lado todavía puede dispararla.

**Una banda única es una simplificación, no una verdad.** El comparador tiene un solo `threshold` global y **no** permite banda por label; se mantiene así porque hoy solo compara las dos familias de `canonical_p99` (`insert_100k_glove100d`, `search_1000_queries_glove100d`) y las dos son del mismo instrumento. La asimetría que queda anotada: las familias vector/híbridas de §20 tienen 16.7-42.2% de spread intra-VM, **por encima** de 10% — si algún día este comparador las mide, 10% las hará avisar por ruido igual que hoy lo hace 5%.

## Decisión

`REGRESSION_THRESHOLD = 10.0` en `scripts/bench_regression.py`, con la fuente del ruido en el comentario de la constante. Doc: §11 (descripción del instrumento) y §20 (veredicto de `FIND-233`) de `BENCHMARKS.md`. Workflow: **sin cambios** — no pasa el flag, así que el default sigue siendo el único sitio.

## Contrato

```bash
rg -n '5\s*%|0\.05|threshold|regression' scripts/bench_regression.py
actionlint .github/workflows/nightly.yml          # 1.7.12 del PATH
# prueba de banda: 8% NO avisa, 20% SÍ (sintéticos en %TEMP%/opencode/bench-band)
```

## Steps

- [x] 1 — Localizar la banda (un solo sitio) y sus consumidores. ✅ `rg` arriba
- [x] 2 — Medir el comportamiento **antes** con sintéticos (8% avisa → ese es el falso positivo). ✅ salida en §Verificación
- [x] 3 — Subir la constante a 10.0 con la fuente del ruido en el comentario. ✅ `scripts/bench_regression.py:22-52`
- [x] 4 — Actualizar §11 y §20 de `BENCHMARKS.md` (sin tocar cifras medidas). ✅
- [x] 5 — Re-correr la prueba (8% no avisa / 20% avisa) + actionlint. ✅
- [x] 6 — Review P2-01 y commit local. ✅ `ci(bench): set the regression band above the measured noise (BENCH-BAND)`

## Verificación

### 1 — La banda, con `rg` sobre el fichero que la define

```console
$ rg -n '5\s*%|0\.05|threshold|regression' scripts/bench_regression.py
7:regressions exceeding a configurable threshold (default 10%, see
31:# At 5% both sides of that spread were reported, so a run that only saw its own
43:# Known ceiling: at 10% a real 8% regression reads as `stable`, and in
139:    threshold = args.threshold
164:                if change_pct > threshold:
167:                elif change_pct < -threshold:
182:            "threshold_pct": threshold,
188:            "has_regression": len(regressions) > 0,
300:    cmp.add_argument("--threshold", type=float, default=REGRESSION_THRESHOLD,
301:                help=f"Regression threshold %% (default: {REGRESSION_THRESHOLD})")
```

Las dos ocurrencias de "5%" que quedan son la explicación del histórico ("at 5%
both sides of that spread were reported"), no una declaración de banda. `5.0` y
`0.05` desaparecieron; `REGRESSION_THRESHOLD = 10.0` es el único sitio.

### 2 — `actionlint` (1.7.12, del PATH)

```console
$ actionlint .github/workflows/nightly.yml
actionlint exit=0
```

El workflow no se edita: no pasa `--threshold`, así que el default de la
constante sigue siendo la única fuente. Añadir el flag sería crear la segunda
copia que el pre-mortem (a) describe.

### 3 — Prueba de banda: 8% NO avisa, 20% SÍ

Sintéticos en `%TEMP%/opencode/bench-band/` (copia del script + baseline
sintético en la ruta relativa que `BASELINE_PATH` resuelve desde `__file__`, de
modo que el fichero versionado del repo no se toca en ningún momento):

**Antes (banda 5%) — el falso positivo, reproducido:**

```console
$ python $root/scripts/bench_regression.py compare $root/report_p08.json --format json --fail-on-regression
  "threshold_pct": 5.0,
  "regressions": [ { "label": "canonical_p99/insert_100k_glove100d", ..., "change_pct": 8.0, "severity": "warning" },
                   { "label": "canonical_p99/search_1000_queries_glove100d", ..., "change_pct": 8.0, "severity": "warning" } ],
  "stable_count": 0, "has_regression": true
::warning::Detected 2 benchmark regression(s) exceeding 5.0%!
exit=1
```

**Después (banda 10%):**

```console
$ python -c "...;print('threshold =', b.REGRESSION_THRESHOLD)"
threshold = 10.0

$ python $root/scripts/bench_regression.py compare $root/report_p08.json --format json --fail-on-regression
  "threshold_pct": 10.0,
  "regressions": [], "improvements": [], "new_benchmarks": [],
  "stable_count": 2, "total": 2, "has_regression": false
exit=0

$ python $root/scripts/bench_regression.py compare $root/report_p20.json --format json --fail-on-regression
  "regressions": [ { "label": "canonical_p99/insert_100k_glove100d", ..., "change_pct": 20.0, "severity": "warning" },
                   { "label": "canonical_p99/search_1000_queries_glove100d", ..., "change_pct": 20.0, "severity": "warning" } ],
  "stable_count": 0, "has_regression": true
::warning::Detected 2 benchmark regression(s) exceeding 10.0%!
exit=1
```

(El 20% exacto cae en `severity: warning` porque el corte de `critical` es
`change_pct > threshold * 2` estricto — 20 > 20 es falso. Con la banda vieja el
mismo 20% salía `critical`. No es un defecto introducido: es el borde del
multiplicador, y sigue saliendo `critical` en cualquier delta >20%.)

**El spread real de `FIND-720`, bajo las dos bandas** (baseline = valores
publicados del run `44228761011`, report = run `44228771621`):

```console
$ ... compare report_find720.json --format json --threshold 5
threshold 5.0  | stable 0 | regressions 0 | improvements [('insert_100k_1536d', -8.97), ('search_1000_queries_1536d', -7.1)]

$ ... compare report_find720.json --format json          # default 10
threshold 10.0 | stable 2 | regressions 0 | improvements 0
```

Eso es exactamente el síntoma que la banda vieja producía (nada comparable
nunca, `stable_count: 0`) y el que la nueva resuelve.

### 4 — Gates de docs y de código

```console
$ node scripts/docs/check-links.mjs          → broken markdown links: 0 (budget 0)   ✅
$ node scripts/docs/check-docs.mjs           → GATING: all clear                      ✅
$ node scripts/docs/gen-index.mjs --write    → regenerated docs/index.md + llms.txt   ✅
$ node scripts/docs/gen-index.mjs --check    → index exit=0                           ✅
$ npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml <los 2 md>
                                             → Summary: 0 issues in 0 files           ✅
$ python -m py_compile scripts/bench_regression.py                                    ✅
```

### 5 — `verify_changed.ps1`: FAIL por WIP ajeno, no por esta tarea

```console
$ pwsh -NoProfile -File dev-tools/verify_changed.ps1
  affected: 202 files
  fmt... FAIL
    vanta-memory/tests/ingest_observability.rs:115      ← MEM-72 (carril R, en vuelo)
    vantadb-mcp/src/proxy.rs:499                        ← MCP-48 (carril R, en vuelo)
    vantadb-mcp/tests/mcp_proxy_reconnect.rs:127,166,…  ← MCP-48
1 FAIL
```

Ninguno de esos ficheros está en el diff de esta tarea
(`docs/index.md`, `docs/user/operations/BENCHMARKS.md`, `llms.txt`,
`scripts/bench_regression.py` — cero Rust). El verify de batch del orquestador
(plan §3.c paso 2) es el que decide; este FAIL no se puede arreglar desde acá sin
tocar ficheros de otra sesión, que el contrato prohíbe.

## Techo conocido (declarado, no escondido)

| Techo | Consecuencia | Dónde queda escrito |
|---|---|---|
| Un delta **real** del 8% ahora lee `stable` | El aviso se pierde si algún día el job se vuelve gate | comentario de la constante + §11 + §20 |
| En `certification.yml` `analyze`, `has_regression` **gatea** la promoción del baseline | Un 8% real no bloquea la promoción → se hornea en el baseline en vez de quedar marcado | comentario de la constante + §11 |
| Banda única para todas las labels | Las familias vector/híbridas de §20 (16.7-42.2% intra-VM) avisarían por ruido si este comparador las midiera | §11 + §20 + este fichero |
| 10% está al borde de su propia fuente | El mismo par medido en base max/min da 9.9%; una corrida con la varianza del peor lado todavía puede disparar | §11 + este fichero |

## Review (P2-01)

- **Tier:** **Fast** (paths del diff: `scripts/`, `docs/`, índice generado — ninguno matchea los globs adversariales de `pipeline-full.md`).
- **Gate:** verify fast mecánico — gates de docs + `actionlint` + `py_compile` + prueba de banda, todos arriba, con salida pegada.
- **Modo:** **degradado y marcado.** Esta sesión es un sub-agente sin acceso a `subagent`: no hay `vanta-review` en contexto distinto. Se hizo re-lectura adversarial del diff completo (doubt-driven-development) en vez de un review por agente distinto; el spot-check definitivo queda en el verify de batch del orquestador (plan §3.c).
- **Waiver:** owner = orquestador del plan `2026-11-23-release-v0.10.0-gates`, ref = §3.c paso 2 (`verify_changed.ps1` de batch antes del push).
- **Chequeos adversariales corridos y su resultado:**
  1. ¿Queda una segunda declaración de la banda vieja? → `rg` repo-wide: ninguna (solo `FIND-720.md`, registro histórico, y `WIRE-06.md`, gate distinto del 15% ya cerrado).
  2. ¿Se tocó alguna cifra medida en `BENCHMARKS.md`? → No: los números citados (`766984.4`, `698162.6`, `4260.7`, `3958.1`, `16.7%`, `34.2%`, `42.2%`) son los ya publicados; lo agregado son porcentajes derivados por aritmética y rotulados como tales.
  3. ¿El comparador sigue avisando de algo? → Sí: 20% → `has_regression: true`, exit 1 con `--fail-on-regression`.
  4. ¿El `--threshold` explícito sigue funcionando? → Sí: `--threshold 5` reproduce el comportamiento viejo (usado arriba como control).
  5. ¿El fichero de baseline versionado se tocó? → No: la prueba corre sobre una copia en `%TEMP%`.

## Lo que esta tarea NO hace

- No toca `benches/**`, `src/**`, `vantadb-mcp/**` ni `tests/**` (los tienen `FIND-732`, `MEM-72`, `MCP-48`).
- No agrega banda por familia de métrica: el comparador tiene un `threshold` global y hoy solo compara las 2 labels de `canonical_p99`, del mismo instrumento. Añadir el mecanismo sin una segunda familia que lo use es infraestructura especulativa.
- No re-mide nada: la banda es una decisión sobre datos ya publicados, no una nueva medición (y correr `canonical_p99` son 2h30 — fuera del apetito).
- No promueve ni degrada el job: sigue informacional (`continue-on-error`, Q5 del owner).
