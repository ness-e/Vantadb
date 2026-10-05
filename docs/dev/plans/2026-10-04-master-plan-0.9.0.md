---
title: "Plan Maestro: Release 0.9.0 + Memoria 1.0 (post-0.8.0)"
kind: plan
description: "SDP v3 (taskType mixto): campaign-executor · progreso · planning-and-task-breakdown · ci-cd-and-automation · shipping-and-launch · documentation-and-adrs · source-driven-development · git-workflow-and-versioning — F0 release-ready (21 tareas full-detail) + F1-F5 campaña (profundización por fase)"
---

# Plan Maestro: Release 0.9.0 + Memoria 1.0

> **Campaign ID:** master-plan-0.9.0-20261004
> **Inicio:** 2026-10-04
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/dev/Backlog.md` (DELTA 2026-09-30 + Alta 2026-10-01 + hallazgos FIND-233..239) + cierre de `docs/dev/plans/2026-09-28-docs-consolidation.md` (F1/F2) + validación externa v0.8.0 (2026-10-03)
> **Autonomous:** false

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 72 |
| 🟡 DEFER | 3 |
| ❌ SKIP | 1 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 6 incógnitas abiertas (repro exacto DX-01 en Node 26; approach de locks WSM-15; alcance de exposición DIST-02/03; hallazgos de las auditorías DUR-01/02) · ⬇️ downhill = ~250 steps pendientes (F0: 21 tareas full-detail · F1-F5: listas compactas que se profundizan al nivel F0 al iniciar cada fase — regla del master-roadmap 2026-09-26).

## SDP (skills del plan)

`campaign_discover_skills_v2` phase=PLAN → base: **source-driven-development · campaign-executor · progreso** · lifecycle: **planning-and-task-breakdown** · keyword-mapped: **ci-cd-and-automation · shipping-and-launch · documentation-and-adrs · git-workflow-and-versioning · security-and-hardening · test-driven-development · spec-driven-development · performance-optimization** (≤10 por sub-agente; el sub-agente corre su propio SDP en DISCOVERY).

## Reglas de ejecución (para los sub-agentes — prompts §6.f)

1. **Profundidad unificada:** cada tarea corre en un sub-agente que sigue `pipeline-full.md` (DISCOVERY → EJECUCIÓN → CIERRE), con el prompt de 10 bloques de `pipeline-run.md` §6.f + el "Contexto verificado del plan" copiado de este archivo (Gate Justificación, Pre-mortem, Appetite, Wave).
2. **Skills:** el sub-agente ejecuta su SDP (`campaign_discover_skills_v2` phase=BUILD) y carga las skills útiles (**máx 10**), declarando `SKILLS_CARGADAS:` en el RESULTADO. Cada tarea lista "Skills sugeridas" como piso, no techo.
3. **MCPs obligatorios:** `codegraph_codegraph_explore` (blast radius), `codebase-memory-mcp` (`detect_changes`, `check_index_coverage`, `get_architecture`), `campaign_*` (`get_next_task`/claim, `verify_cmd`, `update_task_state`, `session_track`). Grep solo si codegraph/CBM no cubren.
4. **Task files canónicos:** `docs/dev/tasks/<ID>.md` con el formato de `task.md` (Impacto Regla 0, Contrato, Steps atómicos ~100 líneas, Invariantes, Review P2-01, Context Save Point, RESULTADO §7). Sin task file no hay cierre.
5. **Cierre por tarea:** verify mecánico por step (`campaign_verify_cmd`) → verify full → OCR delegation (`dev-tools/ocr-review.ps1`; Critical/High bloquean) → review P2-01 por agente distinto → commit **LOCAL** conventional con task ID. **NUNCA push** (solo con instrucción explícita del owner).
6. **Profundización por fase:** F0 está full-detail (ejecutable ya). F1 medium (contrato + archivos; se completa al iniciar). F2-F5 compactas — al iniciar cada fase se expanden al nivel F0 **antes** de ejecutar (gate de fase explícito).
7. **WIP:** MAX_WIP=3 (waves); `FAIL_MODE=parallel` por defecto.

## F0 — Release-ready (antes de 0.9.0)

> **Gate de salida F0:** las 21 tareas cerradas + release 0.9.0 publicado (decisión #238 + changelog curado) + verificación post-release. Las tareas 1-13 son fixes/DX de cara al usuario; 14-19 son la pata de distribución P0; 20-21 cierran el plan de docs anterior.

### Task 1: FIND-237 — CLI `migrate`: aceptar el global `--db` como fallback del `target` posicional

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `src/cli.rs:423-450` (MigrateCommand), `src/bin/vanta-cli.rs:204-222` (dispatch)
- **Verificación real:** ✅ CÓDIGO-REAL — validación externa v0.8.0 (2026-10-03): `vanta-cli migrate check --db X` falla; `--db` ES global (`cli.rs:20-26`, `global=true`, env `VANTADB_STORAGE_PATH`) pero `Check { target: String }` es posicional obligatorio (`cli.rs:446-449`) → clap no lo resuelve.
- **Gate Justificación:** UX real medida por un tester externo; fix acotado a 1 archivo + help; elimina la confusión del único comando con convención distinta.
- **Gate Result:** ✅ DO
- **Contrato:** `vanta-cli migrate check --db <db>` → exit 0 (fallback al global) **y** `vanta-cli migrate check <TARGET>` sigue funcionando (positional gana); `--help` muestra ejemplo de uso; tests CLI verdes.
- **Pre-mortem:** (1) cambio de firma rompe scripts existentes → `target` queda `Option<String>` con precedencia positional > `--db`; (2) target y db resuelven paths distintos → unificar resolución en el handler; (3) clap no muestra ejemplos en help → usar `after_help`.
- **Stop conditions:** 2 iteraciones sin contrato verde → registrar approach alternativo (alias `--target`) y cerrar decisión.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Regresión de invocación existente | Positional-first; test de ambos caminos | VERIFY |
  | 🟢×🟡 | `target` vs `--db` divergen | Test dedicado de precedencia | diseño |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) regresión CLI; (2) ambigüedad target/db; (3) help stale.
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = contrato + tests · commit = `fix(cli):` + verify · release = entrada de changelog.
- **Validación Appetite vs Effort:** 1d ≥ 2h ✓
- **Skills sugeridas:** campaign-executor · source-driven-development · test-driven-development · git-workflow-and-versioning
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-237.md`

### Task 2: FIND-238 — WASM/npm: silenciar logs DEBUG de `Client.create()` (`tracing-wasm` default-on sin filtro)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 2-3h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-wasm/Cargo.toml:29,42` (`default = ["tracing-wasm"]`), `vantadb-wasm/src/lib.rs:1935-1939` (`set_as_global_default()` sin nivel), `src/config.rs:1004+` (origen de los `debug!`)
- **Verificación real:** ✅ CÓDIGO-REAL — validación externa v0.8.0 (2026-10-03): logs verbosos en Node al crear el cliente; root cause verificado: feature en default + init sin filtro.
- **Gate Justificación:** ruido en la primera impresión del paquete npm (la superficie de descubrimiento); fix de 1-2 líneas + decisión de nivel.
- **Gate Result:** ✅ DO
- **Contrato:** `Client.create()` con el wasm build no emite líneas `DEBUG` por defecto (nivel default WARN/INFO o gate por env documentado); smoke del paquete (test en `vantadb-ts`) sin DEBUG; sin cambio funcional.
- **Pre-mortem:** (1) bajar el nivel oculta logs útiles de soporte → gate por env (patrón `RUST_LOG`) documentado en README; (2) `set_as_global_default` no acepta nivel → usar `tracing_wasm::set_as_global_default_with_level` o `LevelFilter`; (3) romper el build wasm → verificar con `wasm-pack build` local.
- **Stop conditions:** el builder no expone filtro → decisión: sacar `tracing-wasm` de default (opt-in) + nota en README.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟡 | Logs útiles ocultos | Gate por env documentado | diseño |
  | 🟢×🟢 | API de tracing-wasm distinta | Validar contra docs del crate | DISCOVERY |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) ocultar señal; (2) API del crate; (3) build wasm roto.
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = contrato + smoke · commit = `fix(wasm):` · release = changelog.
- **Validación Appetite vs Effort:** 1d ≥ 3h ✓
- **Skills sugeridas:** campaign-executor · source-driven-development · ci-cd-and-automation · documentation-skill
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-238.md`

### Task 3: FIND-239 — Docs DX: encoding Windows + puente de formatos de import + ejemplo `get_node` + caveats WASM

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 3-4h
- **Prioridad:** 🟡
- **Archivos clave:** `docs/user/QUICKSTART.md`, `docs/api/PYTHON_SDK.md` (§bulk_import L716-730, §get_node L447-456, §export_all L835), `vantadb-ts/README.md` (tablas Maintenance/Export)
- **Verificación real:** ✅ CÓDIGO-REAL — validación externa v0.8.0 (2026-10-03): (a) 0 notas UTF-8/chcp en README/QUICKSTART; (b) PYTHON_SDK no cruza `export_all`(JSONL)↔`import_file` vs `bulk_import`(binario `.vdbdump`, spec en EMBEDDED_SDK.md:94/102); (c) shape `fields.content` solo en docstring `lib.rs:651`; (d) tablas TS listan métodos fs sin caveat.
- **Gate Justificación:** 4 gaps de docs medidos en flujo usuario-real; docs-only, cero riesgo; mejora directa del "funnel de 60 segundos".
- **Gate Result:** ✅ DO
- **Contrato:** (a) QUICKSTART/README con nota de encoding Windows (`chcp 65001` / `PYTHONIOENCODING=utf-8`); (b) PYTHON_SDK cruza formatos: `export_all`→`import_file` (JSONL) vs `bulk_import_bytes` (`.vdbdump`, link a EMBEDDED_SDK) + hint en el error de magic (si aplica); (c) ejemplo explícito `node["fields"]["content"]`; (d) caveat por método en las 2 tablas TS. `check-links`/`check-docs` exit 0.
- **Pre-mortem:** (1) tocar el error de magic = cambio de código Rust → separar: docs ahora, hint como sub-step opcional; (2) ejemplo que no corre → validar contra el SDK real (patrón F2-T1); (3) links relativos rotos → gates.
- **Stop conditions:** el hint de error requiere cambio no trivial → dejarlo como FIND derivado y cerrar docs.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟡 | Ejemplo inválido | Probar el snippet contra SDK real | VERIFY |
  | 🟢×🟢 | Scope creep a código | Sub-step opcional + FIND derivado | diseño |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) ejemplo no ejecutable; (2) scope creep; (3) links.
- **Uphill/Downhill:** ⬇️ (4 steps)
- **DoD:** task = contrato + gates · commit = `docs:` · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 4h ✓
- **Skills sugeridas:** documentation-skill · writing-guidelines · documentation-and-adrs · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-239.md`

### Task 4: FIND-233 — perf-bench: instrumento cross-VM (A/B same-job o calibración)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟠
- **Archivos clave:** `benchmarks/compare_baseline.py`, `.github/workflows/perf-bench.yml`
- **Verificación real:** ✅ CÓDIGO-REAL — medido en FIND-232 (2026-10-03): mismo método/código, 1.7-2.4x de varianza cross-VM (insert.p99 16.8x); bandas ya calibradas a colapsos; la señal fina (<2-3x) requiere entorno controlado.
- **Gate Justificación:** sin esto, el instrumento no ve regresiones <2-3x — el claim de performance queda ciego en la zona fina; decisiones (1) A/B same-job o (2) calibración CPU o (3) banda documentada.
- **Gate Result:** ✅ DO
- **Contrato:** decisión implementada de UNA de las 3 opciones + `perf-bench` estable (≥2 runs verdes consecutivos post-push) + approach documentado en `BENCHMARKS.md`/task file.
- **Pre-mortem:** (1) A/B same-job duplica el costo CI → medir y decidir con costo a la vista; (2) calibración CPU no representativa → documentar límites; (3) scope grande → stop-condition a las 6h.
- **Stop conditions:** >6h → decisión "banda documentada" (opción 3) y FIND derivado para A/B.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Costo CI x2 | Medir duración; opt-in nocturno | diseño |
  | 🟡×🟡 | Calibración inválida | Documentar límites medidos | decisión |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) costo; (2) calibración no representativa; (3) rabbit hole.
- **Uphill/Downhill:** ⬆️ 1 incógnita (qué opción conviene) → se resuelve en step 1 (medir) → luego ⬇️
- **DoD:** task = contrato + runs verdes · commit = `perf(ci):` · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 6h ✓
- **Skills sugeridas:** performance-optimization · ci-cd-and-automation · doubt-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-233.md`

### Task 5: FIND-234 — `check-avance-coverage.ps1` apunta a `docs/avance` inexistente

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Archivos clave:** `scripts/check-avance-coverage.ps1:10`
- **Verificación real:** ✅ CÓDIGO-REAL — detectado por PROC-04 (2026-10-03): `$dstDir = Join-Path $root "docs/avance"` stale tras migración 2026-08-23; imprime "0/237 (0.0%)" + error de ruta; sale 0 (no bloquea). La skill `progreso` lo referencia como check de cierre.
- **Gate Justificación:** lectura falsa de cobertura en el cierre de cada campaña; fix de 1 línea + re-verificación.
- **Gate Result:** ✅ DO
- **Contrato:** script apunta a `docs/dev/avance`; corre sin errores de ruta; reporte real (≠ "0/237" engañoso) sobre el árbol canónico.
- **Pre-mortem:** (1) otros paths stale en el mismo script → grep del script completo; (2) el reporte cambia números → documentar el antes/después.
- **Stop conditions:** >1h → dejar fix mínimo + nota.
- **Risk Register:** | 🟢×🟢 | más paths stale | revisión completa del script | DISCOVERY |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) paths adicionales; (2) lectura esperada por otros scripts; (3) —
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = contrato · commit = `fix(scripts):` · release = n/a.
- **Validación Appetite vs Effort:** 1h ≥ 1h ✓
- **Skills sugeridas:** campaign-executor · progreso
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-234.md`

### Task 6: FIND-235 — `docs/dev/workflow/ci-rust-10.md` stale (13 jobs vs 20; coverage 59% vs ~80%)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟢
- **Archivos clave:** `docs/dev/workflow/ci-rust-10.md`
- **Verificación real:** ✅ CÓDIGO-REAL — flagged por FIND-231 (2026-10-03, review ronda 1): el doc no refleja release-combo/npm gate/sanitizers.
- **Gate Justificación:** doc de referencia del workflow principal desactualizado; fix de conteos/listas + gates.
- **Gate Result:** ✅ DO
- **Contrato:** conteos reales (jobs/coverage) + links válidos; `check-docs`/`check-links` exit 0.
- **Pre-mortem:** (1) regenerar mal los conteos → derivarlos del YAML real; (2) drift recurrente → agregar nota "keep in sync" (o link al workflow).
- **Stop conditions:** —
- **Risk Register:** | 🟢×🟢 | re-drift | nota de sincronización | cierre |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) conteo mal derivado; (2) re-drift; (3) —
- **Uphill/Downhill:** ⬇️ (1 step)
- **DoD:** task = contrato · commit = `docs(ci):` · release = n/a.
- **Validación Appetite vs Effort:** 1h ≥ 30min ✓
- **Skills sugeridas:** documentation-skill · ci-cd-and-automation
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-235.md`

### Task 7: FIND-236 — Job ASan: excluir `sift1m_competitive_benchmark` (guard release-only en debug)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `.github/workflows/ci-rust.yml` (job `sanitizer-asan`)
- **Verificación real:** ✅ CÓDIGO-REAL — run `37102066714` (2026-10-03, post-push): el job rojo SOLO por panic de `competitive_bench.rs:64` ("must run with --release"); idéntico en el run pre-fix `111086294644` → pre-existente, NO causado por FIND-226 (leaks = 0 desde el fix).
- **Gate Justificación:** con los leaks arreglados, este ruido impide que el rojo del job signifique "leak real"; excluir el test o gatear por perfil.
- **Gate Result:** ✅ DO
- **Contrato:** el job ASan no ejecuta `sift1m_competitive_benchmark` (o lo gatea por perfil) → su rojo significa solo leaks; verificación en el próximo run (comando post-push en el task file); `actionlint` 0.
- **Pre-mortem:** (1) excluir de más y perder cobertura del test → solo ese test (release-only por diseño); (2) filtro de nextest mal escrito → validar expresión `-E 'not test(sift1m...)'`; (3) el job sigue rojo por otro test debug-only → documentar como FIND derivado.
- **Stop conditions:** >1h → dejar exclusión mínima documentada.
- **Risk Register:** | 🟡×🟢 | sobre-exclusión | excluir solo el test release-only | diseño | | 🟢×🟡 | otro ruido oculto | re-run y revisar | post-push |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) sobre-exclusión; (2) filtro mal escrito; (3) otro ruido.
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = contrato + actionlint · commit = `ci:` · release = n/a.
- **Validación Appetite vs Effort:** 1h ≥ 1h ✓
- **Skills sugeridas:** ci-cd-and-automation · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/FIND-236.md`

### Task 8: DX-01 — TS WASM: `get` revienta en Node 26 (bloquea la demo TS)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-wasm/src/lib.rs` (path `get`/memoria), `vantadb-ts/src/vantadb.ts`, CI de `vantadb-ts` (matriz Node)
- **Verificación real:** ✅ CÓDIGO-REAL — H-009: `put` OK pero `get` lanza `memory access out of bounds` en Node v26 win32 (`vantadb_wasm_bg.js:1551`); CI TS usa Node ≥22; traceback completo en H-009.
- **Gate Justificación:** bloquea la demo TS del binding publicado (superficie npm); repro existente; blast radius acotado al path de lectura wasm.
- **Gate Result:** ✅ DO
- **Contrato:** `get` funciona en Node 26 win32 (repro de H-009 pasa) **y** la matriz CI cubre Node 22 + 26 (job agregado) verde; sin regresión en Node 22; repro antes/después documentada en el task file.
- **Pre-mortem:** (1) el bug es del runtime Node 26 (no del binding) → repro mínima primero; si es upstream: workaround + doc + FIND; (2) fix de ownership de memoria wasm con blast radius a otros métodos → correr la suite wasm completa; (3) matriz CI duplicada costosa → un job adicional, no matrix completa.
- **Stop conditions:** 2d sin repro determinista → workaround documentado + FIND upstream; cerrar.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Fix incorrecto del ownership wasm | Repro mínima + suite wasm existente | VERIFY |
  | 🟡×🟡 | Bug upstream de Node 26 | Decidir workaround/doc con evidencia | DISCOVERY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) ownership mal arreglado; (2) causa upstream; (3) matriz CI.
- **Uphill/Downhill:** ⬆️ 1 incógnita (¿nuestro binding o Node 26?) → se resuelve en step 1 (repro) → luego ⬇️
- **DoD:** task = contrato · commit = `fix(wasm):` · release = entrada de changelog (bugfix público).
- **Validación Appetite vs Effort:** 3d ≥ 2d ✓
- **Skills sugeridas:** systematic-debugging · source-driven-development · rust-write-tests · ci-cd-and-automation · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DX-01.md`

### Task 9: WSM-15 — OPFS multi-pestaña: lock (hoy = corrupción silenciosa)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-wasm/src/opfs.rs`, referencia del patrón: `idb.rs:62` (`navigator.locks`)
- **Verificación real:** ✅ CÓDIGO-REAL — OPFS sin `navigator.locks` (IDB sí lo tiene, `idb.rs:62`); dos pestañas concurrentes = corrupción silenciosa; el patrón de fix ya existe en el repo.
- **Gate Justificación:** pérdida/corrupción de datos silenciosa en la superficie browser; el fix replica un patrón propio ya probado (IDB).
- **Gate Result:** ✅ DO
- **Contrato:** OPFS usa `navigator.locks` (o equivalente con degradación explícita si no está disponible) → repro multi-contexto (2 pestañas/workers) sin corrupción; sin regresión single-tab (suite wasm).
- **Pre-mortem:** (1) `navigator.locks` no disponible en todo contexto (worker) → validar + fallback fail-loud documentado; (2) lock global degrada concurrencia → lock por archivo/namespace; (3) romper flujo OPFS existente → tests existentes.
- **Stop conditions:** >2d → decisión: degradación explícita (fail-loud en multi-tab) documentada + FIND del lock real.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Fix incompleto deja corrupción | Repro multi-contexto antes/después | VERIFY |
  | 🟢×🟡 | Locks no disponibles en un contexto | Fallback fail-loud documentado | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) corrupción residual; (2) disponibilidad de locks; (3) regresión single-tab.
- **Uphill/Downhill:** ⬇️ (3 steps)
- **DoD:** task = contrato · commit = `fix(wasm):` · release = changelog.
- **Validación Appetite vs Effort:** 3d ≥ 2d ✓
- **Skills sugeridas:** systematic-debugging · source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/WSM-15.md`

### Task 10: DUR-03 — H-023: `put` sobre key expirada-sin-purgar muere con "Node ID collision"

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟠
- **Archivos clave:** `src/error.rs:207`, write path de `put` (`src/sdk/api/memory.rs`), lógica de purge/expiración
- **Verificación real:** ✅ CÓDIGO-REAL — H-023: mismo id en 3 corridas; `purge_expired()` lo resuelve; DB fresca pasa; workaround del demo: purge antes del seed.
- **Gate Justificación:** bug de motor reproducible que muerde a usuarios reales (un `put` legítimo muere); fix = upsert o purge-on-write (decisión documentada).
- **Gate Result:** ✅ DO
- **Contrato:** `put` sobre una key expirada-sin-purgar hace upsert (o purge-on-write) → exit 0; test de regresión (expirado→put→ok) verde; semántica para keys vivas sin cambio (suite existente).
- **Pre-mortem:** (1) el fix cambia semántica de expiración → decisión explícita (upsert vs purge-on-write) documentada; (2) race con el sweeper de TTL → test concurrente; (3) blast radius al write path → suite completa.
- **Stop conditions:** semántica ambigua → Gate D (question al owner) antes de codear.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Cambio de semántica de expiración | Documentar decisión en task file | diseño |
  | 🟡×🔴 | Race con TTL sweeper | Test concurrente dedicado | VERIFY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) semántica; (2) race; (3) regresión write path.
- **Uphill/Downhill:** ⬇️ (3 steps)
- **DoD:** task = contrato + test · commit = `fix(engine):` · release = changelog (bugfix público).
- **Validación Appetite vs Effort:** 2d ≥ 2d ✓
- **Skills sugeridas:** systematic-debugging · rust-write-tests · source-driven-development · doubt-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DUR-03.md`

### Task 11: DUR-01 — Auditoría del fsync real (WAL, snapshots, GC)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟠
- **Archivos clave:** `src/wal*.rs`, snapshots (`src/`), GC; inventario: `fsync` = 15 hits / `sync_all` = 7 en 93k LOC
- **Verificación real:** ✅ CÓDIGO-REAL — research §1 (2026-09-30): el claim "durable" puntuó 7.0 por cobertura de fsync dudosa.
- **Gate Justificación:** el claim central del producto ("durable") depende de dónde falta `fsync`; auditoría acotada con contrato de veredicto.
- **Gate Result:** ✅ DO
- **Contrato:** mapa path→fsync real (WAL, snapshots, GC) en el task file + gaps clasificados (fix ahora / FIND / OK-justificado) + **al menos 1 gap real resuelto o descartado con evidencia**; si hay fix de fsync: medido contra `canonical_p99` (Regla 9).
- **Pre-mortem:** (1) auditoría sin veredicto → contrato exige clasificación completa; (2) agregar fsyncs degrada p99 → medir antes/después; (3) scope creep a "arreglar todo" → fixes solo 1-línea o FIND.
- **Stop conditions:** >2d → cerrar mapa + FINDs derivados.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Degradar p99 con fsyncs nuevos | `cargo bench canonical_p99` before/after | si hay fix |
  | 🟢×🟢 | Mapa incompleto | Checklist de paths por storage | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) p99; (2) mapa incompleto; (3) scope creep.
- **Uphill/Downhill:** ⬆️ 1 incógnita (dónde falta fsync) → se resuelve en el mapa → luego ⬇️
- **DoD:** task = contrato · commit = `docs:`/`fix:` según hallazgo · release = n/a.
- **Validación Appetite vs Effort:** 2d ≥ 2d ✓
- **Skills sugeridas:** systematic-debugging · performance-optimization · source-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DUR-01.md`

### Task 12: DUR-02 — Auditoría cobertura AES (`encryption`): WAL / text_index / HNSW / edge_index

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟠
- **Archivos clave:** `src/crypto.rs` (22 hits), `src/storage/vfile.rs` (13), `src/config.rs` (12); feature `encryption`
- **Verificación real:** ✅ CÓDIGO-REAL — backlog (verificado HEAD 2026-10-01): "verificar que AES-256-GCM cubre WAL, text_index, HNSW y edge_index (no solo `put`); si no, el índice filtra plaintext".
- **Gate Justificación:** riesgo de fuga de plaintext en artefactos on-disk bajo feature `encryption`; verificación acotada con contrato de veredicto por artefacto.
- **Gate Result:** ✅ DO
- **Contrato:** mapa artefacto→cifrado (WAL, text_index, HNSW, edge_index, snapshots) con evidencia de código o test con tmpdir + gaps clasificados (fix / FIND / wontfix documentado); si hay gap accionable pequeño: fix + test.
- **Pre-mortem:** (1) feature no activable en el entorno → método alternativo (unit test con tmpdir, flags); (2) hallazgo grande (índices en plaintext) → FIND + decisión, no fix apurado; (3) falso positivo de lectura → evidencia por artefacto.
- **Stop conditions:** >2d → cerrar mapa + FINDs.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Índice filtra plaintext | Clasificar + decisión con evidencia | cierre |
  | 🟢×🟡 | Feature no activable local | Método alternativo de verificación | DISCOVERY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) plaintext real; (2) método de verificación; (3) falso positivo.
- **Uphill/Downhill:** ⬆️ 1 incógnita (cobertura real) → mapa la resuelve
- **DoD:** task = contrato · commit = `docs:`/`fix:` · release = n/a.
- **Validación Appetite vs Effort:** 2d ≥ 2d ✓
- **Skills sugeridas:** security-and-hardening · systematic-debugging · source-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DUR-02.md`

### Task 13: BENCH-01 — `competitive_bench`: fix del doble-conteo (mide mal lo que dice medir)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 2-3h
- **Prioridad:** 🟡
- **Archivos clave:** `benchmarks/competitive_bench.py:9-17` (docstring admite el workaround `--batch-size 999`)
- **Verificación real:** ✅ CÓDIGO-REAL — el timer de Ingest cuenta un rebuild HNSW completo Y `rebuild_index()` lo repite; workaround documentado.
- **Gate Justificación:** los números publicados del bench competitivo son la evidencia de marketing (Regla 11); medir mal invalida claims.
- **Gate Result:** ✅ DO
- **Contrato:** el bench no duplica el rebuild (o el workaround deja de ser necesario) + comparación before/after del número afectado documentada + docstring actualizado.
- **Pre-mortem:** (1) números históricos incomparables → nota de comparabilidad; (2) el fix altera el baseline → regenerar/referenciar; (3) el "doble rebuild" era intencional → documentar por qué no.
- **Stop conditions:** >1d → workaround documentado como contrato (mínimo viable).
- **Risk Register:** | 🟢×🟢 | Incomparabilidad histórica | Nota + tabla before/after | cierre |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) incomparabilidad; (2) baseline; (3) intencionalidad.
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = contrato · commit = `fix(bench):` · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 3h ✓
- **Skills sugeridas:** performance-optimization · systematic-debugging · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/BENCH-01.md`

### Task 14: DIST-01 — Publicar `vanta-memory` (quitar `publish = false`)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 2-4h
- **Prioridad:** 🔴
- **Archivos clave:** `vanta-memory/Cargo.toml`, `Cargo.toml` (workspace), `release-plz.toml`, `.github/workflows/release.yml`
- **Verificación real:** ✅ CÓDIGO-REAL — 11 crates con `publish=false` (verificado 2026-09-30); `vanta-memory` = 28k LOC, el diferenciador (L0→L1→L2→L3 + dream); "invisible = no hay demanda posible".
- **Gate Justificación:** P0 del DELTA — el feature set que Mem0 cobra $249/mo; el único path monetizable del tier embebido es ser el motor dentro del producto de otro.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo publish --dry-run -p vanta-memory` verde + smoke del crate como dependencia externa (compila en un proyecto tmp) + decisión release-plz registrada (entra en el próximo release o excluido explícitamente); workspace intacto.
- **Pre-mortem:** (1) deps path-only no publicables → dry-run primero; si bloquea: lista exacta + decisión owner; (2) release-plz lo publicaría en el próximo release sin querer → coordinar con #238; (3) semver de crate nuevo → 0.x.
- **Stop conditions:** dependencias internas bloquean → lista de bloqueantes + decisión owner (BLOQUEADO parcial).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Deps path no publicables | `--dry-run` primero | DISCOVERY |
  | 🟡×🟢 | Publicación no coordinada con #238 | Registrar decisión release-plz | release |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) deps; (2) coordinación release; (3) semver.
- **Uphill/Downhill:** ⬇️ (3 steps)
- **DoD:** task = contrato · commit = `feat(release):` · release = publicado en 0.9.0.
- **Validación Appetite vs Effort:** 1d ≥ 4h ✓
- **Skills sugeridas:** ci-cd-and-automation · shipping-and-launch · git-workflow-and-versioning · source-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DIST-01.md`

### Task 15: DIST-02 — Exponer la capa cognitiva en Python (`memory_recall`/`memory_capture`)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/src/lib.rs`, stubs `vantadb-python/vantadb_py/*.pyi`, `docs/api/PYTHON_SDK.md`
- **Verificación real:** ✅ CÓDIGO-REAL — el binding Python NO re-exporta `vanta-memory` (0 referencias `vanta_memory`): "un usuario Python tiene una BD, no memoria" (research §4; verificado 2026-09-30).
- **Gate Justificación:** P0 del DELTA — sin esto, MEMG-01..09 son features para nadie; el consumidor externo nunca llega a la capa cognitiva.
- **Gate Result:** ✅ DO
- **Contrato:** `vantadb.memory_recall(...)` / `vantadb.memory_capture(...)` (scope mínimo viable; `dream` opcional) funcionan end-to-end en smoke con wheel local; stubs `.pyi` + PYTHON_SDK actualizados; suite Python verde.
- **Pre-mortem:** (1) scope creep (toda la capa) → scope mínimo recall+capture, dream como follow-up; (2) `vanta-memory` como dep del binding (DIST-01 la publica) → coordinar; (3) GIL/perf de las ops nuevas → patrón de las existentes (`py.detach`).
- **Stop conditions:** scope >2d → declarar scope mínimo entregado + FIND del resto.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Scope creep a toda la capa | Scope mínimo escrito ANTES de codear | DISCOVERY |
  | 🟢×🟡 | Coordinación con DIST-01 | Dependencia declarada | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) scope; (2) dep; (3) API shape.
- **Uphill/Downhill:** ⬇️ (4 steps)
- **DoD:** task = contrato + smoke · commit = `feat(python):` · release = changelog (feature → minor).
- **Validación Appetite vs Effort:** 3d ≥ 2d ✓
- **Skills sugeridas:** api-and-interface-design · source-driven-development · rust-write-tests · documentation-skill · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DIST-02.md`

### Task 16: DIST-03 — TS/Node/WASM: exponer la capa cognitiva **o** declarar scope por binding

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-ts/src`, `vantadb-wasm/src`, `docs/api/BINDINGS_NAMESPACES.md`, `docs/api/VANTA_MEMORY.md`
- **Verificación real:** ✅ CÓDIGO-REAL — la capa cognitiva es Rust-only hoy; decisión pendiente "exponer o declarar scope" (DELTA P0 DIST-03; verificado 2026-09-30).
- **Gate Justificación:** la promesa multi-binding debe ser explícita: o se expone el scope mínimo o se declara el alcance por binding (evita expectativa rota).
- **Gate Result:** ✅ DO
- **Contrato:** decisión implementada y verificable: (a) exposición mínima viable en TS/WASM con smoke verde, **o** (b) declaración explícita de scope por binding en docs + `capabilities()` consistente entre bindings; sin breaking changes; matriz en BINDINGS_NAMESPACES.md actualizada.
- **Pre-mortem:** (1) la capa usa fs/persistencia no disponible en wasm → evaluar viabilidad; si inviable, (b) documentado con motivo; (2) blast radius del binding; (3) inconsistencia entre bindings → matriz + test de paridad (DIST-17 lo profundiza).
- **Stop conditions:** inviable en wasm → (b) + FIND derivado del port.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Inviabilidad en wasm | Fallback (b) con motivo escrito | DISCOVERY |
  | 🟢×🟡 | Inconsistencia entre bindings | Matriz + capabilities() | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) inviabilidad wasm; (2) consistencia; (3) scope.
- **Uphill/Downhill:** ⬆️ 1 incógnita (viabilidad wasm) → step 1
- **DoD:** task = contrato · commit = `feat(ts):`/`docs:` · release = changelog.
- **Validación Appetite vs Effort:** 3d ≥ 3d ✓
- **Skills sugeridas:** api-and-interface-design · source-driven-development · documentation-skill · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DIST-03.md`

### Task 17: DIST-04 — `VANTA_MEMORY.md` ↔ realidad (cierre post DIST-01/02/03)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Archivos clave:** `docs/api/VANTA_MEMORY.md`
- **Verificación real:** ✅ CÓDIGO-REAL — el doc ya declara core-only + `publish=false` + "not exposed by any binding" (`:21-30,140-160`, verificado 2026-10-01); residual: publicar la superficie "candidate" cuando DIST-01/02 la expongan.
- **Gate Justificación:** cierre documental que evita drift doc↔código justo cuando DIST-01/02/03 cambian la realidad.
- **Gate Result:** ✅ DO
- **Contrato:** el doc refleja el estado post DIST-01/02/03 (superficies publicadas/declaradas, canales) + `check-docs`/`check-links` exit 0.
- **Pre-mortem:** (1) ejecutarse antes que DIST-01/02/03 → **dependencia dura: va última**; (2) duplicar info de otros docs → link, no copia.
- **Stop conditions:** —
- **Risk Register:** | 🟢×🟢 | Orden de ejecución | Dependencia declarada (post 01/02/03) | cierre |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) orden; (2) duplicación; (3) —
- **Uphill/Downhill:** ⬇️ (1 step)
- **DoD:** task = contrato · commit = `docs:` · release = n/a.
- **Validación Appetite vs Effort:** 1h ≥ 1h ✓
- **Skills sugeridas:** documentation-skill · writing-guidelines
- **Dependencias:** DIST-01, DIST-02, DIST-03 (BLOQUEANTES)
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DIST-04.md`

### Task 18: DIST-05 — Assets del release + verificación post-release real (fix del 404)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟠
- **Archivos clave:** `.github/workflows/release-binaries.yml`, `install.ps1`, `docs/dev/workflow/PUBLISH.md`
- **Verificación real:** ✅ CÓDIGO-REAL — el zip Windows de v0.7.0 da **404 live** (re-verificado 2026-10-01: release v0.7.0 = 4 wheels, sin zip); `install.ps1` apunta a v0.7.0.
- **Gate Justificación:** el "funnel de 60 segundos" no existe si los assets no están; además el flujo post-release debe verificar artefactos de verdad (pip/npm/crates/binarios) — justo lo que el run 0.8.0 aprendió a hacer a mano.
- **Gate Result:** ✅ DO
- **Contrato:** (a) flujo post-release con verificación real de artefactos + smoke del quickstart en máquina limpia (checklist ejecutable en PUBLISH.md); (b) `install.ps1` apuntando a la última release (o parametrizado); (c) ejecutado y verde en el release 0.9.0.
- **Pre-mortem:** (1) el zip Windows faltaba por decisión → verificar historia (FIND-229/backfill) y decidir; (2) checklist que nadie corre → automatizarlo como step de workflow o gate manual en PUBLISH.md con dueño; (3) `install.ps1` hardcodeado → parametrizar con fallback.
- **Stop conditions:** decisión de assets ambiguas → Gate D.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Checklist sin dueño | Dueño explícito + step en flujo | diseño |
  | 🟢×🟢 | install.ps1 stale | Parametrizar + verificación | cierre |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) assets por decisión; (2) checklist muerto; (3) install.ps1.
- **Uphill/Downhill:** ⬇️ (3 steps)
- **DoD:** task = contrato + verificación en 0.9.0 · commit = `ci:`/`docs:` · release = parte del release.
- **Validación Appetite vs Effort:** 1d ≥ 4h ✓
- **Skills sugeridas:** ci-cd-and-automation · shipping-and-launch · documentation-skill · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DIST-05.md`

### Task 19: DIST-06 — Estrategia de los 11 crates `publish = false`

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟡
- **Archivos clave:** los 11 `Cargo.toml` (`vanta-memory`, `vanta-proxy`, `vantadb-mcp`, `vantadb-python`, `vantadb-server`, `vantadb-node`, `vantadb-ffi-core`, fuzz, litellm, ollama, openai), `release-plz.toml`, `docs/dev/workflow/PUBLISH.md`
- **Verificación real:** ✅ CÓDIGO-REAL — 11 crates `publish=false` (verificado 2026-09-30); PyPI/npm dependen de CI propio (punto único de fallo).
- **Gate Justificación:** decidir y documentar qué se publica y por qué canal elimina la ambigüedad que costó FIND-230; DIST-01 ejecuta la parte de `vanta-memory`.
- **Gate Result:** ✅ DO
- **Contrato:** decisión documentada POR crate (publicar/no/publicar/canal + motivo) en `PUBLISH.md` §crates + consistencia con `release-plz.toml` (verificable por inspección + dry-run de los que pasen a publicar); sin cambios de versión.
- **Pre-mortem:** (1) decisiones prematuras → registrar como política revisable con fecha; (2) churn en release-plz → tocar solo lo decidido; (3) duplicar con DIST-01 → DIST-06 = política, DIST-01 = ejecución de vanta-memory.
- **Stop conditions:** decisión de negocio requerida (Pro) → Gate D.
- **Risk Register:** | 🟢×🟢 | Churn de release-plz | Cambios mínimos | cierre |
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) prematuridad; (2) churn; (3) solapamiento.
- **Uphill/Downhill:** ⬇️ (2 steps)
- **DoD:** task = contrato · commit = `docs:`/`chore:` · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 1d ✓
- **Skills sugeridas:** ci-cd-and-automation · shipping-and-launch · documentation-and-adrs
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DIST-06.md`

### Task 20: DOCS-F1 — Cerrar docs-consolidation F1 (triage + mojibake + markdownlint)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟡
- **Archivos clave:** `docs/dev/plans/2026-09-28-docs-consolidation.md` (F1), salida `check-links.mjs --json`, `docs/dev/avance/**` (mojibake), `docs/user/operations/BENCHMARKS.md` + `CONFIGURATION.md` (markdownlint), workflow markdownlint (BASELINE)
- **Verificación real:** ✅ CÓDIGO-REAL — T8 pendiente (triage 246 links P0-P4), T9 pendiente (mojibake `[[bench]]/[[test]]/[[package]]` ~30 en avance/), T12 pendiente (7 errores markdownlint: 5 BENCHMARKS + 2 CONFIGURATION; BASELINE 12→5). T10 SKIPPED y T11 COMPLETED ya (no tocar).
- **Gate Justificación:** cierra un plan abierto con deuda propia acotada; el triage P3/P4 fuera del gate es lo que hace el gate drenable (patrón Qdrant/LanceDB).
- **Gate Result:** ✅ DO
- **Contrato:** (a) cada link roto clasificado P0-P4 con P3/P4 excluidos del gate y motivo en el workflow; (b) 0 mojibake en `avance/` (fuera de `tasks/` — restricción owner); (c) markdownlint BASELINE 12→5 con CI verde; (d) F1 del plan docs-consolidation marcado ✅.
- **Pre-mortem:** (1) restaurar mojibake sin original → `git log -S` del carácter; (2) NO usar `markdownlint --fix` sobre prosa a mano; (3) P0-P2 excede 4h → stop: dejar clasificación completa + fix parcial + FIND.
- **Stop conditions:** >4h → entregar triage completo y derivar el drain restante.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Fix de mojibake sin original | `git log -S` primero | por ítem |
  | 🟢×🟡 | Drain excede appetite | Stop → clasificación + FIND | 4h |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) mojibake irreversible; (2) appetite; (3) markdownlint fix indebido.
- **Uphill/Downhill:** ⬇️ (3 steps)
- **DoD:** task = contrato + gates · commit = `docs:` · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 4h ✓
- **Skills sugeridas:** documentation-skill · writing-guidelines · git-workflow-and-versioning · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DOCS-F1.md`

### Task 21: DOCS-F2 — Cerrar docs-consolidation F2 (ejemplos ejecutables + 2 gates)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 8h
- **Prioridad:** 🟠
- **Archivos clave:** `docs/api/EMBEDDED_SDK.md`, `docs/api/PYTHON_SDK.md`, `docs/user/QUICKSTART.md`, `.github/workflows/ci-examples.yml` + `ci-rustdoc.yml`, `.github/workflows/gate-docs-links.yml`
- **Verificación real:** ✅ CÓDIGO-REAL — T13: `ci-examples.yml` ejecuta `examples/`, NO los bloques dentro de `docs/` → "hoy ningún ejemplo de la documentación se ha ejecutado jamás"; T14: falta gate API↔docs (patrón DuckDB `NeedsDocumentation.yml`); T15: falta gate anti-fuga en `docs/` (patrón CISA/Uber; complementar con push protection).
- **Gate Justificación:** un ejemplo equivocado no compila y nunca llega a un humano — la verificación (no la revisión) es lo que funciona; y el VCS guarda arquitectura Y claves (leak gate).
- **Gate Result:** ✅ DO
- **Contrato:** (a) ejemplos de docs ejecutables en CI, mínimo viable por lenguaje (Rust: `RUSTDOCFLAGS="-D warnings" cargo test --doc`; Python: `pydoclint`; TS: typedoc + ejemplos como tests); (b) gate: cambio de API pública en `src/**` sin cambio en `docs/**` en el mismo commit → fail; (c) gate anti-fuga (gitleaks/trufflehog sobre `docs/`) verde; (d) F2 del plan docs-consolidation marcado ✅.
- **Pre-mortem:** (1) doctests existentes fallan en masa → arrancar warning-only + burn-down con FIND; (2) falsos positivos del gate API↔docs → diff-based acotado a `pub`; (3) scope → stop a las 8h con sub-parte entregada.
- **Stop conditions:** >8h → entregar los gates (b/c) y derivar (a) a FIND con plan de burn-down.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Doctests en masa rojos | Warning-only + burn-down | diseño |
  | 🟢×🟡 | Falsos positivos gate API | Diff acotado a pub | VERIFY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) masa de doctests; (2) falsos positivos; (3) scope.
- **Uphill/Downhill:** ⬇️ (4 steps)
- **DoD:** task = contrato + CI verde · commit = `ci:`/`docs:` · release = n/a.
- **Validación Appetite vs Effort:** 2d ≥ 8h ✓
- **Skills sugeridas:** ci-cd-and-automation · documentation-skill · security-and-hardening · test-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/DOCS-F2.md`

### Task 72: ENC-01 — Cifrado honesto: aviso al activar + docs (FIND-249 parte 1)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 2-4h
- **Prioridad:** 🟠
- **Archivos clave:** `src/config.rs` (feature `encryption`), `src/crypto.rs` / punto de activación, `docs/user/operations/CONFIGURATION.md`, `docs/dev/architecture/FEATURES.md`
- **Verificación real:** ✅ CÓDIGO-REAL — DUR-02 (2026-10-04): **0/6 artefactos cifrados** con la feature activa (WAL/HNSW/VantaFile/backend-KV/text_index/snapshots en plaintext); primitivas AES-256-GCM funcionan pero ningún write path las usa (`with_cipher` 0 callers). Decisión owner 2026-10-04: **avisar ahora, cablear luego**.
- **Gate Justificación:** la opción `encryption` promete una protección que no cumple (falsa sensación de seguridad); ~1d para ser honestos ahora; el cableado completo (FIND-249, 3-5d) queda diferido por decisión del owner.
- **Gate Result:** ✅ DO
- **Contrato:** al activar `encryption` (feature + `VANTADB_ENCRYPTION_KEY`), el sistema emite un **warning explícito y accionable** de que el cifrado aún no protege artefactos on-disk + `CONFIGURATION.md` y `FEATURES.md` lo dicen sin ambigüedad (referencia a FIND-249) + test que fija el aviso.
- **Pre-mortem:** (1) romper builds con la feature on → gate scoped + `--features encryption` en verify; (2) warning ruidoso en tests → emitir una vez / gating por entorno; (3) wording ambiguo → texto explícito "NOT encrypted at rest".
- **Stop conditions:** si el aviso requiere cambiar semántica pública → Gate D (question al owner) antes de codear.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Aviso no visible donde importa | Emitir en el punto de activación + doc | DISCOVERY |
  | 🟢×🟡 | Wording que sugiera protección parcial | Texto explícito "not encrypted at rest" | review |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) visibilidad del aviso; (2) wording; (3) scope creep al cableado (eso es FIND-249).
- **Uphill/Downhill:** ⬇️ (2 steps: aviso + docs/test)
- **DoD:** task = contrato (aviso + docs + test) · commit = `fix(security):`/`docs:` · release = changelog (nota de transparencia).
- **Validación Appetite vs Effort:** 1d ≥ 4h ✓
- **Skills sugeridas:** security-and-hardening · documentation-skill · rust-write-tests
- **Estado:** ✅ COMPLETED
- **Task file:** `docs/dev/tasks/ENC-01.md` (crear en DISCOVERY)
- **Origen:** decisión owner 2026-10-04 (question) + FIND-249 (DUR-02). El cableado completo queda en FIND-249 (Backlog, diferido).

### Checkpoint F0 → Release 0.9.0 — ✅ CERRADO (2026-10-04)

- [x] 22/22 tasks F0 (21 + ENC-01 addendum) ✅ (review P2-01 fresh por tarea; DOCS-F1/F2 escaladas → ronda 2 approve)
- [x] `just verify` verde (fmt · clippy -D warnings · nextest audit · deny) — regresión clippy de WSM-15 detectada y arreglada (`6401261c` + `00452975`)
- [ ] ⏸️ `/audit certify` — **DIFERIDO a la ventana del release** (plan-adjust 2026-10-04, decisión owner: certificar el estado que se publica, no un intermedio)
- [x] **Release 0.9.0**: **diferido por decisión owner (2026-10-04, post-fases)** — changelog 0.8.0 limpiado ✅ + Release PR cuando se retome + verificación post-release (Task 18) — semver esperado: **minor** (features DIST-01/02/03 + fixes)
- [x] Registro: filas del Backlog migradas a avance (skill progreso — por tarea)

## F1 — Distribución (post-0.9.0)

> **Gate de fase:** ✅ CUMPLIDO — Tasks 22-36 expandidas a nivel F0 el 2026-10-04 (`3afe71c6`); listas para ejecutar.

### Task 22: DX-12 — Instalador/wizard como selector de módulos (enable/disable por componente)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟠
- **Archivos clave:** `scripts/install.ps1` (163 líneas; flags `-NoWizard`/`-DryRun`/`-WizardNonInteractive`/`-Version`/`-InstallDir`; chain al wizard al final, FIND-105), `scripts/install.sh` (patrón espejo Linux/macOS), `setup-embeddings.ps1` (517 líneas — wizard parcial: proxy on/off L438-467 + bloques MCP por cliente L299-312: opencode/claude/cursor), `src/cli.rs:338-400` (`server --http|--mcp` + `mcp-call` = binario unificado)
- **Verificación real:** ✅ CÓDIGO-REAL — el instalador hoy instala SOLO `vanta-cli.exe` + encadena el wizard de embeddings (proxy on/off + bloques MCP opencode/claude/cursor); **no existe selección por componente** (motor/MCP/server/proxy/desktop/embeddings/providers). Backlog DX-12: "Hoy: instalador único + vanta-cli unificado (server+MCP en un binario) + wizard parcial (proxy on/off, bloques MCP por cliente)". Visión owner (2026-10-01): "un solo instalador que pregunte qué habilitar".
- **Gate Justificación:** UX de entrada del producto único (decisión owner 2026-10-01); el esqueleto ya existe (flags + wizard encadenado) → delta acotado de selección + idempotencia; sin esto cada módulo se habilita a mano.
- **Gate Result:** ✅ DO
- **Contrato:** `install.ps1`/`install.sh` ofrecen selección por componente (motor / MCP / server / proxy / visor desktop / embeddings / providers) y la aplican de forma idempotente; `-NoWizard`/`-WizardNonInteractive` siguen funcionando (no-interactive = defaults actuales); smoke en máquina limpia (patrón DESKTOP-41); `-DryRun` refleja las decisiones nuevas; estado por módulo declarado (installable / frozen — desktop es viewer frozen, README:231).
- **Pre-mortem:** (1) matriz componentes×plataforma crece sin control → lista fija de 7 módulos del owner, cada uno con su "hoy = X"; (2) romper el flujo no-interactive/CI → test de ambos caminos (interactivo simulado + `-NonInteractive`); (3) prometer módulos no instalables (desktop frozen) → declarar estado por módulo, no fingir soporte.
- **Stop conditions:** 2d sin contrato verde → entregar selector mínimo (motor+MCP+embeddings) + FIND del resto.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Regresión del flujo actual | Test de ambos caminos (interactivo/no-interactive) | VERIFY |
  | 🟡×🟢 | Módulo anunciado sin soporte real | Estado por módulo (installable/frozen) | diseño |
  | 🟢×🟡 | Idempotencia rota al re-correr | Doble corrida = mismo estado (patrón .bak de install.ps1) | VERIFY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) regresión no-interactive; (2) módulos sobre-prometidos; (3) idempotencia.
- **Uphill/Downhill:** ⬇️ (3 steps: selector → aplicación idempotente → smoke)
- **DoD:** task = contrato + smoke limpio · commit = `feat(installer):` · release = changelog (DX).
- **Validación Appetite vs Effort:** 2d ≥ 2d ✓
- **Skills sugeridas:** ci-cd-and-automation · shipping-and-launch · source-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/DX-12.md`

### Task 23: DIST-15 — `graphrag_search` en bindings (Py/TS/Node/WASM)

- **Appetite:** max 2d
- **Esfuerzo:** 🟢 1-2d
- **Prioridad:** 🟠
- **Archivos clave:** `src/sdk/builder.rs:168-177` (`Embedded::graphrag_search` — core, existe), `src/graphrag/{seed,expand,retrieve,context,pipeline}.rs`, `vantadb-python/src/lib.rs`, `vantadb-ts/src/{vantadb.ts,native.ts}`, `vantadb-node/src/lib.rs`, `vantadb-wasm/src/lib.rs`, `docs/api/GRAPH_RAG.md`
- **Verificación real:** ✅ CÓDIGO-REAL — `graphrag_search` existe en el core (`src/sdk/builder.rs:168`: `Embedded::graphrag_search(namespace, query: Option<&str>, query_vector: Option<&[f32]>) -> GraphRagResult`); `rg graphrag` en los 4 `src/` de bindings = **0 hits**; `GRAPH_RAG.md:13-15` lo declara: "Binding availability: Rust only... is **not** exposed by any binding yet — there is no `graphrag_search` method on the Python, WASM, TypeScript, or Node bindings". Backlog: "El feature estrella (GraphRAG) es inalcanzable desde los bindings (`GRAPH_RAG.md:14-15` lo declara). Costo bajo, impacto desproporcionado".
- **Gate Justificación:** el feature diferenciador (grafo + pipeline seed→expand→retrieve→context) es inalcanzable desde la superficie que consumen los usuarios; la implementación core ya existe → el delta es glue de bindings + docs, con impacto desproporcionado.
- **Gate Result:** ✅ DO
- **Contrato:** `graphrag_search` (nombre canónico a fijar en DISCOVERY según convención BINDINGS_NAMESPACES/API-04) invocable desde al menos Py + TS con smoke verde; paridad de resultados con el core (mismo input + mismo DB → mismo `context_text`/nodos); `GRAPH_RAG.md` actualizado (quitar el "inalcanzable"); stubs/docs de los bindings tocados sincronizados (`check-api-docs`).
- **Pre-mortem:** (1) shape de `GraphRagResult` complejo para el wire → serialización canónica a dict/JSON + test que la pina; (2) un target no puede con las deps del pipeline (wasm) → compile check por target ANTES de prometer; scope mínimo Py+TS + FIND del resto; (3) drift de docs → `check-api-docs` por rango en el cierre.
- **Stop conditions:** un target no compila con las deps del pipeline → scope (Py+TS) + FIND por target, sin forzar.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Shape de resultado inestable entre bindings | Serialización canónica + test de paridad | diseño |
  | 🟢×🟡 | Deps del pipeline no disponibles en wasm | Compile check por target antes de codear | DISCOVERY |
  | 🟢×🟢 | Docs drift post-binding | `check-api-docs` por rango | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) shape wire; (2) viabilidad por target; (3) docs.
- **Uphill/Downhill:** ⬇️ (3 steps: core→binding Py→binding TS+docs)
- **DoD:** task = contrato + smoke · commit = `feat(bindings):` · release = changelog (feature → minor).
- **Validación Appetite vs Effort:** 2d ≥ 2d ✓
- **Skills sugeridas:** api-and-interface-design · source-driven-development · rust-write-tests · documentation-skill · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/DIST-15.md`

### Task 24: DIST-16 — `verify` de certificados vía MCP

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs` (`handle_tools_list:81`, `handle_tools_call:1295`; `memory_delete` con `attest:true` ya emite certificado VER-02 en :1549-1571), `src/sdk/api/memory.rs:987-994` (`Embedded::verify_purge_certificate`), `src/attestation.rs`, `src/cli.rs:474-485` (`certificate verify --file`) + `:196-201` (`verify` WAL), `docs/api/MCP.md`
- **Verificación real:** ✅ CÓDIGO-REAL — SDK (`verify_purge_certificate`, memory.rs:987) y CLI (`vanta-cli certificate verify`, cli.rs:474-485; `vanta-cli verify` WAL, :196-201) ya exponen verify; el MCP tiene 79 tools listadas (MCP.md:198) y **ninguna es verify** (rg = 0 en handlers); MCP.md:604-607 declara el hash-chain "cited not duplicated" — el gap es exactamente la superficie MCP.
- **Gate Justificación:** el MCP ya emite certificados (`memory_delete attest:true`) pero no puede verificarlos → el loop de evidencia queda abierto justo en la puerta de agentes (North Star); wrapper de una función existente.
- **Gate Result:** ✅ DO
- **Contrato:** tool MCP de verify disponible y con smoke verde (certificado válido/inválido → resultado tipado, no string), paridad con CLI (mismo veredicto para el mismo certificado); `MCP.md` actualizado + `validate-docs-coverage.ps1` exit 0; conteo de tools re-baselineado; DISCOVERY fija el nombre canónico (`verify` vs `certificate_verify`) y si cubre también el WAL-chain o solo certificados (VER-02).
- **Pre-mortem:** (1) ambigüedad de scope (¿certificado VER-02 o WAL verify VER-01?) → decidir en DISCOVERY con el contrato del plan (certificados) y FIND si el WAL queda fuera; (2) verificación contra engine vivo — el certificado no se ata a una DB (memory.rs:984) → documentar el caveat; (3) doc-coverage gate falla por tabla desincronizada → actualizar MCP.md en el mismo commit.
- **Stop conditions:** >1d → dejar el wrapper mínimo + FIND de la parte WAL.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Scope ambiguo (VER-01 vs VER-02) | Fijarlo en DISCOVERY; contrato del plan = certificados | DISCOVERY |
  | 🟢×🟡 | Doc-coverage gate rojo | MCP.md + validate-docs-coverage en el mismo commit | cierre |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) scope; (2) caveat engine-vivo; (3) gate docs.
- **Uphill/Downhill:** ⬇️ (2 steps: tool + docs/smoke)
- **DoD:** task = contrato + smoke · commit = `feat(mcp):` · release = changelog.
- **Validación Appetite vs Effort:** 1d ≥ 1d ✓
- **Skills sugeridas:** api-and-interface-design · source-driven-development · documentation-skill · campaign-executor
- **Estado:** ✅ COMPLETED

### Task 25: DIST-17 — Test de paridad cross-language (Py/Node/WASM)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** tests de los 3 bindings (`vantadb-python/tests/` — 19 archivos, `vantadb-ts/src/__tests__/` — 17, `vantadb-node/tests/` — 5, `vantadb-wasm/tests/wasm_tests.rs`), CI (job nuevo; patrón de jobs TS en `.github/workflows/release-npm-61.yml` y node en `release-npm-node.yml`), `docs/api/BINDINGS_NAMESPACES.md` (§W1 parity matrix :78 · §v2 wire parity :103)
- **Verificación real:** ✅ CÓDIGO-REAL — existen pruebas sueltas por binding y espejos manuales del "bindings-parity contract" (precedente WIRE-03: `vantadb-ts/src/__tests__/wire03.test.ts` + `vantadb-node/tests/wire03.test.ts` + tests datetime de Python), pero **no hay runner cross-language que compare resultados** (rg `conformance|cross-language` = 0 en CI; `BINDINGS_NAMESPACES.md` tiene la matriz doc, no ejecución). Decisión owner 2026-10-01: los 3 conectores siguen activos.
- **Gate Justificación:** con 3 conectores activos, la paridad es una promesa de producto que hoy solo se sostiene por espejos manuales; un comparador mecánico (mismo escenario → mismo hash) la convierte en verificable y atrapa drift entre bindings.
- **Gate Result:** ✅ DO
- **Contrato:** un escenario canónico (put/search/grafo/IQL) corre en Py/Node/WASM y produce **resultados idénticos** (hash/diff canónico) en CI; job dedicado verde; divergencias encontradas → FINDs (no fixes silenciosos); sin cambios de código de producción salvo fixes que un FIND justifique.
- **Pre-mortem:** (1) WASM no puede correr el mismo escenario (sin fs/IQL completo) → fijar el subconjunto común en DISCOVERY y declarar exclusiones por binding; (2) comparación frágil (orden de campos/float) → normalización canónica + tolerancia float documentada; (3) job lento (3 toolchains) → un job agregado con cache, no matriz completa.
- **Stop conditions:** 3d sin job verde → entregar el comparador Py↔Node + FIND del tercer binding.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Escenario no corre igual en WASM | Subconjunto común fijado en DISCOVERY | diseño |
  | 🟡×🟢 | Comparación frágil (float/orden) | Normalización + tolerancia documentada | VERIFY |
  | 🟢×🟢 | Costo CI del job | Un job agregado con cache | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) viabilidad WASM; (2) fragilidad de la comparación; (3) costo CI.
- **Uphill/Downhill:** ⬆️ 1 incógnita (¿corre el escenario completo en WASM?) → step 1 (DISCOVERY) → luego ⬇️
- **DoD:** task = contrato + job verde · commit = `test(bindings):`/`ci:` · release = n/a.
- **Validación Appetite vs Effort:** 3d ≥ 3d ✓
- **Skills sugeridas:** test-driven-development · ci-cd-and-automation · source-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/DIST-17.md`

### Task 26: WSM-14 — Plan de adopción npm (README + demo Transformers.js + keywords honestas)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-wasm/README.md` (321 líneas, tracked — brief "bundle strategy" con comparativa de tamaño vs Orama y referencia al demo en :310), `vantadb-wasm/demo/` (demo Transformers.js + OPFS **ya existe**: `demo/README.md` "Browser AI Agent Demo", `app.js`, `index.html`), `vantadb-ts/README.md` (README del paquete `vantadb`), `vantadb-wasm/pkg/README.md` (generado + gitignored — mecanismo del README publicado [a verificar en DISCOVERY])
- **Verificación real:** ✅ CÓDIGO-REAL — el demo Transformers.js ya existe y está enlazado desde el README del crate (`vantadb-wasm/README.md:310` → `vantadb-wasm/demo/`); la comparativa honesta de tamaño vs Orama (28x, 5.44M desc/mes vs 187) ya está en el bundle-strategy README; lo que falta es el **posicionamiento npm** ("browser AI agent memory"), keywords y el enlace al demo en la superficie publicada (`vantadb-ts/README.md` no menciona demo/transformers). Estrategia H-21 aprobada.
- **Gate Justificación:** el paquete npm es la superficie de descubrimiento del binding WASM (adopción 12 dl/semana); el demo y la comparativa existen pero la superficie publicada no los capitaliza — delta de README/keywords, no de código.
- **Gate Result:** ✅ DO
- **Contrato:** README npm con posicionamiento "browser AI agent memory" + demo Transformers.js enlazada + keywords/comparativa honesta (incluye el gap de bundle size, sin ocultarlo); **sin claims de performance sin benchmark** (Regla 11); smoke del paquete (npm pack / carga en Node+script) verde; `check-links`/`check-docs` exit 0.
- **Pre-mortem:** (1) el README publicado se arma desde un artefacto generado (`pkg/` gitignored) → localizar el mecanismo de publish en DISCOVERY antes de editar; (2) claims de adopción stale (5.44M/187 cambian) → fecha + fuente en cada número o quitarlo; (3) duplicar el bundle-strategy brief en dos READMEs → enlazar, no copiar.
- **Stop conditions:** el mecanismo de README publicado no es editable desde el repo → FIND + aplicar al tracked más cercano.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | README publicado ≠ README tracked | Localizar mecanismo de publish en DISCOVERY | DISCOVERY |
  | 🟢×🟡 | Números de adopción stale | Fecha + fuente por número (Regla 11) | review |
  | 🟢×🟢 | Duplicación de brief | Enlace, no copia | cierre |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) artefacto publicado; (2) claims stale; (3) duplicación.
- **Uphill/Downhill:** ⬇️ (2 steps: README/keywords → smoke)
- **DoD:** task = contrato + smoke · commit = `docs(npm):` · release = n/a (parte del próximo publish).
- **Validación Appetite vs Effort:** 3d ≥ 3d ✓
- **Skills sugeridas:** documentation-skill · writing-guidelines · ai-seo · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/WSM-14.md`

### Task 27: TS-10 — Plan de distribución/adopción WASM (playground + docs-site + comparativa)

- **Appetite:** max 1sem
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🟠
- **Archivos clave:** repo externo `ness-e/Vantadb-web` (checkout local en `..\web`, remote `github.com/ness-e/Vantadb-web.git`): `src/app/playground/page.tsx`, `src/components/vanta/code-playground.tsx`, `src/app/docs/`, `src/app/benchmarks/`; informe §3 `docs/dev/reviews/archive/research-vantadb-wasm-20260825.md`; coordinación owner (repo separado desde 2026-09-22)
- **Verificación real:** ✅ CÓDIGO-REAL — el repo web existe y ya tiene playground interactivo (`/playground` → `CodePlayground` con iframe WASM real, WEB-07), docs (`/docs`) y benchmarks; la adopción del binding es 12 dl/semana vs 35K-465K competidores (H-06/Backlog TS-10, informe §3); el plan de adopción (secuencia + métrica) no está escrito. Requiere DISCOVERY.
- **Gate Justificación:** la brecha de adopción es la más grande del binding WASM; el contenido ya existe en el web — falta secuencia, comparativa honesta y métrica declarada; es pata de distribución, no feature.
- **Gate Result:** ✅ DO
- **Contrato:** plan de adopción ejecutado en el repo web (playground + docs-site + comparativa honesta) **o** decisión documentada de secuencia con el owner; métrica de éxito declarada (descargas/visitas con baseline); sin claims sin evidencia (Regla 11); el trabajo que toque el repo web se coordina con su propio flujo (no se edita a ciegas).
- **Pre-mortem:** (1) repo externo sin acceso/owner → DISCOVERY primero: decidir alcance ejecutable desde este lado (docs/plan) vs cambios en el web; (2) scope de 1-2sem sin corte → entregar plan + primer slice (p.ej. comparativa) y FIND del resto; (3) métrica no medible → declarar la fuente (npm stats / analytics) antes de prometerla.
- **Stop conditions:** >1sem sin ejecución posible en el web → cerrar como plan documentado + decisión de secuencia del owner.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Dependencia del repo/owner externo | DISCOVERY + decisión de secuencia documentada | DISCOVERY |
  | 🟡×🟢 | Scope sin corte en 1-2sem | Plan + primer slice + FIND | diseño |
  | 🟢×🟡 | Métrica no medible | Fuente declarada antes de prometer | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) dependencia externa; (2) scope; (3) métrica.
- **Uphill/Downhill:** ⬆️ 1 incógnita (alcance ejecutable desde este lado) → DISCOVERY → luego ⬇️
- **DoD:** task = contrato (plan ejecutado o decisión documentada + métrica) · commit = `docs:` · release = n/a.
- **Validación Appetite vs Effort:** 1sem < 2sem ⚠️ → el contrato admite plan + decisión de secuencia (no ejecución completa)
- **Skills sugeridas:** ai-seo · documentation-skill · campaign-executor · vanta-design-orchestrator (si hay UI)
- **Dependencias:** repo `ness-e/Vantadb-web` + OK del owner para cambios allí.
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/TS-10.md`

### Task 28: TS-11 — Roadmap paridad sub-clientes (wiki/conversation/skills vía WASM)

- **Appetite:** max 1sem
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/src/vantadb.ts` (getters actuales: `memory` :306, `graph` :330, `wiki` :373-376 — `Object.freeze({})` documentado, `system` :379), `vantadb-wasm/src/lib.rs` (superficie real expuesta), `docs/api/BINDINGS_NAMESPACES.md` §"Core-Only Capabilities (D43)" :347 + §Sub-Client Design :450, core cuando lo permita
- **Verificación real:** ✅ CÓDIGO-REAL — `db.wiki` es un placeholder vacío documentado ("Empty in TS v1: wiki features are core-only per D43", `vantadb.ts:111-112,369-376`); no existen getters de conversation/skills/thread en TS (rg = 0); `BINDINGS_NAMESPACES.md:347` lista las capacidades core-only diferidas. Backlog: "Planificar exposición vía WASM de wiki/conversation/skills cuando core lo permita".
- **Gate Justificación:** la paridad de sub-clientes es una expectativa de la promesa multi-binding; sin roadmap explícito, cada usuario descubre el gap por su cuenta (peor que un defer declarado).
- **Gate Result:** ✅ DO
- **Contrato:** roadmap documentado (qué sub-cliente, dependencia exacta del core/wasm, orden, criterio de promoción) + **al menos el primer slice ejecutado** (si una dependencia lo permite) **o** decisión de defer por dependencia con fecha de revisión; `BINDINGS_NAMESPACES.md` consistente con el roadmap; sin breaking changes.
- **Pre-mortem:** (1) dependencia del core no resuelta (wiki/conversation/skills son Rust-only hoy) → el roadmap declara la dependencia con evidencia y fecha de revisión, no promete fechas; (2) slice elegido sin valor → priorizar el sub-cliente con más demanda (wiki por su getter visible); (3) tocar TS sin tocar WASM (el getter delega) → el slice incluye la superficie wasm real o queda documentado por qué no.
- **Stop conditions:** todas las dependencias bloqueadas → cerrar como roadmap + defer fechado (contrato cumplido).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Dependencia core no resuelta | Roadmap con evidencia + fecha de revisión | DISCOVERY |
  | 🟢×🟡 | Slice sin valor | Priorizar por demanda (wiki visible hoy) | diseño |
  | 🟢×🟢 | Inconsistencia doc↔código | `check-docs` + matriz | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) dependencia core; (2) valor del slice; (3) consistencia.
- **Uphill/Downhill:** ⬆️ 1 incógnita (qué dependencia se puede mover) → DISCOVERY → luego ⬇️
- **DoD:** task = contrato (roadmap + slice o defer fechado) · commit = `docs:`/`feat(ts):` · release = n/a o changelog.
- **Validación Appetite vs Effort:** 1sem < 2sem ⚠️ → el contrato admite roadmap + defer (no ejecución completa)
- **Skills sugeridas:** api-and-interface-design · documentation-and-adrs · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/TS-11.md`

### Task 29: TS-13 — Posicionamiento vs Orama ("Why VantaDB" en web)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟡
- **Archivos clave:** repo `ness-e/Vantadb-web`: `src/app/why-vantadb/page.tsx` (existe), `src/components/vanta/vanta-data.ts:791-812` (`WHY_VANTADB.comparison` — matriz actual vs pinecone/weaviate/chroma), coordinación con el repo web
- **Verificación real:** ✅ CÓDIGO-REAL — la página "Why VantaDB" y una matriz de comparación **ya existen** (`vanta-data.ts:791-812`: latency/hops/deployment/recovery/hybrid/egress/cost vs pinecone/weaviate/chroma); **Orama no aparece en ningún lugar del web** (rg = 0) — la comparativa del bundle-strategy (`vantadb-wasm/README.md`, tamaño 28x + features) no está traducida a la web. H-13.
- **Gate Justificación:** Orama es el competidor directo del nicho browser (5.44M desc/mes); la matriz honesta existe y solo falta la columna Orama verificada contra código — costo de 1d, impacto directo en posicionamiento.
- **Gate Result:** ✅ DO
- **Contrato:** sección "Why VantaDB" del repo web con matriz verificada contra código (durable WAL browser, híbrido RRF nativo, grafo+IQL, errores tipados vs FTS-first de Orama); **sin claims sin evidencia** (Regla 11 — cada celda con fuente o marcada "not verified"); publicada (o decisión de secuencia con owner si el repo no es editable).
- **Pre-mortem:** (1) claims sobre Orama sin verificar (versión/features cambian) → verificar contra docs oficiales de Orama y fechar; (2) duplicar la matriz de la web (pinecone/weaviate/chroma) en vez de extenderla → extender `WHY_VANTADB.comparison`; (3) el repo web no es editable desde aquí → el contrato admite entrega como contenido listo + coordinación.
- **Stop conditions:** repo no editable → entregar la matriz + texto final como artefacto y registrar la publicación pendiente.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Claims Orama sin verificar | Verificación contra docs oficiales + fecha | DISCOVERY |
  | 🟢×🟡 | Matriz duplicada | Extender `WHY_VANTADB.comparison` | diseño |
  | 🟢×🟢 | Publicación bloqueada por repo | Artefacto + registro | cierre |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) claims; (2) duplicación; (3) publicación.
- **Uphill/Downhill:** ⬇️ (2 steps: matriz verificada → publicación)
- **DoD:** task = contrato · commit = `docs:` (o artefacto) · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 1d ✓
- **Skills sugeridas:** documentation-skill · writing-guidelines · ai-seo
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/TS-13.md`

### Task 30: PROV-12 — Publicar wheels PyPI de providers (estrategia H-04)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 1sem
- **Prioridad:** 🟠
- **Archivos clave:** `providers/{openai,ollama,litellm}/` (hoy: Cargo.toml con `publish=false`, `src/{lib,python}.rs`, `tests/test_*.py`, `.pyi`, README — **sin pyproject.toml**, `git ls-files` lo confirma), patrón: `.github/workflows/release-wheels.yml` (matriz ubuntu/macos/windows + aarch64 manylinux_2_28, maturin-action, input TestPyPI, OIDC PyPI), `.github/workflows/providers-ci.yml` (92 líneas, ubuntu-only), `docs/dev/operations/CI_POLICY.md:151-153`
- **Verificación real:** ✅ CÓDIGO-REAL — los 3 providers no tienen camino de distribución: sin `pyproject.toml`/maturin, sin wheels, PyPI 404 (H-04: "Sin camino de distribución... Decidir: publicar a PyPI vs declarar experimental-interno" — estrategia H-04 **aprobada: publicar**); deps PROV-01/02/04 ✅ (compile fix, tests a firma actual, contrato canónico ADR-0033); providers NO son workspace members (CI_POLICY).
- **Gate Justificación:** desbloquea el diferenciador real (storage embebido local acoplado al embed) en la superficie pip; el patrón de release ya existe en el repo (release-wheels.yml) → delta de pyproject + matriz + publish.
- **Gate Result:** ✅ DO
- **Contrato:** wheels publicados **o** dry-run TestPyPI verde (5+ dists) + checklist owner para el publish real; CI multiplataforma (macos/windows/linux x86_64 + aarch64) verde; smoke de instalación en máquina/venv limpio (pip install → import → embed mock); CI_POLICY actualizado; sin cambios de contrato de los providers (PROV-04 cerrado).
- **Pre-mortem:** (1) publish bloqueado por secrets/OIDC → dry-run TestPyPI + checklist owner (patrón MKT-20/Task 71); (2) wheels que compilan en CI pero no instalan limpio → smoke en venv limpio por plataforma; (3) drift de contrato entre build y publish → re-verificar con el dry-run antes del tag.
- **Stop conditions:** publish bloqueado por credenciales → dry-run verde + checklist owner + cierre con nota.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | OIDC/secrets no listos | Dry-run TestPyPI primero | antes del tag |
  | 🟢×🟡 | Wheel no instala limpio | Smoke en venv limpio por plataforma | VERIFY |
  | 🟢×🟢 | Drift build↔publish | Dry-run re-verificado antes del tag | release |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) credenciales; (2) smoke de instalación; (3) drift.
- **Uphill/Downhill:** ⬇️ (3 steps: pyproject+matriz → dry-run → publish/checklist)
- **DoD:** task = contrato (publicados o dry-run + checklist) · commit = `ci(providers):` · release = wheels en PyPI.
- **Validación Appetite vs Effort:** 1sem ≥ 1sem ✓
- **Skills sugeridas:** ci-cd-and-automation · shipping-and-launch · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/PROV-12.md`

### Task 31: PROV-13 — Providers OpenAI/Ollama/LiteLLM en Windows (compilar)

- **Appetite:** max 4d
- **Esfuerzo:** 🟡 2-4d
- **Prioridad:** 🟠
- **Archivos clave:** `providers/{openai,ollama,litellm}/` (no workspace members — se chequean vía `--manifest-path`), `.github/workflows/providers-ci.yml` (hoy `runs-on: ubuntu-latest` únicamente — sin job Windows), `providers/shared_py.rs` (helpers compartidos PROV-05)
- **Verificación real:** ✅ CÓDIGO-REAL — decisión owner 2026-10-01 (Q5/B3): **arreglar, no declarar límite**; Backlog PROV-13: "Los 3 providers no compilan en Windows; requiere fix + CI Windows"; el CI actual (providers-ci.yml, 92 líneas) corre solo en ubuntu → el gap de Windows no está cubierto ni detectado. Error exacto de compilación [a verificar en DISCOVERY: reproducir `cargo check --manifest-path providers/openai/Cargo.toml` en Windows].
- **Gate Justificación:** Windows es la plataforma del owner/ICP local-LLM; un provider que no compila allí rompe el camino pip del diferenciador; la decisión ya está tomada (fix) y el CI Windows es la garantía de no-regresión.
- **Gate Result:** ✅ DO
- **Contrato:** los 3 providers compilan (`cargo check`/`clippy -D warnings` vía manifest-path) y sus tests pasan en Windows; job CI Windows verde (matriz provider); sin regresión en Linux/macOS (providers-ci.yml actual sigue verde); si el fix toca `shared_py.rs`, los 3 providers verificados.
- **Pre-mortem:** (1) el error es de una dep transitiva (pyo3/openssl/etc.) → aislar por provider antes de tocar; si es upstream: pin/workaround documentado; (2) fix en `shared_py.rs` rompe otro provider → matriz completa en el job; (3) CI Windows lento/caro → job con cache y timeout acotado (patrón providers-ci actual).
- **Stop conditions:** 4d sin compilación Windows → registrar el bloqueante exacto (dep + error) + FIND/issue upstream y cerrar con matriz actualizada.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Dep transitiva no compila en MSVC | Aislar por provider; pin/workaround documentado | DISCOVERY |
  | 🟡×🟢 | Fix compartido rompe otro provider | Matriz completa en el job Windows | VERIFY |
  | 🟢×🟢 | Costo del job Windows | Cache + timeout acotado | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) dep transitiva; (2) regresión cruzada; (3) costo CI.
- **Uphill/Downhill:** ⬆️ 1 incógnita (causa raíz del no-compile) → repro en step 1 → luego ⬇️
- **DoD:** task = contrato + job Windows verde · commit = `fix(providers):`/`ci:` · release = changelog (si aplica).
- **Validación Appetite vs Effort:** 4d ≥ 4d ✓
- **Skills sugeridas:** systematic-debugging · ci-cd-and-automation · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/PROV-13.md`

### Task 32: DESKTOP-41 — Smoke-test instalador en VM Windows limpia

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** instaladores NSIS+MSI (`desktop/src-tauri/target/release/bundle/*` — **no presentes localmente**, build primero; Step 3 de DESKTOP-24), `desktop/src-tauri/tauri.conf.json` (bundle targets nsis/msi/dmg/app/appimage/deb; `webviewInstallMode: embedBootstrapper`; `resources: binaries/vanta-cli* + binaries/vantadb-server*`), sidecar `desktop/src-tauri/binaries/{vanta-cli.exe,vantadb-server.exe}`, deep link `vanta://` (plugin deep-link), VM Windows limpia
- **Verificación real:** ✅ CÓDIGO-REAL — `desktop/README.md:124-129`: "No public installer yet. Bundling targets (NSIS/MSI) are configured and `npm run tauri build` produces local installers" → el bundle hay que generarlo; sidecar y deep-link están configurados (`tauri.conf.json`: resources + `plugins.deep-link.schemes: ["vanta"]`; `Cargo.toml`: tauri-plugin-deep-link 2.4.9 + single-instance 2.4.3). Backlog DESKTOP-41: "Step 3 DESKTOP-24 pendiente: instalar NSIS+MSI, verificar arranque, sidecar server, deep link `vanta://`, WebView2 bootstrapper".
- **Gate Justificación:** es el primer smoke end-to-end del instalador real en el entorno del usuario (VM limpia); los componentes están configurados pero nunca verificados como artefacto instalado — evidencia de distribución o deuda explícita.
- **Gate Result:** ✅ DO
- **Contrato:** instalador smoke-testeado en VM limpia: arranque + sidecar server + deep link `vanta://` + WebView2 (bootstrapper embed) verificados; checklist con evidencia (capturas/logs por paso); hallazgos → FINDs (no fixes apurados fuera de scope); tanto NSIS como MSI probados (o el que falle → FIND con repro).
- **Pre-mortem:** (1) no hay bundle local → primer step es `npm run tauri build` (medir costo/tiempo); (2) VM limpia no disponible → registrar como blocker de entorno y entregar checklist + evidencia parcial; (3) deep link requiere registro del esquema en el instalador → verificar post-install (no en dev).
- **Stop conditions:** sin VM limpia disponible → entregar checklist ejecutable + FIND del smoke pendiente (no simular evidencia).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Bundle no generado / build costoso | Build primero; medir y documentar | DISCOVERY |
  | 🟡×🟢 | VM limpia no disponible | Checklist + FIND, sin evidencia simulada | diseño |
  | 🟢×🟡 | Deep link no registra post-install | Verificación específica post-install | VERIFY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) build/entorno; (2) VM; (3) deep link.
- **Uphill/Downhill:** ⬇️ (3 steps: build → instalar en VM → checklist)
- **DoD:** task = contrato (checklist con evidencia) · commit = `docs(desktop):`/`ci:` · release = n/a.
- **Validación Appetite vs Effort:** 2d ≥ 1d ✓
- **Skills sugeridas:** ci-cd-and-automation · systematic-debugging · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/DESKTOP-41.md`

### Task 33: DESKTOP-43 — Auto-update vía `tauri-plugin-updater`

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** `desktop/src-tauri/Cargo.toml` (hoy solo `tauri-plugin-deep-link` + `single-instance` — **sin `tauri-plugin-updater`**), `desktop/src-tauri/tauri.conf.json` (sin `plugins.updater`), `desktop/src-tauri/capabilities/default.json` (sin permisos updater), infra de releases desktop, `docs/dev/research/installer-personalizado/RESEARCH.md` (§F3: "updater Tauri firmado"; :45: "Firma de updater obligatoria")
- **Verificación real:** ✅ CÓDIGO-REAL — no existe el plugin updater ni su config (rg `tauri-plugin-updater` en `desktop/src-tauri` = 0 fuera de docs); Backlog DESKTOP-43: "⏸️ Bloqueada: firma (wontfix DEVOPS-10) + endpoint de manifests | Desbloquear tras decisión de distribución pública. Dep: decisión distribución"; research: el updater exige firma (JSON estático o server) — sin decisión de distribución pública no hay canal que actualizar.
- **Gate Justificación:** el auto-update es requisito de un instalador público (el smoke DESKTOP-41 no lo cubre); con la decisión de distribución aún abierta, el valor de la tarea es dejar el camino listo o el defer justificado con trigger — no construir a ciegas.
- **Gate Result:** ✅ DO
- **Contrato:** updater configurado y probado (update simulado end-to-end: manifest JSON + firma + app que detecta/aplica) **o** decisión de defer registrada con trigger explícito (decisión de distribución pública) + checklist de habilitación; sin tocar el pipeline de releases si se defiere.
- **Pre-mortem:** (1) firma obligatoria (claves + CI secrets) no disponible → sin firma el updater no funciona: decidir con el owner antes de codear (Gate D si hace falta); (2) endpoint de manifests sin hosting → evaluar JSON estático en GitHub Releases (patrón research) vs server; (3) habilitar updater cambia el modelo de confianza del desktop → documentar (ADR corto o sección en ARCHITECTURE.md).
- **Stop conditions:** decisión de distribución no tomada → defer registrado con trigger + checklist (contrato cumplido).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Firma/claves no disponibles | Decidir con owner; Gate D si bloquea | DISCOVERY |
  | 🟢×🟡 | Endpoint de manifests sin definir | JSON estático en Releases (research) | diseño |
  | 🟢×🟢 | Cambio de modelo de confianza | Documentar en ARCHITECTURE/ADR | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) firma; (2) endpoint; (3) confianza.
- **Uphill/Downhill:** ⬆️ 1 incógnita (¿decisión de distribución tomada?) → step 1 → luego ⬇️ o defer
- **DoD:** task = contrato (updater probado o defer con trigger) · commit = `feat(desktop):`/`docs:` · release = n/a.
- **Validación Appetite vs Effort:** 2d ≥ 2d ✓
- **Skills sugeridas:** source-driven-development · ci-cd-and-automation · campaign-executor
- **Estado:** ✅ COMPLETED

### Task 34: DESKTOP-44 — Validación manual Proxy Dashboard con upstream LLM vivo

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 2-4h
- **Prioridad:** 🟡
- **Archivos clave:** `desktop/src/components/proxy/ProxyDashboard.tsx` (paneles TurnReports/sesiones/write-back/rate-limit; polling 5s; consume `GET /snapshot`), `vanta-proxy` (`capture.rs`, `writeback.rs`, `config.rs` rate_limit_per_minute=60), deuda DESKTOP-38, `FIND-155` (dashboard sin `x-vanta-user-key` → 401 desde API-05)
- **Verificación real:** ✅ CÓDIGO-REAL — el dashboard existe y consume el snapshot REST del proxy (polling 5s, LS_KEY `vanta.proxy.url`); el proxy expone los 4 paneles (TurnReport en report.rs/langfuse.rs, writeback en writeback.rs, sesiones team→agent→task, rate-limit en config.rs); **FIND-155 abierto**: sin `x-vanta-user-key` el snapshot da 401 (API-05) → la validación E2E está bloqueada por ese fix. Backlog: "sesión guiada owner+agente, no tarea autónoma" (Dueño: owner).
- **Gate Justificación:** cierra la deuda DESKTOP-38 con evidencia real (upstream LLM vivo) en vez de mocks; es la única validación end-to-end del loop proxy→dashboard y del write-back — humano-en-el-loop por diseño.
- **Gate Result:** ✅ DO
- **Contrato:** sesión owner+agente ejecutada con upstream LLM vivo; TurnReports/sesiones/write-back/rate-limit verificados end-to-end (FIND-155 resuelto en la misma sesión o registrado como bloqueante con evidencia); hallazgos → FINDs; el resultado queda registrado en el task file con la evidencia de la sesión (no simulado).
- **Pre-mortem:** (1) FIND-155 (401) impide el snapshot → incluir el fix de auth (input/storage de key + header) como sub-step de la sesión; (2) sin upstream LLM configurado → la sesión lo requiere; si no hay key/disponibilidad, entregar checklist + FIND; (3) sesión no reproducible por el agente solo → es owner-assisted: preparar el guion de verificación (qué mirar en cada panel) para que la sesión sea corta y productiva.
- **Stop conditions:** sin upstream vivo → checklist + FIND del bloqueante; no forzar mocks como evidencia.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | FIND-155 (401) bloquea el snapshot | Fix de auth como sub-step de la sesión | DISCOVERY |
  | 🟢×🟡 | Upstream LLM no disponible | Checklist + FIND, sin mocks | sesión |
  | 🟢×🟢 | Sesión sin guion = tiempo perdido | Guion de verificación por panel | diseño |

- **Cynefin:** 🟦 obvio (validación guiada)
- **Top 3 riesgos:** (1) auth 401; (2) upstream; (3) guion.
- **Uphill/Downhill:** ⬇️ (2 steps: fix auth → sesión E2E)
- **DoD:** task = contrato (sesión + evidencia o checklist+FIND) · commit = `fix(desktop):`/`docs:` · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 4h ✓
- **Skills sugeridas:** source-driven-development · systematic-debugging
- **Dependencias:** owner presente (sesión guiada) + upstream LLM vivo; FIND-155 como prerequisito.
- **Estado:** ⏳ EN PROGRESO

### Task 35: SHOW-02 — Recetas clicables del playground (5-6)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** repo `ness-e/Vantadb-web`: `src/components/vanta/code-playground.tsx` (471 líneas; array `EXAMPLES` :46-163; dropdown `loadExample` :258), `src/app/playground/page.tsx`, sandbox `public/playground-executor.html` (iframe WASM real, WEB-07), `playground-executor.tsx`
- **Verificación real:** ✅ CÓDIGO-REAL — el playground ya tiene EXACTAMENTE las 6 recetas del contrato en `EXAMPLES` (:48 RAG Mini · :65 Hybrid Search · :84 Graph BFS · :106 TTL Expiry · :123 Batch Insert · :146 Persistence) con dropdown para cargarlas y ejecución en iframe WASM real. **Posible ya-resuelto** → DISCOVERY debe verificar ejecución E2E de las 6 contra el WASM actual y re-scopear: cerrar como ya-resuelto o quedarse con el delta real (p.ej. deep-links por receta, contador de uso).
- **Gate Justificación:** el contrato original (5-6 recetas clicables reutilizando CodePlayground) parece ya implementado en el web; verificarlo evita trabajo duplicado — y si algo no ejecuta, ahí está el delta real.
- **Gate Result:** ✅ DO
- **Contrato:** las 5-6 recetas (RAG, híbrido, grafo, TTL, batch, persistencia) funcionan en el playground con WASM real (verificación de ejecución, no solo existencia de código); sin código muerto; **o** cierre como ya-resuelto con evidencia de la verificación E2E + delta documentado si emerge.
- **Pre-mortem:** (1) las recetas existen pero alguna no ejecuta contra el WASM actual (API drift) → verificar las 6 una por una y arreglar solo lo roto; (2) repo externo (`ness-e/Vantadb-web`) → coordinar como en TS-10/13; (3) "sin código muerto" puede tentar un refactor amplio → scope = recetas, no el playground.
- **Stop conditions:** recetas ejecutando → cerrar ya-resuelto con evidencia; si 2-3d de fixes no alcanzan → FIND del resto.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Receta rota por API drift | Verificación E2E de las 6, fix puntual | DISCOVERY |
  | 🟢×🟡 | Scope creep al playground | Scope = recetas | diseño |
  | 🟢×🟢 | Coordinación repo externo | Mismo flujo que TS-10/13 | cierre |

- **Cynefin:** 🟦 obvio (verificación) / 🟨 si hay fixes
- **Top 3 riesgos:** (1) API drift; (2) scope; (3) coordinación.
- **Uphill/Downhill:** ⬇️ (2 steps: verificar las 6 → cerrar o fix puntual)
- **DoD:** task = contrato (verificado o delta cerrado) · commit = `docs:`/`fix(web):` · release = n/a.
- **Validación Appetite vs Effort:** 3d ≥ 2-3d ✓
- **Skills sugeridas:** frontend-ui-engineering · source-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/SHOW-02.md`

### Task 36: WEB-09 + MKT-22 — Densidad visual del home + panel de la métrica principal

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** (WEB-09) repo `ness-e/Vantadb-web`: `src/app/page.tsx` (wrapper → `home-view.tsx`), `src/components/vanta/trust-bar.tsx`, `visual-audit/` (baselines); (MKT-22) `scripts/north_star_metric.py` (existe; sessions put+search 7-day window; `--json`), `docs/api/PROXY.md:219-245` §North Star metric (ICP-01), `README.md:230` (North Star medido en MCP), panel/reporte a definir
- **Verificación real:** ✅ CÓDIGO-REAL — (MKT-22) la métrica EXISTE y es medible: `scripts/north_star_metric.py` la computa desde el proxy store vía `vanta-cli mcp-call` (self-test incluido) y PROXY.md documenta las dos mitades (proxy-turns + proxy-memory-events); nadie publica el número. (WEB-09) el home actual es modular (`home-view.tsx`, TrustBar ×3 en HEAD — el conteo "73 usos, trust-bar ×11, hero 5 capas" es del review INV-web-01 H-07 y **no se reproduce en el HEAD actual** [a verificar en DISCOVERY: recontar efectos decorativos]); requiere criterio visual del owner (puede quedar diferida).
- **Gate Justificación:** (MKT-22) publicar el North Star convierte la métrica interna en evidencia externa — barato y de alto valor de credibilidad; (WEB-09) es una decisión de diseño fino que solo el owner puede cerrar — el contrato admite "decisión aplicada o diferida con criterio escrito".
- **Gate Result:** ✅ DO
- **Contrato:** (WEB-09) decisión visual del owner aplicada (densidad reducida) o diferida con criterio escrito y fecha; (MKT-22) número de sesiones visible en un panel/reporte actualizado (README o reporte generado por el script), con comando reproducible documentado; sin claims sin evidencia (Regla 11).
- **Pre-mortem:** (1) el conteo WEB-09 del review está stale (repo evolucionó) → recontar en DISCOVERY antes de proponer cambios; (2) MKT-22: el número depende de datos reales del proxy (¿hay store con sesiones?) → publicar metodología + placeholder honesto si no hay datos, nunca un número inventado; (3) dos tareas en un bloque → si divergen, separar task file `MKT-22.md` (ya previsto).
- **Stop conditions:** WEB-09 sin criterio del owner → diferir con nota; MKT-22 sin datos reales → publicar metodología + comando, marcar el número como pendiente de primera corrida.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Conteo WEB-09 stale | Recontar en DISCOVERY antes de actuar | DISCOVERY |
  | 🟡×🟡 | Número sin datos reales | Metodología + comando; nunca inventar | diseño |
  | 🟢×🟢 | Dos tareas divergen | Separar `MKT-22.md` si hace falta | diseño |

- **Cynefin:** 🟦 obvio (MKT-22) · 🟨 complicado (WEB-09, criterio visual)
- **Top 3 riesgos:** (1) conteo stale; (2) datos reales; (3) split de tareas.
- **Uphill/Downhill:** ⬇️ (2-3 steps: recontar/decidir → publicar número)
- **DoD:** task = contrato (decisión + número visible) · commit = `docs:` · release = n/a.
- **Validación Appetite vs Effort:** 2d ≥ 2d ✓
- **Skills sugeridas:** vanta-design-orchestrator · frontend-ui-engineering · documentation-skill
- **Estado:** ✅ COMPLETED

## F2 — Memoria I (cimientos del diferenciador)

> **Gate de fase:** ✅ CUMPLIDO — Tasks 37-49 expandidas a nivel F0 el 2026-10-05 (pre-mortem + Risk Register + stop conditions incluidos); listas para ejecutar.

### Task 37: MEMG-01 — Detección de contradicción en ingesta L1

- **Appetite:** max 3d
- **Esfuerzo:** 🟠 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/core/record/l1_writer.rs` (`write_memory:124`, `apply_dedup_batch:255`, `put_record:316`), `core/record/l1_dedup.rs` + `core/prompts/l1_dedup.rs` (juicio de dedup L1 — punto de extensión natural), `core/dream/mod.rs:335` (`resolve_contradictions`) + `core/record/lifecycle.rs:86` (`mark_contradiction` — base reutilizable; `ContradictionProvenance` re-exportado en `dream/mod.rs:54`)
- **Verificación real:** ✅ CÓDIGO-REAL — `contradict` en `src/` (core) = 0 hits (rg, case-insensitive); la detección EXISTE en `vanta-memory` solo del lado dream: `resolve_contradictions` (`core/dream/mod.rs:335-371`) llama `mark_contradiction` (`core/record/lifecycle.rs:86` — setea `superseded_by` en el viejo, no borra; test `:252`) y emite `ContradictionProvenance`; el gap es la **ingesta L1**: `apply_dedup_batch` (`l1_writer.rs:255`) solo decide store/update/skip (DELTA §P1, re-verificado HEAD).
- **Gate Justificación:** gap #1 de "store-and-retrieve" vs "memoria" (DELTA P1; ámbito §8 Consistencia y conflictos); la confianza ya existe (SCH-02) y la provenance de dream es la base — el delta es detectar al ingerir, no al soñar; sin esto el hecho viejo compite por relevancia y gana por RRF.
- **Gate Result:** ✅ DO
- **Contrato:** la ingesta L1 detecta contradicciones contra registros vigentes del mismo namespace/sesión y las marca (supersede/flag con provenance) con test dedicado (caso canónico: "me gusta X" → "ya no me gusta X" marca el viejo); registros no-contradictorios sin cambio de semántica (mismos store/update/skip); reutiliza `mark_contradiction` (cero semántica paralela); cuarentena por contradicción queda fuera (MGR-13 §3.4 la difiere a v1.0/FIND — evitar doble semántica con MGR-06).
- **Pre-mortem:** (1) doble juicio LLM (extracción + contradicción) encarece el flush → extender el juicio de dedup existente (patrón `l1_batch` MEM-69) antes que una llamada nueva; (2) falsos positivos marcan vigentes sanos → señal conservadora (solo contradicción explícita), marca revisable, no destructiva; (3) colisión con el ownership de dream (`resolve_contradictions` es de consolidación) → misma función, punto de llamada distinto; dream intacto.
- **Stop conditions:** 3d sin contrato verde → entregar detección conservadora en el path dedup existente + FIND del resto (p.ej. contradicciones cross-sesión).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Costo LLM extra por flush | Extender el juicio de dedup existente (l1_batch) | diseño |
  | 🟡×🟠 | Falso positivo marca vigente sano | Señal conservadora + marca revisable (no destructiva) | VERIFY |
  | 🟢×🟡 | Doble semántica con dream/MGR-06 | Reutilizar `mark_contradiction`; cuarentena diferida (MGR-13 §3.4) | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) costo LLM; (2) falsos positivos; (3) doble semántica.
- **Uphill/Downhill:** ⬆️ 1 incógnita (¿el juicio de contradicción viaja en el prompt de dedup sin degradar la extracción?) → DISCOVERY → luego ⬇️
- **DoD:** task = contrato + test · commit = `feat(memory):` · release = changelog (vanta-memory → minor).
- **Validación Appetite vs Effort:** 3d ≥ 3d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-01.md`

### Task 38: MEMG-02 — Outcome loop → refuerzo de confianza post-recall

- **Appetite:** max 3d
- **Esfuerzo:** 🟠 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs` (`perform_auto_recall`, `RecalledMemory`), `core/record/l1_reader.rs` (recall/scoring), core: `src/sdk/types/record.rs:246-253` (`confidence`/`last_validated_at_ms` — campos v2 SCH-02), `src/sdk/api/memory.rs` + MCP `vantadb-mcp/src/handlers/tools.rs` (superficie de la op de refuerzo)
- **Verificación real:** ✅ CÓDIGO-REAL — `reinforce` en código = 4 hits y ninguno toca confianza (3 prompts: `core/prompts/persona_generation.rs:91,94,197` + 1 comentario: `src/sdk/search/fusion.rs:88`); `outcome` = 14 hits en `vanta-memory/src` / 48 en `src/` (todos usos de la palabra, ninguno es loop de feedback); la confianza post-recall es estática: `perform_auto_recall` devuelve `RecalledMemory` sin señal de uso, y `last_validated_at_ms` (`record.rs:253`) existe sin writer (DELTA §P1, re-verificado HEAD).
- **Gate Justificación:** queja canónica contra Mem0/Letta (DELTA P1): sin feedback la confianza es decorativa y la meta-memoria (dim 8) no cierra; SCH-02/04 ya dejaron campos + consumo read-side — falta el write-side del loop.
- **Gate Result:** ✅ DO
- **Contrato:** el resultado de una recall (¿resolvió? ¿corrigió?) realimenta el registro vía una op real de refuerzo (SDK/MCP) que actualiza `confidence` + `last_validated_at_ms` con política declarada (bump/decay acotado y saturado); test: recall → refuerzo positivo → score sube / negativo → baja o marca; métrica de ordenamiento antes/después documentada; sin cambio de defaults para quien no llama la op; calibración excluida (VER-08 / MGR-12 §5.2, §6.3).
- **Pre-mortem:** (1) "outcome" no falsificable → API explícita del host (used/corrected/unused), nunca inferencia silenciosa; (2) inflación de confianza por refuerzos triviales → saturación + tasa acotada por ventana; (3) scope creep a calibración/jueces LLM → excluido (MGR-12 §6.3); solo contadores.
- **Stop conditions:** 3d sin loop cerrado → entregar la op de refuerzo + test unitario + FIND del consumo automático en `auto_recall`.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Semántica de "outcome" ambigua | API explícita (used/corrected/unused) fijada en DISCOVERY | DISCOVERY |
  | 🟡×🟠 | Inflación de confianza | Saturación + tasa acotada por ventana | VERIFY |
  | 🟢×🟡 | Scope a calibración | Excluido (MGR-12 §6.3); solo contadores | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) semántica outcome; (2) inflación; (3) scope.
- **Uphill/Downhill:** ⬆️ 1 incógnita (shape/semántica de la señal de outcome) → DISCOVERY → luego ⬇️ (op → política → test/métrica)
- **DoD:** task = contrato + test + métrica · commit = `feat(memory):` · release = changelog.
- **Validación Appetite vs Effort:** 3d ≥ 3d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · performance-optimization · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-02.md`

### Task 39: MEMG-11 — Adopción del motor core en `vanta-memory` (recall híbrido + escritura batch)

- **Appetite:** max 2sem
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🔴
- **Archivos clave:** `vanta-memory/src/core/record/l1_reader.rs:104-213` (dual-pool propio: `significant_terms:104`, `rrf_merge:213`, scan `read_namespace_records`), `core/hooks/auto_recall.rs` (consumidor), `core/record/l1_writer.rs:316` (`put_record` → `db.put` 1×1), core: `src/sdk/api/memory.rs:593` (`put_batch` group-commit), `src/index/search/` (`layer.rs` HNSW beam) + `src/planner.rs:79-124` (RRF WIRE-08), `src/sdk/serialization/vector_types.rs:111` (`query_sparse`), `benches/canonical_p99.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `put_batch` en `vanta-memory/src` = 0 hits; `text_index|bm25|tantivy` = 0 usos reales (solo 3 menciones en comentarios/prompts: `skill_extractor.rs:15,124`, `auto_recall.rs:237`); recall propio confirmado (dual-pool D38 + `rrf_merge` local + scan); core con search híbrido (`src/index/search/layer.rs`, planner RRF `src/planner.rs:82-124`) y batching (`put_batch` `memory.rs:593` — chunks + group-commit); VER-08 ✅ con baseline `canonical_p99` (MGR-19 §1: p50 2.2388 / p95 4.248 / p99 4.9987 ms, seed 42).
- **Gate Justificación:** el motor híbrido es el diferenciador del core y la memoria lo reimplementó más débil (sin índice de texto, sin HNSW); batching/sparse ya productizados (WIRE-03/06) pero nadie los consume desde `vanta-memory` — es reutilización, no feature nuevo (DELTA P1).
- **Gate Result:** ✅ DO
- **Contrato:** recall L1 sobre búsqueda híbrida del core (BM25+HNSW+RRF del planner) + escrituras del pipeline vía `put_batch` (group-commit); **A/B documentado sin regresión p99** (`canonical_p99` before/after + métrica de recall); orden: medir → migrar → A/B; paridad de resultados con el recall actual en el fixture (o divergencia justificada y documentada); rollback = dual-path por config durante la migración.
- **Pre-mortem:** (1) la semántica dual-pool legacy (record sin vector nunca se dropea) se pierde → verificar exclusión por vector nulo antes de migrar; (2) A/B con baseline móvil (100k×1536d es largo) → fijar comando/hardware/seed 42 y comparar contra el número de MGR-19 §1; (3) BENCH-02 es F5 (Task 67), no hard blocker → la medición usa `canonical_p99` + VER-08; registrar si falta la pata de calidad BEIR.
- **Stop conditions:** 2sem sin A/B → entregar (a) recall híbrido detrás de flag + (b) `put_batch` migrado, cada uno con su medición; FIND del resto.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Semántica dual-pool se pierde en el core | Test de exclusión por vector nulo antes de migrar | DISCOVERY |
  | 🟡×🟡 | A/B con baseline móvil | Comando/hardware/seed fijados; contra MGR-19 §1 | A/B |
  | 🟢×🟡 | BENCH-02 (F5) no disponible | Medición con `canonical_p99` + VER-08; FIND si falta BEIR | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) semántica dual-pool; (2) baseline del A/B; (3) dependencia F5.
- **Uphill/Downhill:** ⬆️ 1 incógnita (paridad de la semántica dual-pool sobre el motor core) → DISCOVERY → luego ⬇️ (medir → migrar → A/B)
- **DoD:** task = contrato + A/B · commit = `feat(memory):`/`perf(memory):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 2sem ≥ 2sem ✓
- **Skills sugeridas:** source-driven-development · performance-optimization · rust-write-tests · doubt-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-11.md`

### Task 40: MEMG-12 — Semántica v2 write-side en el pipeline (`confidence`/`valid_at`/TTL)

- **Appetite:** max 2sem
- **Esfuerzo:** 🟠 1-1.5sem
- **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/core/conversation/l0_recorder.rs:243` (`ttl_ms: None`), `core/dream/mod.rs:537` (`ttl_ms: None` en promote), `services/pipeline_worker.rs:775` (`ttl_ms: None`), `core/record/l1_writer.rs:316` (`put_record`), core: `src/sdk/types/record.rs:150-157,246-253` (campos v2), `src/sdk/api/memory.rs:98` (`enter_quarantine` + opciones de write), `seed/md_import.rs` (round-trip), specs MGR-10/12/13
- **Verificación real:** ✅ CÓDIGO-REAL — `valid_at`/`confidence`/`quarantine` en `vanta-memory/src` solo aparecen en `seed/` (round-trip de formato: `md_import.rs:15,25`, `mod.rs` — no escritura del pipeline); `ttl_ms: None` confirmado en los 3 sitios citados (rg HEAD: `l0_recorder.rs:243`, `pipeline_worker.rs:775`, `dream/mod.rs:537`); campos v2 y ops de cuarentena existen en el core (`record.rs:246-253`, `enter_quarantine` `memory.rs:98`, ADR-046) y el gate de lectura ya filtra (`include_quarantined: false` `l1_reader.rs:53`).
- **Gate Justificación:** con defaults, confianza y bitemporalidad quedan inertes para la memoria ("confidence=0 en vanta-memory"); el gate de cuarentena funciona en lectura (SCH-05) pero nadie marca dudoso al escribir — la semántica v2 completa solo vale si el pipeline la escribe (DELTA P1; consume MGR-10 §1.3, MGR-12 §3, MGR-13 §3.2).
- **Gate Result:** ✅ DO
- **Contrato:** L0→L3 escribe `confidence`/`valid_at_ms`/TTL semántico reales (no defaults) + transiciones de cuarentena desde dream/ingesta donde la spec las define (T1b promoción derivada — MGR-13 §3.2; invariante sticky I2 preservado); round-trip test (put → export → import → mismos campos); invariantes MGR-10 §1.4 (`valid_at <= invalid_at`; backfill puro); sin breaking del wire (`#[serde(default)]`); política de valores por tipo declarada.
- **Pre-mortem:** (1) ¿qué TTL/valid_at por tipo? — MGR-09 (retention) no tiene research-doc → política mínima declarada en DISCOVERY + [a verificar] antes de inventar fórmula; (2) doble escritura de confianza (extractor vs writer) → un solo punto (`l1_writer`) con valor calculado aguas arriba; (3) cuarentena por contradicción solapa MGR-06/MEMG-01 → solo señales explícitas ya definidas (MGR-13 §3.4).
- **Stop conditions:** 1.5sem → entregar `confidence`+`valid_at` en L1 (los dos sitios del pipeline) + FIND de TTL/dream-promote restante.
- **Dependencias:** SCH-02/04/05 (hechos), MEMG-01/02 (complementan), MGR-09 (spec ausente).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Política TTL/valid_at sin spec (MGR-09) | Política mínima declarada + [a verificar] en DISCOVERY | DISCOVERY |
  | 🟡×🟢 | Doble escritura de confianza | Un solo punto de escritura (l1_writer) | diseño |
  | 🟢×🟡 | Solape cuarentena (MGR-06/MEMG-01) | Solo señales explícitas de MGR-13 §3.4 | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) política sin spec; (2) doble escritura; (3) solape cuarentena.
- **Uphill/Downhill:** ⬆️ 1 incógnita (política de valores por tipo) → DISCOVERY → luego ⬇️ (L1 → dream/promote → round-trip)
- **DoD:** task = contrato + round-trip · commit = `feat(memory):` · release = changelog.
- **Validación Appetite vs Effort:** 2sem ≥ 1.5sem ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-12.md`

### Task 41: MEMG-13 — Superficies core restantes en memoria (IQL / versiones / snapshots / filtros)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/core/record/l1_reader.rs` (recall/list plano), `core/hooks/auto_recall.rs`, core: `src/parser/` + `src/executor.rs:170` (`execute_hybrid` — IQL), `src/sdk/api/memory.rs:842,860` (`get_version`/`versions`), snapshots (CLI `cli.rs:296,411` + MCP `snapshot_create`/`snapshot_restore`), `src/sdk/types/record.rs:48` (`MemoryFilterItem`), `:339` (cursor), `:363` (`min_confidence`)
- **Verificación real:** ✅ CÓDIGO-REAL — `query_iql|IQL|get_version` en `vanta-memory/src` = 0 hits (rg HEAD); el core sí expone las 4 superficies: IQL (`src/parser/mod.rs`, `execute_hybrid` `src/executor.rs:170`, MCP `query_iql`), versiones (`memory.rs:842,860`), snapshots (`Snapshot` CLI `cli.rs:296` + subcomandos `:411`; MCP `snapshot_create`/`restore`), filtros/cursors (`MemoryFilterItem`/`cursor`/`min_confidence` en `record.rs`); en memoria solo `include_quarantined: false` (`l1_reader.rs:53`) usa una op del core — el recall sigue siendo scan plano.
- **Gate Justificación:** cierra los últimos huecos de "reutilizar en vez de reimplementar" y habilita auditoría/diff/backup de memoria sin código core nuevo (DELTA P2; STU-03).
- **Gate Result:** ✅ DO
- **Contrato:** la memoria consume IQL + versiones + snapshots + filtros/cursors del core donde apliquen (mínimo: diff de L1 vía `versions`, backup/restore vía snapshot, filtros/cursor en recall); tests por superficie; cada superficie no consumida queda con motivo explícito (FIND o nota en task file); sin reimplementaciones nuevas.
- **Pre-mortem:** (1) superficie sin caso de uso real en memoria → adoptar solo donde aplique ("evaluar y consumir", no "todo"); (2) IQL en recall cambia semántica de scoring → dual-path/flag; (3) snapshot bloquea por locks → flujo CLI existente + DB temp en test.
- **Stop conditions:** 1sem → entregar las 2 superficies de mayor valor (p.ej. versions + snapshot) + FIND del resto.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Superficie sin caso de uso | Adoptar solo donde aplique; motivo explícito por omisión | DISCOVERY |
  | 🟡×🟡 | IQL cambia semántica del recall | Dual-path/flag | VERIFY |
  | 🟢×🟡 | Snapshot bloquea por locks | Flujo CLI existente + DB temp | test |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) caso de uso; (2) semántica del recall; (3) locks de snapshot.
- **Uphill/Downhill:** ⬇️ (3 steps: evaluar por superficie → adoptar → tests)
- **DoD:** task = contrato + tests por superficie · commit = `feat(memory):` · release = changelog.
- **Validación Appetite vs Effort:** 1sem ≥ 5d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-13.md`

### Task 42: MEMG-07 — Forgetting curves sobre L1

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/core/record/lifecycle.rs` (`bump_heat:62`, `decay_heat:72`, `is_prune_eligible:111`, `PRUNE_HEAT_THRESHOLD:40`), `vanta-memory/tests/heat_decay.rs` (fixture existente), core `src/eviction.rs:93` (`BayesianDecay` — eviction, no L1), FUT-10/N-09 (fórmula sin canónico)
- **Verificación real:** ✅ CÓDIGO-REAL — el decay actual es heat entero (MEM-60): `bump_heat` en cada read, `decay_heat` = shift right por pass, umbral de poda (`lifecycle.rs:62,72,111`); el header declara "no float decay curve" (ponytail) y el test `heat_decay.rs:62` cubre bump→decay→prune; `BayesianDecay` vive en el core (`src/eviction.rs:93`) y no corre sobre L1; `forget` en `vanta-memory` = 2 hits (sanitize + doc) — no hay curva por tipo/edad ni descarte.
- **Gate Justificación:** "el decay existe; el descarte no" (DELTA P1; FUT-10) — sin curva real L1 solo crece o poda por umbral binario; es pata de la memoria duradera (MGR-11/N-09 siguen abiertos → política, no claim calibrado).
- **Gate Result:** ✅ DO
- **Contrato:** curva de olvido real sobre L1 (decay por tipo/edad, configurable; parámetros declarados y testeados con valores conocidos) + integración con el pass de mantenimiento + métrica (cuánto decae / qué no expira); sin pérdida de registros no-expirados (la curva deprioriza, nunca purga — el descarte sigue siendo el gate explícito); fórmula documentada como política (no claim calibrado — N-09 abierto).
- **Pre-mortem:** (1) no existe fórmula canónica (N-09 abierto; MGR-11 sin research-doc) → parámetros como política + [a verificar en DISCOVERY] contra FUT-10; no vender calibración; (2) reemplazar heat rompe tests/umbrales → extender por capas (heat como señal + curva) o migrar con defaults equivalentes; (3) borrado por curva = pérdida → la curva jamás borra.
- **Stop conditions:** 2d → entregar decay configurable por tipo + test + métrica; FIND del descarte automático si no cierra.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Fórmula sin canónico (N-09/MGR-11) | Política declarada + [a verificar] en DISCOVERY | DISCOVERY |
  | 🟡×🟡 | Compat con heat/umbrales actuales | Extender por capas o defaults equivalentes | VERIFY |
  | 🟢×🟠 | Purga por curva | La curva nunca borra; gate explícito intacto | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) sin canónico; (2) compat heat; (3) borrado.
- **Uphill/Downhill:** ⬆️ 1 incógnita (parametrización de la curva) → DISCOVERY → luego ⬇️
- **DoD:** task = contrato + test + métrica · commit = `feat(memory):` · release = changelog.
- **Validación Appetite vs Effort:** 3d ≥ 2d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-07.md`

### Task 43: MEMG-20 — Checkpoints reanudables de tarea (dim 1)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/utils/checkpoint.rs` (`Checkpoint`/`CheckpointManager`/`RunnerSessionState.last_l1_cursor`), `services/pipeline_worker.rs:42,437,550,587,624,861` (integración existente), `core/state/types.rs:16` (`PipelineSessionState`)
- **Verificación real:** ✅ CÓDIGO-REAL — la base existe y está cableada: `Checkpoint` persiste `total_processed`, `runner_states` (cursors L0/L1 por sesión) y `pipeline_states` (`checkpoint.rs:28-60`), con integración en el worker (`pipeline_worker.rs:437,550,587,624,861`; patch de runner state `:859-861`); lo que falta es la semántica de checkpoints **de tarea** (paso actual + resultados parciales + resume tras interrupción/compactación) — dim1 la propone [PROPUESTA] (Backlog MEMG-20; MemGPT arXiv 2310.08560).
- **Gate Justificación:** gap operativo #1 de la memoria de trabajo (dim1); la infraestructura ya está cableada → delta acotado de semántica, no de plumbing (DELTA P2).
- **Gate Result:** ✅ DO
- **Contrato:** API mínima de checkpoint de tarea (paso actual + resultados parciales + estado) reanudable: test de reanudación (interrupción a mitad → nueva instancia retoma del checkpoint sin repetir pasos completados); persiste en el store (principio del módulo: un JSON record, RMW atómico in-process); sin romper `RunnerSessionState`/checkpoints de pipeline existentes.
- **Pre-mortem:** (1) confundir checkpoint de pipeline (cursor) con checkpoint de tarea → tipos/namespaces separados, no reutilizar campos; (2) scope creep a task engine completo → API mínima declarada ANTES (paso + payload parcial + versión), sin orquestador nuevo; (3) "resume tras compactación" del agente es del host, no del pipeline → documentar el límite.
- **Stop conditions:** 5d → entregar API + test de reanudación del pipeline y FIND del consumo por el host.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Confusión checkpoint pipeline vs tarea | Tipos/namespaces separados | diseño |
  | 🟡×🟢 | Scope creep a task engine | API mínima escrita antes de codear | DISCOVERY |
  | 🟢×🟡 | Límite host (compactación del agente) | Documentar el límite en el task file | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) confusión de tipos; (2) scope; (3) límite host.
- **Uphill/Downhill:** ⬇️ (3 steps: API → persistencia → test de reanudación)
- **DoD:** task = contrato + test de reanudación · commit = `feat(memory):` · release = changelog.
- **Validación Appetite vs Effort:** 1sem ≥ 5d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-20.md`

### Task 44: MEMG-21 — Scoring multi-señal L1 (recencia + importancia) + reflexión periódica

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/core/record/l1_reader.rs` (scoring actual: overlap + cosine dual-pool + RRF), `core/hooks/auto_recall.rs`, `core/record/lifecycle.rs` (heat), reflexión: nuevo pase sobre episódica (precedente `core/dream/`), fuentes: Park et al. arXiv 2304.03442 + CrewAI (composite scoring) — MGR-15 sin research-doc
- **Verificación real:** ✅ CÓDIGO-REAL — `recency|importance` en `vanta-memory/src` = 0 hits (rg case-insensitive); el scoring actual es keyword-overlap + cosine + RRF (`l1_reader.rs:104-213`) con heat como única señal temporal (`lifecycle.rs`); no hay reflexión (rg `reflect` = solo comentarios/prompts) — dim2 marca el gap (Backlog MEMG-21; deps MEMG-07 + MGR-15).
- **Gate Justificación:** sin scoring compuesto la recall ordena por relevancia léxica/vectorial y el tiempo/importancia no pesan; sin reflexión el sistema degenera (Generative Agents: 48h simuladas) y CrewAI ya vende composite scoring (DELTA P2).
- **Gate Result:** ✅ DO
- **Contrato:** scoring compuesto (recencia + relevancia + importancia) integrado al recall con decaimiento (extiende MEMG-07) + reflexión periódica sobre episódica (lecciones) con test; métrica de ordenamiento antes/después (fixture); pesos configurables con defaults declarados; sin claims calibrados (política, no benchmark).
- **Pre-mortem:** (1) MGR-15 (research Reflexion) no existe como doc → derivar de la fuente citada (Park) o registrar FIND si la spec falta — no inventar; (2) pesos alteran recall de tests existentes → defaults neutros + opt-in; (3) reflexión = costo LLM → pase periódico opt-in (patrón dream) con degradación P4.
- **Stop conditions:** 3d → entregar scoring compuesto + métrica; reflexión como slice separado con FIND si no cierra.
- **Dependencias:** MEMG-07 (decay), MGR-15 (spec ausente — ver pre-mortem).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | MGR-15 sin research-doc | Derivar de Park/CrewAI o FIND; no inventar spec | DISCOVERY |
  | 🟡×🟠 | Regresión del recall existente | Defaults neutros + opt-in + métrica before/after | VERIFY |
  | 🟢×🟡 | Costo LLM de la reflexión | Pase opt-in patrón dream; degrada P4 | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) spec MGR-15; (2) regresión de recall; (3) costo reflexión.
- **Uphill/Downhill:** ⬆️ 1 incógnita (fórmula/pesos y ausencia de MGR-15) → DISCOVERY → luego ⬇️
- **DoD:** task = contrato + métrica · commit = `feat(memory):` · release = changelog.
- **Validación Appetite vs Effort:** 3d ≥ 3d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · performance-optimization · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-21.md`

### Task 45: WIRE-14 — T1: Seam de host aditivo en el arranque (`conversation_trigger` + servicio)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟠
- **Archivos clave:** `src/server/bootstrap.rs` (`run:284` — punto de wiring; `conversation_trigger: None` `:332`; patrón spawn/join del sweeper `:363`/`:381`), `src/server/state.rs:96` (trait `ConversationTrigger`) + `:134` (campo `ServerState`), `src/server/mod.rs:46-47` + `routing.rs:27-28` (re-exports), `src/gc.rs:101-176` (espejo spawn/join), ADR-0054 T1
- **Verificación real:** ✅ CÓDIGO-REAL — el trait existe (`state.rs:96`, doc: post-save hook de `POST /api/v2/conversations`), el campo existe (`state.rs:134`) y el arranque lo deja en `None` (`bootstrap.rs:332`); el patrón de servicio de fondo con arranque/parada graceful ya existe (TTL sweeper: spawn `bootstrap.rs:363`, join `:381`, `gc.rs:101-176`); **corrección verificada**: la ruta legacy `/conversation/add` ya no existe — migrada a `POST /api/v2/conversations` (e2e `vantadb-server/tests/e2e.rs:781` asserta 404 del legacy) — el seam se ancla a la ruta vigente.
- **Gate Justificación:** ADR-0054 T1 — sin seam aditivo el host (`vantadb-server`) no puede inyectar trigger ni servicio; es prerequisito de WIRE-15/16 y la única vía legal (ciclo Cargo: `vanta-memory → vantadb` prohibido).
- **Gate Result:** ✅ DO
- **Contrato:** seam aditivo (sin breaking) que permite inyectar `ConversationTrigger` + un servicio de fondo con arranque/parada graceful; `run(config)` actual intacto (defaults = comportamiento actual, `None`); test de servicio fake spawneado/joineado; `cargo test -p vantadb --features server` verde.
- **Pre-mortem:** (1) firma nueva rompe callers (`cli_server::run` usado por vanta-cli y vantadb-server) → variante aditiva (`run_with_*`/hooks) con default, no cambio de firma; (2) servicio fake no joinea en tests → patrón exacto del TTL sweeper (watch/join) + test que lo prueba; (3) forma exacta del seam (hooks struct vs builder) [a verificar en DISCOVERY] con el blast radius de `run`.
- **Stop conditions:** 5d → seam mínimo (trigger inyectable + spawn/join genérico) + FIND si falta el servicio genérico.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Breaking de `run` para callers existentes | Variante aditiva con defaults; firma actual intacta | diseño |
  | 🟡×🟡 | Servicio fake no joinea | Patrón exacto TTL sweeper + test de join | VERIFY |
  | 🟢×🟢 | Forma del seam | [a verificar en DISCOVERY] con blast radius de `run` | DISCOVERY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) firma; (2) join; (3) forma del seam.
- **Uphill/Downhill:** ⬇️ (3 steps: seam → servicio fake → test)
- **DoD:** task = contrato + test de servicio fake · commit = `feat(server):` · release = changelog.
- **Validación Appetite vs Effort:** 1sem ≥ 5d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/WIRE-14.md`

### Task 46: WIRE-15 — T2: Servicio scheduler en `vanta-memory` (`run_pass` + loop con shutdown)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/services/` (módulo nuevo `scheduler.rs` — `mod.rs` hoy = pipeline_worker + conversation_hook), `utils/timer_scanner.rs:1-7` (`TimerScanner::run_once`), `services/pipeline_worker.rs:232,251` (`run_once`/`reclaim_stale`), espejo `src/gc.rs:101-176` (`MemoryTtlSweeper`), runner FIND-112, `vanta-memory/Cargo.toml:72` (`http-server = ["vantadb/server"]`), ADR-0054 T2
- **Verificación real:** ✅ CÓDIGO-REAL — las tres piezas del pass existen y son pull-based: `TimerScanner::run_once` (`timer_scanner.rs:1-7` — "the owner calls run_once"), `PipelineWorker::run_once`/`reclaim_stale` (`pipeline_worker.rs:232,251`); no existe servicio de loop en `vanta-memory` (`services/` = pipeline_worker + conversation_hook); el espejo loop/shutdown está en `src/gc.rs` (`MemoryTtlSweeper` spawn/join/shutdown `:101-176`, gated `server` `:100-141`); feature `http-server` existe (`Cargo.toml:72`).
- **Gate Justificación:** ADR-0054 T2 — el crate de lógica aporta el pass reutilizable (cero lógica de pipeline nueva) y el host solo el driver; sin esto WIRE-16 duplicaría lógica de timers/worker.
- **Gate Result:** ✅ DO
- **Contrato:** `run_pass` (timers + worker + reclaim) + loop helper con shutdown graceful (espejo `MemoryTtlSweeper`), feature-gated; runner por pass (patrón FIND-112), sin runner → degrada P4 (skip observable, nunca bloquea); tests feature-gated verdes: `cargo test -p vanta-memory --features http-server` (pass procesa, timers disparan, shutdown joinea, sin runner degrada).
- **Pre-mortem:** (1) el loop exige Tokio en `vanta-memory` (hoy pull-based puro) → helper feature-gated bajo `http-server` (precedente conversation_hook), default build intacto; (2) doble dueño de cola si el host corre dos loops → contrato del ADR: un solo dueño (writer), documentado; (3) secrets del runner → solo env (R-5).
- **Stop conditions:** 5d → entregar `run_pass` + test; FIND del loop helper si no cierra.
- **Dependencias:** WIRE-14 (seam).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Tokio/feature en el crate pull-based | Helper feature-gated (http-server); default intacto | diseño |
  | 🟡×🟠 | Doble dueño de cola | Regla writer del ADR; documentar | cierre |
  | 🟢×🟡 | Secrets del runner | Solo env (R-5) | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) feature/Tokio; (2) doble dueño; (3) secrets.
- **Uphill/Downhill:** ⬇️ (3 steps: pass → loop helper → tests feature-gated)
- **DoD:** task = contrato + tests `--features http-server` · commit = `feat(memory):` · release = changelog.
- **Validación Appetite vs Effort:** 1sem ≥ 5d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/WIRE-15.md`

### Task 47: WIRE-16 — T3: Wiring del wrapper `vantadb-server` (queue + bridge + loop)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-server/src/main.rs` (dispatch http/mcp), `vantadb-server/src/lib.rs`/`server.rs` (re-exports), `vantadb-server/Cargo.toml` (dep `vanta-memory` — hoy transitiva vía `vantadb-mcp/Cargo.toml:27`), `vanta-memory/src/services/conversation_hook.rs:36,93` (`HttpCaptureBridge`/`run_bridge_pass`), env `VANTADB_SCHEDULER_*` (R-5), e2e `vantadb-server/tests/e2e.rs`, ADR-0054 T3
- **Verificación real:** ✅ CÓDIGO-REAL — `vantadb-server` es el wrapper legal (`vanta-memory` llega transitiva por `vantadb-mcp/Cargo.toml:27`; dep directa pendiente); el productor existe: `HttpCaptureBridge` implementa `ConversationTrigger` (captura L0 + enqueue L1, `conversation_hook.rs:36-90`) y `run_bridge_pass` (`:93`); el patrón e2e existe (`vantadb-server/tests/e2e.rs`; ruta vigente `POST /api/v2/conversations`); el arranque hoy no inyecta trigger (`bootstrap.rs:332`).
- **Gate Justificación:** ADR-0054 T3 — cierra el residual MEM-55 (el puente existe y no está cableado) y convierte a `vantadb-server` en host del scheduler; delta de build ~0 (vanta-memory ya en el grafo) y es el único entry point legal.
- **Gate Result:** ✅ DO
- **Contrato:** wiring completo con las 4 verificaciones e2e verdes: (1) `POST /api/v2/conversations` → L0 → pass → `l1/<thread_id>`; (2) restart (cola efímera re-encolable desde L0 persistido); (3) disabled (`VANTADB_SCHEDULER_*` = off, sin loop); (4) sin runner (degrada P4, no bloquea); env config `VANTADB_SCHEDULER_*` documentada (secrets solo env, R-5); shutdown graceful (join).
- **Pre-mortem:** (1) el binario es default-member (Fast Gate <5min, ADR-0031) → feature-gate del scheduler + delta de build medido; (2) doble scheduling si MCP y HTTP corren juntos → regla writer (solo el writer ejecuta) + test disabled; (3) defaults inseguros del intervalo → default seguro y documentado [a verificar en DISCOVERY: default del intervalo].
- **Stop conditions:** 5d → entregar wiring HTTP + verificaciones (1) y (3); FIND de restart/sin-runner si faltan.
- **Dependencias:** WIRE-15 (servicio); WIRE-14 (seam).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Fast Gate por build del binario | Feature-gate + delta medido (ADR-0031) | diseño |
  | 🟡×🟠 | Doble scheduling (MCP+HTTP) | Regla writer + test disabled | VERIFY |
  | 🟢×🟡 | Default del intervalo | [a verificar en DISCOVERY]; default seguro | DISCOVERY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) Fast Gate; (2) doble scheduling; (3) default del intervalo.
- **Uphill/Downhill:** ⬇️ (4 steps: dep+queue → bridge → loop → e2e)
- **DoD:** task = contrato + e2e · commit = `feat(server):` · release = changelog.
- **Validación Appetite vs Effort:** 1sem ≥ 5d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/WIRE-16.md`

### Task 48: WIRE-17 — T4: Docs del scheduler (wired status + promoción del rol)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟡
- **Archivos clave:** `docs/api/VANTA_MEMORY.md` §Operational modules (`:228`), `docs/user/operations/EXPERIMENTAL_FEATURES.md` (filas `:33` vanta-memory, `:38`/`:53`/`:76` vantadb-server), `scripts/docs/check-docs.mjs` + `check-links.mjs`, ADR-0054 T4
- **Verificación real:** ✅ CÓDIGO-REAL — §Operational modules existe (`VANTA_MEMORY.md:228`); las filas a promover están localizadas: `vanta-memory` = "Accelerate (partial — scheduler host pending, WIRE-01)" (`EXPERIMENTAL_FEATURES.md:33`), `vantadb-server` = "Freeze… prospective host… promote that role via the inversion rule if it lands there" (`:38`) + watchlist `:53` + "Frozen except its scheduler-host role (ADR-0054)" (`:76`); los gates `check-docs.mjs`/`check-links.mjs` existen en `scripts/docs/`.
- **Gate Justificación:** ADR-0054 §Scope Budget — la decisión ES el trigger de promoción; sin el row-change documental el wiring queda indocumentado y el Scope Budget miente.
- **Gate Result:** ✅ DO
- **Contrato:** docs del scheduler (wired status + promoción del rol + fila `vanta-memory`) en `VANTA_MEMORY.md` §Operational modules + `EXPERIMENTAL_FEATURES.md`; `check-docs`/`check-links` exit 0; sin prometer más que el rol acotado (no thaw general); dependencia dura: post WIRE-16.
- **Pre-mortem:** (1) ejecutarse antes que el wiring → **va última** (dep WIRE-16); (2) inflar la promoción (thaw general) → solo el rol scheduler-host, como manda ADR-0054; (3) drift de enlaces → gates en el mismo commit.
- **Stop conditions:** —
- **Dependencias:** WIRE-16 (wiring verde) — va última.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟡 | Orden (docs pre-wiring) | Dependencia declarada (post WIRE-16) | cierre |
  | 🟢×🟢 | Promoción inflada | Solo el rol scheduler-host (ADR-0054) | review |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) orden; (2) promoción; (3) gates.
- **Uphill/Downhill:** ⬇️ (2 steps: VANTA_MEMORY → EXPERIMENTAL + gates)
- **DoD:** task = contrato + gates docs exit 0 · commit = `docs:` · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 1d ✓
- **Skills sugeridas:** documentation-skill · documentation-and-adrs
- **Estado:** ✅ COMPLETED

### Task 49: WIRE-18 — T5: Verificación adversarial del scheduler (crash mid-pass + restart)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/tests/` (`e2e_flow.rs` — `run_full_pass:144`, `pipeline_manager.rs`, `conversation_hook.rs`), `vantadb-server/tests/e2e.rs`, `tests/durability_recovery.rs` (precedente crash/recovery), carril vanta-chaos / vanta-review, ADR-0054 T5
- **Verificación real:** ✅ CÓDIGO-REAL — existen precedentes de tests de recuperación (`tests/durability_recovery.rs`, `derived_index_recovery.rs`) y de e2e del hook (`vanta-memory/tests/conversation_hook.rs`, `e2e_flow.rs:144` `run_full_pass`); el objeto a verificar (scheduler WIRE-15/16) aún no existe → la verificación llega con T2/T3; la semántica de cola efímera ya está aceptada (ADR-0054: restart = cola vacía, re-encolable desde capturas persistidas).
- **Gate Justificación:** ADR-0054 T5 — la durabilidad del scheduler es la deuda asumida explícita (cola en RAM); crash mid-pass + restart debe demostrar que no corrompe y que L0 persistido permite reconstruir; review P2-01 por agente distinto cierra el ciclo.
- **Gate Result:** ✅ DO
- **Contrato:** test de crash mid-pass + restart verde (matar a mitad de pass → reiniciar → DB íntegra, sin doble procesamiento corrupto, cola re-encolable) + review P2-01 por agente distinto (vanta-review) con verdicto; hallazgos → FINDs (no fixes silenciosos).
- **Pre-mortem:** (1) crash determinista es difícil en proceso → punto de crash con test hook/feature de test, no timing; (2) "se pierde trabajo" por cola efímera → el contrato ya acepta re-encolado desde L0 persistido: verificar eso, no prometer durabilidad de cola; (3) review del mismo agente que implementó → prohibido (P2-01: agente distinto).
- **Stop conditions:** 5d → entregar crash mid-pass + restart del pass + FIND del crash del loop completo.
- **Dependencias:** WIRE-15/16.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Crash no determinista | Punto de crash con test hook, no timing | diseño |
  | 🟡×🟢 | Expectativa de durabilidad de cola | Contrato: re-encolable desde L0 persistido | review |
  | 🟢×🟡 | Independencia del review | Agente distinto (P2-01) | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) crash determinista; (2) semántica de cola; (3) independencia del review.
- **Uphill/Downhill:** ⬇️ (3 steps: hook de crash → test → review)
- **DoD:** task = contrato + chaos test + review P2-01 · commit = `test(memory):` · release = n/a.
- **Validación Appetite vs Effort:** 1sem ≥ 5d ✓
- **Skills sugeridas:** systematic-debugging · rust-write-tests · doubt-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/WIRE-18.md`

## F3 — Memoria II (profundo)

> **Gate de fase:** ✅ CUMPLIDO — Tasks 50-55 expandidas a nivel F0 el 2026-10-05 (consume specs MGR de P49 + FIND-196); listas para ejecutar.

### Task 50: MEMG-03 — Grafo ↔ memoria (L1–L3 como nodos/aristas)

- **Appetite:** max 2sem
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🔴
- **Archivos clave:** `src/sdk/serialization/mod.rs:77` (`memory_node_id` — XxHash3_128 ns+key→u128) + `:511` (`memory_record_to_node_owned` — proyección record→nodo), `src/sdk/types/record.rs:269` (`node_id` en `MemoryRecord`) + `:215`/`:310` (`derived_from` input/record), `src/sdk/api/memory.rs:352` (cómputo del node_id en `put`) + `:216-236` (validación V1 de `derived_from`), `src/sdk/api/graph.rs:98` (`Embedded::add_edge` — label/weight/ts, bidireccional) + `:141` (`remove_edge`), `src/graph.rs:66` (`GraphTraverser::bfs_traverse`) + `:263` (`topological_sort`), `src/edge_index.rs:29`, `vanta-memory/src/core/record/lifecycle.rs:56,96` (`ContradictionProvenance`/`mark_contradiction`), `vanta-memory/src/core/scene/scene_index.rs:40` (L2 — `scene/<session>`) y `vanta-memory/src/core/persona/persona_generator.rs:108` (L3 — `persona/<session>`)
- **Verificación real:** ✅ CÓDIGO-REAL — el mapeo YA existe: `memory_node_id` (serialization/mod.rs:77, `pub(crate)`) + `MemoryRecord.node_id` (record.rs:269) computado en `put` (memory.rs:352); los records SON nodos (proyección `memory_record_to_node_owned` :511). Grafo: `add_edge`/`remove_edge`/`insert_node`/`get_node` (graph.rs:98,141) + `GraphTraverser` BFS/DFS/topo (graph.rs:66-300) + `edge_index` (edge_index.rs:29). L1–L3 almacenados como records: L1 (memory), L2 escenas `scene/<session>` (scene_index.rs:40), L3 persona `persona/<session>` (persona_generator.rs:108). Gaps re-verificados HEAD: `vanta-memory/src` NO usa `add_edge`/`insert_node`/`graph_bfs` (0 hits; solo serializa `node_id` en seed/offload); `supersede` NO crea arista (0 hits `edge` en memory.rs); MCP solo expone `remove_edge` (tools.rs:636) — no `add_edge`. Riesgo de mecanismo verificado: `memory_record_to_node_owned` construye `UnifiedNode::new(record.node_id)` SIN copiar edges (:511+) → un `put` sobre un nodo con aristas probablemente las pierde (confirmar con test en DISCOVERY).
- **Gate Justificación:** "hoy memoria y grafo viven separados (se busca, no se razona); es el sustrato de la memoria de proyecto/ingeniería" (Backlog MEMG-03; Notion track PI; H-006) — el mapeo determinista y el motor de grafo ya existen → el delta es integración de aristas, no infraestructura nueva; DX-04 (puente ns+key↔nodo) cierra con esto.
- **Gate Result:** ✅ DO
- **Contrato:** L1–L3 como nodos/aristas reales: (a) aristas de linaje entre registros creadas en las ops correspondientes (supersede/derived_from/contradicción) con labels canónicos fijados en DISCOVERY; (b) query "¿quién cambió la fuente de esta decisión y por qué?" respondible vía BFS + provenance con test dedicado; (c) sin regresión de recall (suite `vanta-memory` verde); (d) decisión verificada sobre edges en updates de record (preservar o crear re-idempotente) + DX-04 documentado.
- **Pre-mortem:** (1) `put` reconstruye el nodo y pierde aristas (mecanismo en serialization/mod.rs:511) → confirmar con test y decidir preservación en update vs aristas re-idempotentes por op; (2) doble fuente de verdad (campos `derived_from`/`superseded_by` vs arista) → regla declarada: campo = dato canónico, arista = navegabilidad derivada en la op; (3) scope creep a entity resolution/GraphRAG → solo linaje L1–L3; MGR-05 queda fuera.
- **Stop conditions:** 2sem sin contrato → entregar (a) aristas de linaje (supersede + derived_from) + query de linaje con test + (b) FIND del resto (escenas/persona como nodos enlazados).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | `put` reconstruye el nodo y pierde aristas | Confirmar con test en DISCOVERY; preservar edges o aristas re-idempotentes | DISCOVERY |
  | 🟡×🟡 | Doble fuente de verdad (campos vs aristas) | Regla declarada: campo = dato, arista = navegabilidad | diseño |
  | 🟢×🟡 | Scope creep a entity resolution/GraphRAG | Solo linaje L1–L3; MGR-05 fuera | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) edges perdidas en update; (2) doble fuente de verdad; (3) scope.
- **Uphill/Downhill:** ⬆️ 1 incógnita (¿sobreviven las aristas al `put`? → mecanismo de preservación) → DISCOVERY → luego ⬇️ (aristas → query → tests)
- **DoD:** task = contrato + test de query · commit = `feat(memory):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 2sem ≥ 1-2sem ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · api-and-interface-design · campaign-executor
- **Estado:** ✅ COMPLETED

### Task 51: MEMG-06 — Spill a disco con recall

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/context_engine/engine.rs:68` (`assemble`) + `:199` (`assemble_with_recall`) + `:312-320` (`stub_message` — `[compacted N chars]`) + `:384` (loop de stubs), `vanta-memory/src/context_engine/report_store.rs` (MEM-64 — reportes append-only, sin payload), `vanta-memory/src/offload/` (`storage.rs:17` `OffloadStorage` → records `offload/<session>`, `state_manager.rs`, `hooks/after_tool_call.rs`, `reclaimer.rs` — GC cursor-safe), `vanta-memory/src/services/pipeline_worker.rs:743` (caller real de `assemble_with_recall`), `skills/vantadb-mcp/assets/hooks/TOKEN-BUDGET.md` (política spill de hooks ~2.5k en Codex)
- **Verificación real:** ✅ CÓDIGO-REAL — `spill` en `vanta-memory/src` = 0 hits (repo-wide = 1 comentario: `src/index/diskann.rs:17` "spill nodes to disk… future work"); la compactación reemplaza contenido por stubs `[compacted N chars]` (engine.rs:312-320) y el contenido original NO se persiste — solo el `CompactionReport` (report_store.rs, append-only, sin payload); `offload/` existe (persiste entries vía SDK + cursor + reclaimer) pero `recall` = 0 hits en `offload/` y `after_tool_call` no tiene callers fuera de su módulo → hoy no hay recuperación de lo compactado/spilled.
- **Gate Justificación:** "presupuesto de contexto con spill a disco y recuperación (recall de lo spilled)" (Backlog MEMG-06; Master #11; research §7.5) — la compactación baja el presupuesto perdiendo información; el recall de lo spilled convierte la compresión en reversible.
- **Gate Result:** ✅ DO
- **Contrato:** spill a disco del contenido compactado (persistir payload completo antes del stub, reutilizando el patrón `offload/storage`) + recuperación (recall de lo spilled por id/sesión) con test round-trip (spill → recall devuelve contenido íntegro; sin pérdida de datos); enganchado al path real (`assemble_with_recall`/worker) o hook documentado; opt-in sin cambio de semántica para quien no lo configura; GC reutilizado (reclaimer/cursor).
- **Pre-mortem:** (1) spill duplica storage sin GC → reutilizar `offload/reclaimer.rs` (cursor-safe) + política declarada; (2) recall de spilled compite con el recall L1 → op separada/explícita (no mezclar en el path de recall de memoria); (3) `after_tool_call` sin callers → el wiring es parte del scope o stop con FIND.
- **Stop conditions:** 3-5d → entregar spill+recall por sesión como op explícita + test + FIND del wiring automático en el worker.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Spill duplica storage sin GC | Reutilizar `offload/reclaimer.rs` (cursor-safe) | diseño |
  | 🟡×🟠 | Recall de spilled compite con recall L1 | Op separada/explícita; no mezclar paths | VERIFY |
  | 🟢×🟡 | Hook sin callers (wiring) | Wiring en scope o stop con FIND | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) GC/duplicación; (2) solape con recall L1; (3) wiring sin caller.
- **Uphill/Downhill:** ⬆️ 1 incógnita (punto de enganche: `assemble` vs hook `after_tool_call`) → DISCOVERY → luego ⬇️ (persistir → recuperar → test)
- **DoD:** task = contrato + test round-trip · commit = `feat(memory):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 1sem ≥ 5d ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-06.md`

### Task 52: MEMG-08 — MGR-25: Formatos e ingestores (trait `Ingestor`)

- **Appetite:** max 2sem
- **Esfuerzo:** 🟠 1-2sem
- **Prioridad:** 🟠
- **Archivos clave:** `src/wiki/sources.rs:18` (`SOURCE_CHAR_BUDGET` = 28_000) + `:32` (`scan_local_sources`) + `:82` (skip non-`.md`), `src/sdk/types/record.rs:178` (`MemoryInput` — `payload: String` + `metadata`) + `:215` (`derived_from`), `vanta-memory/src/seed/md_import.rs` (round-trip `.md` con frontmatter), `vantadb-mcp/src/wiki.rs:182` (`handle_wiki_tool` — `wiki_ingest`), Backlog fila MGR-25 (spec completa en Notion — no en repo → [a verificar en DISCOVERY])
- **Verificación real:** ✅ CÓDIGO-REAL — `sources.rs:82` acepta solo `.md` (`path.extension() != Some("md")` → skip con trace log); `SourceFile{rel_path, content}` con presupuesto 28k (:18); `MemoryInput` (record.rs:178) = `payload: String` + `metadata` (sin extractor de formatos); `wiki_ingest` existe (wiki.rs:182); `md_import.rs` = round-trip del formato `.md` (frontmatter v1/v2). La spec MGR-25 (trait + 5 formatos + proveniencia `metadata.source`) está en Notion, no en repo → [a verificar en DISCOVERY] antes de fijar la forma del trait.
- **Gate Justificación:** "el path wiki solo acepta `.md` y `put` recibe `payload: String` sin extractor — no se pueden ingerir documentos reales" (Backlog MEMG-08) — habilita la ingesta de documentos reales (transversal; proveniencia AM7).
- **Gate Result:** ✅ DO
- **Contrato:** trait `Ingestor` (formato → chunks `MemoryInput` con `metadata.source={file,page,chunk}` obligatoria) + mínimo viable ≥2 formatos end-to-end (txt/json/csv) con tests + orden declarado txt/json/csv → html → pdf → docx (por costo/beneficio); respeta `SOURCE_CHAR_BUDGET` (28k) + chunking con overlap; código (rs/py/ts) NO entra (MGR-22); docs.
- **Pre-mortem:** (1) pdf/docx = parsers pesados/deps nuevas → orden por costo, stop en el mínimo, deps solo si la spec las sanciona; (2) proveniencia inconsistente con MGR-12 → `metadata.source` como la spec manda; (3) budget 28k corto para documentos → chunking con overlap + truncación declarada.
- **Stop conditions:** 2sem sin los 5 formatos → entregar trait + txt/json/csv end-to-end + FIND de html/pdf/docx.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Parsers pesados (pdf/docx) → deps nuevas | Orden por costo; mínimo txt/json/csv; deps si la spec las sanciona | DISCOVERY |
  | 🟡×🟢 | Proveniencia inconsistente con MGR-12 | `metadata.source` obligatoria (spec) | diseño |
  | 🟢×🟡 | Budget 28k corto para documentos | Chunking con overlap + truncación declarada | VERIFY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) parsers/deps; (2) proveniencia; (3) budget/chunking.
- **Uphill/Downhill:** ⬆️ 1 incógnita (spec MGR-25: forma del trait + validación) → DISCOVERY → luego ⬇️ (trait → formatos → tests)
- **DoD:** task = contrato + tests por formato · commit = `feat(wiki):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 2sem ≥ 1-2sem ✓
- **Skills sugeridas:** source-driven-development · rust-write-tests · api-and-interface-design · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-08.md`

### Task 53: MEMG-09 — Track PI restante (MGR-23/24: grafo decisión→código→test + taxonomía)

- **Appetite:** max 1mes
- **Esfuerzo:** 🔴 2-4sem
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mcp/src/code.rs` (tool defs `:42-165`, `handle_code_tool:184`, dispatch `:191-328`, stub `code_files:326`), `src/graphrag/pipeline.rs:67` (`GraphRagPipeline::search` — seed→expand→retrieve→context), `src/wiki/sources.rs:82`, `skills/vantadb-mcp/assets/hooks/` (4 clientes + tests + `TOKEN-BUDGET.md`), FIND-196 (repo-map/watcher incremental — sin research/spec), Backlog MGR-23/24 (specs en Notion track PI → [a verificar en DISCOVERY])
- **Verificación real:** ✅ CÓDIGO-REAL — `code_*` tools existen (code_search/code_explore/code_callers/code_callees/code_impact/code_node/code_status; `code_files` = stub "not supported" :326); traversal BFS profundidad-1/parametrizado (`code_impact` max_depth clamp 1-10); `code_index`/`code_watch` = 0 hits (rg); el único watcher del repo es config hot-reload (config.rs:1719); FIND-196 = repo-map/watcher incremental NO implementado (MGR-22 sin research/spec); decisión owner 2026-09-14: watcher Rust+Python+TypeScript desde el inicio (Backlog MEMG-09). Specs MGR-23/24 en Notion → [a verificar en DISCOVERY].
- **Gate Justificación:** "el repo como identidad persistente es el caso de uso natural del grafo+memoria; ventana Kuzu" (Backlog MEMG-09) — cierra la memoria de proyecto/ingeniería (MGR-23 ADRs/convenciones + MGR-24 bug→fix→test) con FIND-196 (repo-map) como input directo.
- **Gate Result:** ✅ DO
- **Contrato:** spec (chunker + watcher Rust/Py/TS + repo-map + API `code_index`/`code_watch`) + primer slice implementado según spec + research-doc; la spec incorpora los detalles de industria validados (presupuestos de carga ~25KB clase Claude Code, ranking dependiente de tarea — Aider, hooks de verificación — OpenHands); taxonomía de lo memorable declarada (mapa, convenciones, ADRs, historia, dependencias, deuda, patrones de fallo); grafo decisión→código→test (linker issue↔commit↔test) como diseño verificable.
- **Pre-mortem:** (1) scope 2-4sem (2 specs + slice) → spec primero, slice mínimo (chunker por símbolo + `code_index` para 1 lenguaje) + FIND del resto; (2) incremental/watcher sin enfoque (FIND-120 R1: Merkle/content-hash vs full scan) → decidir en spec; (3) dep FIND-196 sin research → la spec MGR-22 nace en este task o se cita como sub-entrega.
- **Stop conditions:** 1mes sin slice → entregar research-doc/spec + slice mínimo (chunker por símbolo + `code_index` 1 lenguaje) + FIND.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Scope 2-4sem (2 specs + impl) | Spec primero + slice mínimo + FIND del resto | diseño |
  | 🟡×🟡 | Watcher/incremental sin enfoque (FIND-120/196) | Decidir en spec (Merkle/content-hash vs scan) | spec |
  | 🟢×🟡 | Dep FIND-196 sin research | Spec MGR-22 nace aquí o sub-entrega citada | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) scope; (2) incremental/watcher; (3) dep sin spec.
- **Uphill/Downhill:** ⬆️ 1 incógnita (corte del primer slice: qué entra) → DISCOVERY/spec → luego ⬇️ (chunker → index → watcher)
- **DoD:** task = spec + slice + research-doc · commit = `feat(mcp):`/`docs:` · release = changelog.
- **Validación Appetite vs Effort:** 1mes ≥ 2-4sem ✓
- **Skills sugeridas:** spec-driven-development · source-driven-development · doubt-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED

### Task 54: MEMG-10 — MGR-04: Policy engine (trusted/tainted + RBAC por acción)

- **Appetite:** max 1mes
- **Esfuerzo:** 🔴 2-3sem
- **Prioridad:** 🟠
- **Archivos clave:** `src/rbac.rs` (`Permission`/`AccessMode`/`Rbac::can_access_namespace:70`; `pub(crate)` + `#[allow(dead_code)]`), `src/server/middleware.rs:200,208` (namespace vs permiso global), `src/server/jwt.rs`, `src/audit.rs` (`audit_auth`), `tests/rbac_namespace.rs` (SRV-05), `docs/dev/tasks/VER-04.md` (base budget/ACL/audit), `docs/dev/research/mgr-13-cuarentena.md` (§4.1/§8 — MGR-04 como deuda declarada), Notion 6 áreas §Gobernanza (spec → [a verificar en DISCOVERY])
- **Verificación real:** ✅ CÓDIGO-REAL — la base existe: RBAC role→permissions (`rbac.rs`: Read/Write/Delete/Admin + NamespaceRead/Write; `can_access_namespace:70`, `has_permission:57`), uso en middleware (:200,:208), JWT (`jwt.rs`), test `tests/rbac_namespace.rs` (SRV-05) y audit de auth (`audit_auth`). `tainted` = 0 hits en `src` (solo `trusted_proxies`, otro dominio) → el modelo trusted/tainted NO existe; MGR-04 quedó fuera del plan F0–F6 (clase mínima vía SCH-05/VER-04; mgr-13 §8 la lista como deuda v1.0). Spec MGR-04 en Notion (6 áreas §Gobernanza) → [a verificar en DISCOVERY].
- **Gate Justificación:** gobernanza de memoria: cuarentena (SCH-05) y ACL por namespace (VER-04/SRV-05) son la clase mínima; falta el modelo de confianza por namespace (trusted/tainted) + RBAC por acción para cerrar el residual de mgr-13 §4.1 ("la autenticación protege quién escribe, no qué se escribe; no hay clase de confianza").
- **Gate Result:** ✅ DO
- **Contrato:** spec (namespaces trusted/tainted + RBAC por acción + integración con retrieval/inyección) + implementación sobre la base VER-04: clase de confianza por namespace aplicada en retrieval/inyección (tainted no inyecta por defecto) + RBAC por acción con tests por acción + audit; sin romper ACL/namespace actuales (suite verde); modelo documentado (ADR si cambia semántica de auth).
- **Pre-mortem:** (1) scope spec+impl en 2-3sem → spec primero con corte declarado; (2) solape SCH-05 (registro) vs trusted/tainted (namespace) → semántica complementaria declarada (registro vs namespace), no duplicar gates; (3) cambio de semántica auth rompe clientes → aditivo/opt-in por config.
- **Stop conditions:** 1mes sin contrato → entregar spec + clase mínima (clasificación trusted/tainted + gate de inyección por namespace) + FIND del RBAC por acción restante.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Scope spec + implementación | Spec primero con corte declarado | diseño |
  | 🟡×🟠 | Solape SCH-05 (registro) vs trusted/tainted (namespace) | Semántica complementaria declarada; no duplicar gates | spec |
  | 🟢×🟠 | Cambio de semántica auth rompe clientes | Aditivo/opt-in por config | VERIFY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) scope; (2) solape SCH-05; (3) compat auth.
- **Uphill/Downhill:** ⬆️ 1 incógnita (spec MGR-04: modelo exacto trusted/tainted + acciones) → DISCOVERY → luego ⬇️ (spec → clasificación → RBAC por acción → tests)
- **DoD:** task = spec + tests por acción · commit = `feat(server):` · release = changelog.
- **Validación Appetite vs Effort:** 1mes ≥ 2-3sem ✓
- **Skills sugeridas:** security-and-hardening · spec-driven-development · source-driven-development · campaign-executor
- **Estado:** ✅ COMPLETED · **Task file:** `docs/dev/tasks/MEMG-10.md`

### Task 55: MEMG-17 — Rollback + verificabilidad + erasure criptográfica

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 3-5d
- **Prioridad:** 🟠
- **Archivos clave:** versions (MEMG-13: `vanta-memory/src/core/record/l1_reader.rs:134,150,196` — `read_record_versions`/`read_record_version`/`diff_records`; core `src/sdk/api/memory.rs:935,953` `get_version`/`versions`), snapshot (MEMG-13: `vanta-memory/src/utils/backup.rs:38,43,55`; core `src/sdk/builder.rs:269,275,297` `create_snapshot`/`list_snapshots`/`restore_from`), hash-chain VER-01 (`src/cli.rs:198-201` `Verify` WAL; `src/attestation.rs:66` `ChainEvidence` + `:148` `chain_evidence`), certificados (`src/attestation.rs:90` `PurgeCertificate` + `:118` verificación; `src/sdk/api/memory.rs:1050,1080` `delete_with_attestation`/`verify_purge_certificate`), tombstones (`src/backend.rs:37-67` particiones `TombstoneStorage`/`Tombstones`), FIND-194 (rotación master key), FIND-287 (restore data-only)
- **Verificación real:** ✅ CÓDIGO-REAL — versions + snapshot existen (MEMG-13 ✅: l1_reader :134-196; backup.rs :38-55; core builder.rs :269-297); hash-chain VER-01 existe (CLI `Verify` :198-201; attestation cita la cadena :66,:148); certificados de delete VER-02 existen (attestation.rs:90/:118; memory.rs:1050/:1080) con límites declarados ("no secure erase", "no parametric unlearning" — attestation.rs `declared_limits`); tombstones existen (backend.rs particiones). Gaps re-verificados HEAD: `rollback`/`erasure`/`DEK` = 0 hits en el path de memoria (rg) → las 3 capacidades NO existen; FIND-287: restore de snapshot es data-only (tombstones de delete y supersession sobreviven — canary MEMG-13); FIND-194: rotación de master key invalida envelopes (re-encrypt lazy no implementado).
- **Gate Justificación:** "VMG lista Rollbackability + Verified Forgetting como primitivas de primera clase; EDPS exige proof of unlearning; hoy hay versions (MEMG-13) y hash-chain (VER-01) pero no rollback/recibos/erasure como capacidades" (Backlog MEMG-17; validación externa 2026-09-30: VMG arXiv 2604.16548, ChronoMem arXiv 2607.27773, W3C CG + GDPR Art.17) — cierra el ciclo de confianza de la memoria.
- **Gate Result:** ✅ DO
- **Contrato:** (a) rollback semántico a versión/snapshot con linaje (reusa versions/snapshot; declara explícitamente qué revierte y qué no — caveat FIND-287 data-only); (b) recibos verificables extendiendo hash-chain VER-01 con el contrato de verificación de los certificados existentes (firma ML-DSA-65 solo si la spec lo sanciona → [a verificar]); (c) erasure criptográfica (destrucción DEK + tombstone; scope declarado) — cada pieza con test propio; sin romper certificados/verify existentes (suites verdes).
- **Pre-mortem:** (1) DEK no existe (0 hits) → verificar modelo cripto en DISCOVERY; si no hay capa por registro/namespace, diseñar en el slice o acotar a envelope/VER-03 + FIND; (2) rollback data-only (FIND-287) → semántica exacta declarada en contrato/tests (no asumir); (3) scope triple en 3-5d → orden rollback → recibos → erasure; stop a la primera pieza incompleta.
- **Stop conditions:** 5d sin las 3 piezas → entregar (a) rollback semántico con linaje + test sobre versions/snapshot + FIND de recibos/erasure.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | DEK no existe (0 hits) | Verificar modelo cripto en DISCOVERY; acotar o diseñar en slice | DISCOVERY |
  | 🟡×🟡 | Rollback data-only (FIND-287) | Semántica exacta declarada en contrato/tests | diseño |
  | 🟢×🟠 | Scope triple (3 piezas) | Orden rollback → recibos → erasure; stop | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) DEK inexistente; (2) semántica rollback (FIND-287); (3) scope triple.
- **Uphill/Downhill:** ⬆️ 1 incógnita (modelo cripto para erasure: ¿DEK por registro/namespace?) → DISCOVERY → luego ⬇️ (rollback → recibos → erasure)
- **DoD:** task = contrato + tests por pieza · commit = `feat(memory):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 1sem ≥ 3-5d ✓
- **Skills sugeridas:** security-and-hardening · source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ✅ COMPLETED

## F4 — Sharing & Multi-tenant (caso "equipo" + enterprise)

> **Gate de fase:** ✅ CUMPLIDO — Tasks 56-59 expandidas a nivel F0 el 2026-10-05; listas para ejecutar.

### Task 56: MEMG-04 — Multi-tenant (enforcement + cuotas)

- **Appetite:** max 1mes
- **Esfuerzo:** 🔴 2-3sem
- **Prioridad:** 🔴
- **Archivos clave:** `src/sdk/serialization/mod.rs` (`memory_node_id:77`, `validate_namespace:108` — namespace = partición de keyspace), `src/rbac.rs` (`can_access_namespace:70`; `pub(crate)`), `src/server/middleware.rs:186-208` (enforcement RBAC namespace-scoped en records/search/list), `src/server/router.rs:134-137` (roles admin/reader/writer hardcoded), `src/config.rs:203-206` (`RbacCfg` token→role), `src/entity/mod.rs:10` + `src/entity/checker.rs:89,96` (entidades user/team con namespace — base reutilizable), límites globales (no-tenant): `src/server/bootstrap.rs:99` · `src/server/router.rs:266-284` (rate limit), `src/server/state.rs:191-222` (`AuthRateLimiter`), `src/governor.rs` + `src/memory_governor.rs` (watermarks globales), `vanta-memory/src/core/abstractions/types.rs:106-114` (`team_id`/`agent_id` por registro), `vanta-memory/src/utils/erasure.rs:6` (scope "agent or tenant", DEK por scope — MEMG-17), FIND-301 (`docs/dev/Backlog.md:449` — enforcement RBAC por acción restante)
- **Verificación real:** ✅ CÓDIGO-REAL — `rg tenant` = solo comentarios/citas: `src/entity/mod.rs:10` ("namespace (deployment/tenant)"), `src/entity/scene.rs:8`, `src/node/bitset.rs:5,21` (claim "multi-tenant filtering"), `vanta-memory/src/core/record/l1_writer.rs:108-111` (stamp "default tenancy"), `erasure.rs:6`. Enforcement por tenant = NO existe: `namespace` es partición de keyspace (`memory_node_id` :77) sin barrera de confianza; RBAC ns-scoped (`rbac.rs:70`; middleware `:189-208`) autoriza dentro de un único trust domain (roles hardcoded `router.rs:134-137`; config token→role `RbacCfg:203`). Cuotas por tenant = 0: solo límites globales (rate limit `bootstrap.rs:99`; `AuthRateLimiter` `state.rs:191`; watermarks `memory_governor.rs`). Billing = 0 hits (única mención `vanta-proxy/src/cost.rs:87` "not billing"). Base reutilizable: entidades team/user + `PermissionChecker` (`checker.rs:89,96`) y stamps de vanta-memory (`types.rs:108,114`).
- **Gate Justificación:** en cuanto haya 2 clientes/agentes es problema de datos cruzados; sin enforcement + cuotas no hay enterprise (Backlog MEMG-04). FIND-301 dejó pendiente el enforcement RBAC por acción — el diseño de tenant debe declarar la frontera: aislamiento (MEMG-04) vs autorización (RBAC/FIND-301).
- **Gate Result:** ✅ DO
- **Contrato:** (a) enforcement real por tenant: una credencial/tenant no puede leer/escribir/buscar/listar/exportar datos de otro tenant (barrera en la capa declarada en DISCOVERY: storage y/o API), con test de no-cruce entre 2 tenants por cada superficie habilitada; (b) al menos una cuota por tenant aplicada y testeada (p.ej. bytes o records por namespace/tenant con error explícito y audit) — o decisión registrada + FIND si el punto de intercepción no cabe en el appetite; (c) billing boundary documentado (metrado por tenant exportable o decisión "no billing en motor" registrada); modelo de tenant documentado (¿namespace? ¿entidad `tenant`?) con ADR si cambia semántica de autorización; sin romper RBAC/namespace/flujos actuales (suite verde).
- **Pre-mortem:** (1) "tenant" no está modelado (0 entidad) → decidir en DISCOVERY namespace-vs-entidad nueva + migración de keyspaces existentes; (2) solape con FIND-301/MEMG-10 (RBAC por acción) → frontera declarada: MEMG-04 = barrera de aislamiento + cuota; RBAC sigue siendo el quién-puede; (3) cuotas sin punto de intercepción barato (no hay contador por namespace) → interceptar en `put`/`delete_inner` + contador por namespace; medir coste [a verificar en DISCOVERY]; (4) triple scope (enforcement + cuotas + billing-boundary) en 2-3sem → orden enforcement → test no-cruce → cuota → boundary.
- **Stop conditions:** 2-3sem sin contrato → entregar enforcement mínimo (barrera por tenant + test de no-cruce) + FIND de cuotas/billing restante.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Modelo tenant inexistente (¿namespace = tenant?) | Decisión en DISCOVERY + ADR; migración declarada | DISCOVERY |
  | 🟡×🟡 | Solape con FIND-301 / MGR-04 | Frontera: aislamiento (MEMG-04) vs autorización (RBAC) | diseño |
  | 🟢×🟠 | Cuotas sin punto de intercepción barato | Interceptar en put/delete + contador; medir coste | diseño |
  | 🟢×🟡 | Billing real fuera de alcance del motor | Boundary documentado o decisión registrada | cierre |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) modelo tenant; (2) solape RBAC/FIND-301; (3) enforcement de cuotas.
- **Uphill/Downhill:** ⬆️ 1 incógnita (¿qué es un tenant: namespace vs entidad?) → DISCOVERY → luego ⬇️ (enforcement → test no-cruce → cuota → boundary)
- **DoD:** task = contrato + test de no-cruce · commit = `feat(core):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 1mes ≥ 2-3sem ✓
- **Skills sugeridas:** security-and-hardening · source-driven-development · doubt-driven-development · campaign-executor
- **Estado:** ⏳ EN PROGRESO · **Task file:** `docs/dev/tasks/MEMG-04.md`

### Task 57: MEMG-05 — Multi-escritor (CRDT / vector-clock / LWW declarado)

- **Appetite:** max 1mes
- **Esfuerzo:** 🔴 2-3sem
- **Prioridad:** 🟠
- **Archivos clave:** write path (`src/sdk/api/memory.rs:761` `put` → `:1253` `put_record_exact` → WAL), `src/wal.rs` (framing v3 encadenado `:19-21`; `chain_hash:90`; `append:399-421` — 1-writer), `src/wal_shipping.rs:1-32` (shipping unidireccional a réplica; feature `wal-shipping`, `src/lib.rs:34`), `docs/dev/strategy/VantaDB-Analisis-Arquitectura-Producto-Competencia.md:412` (posición: "merge determinista LWW/authority, CRDT-lite sobre UpdateOperations"), `docs/api/MEMORY_INTERCHANGE_FORMAT.md:64` + `docs/api/TS_SDK.md:195` (LWW declarado en import/OPFS sin merge), `docs/dev/wasm/CRASH_MODEL.md:40` (2 pestañas = corrupción silenciosa; mitigación WSM-15 = Task 9 F0), `vanta-memory/src/core/profile/profile_sync.rs:25-47` (scope `team:{t}|agent:{a}`), `docs/dev/research/mgr-22-repo-map.md:72` (Merkle/content-hash reevaluado con MEMG-05)
- **Verificación real:** ✅ CÓDIGO-REAL — `CRDT`/`vector_clock`/`LWW` = 0 hits en código Rust (verificado 2026-10-05): `CRDT` solo aparece en glosario (`docs/user/glosario/crdt.md`) y como ambición de producto (`strategy:412`); el comportamiento last-write-wins solo existe como dedup por ID de storage (`tests/property_durability.rs:45,110`) y evolución de campos (`src/shred/mod.rs:36`), nunca como política de conflicto multi-escritor declarada. Lo que SÍ existe: WAL encadenado 1-writer (`wal.rs:399-421`) con shipping primario→réplica vía HTTP sin merge (`wal_shipping.rs`) — replicación, no multi-escritor; OPFS documenta "last write wins with no merge" (`CRASH_MODEL.md:40`; `TS_SDK.md:195`). vanta-memory sincroniza persona por scope (`profile_sync.rs`), no réplicas. Escenario + estrategia de merge NO existen → contrato abierto [a verificar en DISCOVERY: multi-device embedded vs multi-proceso vs agentes].
- **Gate Justificación:** multi-device/multi-agente es la posición única "memoria federada local-first" (`strategy:412`; Desktop+WASM+embedded); hoy escrituras paralelas pierden datos silenciosamente (Backlog MEMG-05) y no hay política declarada de conflicto.
- **Gate Result:** ✅ DO
- **Contrato:** una estrategia de resolución de conflictos multi-escritor implementada y declarada (CRDT / vector-clock / LWW explícito — elección documentada con ADR) para el escenario fijado en DISCOVERY (¿multi-device vía event-log/shipping? ¿multi-proceso?) + test de escrituras paralelas concurrentes sin pérdida silenciosa (resultado determinista: merge o winner declarado, nunca "el que escribe gana") + semántica documentada (qué gana, por qué y límites) + `wal-shipping`/write path actuales intactos (suite + failpoints verdes).
- **Pre-mortem:** (1) escenario abierto (device/proceso/agente) → ADR de escenario en DISCOVERY, un solo escenario en v1; (2) CRDT completo excede 2-3sem → mínimo viable = LWW explícito + detección de conflicto (upgrade path CRDT declarado); (3) tocar el write path/WAL puede degradar durabilidad → cambios aditivos, chaos/failpoints verdes; (4) solape WSM-15 (Task 9: OPFS lock = exclusión) → frontera: WSM-15 exclusión, MEMG-05 resolución/merge.
- **Stop conditions:** 2-3sem sin contrato → entregar estrategia declarada + implementación mínima en el escenario único fijado + FIND del resto (réplicas/CRDT).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Escenario multi-escritor no fijado | ADR de escenario en DISCOVERY; 1 escenario v1 | DISCOVERY |
  | 🟡×🟠 | CRDT completo fuera de appetite | Mínimo LWW declarado + detección; upgrade declarado | diseño |
  | 🟡×🟡 | Regresión del write path (WAL) | Aditivo + chaos/failpoints verdes | VERIFY |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) escenario; (2) coste de la estrategia; (3) write path.
- **Uphill/Downhill:** ⬆️ 2 incógnitas (escenario + estrategia) → DISCOVERY → luego ⬇️ (modelo de conflicto → write path → test concurrencia)
- **DoD:** task = contrato + test de concurrencia · commit = `feat(sync):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 1mes ≥ 2-3sem ✓
- **Skills sugeridas:** source-driven-development · doubt-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⏳ EN PROGRESO · **Task file:** `docs/dev/tasks/MEMG-05.md`

### Task 58: MEMG-16 — Compartir/colaboración multi-agente (scopes + permisos + revocación)

- **Appetite:** max 1mes
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🟠
- **Archivos clave:** `src/entity/checker.rs` (`Action::{Read,Write,Assign,Share,Use}:37`, `Visibility::{Private,Team,Restricted,Agent,Task}:48`, `TeamRole:59`, `can_access_asset:122` — cadena allow-only de 7 pasos, `membership:213`, `acl_allows:229`), `src/entity/mod.rs` (collections `user`/`team`/`team_member`/`asset`/`acl`; claves `entity:{ns}:{collection}::{id}`), `src/entity/checker_tests.rs` (`removed_member_denied:134`, `restricted_acl_deny_effect_wins:232`, `acl_agent_subject_requires_agent_id:340`), `src/server/state.rs:297` + `src/server/middleware.rs:179` (comentario: principal → PermissionChecker — sin uso real), `vanta-memory/src/core/profile/profile_sync.rs:25-47` (scope `team:{t}|agent:{a}`), `vanta-memory/src/core/abstractions/types.rs:106-114` (team_id/agent_id por registro), `vanta-memory/src/core/hooks/auto_recall.rs:653-657` (filtro de recall Agent/Team — D22), `vanta-memory/src/utils/erasure.rs` (scope = label; DEK por scope, MEMG-17), `docs/dev/Backlog.md:144` (sharing contracts: temporary/permanent/syndicate + revocation; Dep: MEMG-04/05 + EXE-07), `docs/dev/Backlog.md:851` (EXE-07 P50: cascada multiagente; refs `tests/storage/chaos_integrity.rs`, `vanta-memory/src/core/dream/`)
- **Verificación real:** ✅ CÓDIGO-REAL — la maquinaria de permisos existe COMPLETA y testeada: `PermissionChecker` allow-only 7 pasos (resource → owner → membership → visibility → role-default → ACL → deny; `checker.rs:122-210`), acciones incl. `Share`/`Use`, visibilidad `Agent`, sujetos ACL user/team_role/agent (`checker_tests.rs:340-366`), ACL deny gana (`:232`), revocación de membresía vía `status != active` (`checker.rs:213`; test `:134`). PERO no está cableada a ninguna superficie de producto: `rg PermissionChecker` = solo `checker.rs` + `checker_tests.rs` (+ 2 comentarios en `middleware.rs:179`, `state.rs:297`) → sin ruta HTTP/MCP/SDK. "sharing contracts" (temporary/permanent/syndicate) = 0 hits en código (`syndicate` solo en Backlog/plan). Scopes de vanta-memory = etiquetas de aislamiento (team/agent), no grants. Revocación de acceso ya concedido (propagación/efecto sobre recall) = no existe [a verificar en DISCOVERY].
- **Gate Justificación:** "Share & Propagate" es fase propia del lifecycle 2026; Letta/Cognee/CrewAI ya lo ofrecen; MAST (NeurIPS 2025) documenta fallos inter-agent; sin esto no hay caso "equipo" (Backlog MEMG-16). Costo marginal bajo: checker + ACL + entidades ya existen — falta producto (superficie + scopes + revocación + doc).
- **Gate Result:** ✅ DO
- **Contrato:** memoria compartida con scopes (org/team/proyecto → mapeo declarado a namespace/entidades) + grants y revocación efectiva sobre las operaciones declaradas (read/write/use/recall entre agentes y usuarios): (a) un grant habilita acceso compartido entre 2 agentes/usuarios y (b) tras revocar, el acceso deja de permitirse en el siguiente acceso (test); (c) modelo de datos compartidos documentado (aislamiento, propagación/contagio, revocación, qué pasa con lo ya recordado — frontera con erasure MEMG-17 y cuarentena SCH-05/MEMG-10); reusa `PermissionChecker`/ACL y scopes existentes (no duplicar); containment ante error de un agente declarado (enlace o límite explícito con EXE-07); suites verdes.
- **Pre-mortem:** (1) checker sin cablear → decidir superficie en DISCOVERY (SDK memory API vs server vs MCP) y reutilizar, no re-implementar; (2) "revocación" ambigua: ¿solo acceso futuro o purga de lo compartido? → declarar semántica en contrato + frontera con MEMG-17; (3) propagación/contagio (MAST) → scope explícito de qué se propaga y contención mínima; (4) EXE-07 (P50) pendiente → no bloquear: containment mínimo propio o dependencia declarada.
- **Stop conditions:** 1-2sem sin contrato → entregar scopes + grants + revoke mínimo sobre el checker existente + doc del modelo + FIND (propagación/EXE-07).
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Checker no cableado; superficie sin decidir | Decisión de superficie en DISCOVERY; reuso, no duplicación | DISCOVERY |
  | 🟡×🟠 | Semántica de revocación (acceso vs purga) | Declarar en contrato; frontera con MEMG-17 | diseño |
  | 🟡×🟡 | Propagación/contagio y EXE-07 pendiente | Containment mínimo propio o dependencia declarada | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) superficie de wiring; (2) semántica de revocación; (3) propagación/EXE-07.
- **Uphill/Downhill:** ⬆️ 1-2 incógnitas (superficie + semántica de revocación) → DISCOVERY → luego ⬇️ (scopes → grants → revocación → tests)
- **DoD:** task = contrato + tests share/revoke · commit = `feat(memory):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 1mes ≥ 1-2sem ✓
- **Skills sugeridas:** security-and-hardening · source-driven-development · documentation-and-adrs · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-16.md`

### Task 59: VER-10 — Attestation de escritura (extender el certificado de delete a writes)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Archivos clave:** `src/attestation.rs` (`PurgeCertificate:90`, `build_certificate:436`, `verify_certificate:516`, `PURGE_SURFACES:36-46`, `ChainEvidence:66` + `chain_evidence():151`, `declared_limits:136`, integridad sha256:81), `src/sdk/api/memory.rs` (`delete_certified:1201`, `verify_purge_certificate:1235`, `put:761` → `put_record_exact:1253` = write path sin recibo), `src/wal.rs` (framing v3 encadenado `:19-21`; `chain_hash:90`; `append:399-421` — computa `record_hash` pero devuelve `()`), `src/cli.rs:198-201` (`Verify` — cadena WAL) + `:475-478` (`certificate verify`), `docs/api/CERTIFIED_DELETE.md` (superficies CLI/MCP/SDK del delete certificado — status VER-02), `docs/api/WAL_INTEGRITY.md` (esquema y límites de la cadena — VER-01), precedente receipt: `vanta-memory/src/utils/erasure.rs:38-42` (ErasureReceipt embebe `chain_evidence`, MEMG-17), `docs/dev/Backlog.md:195` (VER-10; Dep: VER-01)
- **Verificación real:** ✅ CÓDIGO-REAL — solo delete tiene attestation: `PurgeCertificate` + builder/verifier (`attestation.rs:90,436,516`), expuesto en CLI/MCP/SDK (`memory.rs:1201,1235`; `cli_handlers/crud.rs:572`; `vantadb-mcp/src/handlers/tools.rs:1681`); la superficie de escritura (`put:761`) no tiene recibo ni API equivalente (`rg receipt` en `src` = solo comentarios). La base ya cubre writes en la cadena: WAL v3 encadena `prev_hash‖record_hash` por frame (`wal.rs:19-21,399-421`) y `chain_evidence()` referencia la cadena (:151), pero `append` NO expone la `record_hash` del frame al write path → ligar un recibo a una escritura requiere API aditiva o re-lectura del WAL [a verificar en DISCOVERY: coste/unidad]. Precedente de receipt verificable con schema/integridad/límites: erasure (MEMG-17).
- **Gate Justificación:** hoy solo el borrado es demostrable ("certified delete"); sin recibo de escritura no hay cadena de custodia completa write→delete; solapa con VER-01 (hash-chain ya lista) — coste marginal bajo (Backlog VER-10).
- **Gate Result:** ✅ DO
- **Contrato:** escrituras con attestation verificable extendiendo el chain: al menos el write path declarado (p.ej. `put`) emite un recibo con referencia al frame WAL encadenado (posición/`record_hash` o evidencia equivalente) + verificación que re-chequee (schema → integridad sha256 → evidencia viva, patrón VER-02) + test válido/inválido (recibo editado/corrupto y payload alterado → detección) + límites declarados (misma honestidad que VER-02: qué NO prueba) + doc (`docs/api/CERTIFIED_DELETE.md` extendido o doc nuevo de write receipts); certificados de delete y `vanta-cli verify` intactos; attestation opt-in (sin coste por defecto).
- **Pre-mortem:** (1) `append` no expone `record_hash` → decidir en DISCOVERY API aditiva (devolver hash/posición) vs recibo derivado por re-escaneo del WAL; (2) solape VER-01/VER-02 → reusar `ChainEvidence`/schema/`finalize` existentes, no forkear el schema; (3) writes son hot path → opt-in explícito (no default) y medir coste; (4) unidad ambigua (por operación vs por frame/batch) → declarar en contrato.
- **Stop conditions:** 3d sin contrato → entregar recibo mínimo por `put` + verify + test inválido + FIND de batches/CLI.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | `append` no expone el `record_hash` | API aditiva o recibo por re-escaneo; decidir en DISCOVERY | DISCOVERY |
  | 🟢×🟠 | Coste en hot path de escritura | Opt-in; off por defecto; medir | diseño |
  | 🟢×🟡 | Solape VER-01/VER-02 | Reusar ChainEvidence + patrón receipt | diseño |

- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** (1) exposición del hash del frame; (2) coste hot path; (3) unidad del recibo.
- **Uphill/Downhill:** ⬆️ 1 incógnita (ligar recibo↔frame: API vs re-escaneo) → DISCOVERY → luego ⬇️ (schema → recibo en put → verify → tests)
- **DoD:** task = contrato + tests válido/inválido · commit = `feat(attestation):` · release = changelog (minor).
- **Validación Appetite vs Effort:** 3d ≥ 2-3d ✓
- **Skills sugeridas:** security-and-hardening · source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/VER-10.md`

## F5 — Frontera & estrategia (research, paralelizable con F2-F4)

> **Gate de fase:** compactas — expandir al nivel F0 al iniciar F5. Estas tareas son research/specs/estrategia; no bloquean releases.

### Task 60: MEMG-14 — Marco 2.0: núcleo + extensiones (reformulación del marco 8/6/10/PI)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟡
- **Archivos clave:** docs del marco + sync Notion; 4 research (≈100 fuentes)
- **Verificación real:** ✅ — la validación externa 2026 muestra que la taxonomía no es cerrada (survey de seguridad arXiv 2604.16548; DAMA-DMBOK); mantener "cerrada" es riesgo de credibilidad (Regla 11).
- **Contrato:** marco reformulado ("núcleo + extensiones") con las 7 sub-decisiones del backlog (áreas 7/8, ámbitos, ejes, presentación, sync Notion, fuera-de-alcance, lista "no elevar"); revisión documentada.
- **Skills sugeridas:** documentation-and-adrs · writing-guidelines · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-14.md`

### Task 61: MEMG-15 — Portabilidad/interoperabilidad (AGENTS.md / MCP / IETF + export firmado)

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟠
- **Archivos clave:** formatos de export/import (VER-05/06), drafts W3C/IETF (AAIF, AIMEM, ALF, AMP)
- **Verificación real:** ✅ — drafts activos 2026; Anthropic vende "no lock-in" con import/export; VER-05/06 ✅ habilitan el paso.
- **Contrato:** mapeo/adopción de formatos de frontera + export con checksum/firma + versionado/migración de formatos; spec + slice mínimo.
- **Skills sugeridas:** api-and-interface-design · documentation-and-adrs · source-driven-development · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-15.md`

### Task 62: MEMG-18 — Multimodalidad: decisión + spec (extensión de modalidad vs 9ª dim)

- **Appetite:** max 2d · **Esfuerzo:** 🟡 1-2d · **Prioridad:** 🟡
- **Archivos clave:** marco de dimensiones; decisión owner 2026-09-14 (fuera de alcance v1.0 con triggers)
- **Verificación real:** ✅ — "3 direcciones fuera de alcance v1.0 con triggers: ontología, multimodal (MGR-25+caso), memoria ejecutable".
- **Contrato:** decisión documentada (extensión vs 9ª dim) + spec mínima + trigger de reevaluación.
- **Skills sugeridas:** documentation-and-adrs · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-18.md`

### Task 63: MEMG-19 — Prospectiva + descartes documentados (sensorial/emocional)

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🟢
- **Archivos clave:** docs del repo (los respaldos hoy viven solo en Notion; grep 0 hits)
- **Verificación real:** ✅ — dims hub §Validación + PM-Bench/MemEmo/TriggerBench (2026).
- **Contrato:** (a) prospectiva/intencional como patrón de uso documentado; (b) descartes sensorial/emocional con respaldos + criterio de revisión (6-12m); grep del repo > 0.
- **Skills sugeridas:** documentation-skill · writing-guidelines
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-19.md`

### Task 64: STRAT-04 — WASM lock-free multi-thread (prep Kuzu)

- **Appetite:** max 1mes · **Esfuerzo:** 🔴 2-4sem · **Prioridad:** 🟠
- **Archivos clave:** `vantadb-wasm` (concurrencia), SharedArrayBuffer/atomics
- **Verificación real:** ✅ — "lo que falta para el claim Kuzu-successor" (STRAT-04, Alta 2026-10-01).
- **Contrato:** diseño + primer slice de concurrencia lock-free en WASM multi-thread con benchmark before/after (Regla 9).
- **Skills sugeridas:** doubt-driven-development · source-driven-development · performance-optimization · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/STRAT-04.md`

### Task 65: STRAT-05 — Ruta object storage (S3/blob)

- **Appetite:** max 1sem · **Esfuerzo:** 🔴 1-2sem · **Prioridad:** 🟡
- **Archivos clave:** backend de storage (diseño), `docs/dev/backlog-futuro.md` (referencias)
- **Verificación real:** ✅ — STRAT-05 (Alta 2026-10-01); research/spec primero.
- **Contrato:** research + spec de la ruta object storage (viabilidad, costo, tradeoffs) con decisión documentada.
- **Skills sugeridas:** doubt-driven-development · documentation-and-adrs · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/STRAT-05.md`

### Task 66: STRAT-06 — Research: licencias (Khoj/Jan/Reor/OpenWebUI/Letta) + ACV OSS DBs

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2d · **Prioridad:** 🟢
- **Archivos clave:** research doc nuevo (`docs/dev/research/`)
- **Verificación real:** ✅ — STRAT-06 (Alta 2026-10-01): "los bands de revenue son modelado, no datos".
- **Contrato:** research doc con licencias comparadas + ACV de OSS DBs + implicaciones para el modelo open-core; fuentes citadas.
- **Skills sugeridas:** coordinated-web-search · documentation-and-adrs · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/STRAT-06.md`

### Task 67: BENCH-02 — BEIR/MTEB recall@k vs pgvector/Chroma/sqlite-vec

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟠
- **Archivos clave:** `benchmarks/` (0 hits de BEIR/MTEB), `competitive_bench.py:336-358` (ya computa recall_at_k)
- **Verificación real:** ✅ — "un número de recall@k verificado vale más que 50 tareas de roadmap" (verificado HEAD 2026-10-01).
- **Contrato:** número de recall@k (BEIR/MTEB) vs ≥1 competidor (pgvector o sqlite-vec), reproducible con comando documentado (Regla 11); doc en BENCHMARKS.md.
- **Skills sugeridas:** performance-optimization · documentation-skill · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/BENCH-02.md`

### Task 68: WIRE-12 — IQL: `LIMIT` / `OFFSET`

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟡
- **Archivos clave:** parser/planner IQL (el token `LIMIT` ya está lexado sin regla)
- **Verificación real:** ✅ — "El token `LIMIT` ya está lexado sin regla; cierra fricción de UX (paginación)".
- **Contrato:** `LIMIT`/`OFFSET` funcionan end-to-end (parser→planner→executor) con tests; docs IQL actualizadas.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WIRE-12.md`

### Task 69: WIRE-13 — IQL: agregaciones (`COUNT`/`SUM`/`GROUP BY`)

- **Appetite:** max 1sem · **Esfuerzo:** 🟡 3-5d · **Prioridad:** 🟡
- **Archivos clave:** parser/planner/executor IQL
- **Verificación real:** ✅ — WIRE-13 (Alta 2026-10-01); habilitado por el pipeline de agregación existente.
- **Contrato:** agregaciones básicas (COUNT/SUM/GROUP BY) end-to-end con tests; docs IQL actualizadas.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WIRE-13.md`

### Task 70: SRV-10 — Cifrado en reposo del server (HTTP)

- **Appetite:** max 1sem · **Esfuerzo:** 🟡 3-5d · **Prioridad:** 🟡
- **Archivos clave:** capa server (`src/server/`), feature `encryption` del core (ya existe); `HTTP_API.md:701` lo declara
- **Verificación real:** ✅ — "El core ya tiene feature `encryption`; falta la capa server" (SRV-10, Alta 2026-10-01).
- **Contrato:** server con cifrado en reposo activable (config + key management documentado) con test; docs HTTP_API actualizadas.
- **Skills sugeridas:** security-and-hardening · source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/SRV-10.md`

## F6 — Cierre: publicación de adapters en PyPI (owner-assisted)

> **Gate de fase:** al final del plan (tras F0-F5). El **publish es lane del owner** (environment `pypi` + tag); el agente prepara, verifica (dry-run) y deja el checklist ejecutable. **Push: solo al final** (instrucción owner 2026-10-04) — el tag/publish de este cierre ES parte del push autorizado del final.

### Task 71: MKT-20 — Publicar los 9 adapters en PyPI (owner-assisted)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1-2h (owner) + preparación del agente
- **Prioridad:** 🟠
- **Archivos clave:** `integrations/{langchain,llamaindex,mem0,crewai,dspy,haystack,letta,openai,ollama}/pyproject.toml`, `.github/workflows/release-adapters.yml`, `docs/dev/tasks/MKT-20.md` (existe — refrescar checklist en DISCOVERY)
- **Verificación real:** ✅ CÓDIGO-REAL — re-baseline 2026-09-29: los 9 nombres en PyPI = **404 (libres)**; builds 10/10 PASS (wheel+sdist, twine PASSED); workflow listo (matriz 9, OIDC, `fail-fast:false`, `skip-existing`, tag `adapters-v*`); `vantadb-py` live en PyPI. Todo lo ejecutable en código ya se hizo (local ✅ `3815afa3`).
- **Gate Justificación:** pata de distribución del track frameworks (ICP-03) — "el trabajo no existe para nadie hasta que sale"; los adapters son el canal de entrada de los usuarios de frameworks.
- **Gate Result:** ✅ DO
- **Contrato:** los 9 adapters publicados en PyPI (`GET /pypi/<nombre>/json` = 200 con versión) **o** dry-run TestPyPI verde (5+ dists) + checklist final ejecutado por el owner (environment `pypi` + tag `adapters-v0.5.0`); post-publish: quitar los avisos "Not yet published" de los READMEs (commit de limpieza).
- **Pre-mortem:** (1) environment `pypi` no creado → paso 1 del checklist (owner); (2) tag mal formado → usar el namespace propio `adapters-v*` (no dispara wheels/npm del core); (3) drift de deps de frameworks entre build y publish → re-verificar con el dry-run antes del tag.
- **Stop conditions:** publish bloqueado por OIDC/credenciales → checklist al owner y cerrar la preparación con nota.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Env/OIDC mal configurado | Dry-run TestPyPI primero | antes del tag |
  | 🟢×🟡 | READMEs con aviso stale post-publish | Commit de limpieza post-publish | post-publish |

- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** (1) OIDC/env; (2) tag; (3) limpieza READMEs.
- **Uphill/Downhill:** ⬇️ (3 steps: dry-run → tag/publish owner → limpieza)
- **DoD:** task = contrato (9 publicados o dry-run + checklist) · commit = `docs:` (limpieza) · release = `adapters-v0.5.0`.
- **Validación Appetite vs Effort:** 1d ≥ 2h ✓
- **Skills sugeridas:** ci-cd-and-automation · shipping-and-launch · git-workflow-and-versioning
- **Estado:** ⬜ PENDING
- **Task file:** `docs/dev/tasks/MKT-20.md`

## DEFER

| ID | Por qué DEFER |
|----|----------------|
| docs-consolidation F3/F4 | Nombres de ADRs, copia `docs/user/book/src/`, task files como lastre, Structurizr C4, sitio estático — **decisión del owner** (marcado DEFER en su propio plan); reevaluar tras F0 |
| P24 I+D futura (v3.0+) | FUT-02..15 restantes (roadmap, no campaña); triggers por decisión de producto |
| MKT-19 / MKT-21 / DIST-18 | Lanes de marketing (humanos) + `vanta-proxy` **congelado hasta 1.0.0** (decisión owner 2026-10-01) |

## SKIP

| ID | Por qué SKIP |
|----|--------------|
| docs-consolidation F1-T3 | Ya SKIPPED en su plan con justificación: los `links: "[[README.md]]"` del glosario son **correctos** (propiedad Obsidian, exenta por diseño). F1-T4 además ya está COMPLETED (32 etiquetas reparadas). |

## BLOQUEADO

| ID | Bloqueante |
|----|-----------|
| ~~**#238**~~ ✅ | **Resuelto 2026-10-04 (decisión owner):** release **diferido** (post-fases) + changelog 0.8.0 limpiado. **Checkpoint F0 destrabado**. |

> **Nota:** MKT-20 (adapters PyPI) ya NO está bloqueado — se movió al cierre del plan como **F6 / Task 71** (owner-assisted), con checklist ejecutable.

## Notas

- **Fuente y contexto:** creado 2026-10-04 a partir de: DELTA 2026-09-30 (fuente designada del próximo plan), Alta 2026-10-01, hallazgos FIND-233..239 (run post-release 0.8.0 + validación externa v0.8.0), y el cierre del plan `2026-09-28-docs-consolidation.md` (F1/F2 absorbidas como Tasks 20/21). F6 (adapters) agregado por instrucción owner 2026-10-04.
- **Push:** ⛔ **solo al FINAL del plan** (instrucción owner 2026-10-04): todos los commits de la campaña se acumulan **locales**; el push —y el tag `adapters-v*` de F6— se ejecutan una sola vez al cierre autorizado.
- **Profundización por fase:** F0 full-detail (21+ENC-01) · F1 medium (15) · F2-F5 compactas (34) · F6 owner-assisted (1) — al iniciar cada fase se expanden al nivel F0 **antes** de ejecutar (regla del master-roadmap 2026-09-26, gate de fase explícito).
- **Dependencias internas:** Task 17 (DIST-04) → depende de 14/15/16 (DIST-01/02/03) · Task 47 (WIRE-16) → depende de 45/46 (WIRE-14/15) · Task 58 (MEMG-16) → consume 56/57 (MEMG-04/05) · Task 21 (DOCS-F2) → independiente · Checkpoint F0 → depende de #238 · Task 71 (F6) → al final del plan.
- **OCR gate:** toda tarea ✅ DO cierra con OCR delegation review (pipeline-full.md §Cierre paso 5); veredicto registrado en el task file.
- **plan-adjust:** registrar acá cualquier cambio de gate/re-estimación con el template de `plan.md` §"Evento plan adjust".
- **PRUEBAS PESADAS SERIALIZADAS (regla owner 2026-10-05):** UNA sola prueba pesada a la vez entre todas las sesiones (cargo nextest de crate/workspace, builds grandes, maturin). Lock: `pwsh dev-tools/heavy-test-lock.ps1 acquire|release|status` (TTL 45 min, espera hasta 30 min). Aplica a TODOS los workers; el orquestador además agenda máximo 1 tarea pesada en vuelo (emparejada con livianas/docs).
  - **2026-10-04 — Checkpoint F0:** `/audit certify` diferido a la ventana del release (decisión owner vía question: certificar el estado que se publica, no un intermedio; `just verify` mecánico = verde ahora). F1 expandida a nivel F0 (`3afe71c6`) antes de ejecutar (gate de fase cumplido).
- **SDP:** `campaign_discover_skills_v2` phase=PLAN (2026-10-04) → ver §SDP. Cada sub-agente corre su propio SDP en BUILD (≤10 skills) + los MCPs (codegraph, codebase-memory-mcp, campaign).
- **Estado inicial:** 72 tareas ⬜ PENDING (71 + ENC-01 addendum owner 2026-10-04) · 0 completed · 0 failed.

=== RECITATION ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: Plan Maestro — Release 0.9.0 + Memoria 1.0
Estado: in-progress
Última acción: Run iniciado 2026-10-04: wave 0 en vuelo (FIND-237/238/239); F6 (adapters owner-assisted, Task 71) agregado al cierre; push diferido al final (instrucción owner)
Resultado: OK
Próxima acción: procesar wave 0 → wave 1 (FIND-233/234/235/236)
Contrato: gates docs verdes + parseTasks 71 tareas
Próxima tarea si completa: F0 completa → Checkpoint release 0.9.0
=== END RECITATION ===

=== RECITATION FIND-237 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: CLI migrate: aceptar el global --db como fallback del target posicional (FIND-237, F0 Task 1)
Estado: completed
Última acción: Steps 1-4 ✅: RED (compile-fail + repro EXIT=2) → GREEN (target Option<String> + fallback en dispatch + after_help) → VERIFY (lib migrate 11/11 · cli::tests 5/5 · E2E 3/3 · cli_tests 94/94 · fmt/clippy exit 0 · campaign_verify_cmd passed) → OCR sin Critical/High → review P2-01 ronda 2 APPROVE → commit local ba9f38c4.
Resultado: OK
Próxima acción: ninguno — tarea cerrada. Siguiente del plan F0: FIND-238 (WASM logs DEBUG).
Contrato: verificacion: `cargo nextest run --profile audit -p vantadb --lib migrate` → 11/11 ✅ · `cargo nextest run --profile audit -p vantadb --test cli_tests --ignore-default-filter migrate_check` → 3/3 ✅ (local con --target-dir target/find237 por lock del exe; en CI corre tal cual). evidencia: claim: `migrate check --db X` exit 0 (fallback) → binario directo FALLBACK_EXIT=0 + E2E con binario real. claim: positional gana → POSITIONAL_EXIT=0 + E2E discriminante (--db a path inexistente). claim: help con ejemplo → `migrate --help` muestra 'vanta-cli migrate check --db ./my-db # global --db fallback'. claim: tests verdes → 11/11 + 5/5 + 3/3 + 94/94. artefactos: docs/dev/tasks/FIND-237.md, target/find237-ocr.json. invariantes: positional-first preservado (scripts existentes); handlers cmd_migrate* conservan firma &str; sin cambios de persistencia/formatos. deuda: ninguna. queda_pendiente: ninguno.
Próxima tarea si completa: FIND-238
=== END RECITATION ===

=== RECITATION FIND-239 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: FIND-239 — Docs DX post-validación v0.8.0 (encoding + formatos + get_node + caveats)
Estado: completed
Última acción: 4 gaps (a-d) cerrados en 3 archivos + FIND-240 derivado (hint magic); review fresco APPROVE con evidencia re-ejecutada (puente vs core/spec, shape vs test real, caveats vs portability tests)
Resultado: COMPLETED
Próxima acción: FIND-234 (claim) / procesar FIND-237 (commit ba9f38c4 ya en HEAD)
Contrato: check-links exit 0 + check-docs exit 0; (a-d) verificados 1:1 contra código por el reviewer
Próxima tarea si completa: FIND-234
=== END RECITATION ===

=== RECITATION FIND-238 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: FIND-238 — WASM/npm: silenciar logs DEBUG de Client.create() (tracing-wasm default-on sin filtro)
Estado: completed
Última acción: Steps 1-4 ✅: RED smoke (1f/1p con 40+ líneas DEBUG) → fix init() a WARN + gate globalThis.VANTADB_LOG + feature explícita → GREEN 2/2; verify: npm test 334/334, tsc/eslint 0, check wasm32 + --no-default-features ✅, wasm-pack build --release ✅, docs-coverage 0 gaps, fmt crate 0; OCR 0 findings; review vanta-review ✅ approve (Low solo proceso); commits locales 805a7668 (fix) + 7e093a2c (docs cierre).
Resultado: OK
Próxima acción: ninguno — tarea cerrada. nextTask: FIND-239.
Contrato: verificacion: `cd vantadb-ts && npx vitest run src/__tests__/find238.test.ts` ✅ 2/2 (vía campaign_verify_cmd) · `cd vantadb-ts && npm test` ✅ 334/334 · `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ · `cargo fmt -p vantadb-wasm --check` ✅ · `wasm-pack build --release` ✅ (3m26s) · `scripts/validate-docs-coverage.ps1` ✅ 0 gaps · pre-commit hook (fmt+clippy+actionlint) ✅
evidencia:
  - claim: Client.create() no emite DEBUG por defecto (nivel WARN)
    evidencia: test find238.test.ts (a) + RED pre-fix con 40+ líneas `DEBUG src\config.rs:*` capturadas
    confianza: alta
  - claim: gate globalThis.VANTADB_LOG re-activa los DEBUG y cae a WARN con valores inválidos
    evidencia: test (b) + probes del reviewer (123/bogus → WARN, sin crash)
    confianza: alta
  - claim: API del crate es set_as_global_default_with_config + WASMLayerConfigBuilder::set_max_level (no `_with_level`)
    evidencia: tracing-wasm-0.2.1/src/lib.rs:265-268,416 (fuente local del crate) + docs.rs
    confianza: alta
artefactos: docs/dev/tasks/FIND-238.md · vantadb-ts/src/__tests__/find238.test.ts · vantadb-wasm/{Cargo.toml,src/lib.rs,README.md} · Cargo.lock
invariantes: superficie pública del binding intacta (sin cambio funcional); --no-default-features compila; VANTADB_OPENAI_API_KEY nunca logueada por valor; gate leído una vez (one-shot).
deuda: ninguna.
queda_pendiente: (a) limpieza de la fila FIND-238 en docs/dev/Backlog.md (progreso); (b) hallazgo colateral pre-existente: clippy wasm32 `src/index/serialize/file.rs:146` drop_non_drop (core, no gateado) — decisión del orquestador; (c) vantadb-ts/README.md quedó en commit ajeno 2ef38561 (barrido FIND-239) — sin acción.
Próxima tarea si completa: FIND-239
=== END RECITATION ===

=== RECITATION FIND-233 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: FIND-233 — perf-bench: A/B same-job opt-in (ab_ref) para señal fina cross-VM
Estado: completed
Última acción: Implementación verificada (pares alternados + mediana de ratios; self-test 15/15; push intacto verificado vs docs GHA); review fresco APPROVE (tramo local); Step 7 (≥2 runs verdes + calibración same-SHA) DIFERIDO REGISTRADO al batch post-push
Resultado: COMPLETED
Próxima acción: DUR-03 (claim) — DX-01 y WSM-15 en vuelo
Contrato: tramo local completo (decisión implementada + self-test + actionlint + docs gates 0); tramo post-push diferido al cierre (comandos en task file)
Próxima tarea si completa: DUR-03
=== END RECITATION ===

=== RECITATION FIND-235 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: FIND-235 — ci-rust-10.md al día (21 jobs, coverage >=80%)
Estado: completed
Última acción: Contrato verificado con doble método (21 jobs, timeouts/paths/scopes 1:1) + gates re-ejecutados 0; review fresco APPROVE; derivadas: doc-drift por FIND-236 (fix en curso) + fila index de FIND-233 (se resuelve al commitear FIND-233)
Resultado: COMPLETED
Próxima acción: WSM-15 (claim) — FIND-233 y DX-01 en vuelo
Contrato: conteo 21 re-derivable + check-links/check-docs/gen-index exit 0
Próxima tarea si completa: WSM-15
=== END RECITATION ===

=== RECITATION FIND-234 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: FIND-234 — check-avance-coverage.ps1 lee docs/dev/avance
Estado: completed
Última acción: Fix de 1 línea (L10) verificado: 0/237 (0.0%) → 1034/1034 (100.0%); review fresco APPROVE con repro del antes desde HEAD~1
Resultado: COMPLETED
Próxima acción: FIND-236 (claim) / procesar FIND-233 y FIND-235 en vuelo
Contrato: script corre sin errores + reporte real 1034/1034 + exit 0
Próxima tarea si completa: FIND-236
=== END RECITATION ===

=== RECITATION FIND-236 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: FIND-236 — Job ASan excluye sift1m_competitive_benchmark (guard release-only)
Estado: completed
Última acción: Fix verificado (diff exacto + actionlint 0 + sintaxis --skip probada); review fresco APPROVE; Step 5 (run post-push) DIFERIDO REGISTRADO al batch de verificacion final
Resultado: COMPLETED
Próxima acción: DX-01 + WSM-15 (claim) — pendientes FIND-235 review y FIND-233 en vuelo
Contrato: skip explícito en el job + actionlint 0; verificación del run post-push diferida al cierre (registrada)
Próxima tarea si completa: DX-01
=== END RECITATION ===

=== RECITATION WSM-15 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: WSM-15 — OPFS multi-pestaña: lock per-archivo vía Web Locks API + fail-loud sin locks; repro multi-contexto browser sin corrupción; sin regresión single-tab
Estado: completed
Última acción: COMPLETADO. Fix: OpfsStorage::{write_file,append_file,delete_file} bajo lock per-archivo 'vantadb-opfs-write:<dir>:<path>' (inline_js hold-promise espejo idb.rs + WebLockGuard RAII) con fail-loud si Web Locks no está; 3 tests browser multi-contexto (página+worker real) RED→GREEN; suite 72/72 y 77/77 --features opfs; READMEs + FIND-242 + índice regenerado; review P2-01 vanta-review APPROVE (verificación independiente en worktree aislado); commits LOCALES 265abe6a (fix, hook verde) + 92055d2a (RESULTADO con hash).
Resultado: OK
Próxima acción: Orquestador: skill progreso (Backlog.md:218 fila WSM-15 + avance de dominio) — no ejecutado desde este leaf por convención de campaña; push diferido (owner). Próxima tarea del plan: DUR-03 (ya en vuelo por otro worker).
Contrato: verificacion: cd vantadb-wasm && wasm-pack test --chrome --headless → 72/72 (default) + 77/77 (--features opfs) ✅; RED pre-fix 69/3 con causas correctas ✅; re-ejecutado independientemente por vanta-review en worktree aislado ✅; verify_changed ALL 4 PASS ✅; check-docs/check-links/gen-index/markdownlint ✅; hook pre-commit del commit 265abe6a: fmt+clippy+actionlint ok ✅
evidencia:
- claim: RED 69/3 (write no espera 13ms<300ms; NoModificationAllowedError al mover el temp compartido; no fail-loud con locks shadowed) y GREEN 72/72 + 77/77 | evidencia: $env:TEMP/wsm15-{baseline,red,green2,green-opfs}.log + logs independientes del reviewer (%TEMP%\wsm15-review-*) | confianza: alta
- claim: review P2-01 APPROVE con verificación independiente (reruns propios + RED reproducido en worktree aislado; 0 Critical/Required) | evidencia: sesión ses_efa1f8a53ffeShYDTcWV7CEETT + docs/dev/tasks/WSM-15.md §Review | confianza: alta
- claim: commit local 265abe6a (8 archivos, hook verde) + 92055d2a (RESULTADO con hash); push NO ejecutado | evidencia: git log --oneline -2 | confianza: alta
- claim: Web Locks disponible en Window y Workers; hold-promise semántica MDN | evidencia: https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API + /LockManager/request (webfetch 2026-10-04) | confianza: alta
artefactos: docs/dev/tasks/WSM-15.md; commit 265abe6a (vantadb-wasm/src/opfs.rs, tests/wasm_tests.rs, README.md, demo/README.md, docs/dev/Backlog.md, docs/index.md, llms.txt); commit 92055d2a
invariantes: formato on-disk (CRC+temp+rename) intacto; sin API pública nueva; reads lock-free; sin deadlock (no se anidan locks; *_unlocked internos); fail-loud sin Web Locks; WIP ajeno intacto (DX-01/DUR-03/opencode.jsonc/plan file sin tocar)
deuda: FIND-242 (delete_file sin await de removeEntry, pre-existente, con repro determinista pendiente)
queda_pendiente: skill progreso por orquestador (eliminar fila Backlog.md:218 WSM-15 + registro en avance/ — no ejecutado desde este leaf por convención de campaña multi-worker); push diferido al final del plan (instrucción owner)
Próxima tarea si completa: DUR-03
=== END RECITATION ===

=== RECITATION DUR-03 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DUR-03 — H-023: put sobre key expirada-sin-purgar (NodeIdCollision) → purge-on-write
Estado: completed
Última acción: 5/5 steps: RED→GREEN (4 tests: expirado→put, put_batch, race vs sweeper stress 20/20, foreign-node) + generation guard RwLock + txn guard; suite -p vantadb 2549/2549 (shared) y rerun aislado; fmt/clippy/docs gates 0; OCR 0 Critical/High; review adversarial vanta-review r1 changes-required→corregido, r2 APPROVE; 3 commits locales (4cc1402d fix, 9f12f536 docs/tasks, 4476e536 docs/avance + fila Backlog removida); avance core-engine + FIND-244/245/250/251/252
Resultado: OK
Próxima acción: ninguno — DUR-03 cerrada; nextTask del plan: DUR-01 (ya en curso por otro worker)
Contrato: verificacion: cargo nextest run --profile audit -p vantadb ✅ 2549/2549 (shared) + rerun aislado CARGO_TARGET_DIR=target/dur03 ✅; focused 4/4 ✅; stress race 20/20 ✅; edge_cases 28 run/27 passed (único fallo FIND-244 pre-existente verificado en HEAD) ✅; fmt/clippy ✅; docs gates ✅. evidencia: claim 'put sobre expirada hace purge-on-write → Ok v1 fresco' — evidencia: tests put_after_ttl_expiry_succeeds_as_fresh_insert + put_batch_after_ttl_expiry_succeeds (RED NodeIdCollision transcripto pre-fix), confianza: alta; claim 'race vs sweeper sin errores' — evidencia: put_after_expiry_racing_ttl_sweeper_stays_consistent stress 20/20, confianza: alta; claim 'review adversarial aprobado' — evidencia: vanta-review ses_efa031798ffesZHUdsGIpjy4k1 r2 approve, confianza: alta. artefactos: docs/dev/tasks/DUR-03.md, tests/edge_cases.rs, src/sdk/api/memory.rs, src/sdk/builder.rs. invariantes: get/delete/list de expirados sin cambio (lazy TTL); upsert de keys vivas sin cambio; NodeIdCollision sigue para colisiones reales; put_record_exact pub(crate) sin test directo. deuda: FIND-245 (TOCTOU residual sin generation guard en batch/delete/multi-handle), FIND-250 (TTL vs cuarentena — decisión owner), FIND-251 (purge_expired sin sparse), FIND-252 (writer-vs-writer same-key), FIND-244 (test pre-existente rojo). queda_pendiente: push diferido al final del plan (instrucción owner); ACCEPT del orquestador.
Próxima tarea si completa: DUR-01
=== END RECITATION ===

=== RECITATION DX-01 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DX-01 — TS WASM `get` revienta en Node 26 (H-009): repro determinista + fix del trap + job CI Node 26
Estado: completed
Última acción: CIERRE COMPLETO. Fix toWireString en el wrapper TS + test dx01 (RED→GREEN) + job tests-node26 (Node 22+26, publish-wasm gateado por ambos) + troubleshooting doc + FIND-243 + fix del extractor TS del gate api-docs (prerequisito). Verify: 338/338 en Node 26.8.1 y 22.23.3; tsc/eslint/actionlint 0; OCR 0 findings; review P2-01 APPROVE. Commits locales bd641387/c5004b0b/c742143b/fd9f1b51. Progreso: fila Backlog removida + registro en docs/dev/avance/activo/bindings.md.
Resultado: OK
Próxima acción: Ninguno — tarea cerrada. Push diferido al cierre del plan (owner); los jobs CI Node 26 correrán post-push.
Contrato: verificacion: cd vantadb-ts && npm test (Node 26.8.1) 338/338 · Node 22.23.3 338/338 · npx tsc --noEmit 0 · npm run lint 0 · actionlint release-npm-61.yml 0 · repro H-009 post-fix: get({key:5}) coerce -> payload, get({key:{}}) -> DbError VANTADB_INVALID_ARGUMENT (sin trap) · gate api-docs --changed HEAD^..HEAD OK · docs gates 0
evidencia:
  - claim: repro determinista H-009 (get con arg no-string → RuntimeError memory access out of bounds @ passStringToWasm0:1551) | evidencia: stack raw + repro en Node 22.23.3/26.0.0/26.8.1/26.10.0 con vantadb@0.8.0 publicado | confianza: alta
  - claim: fix verificado RED→GREEN | evidencia: dx01.test.ts 4/4 fallos pre-fix (trap exacto) → 4/4 pass post-fix; suite 338/338 en 2 versiones | confianza: alta
  - claim: CI cubre Node 22+26 (job agregado) | evidencia: job tests-node26 espejo del gate + publish-wasm needs [tests, tests-node26] + actionlint 0 | confianza: alta
  - claim: review por agente distinto | evidencia: vanta-review APPROVE — re-ejecutó suite, repro independiente del glue crudo, diff normalizado de jobs | confianza: alta
artefactos: docs/dev/tasks/DX-01.md · vantadb-ts/src/__tests__/dx01.test.ts · commits locales bd641387 (fix) + c5004b0b (gate fix) + c742143b (avance) + fd9f1b51 (RESULTADO)
invariantes: get/delete/list con strings intactos; sin cambio de wire format ni firmas; put/search (serde) sin cambios; job Node 22 sin cambios
deuda: ninguna (FIND-243 = residual trazado: superficie vantadb-wasm directa sigue expuesta a la misma clase)
queda_pendiente: push DIFERIDO por instrucción del owner (job tests-node26 correrá en el PR); orquestador: actualizar plan Task 8 → COMPLETED + checkpoint de wave
Próxima tarea si completa: DUR-02 (siguiente ⬜ PENDING del plan; DUR-03/DUR-01 en vuelo con otros workers)
=== END RECITATION ===

=== RECITATION DUR-01 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DUR-01 — Auditoría fsync real (WAL/snapshots/GC) + dir-fsync G1
Estado: completed
Última acción: Ronda 2 APPROVE (fresh): dir-fsync verificado en ambos sitios de creación + fix fs.rs; mapa G1-G12 spot-check ~20 citas; gates 91/91 wal + 46/46 snapshot en worktree limpio; FIND-246..248 registrados (bac31a3e); justificación estructural de canonical_p99 ACEPTADA por el orquestador
Resultado: COMPLETED
Próxima acción: BENCH-01 (claim) — DUR-03 y DUR-02 en vuelo
Contrato: mapa path→fsync completo + G1 resuelto con evidencia + suite verde; canonical_p99 aceptado estructuralmente (fix fuera del path medido, bench in-memory)
Próxima tarea si completa: BENCH-01
=== END RECITATION ===

=== RECITATION DUR-02 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DUR-02 — Auditoría cobertura AES (encryption): WAL / text_index / HNSW / edge_index / snapshots
Estado: completed
Última acción: Cerrada: mapa artefacto→cifrado 0/6 con evidencia por artefacto + probe tmpdir (canary plaintext en vanta.shard0.wal y 0.jnl) + fix encryption_stream (RED→GREEN, nextest 42/42) + docs corregidos (CONFIGURATION/FEATURES) + FIND-249 en Backlog + gates docs 0 + OCR sin Critical/High + review P2-01 vanta-review APPROVE. Commits locales 16afd036 (fix) + 65008548 (avance/cierre).
Resultado: OK
Próxima acción: ninguno — DUR-02 cerrada. Orquestador: migrar la fila DUR-02 del Backlog a avance (skill progreso) al cierre de fase; decidir FIND-249 (wiring AES vs primitivas-only).
Contrato: verificacion: cargo nextest run --profile audit -p vantadb --features encryption -E 'test(/vfile/) + test(/encryption/)' ✅ 42/42 · cargo fmt -p vantadb --check ✅ · cargo clippy -p vantadb --features encryption --all-targets -- -D warnings ✅ · check-links/check-docs/gen-index --check/validate-docs-coverage ✅ | evidencia: claim '0 callers' → grep workspace completo (alta) | claim 'plaintext on-disk' → probe nextest canary en data/vanta.shard0.wal + 0.jnl (alta) | claim 'fix correcto' → RED panic pre-fix + GREEN (alta) | claim 'review aprobado' → vanta-review APPROVE con re-ejecución propia (alta) | artefactos: docs/dev/tasks/DUR-02.md · docs/dev/avance/auditoria/seguridad.md · FIND-249 en docs/dev/Backlog.md | invariantes: src/wal.rs y src/sdk/api/memory.rs NO tocados (WIP DUR-01/03); crypto.rs/envelope.rs intactos; sin cambios de formato on-disk; feature encryption sigue opt-in y compilable | deuda: FIND-249 (cablear cifrado o declarar primitivas-only — decisión owner/arch); H5 opcional (probe transcript-only, reproducibilidad futura) | queda_pendiente: migrar fila DUR-02 del Backlog a avance (skill progreso) al cierre de fase; decisión de diseño FIND-249
Próxima tarea si completa: BENCH-01
=== END RECITATION ===

=== RECITATION BENCH-01 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: BENCH-01 — Fix doble-conteo competitive_bench (timer + clamp)
Estado: completed
Última acción: Timer aislado (init/prep fuera: ~318ms/run) + clamp <1000 por construcción + self-test 9/9; review fresco APPROVE (RED→GREEN reproducido externo, trazado sin caminos ≥1000, rebuilds=1 no circular); FIND-253 ruteada (d497bdb6)
Resultado: COMPLETED
Próxima acción: DIST-02 (claim al cerrar DIST-01 — mismo árbol de archivos) — DIST-01 cerrando
Contrato: región medida aislada + clamp efectivo + self-test + docs sin sobre-claim
Próxima tarea si completa: DIST-02
=== END RECITATION ===

=== RECITATION DIST-01 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-01 — Publicar vanta-memory (quitar publish=false): dry-run verde + smoke externo + decisión release-plz registrada
Estado: completed
Última acción: Steps 1-6 ✅: baseline (exit 101 por publish=false) → Cargo.toml publicable (metadata crates.io + version=0.8.0 en el dep path) → dry-run exit 0 (Packaged 139 files, 1.2MiB; build aislado compiló vantadb 0.8.0 de crates.io) → hold release-plz (release=false + checklist de unblock owner) → doc-sync lib.rs → smoke externo tmp (cargo run exit 0) → gates docs + OCR 0 hallazgos + review P2-01 vanta-review APPROVE → commits locales 53996863 (impl) + 0508fc2f (FIND-254) + 180bfb94 (cierre)
Resultado: OK
Próxima acción: ninguno — tarea cerrada. Próxima del plan F0: DIST-02 (exponer la capa cognitiva en Python).
Contrato: verificacion: `cargo publish --dry-run -p vanta-memory --allow-dirty` ✅ exit 0 (Packaged 139 files, 1.2MiB / 328.7KiB compressed; 'aborting upload due to dry run') [campaign_verify_cmd] · smoke externo `cargo run` en %TEMP%\dist01-smoke ✅ exit 0 ('dist01-smoke OK: vanta-memory loaded as external dep (recall max_results=5)') · `python tomllib` release-plz.toml ✅ · gates docs (check-links/check-docs/gen-index/coverage) ✅ 0 gaps · review P2-01 APPROVE. evidencia: claim: dry-run compila aislado contra crates.io (no path local) → Cargo.lock del paquete con source=registry + checksum 3eeef49, re-verificado por el reviewer | confianza: alta. claim: hold release=false correcto vs alternativas → docs release-plz + prereq trusted publishing (crates.io) + repro `release-plz update` en clone: requirement sincronizado en lockstep pese a release=false | confianza: alta. claim: sin publish real ni push → crates.io API vanta-memory 404; `git branch -r --contains` vacío; dry-run aborta upload | confianza: alta. artefactos: vanta-memory/Cargo.toml · release-plz.toml · vanta-memory/src/lib.rs · docs/dev/tasks/DIST-01.md · docs/dev/Backlog.md (FIND-254) · commits 53996863/0508fc2f/180bfb94. invariantes: consumidores in-process (vantadb-mcp, vanta-proxy, desktop) intactos; Cargo.lock sin cambios; hold vigente para #238; WIP ajeno intacto (opencode.jsonc/plan file/benchmarks/src-sdk). deuda: FIND-254 (tests empaquetados no compilan desde el .crate — Low, no bloquea publish/docs.rs/consumidores); README del crate → DIST-04. queda_pendiente: (a) skill progreso por orquestador (fila Backlog DIST-01:130 → avance; ya hay checkpoint del orquestador); (b) unblock checklist owner en ventana #238 (bootstrap con token → Trusted Publishing → borrar hold release-plz); (c) push diferido al final del plan (instrucción owner).
Próxima tarea si completa: DIST-02
=== END RECITATION ===

=== RECITATION DIST-03 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-03 — TS/Node/WASM: declarar scope de la capa cognitiva por binding (decisión (b) implementada y verificable)
Estado: completed
Última acción: Steps 1-5 ✅: DISCOVERY (4 compile checks wasm + 3 fuentes oficiales + auditoría capabilities 4 bindings) → decisión (b) → declaración en BINDINGS_NAMESPACES.md §Cognitive layer scope (matriz 7×4 + paridad capabilities() + invariante rg) + sync VANTA_MEMORY.md → FIND-255/256 → gates docs verdes (check-links/check-docs/gen-index/coverage 0 gaps/markdownlint) + OCR (markdown N/A) + review P2-01 vanta-review (ronda 1 changes-required R-1 → corregido 03d2a511 → APPROVE ronda 2) → 5 commits locales
Resultado: OK
Próxima acción: ninguno — tarea cerrada. Próxima del plan F0: DIST-04 (depende de 14/15/16; reconciliar VANTA_MEMORY.md + conteos post DIST-01/02/03).
Contrato: verificacion: gates docs ✅ `node scripts/docs/check-links.mjs` exit 0 · `check-docs.mjs` gating all clear · `gen-index.mjs --check` exit 0 · `pwsh scripts/validate-docs-coverage.ps1` exit 0 (0 gaps) · markdownlint 3 archivos 0 issues · invariante `rg "vanta[_-]memory" vantadb-ts vantadb-node vantadb-wasm` → 0 matches. evidencia: claim: decisión (b) justificada (port wasm, no wrapper) | evidencia: 4 compile checks (exit 101 baseline getrandom / Finished con vantadb/wasm / exit 101 llm-driver E0433 / exit 101 embeddings E0432) + web-time/rustc/reqwest docs — todo re-ejecutado por vanta-review | confianza: alta. claim: declaración + matriz + rationale publicadas | evidencia: commit 9b3baa89 (BINDINGS_NAMESPACES.md §Cognitive layer scope + VANTA_MEMORY.md) | confianza: alta. claim: FIND-255 (port wasm, 6 gaps) + FIND-256 (drift capabilities Python) registrados | evidencia: commit 38c728fd, Backlog:429-430 | confianza: alta. claim: review P2-01 APPROVE final | evidencia: ses_ef95dc5faffe989c4NEyl7oabI (ronda 2, re-ejecutó checks y confirmó R-1) | confianza: alta. claim: sin push | evidencia: commits locales 9b3baa89/38c728fd/173103cb/03d2a511/e10710f5, ahead en origin/develop | confianza: alta. artefactos: docs/api/BINDINGS_NAMESPACES.md · docs/api/VANTA_MEMORY.md · docs/dev/tasks/DIST-03.md · docs/dev/Backlog.md (FIND-255/256) · docs/index.md · llms.txt. invariantes: TS/Node/WASM siguen 0 símbolos vanta-memory (invariante mecánico documentado y verificado); vantadb-python intacto por DIST-03; sección D43 intacta (extendida, no reemplazada); sin push. deuda: FIND-255 (port wasm, 🟠 3-5d) + FIND-256 (drift capabilities Python, 🟢 2-3h); conteo 46→48 de BINDINGS_NAMESPACES.md + reconciliación final de VANTA_MEMORY.md (facade/versioning/README) → DIST-04. queda_pendiente: (a) skill progreso/avance por orquestador (convención del campaign, igual que DIST-01); (b) DIST-04 al cerrar 14/15/16; (c) push diferido al final del plan (instrucción owner).
Próxima tarea si completa: DIST-04
=== END RECITATION ===

=== RECITATION DIST-02 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-02 — exponer la capa cognitiva en Python (memory_capture/memory_recall), scope mínimo con smoke de wheel local
Estado: completed
Última acción: Steps 1-4 ✅: DISCOVERY (task file) → dep vanta-memory + 2 métodos Rust (py.detach + OpGate + error mapping) → RED→GREEN (10 AttributeError → 11/11; suite 173 passed) + stubs/wrapper async/docs + smoke e2e con wheel local en venv limpio → gates (fmt/clippy/api-docs por rango/coverage/avance) + OCR 0 findings + review P2-01 (ronda 1 changes-required R1 → fix docs+test ee731884 → APPROVE) → commits 2a3bccbc + ee731884 + 2e181480 (local, sin push) + avance + fila Backlog removida
Resultado: OK
Próxima acción: ninguno — DIST-02 cerrada. Próxima desbloqueada: DIST-04 (deps 01/02/03 ✅).
Contrato: verificacion: `target/dist02-venv/Scripts/python -m pytest vantadb-python/tests/ -q` ✅ 173 passed/4 skipped/4 deselected (re-ejecutada por reviewer) · `pytest tests/test_memory_layer.py -q` ✅ 11/11 · smoke `python target/tmp/dist02-smoke.py` ✅ exit 0 (wheel 0.8.0 desde site-packages del venv; capture→reopen persistente→recall hit→None) · `node scripts/docs/check-api-docs.mjs --changed HEAD~1..HEAD` ✅ (+12 superficie Python, docs+llms movidos) · `cargo check/clippy -p vantadb_py` ✅ · `cargo fmt --all --check` ✅ · docs gates ✅ (check-links/check-docs/gen-index/validate-docs-coverage 0 gaps/check-avance-coverage 1034/1034) · test_stub_drift 7/7. evidencia: claim: métodos end-to-end → RED 10 fallos AttributeError pre-build + GREEN 11/11 post-build + smoke con import path site-packages (verificado por reviewer con sonda) | confianza: alta. claim: dep sin red/LLM → `cargo tree -p vantadb_py -i reqwest` exit 101 (ausente) + vanta-memory solo default (reviewer) | confianza: alta. claim: docs honestos → R1 reproducido por reviewer (capture alone → None) + fix + test negativo que lo pinea | confianza: alta. claim: review APPROVE → ses_ef9569455ffeOawsQHJX4n3kV9 ronda 2 con re-ejecución propia | confianza: alta. artefactos: vantadb-python/{Cargo.toml,src/lib.rs,tests/test_memory_layer.py,vantadb_py/*.pyi,vantadb_py/__init__.py} · docs/api/PYTHON_SDK.md · docs/dev/tasks/DIST-02.md · docs/dev/Backlog.md (FIND-257/258/259) · docs/dev/avance/activo/bindings.md · commits 2a3bccbc/ee731884/2e181480. invariantes: lógica en vanta-memory (binding glue R-8); GIL liberado (py.detach) + OpGate; None = nada que inyectar; degradación keyword reportada; aditivo (0 métodos existentes modificados); vantadb-ts/wasm/vanta-memory/Cargo.toml/release-plz.toml/BINDINGS_NAMESPACES.md intactos; WIP ajeno (plan file/opencode.jsonc) intacto. deuda: dream = follow-up declarado (contrato); FIND-258 (chain-walk de errores anidados); FIND-259 (L1 type inválido invisible + enum en VANTA_MEMORY); FIND-257 (gate api-docs vs WORKTREE CRLF); BINDINGS_NAMESPACES 46→48 (handoff DIST-03/04); Low/Nit declarados (max_results<0 OverflowError, id='' ValidationError, async kw-only). queda_pendiente: (a) push diferido al final del plan (instrucción owner) — 3 commits locales; (b) DIST-04 usa este estado (superficie Python publicada); (c) FIND-258/259 al drain de P2 cuando se priorice.
Próxima tarea si completa: DIST-04
=== END RECITATION ===

=== RECITATION DIST-05 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-05 — Assets del release + verificación post-release real (fix del 404): flujo de verificación ejecutable + install.ps1 parametrizado/verificado contra assets reales
Estado: completed
Última acción: Steps 1-6 ✅: verify-release.ps1 (14 assets + 4 registries + smoke sha256/--version; GH_TOKEN scope api.github.com; no ejecuta con mismatch) + release-verify.yml (schedule Mon 08:00 + dispatch) + PUBLISH.md gate de cierre + install.ps1 (-Version/-InstallDir, fail-loud sin v0.4.0, compat PS5.1 ternary+BOM, layout flat/nested) + inventarios 37→38 + OCR (inyección del workflow corregida via env) + review P2-01 APPROVE (Medium+NIT aplicados) + commit local 2390344c. Hallazgo clave: el zip real es FLAT y install.ps1 buscaba release\ → instalador roto end-to-end contra el asset real.
Resultado: OK
Próxima acción: ninguno — DIST-05 cerrada. Próxima desbloqueada: DIST-06 (F0). Verificación en 0.9.0 = diferida (checkpoint F0→0.9.0; release diferido por owner).
Contrato: verificacion: `pwsh scripts/verify-release.ps1 -Tag v0.8.0 -Smoke` ✅ exit 0 (14/14 assets + 4/4 registries + sha256 + vanta-cli 0.8.0) · `-Tag v0.7.0` ✅ exit 1 (solo 10 binarios — detecta el 404) · `install.ps1 -DryRun` ✅ 5.1 y 7 · installs reales aislados (5.1 y 7, doble corrida = backup) ✅ vanta-cli 0.8.0 · `actionlint release-verify.yml` ✅ 0 · `check-links`/`check-docs` ✅ 0 · workflows count = 38. evidencia: claim: zip v0.8.0 flat → `tar -tf` del asset real (vanta-cli.exe raíz) | confianza: alta. claim: install.ps1 roto contra asset real (buscaba release\) → repro + fix + install real aislado ✅ | confianza: alta. claim: PS5.1 no parseaba (ternary + sin BOM) → repro powershell -File antes/después | confianza: alta. claim: v0.7.0 sin binarios por cascade pre-PAT (no decisión) → release-binaries.yml@v0.7.0 sin input release_tag + gh secret list (PAT 2026-10-02) + runs | confianza: alta. claim: review APPROVE → ses_ef929e5fbffeOOnNT06tUugRjw con re-ejecución propia de los 6 comandos | confianza: alta. artefactos: scripts/verify-release.ps1 · scripts/install.ps1 · .github/workflows/release-verify.yml · docs/dev/workflow/{PUBLISH,README,TRIGGERS}.md · docs/dev/references/verified-numbers.md · docs/dev/tasks/DIST-05.md · commit 2390344c. invariantes: sin push; release-plz/versiones/changelog intactos; release-binaries.yml sin cambios; WIP ajeno intacto (plan file/opencode.jsonc); installer sin dependencias nuevas. deuda: verificación en 0.9.0 diferida (release diferido por owner); backfill v0.7.0 documentado como procedimiento opcional (no ejecutado — remoto); OPTIONALs del reviewer (fail-closed si falta .sha256; regex tags pre-release) → candidatos a FIND del orquestador. queda_pendiente: (a) orquestador: sincronizar plan file (Task 18) + fila Backlog DIST-05 + evaluar FIND de los 2 OPTIONALs; (b) verificación 0.9.0 al retomarse el release (checklist PUBLISH.md §Post-release verification); (c) push diferido al cierre del plan (1 commit local 2390344c).
Próxima tarea si completa: DIST-06
=== END RECITATION ===

=== RECITATION DIST-04 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-04 — VANTA_MEMORY.md ↔ realidad (cierre post DIST-01/02/03)
Estado: completed
Última acción: Reconciliación completa (VANTA_MEMORY.md + BINDINGS_NAMESPACES.md matriz ✅ counts 46→48 + README del crate); ronda 1 changes-required (R-1/R-2) → aplicados c46b2cdd → ronda 2 APPROVE; FIND-260 derivado (query_structured ausente)
Resultado: COMPLETED
Próxima acción: DIST-06 (claim) — DIST-05 en vuelo
Contrato: doc ↔ realidad + gates 0/0/0; R-1/R-2 cerrados con evidencia
Próxima tarea si completa: DIST-06
=== END RECITATION ===

=== RECITATION DIST-06 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-06 — Estrategia de los 11 crates publish = false: decisión por crate en PUBLISH.md §crates + invariante release-plz verificable
Estado: completed
Última acción: Steps 2-4: sección §crates insertada (64 líneas netas, 0 deleciones) + F1/F3 del review aplicados; verify mecánico CMD(a)=10 filas / CMD(b)=3 overrides; gates docs verdes (pre-WIP ajeno DOCS-F1); OCR 0 reviewable; review P2-01 APPROVE (ronda 1 changes-required menor F1 → fix → delta approve); commit local 56b0bc0a (4 paths, sin push)
Resultado: OK
Próxima acción: ninguno — DIST-06 cerrada. Próxima desbloqueada: DOCS-F1 (plan Task 20)
Contrato: verificacion: CMD(a) `rg '^\s*publish = false'` = 10 crates ✅ · CMD(b) `rg '^\[\[package\]\]|^name = |^\s*(release|publish) = '` = vantadb/vantadb-wasm/vanta-memory ✅ (ambos vía campaign_verify_cmd passed:true) · gates docs check-links/check-docs/gen-index exit 0 al momento del review ✅. evidencia: claim=decisión por crate documentada (11/11) evidencia=docs/dev/workflow/PUBLISH.md §crates (L117-177) confianza=alta · claim=consistencia release-plz evidencia=release-plz.toml (solo lectura) + comandos re-ejecutados confianza=alta · claim=Freeze List proxy no-publica-hasta-1.0.0 evidencia=docs/user/operations/EXPERIMENTAL_FEATURES.md §Freeze List confianza=alta · claim=providers no PyPI evidencia=pypi.org 404×3 (live 2026-10-04) confianza=alta. artefactos: docs/dev/workflow/PUBLISH.md, docs/dev/tasks/DIST-06.md, docs/index.md, llms.txt. invariantes: release-plz.toml/versiones/CHANGELOG intactos; WIP ajeno intacto (opencode.jsonc, plan, DOCS-F1 en vuelo); canales reales no cambian. deuda: ninguna (F4 boundary del invariante anotado para próximo review de la política). queda_pendiente: pase de progreso/avance + sincronización del plan = orquestador (precedente DIST-05); push diferido al final del plan (owner).
Próxima tarea si completa: DOCS-F1
=== END RECITATION ===

=== RECITATION 20 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DOCS-F1 — Cerrar docs-consolidation F1 (triage de links + mojibake + markdownlint)
Estado: completed
Última acción: RONDA 2 completa: verdict adversarial fresco (vanta-review) = changes-required -> fixes C-1 (links CI-only: HTTP_API + discord), R-1 (rrf.md completo: 49 runs CP437->0), R-2/O-1 (FIND-264/265 registrados; 263 tomado por DOCS-F2), R-3 (triage reconciliado 55 local/60 canonico), O-2 ('15 de 46' + metodo), O-4 (nota commit). Checkout limpio (git archive HEAD): check-links 0 + gen-index 0. Commits locales a5909e38 (ronda 2) + 5404ebb3 (avance). DOCS-F2 commiteo en paralelo (100e3fef/83ceea01) — staging respetado.
Resultado: OK
Próxima acción: Re-review adversarial del delta (vanta-review) — luego DOCS-F2 ya cerrada; siguiente del plan: lo que defina el orquestador
Contrato: verificacion: checkout limpio (git archive HEAD 83ceea01): check-links 0 broken/20 wikilinks exit 0 | gen-index --check 0. Local: check-links 0 | check-docs 0 | gen-index 0 | markdownlint 0 en superficie controlada (N CI = 0) | actionlint 0
evidencia:
  - claim: C-1 resuelto — 0 links dependientes de archivos gitignored (HTTP_API .opencode -> code span; discord todo.md -> de-link); checkout limpio da 0 (pre-ronda2: 2) | evidencia: git archive HEAD + node scripts/docs/check-links.mjs | confianza: alta
  - claim: R-1 resuelto — rrf.md restaurado completo: 49 runs CP437 -> 0; H1 '# RRF—Reciprocal Rank Fusion'; L231 `[BM25](./bm25.md)` (code-span, no clicable); cross-check contra revision limpia 58a41ad8 | evidencia: rg CP437 = 0; git show 58a41ad8:... | confianza: alta
  - claim: R-2/O-1 resueltos — FIND-264 (recitations MD007 + nota de retiro) y FIND-265 (barrido CP437 restante) registrados en Backlog; 263 ya tomado por DOCS-F2 en vuelo (reasignado) | evidencia: docs/dev/Backlog.md filas 264/265 | confianza: alta
  - claim: R-3 resuelto — triage reconciliado contra snapshot canonico (checkout limpio 3bdc3fd5): local 55 (20/11/24) vs canonico 60 (22/11/27); delta +5 CI-only via .venv/.opencode/todo.md | evidencia: docs/dev/tasks/DOCS-F1.md tabla + N6 | confianza: alta
  - claim: O-2/O-4 resueltos — '21 de 44' -> '15 de 46 fully-in-code' con metodo documentado (N6 + comentario del script); nota de tamano de commit en DoD (generados no cuentan) | evidencia: scripts/docs/check-links.mjs + DOCS-F1.md | confianza: alta
artefactos: docs/dev/tasks/DOCS-F1.md; docs/api/HTTP_API.md; docs/user/discord/README.md; docs/user/glosario/rrf.md; docs/dev/Backlog.md; scripts/docs/check-links.mjs; commits a5909e38 (ronda 2) + 5404ebb3 (avance ronda 2)
invariantes: tasks/plans no se mueven; property links de Obsidian intacta; indices generados no se editan a mano; links solo con destino unico verificado; master plan/opencode.jsonc intactos; no se toco el staging de DOCS-F2
deuda: FIND-264 (master plan MD007 + retiro de exclusion al archivar); FIND-265 (barrido CP437: ann 28, compaction 29, failpoints 13, ci-cd 7, Informe 1, generados 2+2); wikilinks frozen (20) en budget
queda_pendiente: re-review adversarial del delta ronda 2 (vanta-review ses_ef8f093d1ffeuZhUBhU4WR3rW9); nota de retiro en el workflow (orquestador); push diferido al cierre del plan (owner)
Próxima tarea si completa: DOCS-F2 (cerrada por su sub-agente)
=== END RECITATION ===

=== RECITATION 21 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: Cerrar docs-consolidation F2: ejemplos de docs ejecutables en CI (Rust doctests -D warnings + Python pydoclint) + verificar gates T14/T15 + marcar F2 ✅ (TS derivado a FIND-263)
Estado: completed
Última acción: CIERRE COMPLETO: doctests Rust (job doctests, medido EXIT 0) + pydoclint (4→0) cableados a CI; T14/T15 verificados (ya existían desde 71139665; self-tests 17/17 y 29/29); TS→FIND-263; F2 ✅ en el plan (T13/T14/T15 COMPLETED); verify.ps1 ALL 10 PASS; OCR 0 Critical/High. Commits locales 100e3fef (tarea) + 83ceea01 (avance).
Resultado: OK
Próxima acción: ninguno — DOCS-F2 cerrada. Próxima desbloqueada: ENC-01 (Task 72)
Contrato: verificacion: `dev-tools/verify.ps1` ALL 10 PASS ✅ · `RUSTDOCFLAGS="-D warnings" cargo test --doc --workspace` EXIT 0 ✅ (13+1i vantadb, 1 vanta-memory, 1 mcp) · `pydoclint --style=numpy` exit 0 ✅ · actionlint 0 ✅ · check-links/check-docs/gen-index verdes ✅ · self-tests gates 17/17 + 29/29 ✅. evidencia: claim=T14/T15 YA existían (commit 71139665, 2026-09-29) y se verificaron sin reimplementar, evidencia=.github/workflows/gate-api-docs.yml + gate-docs-secrets.yml + self-tests, confianza=alta · claim=doctests workspace verdes con -D warnings, evidencia=log local EXIT 0 (medido Windows; primer run ubuntu en CI), confianza=alta · claim=pydoclint 4 violaciones→0, evidencia='No violations' EXIT 0, confianza=alta · claim=TS docs sin infra CI (typedoc 0.28.20: 0 errores/39 warnings; sin job TS en PRs), evidencia=FIND-263 + workflows, confianza=alta. artefactos: docs/dev/tasks/DOCS-F2.md · .github/workflows/ci-rustdoc.yml · .github/workflows/gate-doc-examples.yml · vantadb-python/vantadb_py/__init__.py · docs/dev/plans/2026-09-28-docs-consolidation.md · docs/dev/Backlog.md (FIND-263) · docs/dev/avance/activo/ci-cd.md. invariantes: master plan/opencode.jsonc/pipeline-state/PUBLISH/release-plz/gate-docs-links intactos; gates T14/T15 solo verificados (no modificados); RULES.md obedecidas (SHA pins, timeouts, permissions, sin continue-on-error nuevo); push diferido (owner). deuda: TS docs → FIND-263 (burn-down: typedoc config+devDep+job; drenar 39 warnings; harness de bloques TS). queda_pendiente: review P2-01 formal vanta-review (fresh) — degradado escalado al orquestador; primer run de los jobs nuevos en CI ubuntu (si un doctest falla por plataforma, es finding real a arreglar); push diferido al cierre del plan (owner).
Próxima tarea si completa: ENC-01
=== END RECITATION ===

=== RECITATION 72 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: ENC-01 — Cifrado honesto: aviso accionable al activar `encryption` (feature+key) + docs sin ambigüedad + test que lo fija (FIND-249 parte 1)
Estado: completed
Última acción: Cierre completo: warning emitido en StorageEngine::open_with_config (gating por entorno, excluye InMemory) + rustdoc/docs honestos (CONFIGURATION.md :73/§7, FEATURES.md:47, config.rs, lib.rs) + 3 tests feature-gated hermeticos (4 escenarios de env). Commits locales b02b1609 (fix) + 4b029c7d (docs(avance)). Registrado en docs/dev/avance/auditoria/seguridad.md.
Resultado: OK
Próxima acción: ninguno — última tarea F0 (22/22). Checkpoint F0 → F1: just verify + /audit certify en develop (lo maneja el orquestador).
Contrato: verificacion: `cargo nextest run --profile audit -p vantadb --features encryption -E "test(/encryption_notice/)"` → 3/3 en 4 escenarios de entorno (limpio / VANTADB_ENCRYPTION_KEY / VANTADB_BACKEND=memory / ambas) ✅ · suites completas `-p vantadb`: 2575/2575 (con feature, 251.9s) + 2549/2549 (sin, 241.7s) ✅ · `cargo fmt --check` 0 · `cargo clippy -p vantadb --features encryption --all-targets -- -D warnings` 0 · gates docs (check-links/check-docs/gen-index --check/validate-docs-coverage) 0 · check-avance-coverage 1034/1034 ✅. Nota: campaign_verify_cmd devolvió exitCode -1 sin output para `cargo fmt`/`cargo nextest` (spawn del wrapper; no falla de código) — verificado por shell directo. · evidencia: [claim: 0/6 artefactos on-disk cifrados con feature activa → evidencia: docs/dev/tasks/DUR-02.md §Mapa, confianza alta] [claim: warning presente y accionable al activar (NOT yet wired/PLAINTEXT/FIND-249) → evidencia: src/storage/engine/init.rs:617-638 + test positivo, confianza alta] [claim: docs sin ambigüedad → evidencia: docs/user/operations/CONFIGURATION.md:73,495 + docs/dev/architecture/FEATURES.md:47, confianza alta] [claim: review adversarial APPROVE → evidencia: ses_ef8aac205ffek6RD7sIQZnc1mA (R1 changes-required hermeticidad → fix → R2 approve), confianza alta] · artefactos: src/storage/engine/init.rs, src/storage/engine/tests/init.rs, src/config.rs, src/lib.rs, docs/user/operations/CONFIGURATION.md, docs/dev/architecture/FEATURES.md, docs/dev/Backlog.md (nota FIND-249), docs/dev/tasks/ENC-01.md, docs/dev/avance/auditoria/seguridad.md, docs/index.md, llms.txt · invariantes: sin símbolos públicos nuevos; open sigue exitoso (solo log); sin feature = byte-idéntico; crypto.rs/vfile.rs/wal.rs intactos; cableado del cifrado NO incluido (FIND-249) · deuda: FIND-249 (wiring 3-5d, diferido por owner; nota 'key sin feature compilada no avisa' registrada en la fila). Nit §7 de docs incorporado post-R2 (texto recomendado por reviewer). · queda_pendiente: checkpoint F0 (just verify + /audit certify) y actualización del plan file por el orquestador (prohibido para esta tarea).
Próxima tarea si completa: checkpoint F0 → F1
=== END RECITATION ===

=== RECITATION 22 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DX-12 — instalador/wizard como selector de módulos (7 componentes; estado installable/frozen; ambos caminos; idempotencia)
Estado: completed
Última acción: Steps 1-4 completos: wizard con tabla de módulos + `-Modules` + gating por módulo; instaladores `-Modules`/`--modules` pass-through + dry-run + exit-code check del wizard; docs README/QUICKSTART; smoke 7/7 (ambos caminos + idempotencia + dry-runs 5.1/7 + sh); OCR pass (0 Critical/High); review P2-01 vanta-review: changes-required (High: coerción [string] en install.ps1) → fix espejo [string[]]+join → APPROVE; commit local 9e211d10 (sin push)
Resultado: OK
Próxima acción: Orquestador: registrar Review/aprobación, regenerar docs/index.md + llms.txt en el checkpoint (stale por WIP DIST-15), y decidir próximo task F1 (DIST-17 pendiente)
Contrato: verificacion: 7/7 comandos del Contrato ✅ (smoke ambos caminos + idempotencia artefactos hash-estables + dry-runs pwsh7/powershell5.1/Git-sh + docs gates 0) | evidencia: claim: 'selección por componente aplicada idempotente' evidencia: doble corrida hashes mcp-*.json+vanta-proxy.toml idénticos, 0 .bak nuevos, solo 0.jnl (live test) confianza: alta; claim: 'no-interactive = defaults actuales' evidencia: modules=mcp,server,proxy,embeddings + provider local + PRUEBA VIVA verde confianza: alta; claim: 'High del reviewer cerrado' evidencia: `& ./scripts/install.ps1 -Modules mcp,embeddings -DryRun` → [modules: mcp,embeddings] + re-review APPROVE ses_ef706bdb1ffeZC3OFHT1X9w7rG confianza: alta | artefactos: setup-embeddings.ps1, scripts/install.ps1, scripts/install.sh, README.md, docs/user/QUICKSTART.md, docs/dev/tasks/DX-12.md, commit 9e211d10 | invariantes: sin push; PS5.1 en install.ps1; defaults no-interactive intactos; secrets solo env; WIP ajeno intacto | deuda: ninguna (Lows del reviewer aplicados; nits aplicados) | queda_pendiente: docs/index.md+llms.txt regen (checkpoint, WIP ajeno); validate-docs-coverage 1 por graphrag_search (DIST-15 in-flight)
Próxima tarea si completa: DIST-17
=== END RECITATION ===

=== RECITATION 24 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-16 — verify de certificados vía MCP (VER-02): tool memory_verify_certificate + paridad CLI + docs re-baselineadas
Estado: completed
Última acción: CIERRE COMPLETO: tool memory_verify_certificate implementada (definición + memory set en 4 perfiles + arm dispatch + counts 86/48/80 re-baselineados en code/tests/docs). Tests RED→GREEN con paridad SDK byte-exacta (objeto/string/envelope) + inválidos tipados (-32009/-32002/-32602). Paquete 260/260, fmt/clippy 0, gates docs 0 gaps (48 items MCP). Review P2-01 adversarial vanta-review APPROVE (0 Critical/High/Medium; fixes Low/Nit post-verdicto re-verificados). Commits locales: 7f30e4cd (feat(mcp), 8 archivos) + 4ce3ed3c (docs(task) RESULTADO). FIND-266 (WAL VER-01) registrado en Backlog. Sin push.
Resultado: OK
Próxima acción: ninguno — DIST-16 completa. El orquestador decide la próxima tarea (DIST-17 PENDING en el plan).
Contrato: verificacion: `cargo nextest run -p vantadb-mcp --ignore-default-filter` → 260/260 ✅ · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` → 0 ✅ · `cargo fmt --check` → 0 ✅ · `pwsh scripts/validate-docs-coverage.ps1` → 48 items MCP / 0 gaps ✅ · check-links/check-docs/gen-index --check → 0 ✅ · pre-commit hooks verdes en ambos commits ✅. Nota: `campaign_verify_cmd` spawn roto en el entorno (exitCode -1 sin output, precedente ENC-01) → verificación mecánica por shell directo. · evidencia: [claim: tool disponible con resultado tipado (válido → structuredContent {valid,verification}; inválido → isError + envelope ERR-MCP-01, nunca string libre) → evidencia: mcp_tests test_mcp_memory_verify_certificate_valid_typed_and_parity + test_mcp_memory_verify_certificate_rejects_tampered_and_garbage, confianza alta] [claim: paridad CLI (mismo veredicto para el mismo certificado) → evidencia: arm llama Embedded::verify_purge_certificate (src/sdk/api/memory.rs:987), el mismo core que cmd_certificate_verify (crud.rs:572); test compara byte-exacto vs SDK; envelope objeto+string aceptado como CLI (crud.rs:563-570); confianza alta] [claim: counts re-baselineados sin stale → evidencia: rg sweep limpio + tests 80/38/37/21 + coverage 48 items + MCP.md/Core 48/Memory CRUD 10, confianza alta] [claim: WAL VER-01 fuera de scope → evidencia: FIND-266 en Backlog (HEAD 66d17014), confianza alta] [claim: review adversarial APPROVE → evidencia: ses_ef6cc10a7ffe6zQaHRYOkvdUpO (R1: 2 Low + 1 Nit cerrados post-verdicto con re-verify 260/260; 1 Nit pre-existente no tocado), confianza alta] · artefactos: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/src/config.rs, vantadb-mcp/tests/mcp_tests.rs, docs/api/MCP.md, docs/user/AI_IDES.md, docs/user/operations/EXPERIMENTAL_FEATURES.md, docs/dev/Backlog-negocio.md, docs/dev/tasks/DIST-16.md · invariantes: wal/storage/attestation/sdk memory.rs intocados; tools existentes intactas; absorbed/aliases intactos; sin símbolos públicos Rust nuevos (public-api.txt intacto); staged ajeno de DIST-15 (26 archivos) no tocado — commits con pathspec · deuda: ninguna nueva (FIND-266 documenta gap pre-existente) · queda_pendiente: bookkeeping del orquestador (plan file/recitation de fase); push diferido al owner (NUNCA ejecutado).
Próxima tarea si completa: DIST-17 (PENDING en el plan — decide el orquestador)
=== END RECITATION ===

=== RECITATION 23 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-15 - graphrag_search en bindings (Py/TS/Node/WASM) + Serialize canonico
Estado: completed
Última acción: Review P2-01 APPROVE (vanta-review) + re-verify scoped del cerrador + task file RESULTADO §7 (commit ae991966)
Resultado: ✅
Próxima acción: Orquestador: decidir mensaje de dd9d3333 (amend opcional - 0 refs al hash) y proximo task F1 (DIST-17)
Contrato: nextest graphrag 6/6 · fmt 0 · clippy root+node 0 · pytest 11 · vitest Node 3/3 + TS 4/4 · docs gates 0 gaps · reviewer APPROVE
Próxima tarea si completa: DIST-17
=== END RECITATION ===

=== RECITATION 26 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: WSM-14 — Plan de adopción npm (README + demo Transformers.js + keywords honestas)
Estado: completed
Última acción: Cerrada: 4 archivos (READMEs wasm/ts + Cargo.toml keywords/description + package.json keywords/description), smoke del artefacto publicado (pkg/README.md == tracked SHA256 23B91360, keywords en pkg/package.json, npm pack OK, TS-07 smoke-pack PASSED), gates (check-links 0, check-docs all clear, check-npm-versions OK, tsc 0, vitest 342/342, verify_changed 4/4, verify.ps1 10/10), review P2-01 vanta-review R1/R2 changes -> fixes -> R3 APPROVE, commits locales 0c3e465e + 8b7269da
Resultado: OK
Próxima acción: ninguna — task cerrada. Registro progreso/Backlog diferido: Backlog.md está siendo editado por DIST-17 en vuelo (evitar colisión); el orquestador corre progreso cuando aterrice el paralelo.
Contrato: verificacion: smoke (hash pkg/README==tracked + keywords en pkg/package.json + npm pack --dry-run + node vantadb-ts/scripts/smoke-pack.mjs PASSED) + check-links/check-docs/check-npm-versions exit 0 + verify.ps1 ALL 10 PASS ✅ | evidencia: claim: mecanismo editable desde tracked -> wasm-pack src/readme.rs + src/manifest/mod.rs (repo wasm-bindgen/wasm-pack, master) + hash idéntico en 3 rebuilds | confianza: alta; claim: números re-medidos 1.77 MB raw / ~739 KB transfer / ~31x vs Orama (23.8 KB) fechados 2026-10-04 con comando repro | confianza: alta (re-ejecutado por el reviewer); claim: review P2-01 fresh vanta-review APPROVE (R3) | confianza: alta | artefactos: vantadb-wasm/README.md, vantadb-wasm/Cargo.toml, vantadb-ts/README.md, vantadb-ts/package.json, docs/dev/tasks/WSM-14.md, commits 0c3e465e + 8b7269da | invariantes: pkg/ generado nunca hand-edit; anchors TS README (#vantadb-vs-vantadb-node-npm, #domain-sub-clients) preservadas; sin claims sin fecha+fuente; commit local sin push | deuda: demo/README.md números stale (1.35 MB/578 KB, 2026-08-30) — pre-existente, follow-up FIND-75 fuera de scope | queda_pendiente: (1) registro progreso/Backlog diferido por colisión con DIST-17; (2) demo/README.md números stale -> candidato FIND; (3) push del owner
Próxima tarea si completa: 27 (TS-10) o el que el orquestador decida
=== END RECITATION ===

=== RECITATION 27 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: TS-10 - Plan de distribucion/adopcion WASM (playground + docs-site + comparativa): plan + primer slice ejecutado en repo web + metrica con fuente
Estado: completed
Última acción: Cierre: review P2-01 APPROVE (vanta-review) + RESULTADO seccion 7; commits locales 0b868789 + b9848852 (principal) y de5e25c (web branch ts10/npm-install-card, nunca push)
Resultado: OK
Próxima acción: Orquestador: regenerar/commitear docs/index.md cuando DIST-17.md/WSM-14.md esten trackeados; correr progreso (fila Backlog TS-10 + avance); el merge del branch web lo decide el owner
Contrato: verificacion: check-links exit 0 · check-docs exit 0 · markdownlint 0 fuera master plan · validate-docs-coverage 0 gaps · web tsc+lint+build+playwright(2 passed) OK · review P2-01 APPROVE | evidencia: metrica re-medida api.npmjs.org (last-week/last-month: vantadb-wasm 232/sem 764/mes, vantadb 210/sem 548/mes, @orama/orama 1.530.732/sem) confianza alta; gap npm web/src 0 hits confianza alta; slice web commit de5e25c verificado (tsc/lint/build/e2e/visual) confianza alta; no-push verificado por reviewer (branch sin upstream, commits no en remotos) confianza alta | artefactos: docs/dev/tasks/TS-10.md · docs/dev/strategy/DISTRIBUTION.md (§2.1/§2.3/§4/§8) · web branch ts10/npm-install-card commit de5e25c · worktree ..\web-ts10 | invariantes: nunca push (ambos repos) · no tocar WSM-14/DIST-17/master plan/opencode.jsonc/WIP owner web · staged ajeno -> pathspec | deuda: secuencia §8 slices 2-5 con dueno; index regenerado diferido (nota g); progreso/avance deferido (Backlog.md bajo edicion concurrente) | queda_pendiente: orquestador corre progreso + index cuando DIST-17/WSM-14 cierren
Próxima tarea si completa: DIST-17
=== END RECITATION ===

=== RECITATION 25 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DIST-17 — test de paridad cross-language (Py/Node/WASM): escenario canónico put/search/grafo/IQL → resultados idénticos (hash/diff) + job CI dedicado
Estado: completed
Última acción: Cerrado: harness completo (fixture + 3 productores + comparador + workflow ci-bindings-parity.yml + docs). Review P2-01 APPROVE (comparador 5/5 adversariales). Commits locales 3c528df3 (changeset 13 archivos) + 6a84fc5a (task file).
Resultado: OK
Próxima acción: ninguno — orquestador: ACCEPT + siguiente task del plan
Contrato: verificacion: node dev-tools/parity-compare.mjs → PARITY OK 3/3 (hashes por step idénticos: put 6d43a8e5…, search 0fb2bacd…, graph_bfs a3f0b45c…, iql 99218974…) + tamper → exit 1 con diff; cargo nextest run --profile audit --workspace --build-jobs 2 → 3745/3745; pytest 182; vitest TS 343; actionlint exit 0; validate-docs-coverage/check-links/check-docs 0. | evidencia: (1) Py/Node/WASM idénticos en put/search/graph — target/bindings-parity/{python,wasm,node}.json + comparador [alta]; (2) WASM corre el escenario completo incluido IQL — wasm.json steps.iql + probes DISCOVERY [alta]; (3) Node sin IQL → exclusión declarada + FIND-268 (vantadb-node/src/lib.rs: 37 #[napi], rg iql = flag) [alta]; (4) comparador no vacuo — 5/5 adversariales re-ejecutados por vanta-review (ses_ef6311366ffeggkqo4m3SW757b): tamper/faltante/sin exclusión/cobertura<2/stale → exit 1 [alta]; (5) job CI válido — actionlint exit 0 + simulación local de la secuencia [alta]. | artefactos: tests/parity/scenario.json · dev-tools/parity-compare.mjs · vantadb-python/tests/test_cross_language_parity.py · vantadb-ts/src/__tests__/parity.test.ts · vantadb-node/tests/parity.test.ts · .github/workflows/ci-bindings-parity.yml · docs/api/BINDINGS_NAMESPACES.md §conformance · FIND-268/269 en Backlog · commits 3c528df3 + 6a84fc5a. | invariantes: cero cambios de producción (src/ y bindings src intactos); artefactos solo en target/ (gitignored); comparador fail-loud (artefacto faltante / step sin exclusión / cobertura <2 / scenario stale); WIP ajeno intacto. | deuda: ninguna de DIST-17; FIND-268 (superficie IQL de Node) + FIND-269 (ENOSPC ambiental suite Node) registrados. | queda_pendiente: orquestador — si TS-11 aterriza antes del merge, regenerar docs/index.md+llms.txt (gen-index); el job CI real corre en el próximo PR que toque paths de bindings.
Próxima tarea si completa: según plan (F1) — el orquestador decide
=== END RECITATION ===

=== RECITATION 29 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: TS-13 — comparación VantaDB vs Orama verificada (código + docs oficiales fechadas 2026-10-04) en la sección Why VantaDB del repo web; branch local ts13/orama-column (nunca push) + artefacto DISTRIBUTION.md §2/§8.
Estado: completed
Última acción: Cierre completo: slice web commiteado (fad5bf3: componente orama-comparison + page + i18n ES/EN, gates tsc/lint/build/playwright/visual verdes); artefacto DISTRIBUTION.md §2.1/§2.2/§2.4/§3/§8 con correcciones verificadas; task file TS-13.md con RESULTADO §7; commit principal 63b54e6e; OCR sin Critical/High; review P2-01 APPROVE (vanta-review fresh tier Fast).
Resultado: OK
Próxima acción: Ninguno (tarea cerrada). Orquestador: decisión de merge del web fad5bf3; regen index cuando TS-11.md tracked; progreso cuando Backlog libre.
Contrato: verificacion: web — npx tsc --noEmit OK · npm run lint OK · npm run build OK (Turbopack) · npx playwright test flujo-critico OK (1 passed) · visual ES+EN OK (chrome-devtools). Docs — check-links OK exit 0 · check-docs OK exit 0 · markdownlint scoped 0 issues · validate-docs-coverage 0 gaps · gen-index --check OK (index no stageado, nota h). OCR — principal .md excluido (unsupported_ext); web fad5bf3 revisado contra Rule Group TS/JS sin Critical/High/Medium. Review P2-01 APPROVE (vanta-review, fresh, tier Fast).
evidencia:
- claim: Comparativa VantaDB vs Orama publicada en branch local del repo web con celdas verificadas (Orama contra docs oficiales fechadas 2026-10-04; VantaDB contra codigo) | evidencia: web commit fad5bf3 (3 archivos, +203; orama-comparison.tsx + page + i18n ES/EN) en ts13/orama-column (worktree ..\web-ts13) | confianza: alta
- claim: Claim 'durable WAL browser' NO publicado (refutado por codigo) | evidencia: init.rs:41-48 wal_writer=None + opfs.rs:524-525 + WASM_STORAGE_REVIEW.md:89 + CRASH_MODEL.md:21; web usa 'OPFS / IndexedDB · survives reloads' + footnote snapshot-based | confianza: alta
- claim: Orama verificado 2026-10-04 (3.1.18 Apache-2.0; hybrid weighted no RRF; vector brute-force; persistencia plugin snapshot; errors code string; 23.8 KB gzip) | evidencia: registry npm + docs.orama.com + source oramasearch/orama + bundlephobia API | confianza: alta
- claim: Artefacto + registro en repo principal | evidencia: commit 63b54e6e (TS-13.md + DISTRIBUTION.md §2/§3/§8; 2 files +217/-22) | confianza: alta
artefactos: docs/dev/tasks/TS-13.md; docs/dev/strategy/DISTRIBUTION.md; C:/Users/Eros/VantaDB Proyect/web-ts13 (branch ts13/orama-column @ fad5bf3); commits: 63b54e6e (principal) + fad5bf3 (web, local).
invariantes: NUNCA push (cumplido: commits locales; web branch sin upstream). No tocado: opencode.jsonc, master plan, pipeline-state, Backlog (DIST-17), archivos TS-11, WIP design/v2. Celdas Orama con fuente+fecha; 'WAL browser' no publicado.
deuda: FINDs F1 (vantadb-wasm/README.md:262/:16 wording WAL browser vs codigo) y F2 (docs/dev/wasm/CRASH_MODEL.md:75 stale: atomicity+locks ya existen) — diferidos, requieren fila Backlog cuando DIST-17 libere la edicion. Low review L1/L2 descartados (registro en TS-13.md). Index regenerado en working tree NO stageado (referencia TS-11.md untracked).
queda_pendiente: Orquestador — (1) decision de merge/cherry-pick del commit web fad5bf3 (branch ts13/orama-column; owner); (2) regen/commit de docs/index.md cuando TS-11.md este trackeado; (3) skill progreso (fila Backlog TS-13 + avance) cuando Backlog salga de edicion concurrente.
Próxima tarea si completa: PROV-12 (Task 30) — sujeto a decisión del orquestador (WIP=3)
=== END RECITATION ===

=== RECITATION 28 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: TS-11 — Roadmap paridad sub-clientes (wiki/conversation/skills vía WASM) + primer slice ejecutado (recoverArchivedNodes WASM/TS)
Estado: completed
Última acción: Cierre completo: verify full (fmt 0 · clippy workspace 0 · nextest audit workspace 3745/3745 · validate-docs-coverage 0 gaps) + OCR spec (3 archivos; sin Critical/High) + review P2-01 vanta-review APPROVE (0 Critical/High/Medium; 15 re-ejecuciones verdes) + fixes Low aplicados + commits LOCALES 9285c1db (feat, 9 archivos +414/-29) y 799ebf1e (docs(task) cierre). Roadmap §Sub-Client Roadmap en BINDINGS_NAMESPACES.md (dependencia exacta file:line por sub-cliente + orden + criterio de promoción + re-review 2026-11-04). Slice: método wasm recover_archived_nodes + d.ts + pkg rebuild + TS flat/sub-client + tests RED→GREEN (18/18; suite 344/344) + docs consistentes. Sin push.
Resultado: OK
Próxima acción: Ninguno de esta tarea. Orquestador: (a) skill progreso/avance por convención (no ejecutado por worker: Backlog con WIP de otras sesiones); (b) push diferido al owner (Regla 7); (c) próximo ⬜ del plan: Task 30 PROV-12 (TS-13 en vuelo).
Contrato: verificacion: roadmap + slice + consistencia + sin breaking ✅ — fmt 0 · clippy workspace 0 · nextest audit workspace 3745/3745 · validate-docs-coverage 0 gaps · check-links/check-docs/gen-index 0 · check-api-docs pass (ts +2/-0) · vitest 344/344 · tsc 0 · wasm check/clippy scoped 0 | evidencia: claim: wiki slice 1 desbloqueado y ejecutado (WASM+TS, paridad Python) | evidencia: commit 9285c1db + reviewer re-ejecutó vitest 18/18 + nextest scoped recover 8/8 + claims file:line 15/15 | confianza: alta. claim: conversation/skills bloqueados con dependencia exacta | evidencia: BINDINGS_NAMESPACES.md §Sub-Client Roadmap + FIND-255 (Backlog:422) + rg SkillStore src/sdk = 0 | confianza: alta. claim: review P2-01 fresh APPROVE | evidencia: ses_ef610820dffeX7Zv07MlipvQMY | confianza: alta. artefactos: docs/api/BINDINGS_NAMESPACES.md §Sub-Client Roadmap · docs/dev/tasks/TS-11.md (RESULTADO §7 + §Review) · vantadb-wasm/src/lib.rs · vantadb-ts/src/vantadb.ts · OCR spec: C:/Users/Eros/AppData/Local/Temp/opencode/ocr-ts11.json. invariantes: sin breaking changes; core (sdk/storage/wiki/skills) intacto; D42/D43 extendidas no reemplazadas; WIP ajeno (master plan, DISTRIBUTION.md, opencode.jsonc, TS-13.md) fuera del commit; sin push. deuda: ninguna de contenido; observaciones documentadas: lint pre-existente core wasm-target (src/index/serialize/file.rs:146 drop_non_drop — no gateado en CI; routing FIND sugerido sin editar Backlog) + duplicación 3L normalización edges (regla de tres). queda_pendiente: (a) skill progreso/avance por orquestador (Backlog intocado por WIP de otras sesiones); (b) push diferido al owner; (c) master plan Task 28 → COMPLETED lo actualiza el orquestador (prohibido para el worker); (d) opcional: fila FIND del lint pre-existente.
Próxima tarea si completa: PROV-12
=== END RECITATION ===

=== RECITATION 32 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DESKTOP-41: smoke-test instalador NSIS+MSI en VM Windows limpia - fallback stop-condition ejecutado (checklist + evidencia parcial estatica + FIND-272; sin evidencia simulada)
Estado: completed
Última acción: Steps 1-5 completos: build (6m46s, exit 0) + evidencia estatica read-only (MSI COM + NSIS/WiX generados) + checklist ejecutable + FIND-272 + avance + gates docs verdes + OCR (0 reviewables propios) + review P2-01 APPROVE (ronda 2; fixes M1/M2/L1/L19/L230 aplicados) + 3 commits LOCALES (78bd4223 chore lockfile, d6d40461 docs, 6cc56e2c cierre). Smoke real pendiente -> FIND-272.
Resultado: OK
Próxima acción: Ninguno - task cerrada. Ejecutar FIND-272 en VM limpia (owner-assisted) con docs/dev/desktop/INSTALLER_SMOKE_CHECKLIST.md; siguiente tarea del plan: DESKTOP-43 (Task 33).
Contrato: verificacion: Fallback stop-condition del plan COMPLETO - entregables 1-4 (build + evidencia estatica read-only + checklist ejecutable + FIND-272); smoke real diferido (FIND-272) por ausencia de VM limpia; gates docs verdes (check-links/check-docs/gen-index --check/validate-docs-coverage/markdownlint exit 0)
evidencia:
  - claim: Build de instaladores exit 0 en 6m46s (cold); NSIS 12.18MB sha256 6B30B044... + MSI 16.75MB sha256 0069AB1A...
    evidencia: desktop/src-tauri/target/release/bundle/{nsis,msi}/* (hashes re-verificados por vanta-review ronda 1)
    confianza: alta
  - claim: vanta:// registrado en AMBOS instaladores (NSIS script :650-653 + MSI Registry table Software\Classes\vanta URL Protocol)
    evidencia: target/release/nsis/x64/installer.nsi + COM read-only sobre el MSI (re-ejecutado por reviewer)
    confianza: alta
  - claim: Sidecars vanta-cli.exe + vantadb-server.exe incluidos en ambos; WebView2 bootstrapper embebido (custom action condicionada)
    evidencia: installer.nsi:642-643 + main.wxs:202-217 + MSI File/Binary/CustomAction tables
    confianza: alta
  - claim: No hay VM limpia utilizable (stop condition legitima)
    evidencia: Get-VM permiso denegado sin elevacion; sin VBox/VMware/Sandbox/multipass/WSL/ISOs; sin instalacion en maquina dev (verificado por reviewer)
    confianza: alta
  - claim: Review P2-01 APPROVE con contexto fresco (ronda 1 CHANGES-REQUIRED -> fixes M1/M2/L1/L19/L230 -> ronda 2 pre-autorizado APPROVE, aplicado exactamente)
    evidencia: ses_ef5e58bb4ffeaZOBh8XSGS3Ew7
    confianza: alta
artefactos:
  - docs/dev/desktop/INSTALLER_SMOKE_CHECKLIST.md (checklist ejecutable Test A NSIS / Test B MSI)
  - docs/dev/tasks/DESKTOP-41.md (task file + RESULTADO §7 + review)
  - docs/dev/Backlog.md (FIND-272 nueva; fila DESKTOP-41 removida -> avance)
  - docs/dev/avance/activo/desktop.md (registro de cierre)
  - desktop/src-tauri/Cargo.lock (refresh mecanico, commit aparte)
invariantes: No instalar en maquina dev; no simular evidencia; no commitear binaries/ (gitignored); no tocar opencode.jsonc/master plan/pipeline-state/providers (PROV-12); commit local sin push
deuda: Smoke real en VM limpia (FIND-272) - arranque + sidecar + deep link funcional + WebView2 install-time + uninstall
queda_pendiente: FIND-272 (owner-assisted VM run) consume el checklist; siguiente tarea del plan: DESKTOP-43 (Task 33)
Próxima tarea si completa: DESKTOP-43 (Task 33)
=== END RECITATION ===

=== RECITATION 30 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: PROV-12 — Publicar wheels PyPI de providers (estrategia H-04): preparación + dry-run local + checklist owner; publish real = owner
Estado: completed
Última acción: CERRADO: changeset 04be0ec1 (17 archivos: 3 pyproject maturin + abi3 ×3 + release-providers.yml + smoke script + docs + FIND-273/274) + 8ee81ef1 (task file RESULTADO). Review P2-01 APPROVE (vanta-review fresh; M1 tag↔versión aplicado post-review; L1/L2 aplicados). Verificación: verify.ps1 ALL 10 · maturin×3+twine PASSED+smoke×3+pytest 18/17/19 · actionlint 0 · docs gates 0. OCR sin Critical/High.
Resultado: OK
Próxima acción: ninguno — orquestador: siguiente task del plan; owner: ejecutar checklist publish (PUBLISH.md §Providers) cuando toque push + tag providers-v0.5.0
Contrato: verificacion: `dev-tools/verify.ps1` ALL 10 PASS ✅ · `maturin build --release` ×3 (abi3) + `twine check` ×3 PASSED ✅ · smoke venv limpio ×3 SMOKE OK ✅ · pytest providers 18/17/19 ✅ · `actionlint` 0 (nuevo + dir completo) ✅ · gates docs 0/0/0 ✅ · review P2-01 APPROVE (ses_ef5dd5fa7ffejGRYYZxxlz3hA7) ✅. evidencia: claim=3 wheels cp311-abi3 instalables en venv limpio evidencia=SMOKE OK ×3 (import+init+store+search+list, sin red) confianza=alta · claim=lane de release correcto (pins SHA/permisos mínimos/timeouts/OIDC/skip-existing/namespace providers-v*) evidencia=actionlint 0 + red-team del review confianza=alta · claim=sin cambios de contrato de providers (PROV-04) evidencia=diff 0 en src/.pyi/tests + firmas #[pyo3(signature)] intactas confianza=alta · claim=colisión de nombres PyPI resuelta por owner evidencia=question 2026-10-04 (providers canónicos) + FIND-273 confianza=alta · claim=verify.ps1 rojo previo era ENOSPC ambiental evidencia=2/2518 StorageFull + pasan en aislamiento + re-run ALL 10 PASS confianza=alta. artefactos: providers/{openai,ollama,litellm}/{pyproject.toml,Cargo.toml,README.md}, .github/workflows/release-providers.yml, .github/scripts/provider_wheel_smoke.py, docs/dev/tasks/PROV-12.md, docs/dev/workflow/PUBLISH.md, docs/dev/operations/CI_POLICY.md, docs/dev/Backlog.md (FIND-273/274), commits 04be0ec1 + 8ee81ef1. invariantes: sin push/sin publish; contrato Python de providers intacto; publish=false intacto; WIP ajeno intacto; Cargo.lock sin churn. deuda: ninguna nueva (M1 aplicado; L3/O1/N1 documentados sin acción). queda_pendiente: publish real = owner (checklist PUBLISH.md §Providers: environments+OIDC → dispatch TestPyPI → tag providers-v0.5.0 → verificación post); FIND-273 (twins integrations rename/retire antes de F6); FIND-274 (higiene IDs campañas archivadas).
Próxima tarea si completa: orquestador decide (PROV-13 en espera / siguiente del plan)
=== END RECITATION ===

=== RECITATION 33 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DESKTOP-43 — Auto-update vía tauri-plugin-updater: defer registrado con trigger + checklist de habilitación (contrato cumplido por stop condition sancionada)
Estado: completed
Última acción: Rama defer ejecutada y cerrada: task file canónico + UPDATER_ENABLEMENT.md + ARCHITECTURE §Distribution & updates + wontfix §DEFER + avance + Backlog (fila removida) + generados; gates docs 8/8 verdes; OCR 0 reviewables; review P2-01 vanta-review APPROVE (2 rondas: fixes M1/M2/L1/L2/N1/N2/O1 + L3); commits LOCALES d1efe133 + 7e63a784
Resultado: OK
Próxima acción: Ninguna — tarea cerrada. Siguiente tarea del plan: DESKTOP-44 (Task 34, owner-assisted, depende de FIND-155); push diferido al owner (Regla 7)
Contrato: verificacion: check-links ✅ + check-docs ✅ + gen-index --check ✅ + validate-docs-coverage ✅ + check-avance-coverage ✅ + markdownlint ✅ + cargo fmt --check ✅ + OCR 0 reviewables. evidencia: (a) updater 0 refs de código (`rg tauri-plugin-updater desktop/src-tauri`) | confianza alta; (b) `gh secret list` solo RELEASE_PLZ_TOKEN (sin TAURI_SIGNING_*) | confianza alta; (c) defer registrado en wontfix.md:55-58 con trigger explícito | confianza alta; (d) checklist anclado a docs Tauri v2 + `action.yml@1deb371b` (uploadUpdaterJson) | confianza alta; (e) review APPROVE ses_ef5c370f2ffeFMnoLIDDcHstAg | confianza alta. artefactos: docs/dev/tasks/DESKTOP-43.md · docs/dev/desktop/UPDATER_ENABLEMENT.md · docs/dev/desktop/ARCHITECTURE.md · docs/dev/avance/decisiones/wontfix.md · docs/dev/avance/activo/desktop.md · docs/dev/Backlog.md · docs/index.md · llms.txt. invariantes: pipeline de releases y código intactos; opencode.jsonc/master plan/providers sin tocar; sin push. deuda: implementación del updater diferida con trigger (decisión de distribución pública); checklist lista para ejecutar.
Próxima tarea si completa: DESKTOP-44
=== END RECITATION ===

=== RECITATION DESKTOP-44 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: DESKTOP-44 — prep owner-assisted completa: fix FIND-155 (x-vanta-user-key + lente alcanzable sin URL), guion por panel + checklist S1-S7 + evidencia estática; sesión E2E owner pendiente (upstream LLM vivo)
Estado: in-progress
Última acción: Prep cerrada: TDD RED (4 fallos por razón correcta) -> GREEN 8/8 scoped; suite desktop 90/90; npm run build exit 0; gates docs 4/4 + markdownlint + OCR sin Critical/High; commits LOCALES 5a8c06bd (fix) + a674849f (docs + FIND-275) + b765dbe9 (task sync); generados excluidos (WIP ajeno); campaña queda in-progress (no se cierra)
Resultado: PARTIAL
Próxima acción: Orquestador: preguntar al owner por upstream LLM vivo y coordinar la sesión S1-S7 del task file; luego cerrar FIND-155 + campaña (review P2-01). No completar sin la sesión (stop condition: sin mocks).
Contrato: verificacion: cd desktop; npx vitest run src/components/proxy/ProxyDashboard.test.tsx ✅ 8/8 | cd desktop; npm test ✅ 90/90 | cd desktop; npm run build ✅ exit 0 | node scripts/docs/check-links.mjs ✅ | check-docs.mjs ✅ | gen-index --write/--check ✅ | validate-docs-coverage.ps1 ✅ 0 gaps | markdownlint task file ✅ | OCR 3 reviewables propios sin Critical/High. evidencia: (a) fix aplicado — fetchSnapshot envía x-vanta-user-key desde localStorage (vanta.proxy.userKey) + input password + hint 401 | ProxyDashboard.tsx + commit 5a8c06bd | confianza alta; (b) RED->GREEN real: 4 fallos por la razón correcta antes del fix, 8/8 después | ProxyDashboard.test.tsx | confianza alta; (c) lente PROXY alcanzable sin configurar (botón sidebar incondicional) | WorkspaceShell.tsx:608-612 | confianza alta; (d) guion S1-S7 + seed api05 + config TOML + control 401/200 | docs/dev/tasks/DESKTOP-44.md §Guion | confianza alta; (e) tsc 6 errores PRE-EXISTENTES en vanta-wasm-map.ts (archivo no tocado) -> FIND-275 | Backlog | confianza alta. artefactos: docs/dev/tasks/DESKTOP-44.md · docs/dev/Backlog.md (FIND-275) · commits 5a8c06bd + a674849f + b765dbe9. invariantes: opencode.jsonc/master plan/pipeline-state.json/providers sin tocar; sin push (Regla 7); auth D34 sin bypass; sin mocks como evidencia E2E. deuda: sesión E2E owner (contrato del plan); FIND-155 sigue Pendiente en Backlog hasta la validación; docs/index.md + llms.txt sin commitear (su regen incluye WIP ajeno SHOW-02/PROV-13 — el próximo cierre regenera). queda_pendiente: orquestador coordina la sesión owner; al cerrar: remover FIND-155 + review P2-01.
Próxima tarea si completa: DESKTOP-44 (sesión owner)
=== END RECITATION ===

=== RECITATION 31 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: PROV-13 — Providers OpenAI/Ollama/LiteLLM en Windows: compilación + tests verdes en Windows + job CI Windows (matriz provider × os)
Estado: completed
Última acción: Cerrada: fix de encoding en verify_pyi.py (stdout UTF-8) + providers-ci.yml con matriz ubuntu/windows (venv condicional por OS, pyi-step cross-shell via env PROVIDER, rust-cache workspaces providers/*/target, timeout 30, paths += script) + verificación local Windows (check/clippy ×3, 54/54 tests, pyi ×3, actionlint 0, verify.ps1 ALL 10) + review P2-01 APPROVE (vanta-review) + commits locales b616e97c + 078e2fe2
Resultado: OK
Próxima acción: Ninguna — tarea cerrada. Push diferido al owner (Regla 7); post-push: confirmar run de los 6 legs y anotar wall-time vs estimado 10-13min. Progreso/avance + fila Backlog PROV-13 = orquestador.
Contrato: verificacion: cargo clippy -D warnings ×3 (Windows) ✅ · pytest 54/54 ✅ · actionlint 0 ✅ · pyi ×3 ✅ · verify.ps1 ALL 10 PASS ✅ (campaign_verify_cmd ×4 exit 0). evidencia: claim=premisa 'no compilan en Windows' stale en HEAD evidencia=cargo check/clippy ×3 exit 0 repro 2026-10-05 (resuelta por PROV-01/02/04; PROV-12.md:38/129 ya lo anotaba) confianza=alta · claim=bloqueante real Windows=encoding de verify_pyi.py evidencia=UnicodeEncodeError U+2713 reproducido con cp1252/ibm437 (exit 1) vs fix exit 0 ✓ ×3 (reviewer reprodujo HEAD exit 1) confianza=alta · claim=job Windows con comandos verificados en Windows real evidencia=actionlint 0 + comandos exactos del leg ejecutados local (maturin develop --release, pytest, pyi; reviewer re-ejecutó 54/54) confianza=alta · claim=sin regresión ubuntu evidencia=diff completo (comandos preservados; solo pyi invocation + cache + rename de check fuera de rulesets — verificado por reviewer con gh api) confianza=media-alta. artefactos: .github/scripts/verify_pyi.py · .github/workflows/providers-ci.yml · docs/dev/tasks/PROV-13.md · commits b616e97c + 078e2fe2. invariantes: sin push; contrato providers intacto (0 cambios en src/.pyi/pyproject); archivos PROV-12 intactos; shared_py.rs no tocado; Cargo.lock ×3 revertidos. deuda: verde real del run GitHub = post-push (owner, Regla 7); nit stderr reconfigure diferido (documentado); ENOSPC ambiental local (no-code). queda_pendiente: progreso/avance + fila Backlog (orquestador); post-push confirmar 6 legs + wall-time vs estimado.
Próxima tarea si completa: orquestador decide (WIP: DESKTOP-44 en vuelo; SHOW-02 cerró)
=== END RECITATION ===

=== RECITATION 35 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: SHOW-02 (taskId 35) — verificar EJECUCION E2E de las 6 recetas del playground (repo ness-e/Vantadb-web) contra el WASM real; cerrar ya-resuelto con evidencia o fix puntual del delta.
Estado: completed
Última acción: Cerrada. E2E real 6/6 (dropdown->run->output, WASM): defecto stacking (5/6 items no clicables, tambien en produccion) + receta Graph BFS rota (put+node_id u128 con add_edge -> Node not found) -> fixes puntuales + guard e2e/playground-recipes.spec.ts; gates web tsc/lint/build/playwright 8/8 verdes; docs gates verdes; OCR N/A-justificado; review P2-01 APPROVE (re-ejecuto el guard: 6 passed); commits web a482da4 (LOCAL) + docs cfac5413.
Resultado: OK
Próxima acción: Ninguno (tarea cerrada). Orquestador: (1) routing FIND de la observacion sistemica Reveal+overlay fuera del playground; (2) progreso/avance + fila Backlog SHOW-02 (Backlog.md bajo edicion concurrente); (3) merge del branch web show02/recipes-clickable = decision del owner.
Contrato: verificacion: `npx playwright test` (worktree web-show02) OK 8/8 + `e2e/playground-recipes.spec.ts` OK 6/6 + `npx tsc --noEmit` OK + `npm run lint` OK + `npm run build` OK; repo principal check-links/check-docs/gen-index --check OK exit 0. evidencia: claim '6 recetas ejecutan' -> show02-evidence.json (6/6 ok, outputs por receta) + show02-shots/ + before/after show02-menu-open*.png; claim 'fix causa-raiz' -> git show a482da4 + reveal.tsx:64; claim 'sin push' -> git log origin/main..show02/recipes-clickable (solo a482da4); claim 'review' -> ses_ef5869fbbffefxEijm7HLgO9zM. artefactos: docs/dev/tasks/SHOW-02.md (commit cfac5413); web branch show02/recipes-clickable (a482da4, worktree ..\web-show02 con evidencia). invariantes: NUNCA push; no tocar design/v2 (WIP owner), branches ts10/ts13, master plan, opencode.jsonc, docs/pipeline-state.json, Backlog.md, areas PROV-13/DESKTOP-44; commit principal por pathspec explicito. deuda: observacion sistemica (patron Reveal+overlay) -> FIND del orquestador; progreso/avance deferido (Backlog.md concurrente). queda_pendiente: routing FIND + progreso + merge owner del branch web.
Próxima tarea si completa: SHOW-03
=== END RECITATION ===

=== RECITATION 36 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: Task 36 WEB-09+MKT-22 — WEB-09: recount HEAD + defer con criterio/fecha (sin inventar criterio owner); MKT-22: publicar North Star (metodología + comando + 'first measurement pending') en README
Estado: in-progress
Última acción: DISCOVERY completo: recount web HEAD (stale 73/×11/5capas no reproduce; atenuación owner 2026-09-02 ya mergeada c4e7c57; design/v2 activo) + probes 5 stores proxy (0 sesiones → stop condition) + task file canónico docs/dev/tasks/WEB-09-MKT-22.md creado
Resultado: PARTIAL
Próxima acción: Step 1: editar README.md (subsección North Star + fila MCP) → gates docs (check-links/check-docs/gen-index/markdownlint/validate-docs-coverage)
Contrato: verificacion: pendiente (gates docs tras Step 1) | evidencia: recount HEAD main fd7b41b (worktree web-web09) — trust-bar 1 token animado + 3 overlays; hero 3 capas + 1 RegMark; 81 animate-* site-wide; atenuación c4e7c57 en main | probes proxy-turns vacío en db/.vantadb-mcp/~/.vantadb/desktop×2 | self-test north_star_metric.py OK | artefactos: docs/dev/tasks/WEB-09-MKT-22.md | invariantes: sin números inventados (R11); web repo sin cambios; no tocar opencode.jsonc/master plan/pipeline-state; commit LOCAL | deuda: WEB-09 revisión 2026-11-02 o merge design/v2; MKT-22 número real pendiente | queda_pendiente: gates + review P2-01 + commit + cierre taskId 36
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION 37 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-01 — Detección de contradicción en ingesta L1 (Task 37)
Estado: completed
Última acción: Steps 1-7 completos. Commit LOCAL 10b35b41 (16 archivos): campo `contradicts` en DedupDecision (juicio de dedup existente, 0 llamadas LLM nuevas) + marcado `superseded_by` vía mark_contradiction en write_memory (choke point de ambos paths de ingesta) + test dedicado l1_contradiction.rs (11/11) + docs + índices regenerados + FIND-277. Review P2-01 vanta-review ✅ approve (Low aplicados: F2 guarda sanitizada+test, F3 2 tests, F1 doc, F4 doc, F5 nota).
Resultado: OK
Próxima acción: Ninguno — tarea completa. Siguiente del plan: Task 38 (MEMG-02) ya en vuelo en otra sesión.
Contrato: verificacion: `cargo nextest run -p vanta-memory --profile audit --build-jobs 2` ✅ 587/587 · `cargo nextest run -p vanta-memory --test l1_contradiction` ✅ 11/11 · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` ✅ 0 · `cargo fmt -p vanta-memory -- --check` ✅ 0 · docs (check-links/check-docs/gen-index/coverage) ✅ 0 | evidencia: [claim: 'la ingesta detecta y marca contradicciones explícitas (caso canónico) sin borrar el viejo' → evidencia: vanta-memory/tests/l1_contradiction.rs + commit 10b35b41, confianza alta] [claim: 'reuso de mark_contradiction, cero semántica paralela; dream/lifecycle intactos' → evidencia: vanta-memory/src/core/record/l1_writer.rs (mark_contradicted_targets) + git diff sin cambios en lifecycle/dream, confianza alta] [claim: 'review adversarial aprobado' → evidencia: sesión vanta-review ses_ef5488be8ffeQFpNJUXFP3yc3Z, confianza alta] | artefactos: docs/dev/tasks/MEMG-01.md · vanta-memory/tests/l1_contradiction.rs · docs/api/VANTA_MEMORY.md · docs/dev/Backlog.md (FIND-277) | invariantes: el viejo nunca se borra (solo superseded_by); no-contradictorios conservan store/update/merge/skip; mark_contradiction única semántica; degradación LLM-free intacta; dream intacto; cuarentena fuera de scope | deuda: cross-sesión + provenance persistente + contradicciones implícitas + consumo read-side (FIND-277) | queda_pendiente: lead — (1) asegurar commit de MEMG-02.md o regenerar índices en push-prep (docs/index.md ya incluye su fila), (2) se corrió `cargo fmt` sobre WIP ajeno no staged (src/sdk/api.rs, src/sdk/types.rs, vantadb-mcp/src/handlers/tools.rs) para desbloquear el hook — sin commitear; (3) push NO ejecutado (política owner)
Próxima tarea si completa: MEMG-02
=== END RECITATION ===

=== RECITATION 38 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-02 — Outcome loop → refuerzo de confianza post-recall (op SDK+MCP + loop vanta-memory)
Estado: completed
Última acción: 6/6 steps: RED→GREEN core (Embedded::reinforce + ReinforceOutcome; política used/corrected/unused), tool MCP memory_reinforce (4 perfiles; counts 80→81 listed / 86→87 defined), loop vanta-memory reinforce_recalled + test E2E, docs (EMBEDDED_SDK/scores/MCP) + snapshot público regenerado (absorbe drift FIND-237/DIST-15), FIND-278; review P2-01 vanta-review APPROVE (6 Low cerrados/documentados); commits locales fcc17ca7 (feat) + 782e111d (docs task).
Resultado: OK
Próxima acción: ninguno — tarea cerrada. Próximo del plan: MEMG-11 (Task 39) según orquestador.
Contrato: verificacion: `cargo nextest run --profile audit -p vantadb --lib -E 'test(/reinforce/)'` 13/13 ✅ · `-p vanta-memory` 589/589 ✅ · `-p vantadb-mcp --ignore-default-filter` 261/261 (mcp_tests 115/115) ✅ · `-p vantadb --lib` 2318/2318 ✅ · `cargo fmt --check` 0 ✅ · `cargo clippy -p vantadb -p vantadb-mcp -p vanta-memory --all-targets -- -D warnings` 0 ✅ · `validate-docs-coverage.ps1` 0 gaps ✅ · check-links/check-docs/gen-index 0 ✅ · public_api snapshot compare 1/1 ✅ | evidencia: claim 'op real de refuerzo SDK+MCP' → `src/sdk/api/memory.rs` (Embedded::reinforce) + `vantadb-mcp/src/handlers/tools.rs` (memory_reinforce) confianza alta; claim 'política declarada bump/decay acotado y saturado' → `docs/api/scores.md` §Reinforcement + rustdoc confianza alta; claim 'loop cerrado recall→refuerzo' → `vanta-memory/tests/reinforce_loop.rs` 2/2 confianza alta; claim 'métrica de selección antes/después' → tests min_confidence + abstención (src/sdk/api.rs) confianza alta; claim 'review P2-01 fresh' → vanta-review APPROVE ses_ef52142a4ffexvKGFDej3kq2vs confianza alta | artefactos: docs/dev/tasks/MEMG-02.md · vanta-memory/tests/reinforce_loop.rs · commits fcc17ca7/782e111d | invariantes: put/put_batch intactos (last_validated_at_ms=None; reinforce único writer); wire v2 sin cambios; ranking default intacto (sin ponderación); sin unwrap/expect/unsafe en producción; WIP MEMG-01 (l1_writer/l1_dedup/prompts/dream) no pisado; opencode.jsonc/master plan no stageados | deuda: calibración empírica del bump/decay excluida por contrato (VER-08/MGR-12 §6.3); consumo automático más profundo del host fuera de scope (stop-condition no disparada: loop cerrado con helper) | queda_pendiente: skill progreso / registro en docs/dev/avance + limpieza fila Backlog MEMG-02 (orquestador); FIND-278 ya en Backlog; push solo con instrucción del owner.
Próxima tarea si completa: 39
=== END RECITATION ===

=== RECITATION 40 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-12 — Semántica v2 write-side en el pipeline: L1 escribe confidence/valid_at reales en los dos sitios (extracción + promoción) vía punto único; TTL + derived/T1b/T1c → FIND (stop 1.5sem)
Estado: completed
Última acción: 4/4 steps. RED 4/4 → GREEN 7/7 (l1_semantics_v2: store/merge/promotion/batch/round-trip/fallback/re-persist contradicción); suite crate 602/602; fmt/clippy 0; gates docs 0; coverage 0 gaps; FIND-279/280/281; review P2-01 vanta-review APPROVE (3 Low, 2 cerrados + 1 FIND); commits LOCALES 9d0e371d + 9ca40219 + 8b71fb89 (sin push). Coordinación MEMG-11: su refactor en vuelo integró el estampado v2 en record_input (builder compartido) — co-autoría documentada en el commit.
Resultado: OK
Próxima acción: Ninguno — task cerrada. El orquestador decide la próxima (MEMG-13 Task 41 o resumir MEMG-11, que sigue en vuelo en el mismo worktree sobre memory.rs/auto_recall.rs/tests).
Contrato: verificacion: cargo nextest run --profile audit -p vanta-memory --test l1_semantics_v2 --build-jobs 2 → 7/7 ✅ · cargo nextest run --profile audit -p vanta-memory --build-jobs 2 → 602/602 ✅ · cargo fmt --all -- --check → 0 ✅ · cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings → 0 ✅ · evidencia: claim='valid_at_ms = nacimiento del contenido en ambos sitios L1' evidencia=vanta-memory/src/core/record/l1_writer.rs (record_input/put_record) + tests l1_semantics_v2 confianza=alta · claim='round-trip export→import preserva v2' evidencia=test pipeline_write_roundtrips_v2_fields_via_export_import confianza=alta · claim='review fresco APPROVE' evidencia=sesión ses_ef4f06c1bffeupwI1MGt7CCS6t (vanta-review) confianza=alta · artefactos=docs/dev/tasks/MEMG-12.md + vanta-memory/tests/l1_semantics_v2.rs · invariantes='wire v2 y payload intactos; valid_at>0 (nunca Some(0)); put_record punto único; cuarentena sticky I2 no tocada; sin símbolos públicos nuevos' · deuda='TTL semántico (FIND-279, MGR-09); derived/T1b/T1c (FIND-280); re-persist contradicción resetea confidence (FIND-281, pre-existente)' · queda_pendiente='coordinar con MEMG-11: l1_writer.rs quedó commiteado con su refactor integrado (base 9d0e371d); sus cambios restantes (memory.rs/auto_recall.rs/tests) siguen sin commitear'
Próxima tarea si completa: MEMG-13
=== END RECITATION ===

=== RECITATION 39 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-11 — Adopción del motor core en vanta-memory (recall híbrido detrás de flag + escrituras put_batch + A/B documentado)
Estado: completed
Última acción: 5/5 steps. (a) RecallConfig.core_search (default false) + search_records_core (BM25+HNSW+RRF; D38 siempre-hybrid; ACL/cuarentena/scope D22) + tests (7/7 fixture). (b) apply_dedup_batch→put_batch (plan_write/record_input) + fix core de índices de put_batch (full-rebuild O(total)→replace_derived_indexes O(batch); DUR-03 con write-lock único + variantes _locked) + tests UPSERT/mixed-live-expired. A/B: escritura 2.36–2.71×; recall 457.25→41.53 ms/q (~11×); canonical_p99 run limpio p50=3.201/p95=4.5415/p99=5.7773 ms vs MGR-19 §1 (path ajeno al diff; varianza de instrumento). Review P2-01 adversarial vanta-review: r1 ❌ (1C+1H+2R) → r2 ❌ (deadlock del fix de guard, cazado con repro) → r3 ✅ APPROVE. Commit local 508e211e.
Resultado: OK
Próxima acción: ninguno — tarea cerrada. Próximo del plan (F2): MEMG-13 (Task 41) según orquestador.
Contrato: verificacion: `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` 605/605 ✅ · `cargo nextest run -p vantadb --lib -E 'test(put_batch) or test(incremental)'` 12/12 ✅ · sweep `-p vantadb --ignore-default-filter -E 'test(put_batch)'` 10/10 ✅ · edge_cases mixed/ttl/racing 3/3 ✅ · `cargo fmt -p vantadb -p vanta-memory -- --check` 0 ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` 0 ✅ · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` 0 ✅ · docs gates (check-links/check-docs/gen-index) 0 ✅ · `campaign_verify_cmd` no operativo en este entorno (no spawnea cargo, exit -1) → verificación mecánica vía shell | evidencia: [claim 'recall híbrido detrás de flag con dual-pool preservado' → evidencia: vanta-memory/src/core/hooks/auto_recall.rs + tests/recall_core_hybrid.rs 7/7, confianza alta] [claim 'escrituras del pipeline vía put_batch con paridad batch vs secuencial' → evidencia: vanta-memory/tests/l1_batch_write.rs + commit 9d0e371d (l1_writer.rs), confianza alta] [claim 'fix core put_batch: O(batch) con consistencia list/count/text/UPSERT y DUR-03 cerrado' → evidencia: src/sdk/api/memory.rs + tests core 12/12 + edge_cases 3/3 + review r3, confianza alta] [claim 'A/B medido reproducible' → evidencia: docs/dev/tasks/MEMG-11.md §A/B, confianza alta] [claim 'review P2-01 fresh APPROVE' → evidencia: vanta-review ses_ef4d5658cffesIYmJtQxXsCY0r r3, confianza alta] | artefactos: docs/dev/tasks/MEMG-11.md · vanta-memory/tests/recall_core_hybrid.rs · vanta-memory/tests/l1_batch_write.rs · commit 508e211e | invariantes: record sin vector nunca se dropea (ambos paths, aun mode Embedding); core_search=false byte-identical; cuarentena/ACL/scope intactos; put_batch consistencia list/count/text/UPSERT + DUR-03 (purge_lock→engine); v2 MEMG-12 intacto (record_input); sin unwrap/unsafe nuevos | deuda: dedup recall_candidates al motor core; query_sparse (L1 sin sparse hoy); BENCH-02/VER-08 (calidad BEIR/LongMemEval, F5) → fixture propio, no claim calibrado | queda_pendiente: orquestador — (1) registrar 3 FINDs propuestos (governance.source sin hits; BM25 sobre payload JSON; angostar sección crítica put_batch) en Backlog; (2) skill progreso/avance + limpieza fila Backlog MEMG-11; (3) índices docs regenerados incluyen MEMG-12.md (ya commiteado en 9d0e371d — OK); (4) push NO ejecutado (política owner).
Próxima tarea si completa: 41
=== END RECITATION ===

=== RECITATION 41 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-13 — superficies core restantes en memoria: consumir versions (historia/diff L1) + snapshot (backup/restore) con tests por superficie; IQL/filtros evaluados con motivo + FIND (stop 1sem L1187)
Estado: completed
Última acción: Steps 1-4 completos: versions (l1_history 2 tests, 3 versiones reales + interior) + snapshot (backup_snapshot 4 tests, Fjall round-trip + canary FIND-287) + FIND-285/286/287 + docs (VANTA_MEMORY.md, Backlog, index/llms) + verify full (suite 611/611, fmt/clippy 0, docs gates 0) + OCR sin Critical/High + review P2-01 vanta-review APPROVE post-fix + 3 commits locales (eb842343 feat, aa9e6843 docs-task, bed44b23 docs-avance) + avance 1034/1034
Resultado: OK
Próxima acción: Ninguno — task completa. El orquestador decide la próxima (sugerida: MEMG-07, Task 42).
Contrato: verificacion: cargo nextest run --profile audit -p vanta-memory --build-jobs 2 => 611/611 passed, 2 skipped ✅; contrato por binario l1_history 2/2 + backup_snapshot 4/4 ✅; cargo fmt --all --check 0 ✅; cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings 0 ✅; check-links/check-docs/gen-index --check/validate-docs-coverage/check-avance-coverage 0 ✅; review P2-01 vanta-review APPROVE (2 rondas: changes-required -> fix -> approve). evidencia: (1) claim 'versions consumido con historia direccionable' evidencia vanta-memory/tests/l1_history.rs (3 versiones reales, get_version interior, 99->None) confianza alta; (2) claim 'backup/restore delegado sin reimplementar' evidencia vanta-memory/src/utils/backup.rs + tests/backup_snapshot.rs (InvalidInput/NotFound/round-trip) confianza alta; (3) claim 'FIND-287 real: restore data/-only' evidencia src/storage/engine/mod.rs:795-859 + canary test + FIND-33.md:53 + plan archivado 2026-08-29 §1308 confianza alta; (4) claim 'IQL/filtros no consumidos con motivo' evidencia Backlog FIND-285/286 + §Spec #7/#8 confianza alta. artefactos: docs/dev/tasks/MEMG-13.md, docs/dev/Backlog.md (FIND-285/286/287), docs/api/VANTA_MEMORY.md, docs/dev/avance/activo/vanta-memory.md. invariantes: firmas existentes intactas (read_session_records fan-in 38); wire/payload L1 intacto; gate cuarentena recall intacto (SCH-05); src/sdk/**/src/storage/** sin tocar; sin deps; sin unwrap/unsafe en produccion. deuda: FIND-285 (IQL), FIND-286 (filtros restantes), FIND-287 (core/storage restore backend) — registradas. queda_pendiente: el orquestador registra el outcome SDP; avance ya migrado.
Próxima tarea si completa: MEMG-07
=== END RECITATION ===

=== RECITATION 43 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-20 — API mínima de checkpoint de tarea (dim 1) reanudable en vanta-memory; tipos/namespace separados del pipeline checkpoint; test interrupción→resume sin repetir pasos
Estado: completed
Última acción: Steps 1-4 completos: API `utils::task_checkpoint` + 9 tests (incl. resume Fjall close/reopen) + suite crate 620/620 + fmt/clippy/doctest + docs (VANTA_MEMORY §Task checkpoints + FIND-288) + OCR sin Critical/High + review P2-01 vanta-review APPROVE (2 Optional doc-only aplicados) + 3 commits locales + cierre avance/Backlog (coverage 1034/1034, 0 gaps)
Resultado: OK
Próxima acción: ninguno (tarea cerrada). Orquestador: continuar con la próxima task del plan (Task 44 MEMG-21 o lo que decida; MEMG-07 Task 42 sigue en vuelo)
Contrato: verificacion: `cargo nextest run --profile audit -p vanta-memory --test task_checkpoint --build-jobs 2` ✅ 9/9 · suite `-p vanta-memory` ✅ 620/620 (2 skipped) · `cargo fmt --check` ✅ · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` ✅ · `cargo test --doc -p vanta-memory` ✅ 2/2 · docs gates ✅ 0 · avance-coverage ✅ 1034/1034 · evidencia: contrato L1241 cumplido (a separación pineada raw-SDK; b manager RMW; c resume test executed==[2,3,4]; d suite sin regresión) · artefactos: vanta-memory/src/utils/task_checkpoint.rs, vanta-memory/tests/task_checkpoint.rs, docs/dev/tasks/MEMG-20.md, docs/api/VANTA_MEMORY.md, docs/dev/Backlog.md (FIND-288), docs/dev/avance/activo/vanta-memory.md · invariantes: tipos/namespace separados del pipeline; firmas CheckpointManager intactas; sin locks/deps nuevas; core src/sdk/** no tocado · deuda: ninguna (FIND-288 = consumo por el host, plan stop L1243) · queda_pendiente: nada para el orquestador salvo decidir próxima task
Próxima tarea si completa: ninguno (lo decide el orquestador)
=== END RECITATION ===

=== RECITATION 42 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-07 — Forgetting curves sobre L1: curva real por tipo/edad configurable + pass read-only + métrica; la curva deprioriza, nunca purga.
Estado: completed
Última acción: Steps 1-4 completos: RED→GREEN (11 unit + 4 integración), verify full scoped verde, OCR sin hallazgos, review P2-01 adversarial APPROVE (NITs doc-only aplicados), commits locales 7e84244f + a9c778e5.
Resultado: OK
Próxima acción: ninguno — MEMG-07 cerrada. Orquestador: MEMG-21 (Task 44) puede arrancar (dependencia dura cumplida).
Contrato: verificacion: cargo nextest run --profile audit -p vanta-memory --test forgetting_curve --build-jobs 2 → 4/4 ✅ · suite -p vanta-memory → 635/635 (2 skipped pre-existentes) ✅ · clippy -p vanta-memory --all-targets --all-features -- -D warnings ✅ · fmt -p vanta-memory --check ✅ · gates docs (check-links/check-docs/gen-index/coverage) 0 ✅ | evidencia: (1) curva por tipo/edad configurable con parámetros declarados testeados con valores conocidos (retención exacta 0/1/2 half-lives; defaults pinneados; report 15/6/9) — lifecycle.rs inline + tests/forgetting_curve.rs 4/4 [alta]; (2) pass read-only idempotente y nunca-borra (payloads byte-idénticos; registros intactos) — forgetting_curve.rs:111-153 [alta]; (3) review P2-01 adversarial APPROVE 0C/0H/0M — ses_ef4078f35ffeknsGryHe2RWk8s [alta]; (4) FIND-289 registrado (descarte automático + scheduler → WIRE-15) — Backlog.md:448 [alta] | artefactos: docs/dev/tasks/MEMG-07.md · commits locales 7e84244f (feat) + a9c778e5 (docs task) · VANTA_MEMORY.md §Forgetting curve | invariantes: la curva jamás borra ni muta; MemoryRecord sin campos nuevos (wire intacto); bump_heat/decay_heat/is_prune_eligible/PRUNE_HEAT_THRESHOLD intactos; sin unwrap/unsafe; commit local (sin push); WIP ajeno no stageado | deuda: descarte automático + wiring del pass a scheduler → FIND-289 (Backlog) | queda_pendiente: skill progreso (registro avance + fila Backlog) por el ejecutor; MEMG-21 (Task 44) desbloqueada — consume retention_factor
Próxima tarea si completa: MEMG-21 (Task 44) — Scoring multi-señal L1 (recencia + importancia)
=== END RECITATION ===

=== RECITATION 44 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-21 (Task 44): scoring compuesto L1 (recencia + relevancia + importancia, opt-in byte-idéntico) + reflexión periódica sobre episódica (lecciones, L1 intacto); métrica before/after; pesos declarados (CrewAI 0.5/0.3/0.2).
Estado: completed
Última acción: 6/6 steps: scoring.rs (13 unit) + integración recall (rrf_merge_scored + perform_auto_recall_scored + inner, 2 unit + 34/34 anti-regresión) + métrica before/after (4 integración) + reflection/mod.rs (5 unit + 5 integración) + docs (VANTA_MEMORY §Composite scoring/§Reflection, FIND-290) + verify full (suite -p vanta-memory 664/664, 2 skipped; fmt/clippy OK; gates docs 0) + OCR Group 3 sin Critical/High + review P2-01 adversarial APPROVE (0C/0H/0M, NIT-1/NIT-2 doc-only aplicados) + commits locales 3fd223ab (feat) + b3f8df68 (docs task).
Resultado: OK
Próxima acción: Ninguno para MEMG-21 — tarea cerrada. Orquestador: continuar con la próxima task del plan; queda_pendiente skill progreso (avance + fila Backlog) si no lo corre el orquestador.
Contrato: verificacion: cargo nextest run --profile audit -p vanta-memory --test composite_scoring --test reflection --build-jobs 2 → 9/9 ✅ · suite -p vanta-memory → 664/664 (2 skipped pre-existentes) ✅ · anti-regresión recall/recall_core_hybrid/semantic_recall/e2e_flow → 34/34 (1 skip) ✅ · cargo fmt --check ✅ · cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings ✅ · gates docs (check-links/check-docs/gen-index --check/validate-docs-coverage) 0 ✅ | evidencia: (1) scoring compuesto opt-in con fórmula Park §4.1 + defaults CrewAI citados; recencia = retention_factor de MEMG-07 consumido; importancia = priority; min-max sobre candidatos — scoring.rs + tests/composite_scoring.rs 4/4 [alta]; (2) métrica de ordenamiento before [r1,r2,r3,r4] → after [r2,r1,r3,r4], rank 2→1; neutralidad con señales uniformes; legacy byte-idéntico con None (11 callers intactos) — composite_scoring.rs + auto_recall.rs:715-725 [alta]; (3) reflexión determinista con L1 byte-idéntico (payloads) + precondición NotEnoughMaterial + runner opcional — reflection/mod.rs + tests/reflection.rs 5/5 [alta]; (4) review P2-01 adversarial APPROVE 0C/0H/0M — ses_ef3ccac26ffe1RJqOfDJL3yXtP [alta]; (5) FIND-290 registrado (activación + promotion + wiring) — Backlog.md:447 [alta]. | artefactos: docs/dev/tasks/MEMG-21.md · commits locales 3fd223ab (feat) + b3f8df68 (docs task) · docs/api/VANTA_MEMORY.md §Composite scoring/§Reflection · FIND-290. | invariantes: default None byte-idéntico (ningún caller cambia comportamiento); reflexión jamás muta L1 (solo reflection/<s>/<run_id>); sin campos nuevos en wire (DecayPolicy solo Eq aditivo); retention_factor consumido sin reimplementar; sin unwrap/unsafe en producción; sin deps nuevas; commit local (sin push); WIP ajeno no stageado. | deuda: activación del scoring en hosts + promotion de lecciones a L1 + wiring del pase de reflexión → FIND-290 (Backlog). | queda_pendiente: skill progreso (registro avance + fila Backlog) si el orquestador no lo corre; nada más.
Próxima tarea si completa: lo decide el orquestador
=== END RECITATION ===

=== RECITATION 45 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: WIRE-14 (T1): seam de host aditivo en el arranque — inyección de ConversationTrigger + servicio de fondo con parada graceful (ADR-0054 T1)
Estado: completed
Última acción: Steps 1-4 completos: RED→GREEN (6 tests del seam), verify scoped verde (flake de carga documentado + aislado verde), OCR 0C/0H/0M, review P2-01 vanta-review APPROVE, commit local b5d294d2 (feat) + 4448788b (docs task). Incidente: el cierre de MEMG-21 barrió mis archivos staged con un add -A concurrente (283e23a1) — resuelto con split local content-preserving (e916c5be docs MEMG-21 + b5d294d2 feat WIRE-14).
Resultado: OK
Próxima acción: ninguno — WIRE-14 cerrada. Orquestador: WIRE-15 (Task 46) puede arrancar (dependencia dura cumplida; superficie ServerHooks/BackgroundService/run_with_hooks lista y estable).
Contrato: verificacion: cargo nextest run --profile audit -p vantadb --features server --build-jobs 2 → 2676 run: 2675 passed, 4 skipped, 1 timeout = flake de carga pre-existente documentado (concurrent_insert_preserves_hnsw_invariants; re-corrido aislado: 1 passed 52.8s) · focused 6/6 · clippy -p vantadb --features server --all-targets -- -D warnings exit 0 · cargo check -p vantadb (sin server) exit 0 · fmt paths propios exit 0 · doc exit 0 (14 warnings pre-existentes, ninguno del seam). | evidencia: (1) RED real E0432/E0425 por símbolos ausentes (log wire14-red.log); (2) seam aditivo: run intacto (delega con ServerHooks::default()), ServerState 17 literales intactos, defaults inertes pinneados; (3) review P2-01 vanta-review APPROVE 0C/0H/0M (ses_ef3d02493ffe7XkwWw6D0BtiMW); (4) commit b5d294d2 (feat, 6 archivos, pre-commit hook verde: fmt+clippy+actionlint). | artefactos: docs/dev/tasks/WIRE-14.md · commits b5d294d2 + 4448788b (+ e916c5be split MEMG-21) · logs $env:TEMP/wire14-*.log | invariantes: run(config) firma/semántica intactas; ServerState sin cambios; join graceful post-loop (Drop best-effort en early-exit, documentado); sin unwrap/unsafe/deps nuevas; WIP ajeno no stageado; commit local (sin push) | deuda: ninguna del seam (wiring productivo = WIRE-16; docs = WIRE-17). Low del review: Drop best-effort obligatorio para la impl de WIRE-15 (documentado); joins secuenciales sin timeout aceptados con ≤2 servicios | queda_pendiente: skill progreso (registro avance + fila Backlog) por el ejecutor; WIRE-15/16 consumen la superficie
Próxima tarea si completa: WIRE-15 (Task 46) — servicio scheduler en vanta-memory (run_pass + loop con shutdown)
=== END RECITATION ===

=== RECITATION 46 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: WIRE-15 (T2): servicio scheduler en vanta-memory — run_pass (timers + worker + reclaim) + loop helper feature-gated con shutdown graceful/Drop/BackgroundService (ADR-0054 T2)
Estado: completed
Última acción: Steps 1-4 completos: RED real (E0432/E0433) → GREEN (scheduler.rs + 8 tests) → verify scoped 676/676 + default 668/668 + fmt/clippy 0 → OCR 0C/0H/0M → review P2-01 vanta-review APPROVE (0C/0H/0M; 3 Lows + 3 NITs aplicados/dispuestos) → commits locales 1d5e1697 (feat, 9 archivos, hook verde) + 9c881047 (docs). 2 correcciones diagnósticas: E0283 de inferencia del factory `None` y `#[cfg]` faltante en spawn_memory_scheduler detectado por el gate default (cargo check sin features).
Resultado: OK
Próxima acción: ninguno — WIRE-15 cerrada. Orquestador: WIRE-16 (Task 47) puede arrancar (dependencia dura cumplida; superficie run_pass/MemoryScheduler/spawn_memory_scheduler + BackgroundService lista y estable).
Contrato: verificacion: cargo nextest run --profile audit -p vanta-memory --features http-server --build-jobs 2 → 676 run: 676 passed, 2 skipped (8 nuevos) ✅ · default → 668 passed, 2 skipped ✅ · cargo check -p vanta-memory ✅ · fmt --check ✅ · clippy --all-targets (feature y default) -D warnings ✅ | evidencia: (1) RED real E0432 services::scheduler (log wire15-red.log) + E0433 tokio en scheduler_loop (consola) [alta]; (2) pass reusa TimerScanner::run_once + PipelineWorker::run_once/reclaim_stale + MemoryTaskHandler (cero lógica nueva) — vanta-memory/src/services/scheduler.rs:79-125 [alta]; (3) Drop best-effort send+abort + BackgroundService UFCS (handoff WIRE-14 Low-1) — scheduler.rs:157-181 [alta]; (4) factory Fn() -> Option<R> por pass; None → skip debug, cola intacta — tests/scheduler_loop.rs:141-171 [alta]; (5) review P2-01 vanta-review APPROVE 0C/0H/0M ses_ef3993124ffeeYg54pY1Q21uYC [alta]; (6) commits locales 1d5e1697 (feat) + 9c881047 (docs) [alta] | artefactos: docs/dev/tasks/WIRE-15.md · commits 1d5e1697 + 9c881047 · log $env:TEMP/wire15-red.log | invariantes: nada existente cambió (run_bridge_pass/PipelineWorker/TimerScanner/LocalStateBackend intactos); default build sin uso de Tokio (dep opcional inactiva); P4 skip sin runner; parada graceful + Drop; single-writer documentado; sin unwrap/unsafe; WIP ajeno no stageado; commit local (sin push) | deuda: ninguna del servicio (wiring productivo = WIRE-16; docs = WIRE-17; timers L2/Dream sin productor por diseño) | queda_pendiente: skill progreso (registro avance) por el ejecutor; WIRE-16 consume la superficie
Próxima tarea si completa: WIRE-16 (Task 47) — wiring del wrapper vantadb-server (queue + bridge + loop)
=== END RECITATION ===

=== RECITATION 47 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: WIRE-16 (T3): wiring del wrapper vantadb-server (queue + bridge + loop) — consume superficies WIRE-14/15 (ADR-0054 T3)
Estado: completed
Última acción: Steps 1-4 completos: RED real (E0433/E0432 e2e + E0609×4 core) → GREEN (seam aditivo on_storage_ready + módulo scheduler + main wiring + 4 e2e + unit tests) → verify 7 gates verdes (12/12 + 39/39 + 2678/2678 + check default + fmt + clippy×2 + delta build 6.9s) → colateral bloqueante ajeno resuelto (vantadb-mcp E0063 core_search: HEAD no compilaba) → OCR 0C/0H/0M → review P2-01 vanta-review APPROVE (Low-1/Optional-1/NITs aplicados; Low-2 → FIND-291) → commits locales c05cd77d + 679b3545 + ac012bd6.
Resultado: OK
Próxima acción: ninguno — WIRE-16 cerrada. Orquestador: WIRE-17 (Task 48, docs del scheduler) queda habilitada (dependencia dura post WIRE-16 cumplida). Nota: el colateral mcp (c05cd77d) desbloquea el build de default-members — considerar push/CI cuando el owner lo autorice.
Contrato: verificacion: cargo nextest run --profile audit -p vantadb-server --build-jobs 2 → 12/12 ✅ · bins excluidos --test e2e --test server --ignore-default-filter → 39/39 ✅ · cargo nextest run --profile audit -p vantadb --features server --build-jobs 2 → 2678 passed, 4 skipped ✅ · cargo check -p vantadb (default) ✅ · fmt ✅ · clippy ×2 ✅ · docs gates ×3 exit 0 ✅ | evidencia: (1) incógnita real resuelta con evidencia dura — segundo open mismo-proceso = DatabaseBusy (tests/storage/multi_process_lock.rs:14-52) + sharing exige el MISMO handle Embedded (builder.rs:14-18,51-59) → ServerHooks::on_storage_ready (src/server/state.rs) [alta]; (2) 4 e2e del contrato verdes (vantadb-server/tests/scheduler_e2e.rs: POST→L0→pass→l1 vía run_with_hooks; restart L0/L1 + cola efímera; disabled sin loop; sin runner skip P4) [alta]; (3) colateral HEAD roto: vantadb-mcp E0063 core_search (gap verify scoped MEMG-11) → fix c05cd77d [alta]; (4) review P2-01 vanta-review APPROVE 0C/0H/0M ses_ef35294d8ffeEQOowuvrmlAMap (Low-1/Optional-1/NIT-1/NIT-3 aplicados; Low-2 → FIND-291; Optional-2/NIT-2 dispensados) [alta]; (5) commits locales c05cd77d + 679b3545 + ac012bd6 [alta] | artefactos: docs/dev/tasks/WIRE-16.md · commits c05cd77d/679b3545/ac012bd6 · logs $env:TEMP/wire16-*.log | invariantes: run/run_with_hooks firma+comportamiento intactos; ServerState sin cambios; single-writer (1 open/1 handle/1 loop); P4 (sin runner → skip, cola intacta); secrets solo env (R-5); sin unwrap/unsafe en producción; WIP ajeno no stageado; commits locales (sin push) | deuda: FIND-291 (flush→join, Low) + Optional-2/NIT-2 dispensados con motivo; docs del scheduler = WIRE-17 | queda_pendiente: skill progreso (registro avance) por el ejecutor; WIRE-17 habilitada; push del colateral mcp cuando el owner autorice
Próxima tarea si completa: WIRE-17 (Task 48) — docs del scheduler (wired status + promoción del rol)
=== END RECITATION ===

=== RECITATION 48 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: WIRE-17 (T4, ADR-0054): docs del scheduler cableado — wired status en VANTA_MEMORY.md §Operational modules + promoción del rol + fila vanta-memory en EXPERIMENTAL_FEATURES.md (post WIRE-16 ✅)
Estado: completed
Última acción: Steps 1-4 completos + cierre: 6 ediciones VANTA_MEMORY.md + 6 filas + revision note EXPERIMENTAL_FEATURES.md → 4 gates docs verdes + markdownlint 0 → OCR (docs excluidos por diseño) → review adversarial P2-01 vanta-review APPROVE (Low-1 aplicado; Low-2 dispensa+coordinación; NIT-1 aplicado; NIT-2 dispensado; FIND-292) → commits locales 75bd5da8 (docs, 6 archivos pathspec) + 20199096 (cierre) + 7a6a2ab9 (avance: operaciones + fila Backlog removida). skill progreso Trigger 1 ejecutado (check-avance-coverage 1034/1034 ✅).
Resultado: OK
Próxima acción: ninguno — WIRE-17 cerrada y registrada. Orquestador: WIRE-18 (Task 49) en vuelo en otra sesión; coordinar push de la ola (WIRE-18.md debe commitearse antes de pushear — Low-2 del review).
Contrato: verificacion: gates docs exit 0 — check-links ✅ · check-docs ✅ · gen-index --check ✅ · validate-docs-coverage ✅ · check-avance-coverage 1034/1034 ✅ · markdownlint 0 (3 archivos) · pre-commit hook verde en los 3 commits | evidencia: (1) wired status exacto vs código HEAD [alta] — vantadb-server/src/scheduler.rs:124-146, main.rs:69-91, state.rs:140-158, bootstrap.rs:351-353; 12 claims verificados 1-a-1 por el reviewer; (2) promoción acotada sin inflación [alta] — ADR-0054:100-104; labs se mantiene; MCP/proxy/dream diferidos; (3) review adversarial APPROVE ses_ef32d1dc9ffeB0W2AjzvwoINs0 [alta]; (4) commits locales sin push 75bd5da8/20199096/7a6a2ab9; WIP ajeno intacto [alta]; (5) FIND-292 + registro avance [alta] — Backlog.md + avance/activo/operaciones.md. | artefactos: docs/api/VANTA_MEMORY.md · docs/user/operations/EXPERIMENTAL_FEATURES.md · docs/dev/tasks/WIRE-17.md · docs/dev/Backlog.md (FIND-292; fila WIRE-17 removida) · docs/dev/avance/activo/operaciones.md · docs/index.md · llms.txt · commits 75bd5da8/20199096/7a6a2ab9 | invariantes: sin promesa de más (solo el rol; labs; MCP/proxy/dream diferidos); wired status exacto; claims mapean a código; WIP ajeno no stageado; gates en los mismos commits | deuda: ninguna propia; FIND-292 (strategy docs stale → GOV-05) + Low-2 (coordinar commit de WIRE-18.md antes del push) | queda_pendiente: push diferido al owner (política git)
Próxima tarea si completa: WIRE-18 (Task 49) — ya en vuelo; luego F3 del plan
=== END RECITATION ===

=== RECITATION 49 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: WIRE-18 — T5: verificación adversarial del scheduler (crash mid-pass + restart)
Estado: completed
Última acción: Steps 1-4 completos: vanta-memory/tests/scheduler_crash.rs (2 tests: crash mid-pass+restart sobre Fjall real + reclaim in-process) con RED-equivalente por mutación (skip→store → FAIL left:2/right:1, revertido) · verify scoped (fmt, clippy ×2, suites 670/670 y 678/678, check) · OCR (0C/0H/0M) · review P2-01 vanta-review APPROVE (Low-1 full-record aplicado; FIND-293 registrada) · commits LOCALES 95cdb2ce + b65ae6d0
Resultado: OK
Próxima acción: ninguno (tarea cerrada). Próximo del plan a criterio del orquestador: Task 50 (MEMG-03, ⬜ PENDING)
Contrato: verificacion: `cargo nextest run --profile audit -p vanta-memory --test scheduler_crash --build-jobs 2` → 2/2 ✅ · suites `-p vanta-memory` 670/670 (2 skip pre) y `--features http-server` 678/678 (2 skip pre) ✅ · fmt/clippy/check exit 0 ✅. evidencia: (1) crash mid-pass determinista (hook panic, no timing) → DB íntegra post-restart (L0/L1 full-record idénticos) — vanta-memory/tests/scheduler_crash.rs:173; (2) re-entrega sin doble procesamiento (dedup skip) — :270-289; (3) cola efímera + re-encolado desde L0 — :250-268; (4) reclaim tras lease — :294-347; (5) review APPROVE — ses_ef3236a90ffeBmiBYAni4QGHmX. artefactos: vanta-memory/tests/scheduler_crash.rs, docs/dev/tasks/WIRE-18.md, docs/dev/Backlog.md (FIND-293). invariantes: test-only (0 producción); crash hook determinista; cola efímera = semántica aceptada (no inventar re-enqueue automático); WIP ajeno sin stagear. deuda: FIND-293 (crash del loop completo — test de resiliencia/reclaim en tick posterior). queda_pendiente: anomalía tool `campaign_verify_cmd` (exitCode -1 sin output en 0.3s — spawn roto; verificación por shell; revisar en el server).
Próxima tarea si completa: MEMG-03 (Task 50) — a criterio del orquestador
=== END RECITATION ===

=== RECITATION 51 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-06 (Task 51): spill a disco del contenido compactado (payload antes del stub) + recall por id/sesion, opt-in en el worker, GC reutilizado
Estado: completed
Última acción: Steps 1-5 completos: (1) engine SpillSink + assemble_inner + assemble_with_recall +1 param (assemble intacto); (2) context_engine/spill.rs (SpillStorage spill/recall/recall_session/reclaim + DbSpillSink + FNV keys); (3) worker opt-in ContextAssemblyConfig.spill_enabled default false + DbSpillSink cableado + log spilled; (4) docs/api/VANTA_MEMORY.md + FIND-294/295; (5) verify full + OCR delegation (0 Critical/High) + review P2-01 vanta-review APPROVE + commits locales a76890f4 (feat) y a4dce566 (task file).
Resultado: OK
Próxima acción: Ninguno — tarea cerrada. Orquestador: proxima tarea F3 (Task 52 MEMG-08) y skill progreso de campana si corresponde.
Contrato: verificacion: cargo nextest run --profile audit -p vanta-memory --build-jobs 2 -> 684/684 passed (2 skipped); focused spill 12/12; e2e_flow 8/8 (memg06 on/off); precise-tokens memg06 2/2; cargo clippy -p vanta-memory --all-targets -- -D warnings OK; cargo check -p vantadb-mcp OK; cargo check --manifest-path desktop/src-tauri/Cargo.toml OK (3m09s); fmt scoped OK; docs gates (check-links/check-docs/gen-index --check) OK; OCR delegation sin Critical/High. evidencia: claim captura-antes-del-stub -> engine.rs stub_message Option<String> + mild_cascade sink (unit test spill_sink_receives_original_before_stub, confianza alta); claim recall round-trip -> spill.rs tests spill_then_recall_by_id_round_trips_full_content + memg06 e2e (confianza alta); claim opt-in default false -> ContextAssemblyConfig.spill_enabled false + memg06_spill_default_off_writes_nothing (confianza alta); claim GC reutilizado -> reclaim_as_of usa MIN_RETENTION_DAYS + iso_to_epoch_secs (reclaimer.rs) + tests (confianza alta); claim wire intacto -> IntegratedContext sin cambios + assemble firma intacta (confianza alta); review P2-01 -> ses_ef2d63433ffeQP02M1S5njyUWV APPROVE (confianza alta). artefactos: vanta-memory/src/context_engine/spill.rs; vanta-memory/src/context_engine/engine.rs; vanta-memory/src/services/pipeline_worker.rs; docs/dev/tasks/MEMG-06.md; target/ocr-memg06.json. invariantes: engine puro (sin offload/SDK en engine.rs); IntegratedContext serde intacto; opt-in default false byte-identico; recall spilled separado del recall L1; dedup D19 first-wins; GC conservador (retencion minima + undatable nunca se borra); sin unwrap/unsafe en produccion. deuda: +1 arg en assemble_with_recall (11 args, pre-existente >3; declarado); GC op explicita sin scheduler in-repo (espeja OffloadReclaimer); FIND-294 (aggressive/emergency sin captura) y FIND-295 (hook offload sin callers) registrados en Backlog. queda_pendiente: skill progreso/campaign-plan sync al orquestador; push SOLO con instruccion del owner (commits locales).
Próxima tarea si completa: 52
=== END RECITATION ===

=== RECITATION 50 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-03 — Grafo ↔ memoria: aristas de linaje + fix wipe + query BFS + DX-04
Estado: completed
Última acción: Cierre completo: e41fef7d (feat memory) + 2b333f04 (docs task RESULTADO) + e6efbc69 (docs avance registro). Coverage gates 1034/1034 IDs + 0 doc gaps. Learnings escritos.
Resultado: OK
Próxima acción: Ninguno. Fold pendiente (orquestador): FIND-296/297 + filas MEMG-03/DX-04 del Backlog (en vuelo por MEMG-08).
Contrato: verificacion: 12/12 memory_graph_lineage ✅ · scoped core 2357/2358 (1 timeout de carga documentado, pasa aislado) · vanta-memory 684/684 ✅ · fmt/clippy ✅ · docs gates ✅ · avance/docs coverage ✅ | evidencia: RED→GREEN del wipe + aristas de linaje + reconciliación + query BFS con tests (confianza alta) | artefactos: commits e41fef7d, 2b333f04, e6efbc69 | invariantes: record=canónico/arista=derivada; sin wire change; sin API pública nueva; no tocar storage/ | deuda: FINDs/rows a foldear por orquestador (Backlog en vuelo) | queda_pendiente: gate workspace-wide (diferido: MEMG-08 activo en el árbol)
Próxima tarea si completa: Siguiente F3 según orquestador (MEMG-08 en vuelo; MEMG-09 disponible)
=== END RECITATION ===

=== RECITATION MEMG-09 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-09 (Task 53): specs MGR-22/23/24 (research-docs) + slice v0 `code_index` (chunker por símbolo Rust + file-per-node) con tests RED→GREEN
Estado: completed
Última acción: Steps 0-6 completos: DISCOVERY + task file + FIND-299/300 + nota FIND-196; RED 10/10 fail ('Tool not found') → GREEN; iteración P2-01 F1-F8 (mcp_tests unfiltered 115/115, MCP.md/config/tools sincronizados 82/88, caps tests + symbols_truncated, impl_self_type robusto, nits) → re-review APPROVE; OCR 0 Critical/High; commits 0905e3c5 + 5c38cf07.
Resultado: OK
Próxima acción: Ninguno — tarea cerrada. Orquestador: skill progreso de campaña; F3 disponible (Task 55 MEMG-17; Task 54 MEMG-10 ya en vuelo por otro agente).
Contrato: verificacion: cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2 -> 170/170 OK · code_index_tests 13/13 OK · mcp_tests --ignore-default-filter 115/115 OK · cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings OK · fmt scoped OK · docs gates (check-links/check-docs) OK · validate-docs-coverage.ps1 0 gaps OK. evidencia: RED->GREEN (10/10 fail 'Tool not found' -> 13/13 pass, confianza alta); idempotencia hash-skip + reconcile sin ghosts/dup edges (version==1 en skip; asserts de edges; confianza alta); WIRE-02 listed solo en full (agent/dev/memory sin el tool; confianza alta); specs con fuentes verificadas por vanta-research (Claude Code 25k tokens/50k chars persist-to-file; Aider PageRank+1k+cache; OpenHands repo.md; repo-rag/cADR NO VERIFICADOS — confianza alta); review P2-01 iterada APPROVE ses_ef26b7620ffenv8FGRSFL86eoc (confianza alta). artefactos: commit 0905e3c5 (feat, 11 archivos +2336/-38); commit 5c38cf07 (docs task); docs/dev/tasks/MEMG-09.md; docs/dev/research/mgr-22-repo-map.md; docs/dev/research/mgr-23-24-memoria-proyecto.md; target/ocr-memg09.json. invariantes: 8 code_* intactos; core intacto (0 cambios en src/); sin deps nuevas; sin unsafe/unwrap/expect en produccion; keys propias file:/sym: en namespace target; WIP ajeno no stageado (MEMG-10 config/hooks/vanta-proxy intactos en worktree; config.rs committed = HEAD+mis comentarios via blob quirurgico). deuda: FIND-299 (tree-sitter Rust/Py/TS + watcher code_watch + scene repo-map + ranking); FIND-300 (graphrag content vacio para memory records); scanner v0 heuristico declarado (ponytail). queda_pendiente: skill progreso de campana (orquestador); fmt mecanico aplicado a vanta-memory (WIP MEMG-10) para desbloquear el hook repo-wide (no stageado); drift de docs/index.md+llms.txt +1 por MEMG-10.md (ajeno; el proximo gen-index lo absorbe).
Próxima tarea si completa: 55
=== END RECITATION ===

=== RECITATION 52 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-08 (Task 52) — MGR-25: trait Ingestor (formato → chunks MemoryInput con metadata source/page/chunk obligatoria) + mínimo viable txt/json/csv end-to-end + budget 28k/overlap; stop L1503 (5 formatos → trait + txt/json/csv + FIND html/pdf/docx).
Estado: completed
Última acción: Tarea COMPLETA: trait Ingestor + Txt/Json/CsvIngestor + default_ingestors + scan_ingestable_sources (proveniencia source/page/chunk plana, budget 28k post-chunk con truncación declarada, keys determinísticas {file}#{chunk}); refactor de sources.rs (collect_text_files compartido — un solo guard traversal; scan_local_sources sin cambios de contrato); tests 4 integración e2e + 8 unit; snapshot tests/api/public-api.txt +101/−0 refrescado; review P2-01 APPROVE (ronda 1, vanta-review fresco) con fold M1/L1/L2/L3/N1; FIND-298 en Backlog (working tree). Commits LOCALES 97039053 (feat) + e281c112 (docs/task). Sin push.
Resultado: OK
Próxima acción: Ninguna — tarea cerrada. Orquestador: seguir con Task 53 (MEMG-09, ya en vuelo por otra sesión) / skill progreso de campaña si corresponde; fold del plan y del Backlog (FIND-298 + filas MEMG-09) pendientes de quien cierre.
Contrato: verificacion: cargo nextest --profile audit -p vantadb --test wiki_ingestors ✅ 4/4 (default-menos-cli y fjall-only) · -p vantadb --lib -E test(wiki) ✅ 32/32 · -p vanta-memory --test ingest ✅ 15/15 · cargo clippy -p vantadb --all-targets -- -D warnings ✅ · cargo fmt --check ✅ · public_api ignored test ✅ 1/1 (compare). evidencia: {claim: 'trait Ingestor formato→chunks MemoryInput con proveniencia obligatoria', evidencia: 'src/wiki/ingestors.rs (trait + método provisto + 3 structs + registry) + test scans_supported_formats_into_chunks_with_mandatory_provenance', confianza: alta} {claim: 'txt/json/csv e2e scan→put_batch→get/search', evidencia: 'tests/wiki_ingestors.rs::ingested_chunks_round_trip_through_the_engine_and_are_searchable ✅', confianza: alta} {claim: 'budget 28k exacto con truncación + overlap 400', evidencia: 'scan_respects_source_char_budget_and_declares_truncation (total==28_000) + large_text_chunks_with_overlap_and_covers_all_content', confianza: alta} {claim: 'scan_local_sources sin regresión', evidencia: '32/32 wiki lib + 15/15 vanta-memory ingest', confianza: alta} {claim: 'review P2-01 fresh APPROVE', evidencia: 'subagente vanta-review ses_ef2818a59ffeE3B1mcXxImapTK, re-ejecutó comandos + lectura del refactor vs HEAD', confianza: alta} {claim: 'cero deps/unsafe/wire', evidencia: 'git diff de la tarea (sin Cargo.toml/unsafe)', confianza: alta}. artefactos: src/wiki/ingestors.rs, tests/wiki_ingestors.rs, src/wiki/sources.rs, src/wiki/mod.rs, tests/api/public-api.txt, docs/dev/tasks/MEMG-08.md, docs/index.md, llms.txt. invariantes: scan_local_sources firma+semántica idénticas (7 tests existentes verdes); proveniencia obligatoria por chunk; budget ≤ 28k; sin tocar wal/vector/storage; WIP ajeno intacto (MEMG-09); commits con pathspec. deuda: ponytail scan single-thread secuencial; keys stale tras shrink de archivo (sin contrato de borrado); NOTICED: gap docs/api del módulo wiki (pre-existente), symlink cycles pre-existente (review L4), keys >512 bytes solo documentado (review L2). queda_pendiente: (1) adr-gate del PR develop→main (ADR o [no-adr]) — lead; (2) pliegue de FIND-298 en Backlog (quedó en working tree, WIP de MEMG-09 adyacente) — quien cierre Backlog; (3) fold del plan — orquestador; (4) nota: docs/index.md+llms.txt commiteados incluyen entradas de archivos de MEMG-09 aún sin commitear (patrón compartido ya presente en HEAD con MEMG-03; se auto-corrige al commitear MEMG-09).
Próxima tarea si completa: MEMG-09 (Task 53) — en vuelo
=== END RECITATION ===

=== RECITATION 55 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-17 (Task 55) — Rollback + verificabilidad + erasure criptográfica: (a) rollback semántico a versión append-only con linaje + snapshot con alcance declarado (FIND-287); (b) recibos verificables de erasure con el contrato VER-02 (no claim-driven; firma ML-DSA-65 no sancionada → declarada); (c) erasure por destrucción de DEK (registry CSPRNG wrapped con master Cipher + tombstone); cada pieza con test propio.
Estado: completed
Última acción: CIERRE COMPLETO + progreso: commits LOCALES 80cb84b0 (feat) + 1ec943fe (docs/task RESULTADO+§Review) + 4c0da351 (docs/avance: registro vanta-memory + fila Backlog MEMG-17 removida). Incidente de cierre reparado: un commit intermedio (ce32ff37) arrastró WIP staged de MEMG-10 (index compartido) → revertido con reset --soft + re-commit con pathspec; staging de MEMG-10 PRESERVADO intacto (M/A en su columna). Learnings: 2 en memory/lessons (pathspec en multi-sesión; gate #![cfg(feature)] en tests). Review P2-01 ronda 2 APPROVE.
Resultado: OK
Próxima acción: Ninguna — tarea cerrada (taskId 55 completed). Orquestador: skill progreso de campaña; MEMG-10 (Task 54) sigue en vuelo con su WIP staged intacto; próximas: Task 58 (MEMG-16).
Contrato: verificacion: cargo nextest run --profile audit -p vanta-memory --build-jobs 2 ✅ 701/701 · --features erasure ✅ 712/712 · --test rollback ✅ 7/7 · --test erasure --features erasure ✅ 11/11 · -p vantadb --lib -E test(attestation) ✅ 10/10 · cargo fmt --check ✅ · clippy -p vanta-memory --all-targets --all-features -D warnings ✅ · validate-docs-coverage.ps1 0 gaps ✅ · check-avance-coverage.ps1 1034/1034 ✅ · gen-index --check ✅ · OCR ✅ · review P2-01 ✅. evidencia: {claim: 'las 3 piezas del contrato L1585 completas con test propio', evidencia: 'tests/rollback.rs 7/7 + tests/erasure.rs 11/11 + §Review APPROVE en docs/dev/tasks/MEMG-17.md', confianza: alta}. artefactos: docs/dev/tasks/MEMG-17.md · docs/api/VANTA_MEMORY.md · docs/dev/Backlog.md (FIND-302, fila MEMG-17 removida) · docs/dev/avance/activo/vanta-memory.md (entrada MEMG-17). invariantes: firmas existentes intactas; wire L1 intacto; recibos nunca inválidos/nunca claim-driven; WIP ajeno no commiteado por este run (incidente ce32ff37 revertido). deuda: FIND-302 · FIND-194 · FIND-287. queda_pendiente: orquestador — MEMG-10 staged intacto para SU commit; si su sesión no retoma, el staging está listo para commit directo.
Próxima tarea si completa: MEMG-16
=== END RECITATION ===

=== RECITATION 54 ===
Campaign ID: master-plan-0.9.0-20261004
Objetivo activo: MEMG-10 / MGR-04 — spec (trusted/tainted + RBAC por acción + integración retrieval/inyección) + impl sobre base VER-04: trust gate por namespace + audit RBAC por acción (corte declarado stop L1559)
Estado: completed
Última acción: Cierre completo: commit e0373d7b (17 archivos, +1012/-40; hooks verdes) + eec590b5 (task RESULTADO) + ed57fb53 (avance + fila Backlog removida); review P2-01 APPROVE (2 rondas); OCR 0 Critical/High; suites + gates verdes
Resultado: OK
Próxima acción: ninguno — tarea cerrada. El orquestador resuelve la próxima vía campaign_get_next_task (F3 completa con MEMG-10)
Contrato: verificacion: cargo nextest -p vanta-memory 694/694 + memg10_trust_gate 4/4 + policy 9/9 ✅ · -p vanta-proxy 322/322 ✅ · -p vantadb-mcp 171/171 ✅ · -p vantadb --features server --test rbac_namespace 11/11 ✅ · auth/rbac 48/48 ✅ · fmt/clippy scoped 4/4 ✅ · docs gates (links/docs/coverage) 0 + gen-index exit 0 ✅
evidencia:
  - claim: trust gate trusted/tainted enforced en las superficies gobernadas (tainted no inyecta por defecto; opt-in include_tainted; gates ACL AND trust) | evidencia: vanta-memory/src/core/hooks/auto_recall.rs:125-227 + tests memg10_trust_gate.rs 4/4 + unit policy 9/9 + wiring proxy (config.rs:109-116) + MCP (config.rs:119-128,195-207) | confianza: alta
  - claim: audit RBAC por acción (auth_rbac) aditivo, sin cambio de semántica de auth, sin token en el log | evidencia: src/server/middleware.rs:203-243 + tests/rbac_namespace.rs 11/11 (read/write/delete) + 48/48 auth/rbac | confianza: alta
  - claim: spec MGR-04 + FIND-301 con el residual declarado (incl. superficie L3 no gobernada del review R1) | evidencia: docs/dev/research/mgr-04-policy-engine.md + Backlog FIND-301 + avance/activo/vanta-memory.md | confianza: alta
  - claim: review P2-01 adversarial por agente distinto | evidencia: vanta-review ses_ef20ae781ffeYWCIMdoxswKav0 — ronda 1 changes-required (R1/R2 doc-only) → fixes → delta re-review APPROVE | confianza: alta
artefactos: commits e0373d7b (17 archivos, +1012/-40) · eec590b5 (task RESULTADO) · ed57fb53 (avance+Backlog); spec docs/dev/research/mgr-04-policy-engine.md; task docs/dev/tasks/MEMG-10.md
invariantes: defaults byte-idénticos (tainted vacío == VER-04); tainted no inyecta sin opt-in en superficies gobernadas; gates ACL/trust AND; sin cambio de semántica de autorización; audit nunca registra tokens; WIP ajeno no commiteado (pathspec)
deuda: FIND-301 — (a) enforcement RBAC por acción (NamespaceDelete + separación estricta opt-in + ADR), (b) roles namespace-scoped configurables, (c) trust en retrieval HTTP, (d) promoción curada, (e) superficie L3 pipeline_worker no gobernada
queda_pendiente: nada del corte declarado (stop L1559 cumplido); avance registrado; race de commits concurrente documentado en task file §Notas
Próxima tarea si completa: siguiente del plan (resolver vía campaign_get_next_task — F3 completa)
=== END RECITATION ===
