---
title: "Plan de Ejecución: Post-release 0.8.0 — Estabilización de infraestructura"
kind: plan
description: "SDP v3 (taskType CI/CD-DevOps): campaign-executor · progreso · ci-cd-and-automation · git-workflow-and-versioning · performance-optimization · doubt-driven-development · planning-and-task-breakdown · documentation-and-adrs · security-and-hardening · shipping-and-launch"
---

# Plan de Ejecución: Post-release 0.8.0 — Estabilización de infraestructura

> **Campaign ID:** post-release-0.8.0-20261002
> **Inicio:** 2026-10-02
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/dev/Backlog.md` (FIND-225..232 + PROC-02..04) + PRs abiertos (#237/#232/#231/#230/#224) + cierre del release 0.8.0
> **Autonomous:** false

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 10 |
| 🟡 DEFER | 3 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 2 |

Status: ⬆️ uphill = 2 incógnitas abiertas (causa raíz del perf-bench crónico; alcance real de los leaks ASan) · ⬇️ downhill = 33 steps pendientes (10 tareas)

## Contexto de campaña

El release 0.8.0 quedó **publicado y verificado** (crates.io · PyPI · npm wasm+TS · binarios 5/5 · SBOM — 2026-10-02). Esta wave cierra la deuda de infraestructura que el release destapó, en orden de ROI: mantenimiento de dependencias → gates mecánicos que evitan repeticiones → higiene CI → triage de seguridad/performance.

**Grafo de dependencias (waves):**

```
Wave 0 (sin archivos compartidos, paralelo):
  PROC-02 (deps) · PROC-03 (secrets) · PROC-04 (registro) · FIND-232 (perf) · FIND-225 (codeql)

Wave 1 (paralelo):
  FIND-230 (gate-api-docs.yml) ∥ FIND-226 (ci-rust.yml: job ASan) ∥ FIND-227 (TSan: suppressions)

Wave 2 (después de Wave 1 — archivos compartidos):
  FIND-231 (ci-rust.yml: job nuevo) → FIND-228 (triggers de 15 workflows + TRIGGERS.md)
```

## Tasks

### Task 1: PROC-02 — Mergear los 5 PRs de Dependabot abiertos

- **Appetite:** 1d
- **Esfuerzo:** 🟢 1h (+CI por PR)
- **Prioridad:** 🟠
- **Archivos clave:** PRs `#237` (rust-toolchain), `#232` (undici/desktop), `#231`+`#230` (codeql-action), `#224` (croaring/rust-minor) — todos contra develop
- **Verificación real:** ✅ CÓDIGO-REAL — `gh pr list --state open` mostró los 5 PRs `chore(deps)` (verificado 2026-10-02 21:20 local); las 8 vulnerabilidades reportadas por GitHub corresponden a undici (7, desktop) + brace-expansion (1, vantadb-ts, sin PR aún — el updater falló el 2026-10-02 18:05, run 37045118748)
- **Gate Justificación:** las 8 alerts de Dependabot viven acá; merges acotados, cada uno validado por CI; cero riesgo para el código del motor
- **Gate Result:** ✅ DO
- **Contrato:** `gh pr list --state open --json title --jq '[.[] | select(.title | startswith("chore(deps)"))] | length'` **= 0** y `gh api '/repos/ness-e/Vantadb/dependabot/alerts?state=open' --jq 'length'` **≤ 1** (residuales triados con nota); develop con CI verde post-merges
- **Pre-mortem:** (1) `#224` (abierto 2026-09-25) quedó stale por el ciclo 0.8.0 → `@dependabot rebase`; (2) el updater de brace-expansion falla repetido → correr `npm audit fix` manual en `vantadb-ts` si hace falta; (3) merge en batch rompe el lock → mergear de a uno con CI verde
- **Stop conditions:** appetite 1d excedido → dejar los restantes con nota en la fila; PR irresoluble → cerrar y dejar que el updater regenere
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | PR stale/conflictos | `@dependabot rebase` antes de mergear | 1 intento |
  | 🟢×🔴 | Merge rompe develop | CI verde por PR, merge secuencial | cada merge |
  | 🟡×🟡 | updater caído (brace-expansion) | fallback `npm audit fix` + PR manual | si no abre PR en 24h |

- **Cynefin:** 🟦 obvio — mantenimiento estándar, causa-efecto claro
- **Top 3 riesgos:** (1) stale/conflictos; (2) orden de merges; (3) updater caído
- **Uphill/Downhill:** ⬇️ downhill (0 incógnitas — 5 merges + verificación)
- **DoD:** task = PRs merged + alerts ≤1 verificados por comando · commit = n/a (merges remotos) · release = n/a
- **Validación Appetite vs Effort:** 1d ≥ 1h ✓
- **Estado:** ⏳ EN PROGRESO
- **Task file:** `docs/dev/tasks/PROC-02.md`

### Task 2: PROC-03 — Descartar las 3 alertas de secret scanning (falsos positivos de test)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 15min
- **Prioridad:** 🟡
- **Archivos clave:** `scripts/docs/check-secrets.mjs` (L323/L325), `scripts/docs/probe-secrets.mjs` (L21)
- **Verificación real:** ✅ CÓDIGO-REAL — `gh api .../secret-scanning/alerts` → alerts #14, #10, #8: **claves de ejemplo** dentro de los scripts que TESTEAN el detector de secretos (los mismos falsos positivos desbloqueados durante los pushes del release)
- **Gate Justificación:** son fixtures de test del propio scanner; el triage "used_in_tests" es la resolución correcta y documentada por GitHub
- **Gate Result:** ✅ DO
- **Contrato:** `gh api '/repos/ness-e/Vantadb/secret-scanning/alerts?state=open' --jq 'length'` **= 0** (resueltas `used_in_tests` con comentario)
- **Pre-mortem:** (1) resolución incorrecta rompe el registro → usar `used_in_tests` exacto; (2) futuras subidas de los mismos fixtures re-abren alerts → nota en la fila + posible `paths` ignore del scanner
- **Stop conditions:** la API rechaza la resolución → escalar al owner
- **Risk Register:** | 🟢×🟢 | resolución incorrecta | verificar `resolution=used_in_tests` | 1 intento | | 🟡×🟢 | re-apertura futura | nota + evaluar ignorar paths de test | próxima edición de fixtures |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) resolución incorrecta; (2) re-apertura; (3) scope creep (limpiar fixtures)
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = 0 open verificadas por comando · commit = n/a · release = n/a
- **Validación Appetite:** 1h ≥ 15min ✓
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/PROC-03.md`

### Task 3: FIND-230 — Gate mecánico de versiones npm (anti skip-silencioso del tren)

- **Appetite:** 1d
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟠
- **Archivos clave:** `.github/workflows/gate-api-docs.yml` (job "Check API Docs Version"), `vantadb-ts/package.json`, `vantadb-node/package.json`, `.github/workflows/release-npm-61.yml` (skip visible), `docs/dev/workflow/PUBLISH.md`
- **Verificación real:** ✅ CÓDIGO-REAL — el job "Check API Docs Version" ya valida `openapi.yaml` + `MCP.md` contra `[workspace.package] version` (bash, verificado en logs del PR #228); `vantadb-ts/package.json` existe y su versión se desincronizó (0.7.0 vs workspace) causando el skip silencioso del publish (evidencia: run 37082820490 + registry npm)
- **Gate Justificación:** el gap ya mordió una vez (npm TS quedó 0.7.0 en el release 0.8.0); el gate es la prevención mecánica (mismo patrón que el gate de docs que ya existe)
- **Gate Result:** ✅ DO
- **Contrato:** con `vantadb-ts/package.json` alterado a una versión ≠ workspace → el check local **FALLA** con mensaje claro (archivo + versión esperada); restaurado → **PASA**. El job de CI corre el mismo script. Bonus: el "already published → skipping" del publish emite `::warning::` visible en vez de skip mudo.
- **Pre-mortem:** (1) el gate rompe PRs de release-plz que aún no bumpearon npm → documentar el orden en PUBLISH.md (el bump npm va ANTES del Release PR); (2) vantadb-node nunca publicado → gate solo exige `== workspace` o `== 0.0.0/not-published` para node (no bloquear por algo que jamás se publicó); (3) bash vs node → extraer a `scripts/docs/check-npm-versions.mjs` (o `.ps1`/`.sh`) reutilizable local+CI
- **Stop conditions:** el gate genera falsos positivos >2 veces → re-diseñar (warning-only) y registrar
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | bloquea Release PRs legítimos | documentar orden en PUBLISH.md + mensaje accionable | al primer bloqueo |
  | 🟡×🟢 | vantadb-node sin tren | regla node = versión de workspace con excepción "nunca publicado" | diseño |
  | 🟢×🟡 | script no reutilizable local | extraer a script en `scripts/docs/` (no inline-only) | diseño |

- **Cynefin:** 🟨 complicado — decisión de diseño: qué paquetes exige (ts sí / node exceptuado) y dónde corre
- **Top 3 riesgos:** (1) falsos positivos en release; (2) semántica node; (3) mantenibilidad del check
- **Uphill/Downhill:** ⬇️ (4 steps definidos)
- **DoD:** task = contrato pasa + job en CI · commit = conventional + verify · release = PUBLISH.md actualizado (mismo PR)
- **Validación Appetite:** 1d ≥ 4-6h ✓
- **Estado:** ⬜ PENDING
- **Task file:** `docs/dev/tasks/FIND-230.md`

### Task 4: PROC-04 — Pase de progreso post-release (cierres + registro 0.8.0)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟢
- **Archivos clave:** `docs/dev/Backlog.md` (FIND-184 ✅, FIND-229 ✅), `docs/dev/avance/` (tree canónico)
- **Verificación real:** ✅ CÓDIGO-REAL — FIND-184 y FIND-229 quedaron marcadas ✅ en el backlog sin migrar al avance (verificado en las ediciones de esta sesión); `docs/dev/avance/activo/ci-cd.md` ya tiene la entrada "Docker & packaging — retired" pendiente de reconciliar
- **Gate Justificación:** higiene del registro que el propio sistema exige al cerrar tareas (skill progreso Trigger 1); evita drift backlog↔avance
- **Gate Result:** ✅ DO
- **Contrato:** `rg "FIND-184|FIND-229" docs/dev/Backlog.md` **= 0 filas** (migradas y removidas) + `docs/dev/avance/` con las entradas del release 0.8.0 (última: npm TS backfill + rustls ARM64) + `pwsh scripts/validate-docs-coverage.ps1` exit 0
- **Pre-mortem:** (1) el skill progreso sobreescribe a mano → usarlo en modo propuesto; (2) duplicar entradas → revisar el tree antes de escribir
- **Stop conditions:** el skill progreso falla 2× → dejar las filas marcadas ✅ + nota para el owner
- **Risk Register:** | 🟢×🟢 | duplicación | revisar avance antes | 1 vez | | 🟡×🟢 | sobreescritura | modo propuesto + diff | 1 vez |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) duplicados; (2) sobreescritura; (3) cobertura docs
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = contrato verde · commit = conventional · release = n/a
- **Validación Appetite:** 1h ≥ 30min ✓
- **Estado:** ⏳ EN PROGRESO
- **Task file:** `docs/dev/tasks/PROC-04.md`

### Task 5: FIND-228 — Dedupe de triggers CI (drop `develop` de `push.branches` en 15 workflows)

- **Appetite:** 1d
- **Esfuerzo:** 🟡 3-5h
- **Prioridad:** 🟠
- **Archivos clave:** 15 workflows (`chaos`, `ci-ai-ides-demo`, `ci-examples`, `ci-frameworks-demo`, `ci-rustdoc`, `desktop`, `gate-api-docs`, `gate-doc-examples`, `gate-docs-links`, `gate-docs-secrets`, `gate-docs`, `icp02-privacy-demo`, `injection-governance-demo`, `providers-ci`, `wal-verify-demo`) + `docs/dev/workflow/TRIGGERS.md` + `docs/dev/workflow/RULES.md` §1
- **Verificación real:** ✅ CÓDIGO-REAL — inventario completo del 2026-10-02 con los bloques `on:` extraídos uno a uno (15 violaciones de RULE 1 + `perf-bench` sin PR trigger = decisión aparte); la regla ya existe en RULES.md §1 ("Must not: list develop under push.branches")
- **Gate Justificación:** duplicados medidos (PR #233: ~80 checks vs ~45 esperados; ~15 workflow runs extra por push a develop con PR abierto); corrección de compliance, no cambio de política
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n 'branches:.*develop' .github/workflows -g '*.yml'` → **solo líneas bajo `pull_request:`** (los 15 procesos verificados por lectura) + `TRIGGERS.md` actualizado a la matriz nueva + `actionlint` exit 0 + un push de prueba a develop NO dispara los workflows arreglados
- **Pre-mortem:** (1) algún workflow depende del push[develop] para artefactos (revisar: perf-bench decidir; ci-rustdoc genera artifact — ¿preview en develop?) → evaluación por workflow; (2) romper el gate de docs en el PR de release → correr los gates locales + un PR real; (3) TRIGGERS.md desincronizado → actualizar en el mismo commit
- **Stop conditions:** un workflow deja de validar algo necesario (descubierto en el push de prueba) → revertir ese archivo y documentar la excepción
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | pérdida de cobertura en develop | revisar los 15 uno a uno + decisión explícita por workflow | diseño |
  | 🟢×🟡 | TRIGGERS.md stale | actualizar en el mismo commit | al cerrar |
  | 🟢×🟢 | perf-bench sin trigger PR | decidir mantener/schedule en la misma pasada | diseño |

- **Cynefin:** 🟨 complicado — 15 decisiones de "dónde pertenece develop"
- **Top 3 riesgos:** (1) cobertura perdida; (2) docs desincronizadas; (3) efecto en PRs de release
- **Uphill/Downhill:** ⬇️ (4 steps: inventario✅ → edición → TRIGGERS.md → validación)
- **DoD:** task = contrato + push de prueba · commit = `ci:` conventional · release = TRIGGERS.md/RULES coherentes
- **Validación Appetite:** 1d ≥ 3-5h ✓
- **Estado:** ⬜ PENDING
- **Task file:** `docs/dev/tasks/FIND-228.md`

### Task 6: FIND-231 — Cubrir el combo release (`server`+allocator, `-D warnings`) en CI

- **Appetite:** 1d
- **Esfuerzo:** 🟡 3-4h
- **Prioridad:** 🟠
- **Archivos clave:** `.github/workflows/ci-rust.yml` (job nuevo), `.github/workflows/release-binaries.yml` (referencia de comandos L117-118)
- **Verificación real:** ✅ CÓDIGO-REAL — `src/sdk/search/{mod.rs,fusion.rs}` ya tienen el `cfg(debug_assertions)` del fix `9004c43f`; el combo que rompió: `cargo build --release --features "server,$ALLOC --bin vanta-cli"` + `-D warnings` (default de setup-rust-toolchain) + `-p vantadb-server --features $ALLOC`. Ningún job actual lo compila (los tests van en debug)
- **Gate Justificación:** previene la clase de bug FIND-229 (primer run de release-binaries = fallo total); un job de 2-4 min evita un ciclo de release quemado
- **Gate Result:** ✅ DO
- **Contrato:** el job nuevo **verde** en CI con el árbol actual Y **rojo** si se revierte el fix (`git stash` del `cfg` en un scratch local reproduce `E0080`/unused-imports — dejar la reproducción documentada en el task file)
- **Pre-mortem:** (1) costo CI (+3-4 min en cada PR) → ubicarlo en el batch de jobs existente con cache (sccache ya configurado); (2) el combo exacto varía por OS (Windows=mimalloc / Linux-macOS=jemalloc) → matriz mínima 1 OS (ubuntu, el que falló) o los 2
- **Stop conditions:** el job tarda >5 min warm → reducirlo (check, no build full)
- **Risk Register:** | 🟡×🟢 | tiempo CI | `cargo check` (no build) + sccache | al medir | | 🟢×🟡 | combo incompleto | replicar exactamente los features del release | diseño |
- **Cynefin:** 🟨 complicado — calibrar el slice mínimo que cubre el fallo real
- **Top 3 riesgos:** (1) costo CI; (2) combo incompleto; (3) falsa sensación de cobertura
- **Uphill/Downhill:** ⬇️ (3 steps)
- **DoD:** task = contrato (verde + reproduce el rojo) · commit = `ci:` · release = n/a
- **Validación Appetite:** 1d ≥ 3-4h ✓
- **Estado:** ⬜ PENDING
- **Task file:** `docs/dev/tasks/FIND-231.md`

### Task 7: FIND-232 — Diagnóstico del perf-bench crónico (6/6 rojos desde 2026-09-25)

- **Appetite:** 1d
- **Esfuerzo:** 🟡 4-6h (diagnóstico; el fix puede destaparlo)
- **Prioridad:** 🟠
- **Archivos clave:** `.github/workflows/perf-bench.yml`, `benchmarks/compare_baseline.py`, `benchmarks/python_baseline.json`
- **Verificación real:** ✅ CÓDIGO-REAL — histÃ³rico `gh run list --workflow=perf-bench.yml`: **6/6 fallos** (main y develop, desde el 2026-09-25); log del último run: `query_text.p99_ms: 0.05 vs baseline 0.01 (+755.7%)` hard-error; p50/p95 en "noise-band quarantined"
- **Gate Justificación:** señal de performance del producto apagada desde hace una semana; todo release mide "verde" sin este dato; riesgo de regresión silenciosa
- **Gate Result:** ✅ DO
- **Contrato:** (a) `gh run list --workflow=perf-bench.yml --limit 1` = **success** tras fix/re-baseline documentado, o (b) si es varianza estructural: banda µs revisada + decisión registrada y **run verde** con el nuevo baseline
- **Pre-mortem:** (1) la "regresión" es varianza de runner en métricas de 0.01ms → re-baseline + banda por percentil; (2) baseline desactualizado por el ciclo 0.8.0 (search cambió: WIRE-08/SCH) → regenerar con `update_baseline=true` (input del workflow) y revisar el delta; (3) el fix real es de motor (hot path) → FIND a vanta-tuner, no forzar en esta task
- **Stop conditions:** si el diagnóstico implica un cambio de motor >1d → cerrar con FIND nuevo para vanta-tuner y dejar perf-bench en re-baseline
- **Risk Register:** | 🟡×🔴 | regresión real enmascarada por "varianza" | comparar 2 corridas del mismo commit + revisar diffs del ciclo 0.8.0 | diagnóstico | | 🟡×🟢 | re-baseline tapa un problema real | documentar delta por métrica | decisión |
- **Cynefin:** 🟧 complejo — causa-efecto emerge al experimentar (correr con/sin cambios)
- **Top 3 riesgos:** (1) regresión real; (2) re-baseline que enmascara; (3) deriva a fix de motor grande
- **Uphill/Downhill:** ⬆️ 1 incógnita (causa raíz) → se resuelve en el step 1 (reproducción + diff de baseline) → luego ⬇️
- **DoD:** task = contrato (run verde o decisión documentada) · commit = según fix · release = n/a
- **Validación Appetite:** 1d ≥ 4-6h ✓
- **Estado:** ⏳ EN PROGRESO
- **Task file:** `docs/dev/tasks/FIND-232.md`

### Task 8: FIND-225 — Triage de las 48 alertas CodeQL (6 critical + 42 high)

- **Appetite:** 3d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** alertas en GitHub code-scanning; afectados: `vanta-memory/tests/dreaming.rs`, `vantadb-mcp/tests/*`, `tests/certified_delete.rs`, `scripts/docs/*.mjs`
- **Verificación real:** ✅ CÓDIGO-REAL — `gh api .../code-scanning/alerts?state=open` → 48 (severidad: 6 critical + 42 high); los 6 critical = "hard-coded value used as a salt" en tests de dreaming (fixtures deterministas); el resto = logging de session keys/cert en tests MCP + regex/escapes en scripts de docs. Ninguna en `src/` de producción
- **Gate Justificación:** deuda de seguridad visible en la pestaña "Security" del repo; triage acotado (dismiss justificado o fix menor) — el reviewer natural es vanta-audit
- **Gate Result:** ✅ DO
- **Contrato:** `gh api '/repos/ness-e/Vantadb/code-scanning/alerts?state=open' --jq 'length'` **= 0** — cada resolución con comentario que cite el motivo (used-in-tests / false-positive con archivo:línea / fixed con commit)
- **Pre-mortem:** (1) CodeQL re-evalúa y re-abre alerts en el próximo push → dismiss con estado `used in tests` es estable; (2) algún alert NO es de test (revisar cada uno ANTES de dismiss) → los reales van a FIND/fix; (3) el triage se hace a mano para 48 → agrupar por regla+archivo (los ~6 grupos ya identificados)
- **Stop conditions:** >3 alertas dudosas que requieran análisis profundo → pausa + delegar el resto a vanta-audit con el listado
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟡 | dismiss de algo real | revisar cada grupo antes; producción primero | triage |
  | 🟡×🟢 | re-apertura | resoluciones estándar de GitHub | próximo push |
  | 🟢×🟢 | volumen | agrupar por rule.id + path | inicio |

- **Cynefin:** 🟨 complicado — clasificación correcta por alerta
- **Top 3 riesgos:** (1) dismiss incorrecto; (2) re-apertura; (3) volumen
- **Uphill/Downhill:** ⬇️ (3 steps: agrupar → revisar 6 grupos → resolver)
- **DoD:** task = 0 abiertas · commit = n/a (API) o fixes puntuales · release = n/a
- **Validación Appetite:** 3d ≥ 1-2d ✓
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-225.md`

### Task 9: FIND-226 — ASan: símbolos + triage de los leaks (1.81 GB)

- **Appetite:** 3d
- **Esfuerzo:** 🔴 1-2d (triaje + job; fixes reales según hallazgo)
- **Prioridad:** 🟡
- **Archivos clave:** `.github/workflows/ci-rust.yml` (job `sanitizer-asan`), `src/lsm.rs:156,263`, `src/storage/vfile.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — job ASan con leak-report sin símbolos (solo direcciones); dominante: ~28 × 64 MB exactos = vfiles LSM no liberados al salir los tests; + ~1000 allocs chicas; pre-existente (mismo patrón el 2026-09-25 en main)
- **Gate Justificación:** job best-effort rojo y sin señal útil (sin símbolos no hay triage posible); la mejora del job (symbolizer) es de bajo riesgo y alto valor; FIND-213 (NL_POOL) puede ser parte de los leaks
- **Gate Result:** ✅ DO
- **Contrato:** (1) el job ASan corre con `llvm-symbolizer` disponible → el próximo reporte tiene símbolos; (2) clasificación documentada: test-exit/estructural vs leaks reales; (3) leaks reales → fix o supresión LSAN documentada; decisión registrada en FIND-226
- **Pre-mortem:** (1) ASan sigue rojo por diseño (best-effort) → el contrato es "símbolos + clasificación", no "verde"; (2) los leaks son del harness de test (engine sin drop a propósito) → supresión documentada con justificación; (3) el fix real es profundo (engine drop) → FIND a vanta-worker/engine
- **Stop conditions:** >1d en un solo leak profundo → FIND específico y cerrar el triage general
- **Risk Register:** | 🟡×🟢 | símbolos no resuelven (build sin debug) | verificar flags del job (`-Cdebuginfo`) | step 1 | | 🟡×🟡 | supresión amplia tapa leaks reales | supresión por clase específica + nota | decisión |
- **Cynefin:** 🟨 complicado — requiere el reporte simbolizado para decidir
- **Top 3 riesgos:** (1) símbolos insuficientes; (2) supresión amplia; (3) deriva a fix de motor
- **Uphill/Downhill:** ⬆️ 1 incógnita (alcance real de los leaks) → se resuelve con símbolos
- **DoD:** task = contrato · commit = cambio de job y/o fix · release = n/a (best-effort)
- **Validación Appetite:** 3d ≥ 1-2d ✓
- **Estado:** ⬜ PENDING
- **Task file:** `docs/dev/tasks/FIND-226.md`

### Task 10: FIND-227 — TSan: supresión targeted o decisión documentada

- **Appetite:** 1d
- **Esfuerzo:** 🟢 3-4h (+1 ciclo CI)
- **Prioridad:** 🟢
- **Archivos clave:** `.github/workflows/ci-rust.yml` (job `sanitizer-tsan`), posible `tsan.supp` nuevo
- **Verificación real:** ✅ CÓDIGO-REAL — race 100% en `std::sync::mpmc` + test harness (frames: `mpmc::list::Channel`, `test::event::CompletedTest`, `compiler_copy`); cero frames de VantaDB; crónico (rojo también en main 2026-09-25)
- **Gate Justificación:** job best-effort; el rojo actual es ruido de std/libtest; una supresión targeted (o decisión explícita) convierte el job en señal accionable
- **Gate Result:** ✅ DO
- **Contrato:** TSan **verde** con `suppressions=tsan.supp` (patrón del mpmc/libtest documentado en el archivo) verificado en 1 run de CI — o si la supresión sobre-suprime: decisión "status quo (best-effort)" registrada en FIND-227 con la evidencia del análisis
- **Pre-mortem:** (1) supresión demasiado amplia (`race:memcpy` global) → acotar por función (`compiler_copy`, `mpmc`); (2) TSan tiene más falsos positivos nuevos → decidir por lotes en el mismo ciclo; (3) el próximo nightly de Rust lo arregla → igual dejar la decisión documentada
- **Stop conditions:** 2 intentos de supresión que no convergen → decisión "status quo" documentada y cerrar
- **Risk Register:** | 🟡×🟡 | supresión sobre-ancha | patrón específico + revisión del diff | diseño | | 🟢×🟢 | std lo arregla solo | la decisión documentada cubre ambos caminos | future |
- **Cynefin:** 🟨 complicado — calibrar la supresión
- **Top 3 riesgos:** (1) sobre-supresión; (2) más FPs; (3) deriva de tiempo
- **Uphill/Downhill:** ⬇️ (3 steps)
- **DoD:** task = contrato/decsión · commit = `ci:` (suppression) · release = n/a
- **Validación Appetite:** 1d ≥ 3-4h ✓
- **Estado:** ⬜ PENDING
- **Task file:** `docs/dev/tasks/FIND-227.md`

## DEFER

| ID | Por qué DEFER |
|----|----------------|
| FIND-219 | Matriz completa de combinaciones de features (🟠 1-2sem) — FIND-231 cubre el slice urgente (el combo que rompió); la matriz completa es una campaña propia |
| FIND-213 | `NL_POOL` leak (🟠 1d) — se absorbe en el triage de FIND-226 si aparece en las pilas simbolizadas; si no, repriorizar |
| Backlog no-release (FIND-180..193, H3-xx, MEMG, WIRE-14, etc.) | Fuera de la campaña: no relacionadas al release 0.8.0 ni a la infraestructura de CI |

## BLOQUEADO

| ID | Bloqueante |
|----|-----------|
| PR **#238** (`chore(vantadb): release v0.9.0`) | **Decisión del owner**: cuándo es el próximo release. Requiere además **curación del changelog** (el squash del 0.8.0 hace que los 184 commits del ciclo reaparezcan como "nuevos" → el changelog de 0.9.0 duplica entradas del 0.8.0; la política de merge-commits nueva evita la recurrencia) |
| **MKT-20** (adapters PyPI) | Acción del owner (tarea externa al pipeline) |

## Notas

- plan-adjust 2026-10-02: plan creado post-release 0.8.0. Fuente = backlog (FIND-225..232) + PRs abiertos + hallazgos de la verificación del release. ⬆️ uphill inicial = 2 (perf-bench causa raíz; alcance ASan). ⬇️ downhill = 33 steps.
- Los planes referenciados por task files se crean bajo demanda en `docs/dev/tasks/<ID>.md` (Fase de Discovery del pipeline).
- OCR gate: toda tarea ✅ DO cierra con OCR delegation review (pipeline-full.md §Cierre paso 5).

=== RECITATION PROC-03 ===
Campaign ID: post-release-0.8.0-20261002
Objetivo activo: PROC-03 — Descartar las 3 alertas de secret scanning (falsos positivos de test)
Estado: completed
Última acción: 3 alertas resueltas used_in_tests + contrato verificado (open=0) + review fresco APPROVE
Resultado: COMPLETED
Próxima acción: FIND-232 — claim + spawn vanta-tuner (perf-bench crónico)
Contrato: open=0 verificado por comando; review P2-01 approve
Próxima tarea si completa: FIND-232
=== END RECITATION ===

=== RECITATION FIND-225 ===
Campaign ID: post-release-0.8.0-20261002
Objetivo activo: FIND-225 — Triage de las 48 alertas CodeQL (6 critical + 42 high)
Estado: completed
Última acción: 48 dismissals (40 used_in_tests + 8 false_positive) + contrato open=0 + review fresco APPROVE
Resultado: COMPLETED
Próxima acción: Continuar run: PROC-04 (docs) en vuelo; W1 = FIND-230/FIND-226/FIND-227
Contrato: code-scanning alerts open=0 verificado por comando; review P2-01 approve (fresh)
Próxima tarea si completa: PROC-04
=== END RECITATION ===

=== RECITATION FIND-232 ===
Campaign ID: post-release-0.8.0-20261002
Objetivo activo: FIND-232 — Diagnostico del perf-bench cronico (13 rojos consecutivos desde 2026-09-25)
Estado: in-progress
Última acción: Discovery cerrado con evidencia dura: causa raiz = mismatch estructural push 1000/100 vs baseline 10000/1000 (mismo commit 114f55f0: push 36094025517 rojo vs dispatch 36094025761 verde; BENCHMARKS.md/README documentan 10000/1000; p99 con n=100 = maximo muestral a escala us) + insert.p99 tail. Descarte de regresion de motor. Fix en edicion. Dispatch diagnostico 37088714140 corriendo.
Resultado: PARTIAL
Próxima acción: Editar .github/workflows/perf-bench.yml (perfil alineado), benchmarks/compare_baseline.py (guarda de perfil + piso absoluto + self-test 5 casos), benchmarks/README.md (ref stale); verify local self-test; commit LOCAL.
Contrato: verificacion: python benchmarks/compare_baseline.py --self-test (pendiente post-edit); gh run list --workflow=perf-bench.yml (contrato final post-push)
evidencia:
- claim: Baseline es 10000/1000 y push corria 1000/100 | evidencia: benchmarks/python_baseline.json (total_records 10000) + logs push 'Dataset Size : 1000 vectors / Queries : 100' vs dispatch '10000/1000' (runs 37082038748 / 36094025761) | confianza: alta
- claim: Mismo commit 114f55f0 push rojo (+92.5% hybrid) vs dispatch verde; delta por perfil, no motor | evidencia: run 36094025517 (failure) vs run 36094025761 (success) + artifacts benchmark_results.json | confianza: alta
- claim: p99 con n=100 es el maximo de la muestra | evidencia: benchmarks/vantadb_local_bench.py:43-45 (int(n*0.99) con n=100 -> 99) | confianza: alta
- claim: 13/13 runs rojos tienen query_text.p99 como bloqueante | evidencia: gh run view <13 runs> --log-failed (tabla en docs/dev/tasks/FIND-232.md) | confianza: alta
- claim: Varianza mismo codigo/perfil +-25% en colas | evidencia: artifacts 36093538630 vs 36094025761 (insert.p99 -28.4%, hybrid.p50 -22.8%) | confianza: alta
artefactos: docs/dev/tasks/FIND-232.md; C:\Users\Eros\AppData\Local\Temp\opencode\find232\{metrics.txt, artifacts}
invariantes: No tocar opencode.jsonc (WIP ajeno); commit LOCAL sin push; no re-baseline preventivo sin documento de delta; no cambios de motor en esta task
deuda: FIND-232-R1 diferido: banda por familia tail (p95/p99) solo con >=2 falsos positivos post-fix y datos alineados
queda_pendiente: Orquestador: push develop -> verificar run verde (contrato a) o re-baseline documentado (contrato b)
Próxima tarea si completa: FIND-225
=== END RECITATION ===
