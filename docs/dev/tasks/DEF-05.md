---
title: "DEF-05: North-star + success criteria (SPEC/VISION)"
kind: task
description: "SPEC/VISION declaran North Star + guardrails (0 hallazgo crítico seguridad, latencia) Y la métrica es medible con el proxy actual (comando/consulta documentada)\""
---

# DEF-05: North-star + success criteria (SPEC/VISION)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 13)
- **Fuente:** Backlog `DEF-05` (L940) · master Task 13 · decisión owner 2026-09-24 (SPEC Adenda #4)
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴
- **Tipo:** Docs (decisión + docs)
- **Turns estimados:** 7
- **Creado:** 2026-09-26T19:55
- **last-synced:** 2026-09-27
- **Estado:** ✅ COMPLETED (2026-09-27 — steps 1–4 ✅; review fresco P2-01 ✅; nit corregido; commit LEAD)
- **Incógnitas (uphill):** 0 abiertas — resuelta en Step 1 (2026-09-27): derivable parcial (PUT ✅ / SEARCH → FIND + DEFER parcial)
- **Pendientes (downhill):** 0 — steps 1–4 ✅ + review fresco P2-01 ✅ (commit delegado al LEAD)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ICP-01 (métrica instrumentada en demo CI) · F5/F6 (métricas de ICP y anuncio cuelgan de esto) · DEF-07 (criterio de clasificación = North Star) · `VISION.md` §Success Metrics |
| Callees | `vanta-proxy` (`/snapshot` turn reports/sessions, `[report]` OTLP spans; WIRE-01 ✅ loop de memoria, commit `679c75a9`) · `SPEC.md` §Success Criteria (campaña) · Backlog `DEF-05` |
| Implicaciones | Doc-only; VISION.md:265-269 ya declara la North Star (2026-09-24) → DEF-05 formaliza en SPEC + fija comando/consulta · guardrails tocan gates existentes (p99 >15% → FIND-154/HARD-06; artefactos sincronizados → HARD-01) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `SPEC.md` (123L, post-DEF-01) · `docs/dev/vision/VISION.md` (261L, post-DEF-01) · `docs/api/PROXY.md` (157L) · `vanta-proxy/src/capture.rs` (266L) · `vanta-proxy/src/memory_tools.rs` (476L) · `vanta-proxy/src/session.rs` (494L) · `vanta-proxy/src/report.rs` (245L) · `vanta-proxy/src/langfuse.rs` (291L) · `vanta-proxy/src/inject.rs` (652L) · `vanta-proxy/src/server.rs` (pipeline 240-499 · tool-loop 560-785 · router//snapshot 788-898) · `vanta-memory/src/core/hooks/auto_recall.rs` (150-309) · `skills/vantadb-mcp/references/recall-policy.md` (61L) · `docs/dev/avance/activo/operaciones.md` (sección WIRE-01)
- **Archivos referenciados hacia dentro (imports/dependencias):** VISION frontmatter `related: GO_TO_MARKET.md`; SPEC Adenda #4 apunta a DEF-05; PROXY.md documenta `/snapshot` y `[report]` (fuentes de medición)
- **Archivos que referencian a los editados (referencias entrantes):** `ICP-01` (Backlog L924, dep "WIRE-01 (métrica)") · master Task 13/14 (DEF-07 usa North Star como criterio)
- **Veredicto impacto:** **bajo** — se agrega/ajusta la sección de producto; riesgo = inconsistencia SPEC↔VISION si se editan sin alinear (Regla 3, mismo PR).

## Contrato
"SPEC/VISION declaran North Star + guardrails (0 hallazgo crítico seguridad, latencia) Y la métrica es medible con el proxy actual (comando/consulta documentada)"

## Spec (SDD — decisión de producto)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | North Star | A) "agentes activos que recuperan una memoria con éxito en ventana de 7 días" (sesiones MCP con put+search la misma semana) / B) otra métrica | A | ✅ decidido owner 2026-09-24 (`SPEC.md:115` Adenda #4; `VISION.md:267`) |
| 2 | Guardrails | A) set de VISION: 0 hallazgos high sin parche ≤7d · 0 regresión p99 >15% · 100% artefactos con versión sincronizada · 0 violaciones Regla 11 / B) subset | A | ✅ decidido-por-evidencia (`VISION.md:269`; confirmar con owner en Step 1 si hay duda) |
| 3 | Ubicación | A) `SPEC.md`: criterios de producto (North Star + guardrails), etiquetando los de campaña del MVP como tales · `VISION.md`: alinear + comando de medición · `ROADMAP.md` histórico = solo referencia (no editar) / B) doc nuevo | A | ✅ decidido-por-evidencia (Backlog `DEF-05` archivos; master Task 13; `SPEC.md:118`) |
| 4 | Medición | A) comando/consulta documentada contra el proxy actual (auth `/snapshot` + report spans); si no alcanza → declarar + FIND + DEFER parcial / B) declarar sin medición | A | ✅ verificado Step 1 (2026-09-27): derivable parcial — PUT ✅ con consulta real (`memory_list` sobre `proxy-turns`); SEARCH → FIND + DEFER parcial |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) North Star = **frase canónica única**, idéntica en SPEC y VISION (sin variantes); (2) guardrails no decorativos — cada uno con fuente/comando de medición; (3) baseline inicial documentado (hoy 0, con fecha); (4) **no instrumentar código** en esta task (si falta medición → FIND + DEFER parcial, no implementar acá).
- **Comandos de verificación:** `rg -n "North Star" SPEC.md docs/dev/vision/VISION.md` (frase idéntica) · `pwsh scripts/validate-docs-coverage.ps1` → exit 0.
- **Deuda pendiente:** si la medición no es derivable del proxy actual → FIND propuesto (el orquestador lo ingresa al Backlog) + DEFER parcial declarado.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# DEF-05: North-star + success criteria (SPEC/VISION)` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | LEAD: review fresco P2-01 + commit local `docs:` (steps 1–4 ✅; ACCEPT bloqueado por HARD-07 sin reviewer fresco) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia (comandos ejecutados) |
| `nextTask` | DEF-07 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (task doc-only; no introduce deuda nueva).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable ✅ (North Star + guardrails en SPEC/VISION + medición documentada o DEFER parcial + validate-docs-coverage verde) |
| **Commit** | 1 commit atómico `docs:` (SPEC + VISION + task file), `git diff` limpio, verificación mecánica registrada (nunca auto-reporte) |
| **Release** | N/A justificado (criterios de producto en docs; sin artefacto de release) — ver Notas |

## Herramientas necesarias
- `curl`/inspección de `vanta-proxy` (medición — Step 1) · `rg` sobre `vanta-proxy/src` (campos de telemetría)
- `pwsh scripts/validate-docs-coverage.ps1` · `npx markdownlint-cli2 SPEC.md docs/dev/vision/VISION.md`
- `pwsh dev-tools/ocr-review.ps1` (input del review P2-01)

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `source-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `doubt-driven-development` · `frontend-ui-engineering` · **PINNED (policy):** ninguno
- `campaign-executor` — flujo pipeline/task system (base).
- `source-driven-development` — verificar la medición contra el código real del proxy (no supuestos).
- `spec-driven-development` — tabla Spec.
- `interview-me` — confirmar guardrails/owner si hiciera falta (keyword mapping "spec").
- `documentation-and-adrs` — estándar de docs.
- `writing-guidelines` — prosa técnica (EN).

## Investigation Notes
- **VISION.md:265-269** ya declara la North Star (2026-09-24): "Agentes activos que recuperan una memoria con éxito en ventana de 7 días (medible en el proxy/MCP: sesiones con put+search la misma semana)" + guardrails (0 high sin parche ≤7d · 0 regresión p99 >15% · 100% artefactos con versión sincronizada · 0 violaciones Regla 11). DEF-05 = formalizar en SPEC + fijar medición.
- **SPEC.md:99-104** §Success Criteria actuales = de campaña MVP; **SPEC.md:118** (Adenda): "los success criteria de campaña siguen vigentes para el MVP; los de producto son la North Star de DEF-05" → etiquetar, no borrar.
- **Medición — estado real:** WIRE-01 ✅ (`679c75a9`, 2026-09-25) cerró el loop de memoria del proxy (dual-write L1 + cost + presupuesto de inyección). `PROXY.md`: `GET /snapshot` (auth `x-vanta-user-key`, sin bypass loopback) expone "recent turn reports, active sessions, write-back queue, rate-limit telemetry, cost snapshot"; `[report]` exporta spans por turno (OTLP-JSON, default off). Memoria en el wire: `memory_tools.rs` TOOL_CAPTURE/TOOL_SEARCH (put+search del proxy).
- **Step 1 — veredicto de medición (2026-09-27):** derivable **parcial**.
  - **PUT ✅:** todo turno proxied exitoso con sesión se auto-captura: `server.rs:256-258` → `capture_turn` (`server.rs:606-629`) → `capture.rs:53-139` escribe en el store del proxy (`server.rs:100`, `[auth] db_path` default `vantadb_data`): namespace `proxy-turns` (payload `{session, protocol, space, model, text}`, key `{ms}-{seq}`) + registro L1 en `l1/{session}` (`created_at`). **Consulta:** `memory_list` (MCP, read-only `handlers/tools.rs:200-218`) o `list` (SDK) sobre `proxy-turns`; filtro client-side `ms ≥ now−7d` sobre los keys (filters metadata-only — patrón página+filtra `recall-policy.md:38,52`) y contar `payload.session` distintos.
  - **SEARCH ❌:** sin telemetría persistida — `memory_tools::search` (`memory_tools.rs:113-138`) no registra; loop de tools solo `tracing::debug!` (`server.rs:750-754`); `perform_auto_recall` read-only (`auto_recall.rs:198-279`); `TurnReport` sin campos de tools (`report.rs:18-41`, ring 100); `/snapshot` vista viva en memoria (`session.rs:34-38,143-243`); span OTLP opt-in sin sesión/tools (`langfuse.rs:97-108`). Superficie de escritura auditada (`rg "\.put\(" vanta-proxy/src`): 3 hits de producción (`capture.rs:120,126` + `mem_command.rs:160`, opt-in off) + 1 hit de test (`inject.rs:604`) → el path de search no escribe nada.
  - **FIND propuesto** (lo registra el orquestador): instrumentación mínima del search — persistir un evento por search vía `WriteBack::track` en `memory_tools::search` (namespace `proxy-memory-events`, key `{ms}-{seq}`, payload `{session, kind:"search", hits}`); métrica final = sesiones en `proxy-turns` ∩ eventos `search` (`hits ≥ 1`) en la ventana de 7d. ~15L + test; sin tocar el wire.
  - **DEFER parcial** ratificado: no se instrumenta código en DEF-05 (stop condition del master Task 13).
- **Frontera honesta:** si los campos actuales de `/snapshot`/report no derivan "sesiones con put+search misma semana", NO inventar: documentar la definición + registrar FIND para la instrumentación mínima (DEFER parcial, stop condition del master Task 13).
- **Downstream:** ICP-01 (demo CI + métrica instrumentada), DEF-07 (criterio de clasificación), F6 (anuncio).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resuelta en Step 1 (2026-09-27): derivable parcial (PUT ✅ / SEARCH → FIND + DEFER parcial) |
| Pendientes de ejecución (downhill) | 0 — Steps 1–4 ✅ (commit delegado al LEAD) |
| % completado | 100% — review fresco P2-01 ✅; commit LEAD |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica a la edición (doc-only); el guardrail de hallazgos ≤7d es política, no instrumentación. Sin dependencias nuevas.
- [ ] **PERFORMANCE** — no aplica a la edición (sin código); el guardrail p99 >15% referencia el gate existente (FIND-154/HARD-06), no lo modifica.

## Steps

### Step 1: Verificar medición de la North Star en el proxy actual
- **Archivos:** `vanta-proxy/src/**` (campos de telemetría), `docs/api/PROXY.md`
- **Acción:** determinar el comando/consulta exacta que produce "sesiones con put+search la misma semana" con el proxy actual (inspeccionar `/snapshot` + report/telemetry; `rg` de campos). Si no es derivable → documentar definición + FIND propuesto + DEFER parcial.
- **Verify:** comando ejecutado contra proxy local (o inspección con refs file:line) y resultado documentado: query exacta **o** gap explícito con FIND.
- **Resultado:** inspección file:line completa (ver Investigation Notes §Step 1). **PUT derivable** con consulta real (`memory_list`/`list` sobre `proxy-turns`, filtro client-side de keys `{ms}-{seq}` por timestamp, contar `payload.session` distintos); **SEARCH no derivable** (ningún write en el path) → FIND propuesto + DEFER parcial. Incógnita #1 resuelta.
- **Estado:** ✅ COMPLETED (2026-09-27)

### Step 2: Draft North Star + guardrails + baseline
- **Archivos:** borrador en este task file
- **Acción:** redactar la frase canónica (idéntica para SPEC/VISION), tabla guardrail→fuente→comando de medición y baseline inicial (0, con fecha).
- **Verify:** ≥4 guardrails con fuente citada (`VISION.md:269`, Backlog L940); baseline registrado con fecha 2026-09-26.
- **Borrador — frase canónica (byte-idéntica en SPEC y VISION):**

  > **North Star: agentes activos que recuperan una memoria con éxito en ventana de 7 días** (proxy operacional: sesiones con put+search en la misma ventana de 7 días).

- **Borrador — guardrails (fuente → medición):**

  | Guardrail | Fuente | Medición |
  |---|---|---|
  | 0 hallazgos high sin parche ≤ 7 días | owner 2026-09-24 (`VISION.md` §North Star; Backlog `DEF-05` L940) | Revisión semanal de filas abiertas high/crítica (`docs/dev/Backlog.md`) + reportes `docs/dev/reviews/` — ninguna con antigüedad > 7 días |
  | 0 regresión p99 > 15% | gate revivido HARD-06/FIND-154 (`benchmarks/compare_baseline.py`, `STABLE_BLOCK_PCT=15`) | `python benchmarks/compare_baseline.py` (bloqueo >15% en familias estables; bandas por familia) · Rust: `cargo bench -p vantadb --bench canonical_p99` vs `docs/user/operations/BENCHMARKS.md` §8 |
  | 100% artefactos con versión sincronizada | rails HARD-01 (`docs/api/COMPATIBILITY.md`, `docs/api/VERSIONING.md`) + release-plz | Post-release: versiones publicadas (crates.io `vantadb` · npm `vantadb`/`vantadb-wasm` · PyPI `vantadb-py` · GitHub Release) == head de `docs/CHANGELOG.md` |
  | 0 violaciones Regla 11 en material público | `AGENTS.md` Regla 11 | Review pre-publicación: todo claim de performance cita bench reproducible + comando (`BENCHMARKS.md`) |

- **Borrador — baseline:** North Star = **0 sesiones** al **2026-09-26** (sin despliegue externo con el loop de memoria activo); el contador arranca con el primer deployment instrumentado (ICP-01 lo consume en demo CI). Guardrails al baseline: gate perf corregido tras HARD-06 (FIND-154 cerrado 2026-09-27), 4 artefactos 0.7.0 sincronizados (2026-09-25), sin violaciones Regla 11 conocidas/registradas.
- **Estado:** ✅ COMPLETED (2026-09-27)

### Step 3: Aplicar a SPEC + VISION
- **Archivos:** `SPEC.md` (criterios de producto; etiquetar los de campaña), `docs/dev/vision/VISION.md` (alinear + comando de medición); `ROADMAP.md` NO se edita (histórico)
- **Acción:** integrar North Star + guardrails; referenciar la medición documentada; mantener coherencia con la jerarquía de DEF-01.
- **Verify:** `rg -n "North Star" SPEC.md docs/dev/vision/VISION.md` → frase idéntica en ambos; `git diff --check` limpio.
- **Resultado:** `SPEC.md` §North Star (producto) nueva (frase canónica + medición PUT/SEARCH + FIND + guardrails) y §Success Criteria etiquetado "(campaña MVP)"; `VISION.md` §North Star alineado (misma frase + pointer a medición/baseline + `last_reviewed: 2026-09-27`). `ROADMAP.md` intacto.
- **Verify ejecutado:** frase byte-idéntica (comparación `-ceq` → True; `rg` → SPEC.md:108 = VISION.md:250) · `git diff --check -- SPEC.md docs/dev/vision/VISION.md docs/dev/tasks/DEF-05.md` → exit 0 (los warnings globales son de `master-roadmap`/`Backlog` ajenos a esta task) · `npx markdownlint-cli2 SPEC.md docs/dev/vision/VISION.md` → 0 issues, exit 0.
- **Estado:** ✅ COMPLETED (2026-09-27)

### Step 4: Gates + cierre
- **Archivos:** `docs/dev/tasks/DEF-05.md` (estado/recitation)
- **Acción:** correr gates, verificar links, registrar evidencia y commit local `docs:`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 · markdownlint sin errores · OCR sin Critical/High.
- **Resultado:** gates mecánicos verdes vía `campaign_verify_cmd` (coverage exit 0/0 gaps · markdownlint exit 0 · frase canónica) · links citados existen (`COMPATIBILITY.md`, `VERSIONING.md`, `BENCHMARKS.md`, `docs/dev/reviews/`) · OCR delegation corrido (advisory) · **commit NO ejecutado — delegado al LEAD** (instrucción wave F1b: el LEAD commitea; PROHIBIDO push).
- **Estado:** ✅ DONE (2026-09-27) — gates verdes + review fresco P2-01 ✅; commit delegado al LEAD

## Dependencias
- **DEF-01** (jerarquía de producto — coherencia "agentes"/núcleo; DEF-05 cierra después o en coordinación; el master declara que DEF-01 bloquea la coherencia de DEF-05).
- **WIRE-01** ✅ (base de la métrica — loop de memoria del proxy, `679c75a9`).
- Downstream: ICP-01 (métrica instrumentada), DEF-07 (criterio), F6 (anuncio).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` (leaf, no implementa) — fallback: `doubt-driven-development` en contexto fresco.
- **Enfoque:** ¿la métrica es **medible de verdad** con el proxy actual (comando verificado, no supuesto)? ¿Guardrails con fuente (no decorativos)? ¿North Star idéntica en SPEC y VISION? ¿Consistente con DEF-01?
- **Cómo se probó:** comando de medición ejecutado/inspeccionado y registrado (no auto-reporte) + diff solo-docs + OCR delegation sin Critical/High.
- **Ejecución P2-01 (2026-09-27):** ⚠️ **DEGRADADO** — esta sesión (subagente leaf de la wave F1b) no dispone de tool de sub-agentes (no se pudo spawnear `vanta-review` fresco); se aplicó `doubt-driven-development` degradado (auto-cuestionamiento con separador, sin contexto fresco real): detectó y corrigió 1 imprecisión (claim "perf gate verde" → "gate corregido tras HARD-06, FIND-154 cerrado") y verificó frase byte-idéntica, links y refs file:line. **Escalado al LEAD:** review fresco P2-01 requerido antes del ACCEPT (HARD-07 bloquea degraded sin waiver).
- **Review fresco P2-01 (LEAD, 2026-09-27):** `vanta-review` sesión `ses_f1b85c5b3ffeJ5C1pR3auyEpTR` (≠ autor) → **✅ APPROVE**. Evidencia: `validate-docs-coverage` exit 0 (0 gaps) · markdownlint 0 issues · frase canónica byte-idéntica (`-ceq`=True; SPEC.md:108 ≡ VISION.md:250) · ~15 refs file:line de `vanta-proxy/src` spot-checkeadas (PUT derivable / SEARCH no-derivable resistió red-team) · consistencia DEF-01 ✅ · checklist anti-hábitos cumplido. Nit detectado: conteo `rg "\.put\("` = 4 hits (3 producción + 1 test `inject.rs:604`) → **corregido** en SPEC.md:123 + §Step 1. Undetermined explícitos: baseline 0 (declaración fechada), "0 violaciones Regla 11" (proceso).
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ **APPROVE** — review fresco `vanta-review` (sesión `ses_f1b85c5b3ffeJ5C1pR3auyEpTR`, 2026-09-27; nit corregido). ACCEPT habilitado (payload review fresh para HARD-07).

## Notas
- **No aplica Fase 1 — Evidencia de Debugging** (tipo Docs, no Bug).
- **Scope discipline:** no instrumentar código (si falta medición → FIND + DEFER parcial); no editar `docs/dev/strategy/ROADMAP.md` (histórico); no duplicar métricas de marketing.
- **Release N/A:** criterios de producto en docs; el cambio no participa del contrato de release.
- Creado por `vanta-docs` (subagente) desde el master roadmap — estado inicial ⬜ PENDING.
