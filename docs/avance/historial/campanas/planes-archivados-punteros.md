# Planes archivados — retrospectivas

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

---

## Planes archivados

- **Plan archivado:** `docs/plans/archive/2026-08-12-ci-deuda.md` — 6/6 completadas (CI-02..07, deuda CI batch)
- **Retrospectiva:** Start: verificación de realidad contra código real antes de DO (Paso 0) + contratos booleanos accionables con actionlint | Stop: batch CI-01 del doc EXTRACCION estaba stale (ROOT1-007 ya resuelto; solo 6 de 26 hallazgos eran reales) — no planificar desde docs sin verificar | Continue: waves paralelas por archivos disjuntos + routing vanta-lead para CI; escalera SARL (RESUME→RETRY) antes de marcar fallo | Acción medida: verify retries/tarea en CI batch = 1/6 (CI-05 requirió RETRY tras 2 salidas vacías; baseline North Star: >90% completado en primer intento)

- **Plan archivado:** `docs/plans/archive/2026-08-16-wave-p20-tsys.md` — 25/25 completadas (wave P20-TSYS: TSYS-06 + R1/R3/R5/R6/R8/R9/R10 + FND-01..24 activas; 21 commits en develop desde `ec7f947a` hasta `a159211b`)
- **Retrospectiva:** Start: waves paralelas con routing por dominio disjunto (vanta-lead/worker/arch/tuner/docs) + commits por batch del lead (regla de plan) | Stop: — | Continue: 0 DO diferidos/SKIP/BLOQUEADO; el gate por batch (contrato booleano + dictamen P2-01) mantuvo 25/25 en primer intento
---

## Planes archivados

- **Plan archivado:** `docs/plans/archive/2026-08-18-vanta-studio-fase3.md` — 7/7 completadas (Vanta Studio Fase 3: web/embebido — transporte pluggable, REST completo del SDK, dashboard `/dashboard` servido por el server)
- **Retrospectiva:** Start: FAIL_MODE=parallel con waves por archivos disjuntos (WEB-00 desktop vs WEB-01 server) | verify mecánico del lead (`cargo check --tests`, no solo `--lib`) antes de cada commit | hallazgo de sub-agente (E2E cazó bug real de namespace default en REST) validado y fixeado por el lead | Stop: aceptar resultado vacío de sub-agente como completado (2 sub-agentes devolvieron `task_result` vacío sin tocar nada → RESUME misma sesión con feedback arregló ambos) | dejar `cargo check --lib` como único gate server (no detectó `cli_tests` roto por WEB-03 — gap corregido con `--tests`) | asumir counts de comandos de planes (~55) sin verificar repo (real: 23) | Continue: relanzar con `task_id` (RESUME) en vez de agente fresco cuando el trabajo previo es cero | correr `cargo fmt` en el verify del sub-agente (pre-commit lo exige — WEB-01 falló por esto) | Acción medida: verify retries/tarea = 1/7 (solo WEB-01 requirió fix post-verify por fmt); baseline North Star: >90% primer intento ✅ (86%)

- **Plan archivado:** `docs/plans/archive/2026-08-18-vanta-studio-fase2.md` — 10/10 completadas (Vanta Studio Fase 2: gaps core VS-CORE-04/05/06 + lente GRAFO R3F + lente ESPACIO + operaciones import/batch)
- **Retrospectiva:** Start: doble gate del lead (verify mecánico + commit selectivo) por tarea — 0 regresiones en 10/10 | DAG secuencial corregido a tiempo (Tasks 2/3 NO disjuntas) | relanzar tareas con digest del task file tras abort sin task_id (Task 10) | force-directed en frame loop fuera del render React (positionsRef) | Stop: confiar en descripciones de tipos del orquestador (VS-CORE-05: `VantaMemoryFilter` es `Vec<VantaMemoryFilterItem>`, no `{op,items}`) | dejar que sub-agentes corrompan plan file (lead único dueño) | asumir React 18 en desktop (real: 19.1.0 → r3f v9+drei v10) | Continue: skills warmup obligatorio al delegar | verify mecánico con `campaign_verify_cmd` en cada step | decisiones de contrato del usuario (D5 force-directed) antes de delegar | Acción medida: verify retries/tarea = 1/10 (solo ESPACIO-02 requirió fix post-commit); baseline North Star: >90% primer intento ✅ (90%)

- **Plan archivado:** `docs/plans/archive/2026-08-17-skills-vantadb.md` - 4/4 completadas (wave SKL: skills de VantaDB corregidas)
- **Retrospectiva:** Start: diagnóstico del lead con evidencia archivo:línea antes de delegar + contratos mecánicos accionables + waves paralelas por archivos disjuntos | Stop: contrato decía 14 tools cuando el server real tiene 15 (corregido en DISCOVERY, no bloqueó) | Continue: routing por dominio (docs→vanta-docs, scripts→vanta-worker) + gate P2-01 que encontró 1 falla real de coherencia doc↔código que los checks mecánicos no veían | Acción medida: verify retries/tarea = 1/4 (SKL-02 requirió fix post-review); baseline North Star: >90% primer intento

- **Plan archivado:** `docs/plans/archive/2026-08-18-vanta-studio-fase1.md` — 9/9 completadas (P26 Studio Fase 1: explicabilidad y tiempo)
- **Retrospectiva:** Start: checkpoint humano D2 antes de implementar (3 decisiones aprobadas: cap 32, snapshot nuevo, import sin snapshots) + waves paralelas con slices aditivos + verify mecánico del lead antes de cada commit | Stop: sub-agentes corrompieron el plan file 3 veces (recitation en Contrato + header "completed") → revertido por el lead cada vez; cargo fmt de sub-agentes reformateó archivos que no tocaban (ruido revertido) | Continue: lead es el único que toca plan/Backlog + merge de slices compartidos + pre-commit hook que atrapó fmt faltante en version_history.rs | Acción medida: corrupciones de plan file por sub-agente = 3/9 tareas; baseline: 0 (regla "no tocar plan file" ya era explícita)

---

## Planes archivados — task-system hardening (2026-08-23)

- **Plan archivado:** `docs/plans/archive/2026-08-23-task-system-hardening.md` — ✅ COMPLETED (hardening I: H1-H9 campaign server + question gates HITL; contrato `node --test .opencode/task-system/mcp/` 38/38; commits `fcd7b243` + `26f68ff7`)
- **Retrospectiva:** ver header del plan (recitation final + retrospectiva en el propio archivo, commit `784e3c68`)

- **Plan archivado:** `docs/plans/archive/2026-08-23-task-system-hardening-2.md` — 12/12 completadas (hardening II: R1-R7 paralelismo real multi-instancia + S1 spec-first + D1-D4 limpieza decidida; 42/42 tests; commits `1a86bd2a`, `ed6c6ae7`, `b8a23939`, `3cc0aa50`)
- **Retrospectiva:** Start: ejecución por fases con gate mecánico entre fases (detectó al instante desbalance de paréntesis y regresión del mensaje de lock) + dogfooding del MCP durante la ejecución (reveló que updateTaskStateCore descartaba el Campaign ID — fix incluido) | Stop: editar archivos línea-a-línea con pipelines PS frágiles (`-NoNewline` corrompió .gitignore) — usar reemplazo regex explícito sobre raw | asumir que el server en memoria refleja el código en disco (los fixes R1 no estaban vivos durante la campaña; verificar versión con uptime/commit al arrancar y reiniciar OpenCode tras fixes del server) | Continue: question gates previos a decisiones estructurales (las 4 respuestas Gate P definieron todo el alcance) + fuente única canónica con punteros (subagent-recovery, spec-template, state-tools.mjs) | Acción medida: adoptar claim:true en todos los flujos /pipeline run para eliminar doble-Discovery — métrica: tareas con 2+ task.started events en traces; objetivo 0 (commit `3cc0aa50`)
