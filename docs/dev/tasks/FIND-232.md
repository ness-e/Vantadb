---
title: "TASK FIND-232: Diagnóstico del perf-bench crónico (13 rojos consecutivos desde 2026-09-25)"
kind: task
description: "Causa raíz: perfil push 1000/100 vs baseline 10000/1000 (apples-to-oranges) + techo relativo puro a escala µs; fix: alineación de perfil + guarda de parámetros + piso absoluto 0.5 ms"
---

# TASK FIND-232: Diagnóstico del perf-bench crónico (13 rojos consecutivos desde 2026-09-25)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 7, Wave 0)
- **Fuente:** `docs/dev/Backlog.md` (FIND-232)
- **Esfuerzo:** 🟡 4-6h | **Appetite:** 1d
- **Prioridad:** 🟠
- **Tipo:** CI/CD + diagnóstico de performance (blast radius CI-only — sin cambios de motor)
- **Creado:** 2026-10-03T02:25Z | **last-synced:** 2026-10-03T03:50Z
- **Estado:** ⏳ IN PROGRESS — review P2-01 ronda 1 = `changes-required` **atendido** (fix aplicado en este commit); pendiente post-push (owner): verificación del run (Step 7) + veredicto final. **No se marca COMPLETED sin veredicto final de review registrado.**
- **Campaign ID:** post-release-0.8.0-20261002

## Blast Radius

| Archivo | Cambio previsto |
|---------|----------------|
| `.github/workflows/perf-bench.yml` | fallback de `SIZE`/`QUERIES` para push: `'1000'`→`'10000'`, `'100'`→`'1000'` + comentarios del perfil y la banda |
| `benchmarks/compare_baseline.py` | guarda de perfil (`insert.total_records`) + piso absoluto `NOISY_BLOCK_ABS_MS=0.5` + 3 casos nuevos de `--self-test` + docstring con el modelo FIND-232 |
| `benchmarks/README.md` | referencia stale `perf-bench-40` → `perf-bench.yml` (L67) |
| `docs/dev/tasks/FIND-232.md` | este archivo (nuevo) |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP de otra sesión), `docs/dev/plans/2026-10-02-post-release-0.8.0.md` en el commit (bookkeeping del orquestador; se actualiza sin stage), `docs/pipeline-state.json`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/perf-bench.yml` (110L), `benchmarks/compare_baseline.py` (206L), `benchmarks/python_baseline.json` (39L), `benchmarks/vantadb_local_bench.py` (257L), `benchmarks/README.md` (78L), `docs/user/operations/BENCHMARKS.md` §2–§3 (L40–114), `.opencode/rules/release-ci.md` (42L), `.opencode/references/clean-code-clean-architecture.md` Apéndice V.
- **Referencias entrantes:** el workflow invoca `compare_baseline.py` (única entrada); `SPEC.md:132` y `benchmarks/README.md` documentan el gate; `docs/user/operations/BENCHMARKS.md` publica resultados. Ningún otro consumidor programático de `python_baseline.json`.
- **Referencias salientes:** `compare_baseline.py` = stdlib puro (`argparse/datetime/json/statistics/sys`) — sin imports del repo; el workflow depende de `./.github/actions/rust-setup` + actions pinneadas por SHA.
- **Veredicto impacto:** **LOCALIZADO / CI-only.** No toca motor, API pública, storage, WAL ni bindings. No hay migración de datos. Blast radius = 1 job de CI + 1 script offline.

## Diagnóstico (evidencia dura — Regla 9/11)

### Hallazgo 1 — Mismatch estructural de perfil (causa raíz primaria)

El gate compara **peras contra manzanas** desde su activación:

| Fuente | size | queries | evidencia |
|--------|------|---------|-----------|
| Baseline (`python_baseline.json`) | **10000** | **1000** | `insert.total_records: 10000`; metadata: run 36093538630 (workflow_dispatch) |
| Defaults de `workflow_dispatch` | 10000 | 1000 | `perf-bench.yml` L14–30 |
| Comando canónico documentado | 10000 | 1000 | `BENCHMARKS.md` L79/L106, `benchmarks/README.md` L16/L59 |
| **Push (el gate real)** | **1000** | **100** | `perf-bench.yml` L74–75 (`|| '1000'`, `|| '100'`); logs: `Dataset Size : 1000 vectors` / `Queries : 100` |

**Prueba del mismo commit (114f55f0, 2026-09-25) con dos caminos:**
- push **36094025517** (1000/100): **ROJO** — `query_hybrid.p50 +92.5%`, `query_hybrid.p95 +78.7%`, `query_text.p99 +521.5%`.
- dispatch **36094025761** (10000/1000): **VERDE** — "No regression > 15.0%"; mediana `query_hybrid.p50 = 4.45 ms` vs 11.05–11.53 ms del push.
→ El mismo código produce +100% de "regresión" solo por el perfil. El delta `query_hybrid` (+65–141% en todos los runs push) es **artefacto de dataset**, no motor.

### Hallazgo 2 — `p99` con n=100 es el **máximo** de la muestra (causa raíz estadística)

`vantadb_local_bench.py:43` → `p99 = sorted_lats[int(n * 0.99)]`; con `queries=100`, índice 99 = **la peor consulta**. El baseline (n=1000) usa el 10.º peor. En los 13 runs push, `query_text.p99` (mediana-de-3) fue 0.040–0.057 ms vs baseline 0.0057 ms → **+547% a +898%** (absoluto: 34–51 µs de jitter de scheduler). El techo "catastrófico" del 300% no sobrevive la escala µs.

Agravante documentado (MGR-19, `BENCHMARKS.md` §2): la workload lexical de la suite (`token_N keyword_N`) es **degenerada** (0 matches en el corpus) — `query_text` mide un camino vacío a escala µs; no es señal accionable por sí sola.

### Hallazgo 3 — `insert.p99` (tail) también flapea

Bloqueó en 3/13 runs (`+238.6%`, `+166.6%`, `+317.7%`). Con 1000 muestras el p99 es el 10.º peor de 1000; con 10000 es el 100.º de 10000 — granularidades distintas. Varianza medida del mismo código y mismo perfil (10000/1000, 8 min aparte): `insert.p99 7.14 → 5.11 ms (−28.4%)`, `query_hybrid.p50 5.76 → 4.45 ms (−22.8%)`.

### Tabla de runs rojos (13/13 con `query_text.p99` como bloqueante)

| Run | Fecha | Rama | Bloqueantes |
|-----|-------|------|-------------|
| 36094025517 | 09-25 | develop | text.p99 +521.5%, hybrid.p50 +92.5%, hybrid.p95 +78.7% (umbral global 15%) |
| 36101773914 | 09-25 | main | text.p99 +546.1%, hybrid.p50 +108.5%, hybrid.p95 +88.1%, hybrid.p99 +18.8% |
| 36162854285 | 09-25 | develop | insert.p99 +238.6%, text.p99 +555.5%, hybrid ×3 |
| 36173006135 | 09-25 | develop | text.p95 +22.2%, text.p99 +580.2%, hybrid ×3 |
| 36178959033 | 09-25 | develop | text.p99 +547.2%, hybrid.p50 +96.5%, hybrid.p95 +71.6% |
| 36956238271 | 10-02 | develop | text.p99 +797.7% |
| 36983788676 | 10-02 | main | text.p99 +616.0% |
| 37045089787 | 10-02 | main | insert.p99 +166.6%, text.p99 +825.4% |
| 37053737292 | 10-02 | develop | text.p99 +594.6% |
| 37053958756 | 10-02 | develop | text.p99 +755.7% |
| 37064937924 | 10-02 | main | text.p99 +855.8% |
| 37068990480 | 10-02 | develop | text.p99 +633.6% |
| 37082038748 | 10-03 | main | insert.p99 +317.7%, text.p99 +897.6% |

### Descarte de regresión de motor (hipótesis 3)

- Los cambios de search del ciclo 0.8.0 (`WIRE-08`, `SCH-03/04/05/07`, `9004c43f`) son **posteriores** al baseline (114f55f0..HEAD). El mismatch ya se manifestaba el 2026-09-25 (mismo commit verde/rojo según camino) **antes** de esos cambios.
- El patrón de cada run rojo se explica **completo** por perfil + escala (H1/H2/H3). No hay métrica estable con delta consistente peor.
- Dispatch diagnóstico **37088714140** (develop HEAD, 10000/1000, compare real vs baseline) → ver §Resultado del dispatch abajo.

### Dispatch diagnóstico (develop, 10000/1000, update_baseline=false)

**37088714140** (2026-10-03T02:08Z, commit dee48711, lanzado por este worker con `gh workflow run perf-bench.yml --ref develop -f size=10000 -f queries=1000 -f dim=128 -f update_baseline=false`):
**ROJO** con regresión en familias estables — deltas vs baseline 2026-09-25:

| Métrica | Actual (mediana-of-3) | Baseline | Δ |
|---|---|---|---|
| insert.total_duration_ms | 64592.68 | 48681.99 | **+32.7%** |
| insert.throughput_records_per_sec | 154.82 | 205.41 | **−24.6%** |
| insert.p95_ms | 7.70 | 5.67 | **+35.9%** |
| insert.p99_ms | 85.30 | 7.14 | **+1094.5%** |
| query_vector.p50_ms | 2.69 | 2.13 | **+26.2%** |
| query_vector.p95_ms | 3.68 | 2.65 | **+39.1%** |
| query_vector.p99_ms | 3.87 | 3.28 | **+17.8%** |
| query_hybrid.p50_ms | 6.85 | 5.76 | +19.0% (warning, cuarentena) |
| query_text.p99_ms | 0.0054–0.0058 | 0.0057 | sin cambio ✓ |

Sub-runs (misma job): ingest 76.2s/64.6s/59.2s (131→155→169 rec/s); vector p50 3.48/2.69/2.11 ms.
Sub-runs del baseline gen 36093538630 (2026-09-25): ingest 50.1s/48.0s/48.7s (199.7→208.4→205.4 rec/s); vector p50 2.43/2.12/2.13 ms — **plano, sin warm-up**.

`vantadb_local_bench.py` NO cambió desde el baseline (`git log 114f55f0..HEAD -- benchmarks/vantadb_local_bench.py` = vacío) → método idéntico; el delta es de motor y/o entorno. Candidatos de código: VER-01 hash-chain WAL (`0cc14247`), SCH-02 schema v2 (`7af34366`), FIND-190 invalidación de cache en put (`06496a9a`), WIRE-06 group-commit (`5e8e8879`). Los tres sub-runs de hoy fueron más lentos que los de 2026-09-25 (incluso el mejor: 59.2s vs 50.1s) → sistemático dentro de la job.

**Reproducibilidad: 37089421873** (2026-10-03T02:19Z, commit bf1e7476, mismo dispatch, 20 min después) — **VERDE**: "No blocking regression detected across 16 metrics (0 warnings)". Sub-runs: ingest 41.8/37.8/37.0 s (239→270 rec/s), vector p50 1.26–1.33 ms, hybrid p50 3.13–3.35 ms — ~40% **más rápido** que el baseline 2026-09-25 en todas las métricas.

**Conclusión de varianza cross-VM (par same-SHA #2/#3 + job lento #1, 20 min aparte):** en el par limpio (37089421873 vs 37089581213, ambos bf1e7476, mismo perfil) las métricas clave oscilan 1.3x–2.1x; incluyendo el job lento (37088714140, distinto Cargo.lock por #224) el rango llega a 2.4x y `insert.p99` a **16.8x** (85.3 vs 5.07/8.52 ms) — el gate a 15% era estructuralmente flaky aun con perfil alineado. Rango completo (5 jobs): ingest 37.0–76.2 s; vector p50 1.26–3.48 ms; hybrid p50 3.13–7.86 ms.

**Re-baseline: 37089581213** (2026-10-03T02:22Z, `update_baseline=true`, commit bf1e7476) — corrió el camino update ("Baseline updated…") ✓; su mediana (artifact `vanta-benchmark-results`) es el nuevo `python_baseline.json`. Números (VM "normal", ≈ baseline 2026-09-25): ingest 47.78 s (209.3 rec/s), vector p50 2.46/p95 3.16/p99 3.39 ms, hybrid p50 6.37/p95 7.38/p99 11.82 ms, text p99 6.46 µs.
**Bug encontrado y arreglado (mismo commit):** el step `Upload Baseline Candidate` skipeaba SIEMPRE (`if: inputs.update_baseline == 'true'` = boolean-vs-string → false) — también en la generación original 36093538630. Fix: step output `baseline_updated` del step de compare + `if: steps.bench-compare.outputs.baseline_updated == 'true'`.

**Delta del re-baseline vs 2026-09-25 (VM normal):** insert −2%, rebuild −1.3%, vector +3.4…+19.3% (p95 el máximo), hybrid +4.5…+10.7%, text +8…+13% (µs). **Dentro de la varianza cross-VM medida (1.7–2.4x)** → sin evidencia de regresión de motor; el "+32.7% insert" de #1 era la VM lenta, no el código.

**FIND-233 creado** (fila en `docs/dev/Backlog.md`): instrumento cross-VM (A/B same-job o calibración) + atribución fina del +10–19% en vector/hybrid post-0.8.0 — no se fuerza en esta task (stop condition del plan).

### OCR review (delegation, advisory)

- `pwsh dev-tools/ocr-review.ps1 -Format json` → preview: 3 reviewables (perf-bench.yml, compare_baseline.py, pipeline-state.json ajeno); Rule Groups 1 (CI/VantaDB) y 2 (Python) aplicados manualmente a MIS archivos: **0 Critical / 0 High**. Reglas verificadas: actions pinneadas ✓, `permissions: contents: read` ✓, timeout 30min ✓, sin `continue-on-error` nuevo ✓ (release-ci R5), sin mutable-defaults/dead-code/edge-case gaps en el diff ✓. Hallazgo propio: `docs/pipeline-state.json` es ajeno (no se toca).

## Contrato (del plan)

(a) `gh run list --workflow=perf-bench.yml --limit 1` = **success** tras fix/re-baseline documentado, **o**
(b) si es varianza estructural: banda µs revisada + decisión registrada y **run verde** con el nuevo baseline.

**Interpretación de cierre:** el fix es (1)+(2)+(3) con decisión registrada; el run verde se obtiene cuando el orquestador pushea develop (el push dispara el workflow con el perfil alineado) y el run queda `success` — **o**, si el run post-alineación destapa drift estable real, se re-baselinea con `update_baseline=true` y delta documentado.

## Steps

- [x] **Step 1 — Evidencia dura de runs:** `gh run list/view --log-failed` sobre 13 runs rojos + verdes; tabla por run con bloqueantes exactos; sizes reales de push (1000/100) vs dispatch (10000/1000); par mismo-commit 36094025517 vs 36094025761. ✅
- [x] **Step 2 — Pares de varianza controlados:** artifacts 2026-09-25 (gen/verify) + pares 2026-10-03 (#1/#2/#3): spread cross-VM 1.7–2.4x; insert.p99 16.8x. ✅
- [x] **Step 3 — Fix:** workflow (perfil 10000/1000 + upload del baseline-candidate por step output + comentarios), `compare_baseline.py` (guarda de perfil fail-closed + bandas calibradas + `insert.p99` absoluto + self-test ×9), README (ref stale), `python_baseline.json` (re-baseline), SPEC.md §Guardrails al día. ✅
- [x] **Step 4 — Verify local:** `--self-test` 9/9; `py_compile`; `actionlint` exit 0; sims: VM-lento vs baseline nuevo → exit 0 con warnings; VM-rápido → exit 0 limpio; regresión 3x → exit 1; fixture 1000/100 → "profile mismatch"; baseline sin `total_records` → fail-closed. ✅
- [x] **Step 5 — Dispatch diagnóstico + re-baseline:** 37088714140 (rojo, VM lenta), 37089421873 (verde), 37089581213 (update_baseline → baseline nuevo). Delta documentado. ✅
- [x] **Step 6 — Commit local** (fix + baseline nuevo + task file + Backlog FIND-233) — este commit. ✅
- [ ] **Step 7 — Cierre (orquestador):** push develop → verificar `gh run list --workflow=perf-bench.yml --limit 1` = `success` (contrato a; con perfil alineado + baseline nuevo el push queda verde). ⬜ PENDING orquestador.

## Decisión (bandas revisadas + re-baseline — contrato b)

1. **Perfil único `10000/128/1000`** para push y dispatch (= baseline = perfil canónico documentado en `BENCHMARKS.md` §3 / `benchmarks/README.md`). Costo CI: +2.5–3 min/run dentro del timeout de 30 min. Se descarta bajar el baseline a 1000/100: `p99` de 100 muestras = máximo muestral y la varianza de throughput a 1000 registros medida fue 192–672 rec/s (3.5×) — peor señal.
2. **`compare_baseline.py` rechaza comparar** cuando `insert.total_records` difiere del baseline ("profile mismatch") — la clase de bug que causó esto no puede volver en silencio.
3. **Bandas recalibradas al instrumento** (evidencia: varianza cross-VM 1.7–2.4x medida en 5 jobs con código idéntico):
   - stable: warn > 25%, block > 200% (≥3x);
   - quarantined (`query_hybrid`, `query_text`): warn > 15%, block > 300% **y** > 0.5 ms absolutos (escala µs);
   - `insert.p99_ms`: nivel absoluto — bloquea solo ≥ 100 ms (spread medido 16.8x: 5.07→85.3 ms).
   - La señal fina (<2–3x) queda delegada a `canonical_p99` (Regla 9, entorno controlado) — documentado en el docstring y en el comentario del workflow.
4. **Re-baseline ejecutado** (run 37089581213, perfil 10000/1000, VM normal): nuevo `python_baseline.json` commiteado con delta documentado (§ arriba). Reemplaza al de 2026-09-25 porque era pre-ciclo-0.8.0 y fue generado con un perfil distinto al que usaba el push.
5. **Bug del upload del baseline-candidate arreglado** (boolean-vs-string → step output): el mecanismo documentado de re-baseline ahora funciona end-to-end.
6. **Riesgo residual aceptado:** el gate detecta colapsos (stable ≥3x; µs ≥4x + 0.5 ms; insert.p99 ≥100 ms), no regresiones finas — límite del instrumento (runner único sin control de entorno), registrado en FIND-233 con la propuesta A/B same-job.
7. **No se fuerza** la atribución fina del +10–19% (vector/hybrid) post-0.8.0 — FIND-233 (stop condition del plan: sin cambio de motor en esta task).

## Review (P2-01)

- **Ronda 1 (2026-10-03):** veredicto **`changes-required`** — fix técnico verificado correcto; 1 requerido (SPEC.md:132 stale) + 3 opcionales. **Fix aplicado** (commit local de esta ronda): `SPEC.md` §Guardrails actualizado a las bandas nuevas + "señal fina (<2–3x) vive en canonical_p99 / FIND-233"; docstring con el par **same-SHA** (37089421873 vs 37089581213, ambos bf1e7476); guarda de perfil **fail-closed** si el baseline pierde `total_records` (+ self-test 9/9); DoD alineado a "evidencia sustituta". ⬜ **Pendiente post-push (owner):** verificación literal del run + veredicto final.
- **Paths del diff (ambas rondas):** `.github/workflows/perf-bench.yml`, `benchmarks/compare_baseline.py`, `benchmarks/python_baseline.json`, `benchmarks/README.md`, `docs/dev/tasks/FIND-232.md`, `docs/dev/Backlog.md`, `SPEC.md` → **Tier Fast** (CI/docs/datos; sin paths adversariales).
- **Gate requerido:** `dev-tools/verify.ps1` ALL PASS + veredicto en este §Review.
- **Checklist para el reviewer:**
  1. `python benchmarks/compare_baseline.py --self-test` → 9/9 PASS.
  2. `git show HEAD --stat` → solo los archivos de arriba (+ plan file sin stagear).
  3. Contrato post-push: `gh run list --workflow=perf-bench.yml --limit 1` = success.
  4. Sanity adversarial: run 1000/100 vs baseline 10000/1000 → FALLA con "profile mismatch"; baseline sin `total_records` → FALLA (fail-closed); run con `insert.p99` 85 ms → warning solamente; regresión 3x → exit 1.
- **OCR delegation (advisory):** preview + Rule Groups CI/Python aplicados; 0 Critical/High (detalle en §Diagnóstico → OCR review).

## DoD (3 niveles)

- **task:** **contrato (b) cumplido en evidencia sustituta** (replays con artifacts reales de los runs + sims locales): banda revisada + decisión registrada (§Decisión) + runs verdes (37089421873 compare success; 37089581213 update_baseline success) + baseline nuevo commiteado. La verificación literal post-push (`gh run list --workflow=perf-bench.yml --limit 1` = success) queda al owner (Step 7; el worker tiene prohibido push).
- **commit:** `fix(ci): FIND-232 — ...` conventional + verify local (commit LOCAL; sin push).
- **release:** n/a (CI-only).

## Deuda técnica (Regla 6)

- **Introducida:** ninguna (Python stdlib, sin dependencias nuevas; YAML 2 literales + comentarios).
- **Pagada:** elimina la deuda activa del gate muerto (señal de performance apagada 8 días) y la clase "apples-to-oranges silencioso".
- **Diferida (FIND-232-R1, trigger ≥2 falsos positivos post-fix):** banda por familia tail (p95/p99) para `insert`/`query_vector`, con datos alineados.
