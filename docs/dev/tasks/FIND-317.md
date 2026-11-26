---
title: "FIND-317: el bench canónico en CI nunca completaba — retiro registrado y presupuesto declarado"
kind: task
description: "El workflow bench-canonical-p99-informational.yml ya no existe: el diagnóstico real es lentitud extrema medida (698 s/iter × 10 muestras), no hang"
---

# FIND-317 — canonical_p99 en CI: retiro registrado + presupuesto medido

## Metadata

- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§5, Fase 1, wave 1.3)
- **Depende de:** ninguna (wave 1.3, carril C — sin `cargo`)
- **Fuente:** fila `FIND-317` del Backlog ("el workflow nunca completa: 30+ corridas canceladas, check rojo permanente, cero datos publicados")
- **Esfuerzo:** ~2h (appetite 2-4h)
- **Prioridad:** media (un check rojo permanente entrena a ignorar el rojo)
- **Tipo:** CI / operaciones (workflow + política)
- **Estado:** ✅ COMPLETED
- **SDP:** `campaign_discover_skills_v2 phase=VERIFY` → `campaign-executor`, `progreso`, `ci-cd-and-automation` (pinned), `git-workflow-and-versioning` (pinned), `performance-optimization` (pinned), `doubt-driven-development`, `systematic-debugging`, `browser-testing-with-devtools` (descartada: no hay navegador). Cargadas: `documentation-skill` (obligatoria: se edita `docs/`), `performance-optimization`, `systematic-debugging`.

## Diagnóstico — la premisa de la fila estaba vencida

| Claim de la fila | Verificado | Fuente |
|---|---|---|
| "el workflow `bench-canonical-p99-informational.yml` nunca completa" | **el archivo ya no existe.** Borrado en `2ba62d46f` (2026-10-15); absorbido por `nightly.yml` en la consolidación CI-08/SRV-10 (jobs `toolchain-check` + `bench-informational`) | `git log --diff-filter=AD --name-status`; `nightly.yml:29-33` |
| "30+ corridas canceladas" | `gh run list --workflow bench-canonical-p99-informational.yml --limit 20` → **`HTTP 404: workflow … not found on the default branch`**. La fila es invisible al comando de triage, misma clase de fallo que `FIND-328`. En las últimas 100 corridas de `nightly.yml`: **0 `cancelled`** | `gh`, 2026-11-24 |
| "el step del bench muere en el job-timeout" | **falso hoy.** El sucesor completa: 2 corridas medidas `success` (ver tabla) | `gh run view` |
| "cero datos publicados" | **publica** un artefacto de 1774 B… con números **no comparables** (ver §"Datos") | `gh run download` |
| "check rojo permanente" | el bench no está en la lista `REQUIRED` de `ci-gate.yml`; en las últimas 100 corridas de `nightly.yml` el bench no es ninguna de las 3 fallas (`text_index_recovery`, `hnsw_validation`, `ci-gate / Main is green`) | `ci-gate.yml:36-55`, `gh run view` |

**Hipótesis de la fila ("hang vs lentitud extrema — insert 100k×1536") → resuelta: LENTITUD, con números.**

El job no se cuelga: imprime progreso de Criterion, recolecta las 10 muestras y termina.

```
Warning: Unable to complete 10 samples in 5.0s. You may wish to increase target time to 7030.0 s.
Benchmarking canonical_p99/insert_100k_1536d: Collecting 10 samples in estimated 7030.0 s (10 iterations)
canonical_p99/insert_100k_1536d
                        time:   [693.88 s 698.16 s 703.29 s]
```

`insert = 698.16 s/iter × 10 muestras = 7030 s (117 min)` — estructural, no patológico.
Compilar 86 s + insert 117 min + search 39.6 s ≈ 120 min de trabajo + setup ≈ **145 min de job**.

La fila del Backlog ya traía el otro extremo de la curva y lo confirma: `Run canonical_p99`
"corrió **5298 s** y fue matado por el timeout del job (**90 min** = 5400 s)". Es decir, el
run moría a 88 minutos de un trabajo que necesitaba ~117 sólo para el insert: **el timeout
de 90 min era el bug, no el bench**. Con el presupuesto de 240 min el mismo trabajo termina
en 145–160 min. Ninguna evidencia de cuelgue en ninguna de las dos mediciones.

## Evidencia medida (la parte que faltaba)

Corridas del sucesor, 2026-11-18, `gh run view <id> --json jobs`:

| Run | Job | `startedAt` | `completedAt` | Duración | Conclusión |
|---|---|---|---|---|---|
| 44228761011 | `Toolchain compiles bench (release, no run)` | 03:12:36Z | 03:22:18Z | 9m42s | success |
| 44228761011 | `Run canonical_p99 vs baseline (informational)` | 03:23:29Z | 06:03:08Z | **159m39s** | success |
| 44228771621 | `Toolchain compiles bench (release, no run)` | 06:03:11Z | 06:12:17Z | 9m06s | success |
| 44228771621 | `Run canonical_p99 vs baseline (informational)` | 06:12:19Z | 08:37:21Z | **145m02s** | success |

Presupuesto actual: `timeout-minutes: 240` (subido 90 → 240 el 2026-10-07 por un commit de esta misma fila, `93b201a76`). Con 159m39s medido como máximo, el margen es **1.5×** — suficiente; **no se toca el timeout** (pre-mortem (a): tornear sin medir).

## Datos publicados: existen, pero no son comparables

`gh run download 44228771621 -n canonical-p99-results` (artefacto de 1774 B):

```json
{ "baseline_commit": "TBH-02-initial-seed", "threshold_pct": 5.0,
  "regressions": [], "improvements": [],
  "new_benchmarks": [{"label": "canonical_p99/insert_100k_1536d", "current_ms": 698162.643},
                     {"label": "canonical_p99/search_1000_queries_1536d", "current_ms": 3958.101}],
  "stable_count": 0, "total": 2, "has_regression": false }
```

`stable_count: 0` y ambos benchmarks en `new_benchmarks` ⇒ **no hay baseline contra el cual comparar**, así que `has_regression` nunca puede ser `true` y el step "Open regression finding" nunca abre issue. La maquinaria de regresión existe y jamás dispara.

Causa: los labels `*_1536d` son del bench **viejo**. `origin/main:benches/canonical_p99.rs` todavía tiene `const DIM: usize = 1536;`; `develop` migró a GloVe-100d (`2b821184a`, 2026-10-29) con labels `insert_100k_glove100d` / `search_1000_queries_glove100d`. Las dos corridas medidas fueron de ramas `release-plz-*` (basadas en `main`) ⇒ midieron el corpus **que `BENCHMARKS.md` §11 declara superseded**. En `develop` el mismo bench cuesta ~29 min de muestreo (insert 176.3 s medido en dev × 10).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/nightly.yml` (L1-100, L320-620); `.github/workflows/ci-gate.yml` (L1-100); `docs/dev/operations/CI_POLICY.md` (L1-651); `.opencode/rules/release-ci.md` (L1-62); `docs/user/operations/BENCHMARKS.md` §11 (L436-457); `benches/canonical_p99.rs` (lectura, sin editar — prohibido).
- **Referencias hacia dentro de `nightly.yml`:** `ci-gate.yml` (reusable, job `ci-gate`), `notify-failure` (no lista jobs informativos en `needs`, por diseño L40-41), `.github/actions/rust-setup`, `scripts/bench_regression.py` (`extract`/`compare`), `docs-gates.yml` (gate de inventario de workflows compara el nº declarado en `CI_POLICY.md` contra el directorio: **no cambia el número de archivos**, 20 → 20).
- **Referencias entrantes al nombre viejo:** `docs/user/operations/BENCHMARKS.md:454` ("es EVALUATED in `bench-canonical-p99-informational.yml`") — **stale**, apunta a un archivo borrado. `nightly.yml:32,334,427` lo citan como *origen* histórico (correcto, se conserva).
- **Veredicto impacto:** **bajo**. No se crea ni se borra ningún workflow (inventario intacto: 20). No se toca `ci-gate.yml` `REQUIRED`, ni ramas, ni permisos, ni `continue-on-error` (se conservan con su `# CATEGORY: INFORMATIONAL`). El único efecto de comportamiento es **cuándo** corre un job informativo y que un merge a `main` ya no pueda matarlo.

## Decisión: (c) — mover a la lane programada con presupuesto declarado, y registrar el retiro

Las tres opciones del contrato, con lo que dice la evidencia:

| Opción | Veredicto | Evidencia |
|---|---|---|
| (a) timeout/scope | **ya aplicada y suficiente.** No se vuelve a tocar | 90 → 240 el 2026-10-07 (`93b201a76`); medido 159m39s máx ⇒ 1.5× de margen |
| (b) retirar el workflow | **ya ocurrió en el árbol** (2026-10-15). Retirar *el bench* sería el error: `develop` sí tiene el corpus canónico y `BENCHMARKS.md` §11 lo exige como señal Regla 9 | `git log`, `benches/canonical_p99.rs:5-15` |
| (c) mover a nightly/scheduled con presupuesto | **elegida** | ver abajo |

**Por qué (c) y no dejar el estado actual:** hoy el job corre en la lane de PR (`if: pull_request || workflow_dispatch`) y el nightly lo salta (`skipped` en las 30 corridas programadas de la muestra). Consecuencias medidas: (1) 2.5 h de job por PR que toque `src/index/`/`src/storage/`, produciendo números que el comparador descarta (`stable_count: 0`); (2) el job queda expuesto al `concurrency` de workflow (`cancel-in-progress: true`, L76-78) — un push a la misma rama lo mata, que es exactamente la forma del síntoma "30+ canceladas" que la fila describe; (3) `CI_POLICY.md` §2b ya designa esta lane como *reception point* de gates >5 min, y este job mide 159 min (32× el umbral).

**Lo que sí se conserva:** `toolchain-check` (9m, compile-gate `--no-run`) **queda en la lane de PR** — es el `consumo guard` documentado en `BENCHMARKS.md` §11 y cuesta 9 minutos. Y `workflow_dispatch` sigue disponible para que un autor de hot path pida el número bajo demanda en su PR.

**Categoría:** `INFORMATIONAL` (Regla 2 de `AGENTS.md`, taxonomía de `CI_POLICY.md`). Todos los `continue-on-error` del job ya llevan `# CATEGORY: INFORMATIONAL`; **no se añade ninguno**, y no se añade exención nueva.

## Steps

1. ✅ **DISCOVERY** — evidencia `gh` (404 del workflow, jobs de las corridas 44228761011/44228771621), artefacto descargado, `git log` del archivo, `ci-gate.yml` REQUIRED, visibilidad del repo.
2. ✅ **Diagnóstico** — lentitud extrema confirmada con números (698 s/iter × 10); no hang.
3. ✅ **ACT 1** — `nightly.yml`: `bench-informational` pasa a `schedule`/`dispatch`; `concurrency` propio con `cancel-in-progress: false`; comentarios con los números medidos (reemplazan la estimación de 2026-10-07); nota de excepción en la cabecera CI-08.
4. ✅ **ACT 2** — `CI_POLICY.md` §6: bloque de decisión `FIND-317` (retiro del archivo, evidencia, presupuesto, categoría, regla "no resucitar", y las dos deudas vivas).
5. ✅ **VERIFY** — `actionlint` sobre `nightly.yml`; `gh run list/view` (contrato mecánico) documentado arriba.
6. ✅ **CIERRE** — review y commit local.

## VERIFY (contrato mecánico)

```text
gh run list --workflow bench-canonical-p99-informational.yml --limit 20 --json conclusion,createdAt
→ HTTP 404: workflow bench-canonical-p99-informational.yml not found on the default branch
  (el archivo fue retirado el 2026-10-15; mismo patrón de fila invisible que FIND-328)

gh run view 44228771621 --json jobs
→ Toolchain compiles bench (release, no run): success 2026-11-18T06:03:11Z 2026-11-18T06:12:17Z
→ Run canonical_p99 vs baseline (informational): success 2026-11-18T06:12:19Z 2026-11-18T08:37:21Z
```

## Deudas que esta tarea NO paga (fuera de su write-scope)

1. **`docs/user/operations/BENCHMARKS.md:454`** sigue citando `bench-canonical-p99-informational.yml` como el lugar donde se evalúa el `>10%`. Destino real: `nightly.yml` job `bench-informational`. Requiere otra fila (este task tiene `Escritura:` limitada a workflow + `CI_POLICY.md` + task file).
2. **`main` todavía sirve el bench 1536d** (`const DIM: usize = 1536`) mientras `develop` ya tiene GloVe-100d. Mientras no se sincronice, la serie programada medirá el corpus superseded y el comparador seguirá con `stable_count: 0`. No se toca `benches/**` (prohibido por contrato). Owner: `vanta-tuner`.
3. **Índices generados desfasados por esta wave, no por esta tarea.** `node scripts/docs/gen-index.mjs --check` falla con `expected 746` vs `744` commiteados. Los dos documentos que faltan son `docs/dev/tasks/FIND-156.md` (untracked, carril Rust de la misma wave) y este `FIND-317.md`. **No se regeneró**: `--write` habría metido el trabajo en vuelo de otra tarea en este commit. El gate `Generated indexes are current` (required) queda rojo hasta que la wave cierre y alguien corra `gen-index.mjs --write` una vez por todos los task files nuevos. Acción del orquestador, no de esta tarea.

## Review

- **Tier:** Fast (`.github/workflows/*.yml`, `docs/dev/operations/CI_POLICY.md` → "cualquier otro path"). Sin paths del tier Adversarial.
- **Verify mecánico:** `actionlint .github/workflows/nightly.yml` → **exit 0**; `continue-on-error: true` 23/23 con `# CATEGORY:` (Regla 2 intacta, 0 exenciones nuevas); gate de inventario de workflows 20/20 PASS; `check-docs` GATING all clear; `check-links` 0 enlaces markdown roto (el wikilink presupuestado de `avance/historial` es preexistente).
- **OCR delegation:** preview + `ocr delegate rule .github/workflows/nightly.yml` (modo delegación, sin modelo). Autorevisión contra la checklist devuelta: sin `pull_request_target`, sin secrets, `permissions` job-level mínimo (`contents: read` + `issues: write` justificado) sin cambios, actions SHA-pinned sin cambios, `if:` sin entrada no confiable, timeout y concurrency presentes, 0 `continue-on-error` nuevo.
- **P2-01:** sesión leaf sin tool de sub-agente ⇒ **degradado**, waiver `{owner: vanta-lead, ref: FIND-317 §Review}`. La revisión cruzada pendiente cubre 1 archivo: ~30 líneas de comentario + 1 `if:` + 1 bloque `concurrency` + borrado de un gate muerto.
