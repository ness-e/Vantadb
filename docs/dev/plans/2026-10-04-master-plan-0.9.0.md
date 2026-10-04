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
- **Estado:** ⏳ EN PROGRESO · **Task file:** `docs/dev/tasks/DX-12.md`

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
- **Estado:** ⏳ EN PROGRESO · **Task file:** `docs/dev/tasks/DIST-15.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/DIST-16.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/DIST-17.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WSM-14.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/TS-10.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/TS-11.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/TS-13.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/PROV-12.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/PROV-13.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/DESKTOP-41.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/DESKTOP-43.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/DESKTOP-44.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/SHOW-02.md`

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
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WEB-09.md` (+ `MKT-22.md` si se separa)

## F2 — Memoria I (cimientos del diferenciador)

> **Gate de fase:** compactas — al iniciar F2 se expanden al nivel F0 (pre-mortem + Risk Register + stop conditions) antes de ejecutar. Los specs MGR de P49 (research) alimentan estas implementaciones.

### Task 37: MEMG-01 — Detección de contradicción en ingesta L1

- **Appetite:** max 3d · **Esfuerzo:** 🟠 2-3d · **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/core/dream/` (contradiction provenance), pipeline L1, `superseded_by`
- **Verificación real:** ✅ — DELTA P1: contradicción en ingesta L1 pendiente; la provenance de dream existe como base.
- **Contrato:** la ingesta L1 detecta contradicciones y las marca (cuarentena/provenance) con test dedicado; no-contradictorios sin cambio de semántica.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-01.md`

### Task 38: MEMG-02 — Outcome loop → refuerzo de confianza post-recall

- **Appetite:** max 3d · **Esfuerzo:** 🟠 2-3d · **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory` (recall path, confianza), `reinforce`
- **Verificación real:** ✅ — `reinforce` = 4 hits, **ninguno actualiza confianza**; `outcome` = 14/48 (verificado HEAD 2026-10-01). Queja canónica contra Mem0/Letta.
- **Contrato:** el resultado de una recall (¿resolvió? ¿corrigió?) realimenta la confianza del registro vía `reinforce` real, con test; métrica antes/después.
- **Skills sugeridas:** source-driven-development · rust-write-tests · performance-optimization · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-02.md`

### Task 39: MEMG-11 — Adopción del motor core en `vanta-memory` (recall híbrido + escritura batch)

- **Appetite:** max 2sem · **Esfuerzo:** 🔴 1-2sem · **Prioridad:** 🔴
- **Archivos clave:** recall L1 de `vanta-memory` (dual-pool propio), escrituras del pipeline, `put_batch`/`query_sparse`/filtros del core
- **Verificación real:** ✅ — `put_batch` = 0 hits en `vanta-memory`; recall usa scan sin índice de texto/HNSW (reimplementación más débil del diferenciador).
- **Contrato:** recall L1 sobre búsqueda híbrida del core (BM25+HNSW+RRF del planner) + escrituras vía `put_batch` (group-commit); **A/B documentado sin regresión p99** (`canonical_p99`); orden: medir → migrar → A/B.
- **Skills sugeridas:** source-driven-development · performance-optimization · rust-write-tests · doubt-driven-development · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-11.md`

### Task 40: MEMG-12 — Semántica v2 write-side en el pipeline (`confidence`/`valid_at`/TTL)

- **Appetite:** max 2sem · **Esfuerzo:** 🟠 1-1.5sem · **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/core/pipeline/l0_recorder.rs:243`, `pipeline_worker.rs:775`, `dream/mod.rs:537` (todos `ttl_ms: None`)
- **Verificación real:** ✅ — con defaults, confianza y bitemporalidad quedan **inertes** ("confidence=0 en vanta-memory"); el gate de cuarentena funciona en lectura (SCH-05) pero nadie marca dudoso al escribir.
- **Contrato:** L0→L3 escribe `confidence`/`valid_at_ms`/TTL semántico reales + transiciones de cuarentena desde dream/ingesta; round-trip test.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-12.md`

### Task 41: MEMG-13 — Superficies core restantes en memoria (IQL / versiones / snapshots / filtros)

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory` (recall/list), core: IQL, `versions`/`get_version`, snapshots, filtros/cursors
- **Verificación real:** ✅ — `query_iql` = 0 hits en `vanta-memory`; recall hoy es `list` plano (verificado HEAD 2026-10-01).
- **Contrato:** la memoria consume IQL + versiones + snapshots + filtros/cursors del core donde apliquen; tests por superficie.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-13.md`

### Task 42: MEMG-07 — Forgetting curves sobre L1

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2d · **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory` (eviction/decay), `BayesianDecay`
- **Verificación real:** ✅ — "Solo `BayesianDecay` de eviction; falta curva de olvido real".
- **Contrato:** curva de olvido real sobre L1 (decay por tipo/edad, configurable) con test + métrica; sin pérdida de registros no-expirados.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-07.md`

### Task 43: MEMG-20 — Checkpoints reanudables de tarea (dim 1)

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory` (working memory / task state)
- **Verificación real:** ✅ — DELTA P2 (dim1): checkpoints reanudables pendientes.
- **Contrato:** checkpoints de tarea reanudables con test de reanudación (interrupción → retoma estado); API mínima.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-20.md`

### Task 44: MEMG-21 — Scoring multi-señal L1 (recencia + importancia) + reflexión periódica

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory` (scoring L1), reflexión sobre episódica
- **Verificación real:** ✅ — dim2 gap: "falta scoring recency-relevance-importance con decaimiento, reflexión periódica" (Generative Agents degenera sin reflexión; CrewAI ya vende composite scoring).
- **Contrato:** scoring compuesto (recencia+relevancia+importancia) + reflexión periódica con test; métrica de ordenamiento.
- **Skills sugeridas:** source-driven-development · rust-write-tests · performance-optimization · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-21.md`

### Task 45: WIRE-14 — T1: Seam de host aditivo en el arranque (`conversation_trigger` + servicio)

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟠
- **Archivos clave:** arranque de host (`vantadb-server`), seam de inyección; dep `vanta-memory`
- **Verificación real:** ✅ — WIRE-14..18 (scheduler T1-T5) del Alta 2026-10-01; verificación e2e: `POST /conversation/add` → L0 → pass → `l1/<session>`.
- **Contrato:** seam aditivo (sin breaking) que permite inyectar `conversation_trigger` + servicio; e2e verde.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WIRE-14.md`

### Task 46: WIRE-15 — T2: Servicio scheduler en `vanta-memory` (`run_pass` + loop con shutdown)

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory` (timers + worker + reclaim), espejo `MemoryTtlSweeper`
- **Verificación real:** ✅ — feature-gated; verificación: `cargo test -p vanta-memory --features http-server`.
- **Contrato:** `run_pass` (timers+worker+reclaim) + loop helper con shutdown graceful; tests feature-gated verdes.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WIRE-15.md`

### Task 47: WIRE-16 — T3: Wiring del wrapper `vantadb-server` (queue + bridge + loop)

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟠
- **Archivos clave:** `vantadb-server` (queue + bridge + loop), env config `VANTADB_SCHEDULER_*` (R-5)
- **Verificación real:** ✅ — e2e `POST /conversation/add` → L0 → pass → `l1/<session>`; restart; disabled; sin runner.
- **Contrato:** wiring completo con las 4 verificaciones e2e del punto anterior verdes.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WIRE-16.md`

### Task 48: WIRE-17 — T4: Docs del scheduler (wired status + promoción del rol)

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🟡
- **Archivos clave:** `docs/api/VANTA_MEMORY.md` §Operational modules, `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Verificación real:** ✅ — WIRE-17 del Alta 2026-10-01.
- **Contrato:** docs del scheduler (wired status + promoción del rol + fila `vanta-memory`); `check-docs`/`check-links` exit 0.
- **Skills sugeridas:** documentation-skill · documentation-and-adrs
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WIRE-17.md`

### Task 49: WIRE-18 — T5: Verificación adversarial del scheduler (crash mid-pass + restart)

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟠
- **Archivos clave:** `tests/` (chaos), scheduler; carril vanta-chaos / vanta-review
- **Verificación real:** ✅ — WIRE-18 del Alta 2026-10-01: crash mid-pass + restart (cola efímera) + review P2-01.
- **Contrato:** test de crash mid-pass + restart verde (cola no corrompe) + review P2-01 por agente distinto.
- **Skills sugeridas:** systematic-debugging · rust-write-tests · doubt-driven-development · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/WIRE-18.md`

## F3 — Memoria II (profundo)

> **Gate de fase:** compactas — expandir al nivel F0 al iniciar F3. Consume los specs MGR ya investigados (P49) + FIND-196 (repo-map).

### Task 50: MEMG-03 — Grafo ↔ memoria (L1–L3 como nodos/aristas)

- **Appetite:** max 2sem · **Esfuerzo:** 🔴 1-2sem · **Prioridad:** 🔴
- **Archivos clave:** `src/graph.rs`, `edge_index`, `memory_node_id` (`src/sdk/serialization/mod.rs:77`), `node_id` (`src/sdk/types/record.rs:213-215`)
- **Verificación real:** ✅ — el mapeo determinista ns+key→u128 YA existe; falta la integración L1–L3 como nodos/aristas (verificado HEAD 2026-10-01).
- **Contrato:** L1–L3 como nodos/aristas reales + query "¿quién cambió la fuente de esta decisión y por qué?" respondible con test; sin regresión de recall.
- **Skills sugeridas:** source-driven-development · rust-write-tests · api-and-interface-design · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-03.md`

### Task 51: MEMG-06 — Spill a disco con recall

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟡
- **Archivos clave:** presupuesto de contexto de `vanta-memory`; `offload/` existente (comprime in-memory sin recall)
- **Verificación real:** ✅ — `spill` ≈ 1 hit; offload (34k) sin recall (verificado HEAD 2026-10-01).
- **Contrato:** spill a disco con recuperación (recall de lo spilled) con test; sin pérdida de datos.
- **Skills sugeridas:** source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-06.md`

### Task 52: MEMG-08 — MGR-25: Formatos e ingestores (trait `Ingestor`)

- **Appetite:** max 2sem · **Esfuerzo:** 🟠 1-2sem · **Prioridad:** 🟠
- **Archivos clave:** `src/wiki/sources.rs:82` (hoy solo `.md`), `src/sdk/types/record.rs:124` (`MemoryInput`), `SOURCE_CHAR_BUDGET` (28k)
- **Verificación real:** ✅ — "el path wiki solo acepta `.md` y `put` recibe `payload: String` sin extractor"; spec completa en Notion (MGR-25).
- **Contrato:** trait `Ingestor` + orden txt/json/csv → html → pdf → docx; mínimo viable ≥2 formatos end-to-end con `metadata.source` (proveniencia); tests.
- **Skills sugeridas:** source-driven-development · rust-write-tests · api-and-interface-design · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-08.md`

### Task 53: MEMG-09 — Track PI restante (MGR-23/24: grafo decisión→código→test + taxonomía)

- **Appetite:** max 1mes · **Esfuerzo:** 🔴 2-4sem · **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mcp/src/code.rs:42-165,192-328`, `src/graphrag/pipeline.rs`, `src/wiki/sources.rs:82`; FIND-196 (repo-map)
- **Verificación real:** ✅ — decisión owner 2026-09-14: watcher Rust+Python+TypeScript desde el inicio; spec pendiente (MGR-23/24).
- **Contrato:** spec (chunker + watcher + repo-map + API `code_index`/`code_watch`) + primer slice implementado según spec; research-doc.
- **Skills sugeridas:** spec-driven-development · source-driven-development · doubt-driven-development · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-09.md`

### Task 54: MEMG-10 — MGR-04: Policy engine (trusted/tainted + RBAC por acción)

- **Appetite:** max 1mes · **Esfuerzo:** 🔴 2-3sem · **Prioridad:** 🟠
- **Archivos clave:** `src/server/middleware.rs`, `src/server/jwt.rs`, `tests/rbac_namespace.rs`; base VER-04 (budget/ACL/audit)
- **Verificación real:** ✅ — MGR-04 quedó fuera del plan F0–F6 (clase mínima vía SCH-05/VER-04); Notion 6 áreas §Gobernanza.
- **Contrato:** spec + namespaces trusted/tainted con RBAC por acción (tests por acción) sobre la base VER-04.
- **Skills sugeridas:** security-and-hardening · spec-driven-development · source-driven-development · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-10.md`

### Task 55: MEMG-17 — Rollback + verificabilidad + erasure criptográfica

- **Appetite:** max 1sem · **Esfuerzo:** 🟠 3-5d · **Prioridad:** 🟠
- **Archivos clave:** versions (MEMG-13), hash-chain (VER-01), DEK/tombstones
- **Verificación real:** ✅ — VMG lista Rollbackability + Verified Forgetting como primitivas; EDPS exige proof of unlearning; base versions+VER-01 existen.
- **Contrato:** rollback semántico (a versión/snapshot con linaje) + recibos verificables + erasure (destrucción DEK + tombstone) con tests.
- **Skills sugeridas:** security-and-hardening · source-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-17.md`

## F4 — Sharing & Multi-tenant (caso "equipo" + enterprise)

> **Gate de fase:** compactas — expandir al nivel F0 al iniciar F4.

### Task 56: MEMG-04 — Multi-tenant (enforcement + cuotas)

- **Appetite:** max 1mes · **Esfuerzo:** 🔴 2-3sem · **Prioridad:** 🔴
- **Archivos clave:** storage/API boundary, cuotas, billing boundary
- **Verificación real:** ✅ — `rg tenant` = 0-5 hits (comentarios); sin enforcement (verificado 2026-09-30).
- **Contrato:** aislamiento real por tenant (enforcement en storage/API) + cuotas; test de no-cruce de datos entre 2 tenants.
- **Skills sugeridas:** security-and-hardening · source-driven-development · doubt-driven-development · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-04.md`

### Task 57: MEMG-05 — Multi-escritor (CRDT / vector-clock / LWW declarado)

- **Appetite:** max 1mes · **Esfuerzo:** 🔴 2-3sem · **Prioridad:** 🟠
- **Archivos clave:** sync multi-device (write path)
- **Verificación real:** ✅ — `CRDT`/`vector_clock`/`LWW` = 0 hits: "hoy el que escribe gana — escrituras paralelas pierden datos silenciosamente".
- **Contrato:** resolución de conflictos multi-escritor implementada (opción elegida documentada) con test de escrituras paralelas sin pérdida.
- **Skills sugeridas:** source-driven-development · doubt-driven-development · rust-write-tests · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-05.md`

### Task 58: MEMG-16 — Compartir/colaboración multi-agente (scopes + permisos + revocación)

- **Appetite:** max 1mes · **Esfuerzo:** 🔴 1-2sem · **Prioridad:** 🟠
- **Archivos clave:** consume MEMG-04/05 + EXE-07 (P50); modelo de datos compartidos
- **Verificación real:** ✅ — "Share & Propagate" es fase propia del lifecycle 2026; Letta/Cognee/CrewAI ya lo ofrecen; MAST (NeurIPS 2025) documenta fallos de inter-agent misalignment; sin esto no hay caso "equipo".
- **Contrato:** memoria compartida con scopes (org/team/proyecto) + permisos + revocación, entre agentes y usuarios; modelo documentado (aislamiento/propagación/revocación); tests.
- **Skills sugeridas:** security-and-hardening · source-driven-development · documentation-and-adrs · campaign-executor
- **Estado:** ⬜ PENDING · **Task file:** `docs/dev/tasks/MEMG-16.md`

### Task 59: VER-10 — Attestation de escritura (extender el certificado de delete a writes)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟡
- **Archivos clave:** hash-chain (VER-01), certificados de delete existentes
- **Verificación real:** ✅ — "Hoy solo delete tiene attestation; extender a escrituras. Solapa con hash-chain (VER-01)".
- **Contrato:** escrituras con attestation verificable (recibo) extendiendo el chain; test válido/inválido.
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
