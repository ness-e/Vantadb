---
title: "TASK FIND-233: perf-bench — instrumento cross-VM (A/B same-job opt-in)"
kind: task
description: "Decisión implementada: A/B same-job opt-in (workflow_dispatch ab_ref, pares alternados, mediana de ratios, bandas provisionales) + límites del instrumento documentados en BENCHMARKS.md"
---

# TASK FIND-233: perf-bench — instrumento cross-VM (A/B same-job opt-in)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 4, Wave F0)
- **Fuente:** `docs/dev/Backlog.md` (FIND-233) + FIND-232 (evidencia de varianza cross-VM)
- **Esfuerzo:** 🟡 4-6h | **Appetite:** 1d
- **Prioridad:** 🟠
- **Tipo:** CI/CD + instrumentación de performance (blast radius CI/dev-tools — sin cambios de motor)
- **Creado:** 2026-10-04T07:00Z | **last-synced:** 2026-10-04T07:00Z
- **Estado:** ⏳ IN PROGRESS (tramo local ✅ — pendiente: review P2-01 + ACCEPT + verificación post-push; orquestador)
- **Incógnitas (uphill):** 0 — resueltas en Discovery (opción elegida con costo medido; ver §Decisión)
- **Pendientes (downhill):** 1 (Step 7 post-push diferido; Steps 1-6 ✅)
- **Campaign ID:** master-plan-0.9.0-20261004

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/perf-bench.yml` (única entrada de `compare_baseline.py`); `docs/user/operations/BENCHMARKS.md` §2/§3 (doc pública del gate); `SPEC.md:132` (guardrail); `docs/dev/workflow/perf-bench-40.md` (runbook) |
| Callees | `compare_baseline.py` = stdlib puro (`argparse/datetime/json/statistics/sys`) — sin imports del repo; el workflow depende de `./.github/actions/rust-setup` + actions pinneadas por SHA |
| Implicaciones | **push path intacto** (steps A/B nuevos gated por `if: ${{ inputs.ab_ref }}` — null en push → skip); self-test 9/9 preservado + casos A/B nuevos; sin cambios de motor/API pública/storage/WAL; sin migración de datos |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, WIP de FIND-237/239 en vuelo (`src/cli.rs`, `src/bin/vanta-cli.rs`, `tests/cli_tests.rs` staged; `docs/user/QUICKSTART.md` etc.).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `benchmarks/compare_baseline.py` (331L), `.github/workflows/perf-bench.yml` (124L), `docs/user/operations/BENCHMARKS.md` (§1–§14 leídas; 791L), `benchmarks/README.md` (78L), `benchmarks/python_baseline.json` (38L), `benchmarks/vantadb_local_bench.py` (257L), `docs/dev/tasks/FIND-232.md` (182L), `docs/dev/workflow/perf-bench-40.md` (45L), `.opencode/rules/release-ci.md` (42L), `SPEC.md:127-135`.
- **Referencias entrantes (grep `compare_baseline|perf-bench|python_baseline`):** `perf-bench.yml` (invoca el script en 2 ramas: compare/update), `SPEC.md:132` (guardrail: "instrumento cross-VM: FIND-233"), `docs/dev/workflow/perf-bench-40.md` (runbook), `docs/dev/workflow/README.md`/`TRIGGERS.md`/`RULES.md` (describen triggers — no cambian), `docs/index.md` (generado — se regenera). Ningún otro consumidor programático de `python_baseline.json`.
- **Referencias salientes:** ninguna al repo (stdlib puro); workflow → `rust-setup` local + `actions/checkout|setup-python|upload-artifact` pinneadas por SHA (se reusan los mismos SHAs).
- **Veredicto impacto:** **LOCALIZADO / CI+dev-tools-only.** No toca motor, API pública, storage, WAL ni bindings. Riesgo de regresión del gate actual: acotado a que los steps nuevos no alteran el camino push (condición `inputs.ab_ref` falsy en push — semántica verificada contra docs oficiales de GHA, ver §Investigation Notes).

## Diagnóstico (evidencia dura — Regla 9/11)

### La varianza cross-VM hace invisible la señal fina (<2-3x)

Medido en FIND-232 (mismo método/código/perfil 10000/1000, 2026-10-03; runs 37088714140 vs 37089421873 vs 37089581213):

| Métrica | Job rápido (37089421873) | Job normal (37089581213) | Job lento (37088714140) | Spread |
|---|---|---|---|---|
| ingest | 37.8 s (264.2 rec/s) | 47.8 s (209.3 rec/s) | 64.6 s (154.8 rec/s) | 1.7x |
| query_vector.p50 | 1.29 ms | 2.46 ms | 2.69 ms | 2.1x |
| query_hybrid.p50 | 3.16 ms | 6.37 ms | 6.85 ms | 2.2x |
| insert.p99 | 5.07 ms | 8.52 ms | 85.3 ms | **16.8x** |

Las bandas actuales (stable warn>25%/block>200%; µs 300%+0.5ms; insert.p99 100ms absoluto — FIND-232) detectan **colapsos**, no regresiones finas. La señal <2-3x es invisible en el gate.

### Costo CI por step (run 37089421873, `gh run view --json jobs`)

| Step | Duración |
|---|---|
| rust-setup | 1.1 min |
| Build & install Python wheel | 0.7 min |
| Bench (3 runs × 10000/1000) | 2.4 min |
| Resto (checkout/setup/compare/upload) | ~0.1 min |
| **Total job** | **4m30s** (VM lenta: 6m18s–6m55s) |

### Noise floor intra-job (mismos SHA, 3 sub-runs dentro del MISMO job — extraído de logs)

| Job | ingest sub-runs | pares adyacentes | vector.p50 sub-runs | hybrid.p50 sub-runs |
|---|---|---|---|---|
| 37089421873 (rápido) | 41.81 / 37.85 / 36.98 s | −9.5%, −2.3% | 1.334/1.290/1.260 | 3.353/3.161/3.129 |
| 37089581213 (normal) | 48.51 / 47.78 / 47.25 s | −1.5%, −1.1% | 2.464/2.435/2.549 | 6.373/6.706/6.288 |
| 37088714140 (lento) | 76.16 / 64.59 / 59.21 s | −15.2%, −8.3% | 3.478/2.690/2.105 | 7.861/6.852/5.848 |

**Hallazgo clave:** en TODOS los jobs hay una **rampa warm-up monotónica** (run N más rápido que run N−1). Un diseño A/B por bloques (A×3 → B×3) sesga B hacia "más rápido" (el lado que corre después) y enmascara regresiones. → El diseño debe usar **pares alternados (A_i, B_i)** y estadístico de **mediana de ratios pareados**.

## Decisión (contrato core — opción 1 implementada, con costo medido)

**Opciones evaluadas (contrato del plan):**

| # | Opción | Veredicto | Evidencia |
|---|--------|-----------|-----------|
| 1 | **A/B same-job** (wheel del ref-baseline + HEAD en la misma VM) | ✅ **ELEGIDA** — única que crea la señal fina; costo medido ~2× solo si se invoca (opt-in) | Costo: job actual 4m30s → A/B est. 9-14 min (2 builds + 6 runs + 6 swaps), timeout 30 min ✓. Precedente repo: A/B pareado WIRE-06 (`BENCHMARKS.md` §13: alternación neutraliza deriva de máquina compartida) |
| 2 | Calibración CPU previa | ❌ descartada | No representativa por diseño: el gate mezcla CPU (vector/HNSW) + fsync (`insert.p99` 16.8x) + PyO3; un factor de calibración sintético no modela los 3 regímenes (pre-mortem del plan #2) |
| 3 | Banda documentada como límite | ⚠️ parcial (se incluye, pero como complemento) | FIND-232 ya la documentó en docstring/workflow/SPEC; sola NO crea señal fina (el objetivo del FIND). Se documenta formalmente en `BENCHMARKS.md` §2 como límite del gate always-on |

**Implementación elegida:** A/B same-job **opt-in** vía `workflow_dispatch` input `ab_ref` (mitigación del pre-mortem #1 "costo CI x2": el gate push queda cheap; el instrumento fino se invoca cuando hay una pregunta de performance). Diseño:

1. `ab_ref` = commit/tag del lado A (típicamente el SHA del baseline vigente); HEAD/ref disparada = lado B.
2. Build de ambos wheels en el mismo job; **3 pares alternados** (A_1,B_1,A_2,B_2,A_3,B_3) con swap de wheel entre runs → cancela rampa/deriva.
3. Comparación por **mediana de ratios pareados** (`compare_baseline.py --ab-runs 1 2 3`), bandas **provisionales** warn>15% / block>50%; `insert.p99` warn-only (jitter fsync).
4. **Protocolo de calibración:** dispatch con `ab_ref=<SHA de HEAD>` (mismo código a ambos lados) → el ratio medido ES el noise floor real same-job → se ajustan bandas con esa evidencia y se documenta (mismo protocolo que FIND-232).
5. El gate push (compare vs baseline almacenado, bandas calibradas) queda **intacto** — detecta colapsos; el A/B opt-in detecta la zona fina.

## Contrato

"decisión implementada de UNA de las 3 opciones (opción 1: A/B same-job opt-in) + `python benchmarks/compare_baseline.py --self-test` verde (9 casos originales + casos A/B nuevos) + `actionlint .github/workflows/perf-bench.yml` exit 0 + fixtures A/B locales (limpio→0, regresión→1, profile mismatch→1) + approach documentado en `BENCHMARKS.md` §2 y `benchmarks/README.md` + **tramo post-push (deferido, comandos documentados):** `gh run list --workflow=perf-bench.yml --limit 3` → ≥2 runs `success` consecutivos tras el push del plan + primer dispatch A/B verde".

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. El **push path** de `perf-bench.yml` (sin `ab_ref`) no cambia comportamiento: mismos steps, mismos comandos, mismas bandas, mismo baseline.
  2. `--self-test` conserva los **9 casos originales** (FIND-154/FIND-232) sin cambio de expectativas.
  3. Sin `continue-on-error` nuevo; actions pinneadas por SHA; `permissions: contents: read`; timeout 30 min (reglas `release-ci.md` R2/R5 + AGENTS.md R2).
  4. El gate sigue siendo **fail-closed** en profile mismatch y en baseline sin `total_records`.
- **Comandos de verificación:** `python benchmarks/compare_baseline.py --self-test` → `15/15 cases as expected` (9 originales + 6 A/B); `actionlint .github/workflows/perf-bench.yml` → exit 0; fixtures locales A/B (ver Step 2) → exit 0/1 según caso.
- **Deuda pendiente:** bandas A/B **provisionales** (warn 15%/block 50%) — requieren calibración con el primer dispatch same-SHA post-push; `insert.p99` no bloquea en A/B (fsync jitter). La atribución fina del +10-19% vector/hybrid post-0.8.0 (Backlog FIND-233, ítem 2) queda para un A/B dispatch post-push con `ab_ref=114f55f0`.

## Deuda técnica (Regla 6)

- **Introducida:** ninguna (Python stdlib; YAML reusa SHAs existentes; sin dependencias nuevas). El modo A/B agrega ~90 líneas a un script de testabilidad interna + ~40 líneas YAML gated.
- **Pagada:** elimina la ceguera estructural del instrumento en la zona <2-3x (la señal que el plan F0 declara como gate de justificación) y formaliza el límite del gate always-on en la doc pública (evita que la próxima regresión fina se atribuya mal, como pasó con los 13 rojos de FIND-232).
- **Diferida:** calibración de bandas A/B (primer dispatch post-push); nightlies A/B automáticos (solo si el uso demuestra valor).

## DoD (3 niveles)

- **task:** contrato local (decisión implementada + self-test 15/15 + actionlint + fixtures + docs) ✅; tramo post-push (≥2 runs verdes + primer A/B dispatch) — **deferido por instrucción del owner** (push al final del plan); comandos documentados en §Steps/Step 7 y en el RESULTADO §7.
- **commit:** `perf(ci): FIND-233 — ...` conventional + verify local (commit LOCAL; sin push).
- **release:** n/a (CI-only).

## Herramientas necesarias

- `python benchmarks/compare_baseline.py --self-test` (gate de bandas, offline)
- `actionlint .github/workflows/perf-bench.yml` (validación YAML)
- `gh run view <id> --json jobs` / `--log` (evidencia de runs: costo + sub-runs)
- `campaign_verify_cmd` (verify mecánico por step)
- `codegraph_codegraph_explore` + `codebase-memory-mcp_check_index_coverage` (blast radius — usados; los benchmarks Python no están en el grafo → grep como fallback justificado para referencias entrantes)

**Skills cargadas (SDP v3):** `ci-cd-and-automation` (pinned CI) · `performance-optimization` (pinned performance) · `git-workflow-and-versioning` (pinned CI/release) · `doubt-driven-development` (base CI/CD) · `campaign-executor`/`progreso` (base, auto) · `documentation-skill` (obligatoria docs/**) · `test-driven-development` (lógica nueva del modo A/B con self-tests) · `incremental-implementation` (slices). SDP: base+lifecycle+keywords, 8 cargadas de 8.

## Investigation Notes

- **GHA — semántica `inputs` en `if` (verificado contra docs oficiales, 2026-10-04):** <https://docs.github.com/en/actions/reference/evaluate-expressions-in-workflows-and-actions> — "falsy values (`false`, `0`, `-0`, `""`, `''`, `null`) are coerced to `false`". Para eventos push el contexto `inputs` no tiene valores → `inputs.ab_ref` es null → `if: ${{ inputs.ab_ref }}` es **false** → los steps A/B se skipean en push. En `workflow_dispatch` con `ab_ref` vacío → `''` → false. Seguro en ambos caminos.
- **GHA — `actions/checkout` con `path:` + `ref:`:** patrón estándar para checkouts adicionales; el wheel A se construye desde `ab-ref/vantadb-python/Cargo.toml` con `maturin build --manifest-path ... --out ab-ref/vantadb-python/dist` (mismo patrón del workflow actual).
- **Metodología A/B:** los 6 runs usan el **mismo script de bench** (checkout HEAD) para ambos lados — la comparación mide la diferencia entre wheels (motor), no entre scripts. La DB del bench se limpia por run (`shutil.rmtree` al inicio) y los outputs A/B usan archivos separados (`ab_results.<i>.json` / `benchmark_results.<i>.json`).
- **Costo:** el modo A/B agrega ~1 build extra (sccache cold para el ref) + 3 runs + 6 swaps de wheel → job ~9-14 min (vs 4m30s), dentro del timeout de 30 min; **solo corre en dispatch con `ab_ref`** (el push no lo ejecuta).
- **Pre-mortem del plan (3 riesgos) — respuesta:** (1) costo x2 → mitigado: opt-in, costo medido documentado; (2) calibración no representativa → opción 2 descartada con evidencia (3 regímenes distintos); (3) rabbit hole → stop-condition 6h respetada con el diseño acotado (no se toca el push path, no se agregan nightlies).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas: (a) opción → 1 con costo medido; (b) diseño → pares alternados por rampa warm-up medida; (c) costo → medido por step |
| Pendientes de ejecución (downhill) | 7 steps (6 locales + 1 post-push diferido) |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: no toca trust boundaries, input de usuario, auth, datos ni dependencias. (Único input nuevo: `ab_ref` en workflow_dispatch → pasa a `actions/checkout ref:` — superficie de maintainers del repo, no usuario final; checkout pinneado por SHA.)
- [x] **PERFORMANCE** — aplica (es el objeto de la tarea): baseline = costo CI medido por step + noise floor intra-job (§Diagnóstico); impacto esperado = push path sin cambio de costo, A/B opt-in +~5-9 min por invocación; sin cambios de hot path de motor.

## Steps

### Step 1 — `compare_baseline.py`: modo A/B (`--ab-runs`) + self-tests

- **Archivos:** `benchmarks/compare_baseline.py`
- **Acción:** agregar `compare_ab()` (mediana de ratios pareados B vs A, bandas provisionales `AB_WARN_PCT=15` / `AB_BLOCK_PCT=50`, `insert.p99` warn-only, guarda de profile fail-closed A/B) + flag `--ab-runs` + `--ab-median-out` (default `ab_results.json`) + 6 casos nuevos de `--self-test` (idéntico→0, +30%→warn, +80%→block, B más rápido→0, insert.p99 +300%→warn-only, profile mismatch A/B→block) + docstring actualizado (modelo FIND-233).
- **Verify:** `python benchmarks/compare_baseline.py --self-test` → `15/15 cases as expected`; `python -m py_compile benchmarks/compare_baseline.py` → exit 0
- **Evidencia:** ✅ 15/15 PASS (exit 0; los 9 casos originales FIND-154/232 intactos + 6 A/B nuevos); py_compile exit 0; registrado vía `campaign_verify_cmd` (2.º intento — el 1.º devolvió exit -1 por spawn transitorio del server, reintentado y verde).
- **Estado:** ✅ COMPLETED

### Step 2 — Fixtures locales A/B (simulación de los 3 desenlaces)

- **Archivos:** `$env:TEMP\opencode\find233\` (fixtures temporales, no versionados)
- **Acción:** construir sets A/B sintéticos (medianas conocidas) y verificar exit codes del modo `--ab-runs`: (a) B≈A → exit 0 sin bloqueo; (b) B +80% sostenido → exit 1 con `::error::`; (c) A/B con distinto `total_records` → exit 1 "A/B profile mismatch"; (d) B más rápido → exit 0.
- **Verify:** 5 casos con exit codes esperados (`ab_fixture_check.py` en `$TEMP\opencode\find233\`)
- **Evidencia:** ✅ 5/5 as expected (identical→0; +80%→1 con "same-job A/B ceiling"; profile mismatch→1; faster→0; +30%→0 con "warn band"); medianas `ab_results.json`/`benchmark_results.json` escritas por caso.
- **Estado:** ✅ COMPLETED

### Step 3 — `perf-bench.yml`: input `ab_ref` + steps A/B (push path intacto)

- **Archivos:** `.github/workflows/perf-bench.yml`
- **Acción:** (a) input `ab_ref` (workflow_dispatch, default `''`); (b) `if: ${{ !inputs.ab_ref }}` en los steps del camino actual (build wheel + bench 3 runs); (c) steps nuevos gated `if: ${{ inputs.ab_ref }}`: checkout del ref a `ab-ref`, build de ambos wheels, loop de 3 pares alternados con swap de wheel, compare A/B; (d) artifact extra `ab_results.json`; (e) comentarios de costo/diseño.
- **Verify:** `actionlint .github/workflows/perf-bench.yml` → exit 0; revisión de que el camino push no cambia (diff)
- **Evidencia:** ✅ actionlint exit 0 (pre y post-edición final). Push path: solo se agregan `if: ${{ !inputs.ab_ref }}` (no-op en push: null → falsy — verificado contra docs oficiales GHA) + steps nuevos gated `if: ${{ inputs.ab_ref }}`; `permissions`, timeout y SHAs sin cambio; workspace members explícitos → `ab-ref/` no colisiona con el workspace root; wheel A/B en dist dirs separados.
- **Estado:** ✅ COMPLETED

### Step 4 — Docs: límites del instrumento + modo A/B + calibración

- **Archivos:** `docs/user/operations/BENCHMARKS.md` (§2 nueva subsección), `benchmarks/README.md`, `docs/dev/workflow/perf-bench-40.md` (defaults stale 1000/100 → 10000/1000 + `ab_ref`), `SPEC.md:132`
- **Acción:** documentar (a) bandas del gate always-on y qué detecta/no detecta (colapsos ≥3x vs señal fina); (b) modo A/B opt-in: comandos exactos de dispatch, protocolo de calibración same-SHA, bandas provisionales, `insert.p99` warn-only; (c) verificación post-push (comandos `gh run list`); (d) referencia a `canonical_p99` para señal fina always-on.
- **Verify:** `node scripts/docs/check-links.mjs` exit 0; `node scripts/docs/check-docs.mjs` exit 0; `node scripts/docs/gen-index.mjs --check` exit 0
- **Evidencia:** ✅ check-links exit 0 (44/58 presupuesto — 0 nuevos; el único link nuevo resuelve); check-docs exit 0 (GATING all clear); gen-index: staleness ajena (FIND-236.md, commit `bec2dd1b` sin regen — FIND-233 ya estaba indexado) → `--write` (diff 6+/5-: `docs/index.md` + `llms.txt`, generados) → `--check` exit 0; markdownlint: 0 issues nuevos (MD028 `benchmarks/README.md:39` preexistente — verificado contra `HEAD:benchmarks/README.md`); `SPEC.md:132` actualizado (§8 stale → §11 + A/B FIND-233).
- **Estado:** ✅ COMPLETED

### Step 5 — Verify local completo + OCR delegation

- **Archivos:** —
- **Acción:** correr verify full del tramo local: `--self-test` (15/15), fixtures (Step 2), `actionlint`, `py_compile`, `campaign_verify_cmd` para los comandos del contrato; OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) sobre los archivos propios.
- **Verify:** todos exit 0; OCR 0 Critical/High en archivos propios
- **Evidencia:** ✅ `verify_changed.ps1` ALL 3 PASS (fmt/check/clippy); `campaign_verify_cmd`: self-test ✅, actionlint ✅, check-links ✅, check-docs ✅, py_compile ✅; OCR delegation spec: 2 reviewables propios (perf-bench.yml → Rule Group 1 CI; compare_baseline.py → Rule Group 2 Python) revisados contra sus reglas → **0 Critical / 0 High** (detalle en §Review).
- **Estado:** ✅ COMPLETED

### Step 6 — Review P2-01 + commit local

- **Archivos:** `docs/dev/tasks/FIND-233.md`
- **Acción:** review P2-01 por agente distinto (tier Fast: CI/docs/scripts → `dev-tools/verify.ps1` ALL PASS + veredicto en §Review; fork a `vanta-review` si disponible); commit **LOCAL** `perf(ci): FIND-233 — ...` (solo archivos propios; nunca push).
- **Verify:** `git show --stat HEAD` limitado a los archivos propios; veredicto registrado en §Review
- **Evidencia:** commit local (hash en el RESULTADO §7; sin push). Review P2-01: veredicto delegado al orquestador (este worker es leaf — no puede forkear `vanta-review`); evidencia mecánica completa en §Review para spot-check Fast.
- **Estado:** ✅ COMPLETED (ACCEPT del orquestador pendiente)

### Step 7 — Post-push (orquestador/owner — DIFERIDO por instrucción)

- **Archivos:** —
- **Acción:** tras el push del plan: (a) verificar ≥2 runs verdes consecutivos del push path: `gh run list --workflow=perf-bench.yml --limit 3 --json databaseId,conclusion,headSha`; (b) primer dispatch A/B de calibración (mismo código): `gh workflow run perf-bench.yml --ref develop -f ab_ref=<SHA HEAD> -f size=10000 -f queries=1000 -f dim=128`; `gh run watch <id> --exit-status`; (c) registrar el noise floor medido y ajustar bandas A/B si corresponde; (d) opcional: A/B de atribución `ab_ref=114f55f0` (delta vector/hybrid post-0.8.0).
- **Verify:** ≥2 `success` consecutivos + dispatch A/B `success`
- **Estado:** ⬜ PENDING (deferido — push al final del plan; comandos exactos en el RESULTADO §7)

## Dependencias

- FIND-232 ✅ COMPLETED (evidencia de varianza + bandas calibradas — input del diagnóstico).
- nextTask: FIND-234 (Task 5 del plan).

## Review (GATE — agente distinto, P2-01)

- **Tier:** **Fast** (paths: `.github/**`, `benchmarks/**`, `docs/**`, `SPEC.md` — sin paths adversariales).
- **Revisor:** ⏳ pendiente — delegado al orquestador (`vanta-review`; este worker es leaf y no puede forkear sub-agentes). Evidencia mecánica completa abajo para el spot-check.
- **Enfoque:** ¿el approach A/B same-job opt-in es correcto? ¿alternativas (calibración/bloques) evaluadas con evidencia? ¿el push path queda realmente intacto?
- **Evidencia de verificación (real, no auto-reporte):**
  1. `python benchmarks/compare_baseline.py --self-test` → **15/15** (9 originales FIND-154/232 intactos + 6 A/B nuevos); registrado vía `campaign_verify_cmd`.
  2. Fixtures A/B end-to-end del CLI (`ab_fixture_check.py`, temp, no versionado): **5/5** — identical→exit 0; +80%→exit 1 (`same-job A/B ceiling`); profile mismatch→exit 1; faster→exit 0; +30%→exit 0 + warning.
  3. `actionlint .github/workflows/perf-bench.yml` → exit 0 (antes y después de las ediciones finales).
  4. `pwsh dev-tools/verify_changed.ps1` → **ALL 3 PASS** (fmt/check/clippy).
  5. `node scripts/docs/check-links.mjs` + `check-docs.mjs` → exit 0; `gen-index --check` exit 0 tras regenerar (staleness ajena: FIND-236.md).
  6. markdownlint: 0 issues nuevos (MD028 `benchmarks/README.md:39` preexistente — verificado contra `HEAD:benchmarks/README.md`).
- **OCR delegation (advisory, sin API key):** `pwsh dev-tools/ocr-review.ps1 -Format json` → preview: 2 reviewables propios; `ocr delegate rule` → Rule Group 1 (CI/VantaDB) aplicado a `perf-bench.yml` y Rule Group 2 (Python) a `compare_baseline.py`: **0 Critical / 0 High**. Verificado: actions pinneadas por SHA ✓, `permissions: contents: read` ✓, timeout 30 ✓, sin `continue-on-error` nuevo ✓, sin secrets/injection en `run:` (`ab_ref` solo en `if:`/`with:`) ✓, sin dead-code/mutable-defaults/edge-case gaps en el diff Python (guarda de conteo, `va > 0`, fail-closed de perfil, median sobre lista no vacía) ✓.
- **Checklist anti-hábitos tóxicos:** sin comandos inventados (todo output transcripto de corridas reales); sin done sin verificar (contrato local con evidencia; post-push explícitamente diferido); fallo parcial reportado (spawn transitorio del 1.er `verify_cmd`, reintentado y documentado).
- **Veredicto:** ⏳ pendiente (orquestador) — evidencia preparada para spot-check Fast. Cierre final también condicionado a la verificación post-push (Step 7).

## Notas

- La opción 2 (calibración) se descarta con evidencia: el gate mezcla CPU + fsync + PyO3; `insert.p99` (16.8x cross-VM) es fsync-jitter y no correlaciona con CPU.
- El push path queda con **una sola diferencia estructural**: los steps actuales ganan `if: ${{ !inputs.ab_ref }}` (no-op en push: null → true). Verificado contra docs GHA.
- Bandas A/B provisionales (15/50) son el punto de partida; el protocolo de calibración (dispatch same-SHA) las ajusta con el noise floor real medido, igual que FIND-232 hizo con las bandas cross-VM.
- Si el review detecta que el A/B no puede calibrarse en el tramo local (no hay CI disponible), se documenta como deuda con trigger post-push (no bloquea el commit local).
- **Artefactos de evidencia (no versionados):** `$TEMP\opencode\find233\ab_fixture_check.py` (5/5), `$TEMP\opencode\find233\ocr_spec.json` + `ocr_rules.txt` (spec OCR), salidas de `gh run view --json jobs/--log` transcriptas en §Diagnóstico.
- **Nota de concurrencia:** durante la ejecución otros workers commitearon FIND-234/235/236/237/238/239 en el mismo árbol (el índice quedó stale por `FIND-236.md` — regenerado en este commit). `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` y `opencode.jsonc` quedan sin stage (ajenos).
