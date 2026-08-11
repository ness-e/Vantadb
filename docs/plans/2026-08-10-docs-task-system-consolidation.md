# Plan de Ejecución: Consolidación de Docs y Task-System

> **Campaign ID: e0db1e63-9330-4737-9b1a-1d9ed042cd12**
> **Inicio:** 2026-08-10
> **Estado:** planning
> **Fuente:** Auditoría multi-agente (6 sub-agentes, solo-lectura) sobre `docs/Investigaciones/2026-08-10-agent-engineering/`, las 10 carpetas de `docs/` objetivo, y el flujo completo del task-system en `.opencode/`.

## Resumen

Este plan fusiona 6 auditorías independientes en UN solo programa de trabajo. Tres dominios:

1. **Pendientes de `agent-engineering`** (lo que falta aplicar del plan P0-P3 ya cerrado).
2. **Consolidación de carpetas `docs/`** (duplicados, huérfanas, drift de rutas, 4 carpetas "archive").
3. **Arreglos del task-system** (flujo, huérfanos, incongruencias, referencia rota a persona `vanta-review`).

**Principio rector:** `docs/progreso/` es el registro canónico (single source of truth); `docs/avance/` es su mirror curado navegable; `docs/Investigaciones/` es la carpeta canónica de investigación. Nada se borra sin grep previo (Regla 0). No se reescriben snapshots/historial (estados pasados congelados).

**Bloqueante conocido pre-existente:** `verify-log.jsonl` = 0 bytes → North Star/DORA/SLA sin datos (transversal a todo).

## Planes/carpetas ya verificados como OK (NO tocar)

- Planes P0-P3: implementados y comiteados (commits 0592695f, f85c8b0d, 350e9725, 0e298f3d, e44b26a5, 44154c0f, b1a117de, c14050eb, fe21a890, 724b355c, 887d0f14).
- `docs/plans/` raíz (6 archivos) + `docs/plans/archive/`: núcleo vivo del pipeline, sin cambios estructurales.
- `docs/reports/` (INDEX + northstar/dora/pipeline-evals): generados por script, correctos.
- `docs/progreso/README.md`: registro canónico, correcto.
- WONTFIT: enforcement absoluto del MCP server, rainbow deploys (dependencias externas, solo registrar).

## Tasks

### Task 1: Commitear WIP pendiente (higiene base)
- **Esfuerzo:** 🟢 | **Prioridad:** P0 | **Ruta:** vanta-lead
- **Archivos clave:** working tree develop (ADR-015-coverage-policy.md, CI_POLICY.md, northstar.md, pipeline-evals.md, test_sdk.py, vitest.config.ts, .pre-commit-config.yaml nuevo)
- **Verificación real:** `git status` en develop muestra 6 modificados + 1 untracked; rama ahead of origin.
- **Gate Result:** 🔵 DO
- **Contrato:** `git status` limpio; commits conventional (`ci:`/`feat:`); `git log` registra los cambios de COV-001/002/004 y CI-01.
- **Estado:** ⬜ PENDING

### Task 2: Poblar verify-log.jsonl (desbloquea North Star/DORA/P3-2/SLA)
- **Esfuerzo:** 🟡 | **Prioridad:** P0 | **Ruta:** vanta-lead
- **Archivos clave:** `.opencode/task-system/enforcement/verify-log.jsonl` (0 bytes), `evals/northstar.mjs`, `evals/eval-metrics.mjs`
- **Verificación real:** Los 3 evaluadores ya funcionan y escriben `docs/reports/`; falla el **dato**: 0 invocaciones de `campaign_verify_cmd`.
- **Gate Result:** 🔵 DO
- **Contrato:** al menos 1 tarea real ejecutada por el pipeline con `campaign_verify_cmd` → `verify-log.jsonl` > 0 líneas; `docs/reports/` regenerado con datos reales.
- **Estado:** ⬜ PENDING
- **Notas:** Es el habilitador transversal; cualquiera de las Tasks 3-13 ejecutadas vía pipeline lo alimenta.

### Task 3: Unificar carpetas de investigación → `docs/Investigaciones/` (4→1)
- **Esfuerzo:** 🟡 | **Prioridad:** P0 | **Ruta:** vanta-docs
- **Archivos clave:**
  - `docs/investigacion/investigacion-equipo-2026-08-09.md` → MOVE a `docs/Investigaciones/` (basename conservado o `INV-021-`)
  - `docs/research/COGNEE_EVALUATION.md`, `docs/research/MVCC_SNAPSHOT_ISOLATION.md` → MOVE a `docs/Investigaciones/`
  - `.opencode/Investigaciones/VantaDB-28-07-2026.md` → DELETE (duplicado byte-casi-exacto de `docs/Investigaciones/VantaDB-28-07-2026.md`, 43 bytes diff, 0 referencias)
- **Verificación real:** Duplicado exacto confirmado (misma investigación Perplexity, 822 líneas, 0 refs a la copia `.opencode/`). `docs/research/` queda con 0 archivos → deprecar carpeta (regla ya la prohíbe como destino).
- **Refs a actualizar (solo vivas):** `docs/Backlog.md:18,51,448`, `docs/architecture/adr/ADR-014-pitr.md:68`, `campaign-executor/tasks/complete/VFY-011.md:44`, `docs/progreso/bitacora.md:386`.
- **Gate Result:** 🔵 DO
- **Contrato:** `git mv` completados; `rg "docs/investigacion|docs/research/COGNEE|docs/research/MVCC"` fuera de docs/ = 0 refs vivas; `.opencode/Investigaciones` eliminado; `docs/Backlog.md` intacto en rastro de origen P16.
- **Estado:** ⬜ PENDING
- **Notas:** Riesgo ALTO en refs de `docs/Backlog.md` (3) — es el doc más activo; actualizar en el mismo commit que el move.

### Task 4: Corregir drift del workflow research.json
- **Esfuerzo:** 🟢 | **Prioridad:** P1 | **Ruta:** vanta-lead
- **Archivos clave:** `.opencode/task-system/workflows/research.json:80` (dice "Guardar output final en docs/research/")
- **Verificación real:** Regla en `.opencode/AGENTS.md:855` y manual L775 prohíben `docs/research/`; el propio task `DESKTOP-01.md:39` documenta el override manual.
- **Gate Result:** 🔵 DO
- **Contrato:** workflows/research.json apunta a `docs/Investigaciones/`; `rg "docs/research"` en `.opencode/task-system/` = 0.
- **Estado:** ⬜ PENDING

### Task 5: Restaurar ACID_TRANSACTIONS.md desde git
- **Esfuerzo:** 🟢 | **Prioridad:** P1 | **Ruta:** vanta-docs
- **Archivos clave:** `git show 8b1c52cd^:docs/research/ACID_TRANSACTIONS.md` → restaurar a `docs/Investigaciones/ACID_TRANSACTIONS.md`
- **Verificación real:** Recomendado explícitamente por `docs/Investigaciones/ACID_ROLLBACK_DESIGN.md:57,61,455,495`; el archivo fue borrado en `8b1c52cd`.
- **Gate Result:** 🔵 DO
- **Contrato:** archivo restaurado en `docs/Investigaciones/`; referencias del ACID_ROLLBACK_DESIGN válidas.
- **Estado:** ⬜ PENDING

### Task 6: Arreglar docs/reports/INDEX.md (4 rutas rotas + 1 estado falso)
- **Esfuerzo:** 🟡 | **Prioridad:** P0 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/reports/INDEX.md` L19 (ruta `docs/audit-reports/audit-full-20260808-002617.md` no existe — el archivo está en `archive/` pero marcado "vigente"), L23/28/29 (sin prefijo `archive/`)
- **Verificación real:** watchdog: 4 rutas rotas verificadas una a una; violación de la regla "cuando `progreso` mueve a `archive/`, INDEX registra con `archive/`".
- **Gate Result:** 🔵 DO
- **Contrato:** `docs/reports/INDEX.md` con rutas reales verificadas (`Test-Path` por cada fila); estado del audit 20260808 = "vigente (en archive)" o resuelto.
- **Estado:** ⬜ PENDING

### Task 7: Unificar `docs/audit-reports/` → `docs/reviews/` (decisión de arquitectura de docs)
- **Esfuerzo:** 🟡 | **Prioridad:** P1 | **Ruta:** vanta-docs
- **Archivos clave:**
  - `docs/audit-reports/` raíz VACÍA (los 14 están en `archive/`) — hub de escritura muerto
  - `docs/reviews/` es el hogar de la skill `unified-review` (`review-<mode>-<ts>.md`)
  - `docs/audit-reports/archive/` (14) vs `docs/reviews/archive/` (1)
- **Verificación real:** El mismo pipeline (`/audit` ≡ `/review full`, alias legacy según `unified-review/SKILL.md:77`) genera ambos tipos → mismo artefacto partido en 2 por modo. `docs/reviews/logs/` vacía (feature `keep_raw_logs` nunca activado).
- **Decisión previa requerida:** (A) unificar TODO en `reviews/` con stub de compat de `audit-reports/`, o (B) dejar separados por modo. **Recomendado: (A)** — la posición perezosa correcta.
- **Gate Result:** 🔵 DO
- **Contrato si (A):** rescatar `audit-full-20260808-002617.md` a `reviews/`; `git mv` de los 13 restantes a `reviews/archive/`; editar escritores `dev-tools/audit-all.ps1:20`, `.opencode/commands/audit.md:141`, `prompts/audit-full.md:194` y lectores `commands/status.md:24`, `progreso/SKILL.md` (Trigger 4), `check-avance-coverage.ps1:16`; `docs/audit-reports/` queda como dir vacío de compat o se elimina con actualización de paths.
- **Estado:** ⬜ PENDING
- **Notas:** Riesgo ALTO — toca 3 escritores + 2 lectores + coverage script. La solución lazy es redirigir escritura futura sin mover histórico, o mover todo coordinado. Requiere decisión del usuario en CLOSE.

### Task 8: Resolver `docs/archived-decisions/` (≈ inactiva)
- **Esfuerzo:** 🟢 | **Prioridad:** P1 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/archived-decisions/ADR-001-ADAPTER-TIERS.md`, `docs/archived-decisions/stabilization-report.md`
- **Verificación real:** ADR-001 (2026-07-22) duplica tema de `docs/architecture/adr/010_adapter_language_classification.md` y el nombre colisiona con `ADR-0001`; `stabilization-report.md` es un reporte, no una decisión. Solo leído por `TEST_MAP.md` (x2).
- **Gate Result:** 🔵 DO
- **Contrato:** ADR-001 → mover/renombrar a `docs/architecture/adr/` (nueva numeración) y actualizar `docs/operations/TEST_MAP.md:130` + `docs/TEST_MAP.md:130`; stabilization-report → `docs/reviews/`; carpeta vaciada y eliminada.
- **Estado:** ⬜ PENDING

### Task 9: Eliminar huérfanos del task-system (o marcarlos legacy)
- **Esfuerzo:** 🟢 | **Prioridad:** P1 | **Ruta:** vanta-lead
- **Archivos clave (candidatos):**
  - `.opencode/task-system/enforcement/pre-call-checks.md` (0 refs)
  - `.opencode/task-system/sandbox/sandbox_manager.py` + `sandbox/Dockerfile` (0 refs externas)
  - `.opencode/task-system/validation/agent_validator.py` (0 refs)
  - `.opencode/task-system/self-modification/dgm-*.ps1` (auto-refs only)
  - `.opencode/task-system/traces/trace-event.ps1` (0 refs)
  - `.opencode/task-system/memory/memory.ps1` (server escribe .md directo)
  - `traces/(generar al arrancar).jsonl` (placeholder de filename vivo en `tracer.mjs` — bug)
- **Verificación real:** rg de cada nombre en todo `.opencode/` + raíz = sin referencias externas (excepto los citados).
- **Gate Result:** 🔵 DO
- **Contrato:** cada eliminado verificado con `rg "<nombre>"` = 0; los que se conserven como manual/legacy quedan anotados en el manual; `(generar al arrancar).jsonl` eliminado (o fix del placeholder en `tracer.mjs`).
- **Estado:** ✅ COMPLETED (2026-08-10) — 8 huérfanos ELIMINADOS + placeholder jsonl eliminado + guardia en `extractCampaignId` (commits `cbcbc0c1`, `ab61de79`, `4daf7ee6`); `pre-call-checks.md` LEGACY anotado en manual.

### Task 10: Crear persona `vanta-review` (referencia rota del pipeline)
- **Esfuerzo:** 🟡 | **Prioridad:** P0 | **Ruta:** vanta-lead
- **Archivos clave:** `.opencode/task-system/prompts/task.md`, `.opencode/skills/campaign-executor/RULES.md` (referencian `vanta-review` como revisor de segunda opinión — P2-1 lo exige), `.opencode/agents/` (7 personas existen, vanta-review no)
- **Verificación real:** `rg "vanta-review"` = 2 refs en prompts/RULES; ninguna persona `.md` con ese nombre en `.opencode/agents/`.
- **Gate Result:** 🔵 DO
- **Contrato:** `.opencode/agents/vanta-review.md` creado (revisor de segunda opinión); refs en task.md/RULES.md válidas; `rg "vanta-review"` resuelve al archivo.
- **Estado:** ⬜ PENDING
- **Notas:** hoy P2-1 delega a doubt-driven-development; la persona formaliza el contrato.

### Task 11: Sincronizar SKILLS-MANIFEST.md (conteo 111 real vs AGENTS.md)
- **Esfuerzo:** 🟢 | **Prioridad:** P2 | **Ruta:** vanta-lead
- **Archivos clave:** `SKILLS-MANIFEST.md`, `.opencode/AGENTS.md` (dice 32 en `.opencode` pero hay 29), `debugging-and-error-recovery` DEPRECATED, `impeccable` global no sincronizado
- **Verificación real:** conteo 111 = 29 + 82 confirmado por 2 agentes independientes.
- **Gate Result:** 🔵 DO
- **Contrato:** AGENTS.md actualizado al conteo correcto; manifest marca skills deprecadas/superseded.
- **Estado:** ⬜ PENDING

### Task 12: Corrección de referencias rotas en AGENTS.md / manual / skills
- **Esfuerzo:** 🟡 | **Prioridad:** P1 | **Ruta:** vanta-docs
- **Archivos clave (refs rotas verificadas):**
  - `.opencode/AGENTS.md:783` → `docs/plans/2026-08-06-oc-vantadb-pro.md` (movido a `archive/`)
  - `.opencode/AGENTS.md:648` + `progreso/SKILL.md:33` → `docs/VantaDB-MPTS/` (no existe)
  - `cliff.toml:59` → `docs/snapshots/` (no existe)
  - `skills/documentation-and-adrs/SKILL.md:38` → `docs/decisions/` (el estándar real es `docs/architecture/adr/`) — divergencia de skill
  - `docs/progreso/bitacora.md:100,213,497,606,612,618,631,634` + `755-759` → refs muertas a `docs/research/*` inexistentes
  - `docs/archive/REPORTE_INVESTIGACION_Y_DECISIONES.md` → link muerto (bitacora L521/532, consolidacion L474)
  - `docs/operations/master-index.md:35` → dice "No archived files currently" pero hay 3 (STALE)
- **Gate Result:** 🔵 DO
- **Contrato:** cada ref rota corregida o marcada explícitamente (no tocar historial/snapshots); `rg "<ruta-rota>"` tras el fix = 0 fuera de histórico.
- **Estado:** ⬜ PENDING

### Task 13: Estado de p3-remaining-fallas.md desincronizado
- **Esfuerzo:** 🟢 | **Prioridad:** P2 | **Ruta:** vanta-lead
- **Archivos clave:** `docs/plans/2026-08-10-p3-remaining-fallas.md` — header `✅ COMPLETADO` con commits, pero 6 tasks `⬜ PENDING`
- **Verificación real:** leído; viola la regla de "live artifact".
- **Gate Result:** 🔵 DO
- **Contrato:** estados de las 6 tasks → `✅ COMPLETED` con evidencia (commits) o header corregido; consistencia verificado por `northstar.mjs`.
- **Estado:** ⬜ PENDING

### Task 14: Items §3.3 sin asignar del REPORTE-FINAL
- **Esfuerzo:** 🟡 | **Prioridad:** P2 (conduce a future backlog) | **Ruta:** vanta-docs
- **Archivos clave:** `docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md` §3.3/§3.5/gap-02
- **Items no asignados a plan P:**
  - Observabilidad de decisión (log de "por qué" cambió de estado)
  - Handoff con invariantes + comandos de verificación + deuda
  - ADR integrado mecánicamente (gate que falle si se toca API pública sin ADR)
  - Estimar con appetite (Shape Up)
  - SLA del pipeline (SLI/SLO/error budget) — requiere Task 2
  - Chaos/resilience del propio task-system (`vanta-chaos` fuzzea código, no `campaign-server.mjs`)
  - Recitation duplicado (3 definiciones, drift cosmético)
  - Triage "es ahora"/appetite
- **Verificación real:** leídos en el REPORTE; el handoff de los planes los deja fuera del lote gaps.
- **Gate Result:** 🔵 DO
- **Contrato:** cada item llevado al backlog con ID, prioridad y effort estimado (Family B o P4); los de runtime dependen de Task 2.
- **Estado:** ⬜ PENDING

### Task 15: Eliminar espejos redundantes de docs/avance ↔ docs/progreso
- **Esfuerzo:** 🟡 | **Prioridad:** P2 | **Ruta:** vanta-docs
- **Archivos clave (duplicados byte-a-byte confirmados):**
  - `docs/progreso/README.backup-2026-08-03.md` ≡ `docs/avance/historial/snapshot-2026-08-03.md` (3320 líneas, 0 diff)
  - `docs/avance/historial/backlog-history.md` ≡ `docs/progreso/BACKLOG_HISTORY.md` (SHA-256 idénticos)
  - `docs/avance/historial/archivo-historico.md` ≡ `docs/progreso/ARCHIVO_HISTORICO.md` (486 líneas, 0 diff)
  - `docs/avance/historial/sdk-gap-audit-2026-07-28.md` ≡ `docs/progreso/2026-07-28-sdk-gap-audit.md` (SHA-256 idénticos)
- **Verificación real:** hashes/Compare-Object por sub-agente. El mecanismo "mirror" de la skill es dual-write manual → drift silencioso (hoy COBERTURA muestra 48% solo en snapshot).
- **Gate Result:** 🔵 DO
- **Contrato:** los mirrors vigentes pasan a link (`> Ver: docs/progreso/<archivo>`) o a checker de hash en `check-avance-coverage.ps1`; eliminar `README.backup-2026-08-03.md` junto con actualización de `scripts/check-avance-coverage.ps1:24` y `docs/avance/COBERTURA.md:19`; snapshots congelados se conservan (historial).
- **Estado:** ⬜ PENDING
- **Notas:** `docs/Backlog.md:54,376` cita `snapshot-2026-08-07.md` como evidencia — NO borrar snapshots.

### Task 16: Mover backlog diferido R5 fuera de docs/archive
- **Esfuerzo:** 🟢 | **Prioridad:** P2 | **Ruta:** vanta-docs
- **Archivos clave:** `docs/backlog-futuro.md` (movido desde `docs/archive/`); `docs/strategy/ROADMAP.md:128` (R5 lo designa como destino)
- **Verificación real:** `backlog-futuro` es operativo vivo (ROADMAP R5 lo escribe), vive en carpeta de "archivo muerto" (doble rol de docs/archive).
- **Gate Result:** 🔵 DO
- **Contrato:** movido; ROADMAP L128 actualizado; `docs/archive/` queda solo con los 2 extractos histórico-estáticos.
- **Estado:** ⬜ PENDING

## Definición de Done (DoD) del plan

- Tasks 1-2 completadas (base de datos métricos y árbol limpio).
- Tasks 3-16: cada una con su contrato verificado; `rg` de rutas migradas = 0 refs vivas rotas.
- `docs/` sin carpetas duplicadas de propósito; 4 carpetas "archive" reducidas a 2 (plans/reviews); 0 carpetas huérfanas del sistema.
- `verify-log.jsonl` > 0 líneas; reportes regenerados con datos reales.
- Backlog actualizado con items §3.3 (Task 14) para campañas futuras.

## Riesgos

- **ALTO:** mover `investigacion-equipo-2026-08-09.md` rompe rastro de origen de ~20 items abiertos de Backlog (P16) si no se actualizan las 3 refs en el mismo commit.
- **ALTO:** Task 7 toca 3 escritores + 2 lectores del pipeline de audit — no hacer sin decisión (A)/(B) del usuario.
- **MEDIO:** eliminar espejos de avance sin actualizar `check-avance-coverage.ps1` + `COBERTURA.md` rompe cobertura (Regla 0: grep antes de borrar).
- **BAJO:** `docs/book/book/progreso/index.html` (mdBook) queda stale al regenerar README — regenerar libro o ignorar (artefacto de build).
- **Histórico (NO tocar):** `docs/avance/historial/*`, `snapshots`, `docs/progreso/README.backup-*`, `docs/plans/archive/*` — describen estados pasados.