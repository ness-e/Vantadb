---
title: "MGR-13: Cuarentena semántica + abstención (research-doc, cero implementación)"
kind: task
description: "research-doc cerrado con estados + transiciones (entrada/promoción/expiración con dueño y trigger) + threat model write-time por superficie (API/dream/import), listo para SCH-01\""
---

# MGR-13: Cuarentena semántica + abstención (research-doc, cero implementación)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 25 (F3) · **Origen:** `docs/dev/Backlog.md:847` · **Decisión owner 2026-09-14:** migración única 0.8.0 con MGR-10/12 (`Backlog:934`)
- **Fuente del prompt:** sub-agente vanta-arch (orquestador pipeline) — ejecución directa de MGR-13
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** research/design documental
- **Turns estimados:** 10-15 · **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ IN PROGRESS (contenido ✅; verify + cierre LEAD pendientes)
- **Incógnitas (uphill):** 1 (criterios de cuarentena sin falsos positivos → resuelto con señales explícitas + I6) · **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Alcance | `docs/dev/research/mgr-13-cuarentena.md` (nuevo), `docs/dev/tasks/MGR-13.md` (nuevo) — docs only |
| Callees | Solo lectura: `vanta-memory/src/core/dream/mod.rs`, `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/core/abstractions/types.rs`, `src/server/middleware.rs`, `src/server/handlers.rs`, `src/sdk/types/record.rs`, `src/sdk/serialization/{mod,impl_export,vector_types}.rs`, `src/sdk/search/page.rs`, `src/sdk/version_history.rs`, `src/wal.rs`, `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/dreams.rs` |
| Implicaciones | Cero cambios de código productivo. Diseño consumido por SCH-01→SCH-02→SCH-05→SCH-06 (F3). Riesgo de diseño: criterios que aíslen contenido válido (mitigado: señales explícitas, I6) y estados sin salida (mitigado: I4) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `vanta-memory/src/core/dream/mod.rs` (invariantes :1-36, tipos :151-212, promote :595-664), `vanta-memory/src/core/abstractions/types.rs` (record L1 :65-130), `vanta-memory/src/core/hooks/auto_recall.rs` (contrato :140-260, scoping :286-315), `src/server/middleware.rs` (auth :39-258), `src/server/handlers.rs` (:240-265, :769), `src/sdk/types/record.rs` (:40-169), `src/sdk/serialization/impl_export.rs` (:280-364), `src/sdk/serialization/mod.rs` (:32-64, :519-549), `src/wal.rs` (:611-642), `vantadb-mcp/src/dreams.rs` (:80-267), `vantadb-mcp/src/handlers/tools.rs` (:22-43, :578-609, :1814-1847)
- **Referencias hacia dentro:** plan Task 25 (L651-675) y Task 30 SCH-05 (L781-805); Backlog :813/:847/:934/:942; convención `docs/dev/research/mgr-19-benchmarks-baseline-suites.md`
- **Referencias entrantes:** SCH-01 (ADR consolidación), SCH-02 (campos), SCH-05 (cuarentena operativa + abstención + test de contención), SCH-06 (bordes); EXE-07 (test cascada multiagente, post-1.0)
- **Veredicto impacto:** bajo — docs-only; el impacto real es de diseño (fija el contrato que SCH-01/05 implementan). `src/` intacto por diseño (no commit de código)

## Contrato
"research-doc cerrado con estados + transiciones (entrada/promoción/expiración con dueño y trigger) + threat model write-time por superficie (API/dream/import), listo para SCH-01"

## Spec (SDD — Phase 1b)
Cero implementación, cero símbolos nuevos. El "spec" ES el research-doc: §3 (máquina de estados + transiciones + invariantes), §4 (threat model por superficie), §6 (preguntas owner), §7 (plan de implementación por tarea). Cierre MGR = research-doc + preguntas owner + plan de implementación (Backlog:813/:934). No es feature-add.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** sin motor de políticas ABAC/namespaces trusted en el corte (stop condition → FIND); hash-chain/PROV-O no se implementa ni duplica (VER-01, F4); no tocar `src/` ni otros archivos del co-batch F3 (MGR-10/12); toda entrada de cuarentena tiene salida (I4); nunca auto-promoción (I1); sticky (I2); default-exclude no altera datos legacy (I6/§2.4).
- **Comandos de verificación:** `pwsh scripts/validate-docs-coverage.ps1` (coverage docs) — evidencia del contrato en §3/§4 del research-doc.
- **Deuda pendiente al abrir:** ninguna

## Recitation
```
=== RECITATION ===
Objetivo activo: MGR-13 — research-doc: cuarentena (estados + transiciones + threat model write-time)
Estado: in-progress (desde: ⏳ EN PROGRESO del plan)
Última acción: DISCOVERY + redacción + verify completos — gap verificado, 12 archivos leídos, 7 fuentes verificadas (webfetch), research-doc §0-§8 (238L) + task file (120L); `validate-docs-coverage` EXIT=0/0 gaps; scope MCP valid=true
Resultado: PARTIAL
Próxima acción: LEAD — review P2-01 (tier Fast) + commit docs-only + cierre MGR (este sub-agente NO commitea ni se auto-revisa por instrucción)
Contrato: research-doc con estados+transiciones (dueño/trigger) + threat model write-time por superficie (API/dream/import) → §3/§4
Invariantes: sin ABAC; hash-chain = VER-01; co-batch MGR-12 intacto; src/ intacto
Deuda: ninguna nueva (docs-only; saldo neto Regla 6 = 0)
Próxima tarea si completa: SCH-01 (tras cierre de MGR-10/12/13)
last-synced: 2026-09-28
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Sin deuda (research/design documental, cero código).

## Definition of Done (3 niveles)
- **Task:** contrato + Cierre MGR (research-doc + preguntas owner §6 + plan de implementación §7) ✅
- **Commit:** `docs:` + MGR-13 (solo `docs/dev/research/mgr-13-cuarentena.md` + `docs/dev/tasks/MGR-13.md`) — **lo ejecuta el LEAD** (sub-agente sin commit por instrucción; review P2-01 delegado)
- **Release:** N/A (research sin cambio funcional)

## Herramientas necesarias
- CodeGraph/codebase-memory (lectura de superficies), `rg`/Read (verificación de gap), webfetch/websearch (fuentes externas), `pwsh scripts/validate-docs-coverage.ps1`
- **Skills cargadas (SDP):** base `campaign-executor`+`progreso` (auto) · pin `deprecation-and-migration` (storage/schema) · `writing-guidelines` + `writing-plans` (docs) · `documentation-and-adrs` · `security-and-hardening` (threat model) · `source-driven-development` · `api-and-interface-design` · `spec-driven-development` · `coordinated-web-search` (mandato AGENTS.md para research web)

## Investigation Notes
- Gap verificado: `quarantine|quarantined` en `src/` = solo WAL salvage (`src/wal.rs:614-638`); `tainted` = 0 de contenido; sin campos de estado en `MemoryRecord` SDK (`record.rs:102-138`); `abstain|abstention` = 0; inyección sin filtro (`auto_recall.rs:198`, `tools.rs:1818-1841`, `inject_context` :585-599).
- Superficies write-time: API (`middleware.rs:42-258` + `handlers.rs:248-265`), dream (promote stub `dream/mod.rs:614-622`), import (`impl_export.rs:290-364` + `mod.rs:522-531`), MCP writes (37 tools no read-only, `tools.rs:30`).
- Hallazgo de integración: `dream_promote` MCP es readOnlyHint:true **porque es stub** — al conectar MEM-65 debe pasar a false (checkpoint de contrato, research-doc §4.2).
- Hallazgo de dos capas: storage record (`src/sdk/...`) vs L1 payload record (`vanta-memory/...`) → estado canónico en storage record + mirror en migración (research-doc §2.3).
- Fuentes externas verificadas (webfetch 2026-09-28): AgentPoison arXiv 2407.12784 · MINJA arXiv 2503.03704 · OWASP Top 10 Agentic 2026 · OWASP ASI06 blog · Cisco MemoryTrap · Azure Quarantine pattern · W3C PROV-O.
- Destraba: SCH-01/02/05/06 (F3). MGR-04 fuera del plan → FIND (clase mínima vía MGR-12).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 — criterios resueltos con señales explícitas + invariante I6 (sin heurísticas que aíslen válidos) |
| Pendientes | 4 steps (ejecutados por este sub-agente; cierre formal LEAD) |
| % completado | 95% (contenido ✅; verify mecánico + review/commit LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — el threat model write-time ES el entregable (clases AgentPoison/MINJA/ASI06 aplicadas a las 3 superficies obligatorias + MCP). Sin código nuevo; no aplica `cargo audit`.
- [x] **PERFORMANCE** — N/A: docs-only, cero hot paths tocados. Los gates de retrieval (default-exclude + fingerprint) ya siguen el patrón de `exclude_superseded` (mismo orden de costo).

## Steps
### Step 1: DISCOVERY (gap + superficies + fuentes)
- **Archivos:** superficies §Investigation Notes + refs externas
- **Acción:** verificar gap en código real (codegraph + rg + lecturas), mapear superficies write-time, verificar 7 fuentes externas vía webfetch (GATE CITAS)
- **Verify:** cada claim con file:line; cada URL resuelve (200)
- **Estado:** ✅ DONE (2026-09-28; 12 archivos leídos; 7/7 URLs verificadas)

### Step 2: Research-doc — estados + transiciones
- **Archivos:** `docs/dev/research/mgr-13-cuarentena.md`
- **Acción:** §2 modelo de datos (campos v2 + precedentes + dos capas), §3 estados + tabla de transiciones (dueño/trigger/efecto/registro) + invariantes I1-I6 + señales 0.8.0 vs diferidas
- **Verify:** tabla con **dueño y trigger** por transición; entrada/promoción/expiración cubiertas
- **Estado:** ✅ DONE (§3 completa; promoción T2 revisable, expiración T3 sin auto-promoción, entrada T1/T1b/T1c/T1d)

### Step 3: Threat model write-time por superficie
- **Archivos:** research-doc §4
- **Acción:** modelo de adversario (3 clases con refs verificadas) + análisis por superficie (API, dream, import, MCP) con controles hoy/gaps/gates/residuales
- **Verify:** las 3 superficies obligatorias (API/dream/import) cubiertas + MCP; cada control citado con file:line; cada clase con URL
- **Estado:** ✅ DONE (§4; incluye checkpoints de integración: dream_promote hint, dos capas)

### Step 4: Cierre MGR (preguntas + plan impl + verify)
- **Archivos:** research-doc §6/§7, task file, plan file (recitation vía MCP)
- **Acción:** §6 preguntas owner (6, con recomendación), §7 plan de implementación por tarea (SCH-01/02/05/06), verify `validate-docs-coverage`, recitation MCP, handoff a LEAD (review + commit)
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` + estado MGR-13 in-progress registrado vía campaign
- **Estado:** ✅ DONE (preguntas §6 + plan §7 + verify `pwsh scripts/validate-docs-coverage.ps1` → **EXIT=0, 0 gaps** (2026-09-28, shell directo) + recitation registrada vía campaign MCP; review P2-01 y commit **delegados al LEAD** por instrucción del orquestador)

## Dependencias
- **Consume:** MGR-12 (co-batch, clase asserted/derived — NO tocado), contexto verificado del bloque Task 25. MGR-04 fuera del plan → FIND.
- **Destraba:** SCH-01 → SCH-02 → SCH-05 → SCH-06. SIN MGR-13 no hay criterios de entrada/salida para `quarantined`.

## Review (GATE P2-01)
- **Revisor:** `ses_f16a0c98affeOYCwFKbcWeDzoi` (fresco — distinto de `ses_f16afedbeffe6STWViWblh75d2`) — **✅ APPROVE** (2026-09-28)
- **Enfoque:** estado binario + T1/T1b/T1c/T1d/T2/T3/T4 con dueño+trigger; I1/I2 anti-bypass; threat model por superficie real (API/dream/import/MCP); stop conditions 0.8.0 respetadas (sin ABAC, hash-chain→VER-01)
- **Cómo se probó:** validate-docs-coverage re-ejecutado EXIT=0/0 gaps; ~30 refs file:line spot-checkeadas; 5/7 claims §1 y 6/7 fuentes re-verificadas por fetch; `git diff -- src/` vacío; tier Fast
- **Veredicto:** ✅ approve — 1 Optional (`heat` en §2.3) + 3 Nits (refs/recitation) aplicados en el commit del LEAD.

## Notas
- **No commit / no push / no self-review** (instrucción del orquestador; cierre en LEAD).
- Co-batch F3: NO tocar `docs/dev/research/mgr-10-*.md` ni `mgr-12-*.md` ni sus task files.
- `src/` debe quedar intacto: verificación `git diff --name-only` (esperado: vacío fuera de docs).
- **Verify en este entorno:** `campaign_verify_cmd` no spawnea `pwsh` (exit -1; `powershell.exe` 5.x mis-parsea el script UTF-8 — ver stderr). El verify canónico corre directo: `pwsh -NoProfile -File scripts/validate-docs-coverage.ps1` → EXIT=0.
